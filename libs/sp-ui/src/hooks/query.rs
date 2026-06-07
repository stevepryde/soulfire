use std::fmt::Debug;

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    future::Future,
    hash::Hash,
    pin::Pin,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use dioxus::prelude::*;
use tokio::sync::Notify;

type ErrorHandlerFuture = Pin<Box<dyn Future<Output = ()> + 'static>>;
type ErrorResultFuture = Pin<Box<dyn Future<Output = anyhow::Result<()>> + 'static>>;
type CacheEntryRef<T> = Rc<RefCell<CacheEntry<T>>>;
type CacheMap<T> = Rc<RefCell<HashMap<String, CacheEntryRef<T>>>>;
type CacheInvalidator = Box<dyn Fn()>;
type KeyToCaches = Rc<RefCell<HashMap<String, Vec<CacheInvalidator>>>>;

/// Configuration for future query retry behaviour.
#[derive(Debug, Clone, Copy)]
pub enum RetryStrategy {
    /// Retry with exponentially increasing delays.
    ExponentialBackoff {
        /// Delay used for the first retry attempt.
        initial_delay: Duration,
        /// Maximum delay between retries.
        max_delay: Duration,
        /// Multiplier applied after each retry.
        multiplier: f64,
    },
    /// Retry after a constant delay each time.
    FixedDelay {
        /// Delay between retry attempts.
        delay: Duration,
    },
    /// Retry with linearly increasing delays.
    LinearBackoff {
        /// Delay used for the first retry attempt.
        initial_delay: Duration,
        /// Amount added to the delay after each retry.
        increment: Duration,
        /// Maximum delay between retries.
        max_delay: Duration,
    },
}

/// Adapter used to forward query failures into app-specific error handling.
#[derive(Clone)]
pub struct QueryErrorHandler {
    inner: Arc<dyn Fn(anyhow::Error) -> ErrorHandlerFuture>,
}

impl std::fmt::Debug for QueryErrorHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("QueryErrorHandler(..)")
    }
}

impl QueryErrorHandler {
    /// Creates a handler from an async callback that consumes the query error.
    pub fn new<F, Fut>(handler: F) -> Self
    where
        F: Fn(anyhow::Error) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        Self {
            inner: Arc::new(move |error| Box::pin(handler(error))),
        }
    }

    /// Creates a handler from an adapter that already wraps `anyhow::Result` futures.
    ///
    /// This is designed for app helpers like `error_handler().with(...)`, allowing code such as:
    /// `QueryErrorHandler::from_result_handler(|fut| error_handler().with(fut))`.
    pub fn from_result_handler<F, Fut>(handler: F) -> Self
    where
        F: Fn(ErrorResultFuture) -> Fut + 'static,
        Fut: Future<Output = anyhow::Result<()>> + 'static,
    {
        Self::new(move |error| {
            let handled = handler(Box::pin(async move { Err::<(), _>(error) }));
            async move {
                let _ = handled.await;
            }
        })
    }

    async fn handle(&self, error: anyhow::Error) {
        (self.inner)(error).await;
    }
}

#[derive(Clone, Debug)]
struct CacheEntry<T> {
    value: T,
    is_stale: bool,
    expires_at: Instant,
    purge_at: Instant,
    notify: Arc<Notify>,
}

impl<T> CacheEntry<T> {
    fn new(value: T, options: &QueryOptions) -> Self {
        Self {
            value,
            is_stale: false,
            expires_at: Instant::now() + options.stale_time.unwrap_or(Duration::from_secs(60)),
            purge_at: Instant::now() + options.cache_time.unwrap_or(Duration::from_secs(300)),
            notify: Arc::new(Notify::new()),
        }
    }

    fn is_stale(&self) -> bool {
        self.is_stale || Instant::now() >= self.expires_at || Instant::now() >= self.purge_at
    }

    fn should_purge(&self) -> bool {
        Instant::now() >= self.purge_at
    }

    fn invalidate(&mut self) {
        self.is_stale = true;
        self.notify();
    }

    fn notify(&self) {
        self.notify.notify_waiters();
    }
}

fn key_to_hierarchical_string<K: Debug>(key: &K) -> String {
    let debug_str = format!("{key:?}");
    let trimmed = debug_str
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'));
    let content = trimmed.unwrap_or(debug_str.as_str());

    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0u32;
    let mut in_string = false;
    let mut escaped = false;

    for ch in content.chars() {
        if in_string {
            current.push(ch);
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                current.push(ch);
            }
            '(' | '[' | '{' => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => {
                let part = current.trim().trim_matches('"');
                if !part.is_empty() {
                    parts.push(part.to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    let part = current.trim().trim_matches('"');
    if !part.is_empty() {
        parts.push(part.to_string());
    }

    parts.join("/")
}

#[derive(Clone, Default)]
struct QueryClientInner {
    last_cleanup: Rc<RefCell<Option<Instant>>>,
    caches: Rc<RefCell<HashMap<TypeId, Box<dyn Any>>>>,
    key_to_caches: KeyToCaches,
    inflight: Rc<RefCell<HashMap<String, Arc<Notify>>>>,
    clear_counter: Rc<RefCell<u64>>,
    logging_enabled: bool,
}

impl QueryClientInner {
    fn new(logging_enabled: bool) -> Self {
        Self {
            last_cleanup: Rc::new(RefCell::new(None)),
            caches: Rc::new(RefCell::new(HashMap::new())),
            key_to_caches: Rc::new(RefCell::new(HashMap::new())),
            inflight: Rc::new(RefCell::new(HashMap::new())),
            clear_counter: Rc::new(RefCell::new(0)),
            logging_enabled,
        }
    }

    fn logging_enabled(&self) -> bool {
        self.logging_enabled
    }

    fn should_cleanup(&self) -> bool {
        let now = Instant::now();
        let mut last_cleanup = self.last_cleanup.borrow_mut();
        if let Some(last) = *last_cleanup
            && now.duration_since(last).as_secs() < 60
        {
            return false;
        }
        *last_cleanup = Some(now);
        true
    }

    fn get_counter(&self) -> u64 {
        *self.clear_counter.borrow()
    }

    fn get_cache_entry_ref<K, T>(&self, key: K) -> Option<Rc<RefCell<CacheEntry<T>>>>
    where
        K: Debug + Hash + Eq + 'static,
        T: Debug + Clone + 'static,
    {
        self.caches
            .borrow()
            .get(&TypeId::of::<(K, T)>())
            .and_then(|cache| cache.downcast_ref::<CacheMap<T>>().cloned())
            .and_then(|cache| {
                let key_str = key_to_hierarchical_string(&key);
                if self.should_cleanup() {
                    cache
                        .borrow_mut()
                        .retain(|_, entry| !entry.borrow().should_purge());
                }
                cache.borrow().get(&key_str).cloned()
            })
    }

    fn get_cached_entry<K, T>(&self, key: K) -> Option<CacheEntry<T>>
    where
        K: Debug + Hash + Eq + 'static,
        T: Debug + Clone + 'static,
    {
        self.get_cache_entry_ref::<K, T>(key)
            .filter(|entry| !entry.borrow().is_stale())
            .map(|entry| entry.borrow().clone())
    }

    fn set_cache_entry<K, T>(&self, key: K, value: CacheEntry<T>, counter: u64)
    where
        K: Debug + Clone + Hash + Eq + 'static,
        T: Clone + 'static,
    {
        if counter != *self.clear_counter.borrow() {
            return;
        }

        let mut caches = self.caches.borrow_mut();
        let entry = caches
            .entry(TypeId::of::<(K, T)>())
            .or_insert(Box::new(Rc::new(RefCell::new(HashMap::<
                String,
                CacheEntryRef<T>,
            >::new()))));

        if let Some(cache_map) = entry.downcast_mut::<CacheMap<T>>().cloned() {
            let value = Rc::new(RefCell::new(value));
            let key_str = key_to_hierarchical_string(&key);
            cache_map
                .borrow_mut()
                .insert(key_str.clone(), value.clone());

            let invalidator = move || {
                value.borrow_mut().invalidate();
            };
            self.key_to_caches
                .borrow_mut()
                .entry(key_str)
                .or_default()
                .push(Box::new(invalidator));
        }
    }

    fn set_in_flight<K>(&self, key: K, is_in_flight: bool) -> Option<Arc<Notify>>
    where
        K: Debug + Hash + Eq + 'static,
    {
        let key_str = key_to_hierarchical_string(&key);
        let mut inflight = self.inflight.borrow_mut();

        if is_in_flight {
            if let Some(notifier) = inflight.get(&key_str).cloned() {
                return Some(notifier);
            }
            let notifier = Arc::new(Notify::new());
            inflight.insert(key_str, notifier);
            None
        } else {
            inflight.remove(&key_str)
        }
    }

    fn get_notifier_for_key<K, T>(&self, key: K) -> Option<Arc<Notify>>
    where
        K: Debug + Clone + Hash + Eq + 'static,
        T: Debug + Clone + 'static,
    {
        self.get_cache_entry_ref::<K, T>(key)
            .map(|cache| cache.borrow().notify.clone())
    }

    fn invalidate<K>(&self, key: &K)
    where
        K: Debug + Clone + Hash + Eq + 'static,
    {
        let key_str = key_to_hierarchical_string(key);
        if let Some(caches) = self.key_to_caches.borrow().get(&key_str) {
            for cache in caches {
                cache();
            }
        }

        let mut inflight = self.inflight.borrow_mut();
        if let Some(notifier) = inflight.remove(&key_str) {
            notifier.notify_waiters();
        }
    }

    fn invalidate_prefix<K>(&self, key: &K)
    where
        K: Debug + Clone + Hash + Eq + 'static,
    {
        let key_str = key_to_hierarchical_string(key);
        for (cache_key, caches) in self.key_to_caches.borrow().iter() {
            if cache_key.starts_with(&key_str) {
                for cache in caches {
                    cache();
                }
            }
        }

        let mut inflight = self.inflight.borrow_mut();
        inflight.retain(|cache_key, notifier| {
            if cache_key.starts_with(&key_str) {
                notifier.notify_waiters();
                false
            } else {
                true
            }
        });
    }

    fn invalidate_all(&self) {
        for caches in self.key_to_caches.borrow().values() {
            for cache in caches {
                cache();
            }
        }

        self.clear_all();
    }

    fn clear_all(&self) {
        self.caches.borrow_mut().clear();
        self.key_to_caches.borrow_mut().clear();

        let mut inflight = self.inflight.borrow_mut();
        for notifier in inflight.values() {
            notifier.notify_waiters();
        }
        inflight.clear();
        *self.clear_counter.borrow_mut() += 1;
    }

    fn dump_cache(&self) {
        if !self.logging_enabled() {
            return;
        }

        let mut dump = String::new();
        dump.push_str("=== Query Cache Dump ===\n");
        dump.push_str(&format!(
            "Clear counter: {}\n",
            *self.clear_counter.borrow()
        ));

        let key_to_caches = self.key_to_caches.borrow();
        dump.push_str(&format!("Total cache keys: {}\n", key_to_caches.len()));
        for (key, caches) in key_to_caches.iter() {
            dump.push_str(&format!("  Key: {key} (invalidators: {})\n", caches.len()));
        }

        let inflight = self.inflight.borrow();
        dump.push_str(&format!("In-flight requests: {}\n", inflight.len()));
        for key in inflight.keys() {
            dump.push_str(&format!("  In-flight: {key}\n"));
        }

        dump.push_str("=== End Query Cache Dump ===");
        tracing::debug!("{dump}");
    }
}

/// Shared cache client used by the query hooks.
#[derive(Clone, Copy)]
pub struct QueryClient {
    inner: CopyValue<QueryClientInner>,
}

impl QueryClient {
    /// Creates a new empty query client.
    pub fn new() -> Self {
        Self::new_with_logging(false)
    }

    fn new_with_logging(logging_enabled: bool) -> Self {
        Self {
            inner: CopyValue::new(QueryClientInner::new(logging_enabled)),
        }
    }

    fn get_counter(&self) -> u64 {
        if let Ok(inner) = self.inner.try_read_unchecked() {
            return inner.get_counter();
        }
        0
    }

    fn get_cached_entry<K, T>(&self, key: K) -> Option<CacheEntry<T>>
    where
        K: Debug + Hash + Eq + 'static,
        T: Debug + Clone + 'static,
    {
        if let Ok(inner) = self.inner.try_read_unchecked() {
            return inner.get_cached_entry::<K, T>(key);
        }
        None
    }

    fn set_cache_entry<K, T>(&self, key: K, value: CacheEntry<T>, counter: u64)
    where
        K: Debug + Clone + Hash + Eq + 'static,
        T: Clone + 'static,
    {
        if let Ok(inner) = self.inner.try_read_unchecked() {
            inner.set_cache_entry::<K, T>(key, value, counter);
        }
    }

    fn set_in_flight<K>(&self, key: K, is_in_flight: bool) -> Option<Arc<Notify>>
    where
        K: Debug + Hash + Eq + 'static,
    {
        if let Ok(inner) = self.inner.try_read_unchecked() {
            return inner.set_in_flight::<K>(key, is_in_flight);
        }
        None
    }

    fn get_notifier_for_key<K, T>(&self, key: K) -> Option<Arc<Notify>>
    where
        K: Debug + Clone + Hash + Eq + 'static,
        T: Debug + Clone + 'static,
    {
        if let Ok(inner) = self.inner.try_read_unchecked() {
            return inner.get_notifier_for_key::<K, T>(key);
        }
        None
    }

    fn logging_enabled(&self) -> bool {
        if let Ok(inner) = self.inner.try_read_unchecked() {
            return inner.logging_enabled();
        }
        false
    }

    /// Invalidates the exact cache entry for `key`.
    pub fn invalidate<K>(&self, key: &K)
    where
        K: Debug + Clone + Hash + Eq + 'static,
    {
        if let Ok(inner) = self.inner.try_read_unchecked() {
            inner.invalidate(key);
        }
    }

    /// Invalidates every cache entry whose hierarchical key starts with `key`.
    pub fn invalidate_prefix<K>(&self, key: &K)
    where
        K: Debug + Clone + Hash + Eq + 'static,
    {
        if let Ok(inner) = self.inner.try_read_unchecked() {
            inner.invalidate_prefix(key);
        }
    }

    /// Invalidates all cached queries and wakes any waiters.
    pub fn invalidate_all(&self) {
        if let Ok(inner) = self.inner.try_read_unchecked() {
            inner.invalidate_all();
        }
    }

    /// Clears all cached queries without marking them stale first.
    pub fn clear(&self) {
        if let Ok(inner) = self.inner.try_read_unchecked() {
            inner.clear_all();
        }
    }

    /// Writes a debug dump of the current cache state to the tracing subscriber.
    pub fn dump_cache(&self) {
        if let Ok(inner) = self.inner.try_read_unchecked() {
            inner.dump_cache();
        }
    }
}

impl Default for QueryClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
struct QueryContext {
    client: QueryClient,
    options: QueryClientProviderOptions,
}

/// Options applied to all queries from a provided [`QueryClient`].
#[non_exhaustive]
#[derive(Debug, Clone, Default, bon::Builder)]
pub struct QueryClientProviderOptions {
    /// Default adapter for app-specific query error handling side effects.
    pub error_handler: Option<QueryErrorHandler>,
    /// Enables internal query-cache logging in debug builds only.
    #[builder(default)]
    pub enable_logging: bool,
}

/// Provides a new [`QueryClient`] in the current Dioxus context.
pub fn provide_query_client() {
    provide_query_client_with_options(QueryClientProviderOptions::default());
}

/// Provides a new [`QueryClient`] with default query behaviour in the current Dioxus context.
pub fn provide_query_client_with_options(options: QueryClientProviderOptions) {
    let client = QueryClient::new_with_logging(query_logging_enabled_for_options(&options));
    provide_context(QueryContext { client, options });
    provide_context(client);
}

/// Provides a new [`QueryClient`] with a default app-specific error handler.
pub fn provide_query_client_with_error_handler(error_handler: QueryErrorHandler) {
    provide_query_client_with_options(
        QueryClientProviderOptions::builder()
            .error_handler(error_handler)
            .build(),
    );
}

/// Returns the [`QueryClient`] from the current Dioxus context.
pub fn use_query_client() -> QueryClient {
    try_use_context::<QueryContext>()
        .map(|context| context.client)
        .or_else(try_use_context::<QueryClient>)
        .expect("QueryClient must be provided in context")
}

fn use_query_provider_options() -> QueryClientProviderOptions {
    try_use_context::<QueryContext>()
        .map(|context| context.options)
        .unwrap_or_default()
}

fn query_logging_enabled_for_options(options: &QueryClientProviderOptions) -> bool {
    cfg!(debug_assertions) && options.enable_logging
}

/// Options controlling cache lifetime and query execution.
#[non_exhaustive]
#[derive(Debug, Clone, Default, bon::Builder)]
pub struct QueryOptions {
    /// After this duration, the entry is purged from the cache.
    pub cache_time: Option<Duration>,
    /// After this duration, the entry is considered stale and is fetched again.
    pub stale_time: Option<Duration>,
    /// Reserved for future retry support and currently ignored by `sp-ui`.
    pub retry_strategy: Option<RetryStrategy>,
    /// Optional adapter for app-specific query error handling side effects.
    pub error_handler: Option<QueryErrorHandler>,
    /// When `false`, the query does not fetch and reports `is_loading = false`.
    #[builder(default = true)]
    pub enabled: bool,
}

/// The current state of a query.
#[derive(Clone)]
pub struct QueryResult<T> {
    /// The most recently resolved query value, if any.
    pub data: Option<T>,
    /// The latest query error string, if the most recent fetch failed.
    pub error: Option<String>,
    /// Whether a fetch is currently in progress for this query.
    pub is_loading: bool,
}

impl<T> Default for QueryResult<T> {
    fn default() -> Self {
        Self {
            data: None,
            error: None,
            is_loading: true,
        }
    }
}

/// Handle returned by [`use_query`] and [`use_query_with_options`].
pub struct UseQuery<T: Clone + 'static> {
    /// Reactive query state.
    pub result: Signal<QueryResult<T>>,
    /// Imperative refetch function.
    pub refetch: CopyValue<Box<dyn Fn()>>,
}

impl<T: Clone + 'static> UseQuery<T> {
    /// Returns the current query data.
    pub fn data(&self) -> Option<T> {
        (self.result)().data
    }

    /// Returns whether the query is currently loading.
    pub fn is_loading(&self) -> bool {
        (self.result)().is_loading
    }

    /// Returns the latest query error string, if any.
    pub fn error(&self) -> Option<String> {
        (self.result)().error.clone()
    }
}

impl<T: Clone + 'static> Clone for UseQuery<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Clone + 'static> Copy for UseQuery<T> {}

impl<T: Clone + 'static> PartialEq for UseQuery<T> {
    fn eq(&self, other: &Self) -> bool {
        self.result == other.result && self.refetch == other.refetch
    }
}

/// Fetches and caches async data using the provided key and options.
pub fn use_query_with_options<FK, K, T, Fut, F>(
    key: FK,
    fetcher: F,
    options: QueryOptions,
) -> UseQuery<T>
where
    FK: Fn() -> K + Clone + 'static,
    K: 'static + Hash + Eq + Clone + Debug,
    T: 'static + Debug + Clone,
    Fut: 'static + Future<Output = anyhow::Result<T>>,
    F: 'static + Fn(K) -> Fut + Clone,
{
    let client = use_query_client();
    let provider_options = use_query_provider_options();
    let logging_enabled = client.logging_enabled();
    let result = use_signal(QueryResult::<T>::default);
    let enabled = options.enabled;

    let key_fn = key.clone();
    let refetch = move || {
        if !enabled {
            return;
        }

        let key = key();
        let fetcher = fetcher.clone();
        let options = options.clone();
        let error_handler = options
            .error_handler
            .clone()
            .or_else(|| provider_options.error_handler.clone());
        let mut result = result;
        if logging_enabled {
            tracing::debug!("Refetch triggered for key: {key:?}");
        }

        spawn(async move {
            result.write().is_loading = true;
            let counter = client.get_counter();
            let key_str = format!("{key:?}");

            loop {
                if let Some(cached) = client.get_cached_entry::<K, T>(key.clone()) {
                    if logging_enabled {
                        tracing::debug!("(cache HIT) Fetching data for key: {key_str:?}");
                    }
                    result.set(QueryResult {
                        data: Some(cached.value),
                        error: None,
                        is_loading: false,
                    });
                    return;
                }

                if let Some(notifier) = client.set_in_flight::<K>(key.clone(), true) {
                    if logging_enabled {
                        tracing::debug!("(in-flight) Waiting for key: {key_str:?}");
                    }
                    notifier.notified().await;
                    if logging_enabled {
                        tracing::debug!("(in-flight) Notified for key: {key_str:?}");
                    }
                    continue;
                }

                break;
            }

            if logging_enabled {
                tracing::debug!("(cache MISS) Fetching data for key: {key_str:?}");
            }
            let key_for_release = key.clone();
            match fetcher(key.clone()).await {
                Ok(value) => {
                    client.set_cache_entry(
                        key.clone(),
                        CacheEntry::new(value.clone(), &options),
                        counter,
                    );
                    result.set(QueryResult {
                        data: Some(value),
                        error: None,
                        is_loading: false,
                    });
                }
                Err(error) => {
                    let existing_data = result.peek().data.clone();
                    let error_message = error.to_string();
                    if let Some(handler) = error_handler {
                        handler.handle(error).await;
                    } else if logging_enabled {
                        tracing::error!("query fetch failed for key {key_str:?}: {error:#}");
                    }
                    result.set(QueryResult {
                        data: existing_data,
                        error: Some(error_message),
                        is_loading: false,
                    });
                }
            }

            if let Some(notifier) = client.set_in_flight::<K>(key_for_release, false) {
                notifier.notify_waiters();
            }
        });
    };

    {
        let key = key_fn();
        let refetch = refetch.clone();
        use_effect(move || {
            if !enabled {
                return;
            }

            let key = key.clone();
            let refetch = refetch.clone();
            spawn(async move {
                loop {
                    if let Some(notifier) = client.get_notifier_for_key::<K, T>(key.clone()) {
                        notifier.notified().await;
                        refetch();
                    } else {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            });
        });
    }

    {
        let refetch = refetch.clone();
        let mut result = result;
        use_effect(move || {
            if enabled {
                refetch();
            } else {
                let existing_data = result.peek().data.clone();
                result.set(QueryResult {
                    data: existing_data,
                    error: None,
                    is_loading: false,
                });
            }
        });
    }

    UseQuery {
        result,
        refetch: CopyValue::new(Box::new(refetch)),
    }
}

/// Fetches and caches async data using default [`QueryOptions`].
pub fn use_query<FK, K, T, Fut, F>(key: FK, fetcher: F) -> UseQuery<T>
where
    FK: Fn() -> K + Clone + 'static,
    K: 'static + Hash + Eq + Clone + Debug,
    T: 'static + Debug + Clone,
    Fut: 'static + Future<Output = anyhow::Result<T>>,
    F: 'static + Fn(K) -> Fut + Clone,
{
    use_query_with_options(key, fetcher, QueryOptions::default())
}

#[cfg(test)]
mod tests {
    use super::key_to_hierarchical_string;

    #[test]
    fn hierarchical_key_single_string() {
        assert_eq!(key_to_hierarchical_string(&"recipes"), "recipes");
    }

    #[test]
    fn hierarchical_key_tuple_of_strings() {
        assert_eq!(
            key_to_hierarchical_string(&("recipes", "all")),
            "recipes/all"
        );
    }

    #[test]
    fn hierarchical_key_preserves_nested_values() {
        assert_eq!(
            key_to_hierarchical_string(&("recipes", ("breakfast", 3), "draft")),
            "recipes/(\"breakfast\", 3)/draft"
        );
    }

    #[test]
    fn hierarchical_key_preserves_commas_inside_strings() {
        assert_eq!(
            key_to_hierarchical_string(&("recipes,archived", "all")),
            "recipes,archived/all"
        );
    }
}
