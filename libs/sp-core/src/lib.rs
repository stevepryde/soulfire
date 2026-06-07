//! Shared utility helpers.
//!
//! Vendored subset of the owner's `sp-std` crate — only the helpers actually
//! used by Soulfire are copied here (not the `full` surface). The `base64` and
//! `mongo` modules are intentionally dropped (mongo was a server concern; base64
//! is unused). See the vendoring decision in `specs/12-platform-packaging.md`.

pub mod datetime;
pub mod secret;
pub mod spid;

// Re-export paste so the `id_type!` macro can resolve `$crate::paste`.
pub use paste;
