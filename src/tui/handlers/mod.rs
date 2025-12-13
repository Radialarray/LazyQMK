//! Input handler modules for different TUI contexts.

pub mod action_handlers;
pub mod actions;
pub mod category;
pub mod layer;
pub mod main;
pub mod popups;
pub mod settings;
pub mod tap_dance;
pub mod templates;

// Re-export handler functions
pub use actions::dispatch_action;
pub use category::handle_category_manager_input;
pub use layer::handle_layer_manager_input;
pub use main::handle_main_input;
pub use popups::handle_popup_input;
pub use settings::handle_settings_manager_input;
pub use tap_dance::handle_tap_dance_editor_input;
pub use templates::{handle_template_browser_input, handle_template_save_dialog_input};
