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
pub enum Action {
    // === NAVIGATION ===
    /// Move cursor up in the current context.
    NavigateUp,
    /// Move cursor down in the current context.
    NavigateDown,
    /// Move cursor left in the current context.
    NavigateLeft,
    /// Move cursor right in the current context.
    NavigateRight,
    /// Jump to the first item in the current context.
    JumpToFirst,
    /// Jump to the last item in the current context.
    JumpToLast,

    // === LAYER SWITCHING ===
    /// Switch to the next layer.
    NextLayer,
    /// Switch to the previous layer.
    PreviousLayer,

    // === KEY EDITING ===
    /// Open the keycode picker dialog for editing a key.
    OpenKeycodePicker,
    /// Clear the current key, removing its keycode.
    ClearKey,

    // === CLIPBOARD ===
    /// Copy the current key to the clipboard.
    CopyKey,
    /// Cut the current key to the clipboard.
    CutKey,
    /// Paste the clipboard contents to the current key.
    PasteKey,
    /// Undo the last paste operation.
    UndoPaste,

    // === SELECTION ===
    /// Toggle selection mode for multiple keys.
    ToggleSelectionMode,
    /// Toggle selection of the current key.
    ToggleCurrentKey,
    /// Start a rectangle selection of multiple keys.
    StartRectangleSelect,
    /// Swap properties between two keys.
    SwapKeys,

    // === COLORS ===
    /// Open color picker to set color for the individual key.
    SetIndividualKeyColor,
    /// Open color picker to set color for the entire layer.
    SetLayerColor,
    /// Toggle the visibility of layer colors.
    ToggleLayerColors,
    /// Toggle the visibility of colors for all layers.
    ToggleAllLayerColors,

    // === CATEGORIES ===
    /// Open the category manager dialog.
    OpenCategoryManager,
    /// Assign a category to the current key.
    AssignCategoryToKey,
    /// Assign a category to the current layer.
    AssignCategoryToLayer,

    // === MANAGERS & DIALOGS ===
    /// Open the layer manager dialog.
    OpenLayerManager,
    /// Open the application settings dialog.
    OpenSettings,
    /// Open the metadata editor dialog.
    EditMetadata,
    /// Open the tap dance editor dialog.
    OpenTapDanceEditor,

    // === FILE OPERATIONS ===
    /// Save the current keyboard configuration.
    Save,
    /// Export the current layout to markdown format.
    ExportLayout,
    /// Quit the application.
    Quit,

    // === BUILD & FIRMWARE ===
    /// Build the firmware for the current keyboard.
    BuildFirmware,
    /// Generate the firmware code for the current keyboard.
    GenerateFirmware,
    /// View the build log from the last firmware build.
    ViewBuildLog,

    // === TEMPLATES ===
    /// Open the template browser to load a template configuration.
    BrowseTemplates,
    /// Save the current configuration as a template.
    SaveAsTemplate,

    // === CONFIGURATION ===
    /// Open the setup wizard for initial configuration.
    SetupWizard,
    /// Switch to a different layout variant.
    SwitchLayoutVariant,

    // === HELP ===
    /// Toggle the help panel display.
    ToggleHelp,

    // === GENERAL ===
    /// Cancel the current operation or close dialogs.
    Cancel,
}

/// Shortcut registry that maps key events to actions for a given context.
///
/// This is the central source of truth for all keyboard shortcuts in the application.
pub struct ShortcutRegistry {
    /// Maps (context, `key_binding`) to Action
    bindings: HashMap<(String, KeyBinding), Action>,
}

/// A key binding (key + modifiers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    /// The key code (character, function key, arrow key, etc.).
    pub code: KeyCode,
    /// The modifier keys (Ctrl, Shift, Alt) pressed with the key.
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    /// Create a new key binding.
    #[must_use]
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    /// Create a key binding from a `KeyEvent`.
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
        // Some terminals send Tab+SHIFT instead of BackTab
        self.register(ctx, K::Tab, M::SHIFT, Action::PreviousLayer);
        // Some terminals emit BackTab with the SHIFT modifier set
        self.register(ctx, K::BackTab, M::SHIFT, Action::PreviousLayer);

        // === KEY EDITING ===
        self.register(ctx, K::Enter, M::NONE, Action::OpenKeycodePicker);
        self.register(ctx, K::Backspace, M::NONE, Action::ClearKey);
        self.register(ctx, K::Delete, M::NONE, Action::ClearKey);

        // === CLIPBOARD ===
        self.register(ctx, K::Char('y'), M::NONE, Action::CopyKey);
        self.register(ctx, K::Char('c'), M::CONTROL, Action::CopyKey);
        self.register(ctx, K::Char('x'), M::CONTROL, Action::CutKey);
        self.register(ctx, K::Char('p'), M::NONE, Action::PasteKey);
        self.register(ctx, K::Char('v'), M::CONTROL, Action::PasteKey);
        self.register(ctx, K::Char('z'), M::CONTROL, Action::UndoPaste);

        // === SELECTION ===
        self.register(ctx, K::Char('V'), M::SHIFT, Action::ToggleSelectionMode);
        self.register(ctx, K::Char(' '), M::NONE, Action::ToggleCurrentKey);
        self.register(ctx, K::Char('R'), M::SHIFT, Action::StartRectangleSelect);
        self.register(ctx, K::Char('W'), M::SHIFT, Action::SwapKeys);

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
        self.register(ctx, K::Char('D'), M::SHIFT, Action::OpenTapDanceEditor);

        // === FILE OPERATIONS ===
        self.register(ctx, K::Char('s'), M::CONTROL, Action::Save);
        self.register(ctx, K::Char('e'), M::CONTROL, Action::ExportLayout);
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
        self.register(ctx, K::Char('Y'), M::SHIFT, Action::SwitchLayoutVariant);

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
    #[allow(dead_code)] // bin/lib split: registry helper (tests use it)
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
mod tests;
