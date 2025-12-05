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
use crate::services::geometry::{build_geometry_for_layout, extract_base_keyboard, GeometryContext};

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

// Import handler functions from the handlers module

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
    /// `SH_T(kc)` - Hold to swap hands, tap for keycode
    SwapHandsTap,
    /// Simple mod-tap like `LCTL_T(kc)`, `LSFT_T(kc)` - modifier is fixed, only need tap keycode
    /// param1 stores the prefix (e.g., "`LCTL_T`", "`LGUI_T`")
    SimpleModTap,
    /// Single modifier keycode like OSM(mod) - only needs modifier selection
    /// param1 stores the prefix (e.g., "OSM")
    SingleMod,
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
                Some(format!("LT({layer}, {keycode})"))
            }
            Some(ParameterizedKeycodeType::ModTap) => {
                let modifier = self.param1.as_ref()?;
                let keycode = self.param2.as_ref()?;
                Some(format!("MT({modifier}, {keycode})"))
            }
            Some(ParameterizedKeycodeType::LayerMod) => {
                let layer = self.param1.as_ref()?;
                let modifier = self.param2.as_ref()?;
                Some(format!("LM({layer}, {modifier})"))
            }
            Some(ParameterizedKeycodeType::SwapHandsTap) => {
                let keycode = self.param1.as_ref()?;
                Some(format!("SH_T({keycode})"))
            }
            Some(ParameterizedKeycodeType::SimpleModTap) => {
                // param1 is the prefix (e.g., "LCTL_T"), param2 is the tap keycode
                let prefix = self.param1.as_ref()?;
                let keycode = self.param2.as_ref()?;
                Some(format!("{prefix}({keycode})"))
            }
            Some(ParameterizedKeycodeType::SingleMod) => {
                // param1 is the prefix (e.g., "OSM"), param2 is the modifier
                let prefix = self.param1.as_ref()?;
                let modifier = self.param2.as_ref()?;
                Some(format!("{prefix}({modifier})"))
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
            let keyboard = state.layout.metadata.keyboard.as_deref().unwrap_or("");
            config_dialogs::render_layout_picker(
                f,
                &state.layout_picker_state,
                keyboard,
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
                state.layout.rgb_enabled,
                state.layout.rgb_brightness,
                state.layout.rgb_timeout_ms,
                state.layout.uncolored_key_behavior,
                &state.layout.tap_hold_settings,
                &state.config,
                &state.layout,
                &state.theme,
            );
        }
        PopupType::TapKeycodePicker => {
            // Reuse keycode picker rendering with custom title context
            // The title will show "Select Tap Keycode" based on pending_keycode state
            keycode_picker::render_keycode_picker(f, state);
        }
        PopupType::ModifierPicker => {
            modifier_picker::render_modifier_picker(f, &state.modifier_picker_state, &state.theme);
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

/// Render template save dialog
fn render_template_save_dialog(f: &mut Frame, state: &AppState) {
    let area = centered_rect(70, 60, f.size());

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
