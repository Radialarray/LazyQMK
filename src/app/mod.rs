//! Application orchestration layer
//!
//! This module provides high-level orchestration functions that coordinate
//! between different parts of the application (TUI, config, parsing, etc.)
//! without containing detailed implementation logic.

/// Editor initialization and startup with keyboard layout configuration
pub mod launch;

/// Layout selection interface for creating or loading existing layouts
pub mod layout_picker;

pub mod onboarding;

// Re-export commonly used functions for convenience
pub use layout_picker::run_layout_picker_terminal;
pub use onboarding::run_onboarding_wizard_terminal;
