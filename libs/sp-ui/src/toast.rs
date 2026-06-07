#![allow(dead_code)]

use std::sync::atomic::AtomicU32;

use dioxus::prelude::*;

static TOAST_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum ToastLevel {
    #[default]
    Info,
    Warning,
    Error,
}

pub const DEFAULT_TOAST_MS: i64 = 3000;

#[derive(Debug, Clone, PartialEq)]
pub struct ToastMessage {
    pub id: u32,
    pub level: ToastLevel,
    pub message: String,
    pub remaining_ms: i64,
}

impl ToastMessage {
    fn new(level: ToastLevel, message: String) -> Self {
        Self {
            id: TOAST_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            level,
            message,
            remaining_ms: DEFAULT_TOAST_MS,
        }
    }

    pub fn info(message: String) -> Self {
        Self::new(ToastLevel::Info, message)
    }

    pub fn warn(message: String) -> Self {
        Self::new(ToastLevel::Warning, message)
    }

    pub fn error(message: String) -> Self {
        Self::new(ToastLevel::Error, message)
    }
}

impl Default for ToastMessage {
    fn default() -> Self {
        Self::info(String::new())
    }
}

static TOASTS: GlobalSignal<Vec<ToastMessage>> = Signal::global(Vec::new);

#[derive(Debug, Clone, Copy, Default)]
pub struct ToastService;

impl ToastService {
    pub fn get_toasts() -> Vec<ToastMessage> {
        TOASTS.read().clone()
    }

    pub fn send(toast: ToastMessage) {
        TOASTS.write().push(toast);
    }

    pub fn remove(id: u32) {
        TOASTS.write().retain(|t| t.id != id);
    }

    pub fn error(message: &str) {
        Self::send(ToastMessage::error(message.to_string()));
    }

    pub fn warn(message: &str) {
        Self::send(ToastMessage::warn(message.to_string()));
    }

    pub fn info(message: &str) {
        Self::send(ToastMessage::info(message.to_string()));
    }
}
