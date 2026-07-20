//! Settings manager input handler — split across sibling files.

pub mod apply;
pub mod browsing;
pub mod event;
pub mod input;

pub use input::handle_settings_manager_input;
