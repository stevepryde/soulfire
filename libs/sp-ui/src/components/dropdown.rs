use dioxus::prelude::*;

use crate::components::new_id;

use super::input::FieldSize;

#[derive(Clone)]
struct DropDownContext<T: Clone + PartialEq + 'static> {
    is_open: Signal<bool>,
    onselect: EventHandler<T>,
}

#[component]
pub fn DropDown<T: Clone + PartialEq + 'static>(
    #[props(default = FieldSize::Medium)] size: FieldSize,
    onselect: EventHandler<T>,
    #[props(default)] label_class: String,
    #[props(default)] label: String,
    #[props(default)] disabled: bool,
    #[props(default = "bg-surface dark:bg-surface".to_string())] background_class: String,
    #[props(default = "border-border dark:border-border".to_string())] border_class: String,
    #[props(default = "px-4 py-3".to_string())] padding_class: String,
    #[props(default)] field_class: String,
    #[props(default)] menu_class: String,
    value: String,
    children: Element,
) -> Element {
    let mut is_open = use_signal(|| false);

    use_context_provider(|| DropDownContext { is_open, onselect });

    let field_id = use_signal(new_id);

    rsx! {
        div {
            class: format!("relative text-left {}", size.class()),

            DropDownField {
                id: field_id,
                is_open,
                label,
                label_class,
                disabled,
                background_class: background_class.clone(),
                border_class: border_class.clone(),
                padding_class,
                field_class,
                value
            }
            if is_open() {
                // Full-screen backdrop captures outside clicks to close the menu.
                // This is the portable equivalent of the upstream web build's
                // `use_outside_click_multi` DOM listener (which needed web-sys).
                div {
                    class: "fixed inset-0 z-30",
                    onclick: move |_| is_open.set(false),
                }
                DropDownMenu {
                    background_class,
                    border_class,
                    menu_class,
                    children
                }
            }
        }
    }
}

#[component]
pub fn DropDownField(
    id: Signal<String>,
    is_open: Signal<bool>,
    label: String,
    label_class: String,
    disabled: bool,
    background_class: String,
    border_class: String,
    padding_class: String,
    field_class: String,
    value: String,
) -> Element {
    rsx! {
        div {
            class: "w-full",

            if !label.is_empty() {
                label {
                    class: "block text-primary-light text-sm font-semibold mb-2 text-left dark:text-primary-light {label_class}",
                    r#for: id,
                    {label}
                }
            }
            button {
                class: "inline-flex w-full min-h-12 gap-x-1.5 rounded-[0.35rem] border {border_class} {padding_class} text-base leading-6 font-medium text-primary-text shadow-sm hover:bg-primary-lighter hover:border-primary transition-all duration-300 focus:outline-none focus:ring-2 focus:ring-primary focus:border-primary dark:hover:bg-primary-lighter dark:hover:border-primary dark:focus:ring-primary dark:focus:border-primary {background_class} {field_class}",
                id,
                aria_expanded: "true",
                aria_haspopup: "true",
                disabled,
                onclick: move |_| {
                    is_open.set(!is_open());
                },

                div {
                    class: "flex justify-between items-center w-full",

                    div {
                        class: "flex-1 whitespace-nowrap overflow-hidden text-ellipsis",
                        {value}
                    }

                    div {
                        class: "flex items-center justify-end",

                        svg {
                            class: "-mr-1 h-5 w-5 opacity-60",
                            view_box: "0 0 20 20",
                            fill: "currentColor",
                            "aria_hidden": "true",
                            path {
                                fill_rule: "evenodd",
                                d: "M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z",
                                clip_rule: "evenodd",
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn DropDownMenu(
    background_class: String,
    border_class: String,
    menu_class: String,
    children: Element,
) -> Element {
    let id = new_id();
    rsx! {
        div {
            id,
            // Sits above the z-30 backdrop rendered by `DropDown` so menu items
            // remain clickable while a click anywhere else closes the menu.
            class: "absolute right-0 z-40 mt-2 w-full min-w-[14rem] origin-top-right rounded-lg border text-primary-text shadow-lg focus:outline-none {background_class} {menu_class}",
            role: "menu",
            aria_orientation: "vertical",
            aria_labelledby: "menu-button",
            tabindex: "-1",
            div {
                class: "py-1",
                role: "none",
                {children}
            }
        }
    }
}

#[component]
pub fn DropDownItem<T: Clone + PartialEq + 'static>(label: String, value: T) -> Element {
    let mut context = try_use_context::<DropDownContext<T>>()
        .unwrap_or_else(|| panic!("DropDownItem must nest below DropDown component"));

    rsx! {
        div {
            class: "block px-4 py-2 text-sm text-inherit hover:bg-primary-lighter hover:text-primary cursor-pointer transition-colors duration-200 dark:hover:bg-primary-lighter dark:hover:text-primary",
            tabindex: "-1",
            role: "menuitem",
            onclick: move |_| {
                context.is_open.set(false);
                context.onselect.call(value.clone());
            },
            {label}
        }
    }
}
