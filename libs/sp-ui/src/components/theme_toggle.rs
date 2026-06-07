use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToggleSize {
    Small,
    Medium,
    Large,
}

impl ToggleSize {
    pub fn switch_classes(&self) -> &'static str {
        match self {
            ToggleSize::Small => "w-8 h-5",
            ToggleSize::Medium => "w-11 h-6",
            ToggleSize::Large => "w-14 h-8",
        }
    }

    pub fn toggle_classes(&self) -> &'static str {
        match self {
            ToggleSize::Small => "w-3 h-3",
            ToggleSize::Medium => "w-4 h-4",
            ToggleSize::Large => "w-6 h-6",
        }
    }

    pub fn translate_classes(&self, is_checked: bool) -> &'static str {
        if is_checked {
            match self {
                ToggleSize::Small => "translate-x-3",
                ToggleSize::Medium => "translate-x-5",
                ToggleSize::Large => "translate-x-6",
            }
        } else {
            match self {
                ToggleSize::Small => "translate-x-0",
                ToggleSize::Medium => "translate-x-0",
                ToggleSize::Large => "translate-x-0",
            }
        }
    }
}

#[component]
pub fn ToggleButton(
    is_checked: bool,
    on_toggle: EventHandler<()>,
    #[props(default = ToggleSize::Medium)] size: ToggleSize,
    #[props(default = String::new())] class: String,
    #[props(default = false)] show_icons: bool,
    #[props(default = String::new())] label: String,
) -> Element {
    rsx! {
        div {
            class: "flex items-center space-x-3 {class}",

            if !label.is_empty() {
                label {
                    class: "text-sm font-medium text-primary-text dark:text-primary-text",
                    {label}
                }
            }

            div {
                class: "relative",
                input {
                    r#type: "checkbox",
                    class: "sr-only peer",
                    checked: is_checked,
                    onchange: move |_| on_toggle.call(()),
                }
                div {
                    class: format!(
                        "relative {} bg-gray-300 rounded-full peer-checked:bg-primary transition-colors duration-300 cursor-pointer dark:bg-gray-600 dark:peer-checked:bg-primary",
                        size.switch_classes()
                    ),
                    onclick: move |_| on_toggle.call(()),

                    div {
                        class: format!(
                            "absolute top-1/2 left-1 transform -translate-y-1/2 {} bg-white rounded-full shadow-md transition-transform duration-300 dark:bg-white {}",
                            size.toggle_classes(),
                            size.translate_classes(is_checked)
                        ),

                        if show_icons {
                            div {
                                class: "w-full h-full flex items-center justify-center",
                                if is_checked {
                                    // Moon icon
                                    svg {
                                        class: "w-2.5 h-2.5 text-gray-600 dark:text-gray-400",
                                        fill: "currentColor",
                                        view_box: "0 0 20 20",
                                        path {
                                            d: "M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z"
                                        }
                                    }
                                } else {
                                    // Sun icon
                                    svg {
                                        class: "w-2.5 h-2.5 text-yellow-500 dark:text-yellow-400",
                                        fill: "currentColor",
                                        view_box: "0 0 20 20",
                                        path {
                                            fill_rule: "evenodd",
                                            d: "M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z",
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

#[component]
pub fn ThemeSelector(
    current_theme: String,
    on_theme_change: EventHandler<String>,
    #[props(default = String::new())] class: String,
    #[props(default = String::new())] label: String,
) -> Element {
    let themes = vec![("system", "System"), ("light", "Light"), ("dark", "Dark")];

    let current_theme_clone = current_theme.clone();

    rsx! {
        div {
            class: "space-y-2 dark:text-primary-text {class}",

            if !label.is_empty() {
                label {
                    class: "block text-sm font-semibold text-primary-light mb-2 dark:text-primary-light",
                    {label}
                }
            }

            div {
                class: "flex space-x-1 p-1 bg-surface border border-border rounded-lg dark:bg-surface dark:border-border",

                for (theme_value, theme_label) in themes {
                    {
                        let current = current_theme_clone.clone();
                        let value = theme_value.to_string();
                        rsx! {
                            button {
                                class: format!(
                                    "flex-1 px-3 py-2 text-sm font-medium rounded-md transition-all duration-200 {}",
                                    if current == theme_value {
                                        "bg-primary text-primary-text shadow-sm"
                                    } else {
                                        "text-secondary-text hover:text-primary-text hover:bg-primary-lighter dark:text-secondary-text dark:hover:text-primary-text dark:hover:bg-primary-lighter"
                                    }
                                ),
                                onclick: move |_| {
                                    if current != theme_value {
                                        on_theme_change.call(value.clone());
                                    }
                                },
                                {theme_label}
                            }
                        }
                    }
                }
            }
        }
    }
}
