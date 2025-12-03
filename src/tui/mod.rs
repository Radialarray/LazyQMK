//! Terminal user interface components and state management.
//!
//! This module contains the main TUI loop, `AppState`, event handling,
//! and all UI widgets using Ratatui.

pub mod build_log;
pub mod category_manager;
pub mod category_picker;
pub mod clipboard;
pub mod color_picker;
pub mod config_dialogs;
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
pub mod template_browser;
pub mod theme;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout as RatatuiLayout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crate::config::Config;
use crate::firmware::BuildState;
use crate::keycode_db::KeycodeDb;
use crate::models::{KeyboardGeometry, Layout, Position, VisualLayoutMapping};

// Re-export TUI components
pub use category_manager::CategoryManagerState;
pub use category_picker::CategoryPickerState;
pub use color_picker::ColorPickerState;
pub use help_overlay::HelpOverlayState;
pub use key_editor::KeyEditorState;
pub use keyboard::KeyboardWidget;
pub use keycode_picker::KeycodePickerState;
pub use layer_manager::LayerManagerState;
pub use layer_picker::LayerPickerState;
pub use metadata_editor::MetadataEditorState;
pub use modifier_picker::ModifierPickerState;
pub use settings_manager::SettingsManagerState;
pub use status_bar::StatusBar;
pub use template_browser::TemplateBrowserState;
pub use theme::Theme;

/// Color picker context - what are we setting the color for?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorPickerContext {
    /// Setting color for individual key
    IndividualKey,
    /// Setting layer default color
    LayerDefault,
    /// Setting color for new or existing category
    Category,
}

/// Category picker context - what are we setting the category for?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CategoryPickerContext {
    /// Setting category for individual key
    IndividualKey,
    /// Setting category for entire layer
    Layer,
}

/// Type of parameterized keycode being built
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterizedKeycodeType {
    /// LT(layer, kc) - Hold for layer, tap for keycode
    LayerTap,
    /// MT(mod, kc) - Hold for modifier, tap for keycode (custom modifier combo)
    ModTap,
    /// LM(layer, mod) - Layer with modifier active
    LayerMod,
    /// SH_T(kc) - Hold to swap hands, tap for keycode
    SwapHandsTap,
    /// Simple mod-tap like LCTL_T(kc), LSFT_T(kc) - modifier is fixed, only need tap keycode
    /// param1 stores the prefix (e.g., "LCTL_T", "LGUI_T")
    SimpleModTap,
}

/// State for building parameterized keycodes through multi-stage picker flow
#[derive(Debug, Clone, Default)]
pub struct PendingKeycodeState {
    /// The keycode type being built (e.g., "LT", "MT", "LM")
    pub keycode_type: Option<ParameterizedKeycodeType>,
    /// First parameter (layer UUID for LT/LM, modifier bits for MT)
    pub param1: Option<String>,
    /// Second parameter (tap keycode for LT/MT, modifier for LM)
    pub param2: Option<String>,
}

impl PendingKeycodeState {
    /// Create a new empty pending keycode state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            keycode_type: None,
            param1: None,
            param2: None,
        }
    }

    /// Reset the pending state
    pub fn reset(&mut self) {
        self.keycode_type = None;
        self.param1 = None;
        self.param2 = None;
    }

    /// Build the final keycode string from collected parameters
    #[must_use]
    pub fn build_keycode(&self) -> Option<String> {
        match &self.keycode_type {
            Some(ParameterizedKeycodeType::LayerTap) => {
                let layer = self.param1.as_ref()?;
                let keycode = self.param2.as_ref()?;
                Some(format!("LT({}, {})", layer, keycode))
            }
            Some(ParameterizedKeycodeType::ModTap) => {
                let modifier = self.param1.as_ref()?;
                let keycode = self.param2.as_ref()?;
                Some(format!("MT({}, {})", modifier, keycode))
            }
            Some(ParameterizedKeycodeType::LayerMod) => {
                let layer = self.param1.as_ref()?;
                let modifier = self.param2.as_ref()?;
                Some(format!("LM({}, {})", layer, modifier))
            }
            Some(ParameterizedKeycodeType::SwapHandsTap) => {
                let keycode = self.param1.as_ref()?;
                Some(format!("SH_T({})", keycode))
            }
            Some(ParameterizedKeycodeType::SimpleModTap) => {
                // param1 is the prefix (e.g., "LCTL_T"), param2 is the tap keycode
                let prefix = self.param1.as_ref()?;
                let keycode = self.param2.as_ref()?;
                Some(format!("{}({})", prefix, keycode))
            }
            None => None,
        }
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
    /// Error message to display
    pub error_message: Option<String>,

    // Component states
    /// Keycode picker component state
    pub keycode_picker_state: KeycodePickerState,
    /// Color picker component state
    pub color_picker_state: ColorPickerState,
    /// Context for color picker (what's being colored)
    pub color_picker_context: Option<ColorPickerContext>,
    /// Category picker component state
    pub category_picker_state: CategoryPickerState,
    /// Context for category picker (what's being categorized)
    pub category_picker_context: Option<CategoryPickerContext>,
    /// Category manager component state
    pub category_manager_state: CategoryManagerState,
    /// Layer manager component state
    pub layer_manager_state: LayerManagerState,
    /// Build log component state
    pub build_log_state: build_log::BuildLogState,
    /// Template browser component state
    pub template_browser_state: TemplateBrowserState,
    /// Template save dialog component state
    pub template_save_dialog_state: TemplateSaveDialogState,
    /// Metadata editor component state
    pub metadata_editor_state: MetadataEditorState,
    /// Help overlay component state
    pub help_overlay_state: HelpOverlayState,
    /// Layout picker component state
    pub layout_picker_state: config_dialogs::LayoutPickerState,
    /// Setup wizard component state
    pub wizard_state: onboarding_wizard::OnboardingWizardState,
    /// Settings manager component state
    pub settings_manager_state: SettingsManagerState,
    /// Layer picker component state
    pub layer_picker_state: LayerPickerState,
    /// Pending parameterized keycode state (for multi-stage keycode building)
    pub pending_keycode: PendingKeycodeState,
    /// Modifier picker component state
    pub modifier_picker_state: ModifierPickerState,
    /// Key editor component state
    pub key_editor_state: KeyEditorState,
    /// Key clipboard for copy/cut/paste operations
    pub clipboard: clipboard::KeyClipboard,
    /// Flash highlight position (for paste feedback) - (layer, position, remaining_frames)
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
    pub fn extract_base_keyboard(keyboard_path: &str) -> String {
        let keyboard_parts: Vec<&str> = keyboard_path.split('/').collect();
        if keyboard_parts.len() > 2 
            && ["standard", "mini", "normal", "full", "compact"].contains(&keyboard_parts[keyboard_parts.len() - 1]) {
            // Has variant subdirectory - use parent path
            keyboard_parts[..keyboard_parts.len() - 1].join("/")
        } else {
            // No variant subdirectory
            keyboard_path.to_string()
        }
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
        let selected_position = mapping.get_first_position().unwrap_or(Position { row: 0, col: 0 });

        Ok(Self {
            layout,
            source_path,
            dirty: false,
            theme,
            current_layer: 0,
            selected_position,
            active_popup: None,
            status_message: "Press ? for help".to_string(),
            error_message: None,
            keycode_picker_state: KeycodePickerState::new(),
            color_picker_state: ColorPickerState::new(),
            color_picker_context: None,
            category_picker_state: CategoryPickerState::new(),
            category_picker_context: None,
            category_manager_state: CategoryManagerState::new(),
            layer_manager_state: LayerManagerState::new(),
            build_log_state: build_log::BuildLogState::new(),
            template_browser_state: TemplateBrowserState::new(),
            template_save_dialog_state: TemplateSaveDialogState::default(),
            metadata_editor_state: MetadataEditorState::default(),
            help_overlay_state: HelpOverlayState::new(),
            layout_picker_state: config_dialogs::LayoutPickerState::new(),
            wizard_state: onboarding_wizard::OnboardingWizardState::new(),
            settings_manager_state: SettingsManagerState::new(),
            layer_picker_state: LayerPickerState::new(),
            pending_keycode: PendingKeycodeState::new(),
            modifier_picker_state: ModifierPickerState::new(),
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
        use crate::models::VisualLayoutMapping;
        use crate::parser::keyboard_json::{build_keyboard_geometry_with_rgb, build_matrix_to_led_map, parse_keyboard_info_json, parse_variant_keyboard_json};

        // Parse keyboard info.json to get layout definition
        let qmk_path = self
            .config
            .paths
            .qmk_firmware
            .as_ref()
            .context("QMK firmware path not configured")?;

        // Get the base keyboard name (without any variant subdirectory)
        let base_keyboard = Self::extract_base_keyboard(&self.config.build.keyboard);
        
        let info = parse_keyboard_info_json(qmk_path, &base_keyboard)
            .context("Failed to parse keyboard info.json")?;

        // Get the key count for the selected layout to determine variant
        let layout_def = info.layouts.get(layout_name)
            .context(format!("Layout '{}' not found in keyboard info.json", layout_name))?;
        let key_count = layout_def.layout.len();

        // Determine keyboard variant first so we can look for RGB matrix config
        let variant_path = match self.config.build.determine_keyboard_variant(qmk_path, &base_keyboard, key_count) {
            Ok(path) => path,
            Err(_) => base_keyboard.to_string(),
        };

        // Try to get RGB matrix mapping from variant keyboard.json
        let matrix_to_led = parse_variant_keyboard_json(qmk_path, &variant_path)
            .and_then(|variant| variant.rgb_matrix)
            .map(|rgb_config| build_matrix_to_led_map(&rgb_config));

        // Build new geometry for selected layout with RGB matrix mapping if available
        let new_geometry = build_keyboard_geometry_with_rgb(&info, &base_keyboard, layout_name, matrix_to_led.as_ref())
            .context("Failed to build keyboard geometry")?;

        // Build new visual layout mapping
        let new_mapping = VisualLayoutMapping::build(&new_geometry);

        // Update config to persist layout choice
        self.config.build.layout = layout_name.to_string();

        // Update keyboard variant (already determined above for RGB matrix lookup)
        self.config.build.keyboard = variant_path;

        // Store the layout variant in the layout metadata for persistence
        self.layout.metadata.layout_variant = Some(layout_name.to_string());

        // Update AppState with new geometry and mapping
        self.geometry = new_geometry;
        self.mapping = new_mapping;

        // Adjust all layers to match new geometry
        // Add KC_NO keys for new positions, keep existing keys where they still fit
        self.adjust_layers_to_geometry()?;

        // Reset selection to first valid position
        self.selected_position = self.mapping.get_first_position().unwrap_or(Position { row: 0, col: 0 });

        Ok(())
    }

    /// Adjusts all layers to match the current geometry.
    ///
    /// This ensures that:
    /// - All key positions in the geometry have corresponding keys in each layer
    /// - Keys are added as KC_NO for new positions
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
            layer.keys.retain(|key| valid_positions.contains(&key.position));

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
    }

    /// Set error message
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error_message = Some(error.into());
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }
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
        // Re-detect OS theme on each loop iteration to respond to system theme changes
        state.theme = Theme::detect();

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
    let chunks = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title bar
            Constraint::Min(10),   // Main content
            Constraint::Length(6), // Status bar (increased for description + clipboard + build + help)
        ])
        .split(f.size());

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
}

/// Render title bar with layout name and dirty indicator
fn render_title_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let dirty_indicator = if state.dirty { " *" } else { "" };
    let title = format!(
        " {} - Layer {} {}",
        state.layout.metadata.name, state.current_layer, dirty_indicator
    );

    let title_widget = Paragraph::new(title)
        .style(Style::default().fg(state.theme.primary))
        .block(Block::default().borders(Borders::ALL));

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
            keycode_picker::render_keycode_picker(f, state);
        }
        PopupType::ColorPicker => {
            color_picker::render_color_picker(f, state);
        }
        PopupType::CategoryPicker => {
            category_picker::render_category_picker(f, state);
        }
        PopupType::CategoryManager => {
            category_manager::render_category_manager(
                f,
                f.size(),
                &state.category_manager_state,
                &state.layout.categories,
                &state.theme,
            );
        }
        PopupType::LayerManager => {
            layer_manager::render_layer_manager(
                f,
                f.size(),
                &state.layer_manager_state,
                &state.layout.layers,
                &state.theme,
            );
        }
        PopupType::LayerPicker => {
            layer_picker::render_layer_picker(
                f,
                &state.layer_picker_state,
                &state.layout.layers,
                &state.theme,
            );
        }
        PopupType::TemplateBrowser => {
            template_browser::render(f, &state.template_browser_state, f.size(), &state.theme);
        }
        PopupType::TemplateSaveDialog => {
            render_template_save_dialog(f, state);
        }
        PopupType::UnsavedChangesPrompt => {
            render_unsaved_prompt(f, &state.theme);
        }
        PopupType::BuildLog => {
            if let Some(build_state) = &state.build_state {
                build_log::render_build_log(f, build_state, &state.build_log_state, &state.theme);
            }
        }
        PopupType::HelpOverlay => {
            state.help_overlay_state.render(f, f.size(), &state.theme);
        }
        PopupType::LayoutPicker => {
            config_dialogs::render_layout_picker(
                f,
                &state.layout_picker_state,
                &state.config.build.keyboard,
                &state.theme,
            );
        }
        PopupType::MetadataEditor => {
            metadata_editor::render_metadata_editor(f, &state.metadata_editor_state, &state.theme);
        }
        PopupType::SetupWizard => {
            onboarding_wizard::render(f, &state.wizard_state, &state.theme);
        }
        PopupType::SettingsManager => {
            settings_manager::render_settings_manager(
                f,
                f.size(),
                &state.settings_manager_state,
                state.layout.inactive_key_behavior,
                &state.layout.tap_hold_settings,
                state.layout.rgb_timeout_ms,
                &state.config,
                &state.theme,
            );
        }
        PopupType::TapKeycodePicker => {
            // Reuse keycode picker rendering with custom title context
            // The title will show "Select Tap Keycode" based on pending_keycode state
            keycode_picker::render_keycode_picker(f, state);
        }
        PopupType::ModifierPicker => {
            modifier_picker::render_modifier_picker(
                f,
                &state.modifier_picker_state,
                &state.theme,
            );
        }
        PopupType::KeyEditor => {
            key_editor::render_key_editor(f, state);
        }
    }
}

/// Render unsaved changes prompt
fn render_unsaved_prompt(f: &mut Frame, theme: &Theme) {
    let area = centered_rect(60, 30, f.size());

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background
    let background = Block::default()
        .style(Style::default().bg(theme.background));
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

/// Render template save dialog
fn render_template_save_dialog(f: &mut Frame, state: &AppState) {
    let area = centered_rect(70, 60, f.size());

    let dialog_state = &state.template_save_dialog_state;
    let theme = &state.theme;

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background
    let background = Block::default()
        .style(Style::default().bg(theme.background));
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
    // Route to popup handler if popup is active
    if state.active_popup.is_some() {
        return handle_popup_input(state, key);
    }

    // Main UI key handling
    handle_main_input(state, key)
}

/// Handle input when popup is active
fn handle_popup_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    let popup_type = state.active_popup.clone();

    match popup_type {
        Some(PopupType::KeycodePicker) => keycode_picker::handle_input(state, key),
        Some(PopupType::ColorPicker) => color_picker::handle_input(state, key),
        Some(PopupType::CategoryPicker) => category_picker::handle_input(state, key),
        Some(PopupType::CategoryManager) => handle_category_manager_input(state, key),
        Some(PopupType::LayerManager) => handle_layer_manager_input(state, key),
        Some(PopupType::LayerPicker) => handle_layer_picker_input(state, key),
        Some(PopupType::TemplateBrowser) => handle_template_browser_input(state, key),
        Some(PopupType::TemplateSaveDialog) => handle_template_save_dialog_input(state, key),
        Some(PopupType::UnsavedChangesPrompt) => handle_unsaved_prompt_input(state, key),
        Some(PopupType::BuildLog) => handle_build_log_input(state, key),
        Some(PopupType::HelpOverlay) => handle_help_overlay_input(state, key),
        Some(PopupType::MetadataEditor) => handle_metadata_editor_input(state, key),
        Some(PopupType::LayoutPicker) => handle_layout_picker_input(state, key),
        Some(PopupType::SetupWizard) => handle_setup_wizard_input(state, key),
        Some(PopupType::SettingsManager) => handle_settings_manager_input(state, key),
        Some(PopupType::TapKeycodePicker) => handle_tap_keycode_picker_input(state, key),
        Some(PopupType::ModifierPicker) => handle_modifier_picker_input(state, key),
        Some(PopupType::KeyEditor) => key_editor::handle_input(state, key),
        _ => {
            // Escape closes any popup
            if key.code == KeyCode::Esc {
                state.active_popup = None;
                state.set_status("Cancelled");
            }
            Ok(false)
        }
    }
}

/// Handle input for build log viewer
fn handle_build_log_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.active_popup = None;
            state.build_log_state.visible = false;
            state.set_status("Build log closed");
            Ok(false)
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Copy build log to clipboard
            if let Some(build_state) = &state.build_state {
                let log_text = build_state
                    .log_lines
                    .iter()
                    .map(|(_, message)| message.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                
                match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(log_text)) {
                    Ok(()) => state.set_status("Build log copied to clipboard"),
                    Err(e) => state.set_error(&format!("Failed to copy to clipboard: {}", e)),
                }
            } else {
                state.set_error("No build log available");
            }
            Ok(false)
        }
        KeyCode::Up => {
            state.build_log_state.scroll_up();
            Ok(false)
        }
        KeyCode::Down => {
            if let Some(build_state) = &state.build_state {
                let max_lines = build_state.log_lines.len();
                state.build_log_state.scroll_down(max_lines, 20); // Approximate visible lines
            }
            Ok(false)
        }
        KeyCode::Home => {
            state.build_log_state.scroll_to_top();
            Ok(false)
        }
        KeyCode::End => {
            if let Some(build_state) = &state.build_state {
                let max_lines = build_state.log_lines.len();
                state.build_log_state.scroll_to_bottom(max_lines, 20); // Approximate visible lines
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle input for help overlay
fn handle_help_overlay_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        // Close help with Escape or '?'
        KeyCode::Esc | KeyCode::Char('?') => {
            state.active_popup = None;
            state.set_status("Press ? for help");
            Ok(false)
        }
        // Scroll up
        KeyCode::Up => {
            state.help_overlay_state.scroll_up();
            Ok(false)
        }
        // Scroll down
        KeyCode::Down => {
            state.help_overlay_state.scroll_down();
            Ok(false)
        }
        // Page up
        KeyCode::PageUp => {
            state.help_overlay_state.page_up(20); // Approximate visible height
            Ok(false)
        }
        // Page down
        KeyCode::PageDown => {
            state.help_overlay_state.page_down(20); // Approximate visible height
            Ok(false)
        }
        // Home - scroll to top
        KeyCode::Home => {
            state.help_overlay_state.scroll_to_top();
            Ok(false)
        }
        // End - scroll to bottom
        KeyCode::End => {
            state.help_overlay_state.scroll_to_bottom();
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle input for metadata editor
fn handle_metadata_editor_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    let action =
        metadata_editor::handle_metadata_editor_input(&mut state.metadata_editor_state, key);

    match action {
        metadata_editor::MetadataEditorAction::Confirm => {
            // Check if name changed before applying (need to do this before apply mutates state)
            let name_changed = state.metadata_editor_state.name_changed();
            let new_name = state.metadata_editor_state.name.clone();

            // Validate and apply changes
            match state
                .metadata_editor_state
                .apply_to_layout(&mut state.layout)
            {
                Ok(()) => {
                    state.mark_dirty();
                    
                    // If name changed and we have a source file, rename it
                    if name_changed {
                        if let Some(ref old_path) = state.source_path {
                            if old_path.exists() {
                                // Build new filename from new name (sanitized)
                                let sanitized_name = new_name
                                    .replace('/', "_")
                                    .replace('\\', "_")
                                    .replace(':', "_")
                                    .replace(' ', "_");
                                
                                if let Some(parent) = old_path.parent() {
                                    let new_path = parent.join(format!("{}.md", sanitized_name));
                                    
                                    // Only rename if the new path is different
                                    if new_path != *old_path {
                                        match std::fs::rename(old_path, &new_path) {
                                            Ok(()) => {
                                                state.source_path = Some(new_path);
                                                state.set_status(format!("Layout renamed to '{}'", new_name));
                                            }
                                            Err(e) => {
                                                state.set_error(format!("Failed to rename file: {}", e));
                                            }
                                        }
                                    } else {
                                        state.set_status("Metadata updated");
                                    }
                                } else {
                                    state.set_status("Metadata updated");
                                }
                            } else {
                                state.set_status("Metadata updated (file will be renamed on save)");
                            }
                        } else {
                            state.set_status("Metadata updated");
                        }
                    } else {
                        state.set_status("Metadata updated");
                    }
                    
                    state.active_popup = None;
                }
                Err(err) => {
                    state.set_error(format!("Validation failed: {err}"));
                }
            }
            Ok(false)
        }
        metadata_editor::MetadataEditorAction::Cancel => {
            state.active_popup = None;
            state.set_status("Metadata editing cancelled");
            Ok(false)
        }
        metadata_editor::MetadataEditorAction::Continue => Ok(false),
    }
}

/// Handle input for layout picker
fn handle_layout_picker_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    if let Some(selected) =
        config_dialogs::handle_layout_picker_input(&mut state.layout_picker_state, key)
    {
        if selected.is_empty() {
            // User cancelled
            if state.return_to_settings_after_picker {
                state.return_to_settings_after_picker = false;
                state.active_popup = Some(PopupType::SettingsManager);
                state.set_status("Layout selection cancelled");
            } else {
                state.active_popup = None;
                state.set_status("Layout selection cancelled");
            }
            return Ok(false);
        }

        // User selected a layout - rebuild geometry and mapping
        match state.rebuild_geometry(&selected) {
            Ok(()) => {
                if state.return_to_settings_after_picker {
                    state.return_to_settings_after_picker = false;
                    state.active_popup = Some(PopupType::SettingsManager);
                    state.set_status(format!("Switched to layout: {selected}"));
                } else {
                    state.active_popup = None;
                    state.set_status(format!("Switched to layout: {selected}"));
                }
                state.mark_dirty(); // Config change requires save
            }
            Err(e) => {
                state.set_error(format!("Failed to switch layout: {e}"));
                if state.return_to_settings_after_picker {
                    state.return_to_settings_after_picker = false;
                    state.active_popup = Some(PopupType::SettingsManager);
                }
            }
        }
    }
    Ok(false)
}

/// Handle input for layer picker (for layer-switching keycodes)
fn handle_layer_picker_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            // Check if this was part of a parameterized keycode flow
            if state.pending_keycode.keycode_type.is_some() {
                state.pending_keycode.reset();
            }
            state.active_popup = None;
            state.layer_picker_state.reset();
            state.set_status("Layer selection cancelled");
            Ok(false)
        }
        KeyCode::Up => {
            let layer_count = state.layout.layers.len();
            state.layer_picker_state.select_previous(layer_count);
            Ok(false)
        }
        KeyCode::Down => {
            let layer_count = state.layout.layers.len();
            state.layer_picker_state.select_next(layer_count);
            Ok(false)
        }
        KeyCode::Enter => {
            // Get the selected layer
            let selected_idx = state.layer_picker_state.selected;
            if let Some(layer) = state.layout.layers.get(selected_idx) {
                let layer_ref = format!("@{}", layer.id);
                
                // Check if we're in a parameterized keycode flow
                match &state.pending_keycode.keycode_type {
                    Some(ParameterizedKeycodeType::LayerTap) => {
                        // LT: Store layer, go to tap keycode picker
                        state.pending_keycode.param1 = Some(layer_ref);
                        state.active_popup = Some(PopupType::TapKeycodePicker);
                        state.keycode_picker_state = KeycodePickerState::new();
                        state.layer_picker_state.reset();
                        state.set_status("Select tap keycode for LT");
                        return Ok(false);
                    }
                    Some(ParameterizedKeycodeType::LayerMod) => {
                        // LM: Store layer, go to modifier picker
                        state.pending_keycode.param1 = Some(layer_ref);
                        state.active_popup = Some(PopupType::ModifierPicker);
                        state.layer_picker_state.reset();
                        state.set_status("Select modifier(s) for LM");
                        return Ok(false);
                    }
                    _ => {
                        // Regular layer keycode (MO, TG, TO, etc.) - assign directly
                        let keycode = state.layer_picker_state.build_keycode(layer);
                        
                        if let Some(key) = state.get_selected_key_mut() {
                            key.keycode = keycode.clone();
                            state.mark_dirty();
                            state.set_status(format!("Assigned: {}", keycode));
                        }
                    }
                }
            }
            
            state.active_popup = None;
            state.layer_picker_state.reset();
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle input for tap keycode picker (second stage of LT/MT/SH_T)
fn handle_tap_keycode_picker_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            // Cancel the whole parameterized keycode flow
            state.pending_keycode.reset();
            state.active_popup = None;
            state.keycode_picker_state = KeycodePickerState::new();
            state.set_status("Cancelled");
            Ok(false)
        }
        KeyCode::Enter => {
            // Get the selected keycode
            let keycodes = keycode_picker::get_filtered_keycodes(state);
            if let Some(kc) = keycodes.get(state.keycode_picker_state.selected) {
                // Only allow basic keycodes for tap action (no parameterized keycodes)
                if is_basic_keycode(&kc.code) {
                    // Store as param2 (tap keycode) for LT/MT, or param1 for SH_T
                    match &state.pending_keycode.keycode_type {
                        Some(ParameterizedKeycodeType::SwapHandsTap) => {
                            state.pending_keycode.param1 = Some(kc.code.clone());
                        }
                        _ => {
                            state.pending_keycode.param2 = Some(kc.code.clone());
                        }
                    }
                    
                    // Build and assign the final keycode
                    if let Some(final_keycode) = state.pending_keycode.build_keycode() {
                        if let Some(key) = state.get_selected_key_mut() {
                            key.keycode = final_keycode.clone();
                            state.mark_dirty();
                            state.set_status(format!("Assigned: {}", final_keycode));
                        }
                    }
                    
                    // Reset and close
                    state.pending_keycode.reset();
                    state.active_popup = None;
                    state.keycode_picker_state = KeycodePickerState::new();
                } else {
                    state.set_error("Only basic keycodes allowed for tap action");
                }
            }
            Ok(false)
        }
        // Delegate navigation to standard keycode picker
        _ => keycode_picker::handle_navigation(state, key),
    }
}

/// Handle input for modifier picker (for MT/LM keycodes)
fn handle_modifier_picker_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            state.pending_keycode.reset();
            state.modifier_picker_state.reset();
            state.active_popup = None;
            state.set_status("Cancelled");
            Ok(false)
        }
        KeyCode::Up => {
            state.modifier_picker_state.focus_up();
            Ok(false)
        }
        KeyCode::Down => {
            state.modifier_picker_state.focus_down();
            Ok(false)
        }
        KeyCode::Left => {
            state.modifier_picker_state.focus_left();
            Ok(false)
        }
        KeyCode::Right => {
            state.modifier_picker_state.focus_right();
            Ok(false)
        }
        KeyCode::Char(' ') => {
            // Toggle the focused modifier
            state.modifier_picker_state.toggle_focused();
            Ok(false)
        }
        KeyCode::Enter => {
            // Confirm selection - need at least one modifier
            if !state.modifier_picker_state.has_selection() {
                state.set_error("Select at least one modifier");
                return Ok(false);
            }

            let mod_string = state.modifier_picker_state.to_mod_string();

            match &state.pending_keycode.keycode_type {
                Some(ParameterizedKeycodeType::ModTap) => {
                    // MT: Store modifier as param1, go to tap keycode picker
                    state.pending_keycode.param1 = Some(mod_string);
                    state.active_popup = Some(PopupType::TapKeycodePicker);
                    state.keycode_picker_state = KeycodePickerState::new();
                    state.modifier_picker_state.reset();
                    state.set_status("Select tap keycode for MT");
                }
                Some(ParameterizedKeycodeType::LayerMod) => {
                    // LM: Store modifier as param2, build and assign
                    state.pending_keycode.param2 = Some(mod_string);
                    
                    if let Some(final_keycode) = state.pending_keycode.build_keycode() {
                        if let Some(key) = state.get_selected_key_mut() {
                            key.keycode = final_keycode.clone();
                            state.mark_dirty();
                            state.set_status(format!("Assigned: {}", final_keycode));
                        }
                    }
                    
                    state.pending_keycode.reset();
                    state.modifier_picker_state.reset();
                    state.active_popup = None;
                }
                _ => {
                    // Unexpected state - cancel
                    state.pending_keycode.reset();
                    state.modifier_picker_state.reset();
                    state.active_popup = None;
                    state.set_error("Unexpected state");
                }
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Check if a keycode is a basic keycode (not parameterized)
fn is_basic_keycode(code: &str) -> bool {
    // Basic keycodes: KC_A-Z, KC_0-9, KC_F1-24, navigation, symbols, etc.
    // Exclude: layer keycodes, mod-taps, parameterized keycodes
    !code.contains('(') && !code.contains('@')
}

/// Handle input for setup wizard
fn handle_setup_wizard_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Delegate to wizard's handle_input function
    match onboarding_wizard::handle_input(&mut state.wizard_state, key) {
        Ok(should_exit) => {
            if should_exit {
                // Wizard completed or cancelled
                if state.wizard_state.is_complete {
                    // Check if this was a keyboard-only change
                    if state.wizard_state.keyboard_change_only {
                        // Only update keyboard and layout fields
                        if let Some(keyboard) = state.wizard_state.inputs.get("keyboard") {
                            state.config.build.keyboard = keyboard.clone();
                        }
                        if let Some(layout) = state.wizard_state.inputs.get("layout") {
                            state.config.build.layout = layout.clone();
                        }
                        
                        // Save the config
                        if let Err(e) = state.config.save() {
                            state.set_error(format!("Failed to save configuration: {e}"));
                        } else {
                            // Rebuild geometry for new keyboard/layout
                            let layout_name = state.config.build.layout.clone();
                            match state.rebuild_geometry(&layout_name) {
                                Ok(()) => {
                                    state.set_status(format!(
                                        "Keyboard changed to: {}",
                                        state.config.build.keyboard
                                    ));
                                }
                                Err(e) => {
                                    state.set_error(format!("Failed to rebuild geometry: {e}"));
                                }
                            }
                        }
                        
                        // Return to settings manager
                        state.active_popup = Some(PopupType::SettingsManager);
                    } else {
                        // Full wizard - build and save the new config
                        match state.wizard_state.build_config() {
                            Ok(new_config) => {
                                // Update the app config
                                state.config = new_config;
                                
                                // Save the config
                                if let Err(e) = state.config.save() {
                                    state.set_error(format!("Failed to save configuration: {e}"));
                                } else {
                                    state.set_status("Configuration saved successfully");
                                }
                            }
                            Err(e) => {
                                state.set_error(format!("Failed to build configuration: {e}"));
                            }
                        }
                        state.active_popup = None;
                    }
                } else {
                    // Wizard cancelled
                    if state.wizard_state.keyboard_change_only {
                        // Return to settings manager
                        state.active_popup = Some(PopupType::SettingsManager);
                        state.set_status("Keyboard selection cancelled");
                    } else {
                        state.active_popup = None;
                        state.set_status("Setup wizard cancelled");
                    }
                }
                
                // Reset wizard state for next time
                state.wizard_state = onboarding_wizard::OnboardingWizardState::new();
            }
            Ok(false)
        }
        Err(e) => {
            state.set_error(format!("Wizard error: {e}"));
            Ok(false)
        }
    }
}

/// Handle input for settings manager
fn handle_settings_manager_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    use crate::models::{HoldDecisionMode, InactiveKeyBehavior, TapHoldPreset};
    use settings_manager::{ManagerMode, SettingItem};

    match &state.settings_manager_state.mode.clone() {
        ManagerMode::Browsing => match key.code {
            KeyCode::Esc => {
                state.active_popup = None;
                state.set_status("Settings closed");
                Ok(false)
            }
            KeyCode::Up => {
                let count = SettingItem::all().len();
                state.settings_manager_state.select_previous(count);
                Ok(false)
            }
            KeyCode::Down => {
                let count = SettingItem::all().len();
                state.settings_manager_state.select_next(count);
                Ok(false)
            }
            KeyCode::Enter => {
                // Start editing the selected setting
                let settings = SettingItem::all();
                if let Some(setting) = settings.get(state.settings_manager_state.selected) {
                    match setting {
                        SettingItem::InactiveKeyBehavior => {
                            state
                                .settings_manager_state
                                .start_selecting_inactive_behavior(
                                    state.layout.inactive_key_behavior,
                                );
                        }
                        SettingItem::TapHoldPreset => {
                            state
                                .settings_manager_state
                                .start_selecting_tap_hold_preset(
                                    state.layout.tap_hold_settings.preset,
                                );
                        }
                        SettingItem::TappingTerm => {
                            state.settings_manager_state.start_editing_numeric(
                                *setting,
                                state.layout.tap_hold_settings.tapping_term,
                                100,
                                500,
                            );
                        }
                        SettingItem::QuickTapTerm => {
                            let current =
                                state.layout.tap_hold_settings.quick_tap_term.unwrap_or(0);
                            state
                                .settings_manager_state
                                .start_editing_numeric(*setting, current, 0, 500);
                        }
                        SettingItem::HoldMode => {
                            state.settings_manager_state.start_selecting_hold_mode(
                                state.layout.tap_hold_settings.hold_mode,
                            );
                        }
                        SettingItem::RetroTapping => {
                            state.settings_manager_state.start_toggling_boolean(
                                *setting,
                                state.layout.tap_hold_settings.retro_tapping,
                            );
                        }
                        SettingItem::TappingToggle => {
                            state.settings_manager_state.start_editing_numeric(
                                *setting,
                                state.layout.tap_hold_settings.tapping_toggle as u16,
                                1,
                                10,
                            );
                        }
                        SettingItem::FlowTapTerm => {
                            let current =
                                state.layout.tap_hold_settings.flow_tap_term.unwrap_or(0);
                            state
                                .settings_manager_state
                                .start_editing_numeric(*setting, current, 0, 300);
                        }
                        SettingItem::ChordalHold => {
                            state.settings_manager_state.start_toggling_boolean(
                                *setting,
                                state.layout.tap_hold_settings.chordal_hold,
                            );
                        }
                        SettingItem::RgbTimeout => {
                            // RGB timeout is stored as ms but we edit in seconds for usability
                            // Max 10 minutes = 600000ms, stored as u32 but edited as u16 seconds
                            let current_secs = (state.layout.rgb_timeout_ms / 1000) as u16;
                            state.settings_manager_state.start_editing_numeric(
                                *setting,
                                current_secs,
                                0,
                                600, // Max 10 minutes in seconds
                            );
                        }
                        // Global settings
                        SettingItem::QmkFirmwarePath => {
                            state.settings_manager_state.start_editing_path(
                                *setting,
                                state.config.paths.qmk_firmware.clone()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_default(),
                            );
                        }
                        SettingItem::Keyboard => {
                            // Check if QMK path is configured
                            let qmk_path = if let Some(path) = &state.config.paths.qmk_firmware {
                                path.clone()
                            } else {
                                state.set_error("QMK firmware path not configured. Set it first.");
                                return Ok(false);
                            };

                            // Start wizard directly at keyboard selection step
                            match onboarding_wizard::OnboardingWizardState::new_for_keyboard_selection(&qmk_path) {
                                Ok(wizard_state) => {
                                    state.wizard_state = wizard_state;
                                    state.active_popup = Some(PopupType::SetupWizard);
                                    state.settings_manager_state.cancel();
                                    state.set_status("Select keyboard - Type to filter, Enter to select");
                                }
                                Err(e) => {
                                    state.set_error(format!("Failed to scan keyboards: {}", e));
                                }
                            }
                        }
                        SettingItem::LayoutVariant => {
                            // Trigger layout variant picker - same as Ctrl+Y
                            // Mark that we came from settings so we return there
                            state.return_to_settings_after_picker = true;
                            
                            // Load available layouts for current keyboard
                            let qmk_path = if let Some(path) = &state.config.paths.qmk_firmware {
                                path.clone()
                            } else {
                                state.set_error("QMK firmware path not configured");
                                return Ok(false);
                            };

                            // Extract base keyboard path
                            let base_keyboard = AppState::extract_base_keyboard(&state.config.build.keyboard);

                            if let Err(e) = state
                                .layout_picker_state
                                .load_layouts(&qmk_path, &base_keyboard)
                            {
                                state.set_error(format!("Failed to load layouts: {e}"));
                                return Ok(false);
                            }

                            state.active_popup = Some(PopupType::LayoutPicker);
                            state.set_status("Select layout variant - ↑↓: Navigate, Enter: Select");
                        }
                        SettingItem::KeymapName => {
                            state.settings_manager_state.start_editing_string(
                                *setting,
                                state.config.build.keymap.clone(),
                            );
                        }
                        SettingItem::OutputFormat => {
                            let current_format = &state.config.build.output_format;
                            let selected = match current_format.as_str() {
                                "hex" => 1,
                                "bin" => 2,
                                _ => 0, // uf2
                            };
                            state.settings_manager_state.start_selecting_output_format(selected);
                        }
                        SettingItem::OutputDir => {
                            state.settings_manager_state.start_editing_path(
                                *setting,
                                state.config.build.output_dir.to_string_lossy().to_string(),
                            );
                        }
                        SettingItem::ShowHelpOnStartup => {
                            state.settings_manager_state.start_toggling_boolean(
                                *setting,
                                state.config.ui.show_help_on_startup,
                            );
                        }
                    }
                    state.set_status("Select option with ↑↓, Enter to apply");
                }
                Ok(false)
            }
            _ => Ok(false),
        },
        ManagerMode::SelectingInactiveKeyBehavior { .. } => match key.code {
            KeyCode::Esc => {
                state.settings_manager_state.cancel();
                state.set_status("Cancelled");
                Ok(false)
            }
            KeyCode::Up => {
                let count = InactiveKeyBehavior::all().len();
                state.settings_manager_state.option_previous(count);
                Ok(false)
            }
            KeyCode::Down => {
                let count = InactiveKeyBehavior::all().len();
                state.settings_manager_state.option_next(count);
                Ok(false)
            }
            KeyCode::Enter => {
                if let Some(selected_idx) = state.settings_manager_state.get_selected_option() {
                    if let Some(&behavior) = InactiveKeyBehavior::all().get(selected_idx) {
                        state.layout.inactive_key_behavior = behavior;
                        state.mark_dirty();
                        state.settings_manager_state.cancel();
                        state.set_status(format!(
                            "Inactive key behavior set to: {}",
                            behavior.display_name()
                        ));
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        },
        ManagerMode::SelectingTapHoldPreset { .. } => match key.code {
            KeyCode::Esc => {
                state.settings_manager_state.cancel();
                state.set_status("Cancelled");
                Ok(false)
            }
            KeyCode::Up => {
                let count = TapHoldPreset::all().len();
                state.settings_manager_state.option_previous(count);
                Ok(false)
            }
            KeyCode::Down => {
                let count = TapHoldPreset::all().len();
                state.settings_manager_state.option_next(count);
                Ok(false)
            }
            KeyCode::Enter => {
                if let Some(selected_idx) = state.settings_manager_state.get_selected_option() {
                    if let Some(&preset) = TapHoldPreset::all().get(selected_idx) {
                        state.layout.tap_hold_settings.apply_preset(preset);
                        state.mark_dirty();
                        state.settings_manager_state.cancel();
                        state.set_status(format!(
                            "Tap-hold preset set to: {}",
                            preset.display_name()
                        ));
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        },
        ManagerMode::SelectingHoldMode { .. } => match key.code {
            KeyCode::Esc => {
                state.settings_manager_state.cancel();
                state.set_status("Cancelled");
                Ok(false)
            }
            KeyCode::Up => {
                let count = HoldDecisionMode::all().len();
                state.settings_manager_state.option_previous(count);
                Ok(false)
            }
            KeyCode::Down => {
                let count = HoldDecisionMode::all().len();
                state.settings_manager_state.option_next(count);
                Ok(false)
            }
            KeyCode::Enter => {
                if let Some(selected_idx) = state.settings_manager_state.get_selected_option() {
                    if let Some(&mode) = HoldDecisionMode::all().get(selected_idx) {
                        state.layout.tap_hold_settings.hold_mode = mode;
                        state.layout.tap_hold_settings.mark_custom();
                        state.mark_dirty();
                        state.settings_manager_state.cancel();
                        state.set_status(format!("Hold mode set to: {}", mode.display_name()));
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        },
        ManagerMode::EditingNumeric { setting, .. } => {
            let setting = *setting;
            match key.code {
                KeyCode::Esc => {
                    state.settings_manager_state.cancel();
                    state.set_status("Cancelled");
                    Ok(false)
                }
                KeyCode::Up => {
                    state.settings_manager_state.increment_numeric(10);
                    Ok(false)
                }
                KeyCode::Down => {
                    state.settings_manager_state.decrement_numeric(10);
                    Ok(false)
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    state.settings_manager_state.handle_char_input(c);
                    Ok(false)
                }
                KeyCode::Backspace => {
                    state.settings_manager_state.handle_backspace();
                    Ok(false)
                }
                KeyCode::Enter => {
                    if let Some(value) = state.settings_manager_state.get_numeric_value() {
                        apply_numeric_setting(state, setting, value);
                        state.mark_dirty();
                        state.settings_manager_state.cancel();
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::TogglingBoolean { setting, .. } => {
            let setting = *setting;
            match key.code {
                KeyCode::Esc => {
                    state.settings_manager_state.cancel();
                    state.set_status("Cancelled");
                    Ok(false)
                }
                KeyCode::Up | KeyCode::Down => {
                    state.settings_manager_state.option_previous(2);
                    Ok(false)
                }
                KeyCode::Enter => {
                    if let Some(value) = state.settings_manager_state.get_boolean_value() {
                        apply_boolean_setting(state, setting, value);
                        state.mark_dirty();
                        state.settings_manager_state.cancel();
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::EditingString { setting, .. } => {
            let setting = *setting;
            match key.code {
                KeyCode::Esc => {
                    state.settings_manager_state.cancel();
                    state.set_status("Cancelled");
                    Ok(false)
                }
                KeyCode::Char(c) => {
                    state.settings_manager_state.handle_string_char_input(c);
                    Ok(false)
                }
                KeyCode::Backspace => {
                    state.settings_manager_state.handle_string_backspace();
                    Ok(false)
                }
                KeyCode::Enter => {
                    if let Some(value) = state.settings_manager_state.get_string_value() {
                        apply_string_setting(state, setting, value.to_string())?;
                        state.settings_manager_state.cancel();
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::SelectingOutputFormat { .. } => {
            match key.code {
                KeyCode::Esc => {
                    state.settings_manager_state.cancel();
                    state.set_status("Cancelled");
                    Ok(false)
                }
                KeyCode::Up => {
                    state.settings_manager_state.option_previous(3);
                    Ok(false)
                }
                KeyCode::Down => {
                    state.settings_manager_state.option_next(3);
                    Ok(false)
                }
                KeyCode::Enter => {
                    if let Some(selected_idx) = state.settings_manager_state.get_output_format_selected() {
                        let format = match selected_idx {
                            0 => "uf2",
                            1 => "hex",
                            2 => "bin",
                            _ => "uf2",
                        };
                        state.config.build.output_format = format.to_string();
                        if let Err(e) = state.config.save() {
                            state.set_status(format!("Failed to save config: {}", e));
                        } else {
                            state.set_status(format!("Output format set to: {}", format));
                        }
                        state.settings_manager_state.cancel();
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::EditingPath { setting, .. } => {
            let setting = *setting;
            match key.code {
                KeyCode::Esc => {
                    state.settings_manager_state.cancel();
                    state.set_status("Cancelled");
                    Ok(false)
                }
                KeyCode::Char(c) => {
                    state.settings_manager_state.handle_string_char_input(c);
                    Ok(false)
                }
                KeyCode::Backspace => {
                    state.settings_manager_state.handle_string_backspace();
                    Ok(false)
                }
                KeyCode::Enter => {
                    if let Some(value) = state.settings_manager_state.get_string_value() {
                        apply_path_setting(state, setting, value.to_string())?;
                        state.settings_manager_state.cancel();
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
    }
}

/// Apply a numeric setting value
fn apply_numeric_setting(state: &mut AppState, setting: settings_manager::SettingItem, value: u16) {
    use settings_manager::SettingItem;

    match setting {
        SettingItem::TappingTerm => {
            state.layout.tap_hold_settings.tapping_term = value;
            state.layout.tap_hold_settings.mark_custom();
            state.set_status(format!("Tapping term set to: {}ms", value));
        }
        SettingItem::QuickTapTerm => {
            state.layout.tap_hold_settings.quick_tap_term = if value == 0 { None } else { Some(value) };
            state.layout.tap_hold_settings.mark_custom();
            let display = if value == 0 {
                "Auto".to_string()
            } else {
                format!("{}ms", value)
            };
            state.set_status(format!("Quick tap term set to: {}", display));
        }
        SettingItem::TappingToggle => {
            state.layout.tap_hold_settings.tapping_toggle = value as u8;
            state.layout.tap_hold_settings.mark_custom();
            state.set_status(format!("Tapping toggle set to: {} taps", value));
        }
        SettingItem::FlowTapTerm => {
            state.layout.tap_hold_settings.flow_tap_term = if value == 0 { None } else { Some(value) };
            state.layout.tap_hold_settings.mark_custom();
            let display = if value == 0 {
                "Disabled".to_string()
            } else {
                format!("{}ms", value)
            };
            state.set_status(format!("Flow tap term set to: {}", display));
        }
        SettingItem::RgbTimeout => {
            // value is in seconds, convert to milliseconds for storage
            state.layout.rgb_timeout_ms = u32::from(value) * 1000;
            let display = if value == 0 {
                "Disabled".to_string()
            } else if value >= 60 && value % 60 == 0 {
                format!("{} min", value / 60)
            } else {
                format!("{} sec", value)
            };
            state.set_status(format!("RGB timeout set to: {}", display));
        }
        _ => {}
    }
}

/// Apply a boolean setting value
fn apply_boolean_setting(state: &mut AppState, setting: settings_manager::SettingItem, value: bool) {
    use settings_manager::SettingItem;

    match setting {
        SettingItem::RetroTapping => {
            state.layout.tap_hold_settings.retro_tapping = value;
            state.layout.tap_hold_settings.mark_custom();
            let display = if value { "On" } else { "Off" };
            state.set_status(format!("Retro tapping set to: {}", display));
        }
        SettingItem::ChordalHold => {
            state.layout.tap_hold_settings.chordal_hold = value;
            state.layout.tap_hold_settings.mark_custom();
            let display = if value { "On" } else { "Off" };
            state.set_status(format!("Chordal hold set to: {}", display));
        }
        SettingItem::ShowHelpOnStartup => {
            state.config.ui.show_help_on_startup = value;
            if let Err(e) = state.config.save() {
                state.set_status(format!("Failed to save config: {}", e));
            } else {
                let display = if value { "On" } else { "Off" };
                state.set_status(format!("Show help on startup set to: {}", display));
            }
        }
        _ => {}
    }
}

/// Apply a string setting value
fn apply_string_setting(state: &mut AppState, setting: settings_manager::SettingItem, value: String) -> Result<()> {
    use settings_manager::SettingItem;

    match setting {
        SettingItem::KeymapName => {
            let keymap = if value.is_empty() { "default".to_string() } else { value.clone() };
            state.config.build.keymap = keymap.clone();
            if let Err(e) = state.config.save() {
                state.set_status(format!("Failed to save config: {}", e));
            } else {
                state.set_status(format!("Keymap name set to: {}", keymap));
            }
        }
        _ => {}
    }
    Ok(())
}

/// Apply a path setting value
fn apply_path_setting(state: &mut AppState, setting: settings_manager::SettingItem, value: String) -> Result<()> {
    use settings_manager::SettingItem;

    match setting {
        SettingItem::QmkFirmwarePath => {
            state.config.paths.qmk_firmware = if value.is_empty() { None } else { Some(std::path::PathBuf::from(&value)) };
            if let Err(e) = state.config.save() {
                state.set_status(format!("Failed to save config: {}", e));
            } else {
                state.set_status(format!("QMK firmware path set to: {}", if value.is_empty() { "(not set)" } else { &value }));
            }
        }
        SettingItem::OutputDir => {
            state.config.build.output_dir = std::path::PathBuf::from(&value);
            if let Err(e) = state.config.save() {
                state.set_status(format!("Failed to save config: {}", e));
            } else {
                state.set_status(format!("Output directory set to: {}", value));
            }
        }
        _ => {}
    }
    Ok(())
}

/// Handle input for unsaved changes prompt
fn handle_unsaved_prompt_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('s' | 'S') => {
            // Save and quit
            if let Some(path) = &state.source_path.clone() {
                crate::parser::save_markdown_layout(&state.layout, path)?;
                state.mark_clean();
                state.set_status("Saved");
            }
            state.should_quit = true;
            Ok(true)
        }
        KeyCode::Char('q' | 'Q') => {
            // Quit without saving
            state.should_quit = true;
            Ok(true)
        }
        KeyCode::Esc => {
            // Cancel
            state.active_popup = None;
            state.set_status("Cancelled");
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle input for template browser
fn handle_template_browser_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    let browser_state = &mut state.template_browser_state;

    if browser_state.search_active {
        // Search mode
        match key.code {
            KeyCode::Char(c) => {
                browser_state.search_push(c);
                Ok(false)
            }
            KeyCode::Backspace => {
                browser_state.search_pop();
                Ok(false)
            }
            KeyCode::Esc => {
                browser_state.toggle_search();
                state.set_status("Search mode exited");
                Ok(false)
            }
            KeyCode::Enter => {
                // Load selected template
                match browser_state.load_selected_template() {
                    Ok(layout) => {
                        state.layout = layout;
                        state.source_path = None; // New layout from template
                        state.mark_dirty(); // Mark as dirty since it's unsaved
                        state.active_popup = None;
                        state.set_status("Template loaded");
                        Ok(false)
                    }
                    Err(e) => {
                        state.set_error(format!("Failed to load template: {e}"));
                        Ok(false)
                    }
                }
            }
            _ => Ok(false),
        }
    } else {
        // Navigation mode
        match key.code {
            KeyCode::Up => {
                browser_state.select_previous();
                Ok(false)
            }
            KeyCode::Down => {
                browser_state.select_next();
                Ok(false)
            }
            KeyCode::Char('/') => {
                browser_state.toggle_search();
                state.set_status("Search mode - type to filter templates");
                Ok(false)
            }
            KeyCode::Enter => {
                // Load selected template
                match browser_state.load_selected_template() {
                    Ok(layout) => {
                        state.layout = layout;
                        state.source_path = None; // New layout from template
                        state.mark_dirty(); // Mark as dirty since it's unsaved
                        state.active_popup = None;
                        state.set_status("Template loaded");
                        Ok(false)
                    }
                    Err(e) => {
                        state.set_error(format!("Failed to load template: {e}"));
                        Ok(false)
                    }
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                state.active_popup = None;
                state.set_status("Template browser closed");
                Ok(false)
            }
            _ => Ok(false),
        }
    }
}

/// Handle input for template save dialog
fn handle_template_save_dialog_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char(c) => {
            // Add character to active field
            state
                .template_save_dialog_state
                .get_active_field_mut()
                .push(c);
            Ok(false)
        }
        KeyCode::Backspace => {
            // Remove character from active field
            state
                .template_save_dialog_state
                .get_active_field_mut()
                .pop();
            Ok(false)
        }
        KeyCode::Tab => {
            // Move to next field
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                state.template_save_dialog_state.previous_field();
            } else {
                state.template_save_dialog_state.next_field();
            }
            Ok(false)
        }
        KeyCode::Enter => {
            // Save template
            let dialog_state = &state.template_save_dialog_state;

            // Validate name is not empty
            if dialog_state.name.trim().is_empty() {
                state.set_error("Template name cannot be empty");
                return Ok(false);
            }

            // Create template directory if it doesn't exist
            let templates_dir = template_browser::TemplateBrowserState::templates_dir()?;
            std::fs::create_dir_all(&templates_dir).context(format!(
                "Failed to create templates directory: {}",
                templates_dir.display()
            ))?;

            // Update layout metadata for template
            let mut template_layout = state.layout.clone();
            template_layout.metadata.name = dialog_state.name.clone();
            template_layout.metadata.description = dialog_state.description.clone();
            template_layout.metadata.author = dialog_state.author.clone();
            template_layout.metadata.tags = dialog_state.parse_tags();
            template_layout.metadata.is_template = true;
            template_layout.metadata.touch();

            // Generate filename from name (sanitize)
            let filename = dialog_state
                .name
                .to_lowercase()
                .replace(' ', "-")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>();
            let template_path = templates_dir.join(format!("{filename}.md"));

            // Save template
            match crate::parser::save_markdown_layout(&template_layout, &template_path) {
                Ok(()) => {
                    state.active_popup = None;
                    state.set_status(format!("Template saved: {}", template_path.display()));
                    Ok(false)
                }
                Err(e) => {
                    state.set_error(format!("Failed to save template: {e}"));
                    Ok(false)
                }
            }
        }
        KeyCode::Esc => {
            // Cancel
            state.active_popup = None;
            state.set_status("Template save cancelled");
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Calculate all positions within a rectangle defined by two corner positions.
fn calculate_rectangle_selection(start: Position, end: Position, mapping: &VisualLayoutMapping) -> Vec<Position> {
    let min_row = start.row.min(end.row);
    let max_row = start.row.max(end.row);
    let min_col = start.col.min(end.col);
    let max_col = start.col.max(end.col);
    
    let mut selected = Vec::new();
    for row in min_row..=max_row {
        for col in min_col..=max_col {
            let pos = Position::new(row, col);
            // Only include positions that exist in the keyboard mapping
            if mapping.is_valid_position(pos) {
                selected.push(pos);
            }
        }
    }
    selected
}

/// Handle input for main UI
fn handle_main_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match (key.code, key.modifiers) {
        // Quit
        (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
            if state.dirty {
                state.active_popup = Some(PopupType::UnsavedChangesPrompt);
                Ok(false)
            } else {
                Ok(true) // Quit immediately
            }
        }

        // Save
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
            if let Some(path) = &state.source_path.clone() {
                crate::parser::save_markdown_layout(&state.layout, path)?;
                state.mark_clean();
                state.set_status("Saved");
            } else {
                state.set_error("No file path set");
            }
            Ok(false)
        }

        // Navigation - Arrow keys (with rectangle selection support)
        (KeyCode::Up, _) => {
            if let Some(new_pos) = state.mapping.find_position_up(state.selected_position) {
                state.selected_position = new_pos;
                // Update rectangle selection if active
                if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
                    state.selected_keys = calculate_rectangle_selection(start, new_pos, &state.mapping);
                }
                state.clear_error();
            }
            Ok(false)
        }
        (KeyCode::Down, _) => {
            if let Some(new_pos) = state.mapping.find_position_down(state.selected_position) {
                state.selected_position = new_pos;
                if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
                    state.selected_keys = calculate_rectangle_selection(start, new_pos, &state.mapping);
                }
                state.clear_error();
            }
            Ok(false)
        }
        (KeyCode::Left, _) => {
            if let Some(new_pos) = state.mapping.find_position_left(state.selected_position) {
                state.selected_position = new_pos;
                if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
                    state.selected_keys = calculate_rectangle_selection(start, new_pos, &state.mapping);
                }
                state.clear_error();
            }
            Ok(false)
        }
        (KeyCode::Right, _) => {
            if let Some(new_pos) = state.mapping.find_position_right(state.selected_position) {
                state.selected_position = new_pos;
                if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
                    state.selected_keys = calculate_rectangle_selection(start, new_pos, &state.mapping);
                }
                state.clear_error();
            }
            Ok(false)
        }

        // Navigation - VIM style (hjkl) (with rectangle selection support)
        (KeyCode::Char('h'), _) => {
            if let Some(new_pos) = state.mapping.find_position_left(state.selected_position) {
                state.selected_position = new_pos;
                if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
                    state.selected_keys = calculate_rectangle_selection(start, new_pos, &state.mapping);
                }
                state.clear_error();
            }
            Ok(false)
        }
        (KeyCode::Char('j'), _) => {
            if let Some(new_pos) = state.mapping.find_position_down(state.selected_position) {
                state.selected_position = new_pos;
                if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
                    state.selected_keys = calculate_rectangle_selection(start, new_pos, &state.mapping);
                }
                state.clear_error();
            }
            Ok(false)
        }
        (KeyCode::Char('k'), _) => {
            if let Some(new_pos) = state.mapping.find_position_up(state.selected_position) {
                state.selected_position = new_pos;
                if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
                    state.selected_keys = calculate_rectangle_selection(start, new_pos, &state.mapping);
                }
                state.clear_error();
            }
            Ok(false)
        }
        // Toggle build log (Ctrl+L) - must come before general 'l' pattern
        (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
            if state.build_state.is_some() {
                if state.build_log_state.visible {
                    state.active_popup = None;
                    state.build_log_state.visible = false;
                } else {
                    state.active_popup = Some(PopupType::BuildLog);
                    state.build_log_state.visible = true;
                }
                state.set_status("Build log toggled");
            } else {
                state.set_error("No build active");
            }
            Ok(false)
        }
        (KeyCode::Char('l'), _) => {
            if let Some(new_pos) = state.mapping.find_position_right(state.selected_position) {
                state.selected_position = new_pos;
                if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
                    state.selected_keys = calculate_rectangle_selection(start, new_pos, &state.mapping);
                }
                state.clear_error();
            }
            Ok(false)
        }

        // Layer switching
        (KeyCode::BackTab, _) => {
            if state.current_layer > 0 {
                state.current_layer -= 1;
                state.set_status(format!("Layer {}", state.current_layer));
                state.clear_error();
            }
            Ok(false)
        }
        (KeyCode::Tab, _) => {
            if state.current_layer < state.layout.layers.len() - 1 {
                state.current_layer += 1;
                state.set_status(format!("Layer {}", state.current_layer));
                state.clear_error();
            }
            Ok(false)
        }

        // Clear key (x or Delete - but not Ctrl+X which is cut)
        (KeyCode::Char('x'), KeyModifiers::NONE) | (KeyCode::Delete, _) => {
            if state.selection_mode.is_some() && !state.selected_keys.is_empty() {
                // Clear all selected keys
                let layer = state.current_layer;
                for pos in &state.selected_keys.clone() {
                    if let Some(layer) = state.layout.layers.get_mut(layer) {
                        if let Some(key) = layer.keys.iter_mut().find(|k| k.position == *pos) {
                            key.keycode = "KC_TRNS".to_string();
                        }
                    }
                }
                let count = state.selected_keys.len();
                state.selected_keys.clear();
                state.selection_mode = None;
                state.mark_dirty();
                state.set_status(format!("Cleared {} keys", count));
            } else if let Some(key) = state.get_selected_key_mut() {
                key.keycode = "KC_TRNS".to_string();
                state.mark_dirty();
                state.set_status("Key cleared (KC_TRNS)");
            }
            Ok(false)
        }

        // Enter/exit selection mode (Shift+V for toggle, Space to add/remove current key)
        (KeyCode::Char('V'), KeyModifiers::SHIFT) => {
            if state.selection_mode.is_some() {
                // Exit selection mode
                state.selection_mode = None;
                state.selected_keys.clear();
                state.set_status("Selection mode cancelled");
            } else {
                // Enter selection mode with current key selected
                state.selection_mode = Some(SelectionMode::Normal);
                state.selected_keys.clear();
                state.selected_keys.push(state.selected_position);
                state.set_status("Selection mode - Space: toggle key, y: copy, d: cut, Esc: cancel");
            }
            Ok(false)
        }

        // Toggle current key in selection (Space in selection mode)
        (KeyCode::Char(' '), KeyModifiers::NONE) => {
            if state.selection_mode.is_some() {
                let pos = state.selected_position;
                if let Some(idx) = state.selected_keys.iter().position(|p| *p == pos) {
                    state.selected_keys.remove(idx);
                } else {
                    state.selected_keys.push(pos);
                }
                state.set_status(format!("{} keys selected", state.selected_keys.len()));
                Ok(false)
            } else {
                // Space outside selection mode - no action
                Ok(false)
            }
        }

        // Rectangle selection start (Shift+R in selection mode or as entry)
        (KeyCode::Char('R'), KeyModifiers::SHIFT) => {
            if state.selection_mode.is_some() {
                // Start rectangle selection from current position
                state.selection_mode = Some(SelectionMode::Rectangle { start: state.selected_position });
                state.selected_keys.clear();
                state.selected_keys.push(state.selected_position);
                state.set_status("Rectangle select - move to opposite corner, Enter to confirm");
            } else {
                // Enter rectangle selection mode
                state.selection_mode = Some(SelectionMode::Rectangle { start: state.selected_position });
                state.selected_keys.clear();
                state.selected_keys.push(state.selected_position);
                state.set_status("Rectangle select - move to opposite corner, Enter to confirm");
            }
            Ok(false)
        }

        // Copy key (y or Ctrl+C) - handles both single and multi-selection
        (KeyCode::Char('y'), KeyModifiers::NONE) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            if state.selection_mode.is_some() && !state.selected_keys.is_empty() {
                // Copy all selected keys
                let layer = state.current_layer;
                let anchor = state.selected_keys[0];
                let mut keys: Vec<(Position, clipboard::ClipboardContent)> = Vec::new();
                
                for pos in &state.selected_keys {
                    if let Some(layer) = state.layout.layers.get(layer) {
                        if let Some(key) = layer.keys.iter().find(|k| k.position == *pos) {
                            keys.push((*pos, clipboard::ClipboardContent {
                                keycode: key.keycode.clone(),
                                color_override: key.color_override,
                                category_id: key.category_id.clone(),
                            }));
                        }
                    }
                }
                
                let msg = state.clipboard.copy_multi(keys, anchor);
                state.selection_mode = None;
                state.selected_keys.clear();
                state.set_status(msg);
            } else if let Some(key) = state.get_selected_key() {
                // Clone key data to avoid borrow conflict with clipboard
                let keycode = key.keycode.clone();
                let color_override = key.color_override;
                let category_id = key.category_id.clone();
                let msg = state.clipboard.copy(
                    &keycode,
                    color_override,
                    category_id.as_deref(),
                );
                state.set_status(msg);
            } else {
                state.set_error("No key to copy");
            }
            Ok(false)
        }

        // Cut key (d or Ctrl+X) - handles both single and multi-selection
        (KeyCode::Char('d'), KeyModifiers::NONE) | (KeyCode::Char('x'), KeyModifiers::CONTROL) => {
            if state.selection_mode.is_some() && !state.selected_keys.is_empty() {
                // Cut all selected keys
                let layer = state.current_layer;
                let anchor = state.selected_keys[0];
                let positions = state.selected_keys.clone();
                let mut keys: Vec<(Position, clipboard::ClipboardContent)> = Vec::new();
                
                for pos in &state.selected_keys {
                    if let Some(layer_ref) = state.layout.layers.get(layer) {
                        if let Some(key) = layer_ref.keys.iter().find(|k| k.position == *pos) {
                            keys.push((*pos, clipboard::ClipboardContent {
                                keycode: key.keycode.clone(),
                                color_override: key.color_override,
                                category_id: key.category_id.clone(),
                            }));
                        }
                    }
                }
                
                let msg = state.clipboard.cut_multi(keys, anchor, layer, positions);
                state.selection_mode = None;
                state.selected_keys.clear();
                state.set_status(msg);
            } else if let Some(key) = state.get_selected_key() {
                // Clone key data to avoid borrow conflict with clipboard
                let keycode = key.keycode.clone();
                let color_override = key.color_override;
                let category_id = key.category_id.clone();
                let msg = state.clipboard.cut(
                    &keycode,
                    color_override,
                    category_id.as_deref(),
                    state.current_layer,
                    state.selected_position,
                );
                state.set_status(msg);
            } else {
                state.set_error("No key to cut");
            }
            Ok(false)
        }

        // Paste key (p or Ctrl+V)
        (KeyCode::Char('p'), KeyModifiers::NONE) | (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
            // Check for multi-key paste first
            if let Some(multi) = state.clipboard.get_multi_content().cloned() {
                // Calculate target positions relative to current position
                // The anchor is the reference point, we need to shift all keys
                let anchor = multi.anchor;
                let current = state.selected_position;
                
                // Calculate offset from anchor to current position
                let row_offset = current.row as isize - anchor.row as isize;
                let col_offset = current.col as isize - anchor.col as isize;
                
                // Collect valid target positions and save undo state
                let mut paste_targets: Vec<(Position, clipboard::ClipboardContent)> = Vec::new();
                let mut undo_keys: Vec<(Position, clipboard::ClipboardContent)> = Vec::new();
                
                for (pos, content) in &multi.keys {
                    // Calculate target position
                    let target_row = pos.row as isize + row_offset;
                    let target_col = pos.col as isize + col_offset;
                    
                    if target_row >= 0 && target_col >= 0 && target_row <= u8::MAX as isize && target_col <= u8::MAX as isize {
                        let target_pos = Position::new(target_row as u8, target_col as u8);
                        
                        // Check if target position is valid
                        if state.mapping.is_valid_position(target_pos) {
                            // Save original for undo
                            if let Some(layer) = state.layout.layers.get(state.current_layer) {
                                if let Some(key) = layer.keys.iter().find(|k| k.position == target_pos) {
                                    undo_keys.push((target_pos, clipboard::ClipboardContent {
                                        keycode: key.keycode.clone(),
                                        color_override: key.color_override,
                                        category_id: key.category_id.clone(),
                                    }));
                                }
                            }
                            paste_targets.push((target_pos, content.clone()));
                        }
                    }
                }
                
                if paste_targets.is_empty() {
                    state.set_error("No valid positions for paste");
                    return Ok(false);
                }
                
                // Save undo state
                state.clipboard.save_undo(
                    state.current_layer,
                    undo_keys,
                    format!("Pasted {} keys", paste_targets.len()),
                );
                
                // Get cut sources before paste
                let cut_sources: Vec<(usize, Position)> = state.clipboard.get_multi_cut_sources().to_vec();
                
                // Apply pastes
                let paste_count = paste_targets.len();
                for (target_pos, content) in &paste_targets {
                    if let Some(layer) = state.layout.layers.get_mut(state.current_layer) {
                        if let Some(key) = layer.keys.iter_mut().find(|k| k.position == *target_pos) {
                            key.keycode = content.keycode.clone();
                            key.color_override = content.color_override;
                            key.category_id = content.category_id.clone();
                        }
                    }
                }
                
                // Clear cut sources if this was a cut operation
                for (layer_idx, pos) in cut_sources {
                    if let Some(layer) = state.layout.layers.get_mut(layer_idx) {
                        if let Some(source_key) = layer.keys.iter_mut().find(|k| k.position == pos) {
                            source_key.keycode = "KC_TRNS".to_string();
                            source_key.color_override = None;
                            source_key.category_id = None;
                        }
                    }
                }
                state.clipboard.clear_cut_source();
                
                // Flash the first pasted key (current position)
                state.flash_highlight = Some((state.current_layer, current, 5));
                
                state.mark_dirty();
                state.set_status(format!("Pasted {} keys", paste_count));
            } else if let Some(content) = state.clipboard.get_content().cloned() {
                // Single key paste (original logic)
                // Get cut source before modifying clipboard
                let cut_source = state.clipboard.get_cut_source();
                
                // Save undo state before making changes
                if let Some(key) = state.get_selected_key() {
                    let original = clipboard::ClipboardContent {
                        keycode: key.keycode.clone(),
                        color_override: key.color_override,
                        category_id: key.category_id.clone(),
                    };
                    state.clipboard.save_undo(
                        state.current_layer,
                        vec![(state.selected_position, original)],
                        format!("Pasted: {}", content.keycode),
                    );
                }
                
                // Apply clipboard content to selected key
                if let Some(key) = state.get_selected_key_mut() {
                    key.keycode = content.keycode.clone();
                    key.color_override = content.color_override;
                    key.category_id = content.category_id.clone();
                    state.mark_dirty();
                    state.set_status(format!("Pasted: {}", content.keycode));
                    
                    // Trigger flash highlight (5 frames ~= 250ms at 50ms/frame)
                    state.flash_highlight = Some((state.current_layer, state.selected_position, 5));
                }

                // If this was a cut operation, clear the source key
                if let Some((layer_idx, pos)) = cut_source {
                    if let Some(layer) = state.layout.layers.get_mut(layer_idx) {
                        if let Some(source_key) = layer.keys.iter_mut().find(|k| k.position == pos) {
                            source_key.keycode = "KC_TRNS".to_string();
                            source_key.color_override = None;
                            source_key.category_id = None;
                        }
                    }
                    state.clipboard.clear_cut_source();
                }
            } else {
                state.set_error("Nothing in clipboard");
            }
            Ok(false)
        }

        // Undo last paste operation (Ctrl+Z)
        (KeyCode::Char('z'), KeyModifiers::CONTROL) => {
            if let Some(undo) = state.clipboard.take_undo() {
                // Restore original keys
                for (pos, content) in undo.original_keys {
                    if let Some(layer) = state.layout.layers.get_mut(undo.layer_index) {
                        if let Some(key) = layer.keys.iter_mut().find(|k| k.position == pos) {
                            key.keycode = content.keycode;
                            key.color_override = content.color_override;
                            key.category_id = content.category_id;
                        }
                    }
                }
                state.mark_dirty();
                state.set_status("Undone paste operation");
            } else {
                state.set_error("Nothing to undo");
            }
            Ok(false)
        }

        // Open key editor (if key is assigned) or keycode picker (if empty)
        (KeyCode::Enter, _) => {
            // Check if key is assigned (read-only first)
            let key_info = state.get_selected_key().map(|key| {
                (key.position, key_editor::is_key_assigned(&key.keycode), key.description.clone())
            });
            
            if let Some((position, is_assigned, description)) = key_info {
                if is_assigned {
                    // Key is assigned - open key editor
                    // Re-initialize editor state with copied data
                    state.key_editor_state.position = position;
                    state.key_editor_state.layer_idx = state.current_layer;
                    state.key_editor_state.mode = key_editor::KeyEditorMode::View;
                    state.key_editor_state.description_buffer = description.clone().unwrap_or_default();
                    state.key_editor_state.cursor_position = state.key_editor_state.description_buffer.len();
                    state.key_editor_state.original_description = description;
                    state.active_popup = Some(PopupType::KeyEditor);
                    state.set_status("Key editor - Enter: Reassign, D: Description, C: Color");
                } else {
                    // Key is empty (KC_NO, KC_TRNS) - open keycode picker directly
                    state.active_popup = Some(PopupType::KeycodePicker);
                    state.keycode_picker_state = KeycodePickerState::new();
                }
            } else {
                // No key selected - just open keycode picker
                state.active_popup = Some(PopupType::KeycodePicker);
                state.keycode_picker_state = KeycodePickerState::new();
            }
            Ok(false)
        }

        // Color picker for individual key (Shift+C)
        (KeyCode::Char('C'), KeyModifiers::SHIFT) => {
            if let Some(key) = state.get_selected_key() {
                // Initialize color picker with current key color
                let current_color = state.layout.resolve_key_color(state.current_layer, key);
                state.color_picker_state = ColorPickerState::with_color(current_color);
                state.color_picker_context = Some(ColorPickerContext::IndividualKey);
                state.active_popup = Some(PopupType::ColorPicker);
                state
                    .set_status("Adjust color with arrows, Tab to switch channels, Enter to apply");
            } else {
                state.set_error("No key selected");
            }
            Ok(false)
        }

        // Color picker for layer default color (c key)
        (KeyCode::Char('c'), _) => {
            if let Some(layer) = state.layout.layers.get(state.current_layer) {
                // Initialize color picker with current layer default color
                state.color_picker_state = ColorPickerState::with_color(layer.default_color);
                state.color_picker_context = Some(ColorPickerContext::LayerDefault);
                state.active_popup = Some(PopupType::ColorPicker);
                state.set_status("Setting layer default color - Enter to apply");
            }
            Ok(false)
        }

        // Toggle colors for all layers (Alt+V)
        (KeyCode::Char('v'), KeyModifiers::ALT) => {
            let enabled = state.layout.toggle_all_layer_colors();
            state.dirty = true;
            let status = if enabled {
                "All layer colors enabled".to_string()
            } else {
                "All layer colors disabled".to_string()
            };
            state.set_status(status);
            Ok(false)
        }

        // Toggle colors for current layer (v key, no modifier)
        (KeyCode::Char('v'), KeyModifiers::NONE) => {
            if let Some(enabled) = state.layout.toggle_layer_colors(state.current_layer) {
                state.dirty = true;
                let status = if enabled {
                    format!("Layer {} colors enabled", state.current_layer)
                } else {
                    format!("Layer {} colors disabled", state.current_layer)
                };
                state.set_status(status);
            }
            Ok(false)
        }

        // Category picker for individual key (Shift+K)
        (KeyCode::Char('K'), KeyModifiers::SHIFT) => {
            if state.get_selected_key().is_some() {
                state.category_picker_state = CategoryPickerState::new();
                state.category_picker_context = Some(CategoryPickerContext::IndividualKey);
                state.active_popup = Some(PopupType::CategoryPicker);
                state.set_status("Select category for key - Enter to apply");
            } else {
                state.set_error("No key selected");
            }
            Ok(false)
        }

        // Category picker for layer (Shift+L)
        (KeyCode::Char('L'), KeyModifiers::SHIFT) => {
            state.category_picker_state = CategoryPickerState::new();
            state.category_picker_context = Some(CategoryPickerContext::Layer);
            state.active_popup = Some(PopupType::CategoryPicker);
            state.set_status("Select category for layer - Enter to apply");
            Ok(false)
        }

        // Category Manager (Ctrl+T)
        (KeyCode::Char('t'), KeyModifiers::CONTROL) => {
            state.category_manager_state.reset();
            state.active_popup = Some(PopupType::CategoryManager);
            state.set_status("Category Manager - n: New, r: Rename, c: Color, d: Delete");
            Ok(false)
        }

        // Layout Picker (Ctrl+Y)
        (KeyCode::Char('y'), KeyModifiers::CONTROL) => {
            // Load available layouts for current keyboard
            let qmk_path = if let Some(path) = &state.config.paths.qmk_firmware {
                path.clone()
            } else {
                state.set_error("QMK firmware path not configured");
                return Ok(false);
            };

            // Extract base keyboard path (remove variant subdirectory if present)
            let base_keyboard = AppState::extract_base_keyboard(&state.config.build.keyboard);

            if let Err(e) = state
                .layout_picker_state
                .load_layouts(&qmk_path, &base_keyboard)
            {
                state.set_error(format!("Failed to load layouts: {e}"));
                return Ok(false);
            }

            state.active_popup = Some(PopupType::LayoutPicker);
            state.set_status("Select layout variant - ↑↓: Navigate, Enter: Select");
            Ok(false)
        }

        // Save as template (Shift+T)
        (KeyCode::Char('T'), KeyModifiers::SHIFT) => {
            // Initialize template save dialog with current layout name
            state.template_save_dialog_state =
                TemplateSaveDialogState::new(state.layout.metadata.name.clone());
            state.active_popup = Some(PopupType::TemplateSaveDialog);
            state.set_status("Save as template - Enter field values");
            Ok(false)
        }

        // Layer Manager (Shift+N)
        (KeyCode::Char('N'), KeyModifiers::SHIFT) => {
            state.layer_manager_state.reset(state.current_layer);
            state.active_popup = Some(PopupType::LayerManager);
            state.set_status("Layer Manager - n: New, r: Rename, v: Toggle Colors, d: Delete");
            Ok(false)
        }

        // Settings Manager (Shift+S)
        (KeyCode::Char('S'), KeyModifiers::SHIFT) => {
            state.settings_manager_state.reset();
            state.active_popup = Some(PopupType::SettingsManager);
            state.set_status("Settings - ↑↓: Navigate, Enter: Change, Esc: Close");
            Ok(false)
        }

        // Browse templates (t key)
        (KeyCode::Char('t'), _) => {
            // Check for unsaved changes before loading template
            if state.dirty {
                // Show warning first
                state.set_status(
                    "You have unsaved changes. Press 't' again to open template browser anyway.",
                );
                // Mark that we want to open template browser
                // For now, open directly - T143 will add proper warning
                state.template_browser_state = TemplateBrowserState::new();
                if let Err(e) = state.template_browser_state.scan_templates() {
                    state.set_error(format!("Failed to scan templates: {e}"));
                    return Ok(false);
                }
                state.active_popup = Some(PopupType::TemplateBrowser);
                state.set_status("Select a template - Enter to load");
            } else {
                state.template_browser_state = TemplateBrowserState::new();
                if let Err(e) = state.template_browser_state.scan_templates() {
                    state.set_error(format!("Failed to scan templates: {e}"));
                    return Ok(false);
                }
                state.active_popup = Some(PopupType::TemplateBrowser);
                state.set_status("Select a template - Enter to load");
            }
            Ok(false)
        }

        // Generate firmware (Ctrl+G)
        (KeyCode::Char('g'), KeyModifiers::CONTROL) => {
            handle_firmware_generation(state)?;
            Ok(false)
        }

        // Build firmware (Ctrl+B)
        (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
            handle_firmware_build(state)?;
            Ok(false)
        }

        // Help
        (KeyCode::Char('?'), _) => {
            state.help_overlay_state = HelpOverlayState::new();
            state.active_popup = Some(PopupType::HelpOverlay);
            state.set_status("Use arrow keys to scroll, '?' or Escape to close");
            Ok(false)
        }

        // Metadata Editor (Ctrl+E)
        (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
            state.metadata_editor_state = MetadataEditorState::new(&state.layout.metadata);
            state.active_popup = Some(PopupType::MetadataEditor);
            state.set_status("Edit metadata - Tab to navigate, Enter to save");
            Ok(false)
        }

        // Setup Wizard (Ctrl+W)
        (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
            // Pre-populate wizard with current config
            state.wizard_state = onboarding_wizard::OnboardingWizardState::from_config(&state.config);
            state.active_popup = Some(PopupType::SetupWizard);
            state.set_status("Setup Wizard - configure QMK path, keyboard, and layout");
            Ok(false)
        }

        // Escape - cancel selection mode or cut operation
        (KeyCode::Esc, _) => {
            if state.selection_mode.is_some() {
                state.selection_mode = None;
                state.selected_keys.clear();
                state.set_status("Selection cancelled");
            } else if state.clipboard.is_cut() {
                state.clipboard.cancel_cut();
                state.set_status("Cut cancelled");
            }
            Ok(false)
        }

        _ => Ok(false),
    }
}

/// Handle firmware generation with validation
fn handle_firmware_generation(state: &mut AppState) -> Result<()> {
    use crate::firmware::{FirmwareGenerator, FirmwareValidator};

    // Step 1: Validate layout
    state.set_status("Validating layout...");

    let validator = FirmwareValidator::new(
        &state.layout,
        &state.geometry,
        &state.mapping,
        &state.keycode_db,
    );
    let report = validator.validate()?;

    if !report.is_valid() {
        // Show validation errors
        let error_msg = report.format_message();
        state.set_error(format!("Validation failed:\n{error_msg}"));
        return Ok(());
    }

    // Step 2: Generate firmware files
    state.set_status("Generating firmware files...");

    let generator = FirmwareGenerator::new(
        &state.layout,
        &state.geometry,
        &state.mapping,
        &state.config,
    );

    match generator.generate() {
        Ok((keymap_path, vial_path, config_path)) => {
            state.set_status(format!("✓ Generated: {keymap_path}, {vial_path}, {config_path}"));
        }
        Err(e) => {
            state.set_error(format!("Generation failed: {e}"));
        }
    }

    Ok(())
}

/// Handle firmware build in background
fn handle_firmware_build(state: &mut AppState) -> Result<()> {
    // Generate firmware files first (keymap.c, vial.json, config.h)
    handle_firmware_generation(state)?;
    
    // Check that QMK firmware path is configured
    let qmk_path = if let Some(path) = &state.config.paths.qmk_firmware {
        path.clone()
    } else {
        state.set_error("QMK firmware path not configured");
        return Ok(());
    };

    // Initialize build state if needed
    if state.build_state.is_none() {
        state.build_state = Some(BuildState::new());
    }

    let build_state = state.build_state.as_mut().unwrap();

    // Check if build already in progress
    if build_state.is_building() {
        state.set_error("Build already in progress");
        return Ok(());
    }

    // Determine correct keyboard variant path for building
    // This ensures we target the specific variant (e.g. "keebart/corne_choc_pro/standard")
    // so that QMK loads the correct configuration (including RGB settings)
    let base_keyboard = AppState::extract_base_keyboard(&state.config.build.keyboard);
    let key_count = state.geometry.keys.len();
    
    let build_keyboard = state.config.build
        .determine_keyboard_variant(&qmk_path, &base_keyboard, key_count)
        .unwrap_or_else(|e| {
            // Log warning but fall back to configured keyboard path
            eprintln!("Warning: Could not determine variant: {}", e);
            state.config.build.keyboard.clone()
        });

    // Start the build
    build_state.start_build(
        qmk_path,
        build_keyboard,
        state.config.build.keymap.clone(),
    )?;

    state.set_status("Build started - check status with Ctrl+L");

    Ok(())
}

/// Handle input for category manager
fn handle_category_manager_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    use category_manager::ManagerMode;

    match &state.category_manager_state.mode.clone() {
        ManagerMode::Browsing => {
            match key.code {
                KeyCode::Esc => {
                    state.active_popup = None;
                    state.set_status("Category manager closed");
                    Ok(false)
                }
                KeyCode::Up => {
                    state
                        .category_manager_state
                        .select_previous(state.layout.categories.len());
                    Ok(false)
                }
                KeyCode::Down => {
                    state
                        .category_manager_state
                        .select_next(state.layout.categories.len());
                    Ok(false)
                }
                KeyCode::Char('n') => {
                    // Start creating new category (T107)
                    state.category_manager_state.start_creating();
                    state.set_status("Enter category name (kebab-case, e.g., 'navigation')");
                    Ok(false)
                }
                KeyCode::Char('r') => {
                    // Start renaming (T109)
                    let selected_idx = state.category_manager_state.selected;
                    if let Some(category) = state.layout.categories.get(selected_idx) {
                        let category_clone = category.clone();
                        state.category_manager_state.start_renaming(&category_clone);
                        state.set_status("Enter new category name");
                    } else {
                        state.set_error("No category selected");
                    }
                    Ok(false)
                }
                KeyCode::Char('c') => {
                    // Change color (T110)
                    let selected_idx = state.category_manager_state.selected;
                    if let Some(category) = state.layout.categories.get(selected_idx) {
                        let color = category.color;
                        state.color_picker_state = ColorPickerState::with_color(color);
                        state.color_picker_context = Some(ColorPickerContext::Category);
                        state.active_popup = Some(PopupType::ColorPicker);
                        state.set_status("Set category color");
                    } else {
                        state.set_error("No category selected");
                    }
                    Ok(false)
                }
                KeyCode::Char('d') => {
                    // Start delete confirmation (T111)
                    let selected_idx = state.category_manager_state.selected;
                    if let Some(category) = state.layout.categories.get(selected_idx) {
                        let category_clone = category.clone();
                        state.category_manager_state.start_deleting(&category_clone);
                        state.set_status("Confirm deletion - y: Yes, n: No");
                    } else {
                        state.set_error("No category selected");
                    }
                    Ok(false)
                }
                KeyCode::Char('L') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    // Assign category to layer (T113)
                    let selected_idx = state.category_manager_state.selected;
                    if let Some(category) = state.layout.categories.get(selected_idx) {
                        let category_id = category.id.clone();
                        let category_name = category.name.clone();
                        if let Some(layer) = state.layout.layers.get_mut(state.current_layer) {
                            layer.category_id = Some(category_id);
                            state.mark_dirty();
                            state.set_status(format!(
                                "Layer {} assigned to category '{}'",
                                state.current_layer, category_name
                            ));
                        }
                    } else {
                        state.set_error("No category selected");
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::CreatingName { .. } | ManagerMode::Renaming { .. } => {
            // Handle text input
            match key.code {
                KeyCode::Esc => {
                    state.category_manager_state.cancel();
                    state.set_status("Cancelled");
                    Ok(false)
                }
                KeyCode::Enter => {
                    // Process the input
                    if let Some(input) = state.category_manager_state.get_input() {
                        let input = input.to_string();

                        match &state.category_manager_state.mode {
                            ManagerMode::CreatingName { .. } => {
                                // Generate ID from name (T107)
                                let id = input.to_lowercase().replace(' ', "-");

                                // Check if ID already exists
                                if state.layout.categories.iter().any(|c| c.id == id) {
                                    state.set_error("Category with this ID already exists");
                                    return Ok(false);
                                }

                                // Move to color selection (T108)
                                state.category_manager_state.mode =
                                    ManagerMode::CreatingColor { name: input };
                                state.color_picker_state = ColorPickerState::new();
                                state.color_picker_context = Some(ColorPickerContext::Category);
                                state.active_popup = Some(PopupType::ColorPicker);
                                state.set_status("Select color for new category");
                            }
                            ManagerMode::Renaming { category_id, .. } => {
                                // Update category name (T109)
                                if let Some(category) = state
                                    .layout
                                    .categories
                                    .iter_mut()
                                    .find(|c| &c.id == category_id)
                                {
                                    if let Err(e) = category.set_name(&input) {
                                        state.set_error(format!("Invalid name: {e}"));
                                        return Ok(false);
                                    }
                                    state.mark_dirty();
                                    state.category_manager_state.cancel();
                                    state.set_status(format!("Category renamed to '{input}'"));
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(false)
                }
                KeyCode::Char(c) => {
                    if let Some(input) = state.category_manager_state.get_input_mut() {
                        input.push(c);
                    }
                    Ok(false)
                }
                KeyCode::Backspace => {
                    if let Some(input) = state.category_manager_state.get_input_mut() {
                        input.pop();
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::ConfirmingDelete { category_id } => {
            match key.code {
                KeyCode::Char('y' | 'Y') => {
                    // Delete category (T111, T112)
                    let category_id = category_id.clone();

                    // Remove category
                    state.layout.categories.retain(|c| c.id != category_id);

                    // Clean up references in keys (T112)
                    for layer in &mut state.layout.layers {
                        if layer.category_id.as_ref() == Some(&category_id) {
                            layer.category_id = None;
                        }
                        for key in &mut layer.keys {
                            if key.category_id.as_ref() == Some(&category_id) {
                                key.category_id = None;
                            }
                        }
                    }

                    state.mark_dirty();
                    state.category_manager_state.cancel();

                    // Adjust selection if needed
                    if state.category_manager_state.selected >= state.layout.categories.len()
                        && state.category_manager_state.selected > 0
                    {
                        state.category_manager_state.selected -= 1;
                    }

                    state.set_status("Category deleted");
                    Ok(false)
                }
                KeyCode::Char('n' | 'N' | '\x1b') => {
                    state.category_manager_state.cancel();
                    state.set_status("Deletion cancelled");
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::CreatingColor { name: _ } => {
            // Color picker is handled by the color picker handler
            // We just need to handle the completion
            // This will be managed by returning from the color picker
            Ok(false)
        }
    }
}

/// Handle input for layer manager
fn handle_layer_manager_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    use layer_manager::ManagerMode;

    match &state.layer_manager_state.mode.clone() {
        ManagerMode::Browsing => {
            match key.code {
                KeyCode::Esc => {
                    state.active_popup = None;
                    state.set_status("Layer manager closed");
                    Ok(false)
                }
                KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    // Move layer up (reorder)
                    let selected = state.layer_manager_state.selected;
                    if selected > 0 && state.layout.layers.len() > 1 {
                        state.layout.layers.swap(selected, selected - 1);
                        state.layer_manager_state.selected = selected - 1;
                        // Renumber layers
                        for (i, layer) in state.layout.layers.iter_mut().enumerate() {
                            layer.number = i as u8;
                        }
                        state.mark_dirty();
                        state.set_status(format!("Layer moved up to position {}", selected - 1));
                    }
                    Ok(false)
                }
                KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    // Move layer down (reorder)
                    let selected = state.layer_manager_state.selected;
                    if selected < state.layout.layers.len() - 1 {
                        state.layout.layers.swap(selected, selected + 1);
                        state.layer_manager_state.selected = selected + 1;
                        // Renumber layers
                        for (i, layer) in state.layout.layers.iter_mut().enumerate() {
                            layer.number = i as u8;
                        }
                        state.mark_dirty();
                        state.set_status(format!("Layer moved down to position {}", selected + 1));
                    }
                    Ok(false)
                }
                KeyCode::Up => {
                    state
                        .layer_manager_state
                        .select_previous(state.layout.layers.len());
                    Ok(false)
                }
                KeyCode::Down => {
                    state
                        .layer_manager_state
                        .select_next(state.layout.layers.len());
                    Ok(false)
                }
                KeyCode::Enter => {
                    // Go to selected layer
                    state.current_layer = state.layer_manager_state.selected;
                    state.active_popup = None;
                    state.set_status(format!("Switched to layer {}", state.current_layer));
                    Ok(false)
                }
                KeyCode::Char('n') => {
                    // Start creating new layer
                    state.layer_manager_state.start_creating();
                    state.set_status("Enter layer name");
                    Ok(false)
                }
                KeyCode::Char('r') => {
                    // Start renaming
                    let selected_idx = state.layer_manager_state.selected;
                    if let Some(layer) = state.layout.layers.get(selected_idx) {
                        let layer_clone = layer.clone();
                        state.layer_manager_state.start_renaming(&layer_clone);
                        state.set_status("Enter new layer name");
                    } else {
                        state.set_error("No layer selected");
                    }
                    Ok(false)
                }
                KeyCode::Char('v') => {
                    // Toggle layer colors
                    let selected_idx = state.layer_manager_state.selected;
                    if let Some(enabled) = state.layout.toggle_layer_colors(selected_idx) {
                        state.mark_dirty();
                        state.set_status(if enabled {
                            format!("Layer {} colors enabled", selected_idx)
                        } else {
                            format!("Layer {} colors disabled", selected_idx)
                        });
                    }
                    Ok(false)
                }
                KeyCode::Char('d') => {
                    // Start delete confirmation
                    if state.layout.layers.len() <= 1 {
                        state.set_error("Cannot delete the last layer");
                    } else {
                        state.layer_manager_state.start_deleting();
                        state.set_status("Confirm deletion - y: Yes, n: No");
                    }
                    Ok(false)
                }
                KeyCode::Char('D') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    // Start duplicating
                    let selected_idx = state.layer_manager_state.selected;
                    if let Some(layer) = state.layout.layers.get(selected_idx) {
                        let layer_clone = layer.clone();
                        state.layer_manager_state.start_duplicating(&layer_clone);
                        state.set_status("Enter name for duplicate layer");
                    }
                    Ok(false)
                }
                KeyCode::Char('c') => {
                    // Start copy to another layer
                    if state.layout.layers.len() <= 1 {
                        state.set_error("Need at least 2 layers to copy");
                    } else {
                        state.layer_manager_state.start_copy_to(state.layout.layers.len());
                        state.set_status("Select target layer");
                    }
                    Ok(false)
                }
                KeyCode::Char('s') => {
                    // Start swap with another layer
                    if state.layout.layers.len() <= 1 {
                        state.set_error("Need at least 2 layers to swap");
                    } else {
                        state.layer_manager_state.start_swapping(state.layout.layers.len());
                        state.set_status("Select layer to swap with");
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::CreatingName { .. } | ManagerMode::Renaming { .. } | ManagerMode::Duplicating { .. } => {
            // Handle text input
            match key.code {
                KeyCode::Esc => {
                    state.layer_manager_state.cancel();
                    state.set_status("Cancelled");
                    Ok(false)
                }
                KeyCode::Enter => {
                    // Process the input
                    if let Some(input) = state.layer_manager_state.get_input() {
                        let input = input.to_string();

                        if input.trim().is_empty() {
                            state.set_error("Layer name cannot be empty");
                            return Ok(false);
                        }

                        match &state.layer_manager_state.mode {
                            ManagerMode::CreatingName { .. } => {
                                // Create new layer
                                use crate::models::layer::Layer;
                                use crate::models::RgbColor;
                                let new_index = state.layout.layers.len();
                                
                                // Use a default color (gray) for new layers
                                let default_color = RgbColor::new(128, 128, 128);
                                
                                match Layer::new(new_index as u8, &input, default_color) {
                                    Ok(mut new_layer) => {
                                        // Copy key positions from first layer (with transparent keycodes)
                                        if let Some(first_layer) = state.layout.layers.first() {
                                            for key in &first_layer.keys {
                                                use crate::models::layer::KeyDefinition;
                                                new_layer.add_key(KeyDefinition::new(key.position, "KC_TRNS"));
                                            }
                                        }
                                        
                                        state.layout.layers.push(new_layer);
                                        state.layer_manager_state.selected = new_index;
                                        state.mark_dirty();
                                        state.layer_manager_state.cancel();
                                        state.set_status(format!("Layer '{}' created", input));
                                    }
                                    Err(e) => {
                                        state.set_error(format!("Failed to create layer: {}", e));
                                    }
                                }
                            }
                            ManagerMode::Renaming { layer_index, .. } => {
                                // Update layer name
                                let layer_index = *layer_index;
                                if let Some(layer) = state.layout.layers.get_mut(layer_index) {
                                    layer.name = input.clone();
                                    state.mark_dirty();
                                    state.layer_manager_state.cancel();
                                    state.set_status(format!("Layer renamed to '{}'", input));
                                }
                            }
                            ManagerMode::Duplicating { source_index, .. } => {
                                // Duplicate layer with new name
                                use crate::models::layer::Layer;
                                let source_index = *source_index;
                                let new_index = state.layout.layers.len();
                                
                                if let Some(source) = state.layout.layers.get(source_index) {
                                    match Layer::new(new_index as u8, &input, source.default_color) {
                                        Ok(mut new_layer) => {
                                            // Copy all keys from source
                                            for key in &source.keys {
                                                use crate::models::layer::KeyDefinition;
                                                let mut new_key = KeyDefinition::new(key.position, &key.keycode);
                                                new_key.color_override = key.color_override;
                                                new_key.category_id = key.category_id.clone();
                                                new_layer.add_key(new_key);
                                            }
                                            // Copy layer settings
                                            new_layer.layer_colors_enabled = source.layer_colors_enabled;
                                            new_layer.category_id = source.category_id.clone();
                                            
                                            state.layout.layers.push(new_layer);
                                            state.layer_manager_state.selected = new_index;
                                            state.mark_dirty();
                                            state.layer_manager_state.cancel();
                                            state.set_status(format!("Duplicated as '{}'", input));
                                        }
                                        Err(e) => {
                                            state.set_error(format!("Failed to duplicate layer: {}", e));
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(false)
                }
                KeyCode::Char(c) => {
                    if let Some(input) = state.layer_manager_state.get_input_mut() {
                        input.push(c);
                    }
                    Ok(false)
                }
                KeyCode::Backspace => {
                    if let Some(input) = state.layer_manager_state.get_input_mut() {
                        input.pop();
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::ConfirmingDelete { layer_index } => {
            let layer_index = *layer_index;
            match key.code {
                KeyCode::Char('y' | 'Y') => {
                    // Delete layer (only if not the last one)
                    if state.layout.layers.len() > 1 {
                        state.layout.layers.remove(layer_index);

                        // Renumber remaining layers
                        for (i, layer) in state.layout.layers.iter_mut().enumerate() {
                            layer.number = i as u8;
                        }

                        // Adjust current layer if needed
                        if state.current_layer >= state.layout.layers.len() {
                            state.current_layer = state.layout.layers.len() - 1;
                        }

                        // Adjust selection if needed
                        if state.layer_manager_state.selected >= state.layout.layers.len() {
                            state.layer_manager_state.selected = state.layout.layers.len() - 1;
                        }

                        state.mark_dirty();
                        state.layer_manager_state.cancel();
                        state.set_status("Layer deleted");
                    } else {
                        state.set_error("Cannot delete the last layer");
                        state.layer_manager_state.cancel();
                    }
                    Ok(false)
                }
                KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                    state.layer_manager_state.cancel();
                    state.set_status("Deletion cancelled");
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::CopyingTo { source_index, target_selected } => {
            let source_index = *source_index;
            let target_selected = *target_selected;
            match key.code {
                KeyCode::Esc => {
                    state.layer_manager_state.cancel();
                    state.set_status("Copy cancelled");
                    Ok(false)
                }
                KeyCode::Up => {
                    state.layer_manager_state.select_target_previous(state.layout.layers.len());
                    Ok(false)
                }
                KeyCode::Down => {
                    state.layer_manager_state.select_target_next(state.layout.layers.len());
                    Ok(false)
                }
                KeyCode::Enter => {
                    // Copy all keys from source to target
                    if let Some(source) = state.layout.layers.get(source_index) {
                        let keys_to_copy: Vec<_> = source.keys.iter().map(|k| {
                            (k.position, k.keycode.clone(), k.color_override, k.category_id.clone())
                        }).collect();
                        
                        if let Some(target) = state.layout.layers.get_mut(target_selected) {
                            for (pos, keycode, color, category) in keys_to_copy {
                                if let Some(key) = target.keys.iter_mut().find(|k| k.position == pos) {
                                    key.keycode = keycode;
                                    key.color_override = color;
                                    key.category_id = category;
                                }
                            }
                            state.mark_dirty();
                            state.layer_manager_state.cancel();
                            state.set_status(format!("Copied layer {} to layer {}", source_index, target_selected));
                        }
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::Swapping { source_index, target_selected } => {
            let source_index = *source_index;
            let target_selected = *target_selected;
            match key.code {
                KeyCode::Esc => {
                    state.layer_manager_state.cancel();
                    state.set_status("Swap cancelled");
                    Ok(false)
                }
                KeyCode::Up => {
                    state.layer_manager_state.select_target_previous(state.layout.layers.len());
                    Ok(false)
                }
                KeyCode::Down => {
                    state.layer_manager_state.select_target_next(state.layout.layers.len());
                    Ok(false)
                }
                KeyCode::Enter => {
                    // Swap all keys between source and target
                    // We need to collect both sets of keys first to avoid borrow issues
                    let source_keys: Vec<_>;
                    let target_keys: Vec<_>;
                    
                    if let (Some(source), Some(target)) = (
                        state.layout.layers.get(source_index),
                        state.layout.layers.get(target_selected),
                    ) {
                        source_keys = source.keys.iter().map(|k| {
                            (k.position, k.keycode.clone(), k.color_override, k.category_id.clone())
                        }).collect();
                        target_keys = target.keys.iter().map(|k| {
                            (k.position, k.keycode.clone(), k.color_override, k.category_id.clone())
                        }).collect();
                    } else {
                        return Ok(false);
                    }
                    
                    // Apply target keys to source layer
                    if let Some(source) = state.layout.layers.get_mut(source_index) {
                        for (pos, keycode, color, category) in &target_keys {
                            if let Some(key) = source.keys.iter_mut().find(|k| k.position == *pos) {
                                key.keycode = keycode.clone();
                                key.color_override = *color;
                                key.category_id = category.clone();
                            }
                        }
                    }
                    
                    // Apply source keys to target layer
                    if let Some(target) = state.layout.layers.get_mut(target_selected) {
                        for (pos, keycode, color, category) in &source_keys {
                            if let Some(key) = target.keys.iter_mut().find(|k| k.position == *pos) {
                                key.keycode = keycode.clone();
                                key.color_override = *color;
                                key.category_id = category.clone();
                            }
                        }
                    }
                    
                    state.mark_dirty();
                    state.layer_manager_state.cancel();
                    state.set_status(format!("Swapped layers {} and {}", source_index, target_selected));
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
    }
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
        // "rev2" is not a variant subdirectory, so it should be kept
        let result = AppState::extract_base_keyboard("manufacturer/keyboard/rev2");
        assert_eq!(result, "manufacturer/keyboard/rev2");
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
        assert!(state.keycode_type.is_none());
        assert!(state.param1.is_none());
        assert!(state.param2.is_none());
    }

    #[test]
    fn test_pending_keycode_reset() {
        let mut state = PendingKeycodeState::new();
        state.keycode_type = Some(ParameterizedKeycodeType::LayerTap);
        state.param1 = Some("@layer-uuid".to_string());
        state.param2 = Some("KC_SPC".to_string());

        state.reset();

        assert!(state.keycode_type.is_none());
        assert!(state.param1.is_none());
        assert!(state.param2.is_none());
    }

    #[test]
    fn test_build_keycode_layer_tap() {
        let mut state = PendingKeycodeState::new();
        state.keycode_type = Some(ParameterizedKeycodeType::LayerTap);
        state.param1 = Some("@abc-123".to_string());
        state.param2 = Some("KC_SPC".to_string());

        let result = state.build_keycode();
        assert_eq!(result, Some("LT(@abc-123, KC_SPC)".to_string()));
    }

    #[test]
    fn test_build_keycode_mod_tap() {
        let mut state = PendingKeycodeState::new();
        state.keycode_type = Some(ParameterizedKeycodeType::ModTap);
        state.param1 = Some("MOD_LCTL | MOD_LSFT".to_string());
        state.param2 = Some("KC_A".to_string());

        let result = state.build_keycode();
        assert_eq!(result, Some("MT(MOD_LCTL | MOD_LSFT, KC_A)".to_string()));
    }

    #[test]
    fn test_build_keycode_layer_mod() {
        let mut state = PendingKeycodeState::new();
        state.keycode_type = Some(ParameterizedKeycodeType::LayerMod);
        state.param1 = Some("@layer-uuid".to_string());
        state.param2 = Some("MOD_LSFT".to_string());

        let result = state.build_keycode();
        assert_eq!(result, Some("LM(@layer-uuid, MOD_LSFT)".to_string()));
    }

    #[test]
    fn test_build_keycode_swap_hands_tap() {
        let mut state = PendingKeycodeState::new();
        state.keycode_type = Some(ParameterizedKeycodeType::SwapHandsTap);
        state.param1 = Some("KC_SPC".to_string());

        let result = state.build_keycode();
        assert_eq!(result, Some("SH_T(KC_SPC)".to_string()));
    }

    #[test]
    fn test_build_keycode_incomplete_lt() {
        let mut state = PendingKeycodeState::new();
        state.keycode_type = Some(ParameterizedKeycodeType::LayerTap);
        state.param1 = Some("@abc-123".to_string());
        // Missing param2

        let result = state.build_keycode();
        assert!(result.is_none(), "Incomplete LT should return None");
    }

    #[test]
    fn test_build_keycode_no_type() {
        let state = PendingKeycodeState::new();
        let result = state.build_keycode();
        assert!(result.is_none(), "No keycode type should return None");
    }
}
