use dioxus::prelude::*;

#[derive(Clone)]
struct CloseSignal(Signal<bool>);

#[component]
pub fn ContextMenu(
    show: Signal<bool>,
    #[props(default = String::new())] class: String,
    children: Element,
) -> Element {
    provide_context(CloseSignal(show));

    rsx! {
        if show() {
            Fragment {
                div {
                    class: "fixed inset-0 z-30",
                    onclick: move |_| show.set(false),
                }
                div {
                    class: "absolute right-0 top-full z-40 w-48 rounded-lg border border-border bg-surface shadow-lg dark:border-border dark:bg-surface {class}",
                    onclick: move |event| event.stop_propagation(),
                    ul {
                        class: "py-1",
                        {children}
                    }
                }
            }
        }
    }
}

#[component]
pub fn ContextMenuItemText(text: String, onclick: Callback<MouseEvent>) -> Element {
    let mut show = use_context::<CloseSignal>().0;

    rsx! {
        li {
            class: "block px-4 py-2 text-sm text-primary-text hover:bg-primary-lighter hover:text-primary cursor-pointer transition-colors duration-200 first:rounded-t-lg last:rounded-b-lg dark:text-primary-text dark:hover:bg-primary-lighter dark:hover:text-primary",
            onclick: move |e| {
                show.set(false);
                onclick.call(e)
            },

            {text}
        }
    }
}
