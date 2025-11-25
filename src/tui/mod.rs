//! Terminal user interface components and state management.
//!
//! This module contains the main TUI loop, `AppState`, event handling,
//! and all UI widgets using Ratatui.

pub mod build_log;
pub mod category_manager;
pub mod category_picker;
pub mod color_picker;
pub mod config_dialogs;
pub mod help_overlay;
pub mod keyboard;
pub mod keycode_picker;
pub mod metadata_editor;
#[allow(dead_code)]
pub mod onboarding_wizard;
pub mod status_bar;
pub mod template_browser;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout as RatatuiLayout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
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
pub use keyboard::KeyboardWidget;
pub use keycode_picker::KeycodePickerState;
pub use metadata_editor::MetadataEditorState;
pub use status_bar::StatusBar;
pub use template_browser::TemplateBrowserState;

/// Color picker context - what are we setting the color for?
#[derive(Debug, Clone, PartialEq)]
pub enum ColorPickerContext {
    /// Setting color for individual key
    IndividualKey,
    /// Setting layer default color
    LayerDefault,
    /// Setting color for new or existing category
    Category,
}

/// Category picker context - what are we setting the category for?
#[derive(Debug, Clone, PartialEq)]
pub enum CategoryPickerContext {
    /// Setting category for individual key
    IndividualKey,
    /// Setting category for entire layer
    Layer,
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
    pub fn new(layout_name: String) -> Self {
        Self {
            active_field: TemplateSaveField::Name,
            name: layout_name,
            description: String::new(),
            author: String::new(),
            tags_input: String::new(),
        }
    }

    /// Get the active field's input string (mutable).
    pub fn get_active_field_mut(&mut self) -> &mut String {
        match self.active_field {
            TemplateSaveField::Name => &mut self.name,
            TemplateSaveField::Description => &mut self.description,
            TemplateSaveField::Author => &mut self.author,
            TemplateSaveField::Tags => &mut self.tags_input,
        }
    }

    /// Move to the next field.
    pub fn next_field(&mut self) {
        self.active_field = match self.active_field {
            TemplateSaveField::Name => TemplateSaveField::Description,
            TemplateSaveField::Description => TemplateSaveField::Author,
            TemplateSaveField::Author => TemplateSaveField::Tags,
            TemplateSaveField::Tags => TemplateSaveField::Name,
        };
    }

    /// Move to the previous field.
    pub fn previous_field(&mut self) {
        self.active_field = match self.active_field {
            TemplateSaveField::Name => TemplateSaveField::Tags,
            TemplateSaveField::Description => TemplateSaveField::Name,
            TemplateSaveField::Author => TemplateSaveField::Description,
            TemplateSaveField::Tags => TemplateSaveField::Author,
        };
    }

    /// Parse tags from comma-separated input.
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemplateSaveField {
    Name,
    Description,
    Author,
    Tags,
}

/// Popup types that can be displayed over the main UI
#[derive(Debug, Clone, PartialEq)]
pub enum PopupType {
    KeycodePicker,
    ColorPicker,
    CategoryPicker,
    CategoryManager,
    TemplateBrowser,
    TemplateSaveDialog,
    HelpOverlay,
    BuildLog,
    MetadataEditor,
    UnsavedChangesPrompt,
    LayoutPicker,
}

/// Application state - single source of truth
///
/// All UI components read from this state immutably.
/// Only event handlers modify state explicitly.
pub struct AppState {
    // Core data
    pub layout: Layout,
    pub source_path: Option<PathBuf>,
    pub dirty: bool,

    // UI state
    pub current_layer: usize,
    pub selected_position: Position,
    pub active_popup: Option<PopupType>,
    pub status_message: String,
    pub error_message: Option<String>,

    // Component states
    pub keycode_picker_state: KeycodePickerState,
    pub color_picker_state: ColorPickerState,
    pub color_picker_context: Option<ColorPickerContext>,
    pub category_picker_state: CategoryPickerState,
    pub category_picker_context: Option<CategoryPickerContext>,
    pub category_manager_state: CategoryManagerState,
    pub build_log_state: build_log::BuildLogState,
    pub template_browser_state: TemplateBrowserState,
    pub template_save_dialog_state: TemplateSaveDialogState,
    pub metadata_editor_state: MetadataEditorState,
    pub help_overlay_state: HelpOverlayState,
    pub layout_picker_state: config_dialogs::LayoutPickerState,

    // System resources
    pub keycode_db: KeycodeDb,
    pub geometry: KeyboardGeometry,
    pub mapping: VisualLayoutMapping,
    pub config: Config,

    // Firmware build state
    pub build_state: Option<BuildState>,

    // Control flags
    pub should_quit: bool,
}

impl AppState {
    /// Create a new AppState with the given layout and resources
    pub fn new(
        layout: Layout,
        source_path: Option<PathBuf>,
        geometry: KeyboardGeometry,
        mapping: VisualLayoutMapping,
        config: Config,
    ) -> Result<Self> {
        let keycode_db = KeycodeDb::load().context("Failed to load keycode database")?;

        Ok(Self {
            layout,
            source_path,
            dirty: false,
            current_layer: 0,
            selected_position: Position { row: 0, col: 0 },
            active_popup: None,
            status_message: "Press ? for help".to_string(),
            error_message: None,
            keycode_picker_state: KeycodePickerState::new(),
            color_picker_state: ColorPickerState::new(),
            color_picker_context: None,
            category_picker_state: CategoryPickerState::new(),
            category_picker_context: None,
            category_manager_state: CategoryManagerState::new(),
            build_log_state: build_log::BuildLogState::new(),
            template_browser_state: TemplateBrowserState::new(),
            template_save_dialog_state: TemplateSaveDialogState::default(),
            metadata_editor_state: MetadataEditorState::default(),
            help_overlay_state: HelpOverlayState::new(),
            layout_picker_state: config_dialogs::LayoutPickerState::new(),
            keycode_db,
            geometry,
            mapping,
            config,
            build_state: None,
            should_quit: false,
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
    pub fn get_selected_key(&self) -> Option<&crate::models::KeyDefinition> {
        let layer = self.layout.layers.get(self.current_layer)?;
        layer
            .keys
            .iter()
            .find(|k| k.position == self.selected_position)
    }

    /// Mark layout as dirty (unsaved changes)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clear dirty flag (after save)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Rebuild keyboard geometry and visual layout mapping for a new layout variant.
    ///
    /// # Arguments
    ///
    /// * `layout_name` - Name of the layout variant (e.g., "LAYOUT_split_3x6_3")
    ///
    /// # Returns
    ///
    /// Result indicating success or error with context
    pub fn rebuild_geometry(&mut self, layout_name: &str) -> Result<()> {
        use crate::models::VisualLayoutMapping;
        use crate::parser::keyboard_json::{build_keyboard_geometry, parse_keyboard_info_json};

        // Parse keyboard info.json to get layout definition
        let qmk_path = self
            .config
            .paths
            .qmk_firmware
            .as_ref()
            .context("QMK firmware path not configured")?;

        let info = parse_keyboard_info_json(qmk_path, &self.config.build.keyboard)
            .context("Failed to parse keyboard info.json")?;

        // Build new geometry for selected layout
        let new_geometry = build_keyboard_geometry(&info, &self.config.build.keyboard, layout_name)
            .context("Failed to build keyboard geometry")?;

        // Build new visual layout mapping
        let new_mapping = VisualLayoutMapping::build(&new_geometry);

        // Update AppState with new geometry and mapping
        self.geometry = new_geometry;
        self.mapping = new_mapping;

        // Update config to persist layout choice
        self.config.build.layout = layout_name.to_string();

        // Reset selection to (0, 0) to avoid out-of-bounds
        self.selected_position = Position { row: 0, col: 0 };

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
            Constraint::Length(3), // Status bar
        ])
        .split(f.size());

    // Title bar with dirty indicator
    render_title_bar(f, chunks[0], state);

    // Main content area
    render_main_content(f, chunks[1], state);

    // Status bar
    StatusBar::render(f, chunks[2], state);

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
        .style(Style::default().fg(Color::Cyan))
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
            );
        }
        PopupType::TemplateBrowser => {
            template_browser::render(f, &state.template_browser_state, f.size());
        }
        PopupType::TemplateSaveDialog => {
            render_template_save_dialog(f, state);
        }
        PopupType::UnsavedChangesPrompt => {
            render_unsaved_prompt(f);
        }
        PopupType::BuildLog => {
            if let Some(build_state) = &state.build_state {
                build_log::render_build_log(f, build_state, &state.build_log_state);
            }
        }
        PopupType::HelpOverlay => {
            state.help_overlay_state.render(f, f.size());
        }
        PopupType::LayoutPicker => {
            config_dialogs::render_layout_picker(
                f,
                &state.layout_picker_state,
                &state.config.build.keyboard,
            );
        }
        PopupType::MetadataEditor => {
            metadata_editor::render_metadata_editor(f, &state.metadata_editor_state);
        }
    }
}

/// Render unsaved changes prompt
fn render_unsaved_prompt(f: &mut Frame) {
    let area = centered_rect(60, 30, f.size());

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
            .style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(prompt, area);
}

/// Render template save dialog
fn render_template_save_dialog(f: &mut Frame, state: &AppState) {
    let area = centered_rect(70, 60, f.size());

    let dialog_state = &state.template_save_dialog_state;

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
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Name field
    let name_style = if matches!(dialog_state.active_field, TemplateSaveField::Name) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
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
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
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
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
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
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
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
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::Gray));
    f.render_widget(help, chunks[5]);

    // Action buttons
    let actions = Paragraph::new("Enter: save template | Esc: cancel")
        .style(Style::default().fg(Color::Green))
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
        Some(PopupType::TemplateBrowser) => handle_template_browser_input(state, key),
        Some(PopupType::TemplateSaveDialog) => handle_template_save_dialog_input(state, key),
        Some(PopupType::UnsavedChangesPrompt) => handle_unsaved_prompt_input(state, key),
        Some(PopupType::BuildLog) => handle_build_log_input(state, key),
        Some(PopupType::HelpOverlay) => handle_help_overlay_input(state, key),
        Some(PopupType::MetadataEditor) => handle_metadata_editor_input(state, key),
        Some(PopupType::LayoutPicker) => handle_layout_picker_input(state, key),
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
            // Validate and apply changes
            match state
                .metadata_editor_state
                .apply_to_layout(&mut state.layout)
            {
                Ok(()) => {
                    state.mark_dirty();
                    state.active_popup = None;
                    state.set_status("Metadata updated");
                }
                Err(err) => {
                    state.set_error(format!("Validation failed: {}", err));
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
            state.active_popup = None;
            state.set_status("Layout selection cancelled");
            return Ok(false);
        }

        // User selected a layout - rebuild geometry and mapping
        match state.rebuild_geometry(&selected) {
            Ok(()) => {
                state.active_popup = None;
                state.set_status(format!("Switched to layout: {}", selected));
                state.mark_dirty(); // Config change requires save
            }
            Err(e) => {
                state.set_error(format!("Failed to switch layout: {}", e));
            }
        }
    }
    Ok(false)
}

/// Handle input for unsaved changes prompt
fn handle_unsaved_prompt_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('s') | KeyCode::Char('S') => {
            // Save and quit
            if let Some(path) = &state.source_path.clone() {
                crate::parser::save_markdown_layout(&state.layout, path)?;
                state.mark_clean();
                state.set_status("Saved");
            }
            state.should_quit = true;
            Ok(true)
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => {
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
                        state.set_error(format!("Failed to load template: {}", e));
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
                        state.set_error(format!("Failed to load template: {}", e));
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
                .replace(" ", "-")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>();
            let template_path = templates_dir.join(format!("{}.md", filename));

            // Save template
            match crate::parser::save_markdown_layout(&template_layout, &template_path) {
                Ok(_) => {
                    state.active_popup = None;
                    state.set_status(format!("Template saved: {}", template_path.display()));
                    Ok(false)
                }
                Err(e) => {
                    state.set_error(format!("Failed to save template: {}", e));
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

        // Navigation - Arrow keys
        (KeyCode::Up, _) => {
            if state.selected_position.row > 0 {
                state.selected_position.row -= 1;
                state.clear_error();
            }
            Ok(false)
        }
        (KeyCode::Down, _) => {
            // Check bounds based on geometry
            if state.selected_position.row < 3 {
                state.selected_position.row += 1;
                state.clear_error();
            }
            Ok(false)
        }
        (KeyCode::Left, _) => {
            if state.selected_position.col > 0 {
                state.selected_position.col -= 1;
                state.clear_error();
            }
            Ok(false)
        }
        (KeyCode::Right, _) => {
            // Check bounds based on geometry
            if state.selected_position.col < 13 {
                state.selected_position.col += 1;
                state.clear_error();
            }
            Ok(false)
        }

        // Navigation - VIM style (hjkl)
        (KeyCode::Char('h'), _) => {
            if state.selected_position.col > 0 {
                state.selected_position.col -= 1;
                state.clear_error();
            }
            Ok(false)
        }
        (KeyCode::Char('j'), _) => {
            if state.selected_position.row < 3 {
                state.selected_position.row += 1;
                state.clear_error();
            }
            Ok(false)
        }
        (KeyCode::Char('k'), _) => {
            if state.selected_position.row > 0 {
                state.selected_position.row -= 1;
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
            if state.selected_position.col < 13 {
                state.selected_position.col += 1;
                state.clear_error();
            }
            Ok(false)
        }

        // Layer switching
        (KeyCode::Tab, KeyModifiers::SHIFT) => {
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

        // Clear key
        (KeyCode::Char('x'), _) | (KeyCode::Delete, _) => {
            if let Some(key) = state.get_selected_key_mut() {
                key.keycode = "KC_TRNS".to_string();
                state.mark_dirty();
                state.set_status("Key cleared (KC_TRNS)");
            }
            Ok(false)
        }

        // Open keycode picker
        (KeyCode::Enter, _) => {
            state.active_popup = Some(PopupType::KeycodePicker);
            state.keycode_picker_state = KeycodePickerState::new();
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
            let qmk_path = match &state.config.paths.qmk_firmware {
                Some(path) => path.clone(),
                None => {
                    state.set_error("QMK firmware path not configured");
                    return Ok(false);
                }
            };

            if let Err(e) = state
                .layout_picker_state
                .load_layouts(&qmk_path, &state.config.build.keyboard)
            {
                state.set_error(format!("Failed to load layouts: {}", e));
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
                    state.set_error(format!("Failed to scan templates: {}", e));
                    return Ok(false);
                }
                state.active_popup = Some(PopupType::TemplateBrowser);
                state.set_status("Select a template - Enter to load");
            } else {
                state.template_browser_state = TemplateBrowserState::new();
                if let Err(e) = state.template_browser_state.scan_templates() {
                    state.set_error(format!("Failed to scan templates: {}", e));
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
        state.set_error(format!("Validation failed:\n{}", error_msg));
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
        Ok((keymap_path, vial_path)) => {
            state.set_status(format!("✓ Generated: {} and {}", keymap_path, vial_path));
        }
        Err(e) => {
            state.set_error(format!("Generation failed: {}", e));
        }
    }

    Ok(())
}

/// Handle firmware build in background
fn handle_firmware_build(state: &mut AppState) -> Result<()> {
    // Check that QMK firmware path is configured
    let qmk_path = match &state.config.paths.qmk_firmware {
        Some(path) => path.clone(),
        None => {
            state.set_error("QMK firmware path not configured");
            return Ok(());
        }
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

    // Start the build
    build_state.start_build(
        qmk_path,
        state.config.build.keyboard.clone(),
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
                                        state.set_error(format!("Invalid name: {}", e));
                                        return Ok(false);
                                    }
                                    state.mark_dirty();
                                    state.category_manager_state.cancel();
                                    state.set_status(format!("Category renamed to '{}'", input));
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
                KeyCode::Char('y') | KeyCode::Char('Y') => {
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
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
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
