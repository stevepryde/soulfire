#![allow(dead_code)]

use std::time::Duration;

use dioxus::prelude::*;
use tokio::time::sleep;

use crate::toast::{ToastLevel, ToastMessage, ToastService};

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ToastPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    #[default]
    BottomRight,
}

#[component]
pub fn ToastContainer(
    #[props(default = ToastPosition::BottomRight)] position: ToastPosition,
) -> Element {
    let toasts = ToastService::get_toasts();

    let position_class = match position {
        ToastPosition::TopLeft => "fixed top-4 left-4",
        ToastPosition::TopRight => "fixed top-4 right-4",
        ToastPosition::BottomLeft => "fixed bottom-4 left-4",
        ToastPosition::BottomRight => "fixed bottom-4 right-4",
    };

    let stack_direction = match position {
        ToastPosition::TopLeft | ToastPosition::TopRight => "flex-col",
        ToastPosition::BottomLeft | ToastPosition::BottomRight => "flex-col-reverse",
    };

    rsx! {
        div {
            class: "{position_class} z-50 flex {stack_direction} gap-2 pointer-events-none",
            style: "min-width: 24rem; max-width: 28rem;",
            for (index, toast) in toasts.iter().take(5).enumerate() {
                Toast {
                    key: "{toast.id}",
                    toast: toast.clone(),
                    index,
                }
            }
        }
    }
}

#[component]
pub fn Toast(toast: ToastMessage, index: usize) -> Element {
    let mut is_visible = use_signal(|| false);
    let mut is_removing = use_signal(|| false);
    let remaining_ms = use_hook(|| toast.remaining_ms);

    use_effect(move || {
        spawn(async move {
            sleep(Duration::from_millis(10)).await;
            is_visible.set(true);
        });
    });

    let remove_toast = move || {
        spawn(async move {
            is_removing.set(true);
            sleep(Duration::from_millis(300)).await;
            ToastService::remove(toast.id);
        });
    };

    spawn(async move {
        sleep(Duration::from_millis(remaining_ms as u64)).await;
        remove_toast();
    });

    let (toast_class, icon) = match &toast.level {
        ToastLevel::Info => (
            "crm-toast crm-toast-info",
            rsx! {
                svg {
                    class: "crm-toast-icon w-8 h-8",
                    fill: "currentColor",
                    view_box: "0 0 20 20",
                    path {
                        fill_rule: "evenodd",
                        d: "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z",
                        clip_rule: "evenodd"
                    }
                }
            },
        ),
        ToastLevel::Warning => (
            "crm-toast crm-toast-warning",
            rsx! {
                svg {
                    class: "crm-toast-icon w-8 h-8",
                    fill: "currentColor",
                    view_box: "0 0 20 20",
                    path {
                        fill_rule: "evenodd",
                        d: "M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z",
                        clip_rule: "evenodd"
                    }
                }
            },
        ),
        ToastLevel::Error => (
            "crm-toast crm-toast-error",
            rsx! {
                svg {
                    class: "crm-toast-icon w-8 h-8",
                    fill: "currentColor",
                    view_box: "0 0 20 20",
                    path {
                        fill_rule: "evenodd",
                        d: "M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z",
                        clip_rule: "evenodd"
                    }
                }
            },
        ),
    };

    let transform = if is_visible() {
        if is_removing() {
            "translate-x-full opacity-0"
        } else {
            "translate-x-0 opacity-100"
        }
    } else {
        "translate-x-full opacity-0"
    };

    rsx! {
        div {
            class: "pointer-events-auto w-full transition-all duration-300 ease-in-out transform {transform}",
            style: "animation-delay: {index * 50}ms;",
            div {
                class: "{toast_class}",
                div {
                    class: "p-4",
                    div {
                        class: "flex items-start",
                        div {
                            class: "flex-shrink-0",
                            {icon}
                        }
                        div {
                            class: "ml-3 w-0 flex-1",
                            p {
                                class: "crm-toast-message text-sm font-medium",
                                {toast.message.clone()}
                            }
                        }
                        div {
                            class: "ml-4 flex-shrink-0 flex",
                            button {
                                class: "crm-toast-close inline-flex rounded-[0.35rem] focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2",
                                onclick: move |_| remove_toast(),
                                svg {
                                    class: "h-5 w-5",
                                    fill: "currentColor",
                                    view_box: "0 0 20 20",
                                    path {
                                        fill_rule: "evenodd",
                                        d: "M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z",
                                        clip_rule: "evenodd"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
