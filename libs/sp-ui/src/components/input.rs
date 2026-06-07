use dioxus::prelude::*;

use crate::components::new_id;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FieldSize {
    Tiny,
    Small,
    #[default]
    Medium,
    Large,
    Full,
    Custom(usize),
}

impl FieldSize {
    pub fn class(&self) -> String {
        match self {
            FieldSize::Tiny => "w-24".to_string(),
            FieldSize::Small => "w-32".to_string(),
            FieldSize::Medium => "w-64".to_string(),
            FieldSize::Large => "w-96".to_string(),
            FieldSize::Full => "w-full".to_string(),
            FieldSize::Custom(size) => format!("w-{size}"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FieldType {
    #[default]
    Text,
    Password,
    Email,
    Number,
}

impl FieldType {
    pub fn r#type(&self) -> &'static str {
        match self {
            FieldType::Text => "text",
            FieldType::Password => "password",
            FieldType::Email => "email",
            FieldType::Number => "number",
        }
    }
}

fn signal_value(value: Signal<String>) -> String {
    value()
}

#[component]
pub fn TextInput(
    id: Option<String>,
    name: Option<String>,
    #[props(default = String::new())] class: String,
    #[props(default = FieldSize::Medium)] size: FieldSize,
    #[props(default)] r#type: FieldType,
    label: Option<String>,
    #[props(default = String::new())] label_class: String,
    placeholder: Option<String>,
    max_length: Option<usize>,
    mut value: Signal<String>,
    oninput: Option<Callback<String>>,
    onkeydown: Option<Callback<Event<KeyboardData>>>,
    #[props(default = false)] show_clear: bool, // <-- Add this line
) -> Element {
    let id = id.unwrap_or_else(new_id);
    let mut live_value = use_signal(move || signal_value(value));
    rsx! {
        div {
            class: "my-2 relative w-full",

            if label.is_some() {
                label {
                    class: "block text-primary-light text-sm font-semibold mb-2 text-left dark:text-primary-light {label_class}",
                    r#for: id.clone(),
                    {label}
                }
            }
            div {
                class: "relative flex items-center w-full",
                input {
                    class: format!("shadow appearance-none border border-border rounded-lg py-3 px-4 text-primary-text bg-surface leading-tight focus:outline-none focus:ring-2 focus:ring-primary focus:border-primary transition-all duration-300 dark:border-border dark:text-primary-text dark:bg-surface dark:focus:ring-primary dark:focus:border-primary {class} {}", size.class()),
                    id: id,
                    name: name,
                    r#type: r#type.r#type(),
                    placeholder: placeholder,
                    maxlength: max_length,
                    required: true,
                    value: value,
                    onchange: move |e| {
                        *value.write() = e.value();
                        *live_value.write() = e.value();
                    },
                    oninput: move |e| {
                        *live_value.write() = e.value();
                        if let Some(oninput) = &oninput {
                            oninput.call(e.value());
                        }
                    },
                    onkeydown: move |e| {
                        if let Some(onkeydown) = &onkeydown {
                            onkeydown.call(e.clone());
                        }
                    }
                }
                if show_clear && !live_value().is_empty() {
                    button {
                        r#type: "button",
                        class: "absolute right-3 text-secondary-text hover:text-primary-text focus:outline-none transition-colors duration-200 dark:text-secondary-text dark:hover:text-primary-text",
                        onclick: move |_| {
                            *value.write() = String::new();
                            *live_value.write() = String::new();
                            if let Some(oninput) = &oninput {
                                oninput.call(String::new());
                            }
                        },
                        "✕"
                    }
                }
            }
        }
    }
}

#[component]
pub fn TextArea(
    id: Option<String>,
    name: Option<String>,
    #[props(default = String::new())] class: String,
    #[props(default = FieldSize::Medium)] size: FieldSize,
    label: String,
    #[props(default = String::new())] label_class: String,
    placeholder: Option<String>,
    max_length: Option<usize>,
    rows: Option<i64>,
    mut value: Signal<String>,
) -> Element {
    let id = id.unwrap_or_else(new_id);
    rsx! {
        div {
            class: "my-2",

            label {
                class: "block text-primary-light text-sm font-semibold mb-2 text-left dark:text-primary-light {label_class}",
                r#for: id.clone(),
                {label}
            },
            textarea {
                id: id,
                name: name,
                class: format!("shadow appearance-none border border-border rounded-lg py-3 px-4 text-primary-text bg-surface leading-tight focus:outline-none focus:ring-2 focus:ring-primary focus:border-primary transition-all duration-300 resize-vertical dark:border-border dark:text-primary-text dark:bg-surface dark:focus:ring-primary dark:focus:border-primary {class} {}", size.class()),
                placeholder: placeholder,
                maxlength: max_length,
                rows: rows,
                required: true,
                value: value,
                onchange: move |e| { *value.write() = e.value()}
            }
        }
    }
}
