//! Terminal user interface components and state management.
//!
//! This module contains the main TUI loop, `AppState`, event handling,
//! and all UI widgets using Ratatui.

pub mod config_dialogs;
pub mod onboarding_wizard;
pub mod keyboard;
pub mod status_bar;
pub mod keycode_picker;
pub mod color_picker;
pub mod category_picker;

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
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crate::config::Config;
use crate::keycode_db::KeycodeDb;
use crate::models::{KeyboardGeometry, Layout, Position, VisualLayoutMapping};

// Re-export TUI components
pub use config_dialogs::{
    KeyboardPickerState, LayoutPickerState, PathConfigDialogState,
};
pub use keyboard::KeyboardWidget;
pub use keycode_picker::KeycodePickerState;
pub use onboarding_wizard::OnboardingWizardState;
pub use status_bar::StatusBar;
pub use color_picker::{ColorPickerState, RgbChannel};
pub use category_picker::CategoryPickerState;

/// Color picker context - what are we setting the color for?
#[derive(Debug, Clone, PartialEq)]
pub enum ColorPickerContext {
    /// Setting color for individual key
    IndividualKey,
    /// Setting layer default color
    LayerDefault,
}

/// Category picker context - what are we setting the category for?
#[derive(Debug, Clone, PartialEq)]
pub enum CategoryPickerContext {
    /// Setting category for individual key
    IndividualKey,
    /// Setting category for entire layer
    Layer,
}

/// Popup types that can be displayed over the main UI
#[derive(Debug, Clone, PartialEq)]
pub enum PopupType {
    KeycodePicker,
    ColorPicker,
    CategoryPicker,
    CategoryManager,
    TemplateBrowser,
    HelpOverlay,
    BuildLog,
    MetadataEditor,
    UnsavedChangesPrompt,
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

    // System resources
    pub keycode_db: KeycodeDb,
    pub geometry: KeyboardGeometry,
    pub mapping: VisualLayoutMapping,
    pub config: Config,

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
        let keycode_db = KeycodeDb::load()
            .context("Failed to load keycode database")?;

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
            keycode_db,
            geometry,
            mapping,
            config,
            should_quit: false,
        })
    }

    /// Get the currently selected key (mutable)
    pub fn get_selected_key_mut(&mut self) -> Option<&mut crate::models::KeyDefinition> {
        let layer = self.layout.layers.get_mut(self.current_layer)?;
        layer.keys.iter_mut()
            .find(|k| k.position == self.selected_position)
    }

    /// Get the currently selected key (immutable)
    pub fn get_selected_key(&self) -> Option<&crate::models::KeyDefinition> {
        let layer = self.layout.layers.get(self.current_layer)?;
        layer.keys.iter()
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
pub fn run_tui(state: &mut AppState, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
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
            Constraint::Length(3),      // Title bar
            Constraint::Min(10),         // Main content
            Constraint::Length(3),       // Status bar
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
        state.layout.metadata.name,
        state.current_layer,
        dirty_indicator
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
        PopupType::UnsavedChangesPrompt => {
            render_unsaved_prompt(f);
        }
        _ => {
            // Other popups not implemented yet
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

    let prompt = Paragraph::new(text)
        .block(Block::default()
            .title(" Unsaved Changes ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow)));

    f.render_widget(prompt, area);
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
        Some(PopupType::KeycodePicker) => {
            keycode_picker::handle_input(state, key)
        }
        Some(PopupType::ColorPicker) => {
            color_picker::handle_input(state, key)
        }
        Some(PopupType::CategoryPicker) => {
            category_picker::handle_input(state, key)
        }
        Some(PopupType::UnsavedChangesPrompt) => {
            handle_unsaved_prompt_input(state, key)
        }
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
                state.set_status("Adjust color with arrows, Tab to switch channels, Enter to apply");
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

        // Help
        (KeyCode::Char('?'), _) => {
            state.set_status("Help system not implemented yet - coming in Phase 10");
            Ok(false)
        }

        _ => Ok(false),
    }
}
