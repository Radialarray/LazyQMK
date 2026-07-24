//! Application state, active component enum, and supporting types.
//!
//! This is the single source of truth for all UI state, including the layout
//! being edited, active popups, selection state, and system resources.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::Config;
use crate::firmware::BuildState;
use crate::keycode_db::KeycodeDb;
use crate::models::{
    ComboAction, ComboDefinition, KeyboardGeometry, Layout, Position, VisualLayoutMapping,
};
use crate::services::geometry::{
    build_geometry_for_layout, extract_base_keyboard, GeometryContext,
};
use crate::services::layer_refs::{build_layer_ref_index, LayerRef};
use crate::tui::build_log::BuildLog;
use crate::tui::category_manager::{CategoryManager, CategoryManagerState};
use crate::tui::category_picker::CategoryPicker;
use crate::tui::clipboard;
use crate::tui::color_picker::ColorPicker;
use crate::tui::component;
use crate::tui::config_dialogs::LayoutPicker as LayoutVariantPicker;
use crate::tui::editor::key_editor::KeyEditorState;
use crate::tui::help_overlay::HelpOverlay;
use crate::tui::keycode_picker::KeycodePicker;
use crate::tui::layer_manager::LayerManager;
use crate::tui::layer_picker::LayerPicker;
use crate::tui::layout_picker::LayoutPicker;
use crate::tui::metadata_editor;
use crate::tui::modifier_picker::ModifierPicker;
use crate::tui::onboarding_wizard;
use crate::tui::template_browser::TemplateBrowser;
use crate::tui::theme::Theme;
use crate::tui::PopupType;

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
    /// The original keycode template (e.g., "`LT()`", "`MT()`", "`LCTL_T()`")
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

/// State for the export filename dialog.
#[derive(Debug, Clone)]
pub struct ExportFilenameDialogState {
    /// Export filename input
    pub filename: String,
}

impl ExportFilenameDialogState {
    /// Creates a new export filename dialog with a default filename.
    #[must_use]
    pub fn new(layout_name: &str) -> Self {
        let date = chrono::Local::now().format("%Y-%m-%d");
        let default_filename = format!("{layout_name}_export_{date}.md");
        Self {
            filename: default_filename,
        }
    }
}

impl Default for ExportFilenameDialogState {
    fn default() -> Self {
        Self::new("layout")
    }
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
    /// Swap mode - select two keys to swap all properties
    Swap {
        /// First key selected for swap
        first: Position,
    },
}

/// Active component - holds the currently active popup component
///
/// This enum wraps all component types that implement the Component or `ContextualComponent` trait.
/// Only one component can be active at a time.
#[derive(Debug)]
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
    #[allow(dead_code)] // bin/lib split: variant present in enum (legacy/disabled)
    LayoutPicker(LayoutPicker),
    /// Tap dance editor component
    TapDanceEditor(crate::tui::tap_dance_editor::TapDanceEditor),
    /// Tap dance form component
    TapDanceForm(crate::tui::tap_dance_form::TapDanceForm),
    /// Build log component
    BuildLog(BuildLog),
    /// Help overlay component
    HelpOverlay(HelpOverlay),
    /// Settings manager component
    SettingsManager(crate::tui::settings_manager::SettingsManager),
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

    // Cross-popup state — fields that bridge popup boundaries.
    //
    // The Component-trait migration (`active_component: Option<ActiveComponent>`) is
    // complete for popups that hold all their state internally. The fields below
    // remain because they cross popup boundaries — e.g. `category_manager_state` is
    // preserved across color picker interactions so opening a color picker mid-edit
    // doesn't drop the user's mode/selection.
    //
    // A full migration would require each popup to persist+restore state through a
    // dedicated bridge (e.g. `Component::suspend()` / `Component::restore()`). This is
    // large-scale refactoring — tracked in LazyQMK-aopx.5.5 as future work; the
    // fields below are intentionally retained.
    //
    /// Context for category picker (what's being categorized)
    pub category_picker_context: Option<CategoryPickerContext>,
    /// Category manager component state (preserved across color picker interactions)
    pub category_manager_state: CategoryManagerState,
    /// Template save dialog component state
    pub template_save_dialog_state: TemplateSaveDialogState,
    /// Export filename dialog component state
    pub export_filename_dialog_state: ExportFilenameDialogState,
    /// Setup wizard component state
    pub wizard_state: onboarding_wizard::OnboardingWizardState,
    /// Pending parameterized keycode state (for multi-stage keycode building)
    pub pending_keycode: PendingKeycodeState,
    /// Tap dance form cache (preserved when opening picker)
    pub tap_dance_form_cache: Option<crate::tui::tap_dance_form::TapDanceForm>,
    /// Tap dance form picker target (which field is being picked)
    pub tap_dance_form_pick_target: Option<crate::tui::tap_dance_form::FormRow>,
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
            export_filename_dialog_state: ExportFilenameDialogState::default(),
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

    /// Append a new combo entry with default settings.
    ///
    /// Returns silently if the layout already has the maximum number of combos.
    pub fn apply_add_combo(&mut self) {
        use crate::models::layout::combo::MAX_COMBOS;
        if self.layout.combo_settings.combos.len() >= MAX_COMBOS {
            self.set_error(format!("Maximum of {MAX_COMBOS} combos allowed"));
            return;
        }
        self.layout
            .combo_settings
            .combos
            .push(ComboDefinition::new(
                Position { row: 0, col: 0 },
                Position { row: 0, col: 0 },
                ComboAction::DisableEffects,
            ));
        self.mark_dirty();
    }

    /// Remove the combo entry at `idx` if it exists.
    pub fn apply_remove_combo(&mut self, idx: usize) {
        if idx < self.layout.combo_settings.combos.len() {
            self.layout.combo_settings.combos.remove(idx);
            self.mark_dirty();
        }
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
        let manager = crate::tui::settings_manager::SettingsManager::new();
        self.active_component = Some(ActiveComponent::SettingsManager(manager));
        self.active_popup = Some(PopupType::SettingsManager);
    }

    /// Open the tap dance editor component
    pub fn open_tap_dance_editor(&mut self) {
        let editor = crate::tui::tap_dance_editor::TapDanceEditor::new(&self.layout);
        self.active_component = Some(ActiveComponent::TapDanceEditor(editor));
        self.active_popup = Some(PopupType::TapDanceEditor);
    }

    /// Close the currently active component
    pub fn close_component(&mut self) {
        self.active_component = None;
        self.active_popup = None;
    }
}
