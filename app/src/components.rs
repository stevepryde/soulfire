//! Reusable UI components: the confirmation dialog and info modal (`UI-7`).

use dioxus::prelude::*;

/// A confirmation dialog that gates destructive actions (`UI-7`). When
/// `confirm_word` is set, the confirm button is disabled until the user types
/// that word (for high-risk deletes, `AC-UI-g`).
#[component]
pub fn ConfirmDialog(
    open: bool,
    title: String,
    message: String,
    #[props(default)] danger: bool,
    #[props(default = "Confirm".to_string())] confirm_label: String,
    #[props(default)] confirm_word: Option<String>,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut typed = use_signal(String::new);
    if !open {
        return rsx! {};
    }
    let ready = match &confirm_word {
        Some(word) => &typed() == word,
        None => true,
    };
    let confirm_cls = if danger {
        "bg-red-600 hover:bg-red-700 text-white"
    } else {
        "crm-primary-button"
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60",
            onclick: move |_| on_cancel.call(()),
            div {
                class: "w-full max-w-sm bg-surface border border-border rounded-2xl shadow-2xl p-6",
                onclick: move |e| e.stop_propagation(),
                h2 { class: "text-lg font-semibold text-primary-text mb-2", "{title}" }
                p { class: "text-sm text-secondary-text mb-4", "{message}" }
                if let Some(word) = confirm_word.clone() {
                    p { class: "text-xs text-secondary-text mb-1", "Type \"{word}\" to confirm." }
                    input {
                        class: "input-premium w-full mb-4",
                        value: "{typed}",
                        oninput: move |e| typed.set(e.value()),
                    }
                }
                div { class: "flex gap-2 justify-end",
                    button {
                        class: "px-4 py-2 rounded-lg border border-border text-secondary-text",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "px-4 py-2 rounded-lg disabled:opacity-40 {confirm_cls}",
                        disabled: !ready,
                        onclick: move |_| on_confirm.call(()),
                        "{confirm_label}"
                    }
                }
            }
        }
    }
}
