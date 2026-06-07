use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

/// Returns a debounced signal that updates only after `value` stops changing for `delay`.
///
/// This is useful for text inputs and other rapidly changing state that should not trigger
/// downstream work, such as filtering or requests, on every intermediate update.
pub fn use_debounce<T>(value: Signal<T>, delay: Duration) -> Signal<T>
where
    T: Clone + PartialEq + 'static,
{
    let initial_value = value();
    let mut debounced_value = use_signal(move || initial_value.clone());

    use_effect(move || {
        let current_value = value();
        spawn(async move {
            sleep(delay).await;
            if value() == current_value {
                debounced_value.set(current_value);
            }
        });
    });

    debounced_value
}

/// Returns a debounced memo that updates only after `value` stops changing for `delay`.
///
/// This variant is convenient when consumers prefer a `Memo<T>` interface over a `Signal<T>`.
pub fn use_debounce_memo<T>(value: Signal<T>, delay: Duration) -> Memo<T>
where
    T: Clone + PartialEq + 'static,
{
    let initial_value = value();
    let mut debounced_value = use_memo(move || initial_value.clone());

    use_effect(move || {
        let current_value = value();
        spawn(async move {
            sleep(delay).await;
            if value() == current_value {
                debounced_value.set(current_value);
            }
        });
    });

    debounced_value
}
