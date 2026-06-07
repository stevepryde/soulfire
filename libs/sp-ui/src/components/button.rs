use dioxus::prelude::*;

use super::input::FieldSize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonType {
    #[default]
    Button,
    PrimaryButton,
    Submit,
    Secondary,
    Danger,
    Ghost,
}

impl ButtonType {
    pub fn r#type(&self) -> &'static str {
        match self {
            ButtonType::Button => "button",
            ButtonType::PrimaryButton => "button",
            ButtonType::Submit => "submit",
            ButtonType::Secondary => "button",
            ButtonType::Danger => "button",
            ButtonType::Ghost => "button",
        }
    }
}

#[component]
pub fn Button(
    #[props(default = String::new())] class: String,
    #[props(default = ButtonType::Button)] r#type: ButtonType,
    value: String,
    #[props(default = FieldSize::Medium)] size: FieldSize,
    onclick: Option<EventHandler<MouseEvent>>,
    #[props(default = false)] disabled: bool,
) -> Element {
    let colour = match r#type {
        ButtonType::Button => {
            "bg-gray-200 hover:bg-gray-300 text-primary-text hover:text-primary-text border border-gray-300 hover:border-gray-400 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-primary-text dark:hover:text-primary-text dark:border-gray-600 dark:hover:border-gray-500"
        }
        ButtonType::PrimaryButton => "crm-primary-button",
        ButtonType::Submit => {
            "bg-green-600 hover:bg-green-700 text-white border border-green-600 hover:border-green-700 dark:bg-green-700 dark:hover:bg-green-500 dark:text-white dark:border-green-700 dark:hover:border-green-500"
        }
        ButtonType::Secondary => {
            "bg-secondary hover:bg-secondary text-gray-900 border border-secondary hover:border-secondary dark:bg-secondary dark:hover:bg-secondary dark:text-gray-900 dark:border-secondary dark:hover:border-secondary"
        }
        ButtonType::Danger => {
            "bg-red-600 hover:bg-red-700 text-white border border-red-600 hover:border-red-700 dark:bg-red-600 dark:hover:bg-red-500 dark:text-white dark:border-red-600 dark:hover:border-red-500"
        }
        ButtonType::Ghost => {
            "bg-transparent hover:bg-primary-lighter text-primary-text hover:text-primary border border-border hover:border-primary dark:bg-transparent dark:hover:bg-primary-lighter dark:text-primary-text dark:hover:text-primary dark:border-border dark:hover:border-primary"
        }
    };

    rsx! {
        button {
            class: format!("{colour} font-semibold py-2 px-4 rounded-[0.35rem] transition-all duration-300 focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 disabled:opacity-50 cursor-pointer disabled:cursor-not-allowed min-h-12 {class} {}", size.class()),
            r#type: r#type.r#type(),
            onclick: move |e| if let Some(onclick) = onclick { onclick.call(e) },
            disabled,
            {value}
        }
    }
}
