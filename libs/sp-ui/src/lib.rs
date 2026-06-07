//! Shared Dioxus UI components.
//!
//! Vendored from the owner's `sp-ui` crate and **adapted from web to native**:
//! the web-only paths (`wasm-bindgen`/`web-sys`/`gloo`/`gloo-net`/`wasmtimer`)
//! are stripped and only the components Soulfire uses are kept (toasts, context
//! menu, dropdown, text input, query client, debounce, color-theme selector).
//! See the vendoring decision in `specs/12-platform-packaging.md`; UI contract
//! in `specs/09-ui.md`.
//!
//! Notable adaptations from the upstream web build:
//! - The `http` module (a `gloo-net` server client with bearer-token refresh) is
//!   dropped: the rebuild is a local-first single app with no backend (`PROD-12`).
//! - The `hooks::events` DOM-listener hooks (`wasm-bindgen`/`web-sys`) are dropped;
//!   the dropdown now closes via a full-screen backdrop element, the same portable
//!   pattern the context menu already used — no global DOM listeners required.
//! - `gloo`/`wasmtimer` timers are replaced with `tokio::time` (the native
//!   desktop/mobile renderers run on Tokio).

pub mod components;
pub mod hooks;
pub mod toast;

pub mod prelude {
    pub use crate::components::button::{Button, ButtonType};
    pub use crate::components::color_theme_selector::ColorThemeSelector;
    pub use crate::components::context_menu::{ContextMenu, ContextMenuItemText};
    pub use crate::components::dropdown::{DropDown, DropDownItem};
    pub use crate::components::input::{FieldSize, FieldType, TextArea, TextInput};
    pub use crate::components::new_id;
    pub use crate::components::theme_toggle::{ThemeSelector, ToggleButton, ToggleSize};
    pub use crate::components::toastcontainer::{ToastContainer, ToastPosition};
    pub use crate::hooks::debounce::{use_debounce, use_debounce_memo};
    pub use crate::hooks::query::{
        QueryClient, QueryClientProviderOptions, QueryErrorHandler, QueryOptions, QueryResult,
        RetryStrategy, UseQuery, provide_query_client, provide_query_client_with_error_handler,
        provide_query_client_with_options, use_query, use_query_client, use_query_with_options,
    };
    pub use crate::toast::{ToastLevel, ToastMessage, ToastService};
}
