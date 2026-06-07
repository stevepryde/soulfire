//! Soulfire — single-user, local, BYOK interactive-fiction & AI-chat app.
//!
//! The Dioxus desktop/mobile UI shell (`specs/09-ui.md`). The pure-Rust engine
//! lives in `soulfire-core`; this crate renders it and drives user interaction.

#![allow(non_snake_case)]

mod app;
mod components;
mod data;
mod image_util;
mod nav;
mod screens;
mod state;
mod theme;

fn main() {
    dioxus::launch(app::App);
}
