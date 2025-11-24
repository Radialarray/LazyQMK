//! Terminal user interface components and state management.
//!
//! This module contains the main TUI loop, `AppState`, event handling,
//! and all UI widgets using Ratatui.

pub mod config_dialogs;
pub mod onboarding_wizard;

// Re-export TUI components
pub use config_dialogs::{
    KeyboardPickerState, LayoutPickerState, PathConfigDialogState,
};
pub use onboarding_wizard::OnboardingWizardState;
