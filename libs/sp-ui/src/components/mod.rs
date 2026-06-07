use uuid::Uuid;

pub mod button;
pub mod color_theme_selector;
pub mod context_menu;
pub mod dropdown;
pub mod input;
pub mod theme_toggle;
pub mod toastcontainer;

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}
