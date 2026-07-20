//! Popup type definitions, visual kind classification, and shared chrome helpers.
//!
//! Provides `PopupType`, `PopupVisualKind`, and the associated color/title helpers
//! used by popup rendering throughout the TUI.

use ratatui::style::Style;

use crate::tui::Theme;

/// Visual category for popup chrome and mode labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupVisualKind {
    /// Selection or lookup task that returns to prior context.
    Picker,
    /// Editing task that changes draft content directly.
    Editor,
    /// Settings management task for global or layout preferences.
    Settings,
    /// Guided onboarding or setup flow.
    Wizard,
    /// Read-only feedback, logs, or help content.
    Feedback,
    /// Confirmation step for risky actions.
    Confirm,
}

impl PopupVisualKind {
    /// Human-readable mode label for title bar and popup chrome.
    #[must_use]
    pub const fn mode_label(self) -> &'static str {
        match self {
            Self::Picker => "Picker",
            Self::Editor => "Editor",
            Self::Settings => "Settings",
            Self::Wizard => "Wizard",
            Self::Feedback => "Feedback",
            Self::Confirm => "Confirm",
        }
    }
}

/// Popup types that can be displayed over the main UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupType {
    /// Keycode picker popup
    KeycodePicker,
    /// Color picker popup
    ColorPicker,
    /// Category picker popup
    CategoryPicker,
    /// Category manager popup
    CategoryManager,
    /// Layer manager popup
    LayerManager,
    /// Layer picker popup (for layer-switching keycodes)
    LayerPicker,
    /// Template browser popup
    TemplateBrowser,
    /// Template save dialog popup
    TemplateSaveDialog,
    /// Export filename dialog popup
    ExportFilenameDialog,
    /// Help overlay popup
    HelpOverlay,
    /// Build log popup
    BuildLog,
    /// Metadata editor popup
    MetadataEditor,
    /// Unsaved changes confirmation popup
    UnsavedChangesPrompt,
    /// Layout picker popup
    LayoutPicker,
    /// Setup wizard popup
    SetupWizard,
    /// Settings manager popup
    SettingsManager,
    /// Tap keycode picker for parameterized keycodes (second stage of LT/MT)
    TapKeycodePicker,
    /// Modifier picker for MT/LM keycodes
    ModifierPicker,
    /// Key editor popup for viewing/editing key properties
    KeyEditor,
    /// Tap dance editor popup
    TapDanceEditor,
    /// Tap dance form dialog (create/edit)
    TapDanceForm,
}

impl PopupType {
    /// Visual task category used for popup chrome and orientation cues.
    #[must_use]
    pub const fn visual_kind(&self) -> PopupVisualKind {
        match self {
            Self::KeycodePicker
            | Self::ColorPicker
            | Self::CategoryPicker
            | Self::LayerPicker
            | Self::LayoutPicker
            | Self::TapKeycodePicker
            | Self::ModifierPicker => PopupVisualKind::Picker,
            Self::CategoryManager
            | Self::LayerManager
            | Self::TemplateBrowser
            | Self::MetadataEditor
            | Self::KeyEditor
            | Self::TapDanceEditor
            | Self::TapDanceForm
            | Self::TemplateSaveDialog
            | Self::ExportFilenameDialog => PopupVisualKind::Editor,
            Self::SettingsManager => PopupVisualKind::Settings,
            Self::SetupWizard => PopupVisualKind::Wizard,
            Self::BuildLog | Self::HelpOverlay => PopupVisualKind::Feedback,
            Self::UnsavedChangesPrompt => PopupVisualKind::Confirm,
        }
    }
}

/// Get popup accent color by task type.
fn popup_kind_color(kind: PopupVisualKind, theme: &Theme) -> ratatui::style::Color {
    match kind {
        PopupVisualKind::Picker => theme.primary,
        PopupVisualKind::Editor => theme.accent,
        PopupVisualKind::Settings => theme.success,
        PopupVisualKind::Wizard => theme.warning,
        PopupVisualKind::Feedback => theme.text_secondary,
        PopupVisualKind::Confirm => theme.error,
    }
}

/// Shared popup border style based on popup task type.
#[must_use]
pub fn popup_border_style(popup_type: &PopupType, theme: &Theme) -> Style {
    Style::default().fg(popup_kind_color(popup_type.visual_kind(), theme))
}

/// Shared popup title format with task-type label.
#[must_use]
pub fn popup_title(popup_type: &PopupType, label: &str) -> String {
    format!(" {} · {} ", popup_type.visual_kind().mode_label(), label)
}
