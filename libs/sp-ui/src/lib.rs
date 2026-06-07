//! Shared Dioxus UI components.
//!
//! Vendored from the owner's `sp-ui` crate and **adapted from web to native**:
//! web-only paths (wasm-bindgen/web-sys/gloo/js-sys) are stripped and only the
//! components Soulfire uses are kept (toasts, context menu, dropdown, text
//! input, query client, debounce, color-theme selector). See the vendoring
//! decision in `specs/12-platform-packaging.md`; UI contract in `specs/09-ui.md`.
//!
//! Scaffold stub — components are vendored/adapted as a deliberate slice
//! (the heaviest port, done after the non-UI libs).
