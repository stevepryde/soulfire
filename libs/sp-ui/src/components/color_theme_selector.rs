use dioxus::prelude::*;

#[component]
pub fn ColorThemeSelector<T: Clone + PartialEq + 'static>(
    themes: Vec<T>,
    current_theme: T,
    on_theme_change: EventHandler<T>,
    get_display_name: Callback<T, String>,
    get_preview_color: Callback<T, String>,
    #[props(default = String::new())] class: String,
    #[props(default = String::new())] label: String,
) -> Element {
    rsx! {
        div {
            class: "space-y-3 {class}",

            if !label.is_empty() {
                label {
                    class: "block text-sm font-semibold text-primary-light mb-2 dark:text-primary-light",
                    {label}
                }
            }

            div {
                class: "flex flex-wrap gap-3",

                for theme in themes {
                    {
                        let is_selected = theme == current_theme;
                        let display_name = get_display_name.call(theme.clone());
                        let preview_color = get_preview_color.call(theme.clone());
                        let theme_clone = theme.clone();
                        let current_theme_clone = current_theme.clone();

                        rsx! {
                            button {
                                class: format!(
                                    "group relative flex flex-col items-center space-y-2 p-3 rounded-lg border-2 transition-all duration-200 hover:scale-105 cursor-pointer {}",
                                    if is_selected {
                                        "border-primary bg-primary-lighter dark:bg-primary-lighter"
                                    } else {
                                        "border-border bg-surface hover:border-primary hover:bg-primary-lighter dark:border-border dark:bg-surface dark:hover:border-primary dark:hover:bg-primary-lighter"
                                    }
                                ),
                                onclick: move |_| {
                                    if theme_clone != current_theme_clone {
                                        on_theme_change.call(theme_clone.clone());
                                    }
                                },

                                // Color preview circle
                                div {
                                    class: format!(
                                        "w-8 h-8 rounded-full border-2 {}",
                                        if is_selected {
                                            "border-white shadow-lg ring-2 ring-primary"
                                        } else {
                                            "border-gray-300 dark:border-gray-600"
                                        }
                                    ),
                                    style: "background-color: {preview_color};"
                                }

                                // Theme name
                                span {
                                    class: format!(
                                        "text-xs font-medium {}",
                                        if is_selected {
                                            "text-primary-dark dark:text-primary-dark"
                                        } else {
                                            "text-secondary-text dark:text-secondary-text group-hover:text-primary dark:group-hover:text-primary"
                                        }
                                    ),
                                    {display_name}
                                }

                                // Selection indicator
                                if is_selected {
                                    div {
                                        class: "absolute -top-1 -right-1 w-5 h-5 bg-primary rounded-full flex items-center justify-center",
                                        svg {
                                            class: "w-3 h-3 text-primary-text",
                                            fill: "currentColor",
                                            view_box: "0 0 20 20",
                                            path {
                                                fill_rule: "evenodd",
                                                d: "M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z",
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
    }
}
