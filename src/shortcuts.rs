//! Centralized shortcut and action system.
//!
//! This module provides a unified system for keyboard shortcuts and actions,
//! connecting help text definitions with actual event handling logic.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// All possible actions in the application.
///
/// This enum represents every action a user can take. It serves as the
/// bridge between keyboard shortcuts and application behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum Action {
    // === NAVIGATION ===
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    JumpToFirst,
    JumpToLast,

    // === LAYER SWITCHING ===
    NextLayer,
    PreviousLayer,

    // === KEY EDITING ===
    OpenKeycodePicker,
    ClearKey,

    // === CLIPBOARD ===
    CopyKey,
    CutKey,
    PasteKey,
    UndoPaste,

    // === SELECTION ===
    ToggleSelectionMode,
    ToggleCurrentKey,
    StartRectangleSelect,

    // === COLORS ===
    SetIndividualKeyColor,
    SetLayerColor,
    ToggleLayerColors,
    ToggleAllLayerColors,

    // === CATEGORIES ===
    OpenCategoryManager,
    AssignCategoryToKey,
    AssignCategoryToLayer,

    // === MANAGERS & DIALOGS ===
    OpenLayerManager,
    OpenSettings,
    EditMetadata,

    // === FILE OPERATIONS ===
    Save,
    Quit,

    // === BUILD & FIRMWARE ===
    BuildFirmware,
    GenerateFirmware,
    ViewBuildLog,

    // === TEMPLATES ===
    BrowseTemplates,
    SaveAsTemplate,

    // === CONFIGURATION ===
    SetupWizard,
    SwitchLayoutVariant,

    // === HELP ===
    ToggleHelp,

    // === GENERAL ===
    Cancel,
}

impl Action {
    /// Get the action ID string used in help.toml
    #[must_use]
    pub const fn id(&self) -> &'static str {
        match self {
            // Navigation
            Self::NavigateUp => "navigate_up",
            Self::NavigateDown => "navigate_down",
            Self::NavigateLeft => "navigate_left",
            Self::NavigateRight => "navigate_right",
            Self::JumpToFirst => "jump_first",
            Self::JumpToLast => "jump_last",

            // Layers
            Self::NextLayer => "next_layer",
            Self::PreviousLayer => "previous_layer",

            // Key editing
            Self::OpenKeycodePicker => "open_keycode_picker",
            Self::ClearKey => "clear_key",

            // Clipboard
            Self::CopyKey => "copy_key",
            Self::CutKey => "cut_key",
            Self::PasteKey => "paste_key",
            Self::UndoPaste => "undo_paste",

            // Selection
            Self::ToggleSelectionMode => "toggle_selection_mode",
            Self::ToggleCurrentKey => "toggle_current_key",
            Self::StartRectangleSelect => "rectangle_select",

            // Colors
            Self::SetIndividualKeyColor => "set_individual_key_color",
            Self::SetLayerColor => "set_layer_color",
            Self::ToggleLayerColors => "toggle_layer_colors",
            Self::ToggleAllLayerColors => "toggle_all_layer_colors",

            // Categories
            Self::OpenCategoryManager => "open_category_manager",
            Self::AssignCategoryToKey => "assign_category_to_key",
            Self::AssignCategoryToLayer => "assign_category_to_layer",

            // Managers
            Self::OpenLayerManager => "open_layer_manager",
            Self::OpenSettings => "open_settings",
            Self::EditMetadata => "edit_metadata",

            // File operations
            Self::Save => "save",
            Self::Quit => "quit",

            // Build
            Self::BuildFirmware => "build_firmware",
            Self::GenerateFirmware => "generate_firmware",
            Self::ViewBuildLog => "view_build_log",

            // Templates
            Self::BrowseTemplates => "browse_templates",
            Self::SaveAsTemplate => "save_template",

            // Configuration
            Self::SetupWizard => "setup_wizard",
            Self::SwitchLayoutVariant => "switch_layout_variant",

            // Help
            Self::ToggleHelp => "toggle_help",

            // General
            Self::Cancel => "cancel",
        }
    }
}

/// Shortcut registry that maps key events to actions for a given context.
///
/// This is the central source of truth for all keyboard shortcuts in the application.
pub struct ShortcutRegistry {
    /// Maps (context, key_binding) to Action
    bindings: HashMap<(String, KeyBinding), Action>,
}

/// A key binding (key + modifiers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    /// Create a new key binding.
    #[must_use]
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    /// Create a key binding from a KeyEvent.
    #[must_use]
    pub const fn from_event(event: KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

impl ShortcutRegistry {
    /// Create a new shortcut registry with default bindings.
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self {
            bindings: HashMap::new(),
        };

        registry.register_main_shortcuts();
        registry
    }

    /// Register all shortcuts for the main context.
    fn register_main_shortcuts(&mut self) {
        use KeyCode as K;
        use KeyModifiers as M;

        let ctx = "main";

        // === NAVIGATION ===
        self.register(ctx, K::Up, M::NONE, Action::NavigateUp);
        self.register(ctx, K::Down, M::NONE, Action::NavigateDown);
        self.register(ctx, K::Left, M::NONE, Action::NavigateLeft);
        self.register(ctx, K::Right, M::NONE, Action::NavigateRight);
        self.register(ctx, K::Char('k'), M::NONE, Action::NavigateUp);
        self.register(ctx, K::Char('j'), M::NONE, Action::NavigateDown);
        self.register(ctx, K::Char('h'), M::NONE, Action::NavigateLeft);
        self.register(ctx, K::Char('l'), M::NONE, Action::NavigateRight);
        self.register(ctx, K::Home, M::NONE, Action::JumpToFirst);
        self.register(ctx, K::End, M::NONE, Action::JumpToLast);

        // === LAYERS ===
        self.register(ctx, K::Tab, M::NONE, Action::NextLayer);
        self.register(ctx, K::BackTab, M::NONE, Action::PreviousLayer);

        // === KEY EDITING ===
        self.register(ctx, K::Enter, M::NONE, Action::OpenKeycodePicker);
        self.register(ctx, K::Char('x'), M::NONE, Action::ClearKey);
        self.register(ctx, K::Delete, M::NONE, Action::ClearKey);

        // === CLIPBOARD ===
        self.register(ctx, K::Char('y'), M::NONE, Action::CopyKey);
        self.register(ctx, K::Char('c'), M::CONTROL, Action::CopyKey);
        self.register(ctx, K::Char('d'), M::NONE, Action::CutKey);
        self.register(ctx, K::Char('x'), M::CONTROL, Action::CutKey);
        self.register(ctx, K::Char('p'), M::NONE, Action::PasteKey);
        self.register(ctx, K::Char('v'), M::CONTROL, Action::PasteKey);
        self.register(ctx, K::Char('z'), M::CONTROL, Action::UndoPaste);

        // === SELECTION ===
        self.register(ctx, K::Char('V'), M::SHIFT, Action::ToggleSelectionMode);
        self.register(ctx, K::Char(' '), M::NONE, Action::ToggleCurrentKey);
        self.register(ctx, K::Char('R'), M::SHIFT, Action::StartRectangleSelect);

        // === COLORS (v0.4.0: c = individual, Shift+C = layer) ===
        self.register(ctx, K::Char('c'), M::NONE, Action::SetIndividualKeyColor);
        self.register(ctx, K::Char('C'), M::SHIFT, Action::SetLayerColor);
        self.register(ctx, K::Char('v'), M::NONE, Action::ToggleLayerColors);
        self.register(ctx, K::Char('V'), M::ALT, Action::ToggleAllLayerColors);

        // === CATEGORIES (v0.4.0: Shift+K = manager, Ctrl+K/L = quick assign) ===
        self.register(ctx, K::Char('K'), M::SHIFT, Action::OpenCategoryManager);
        self.register(ctx, K::Char('k'), M::CONTROL, Action::AssignCategoryToKey);
        self.register(ctx, K::Char('l'), M::CONTROL, Action::AssignCategoryToLayer);

        // === MANAGERS & DIALOGS (v0.4.0: Shift+L = layers, Shift+E = metadata) ===
        self.register(ctx, K::Char('L'), M::SHIFT, Action::OpenLayerManager);
        self.register(ctx, K::Char('S'), M::SHIFT, Action::OpenSettings);
        self.register(ctx, K::Char('E'), M::SHIFT, Action::EditMetadata);

        // === FILE OPERATIONS ===
        self.register(ctx, K::Char('s'), M::CONTROL, Action::Save);
        self.register(ctx, K::Char('q'), M::CONTROL, Action::Quit);

        // === BUILD & FIRMWARE (v0.4.0: Shift+B = build log) ===
        self.register(ctx, K::Char('b'), M::CONTROL, Action::BuildFirmware);
        self.register(ctx, K::Char('g'), M::CONTROL, Action::GenerateFirmware);
        self.register(ctx, K::Char('B'), M::SHIFT, Action::ViewBuildLog);

        // === TEMPLATES ===
        self.register(ctx, K::Char('t'), M::NONE, Action::BrowseTemplates);
        self.register(ctx, K::Char('T'), M::SHIFT, Action::SaveAsTemplate);

        // === CONFIGURATION ===
        self.register(ctx, K::Char('w'), M::CONTROL, Action::SetupWizard);
        self.register(ctx, K::Char('y'), M::CONTROL, Action::SwitchLayoutVariant);

        // === HELP ===
        self.register(ctx, K::Char('?'), M::NONE, Action::ToggleHelp);

        // === GENERAL ===
        self.register(ctx, K::Esc, M::NONE, Action::Cancel);
    }

    /// Register a shortcut binding.
    fn register(&mut self, context: &str, code: KeyCode, modifiers: KeyModifiers, action: Action) {
        let binding = KeyBinding::new(code, modifiers);
        self.bindings.insert((context.to_string(), binding), action);
    }

    /// Look up an action for a given context and key event.
    #[must_use]
    pub fn lookup(&self, context: &str, event: KeyEvent) -> Option<Action> {
        let binding = KeyBinding::from_event(event);
        self.bindings.get(&(context.to_string(), binding)).copied()
    }

    /// Check if a key event matches a specific action in the given context.
    #[must_use]
    pub fn matches(&self, context: &str, event: KeyEvent, action: Action) -> bool {
        self.lookup(context, event) == Some(action)
    }
}

impl Default for ShortcutRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_lookup() {
        let registry = ShortcutRegistry::new();

        // Test navigation
        let event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(registry.lookup("main", event), Some(Action::NavigateUp));

        // Test save
        let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert_eq!(registry.lookup("main", event), Some(Action::Save));
    }

    #[test]
    fn test_v040_shortcuts() {
        let registry = ShortcutRegistry::new();

        // Test flipped color shortcuts
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        assert_eq!(
            registry.lookup("main", event),
            Some(Action::SetIndividualKeyColor)
        );

        let event = KeyEvent::new(KeyCode::Char('C'), KeyModifiers::SHIFT);
        assert_eq!(registry.lookup("main", event), Some(Action::SetLayerColor));

        // Test new category manager shortcut
        let event = KeyEvent::new(KeyCode::Char('K'), KeyModifiers::SHIFT);
        assert_eq!(
            registry.lookup("main", event),
            Some(Action::OpenCategoryManager)
        );

        // Test new layer manager shortcut
        let event = KeyEvent::new(KeyCode::Char('L'), KeyModifiers::SHIFT);
        assert_eq!(
            registry.lookup("main", event),
            Some(Action::OpenLayerManager)
        );

        // Test new build log shortcut
        let event = KeyEvent::new(KeyCode::Char('B'), KeyModifiers::SHIFT);
        assert_eq!(registry.lookup("main", event), Some(Action::ViewBuildLog));

        // Test new metadata editor shortcut
        let event = KeyEvent::new(KeyCode::Char('E'), KeyModifiers::SHIFT);
        assert_eq!(registry.lookup("main", event), Some(Action::EditMetadata));

        // Test quick category assignment
        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
        assert_eq!(
            registry.lookup("main", event),
            Some(Action::AssignCategoryToKey)
        );

        let event = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL);
        assert_eq!(
            registry.lookup("main", event),
            Some(Action::AssignCategoryToLayer)
        );
    }

    #[test]
    fn test_vim_navigation() {
        let registry = ShortcutRegistry::new();

        // Vim keys should work for navigation
        assert_eq!(
            registry.lookup(
                "main",
                KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE)
            ),
            Some(Action::NavigateLeft)
        );
        assert_eq!(
            registry.lookup(
                "main",
                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)
            ),
            Some(Action::NavigateDown)
        );
        assert_eq!(
            registry.lookup(
                "main",
                KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)
            ),
            Some(Action::NavigateUp)
        );
        assert_eq!(
            registry.lookup(
                "main",
                KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE)
            ),
            Some(Action::NavigateRight)
        );
    }
}
