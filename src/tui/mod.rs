//! Terminal user interface components and state management.
//!
//! This module contains the main TUI loop, `AppState`, event handling,
//! and all UI widgets using Ratatui.

// Allow clone assignment patterns - common in UI state management
#![allow(clippy::assigning_clones)]
// Input handlers use Result<bool> for consistency even when they never fail
#![allow(clippy::unnecessary_wraps)]
// Allow small types passed by reference for API consistency
#![allow(clippy::trivially_copy_pass_by_ref)]
// Allow intentional type casts for terminal coordinates
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]

pub mod build_log;
pub mod category_manager;
pub mod category_picker;
pub mod clipboard;
pub mod color_picker;
pub mod component;
pub mod config_dialogs;
pub mod handlers;
pub mod help_overlay;
pub mod help_registry;
pub mod key_editor;
pub mod keyboard;
pub mod keycode_picker;
pub mod layer_manager;
pub mod layer_picker;
pub mod layout_picker;
pub mod metadata_editor;
pub mod modifier_picker;
#[allow(dead_code)]
pub mod onboarding_wizard;
pub mod settings_manager;
pub mod status_bar;
pub mod tap_dance_editor;
pub mod tap_dance_form;
pub mod template_browser;
pub mod theme;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout as RatatuiLayout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crate::config::Config;
use crate::firmware::BuildState;
use crate::keycode_db::KeycodeDb;
use crate::models::{KeyboardGeometry, Layout, Position, VisualLayoutMapping};
use crate::services::geometry::{
    build_geometry_for_layout, extract_base_keyboard, GeometryContext,
};
use crate::services::layer_refs::{build_layer_ref_index, LayerRef};
use std::collections::HashMap;

// Re-export TUI components
pub use build_log::BuildLog;
pub use category_manager::{CategoryManager, CategoryManagerState};
pub use category_picker::{CategoryPicker, CategoryPickerEvent};
pub use color_picker::ColorPicker;
pub use component::{Component, ContextualComponent};
pub use config_dialogs::{
    LayoutPicker as LayoutVariantPicker, LayoutPickerEvent as LayoutVariantPickerEvent,
};
pub use help_overlay::HelpOverlay;
pub use key_editor::KeyEditorState;
pub use keyboard::KeyboardWidget;
pub use keycode_picker::KeycodePicker;
pub use layer_manager::{LayerManager, LayerManagerEvent};
#[allow(unused_imports)]
pub use layer_picker::{LayerPicker, LayerPickerState};
pub use layout_picker::LayoutPicker;
// MetadataEditor component migrated in Wave 4c - uses Component trait pattern
// SettingsManager component migrated in Wave 8 - uses custom ContextualComponent pattern
pub use modifier_picker::ModifierPicker;
pub use status_bar::StatusBar;
pub use template_browser::TemplateBrowser;
pub use theme::Theme;

// Import handler functions from the handlers module

/// Category picker context - what are we setting the category for?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryPickerContext {
    /// Setting category for individual key
    IndividualKey,
    /// Setting category for entire layer
    Layer,
    /// Setting category for multiple selected keys
    MultiKeySelection,
}

/// Type of parameterized keycode being built
/// State for building parameterized keycodes through multi-stage picker flow
#[derive(Debug, Clone, Default)]
pub struct PendingKeycodeState {
    /// The original keycode template (e.g., "LT()", "MT()", "LCTL_T()")
    pub keycode_template: Option<String>,
    /// Collected parameters in order (e.g., `["@layer-id", "KC_SPC"]` for LT)
    pub params: Vec<String>,
}

impl PendingKeycodeState {
    /// Create a new empty pending keycode state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            keycode_template: None,
            params: Vec::new(),
        }
    }

    /// Reset the pending state
    pub fn reset(&mut self) {
        self.keycode_template = None;
        self.params.clear();
    }

    /// Build the final keycode string from collected parameters
    #[must_use]
    pub fn build_keycode(&self) -> Option<String> {
        // Data-driven approach: use template + params array
        if let Some(template) = &self.keycode_template {
            if !self.params.is_empty() {
                use crate::keycode_db::KeycodeDb;
                let prefix = KeycodeDb::get_prefix(template)?;
                let params_str = self.params.join(", ");
                return Some(format!("{prefix}({params_str})"));
            }
        }
        None
    }
}

/// Origin for tap dance form flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapDanceFormContext {
    /// Launched from tap dance editor list (manage library only)
    FromEditor,
    /// Launched from keycode picker to assign TD(name) to selected key
    FromKeycodePicker,
}

/// State for the template save dialog.
#[derive(Debug, Clone)]
pub struct TemplateSaveDialogState {
    /// Current input field
    pub active_field: TemplateSaveField,
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template author
    pub author: String,
    /// Template tags (comma-separated input)
    pub tags_input: String,
}

impl TemplateSaveDialogState {
    /// Creates a new template save dialog state with pre-filled values from current layout.
    #[must_use]
    pub const fn new(layout_name: String) -> Self {
        Self {
            active_field: TemplateSaveField::Name,
            name: layout_name,
            description: String::new(),
            author: String::new(),
            tags_input: String::new(),
        }
    }

    /// Get the active field's input string (mutable).
    pub const fn get_active_field_mut(&mut self) -> &mut String {
        match self.active_field {
            TemplateSaveField::Name => &mut self.name,
            TemplateSaveField::Description => &mut self.description,
            TemplateSaveField::Author => &mut self.author,
            TemplateSaveField::Tags => &mut self.tags_input,
        }
    }

    /// Move to the next field.
    pub const fn next_field(&mut self) {
        self.active_field = match self.active_field {
            TemplateSaveField::Name => TemplateSaveField::Description,
            TemplateSaveField::Description => TemplateSaveField::Author,
            TemplateSaveField::Author => TemplateSaveField::Tags,
            TemplateSaveField::Tags => TemplateSaveField::Name,
        };
    }

    /// Move to the previous field.
    pub const fn previous_field(&mut self) {
        self.active_field = match self.active_field {
            TemplateSaveField::Name => TemplateSaveField::Tags,
            TemplateSaveField::Description => TemplateSaveField::Name,
            TemplateSaveField::Author => TemplateSaveField::Description,
            TemplateSaveField::Tags => TemplateSaveField::Author,
        };
    }

    /// Parse tags from comma-separated input.
    #[must_use]
    pub fn parse_tags(&self) -> Vec<String> {
        self.tags_input
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

impl Default for TemplateSaveDialogState {
    fn default() -> Self {
        Self::new("Untitled Template".to_string())
    }
}

/// Fields in the template save dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSaveField {
    /// Template name field
    Name,
    /// Template description field
    Description,
    /// Template author field
    Author,
    /// Template tags field
    Tags,
}

/// Popup types that can be displayed over the main UI
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
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

/// Selection mode for multi-key operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionMode {
    /// Normal selection - adding/removing individual keys
    Normal,
    /// Rectangle selection - selecting a block of keys
    Rectangle {
        /// Starting corner of the rectangle
        start: Position,
    },
}

/// Active component - holds the currently active popup component
///
/// This enum wraps all component types that implement the Component or ContextualComponent trait.
/// Only one component can be active at a time.
#[derive(Debug)]
#[allow(dead_code)]
pub enum ActiveComponent {
    /// Color picker component
    ColorPicker(ColorPicker),
    /// Keycode picker component
    KeycodePicker(KeycodePicker),
    /// Layer picker component
    LayerPicker(LayerPicker),
    /// Category picker component
    CategoryPicker(CategoryPicker),
    /// Modifier picker component
    ModifierPicker(ModifierPicker),
    /// Category manager component
    CategoryManager(CategoryManager),
    /// Layer manager component
    LayerManager(LayerManager),
    /// Metadata editor component
    MetadataEditor(metadata_editor::MetadataEditor),
    /// Template browser component
    TemplateBrowser(TemplateBrowser),
    /// Layout picker component (for loading saved layouts)
    LayoutPicker(LayoutPicker),
    /// Tap dance editor component
    TapDanceEditor(tap_dance_editor::TapDanceEditor),
    /// Tap dance form component
    TapDanceForm(tap_dance_form::TapDanceForm),
    /// Build log component
    BuildLog(BuildLog),
    /// Help overlay component
    HelpOverlay(HelpOverlay),
    /// Settings manager component
    SettingsManager(settings_manager::SettingsManager),
    /// Layout variant picker component (for switching QMK layout variants)
    LayoutVariantPicker(LayoutVariantPicker),
}

/// Application state - single source of truth
///
/// All UI components read from this state immutably.
/// Only event handlers modify state explicitly.
pub struct AppState {
    // Core data
    /// Current keyboard layout
    pub layout: Layout,
    /// Path to source layout file
    pub source_path: Option<PathBuf>,
    /// Whether layout has unsaved changes
    pub dirty: bool,

    // UI state
    /// Current UI theme
    pub theme: Theme,
    /// Currently displayed layer index
    pub current_layer: usize,
    /// Currently selected key position
    pub selected_position: Position,
    /// Currently active popup (if any)
    pub active_popup: Option<PopupType>,
    /// Status bar message
    pub status_message: String,
    /// Optional color override for status message (warnings/errors rendered via status)
    pub status_color_override: Option<ratatui::style::Color>,
    /// Current error message (if any)
    pub error_message: Option<String>,

    // Active component (Component trait pattern - replaces individual state fields)
    /// Currently active component (if any)
    pub active_component: Option<ActiveComponent>,

    // Legacy component states (to be removed incrementally during migration)
    /// Context for category picker (what's being categorized)
    pub category_picker_context: Option<CategoryPickerContext>,
    /// Category manager component state (preserved across color picker interactions)
    /// NOTE: Unlike fully migrated components, this state is synced with the component
    /// to preserve mode/selection when color picker opens (replacing the component)
    pub category_manager_state: CategoryManagerState,
    /// Template save dialog component state
    pub template_save_dialog_state: TemplateSaveDialogState,
    /// Setup wizard component state
    pub wizard_state: onboarding_wizard::OnboardingWizardState,
    /// Pending parameterized keycode state (for multi-stage keycode building)
    pub pending_keycode: PendingKeycodeState,
    /// Tap dance form cache (preserved when opening picker)
    pub tap_dance_form_cache: Option<tap_dance_form::TapDanceForm>,
    /// Tap dance form picker target (which field is being picked)
    pub tap_dance_form_pick_target: Option<tap_dance_form::FormRow>,
    /// Tap dance form context (where flow was launched from)
    pub tap_dance_form_context: Option<TapDanceFormContext>,
    /// Key editor component state
    pub key_editor_state: KeyEditorState,
    /// Key clipboard for copy/cut/paste operations
    pub clipboard: clipboard::KeyClipboard,
    /// Flash highlight position (for paste feedback) - (layer, position, `remaining_frames`)
    pub flash_highlight: Option<(usize, Position, u8)>,
    /// Visual selection mode for multi-key operations
    pub selection_mode: Option<SelectionMode>,
    /// Selected keys in selection mode (positions on current layer)
    pub selected_keys: Vec<Position>,

    // System resources
    /// Keycode database
    pub keycode_db: KeycodeDb,
    /// Keyboard physical geometry
    pub geometry: KeyboardGeometry,
    /// Visual-to-matrix position mapping
    pub mapping: VisualLayoutMapping,
    /// Application configuration
    pub config: Config,

    // Firmware build state
    /// Current firmware build state (if building)
    pub build_state: Option<BuildState>,

    // Layer reference tracking
    /// Index of layer references (which keys on which layers reference this layer)
    /// Key: target layer index, Value: list of references to that layer
    pub layer_refs: HashMap<usize, Vec<LayerRef>>,

    // Control flags
    /// Whether application should exit
    pub should_quit: bool,
    /// Whether to return to settings manager after picker closes
    pub return_to_settings_after_picker: bool,
}

impl AppState {
    /// Extracts the base keyboard name from a keyboard path that may include a variant.
    ///
    /// # Examples
    ///
    /// - `"keebart/corne_choc_pro/standard"` → `"keebart/corne_choc_pro"`
    /// - `"keebart/corne_choc_pro"` → `"keebart/corne_choc_pro"`
    /// - `"crkbd"` → `"crkbd"`
    #[must_use]
    pub fn extract_base_keyboard(keyboard_path: &str) -> String {
        extract_base_keyboard(keyboard_path)
    }

    /// Creates a new `AppState` from config, layout, and keyboard geometry.
    ///
    /// # Errors
    ///
    /// Returns error if config or layout validation fails
    pub fn new(
        layout: Layout,
        source_path: Option<PathBuf>,
        geometry: KeyboardGeometry,
        mapping: VisualLayoutMapping,
        config: Config,
    ) -> Result<Self> {
        let keycode_db = KeycodeDb::load().context("Failed to load keycode database")?;
        let theme = Theme::detect();

        // Initialize selected position to first valid key position
        let selected_position = mapping
            .get_first_position()
            .unwrap_or(Position { row: 0, col: 0 });

        // Build initial layer reference index
        let layer_refs = build_layer_ref_index(&layout.layers);

        Ok(Self {
            layout,
            source_path,
            dirty: false,
            theme,
            current_layer: 0,
            selected_position,
            active_popup: None,
            status_message: "Press ? for help".to_string(),
            status_color_override: None,
            error_message: None,
            active_component: None,
            category_picker_context: None,
            category_manager_state: CategoryManagerState::new(),
            template_save_dialog_state: TemplateSaveDialogState::default(),
            wizard_state: onboarding_wizard::OnboardingWizardState::new(),
            pending_keycode: PendingKeycodeState::new(),
            tap_dance_form_cache: None,
            tap_dance_form_pick_target: None,
            tap_dance_form_context: None,
            key_editor_state: KeyEditorState::new(),

            clipboard: clipboard::KeyClipboard::new(),
            flash_highlight: None,
            selection_mode: None,
            selected_keys: Vec::new(),
            keycode_db,
            geometry,
            mapping,
            config,
            build_state: None,
            layer_refs,
            should_quit: false,
            return_to_settings_after_picker: false,
        })
    }

    /// Get the currently selected key (mutable)
    pub fn get_selected_key_mut(&mut self) -> Option<&mut crate::models::KeyDefinition> {
        let layer = self.layout.layers.get_mut(self.current_layer)?;
        layer
            .keys
            .iter_mut()
            .find(|k| k.position == self.selected_position)
    }

    /// Get the currently selected key (immutable)
    #[must_use]
    pub fn get_selected_key(&self) -> Option<&crate::models::KeyDefinition> {
        let layer = self.layout.layers.get(self.current_layer)?;
        layer
            .keys
            .iter()
            .find(|k| k.position == self.selected_position)
    }

    /// Mark layout as dirty (unsaved changes)
    pub const fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clear dirty flag (after save)
    pub const fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Returns a reference to the current theme
    #[must_use]
    #[allow(dead_code)]
    pub const fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Rebuild keyboard geometry and visual layout mapping for a new layout variant.
    ///
    /// This also automatically determines and updates the keyboard variant subdirectory
    /// (e.g., "standard", "mini") based on the selected layout's characteristics.
    ///
    /// # Arguments
    ///
    /// * `layout_name` - Name of the layout variant (e.g., "`LAYOUT_split_3x6_3`")
    ///
    /// # Returns
    ///
    /// Result indicating success or error with context
    pub fn rebuild_geometry(&mut self, layout_name: &str) -> Result<()> {
        // Build geometry context
        let geo_context = GeometryContext {
            config: &self.config,
            metadata: &self.layout.metadata,
        };

        // Build geometry using the service
        let geo_result = build_geometry_for_layout(geo_context, layout_name)?;

        // Update AppState with new geometry and mapping
        self.geometry = geo_result.geometry;
        self.mapping = geo_result.mapping;

        // Store the layout variant in the layout metadata for persistence
        self.layout.metadata.layout_variant = Some(layout_name.to_string());

        // Update keyboard variant path
        self.layout.metadata.keyboard = Some(geo_result.variant_path);

        // Adjust all layers to match new geometry
        // Add KC_NO keys for new positions, keep existing keys where they still fit
        self.adjust_layers_to_geometry()?;

        // Reset selection to first valid position
        self.selected_position = self
            .mapping
            .get_first_position()
            .unwrap_or(Position { row: 0, col: 0 });

        Ok(())
    }

    /// Adjusts all layers to match the current geometry.
    ///
    /// This ensures that:
    /// - All key positions in the geometry have corresponding keys in each layer
    /// - Keys are added as `KC_NO` for new positions
    /// - Existing keys at valid positions are preserved
    ///
    /// Call this after loading a layout to ensure keys match the geometry.
    pub fn adjust_layers_to_geometry(&mut self) -> Result<()> {
        use crate::models::layer::KeyDefinition;

        // Get all valid positions from the mapping
        let valid_positions: std::collections::HashSet<Position> = self
            .mapping
            .get_all_visual_positions()
            .into_iter()
            .collect();

        // Adjust each layer
        for layer in &mut self.layout.layers {
            // Keep only keys that are still in valid positions
            layer
                .keys
                .retain(|key| valid_positions.contains(&key.position));

            // Find which positions are missing
            let existing_positions: std::collections::HashSet<Position> =
                layer.keys.iter().map(|k| k.position).collect();

            let missing_positions: Vec<Position> = valid_positions
                .iter()
                .filter(|pos| !existing_positions.contains(pos))
                .copied()
                .collect();

            // Add KC_NO keys for missing positions
            for pos in missing_positions {
                layer.add_key(KeyDefinition::new(pos, "KC_NO"));
            }
        }

        Ok(())
    }

    /// Set status message
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
        self.error_message = None;
        self.status_color_override = None;
    }

    /// Set status message with custom foreground color (used for warnings)
    pub fn set_status_with_style(
        &mut self,
        message: impl Into<String>,
        color: ratatui::style::Color,
    ) {
        self.status_message = message.into();
        self.error_message = None;
        self.status_color_override = Some(color);
    }

    /// Set error message
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error_message = Some(error.into());
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Refresh the layer reference index after layer changes
    ///
    /// Call this after operations that add/remove/modify layer-switching keycodes:
    /// - Assigning a layer keycode (MO, LT, TG, etc.)
    /// - Deleting a layer keycode
    /// - Pasting keys that may contain layer keycodes
    /// - Adding/removing layers
    pub fn refresh_layer_refs(&mut self) {
        self.layer_refs = build_layer_ref_index(&self.layout.layers);
    }

    // === Component Management Methods (Component Trait Pattern) ===

    /// Open the color picker component
    pub fn open_color_picker(
        &mut self,
        context: component::ColorPickerContext,
        color: crate::models::RgbColor,
    ) {
        let picker = ColorPicker::new(context, color);
        self.active_component = Some(ActiveComponent::ColorPicker(picker));
        self.active_popup = Some(PopupType::ColorPicker);
    }

    /// Open the keycode picker component
    pub fn open_keycode_picker(&mut self) {
        let picker =
            KeycodePicker::with_language(self.config.ui.last_language.clone(), &self.keycode_db);
        self.active_component = Some(ActiveComponent::KeycodePicker(picker));
        self.active_popup = Some(PopupType::KeycodePicker);
    }

    /// Open the layer picker component
    pub fn open_layer_picker(&mut self, keycode_type: &str) {
        let picker = LayerPicker::new(keycode_type);
        self.active_component = Some(ActiveComponent::LayerPicker(picker));
        self.active_popup = Some(PopupType::LayerPicker);
    }

    /// Open the modifier picker component
    pub fn open_modifier_picker(&mut self) {
        let picker = ModifierPicker::new();
        self.active_component = Some(ActiveComponent::ModifierPicker(picker));
        self.active_popup = Some(PopupType::ModifierPicker);
    }

    /// Open the category manager component
    pub fn open_category_manager(&mut self) {
        self.category_manager_state.reset();
        let manager = CategoryManager::new(self.layout.categories.clone());
        self.active_component = Some(ActiveComponent::CategoryManager(manager));
        self.active_popup = Some(PopupType::CategoryManager);
    }

    /// Open the layer manager component
    pub fn open_layer_manager(&mut self) {
        let manager = LayerManager::new(self.layout.layers.clone(), self.current_layer);
        self.active_component = Some(ActiveComponent::LayerManager(manager));
        self.active_popup = Some(PopupType::LayerManager);
    }

    /// Open the metadata editor component
    pub fn open_metadata_editor(&mut self) {
        let editor = metadata_editor::MetadataEditor::new(&self.layout.metadata);
        self.active_component = Some(ActiveComponent::MetadataEditor(editor));
        self.active_popup = Some(PopupType::MetadataEditor);
    }

    /// Open the template browser component
    pub fn open_template_browser(&mut self) {
        let browser = TemplateBrowser::new();
        self.active_component = Some(ActiveComponent::TemplateBrowser(browser));
        self.active_popup = Some(PopupType::TemplateBrowser);
    }

    /// Open the layout variant picker component (for switching QMK keyboard layout variants)
    pub fn open_layout_variant_picker(&mut self, qmk_path: &PathBuf, keyboard: &str) -> Result<()> {
        let picker = LayoutVariantPicker::new(qmk_path, keyboard);
        self.active_component = Some(ActiveComponent::LayoutVariantPicker(picker));
        self.active_popup = Some(PopupType::LayoutPicker);
        Ok(())
    }

    /// Open the build log component
    pub fn open_build_log(&mut self) {
        let log = BuildLog::new();
        self.active_component = Some(ActiveComponent::BuildLog(log));
        self.active_popup = Some(PopupType::BuildLog);
    }

    /// Open the help overlay component
    pub fn open_help_overlay(&mut self) {
        let help = HelpOverlay::new();
        self.active_component = Some(ActiveComponent::HelpOverlay(help));
        self.active_popup = Some(PopupType::HelpOverlay);
    }

    /// Open the settings manager component
    pub fn open_settings_manager(&mut self) {
        let manager = settings_manager::SettingsManager::new();
        self.active_component = Some(ActiveComponent::SettingsManager(manager));
        self.active_popup = Some(PopupType::SettingsManager);
    }

    /// Open the tap dance editor component
    pub fn open_tap_dance_editor(&mut self) {
        let editor = tap_dance_editor::TapDanceEditor::new(&self.layout);
        self.active_component = Some(ActiveComponent::TapDanceEditor(editor));
        self.active_popup = Some(PopupType::TapDanceEditor);
    }

    /// Close the currently active component
    pub fn close_component(&mut self) {
        self.active_component = None;
        self.active_popup = None;
    }

    // === Tap Dance Management Methods ===
}

/// Initialize terminal for TUI
pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).context("Failed to create terminal")?;
    Ok(terminal)
}

/// Restore terminal to normal state
pub fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;
    Ok(())
}

/// Main event loop
pub fn run_tui(
    state: &mut AppState,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<()> {
    loop {
        // Apply theme based on user preference (Auto detects OS, Dark/Light are explicit)
        state.theme = Theme::from_mode(state.config.ui.theme_mode);

        // Decrement flash highlight counter
        if let Some((layer, pos, frames)) = state.flash_highlight {
            if frames > 1 {
                state.flash_highlight = Some((layer, pos, frames - 1));
            } else {
                state.flash_highlight = None;
            }
        }

        // Render current state
        terminal.draw(|f| render(f, state))?;

        // Poll for events with 100ms timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if handle_key_event(state, key)? {
                    break; // User quit
                }
            } else if let Event::Resize(_, _) = event::read()? {
                // Terminal resized, will re-render on next loop
            }
        }

        // Poll build state for updates
        if let Some(build_state) = &mut state.build_state {
            if build_state.poll() {
                // Build message received, will update on next render
            }
        }

        // Check if should quit
        if state.should_quit {
            break;
        }
    }

    Ok(())
}

/// Render the UI from current state
fn render(f: &mut Frame, state: &AppState) {
    // Fill entire screen with theme background color first
    // This ensures consistent background regardless of terminal settings
    let full_bg = Block::default().style(Style::default().bg(state.theme.background));
    f.render_widget(full_bg, f.area());

    let chunks = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title bar
            Constraint::Min(10),   // Main content
            Constraint::Length(6), // Status bar (increased for description + clipboard + build + help)
        ])
        .split(f.area());

    // Title bar with dirty indicator
    render_title_bar(f, chunks[0], state);

    // Main content area
    render_main_content(f, chunks[1], state);

    // Status bar
    StatusBar::render(f, chunks[2], state, &state.theme);

    // Render popup if active
    if let Some(popup_type) = &state.active_popup {
        render_popup(f, popup_type, state);
    }

    // Render error overlay on top of everything if error is present
    if let Some(ref error) = state.error_message {
        render_error_overlay(f, error, &state.theme);
    }
}

/// Render title bar with layout name and dirty indicator
fn render_title_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let dirty_indicator = if state.dirty { " *" } else { "" };
    let title = format!(
        " {} - Layer {} {}",
        state.layout.metadata.name, state.current_layer, dirty_indicator
    );

    let title_widget = Paragraph::new(title)
        .style(
            Style::default()
                .fg(state.theme.primary)
                .bg(state.theme.background),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(state.theme.background)),
        );

    f.render_widget(title_widget, area);
}

/// Render main content (keyboard widget)
fn render_main_content(f: &mut Frame, area: Rect, state: &AppState) {
    KeyboardWidget::render(f, area, state);
}

/// Render active popup
fn render_popup(f: &mut Frame, popup_type: &PopupType, state: &AppState) {
    match popup_type {
        PopupType::KeycodePicker => {
            // Use ContextualComponent trait pattern
            if let Some(ActiveComponent::KeycodePicker(ref picker)) = state.active_component {
                picker.render(f, f.area(), &state.theme, &state.keycode_db);
            }
        }
        PopupType::ColorPicker => {
            // Use Component trait pattern
            if let Some(ActiveComponent::ColorPicker(ref picker)) = state.active_component {
                picker.render(f, f.area(), &state.theme);
            }
        }
        PopupType::CategoryPicker => {
            // Use ContextualComponent trait pattern
            if let Some(ActiveComponent::CategoryPicker(ref picker)) = state.active_component {
                picker.render(f, f.area(), &state.theme, &state.layout.categories);
            }
        }
        PopupType::CategoryManager => {
            // Use Component trait pattern
            if let Some(ActiveComponent::CategoryManager(ref manager)) = state.active_component {
                manager.render(f, f.area(), &state.theme);
            }
        }
        PopupType::LayerManager => {
            // Use Component trait pattern
            if let Some(ActiveComponent::LayerManager(ref manager)) = state.active_component {
                manager.render(f, f.area(), &state.theme);
            }
        }
        PopupType::LayerPicker => {
            // Use ContextualComponent trait pattern
            if let Some(ActiveComponent::LayerPicker(ref picker)) = state.active_component {
                picker.render(f, f.area(), &state.theme, &state.layout.layers);
            }
        }
        PopupType::TemplateBrowser => {
            if let Some(ActiveComponent::TemplateBrowser(ref browser)) = state.active_component {
                browser.render(f, f.area(), &state.theme);
            }
        }
        PopupType::TemplateSaveDialog => {
            render_template_save_dialog(f, state);
        }
        PopupType::UnsavedChangesPrompt => {
            render_unsaved_prompt(f, &state.theme);
        }
        PopupType::BuildLog => {
            // Use ContextualComponent trait pattern
            if let Some(ActiveComponent::BuildLog(ref log)) = state.active_component {
                if let Some(ref build_state) = state.build_state {
                    log.render(f, f.area(), &state.theme, build_state);
                }
            }
        }
        PopupType::HelpOverlay => {
            if let Some(ActiveComponent::HelpOverlay(ref help)) = state.active_component {
                help.render(f, f.area(), &state.theme);
            }
        }
        PopupType::LayoutPicker => {
            if let Some(ActiveComponent::LayoutVariantPicker(ref picker)) = state.active_component {
                picker.render(f, f.area(), &state.theme);
            }
        }
        PopupType::MetadataEditor => {
            if let Some(ActiveComponent::MetadataEditor(ref editor)) = state.active_component {
                editor.render(f, f.area(), &state.theme);
            }
        }
        PopupType::SetupWizard => {
            onboarding_wizard::render(f, &state.wizard_state, &state.theme);
        }
        PopupType::SettingsManager => {
            if let Some(ActiveComponent::SettingsManager(ref manager)) = state.active_component {
                let context = settings_manager::SettingsManagerContext {
                    rgb_enabled: state.layout.rgb_enabled,
                    rgb_brightness: state.layout.rgb_brightness,
                    rgb_timeout_ms: state.layout.rgb_timeout_ms,
                    uncolored_key_behavior: state.layout.uncolored_key_behavior,
                    idle_effect_settings: state.layout.idle_effect_settings.clone(),
                    tap_hold_settings: state.layout.tap_hold_settings.clone(),
                    config: state.config.clone(),
                    layout: state.layout.clone(),
                };
                manager.render_with_context(f, f.area(), &state.theme, &context);
            }
        }
        PopupType::TapKeycodePicker => {
            // Use component-based rendering (same as KeycodePicker)
            if let Some(ActiveComponent::KeycodePicker(ref picker)) = state.active_component {
                picker.render(f, f.area(), &state.theme, &state.keycode_db);
            }
        }
        PopupType::ModifierPicker => {
            if let Some(ActiveComponent::ModifierPicker(ref picker)) = state.active_component {
                picker.render(f, f.area(), &state.theme);
            }
        }
        PopupType::KeyEditor => {
            key_editor::render_key_editor(f, state);
        }
        PopupType::TapDanceEditor => {
            if let Some(ActiveComponent::TapDanceEditor(ref editor)) = state.active_component {
                editor.render(f, f.area(), &state.theme);
            }
        }
        PopupType::TapDanceForm => {
            if let Some(ActiveComponent::TapDanceForm(ref form)) = state.active_component {
                form.render(f, f.area(), &state.theme);
            }
        }
    }
}

/// Render unsaved changes prompt
fn render_unsaved_prompt(f: &mut Frame, theme: &Theme) {
    let area = centered_rect(60, 30, f.area());

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    let text = vec![
        Line::from(""),
        Line::from("You have unsaved changes."),
        Line::from(""),
        Line::from("  [S] Save and quit"),
        Line::from("  [Q] Quit without saving"),
        Line::from("  [Esc] Cancel"),
    ];

    let prompt = Paragraph::new(text).block(
        Block::default()
            .title(" Unsaved Changes ")
            .borders(Borders::ALL)
            .style(Style::default().fg(theme.warning)),
    );

    f.render_widget(prompt, area);
}

/// Render error overlay on top of all other UI elements
fn render_error_overlay(f: &mut Frame, error: &str, theme: &Theme) {
    let area = centered_rect(70, 40, f.area());

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background with error color
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    // Split into title and message
    let chunks = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(3),    // Error message
            Constraint::Length(2), // Help text
        ])
        .split(area);

    // Title with error styling
    let title = Paragraph::new("ERROR")
        .style(
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(theme.error).bg(theme.background)),
        );
    f.render_widget(title, chunks[0]);

    // Error message with word wrap
    let error_text = Paragraph::new(error)
        .style(Style::default().fg(theme.text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Details ")
                .style(Style::default().bg(theme.background)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(error_text, chunks[1]);

    // Help text
    let help = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "Enter/Esc",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Dismiss"),
    ])])
    .style(Style::default().fg(theme.text).bg(theme.background))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(theme.background)),
    );
    f.render_widget(help, chunks[2]);
}

/// Render template save dialog
fn render_template_save_dialog(f: &mut Frame, state: &AppState) {
    let area = centered_rect(70, 60, f.area());

    let dialog_state = &state.template_save_dialog_state;
    let theme = &state.theme;

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    // Split into fields
    let chunks = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Name field
            Constraint::Length(3), // Description field
            Constraint::Length(3), // Author field
            Constraint::Length(3), // Tags field
            Constraint::Min(2),    // Help text
            Constraint::Length(2), // Action buttons
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Save as Template")
        .style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Name field
    let name_style = if matches!(dialog_state.active_field, TemplateSaveField::Name) {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.text)
    };
    let name_text = if matches!(dialog_state.active_field, TemplateSaveField::Name) {
        format!("Name: {}█", dialog_state.name)
    } else {
        format!("Name: {}", dialog_state.name)
    };
    let name = Paragraph::new(name_text)
        .style(name_style)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(name, chunks[1]);

    // Description field
    let desc_style = if matches!(dialog_state.active_field, TemplateSaveField::Description) {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.text)
    };
    let desc_text = if matches!(dialog_state.active_field, TemplateSaveField::Description) {
        format!("Description: {}█", dialog_state.description)
    } else {
        format!("Description: {}", dialog_state.description)
    };
    let description = Paragraph::new(desc_text)
        .style(desc_style)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(description, chunks[2]);

    // Author field
    let author_style = if matches!(dialog_state.active_field, TemplateSaveField::Author) {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.text)
    };
    let author_text = if matches!(dialog_state.active_field, TemplateSaveField::Author) {
        format!("Author: {}█", dialog_state.author)
    } else {
        format!("Author: {}", dialog_state.author)
    };
    let author = Paragraph::new(author_text)
        .style(author_style)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(author, chunks[3]);

    // Tags field
    let tags_style = if matches!(dialog_state.active_field, TemplateSaveField::Tags) {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.text)
    };
    let tags_text = if matches!(dialog_state.active_field, TemplateSaveField::Tags) {
        format!("Tags (comma-separated): {}█", dialog_state.tags_input)
    } else {
        format!("Tags (comma-separated): {}", dialog_state.tags_input)
    };
    let tags = Paragraph::new(tags_text)
        .style(tags_style)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(tags, chunks[4]);

    // Help text
    let help_text = vec![
        Line::from(""),
        Line::from("Tab/Shift+Tab: navigate fields"),
        Line::from("Type: enter text | Backspace: delete"),
    ];
    let help = Paragraph::new(help_text).style(Style::default().fg(theme.text_muted));
    f.render_widget(help, chunks[5]);

    // Action buttons
    let actions = Paragraph::new("Enter: save template | Esc: cancel")
        .style(Style::default().fg(theme.success))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(actions, chunks[6]);
}

/// Helper to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Handle keyboard input events
fn handle_key_event(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    use crossterm::event::KeyCode;

    // If error overlay is shown, allow dismissing with Enter or Esc
    if state.error_message.is_some() {
        if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
            state.clear_error();
            return Ok(false);
        }
        // Block all other input while error is shown
        return Ok(false);
    }

    // Route to popup handler if popup is active
    if state.active_popup.is_some() {
        return handlers::handle_popup_input(state, key);
    }

    // Main UI key handling
    handlers::handle_main_input(state, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_base_keyboard_no_variant() {
        let result = AppState::extract_base_keyboard("crkbd");
        assert_eq!(result, "crkbd");
    }

    #[test]
    fn test_extract_base_keyboard_with_manufacturer() {
        let result = AppState::extract_base_keyboard("keebart/corne_choc_pro");
        assert_eq!(result, "keebart/corne_choc_pro");
    }

    #[test]
    fn test_extract_base_keyboard_standard_variant() {
        let result = AppState::extract_base_keyboard("keebart/corne_choc_pro/standard");
        assert_eq!(result, "keebart/corne_choc_pro");
    }

    #[test]
    fn test_extract_base_keyboard_mini_variant() {
        let result = AppState::extract_base_keyboard("keebart/corne_choc_pro/mini");
        assert_eq!(result, "keebart/corne_choc_pro");
    }

    #[test]
    fn test_extract_base_keyboard_normal_variant() {
        let result = AppState::extract_base_keyboard("manufacturer/keyboard/normal");
        assert_eq!(result, "manufacturer/keyboard");
    }

    #[test]
    fn test_extract_base_keyboard_full_variant() {
        let result = AppState::extract_base_keyboard("manufacturer/keyboard/full");
        assert_eq!(result, "manufacturer/keyboard");
    }

    #[test]
    fn test_extract_base_keyboard_compact_variant() {
        let result = AppState::extract_base_keyboard("manufacturer/keyboard/compact");
        assert_eq!(result, "manufacturer/keyboard");
    }

    #[test]
    fn test_extract_base_keyboard_non_variant_subdirectory() {
        // "custom" is not a recognized variant pattern, so it should be kept
        let result = AppState::extract_base_keyboard("manufacturer/keyboard/custom");
        assert_eq!(result, "manufacturer/keyboard/custom");
    }

    #[test]
    fn test_extract_base_keyboard_revision_variant() {
        // "rev2" IS recognized as a variant pattern (starts with "rev"), so it's stripped
        let result = AppState::extract_base_keyboard("manufacturer/keyboard/rev2");
        assert_eq!(result, "manufacturer/keyboard");
    }

    #[test]
    fn test_extract_base_keyboard_deep_path_with_variant() {
        let result = AppState::extract_base_keyboard("a/b/c/standard");
        assert_eq!(result, "a/b/c");
    }

    // === PendingKeycodeState Tests ===

    #[test]
    fn test_pending_keycode_new() {
        let state = PendingKeycodeState::new();
        assert!(state.keycode_template.is_none());
        assert!(state.params.is_empty());
    }

    #[test]
    fn test_pending_keycode_reset() {
        let mut state = PendingKeycodeState::new();
        state.keycode_template = Some("LT()".to_string());
        state.params.push("@layer-uuid".to_string());
        state.params.push("KC_SPC".to_string());

        state.reset();

        assert!(state.keycode_template.is_none());
        assert!(state.params.is_empty());
    }

    #[test]
    fn test_build_keycode_layer_tap() {
        let mut state = PendingKeycodeState::new();
        state.keycode_template = Some("LT()".to_string());
        state.params.push("@abc-123".to_string());
        state.params.push("KC_SPC".to_string());

        let result = state.build_keycode();
        assert_eq!(result, Some("LT(@abc-123, KC_SPC)".to_string()));
    }

    #[test]
    fn test_build_keycode_mod_tap() {
        let mut state = PendingKeycodeState::new();
        state.keycode_template = Some("MT()".to_string());
        state.params.push("MOD_LCTL | MOD_LSFT".to_string());
        state.params.push("KC_A".to_string());

        let result = state.build_keycode();
        assert_eq!(result, Some("MT(MOD_LCTL | MOD_LSFT, KC_A)".to_string()));
    }

    #[test]
    fn test_build_keycode_layer_mod() {
        let mut state = PendingKeycodeState::new();
        state.keycode_template = Some("LM()".to_string());
        state.params.push("@layer-uuid".to_string());
        state.params.push("MOD_LSFT".to_string());

        let result = state.build_keycode();
        assert_eq!(result, Some("LM(@layer-uuid, MOD_LSFT)".to_string()));
    }

    #[test]
    fn test_build_keycode_swap_hands_tap() {
        let mut state = PendingKeycodeState::new();
        state.keycode_template = Some("SH_T()".to_string());
        state.params.push("KC_SPC".to_string());

        let result = state.build_keycode();
        assert_eq!(result, Some("SH_T(KC_SPC)".to_string()));
    }

    #[test]
    fn test_build_keycode_incomplete_lt() {
        let mut state = PendingKeycodeState::new();
        state.keycode_template = Some("LT()".to_string());
        state.params.push("@abc-123".to_string());
        // Missing second parameter

        let result = state.build_keycode();
        // Should still build with one param (though it's invalid QMK)
        assert_eq!(result, Some("LT(@abc-123)".to_string()));
    }

    #[test]
    fn test_build_keycode_no_template() {
        let state = PendingKeycodeState::new();
        let result = state.build_keycode();
        assert!(result.is_none(), "No template should return None");
    }

    #[test]
    fn test_build_keycode_empty_params() {
        let mut state = PendingKeycodeState::new();
        state.keycode_template = Some("LT()".to_string());
        // No params added

        let result = state.build_keycode();
        assert!(result.is_none(), "Empty params should return None");
    }
}
