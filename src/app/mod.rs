//! Application orchestration layer
//!
//! This module provides high-level orchestration functions that coordinate
//! between different parts of the application (TUI, config, parsing, etc.)
//! without containing detailed implementation logic.

pub mod launch;
pub mod layout_picker;
pub mod onboarding;

// Re-export commonly used functions for convenience
pub use launch::launch_editor_with_default_layout;
pub use layout_picker::run_layout_picker_terminal;
pub use onboarding::{run_new_layout_wizard_terminal, run_onboarding_wizard_terminal};
