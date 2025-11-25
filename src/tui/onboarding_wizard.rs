//! Onboarding wizard for first-time setup.
//!
//! This module implements a step-by-step wizard to guide users through
//! initial configuration: QMK path, keyboard selection, and layout variant.
//!
//! NOTE: This module is currently unused but preserved for future onboarding features.

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::Config;
use crate::parser::keyboard_json::{
    extract_layout_names, parse_keyboard_info_json, scan_keyboards,
};

/// Onboarding wizard steps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardStep {
    /// Welcome screen
    Welcome,
    /// Enter QMK firmware path
    QmkPath,
    /// Select keyboard from scanned list
    KeyboardSelection,
    /// Select layout variant
    LayoutSelection,
    /// Confirmation and save
    Confirmation,
}

impl WizardStep {
    /// Gets the next step in the wizard
    #[must_use] pub const fn next(&self) -> Option<Self> {
        match self {
            Self::Welcome => Some(Self::QmkPath),
            Self::QmkPath => Some(Self::KeyboardSelection),
            Self::KeyboardSelection => Some(Self::LayoutSelection),
            Self::LayoutSelection => Some(Self::Confirmation),
            Self::Confirmation => None,
        }
    }

    /// Gets the previous step in the wizard
    #[must_use] pub const fn previous(&self) -> Option<Self> {
        match self {
            Self::Welcome => None,
            Self::QmkPath => Some(Self::Welcome),
            Self::KeyboardSelection => Some(Self::QmkPath),
            Self::LayoutSelection => Some(Self::KeyboardSelection),
            Self::Confirmation => Some(Self::LayoutSelection),
        }
    }

    /// Gets the step title
    #[must_use] pub const fn title(&self) -> &'static str {
        match self {
            Self::Welcome => "Welcome to Keyboard TUI",
            Self::QmkPath => "QMK Firmware Path",
            Self::KeyboardSelection => "Select Keyboard",
            Self::LayoutSelection => "Select Layout",
            Self::Confirmation => "Confirm Configuration",
        }
    }
}

/// Onboarding wizard state
#[derive(Debug, Clone)]
pub struct OnboardingWizardState {
    /// Current wizard step
    pub current_step: WizardStep,
    /// User inputs collected so far
    pub inputs: HashMap<String, String>,
    /// Current text input buffer
    pub input_buffer: String,
    /// List of available keyboards (populated after QMK path is set)
    pub available_keyboards: Vec<String>,
    /// Filter text for keyboard search
    pub keyboard_filter: String,
    /// Selected keyboard index in list
    pub keyboard_selected_index: usize,
    /// List of available layouts (populated after keyboard is selected)
    pub available_layouts: Vec<String>,
    /// Selected layout index in list
    pub layout_selected_index: usize,
    /// Error message to display
    pub error_message: Option<String>,
    /// Whether the wizard is complete
    pub is_complete: bool,
}

impl OnboardingWizardState {
    /// Creates a new onboarding wizard state
    #[must_use] pub fn new() -> Self {
        Self {
            current_step: WizardStep::Welcome,
            inputs: HashMap::new(),
            input_buffer: String::new(),
            available_keyboards: Vec::new(),
            keyboard_filter: String::new(),
            keyboard_selected_index: 0,
            available_layouts: Vec::new(),
            layout_selected_index: 0,
            error_message: None,
            is_complete: false,
        }
    }

    /// Gets the filtered list of keyboards based on current filter
    fn get_filtered_keyboards(&self) -> Vec<String> {
        if self.keyboard_filter.is_empty() {
            self.available_keyboards.clone()
        } else {
            let filter_lower = self.keyboard_filter.to_lowercase();
            self.available_keyboards
                .iter()
                .filter(|kb| kb.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect()
        }
    }

    /// Advances to the next step
    pub fn next_step(&mut self) -> Result<()> {
        self.error_message = None;

        match self.current_step {
            WizardStep::Welcome => {
                self.current_step = WizardStep::QmkPath;
            }
            WizardStep::QmkPath => {
                // Validate and save QMK path
                if self.input_buffer.is_empty() {
                    self.error_message = Some("QMK path cannot be empty".to_string());
                    return Ok(());
                }

                let qmk_path = PathBuf::from(&self.input_buffer);
                if !qmk_path.exists() {
                    self.error_message =
                        Some(format!("Path does not exist: {}", qmk_path.display()));
                    return Ok(());
                }

                // Validate it's a QMK firmware directory
                if !qmk_path.join("Makefile").exists() {
                    self.error_message =
                        Some("Not a QMK firmware directory: Makefile not found".to_string());
                    return Ok(());
                }

                if !qmk_path.join("keyboards").is_dir() {
                    self.error_message =
                        Some("Not a QMK firmware directory: keyboards/ not found".to_string());
                    return Ok(());
                }

                self.inputs
                    .insert("qmk_path".to_string(), self.input_buffer.clone());
                self.input_buffer.clear();

                // Scan keyboards
                match scan_keyboards(&qmk_path) {
                    Ok(keyboards) => {
                        self.available_keyboards = keyboards;
                        self.keyboard_selected_index = 0;
                        self.current_step = WizardStep::KeyboardSelection;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to scan keyboards: {e}"));
                    }
                }
            }
            WizardStep::KeyboardSelection => {
                // Save selected keyboard
                let filtered_keyboards = self.get_filtered_keyboards();
                if filtered_keyboards.is_empty() {
                    self.error_message = Some("No keyboards match filter".to_string());
                    return Ok(());
                }

                let keyboard = filtered_keyboards[self.keyboard_selected_index].clone();
                self.inputs.insert("keyboard".to_string(), keyboard.clone());

                // Clear the filter for next time
                self.keyboard_filter.clear();

                // Parse keyboard info.json to get layouts
                let qmk_path = PathBuf::from(self.inputs.get("qmk_path").unwrap());
                match parse_keyboard_info_json(&qmk_path, &keyboard) {
                    Ok(info) => {
                        self.available_layouts = extract_layout_names(&info);
                        self.layout_selected_index = 0;
                        self.current_step = WizardStep::LayoutSelection;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to parse keyboard info: {e}"));
                    }
                }
            }
            WizardStep::LayoutSelection => {
                // Save selected layout
                if self.available_layouts.is_empty() {
                    self.error_message = Some("No layouts available".to_string());
                    return Ok(());
                }

                let layout = self.available_layouts[self.layout_selected_index].clone();
                self.inputs.insert("layout".to_string(), layout);
                self.current_step = WizardStep::Confirmation;
            }
            WizardStep::Confirmation => {
                // Save configuration and complete
                self.is_complete = true;
            }
        }

        Ok(())
    }

    /// Goes back to the previous step
    pub fn previous_step(&mut self) {
        self.error_message = None;

        if let Some(prev_step) = self.current_step.previous() {
            self.current_step = prev_step;

            // Restore input buffer if going back to QmkPath
            if self.current_step == WizardStep::QmkPath {
                if let Some(qmk_path) = self.inputs.get("qmk_path") {
                    self.input_buffer = qmk_path.clone();
                }
            }

            // Clear keyboard filter when returning to keyboard selection
            if self.current_step == WizardStep::KeyboardSelection {
                self.keyboard_filter.clear();
                self.keyboard_selected_index = 0;
            }
        }
    }

    /// Builds a Config from the collected inputs
    pub fn build_config(&self) -> Result<Config> {
        let mut config = Config::new();

        if let Some(qmk_path) = self.inputs.get("qmk_path") {
            config
                .set_qmk_firmware_path(PathBuf::from(qmk_path))
                .context("Failed to set QMK path")?;
        }

        if let Some(keyboard) = self.inputs.get("keyboard") {
            config.set_keyboard(keyboard.clone());
        }

        if let Some(layout) = self.inputs.get("layout") {
            config.set_layout(layout.clone());
        }

        Ok(config)
    }
}

impl Default for OnboardingWizardState {
    fn default() -> Self {
        Self::new()
    }
}

/// Renders the onboarding wizard
pub fn render(f: &mut Frame, state: &OnboardingWizardState) {
    let size = f.size();

    // Create centered layout
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Instructions
            Constraint::Length(2), // Error message
        ])
        .split(size);

    // Render title
    let title = Paragraph::new(state.current_step.title())
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, vertical_chunks[0]);

    // Render content based on current step
    match state.current_step {
        WizardStep::Welcome => render_welcome(f, vertical_chunks[1]),
        WizardStep::QmkPath => render_qmk_path_input(f, state, vertical_chunks[1]),
        WizardStep::KeyboardSelection => render_keyboard_selection(f, state, vertical_chunks[1]),
        WizardStep::LayoutSelection => render_layout_selection(f, state, vertical_chunks[1]),
        WizardStep::Confirmation => render_confirmation(f, state, vertical_chunks[1]),
    }

    // Render instructions
    render_instructions(f, state, vertical_chunks[2]);

    // Render error message if any
    if let Some(error) = &state.error_message {
        let error_widget = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_widget, vertical_chunks[3]);
    }
}

fn render_welcome(f: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from("Welcome to Keyboard TUI!"),
        Line::from(""),
        Line::from("This wizard will help you set up your configuration."),
        Line::from(""),
        Line::from("You will need:"),
        Line::from("  • Path to your QMK firmware directory"),
        Line::from("  • Knowledge of which keyboard you want to configure"),
        Line::from(""),
        Line::from("Press Enter to continue..."),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

fn render_qmk_path_input(f: &mut Frame, state: &OnboardingWizardState, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from("Enter the path to your QMK firmware directory:"),
        Line::from(""),
        Line::from(Span::styled(
            format!("> {}_", state.input_buffer),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from("Example: /home/user/qmk_firmware"),
        Line::from("         C:\\Users\\user\\qmk_firmware"),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title("QMK Path"));
    f.render_widget(paragraph, area);
}

fn render_keyboard_selection(f: &mut Frame, state: &OnboardingWizardState, area: Rect) {
    // Split area into filter input and list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter input
            Constraint::Min(5),    // Keyboard list
        ])
        .split(area);

    // Render filter input
    let filter_input = Paragraph::new(format!("Filter: {}_", state.keyboard_filter))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(filter_input, chunks[0]);

    // Get filtered keyboards
    let filtered_keyboards = state.get_filtered_keyboards();
    
    let keyboards: Vec<ListItem> = filtered_keyboards
        .iter()
        .enumerate()
        .map(|(i, kb)| {
            let style = if i == state.keyboard_selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(kb.as_str()).style(style)
        })
        .collect();

    let list = List::new(keyboards)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Available Keyboards ({} of {} total)",
            filtered_keyboards.len(),
            state.available_keyboards.len()
        )))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, chunks[1]);
}

fn render_layout_selection(f: &mut Frame, state: &OnboardingWizardState, area: Rect) {
    let layouts: Vec<ListItem> = state
        .available_layouts
        .iter()
        .enumerate()
        .map(|(i, layout)| {
            let style = if i == state.layout_selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(layout.as_str()).style(style)
        })
        .collect();

    let keyboard = state.inputs.get("keyboard").unwrap();
    let list = List::new(layouts)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Layouts for {} ({} available)",
            keyboard,
            state.available_layouts.len()
        )))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

fn render_confirmation(f: &mut Frame, state: &OnboardingWizardState, area: Rect) {
    let default_value = "<not set>".to_string();
    let qmk_path = state.inputs.get("qmk_path").unwrap_or(&default_value);
    let keyboard = state.inputs.get("keyboard").unwrap_or(&default_value);
    let layout = state.inputs.get("layout").unwrap_or(&default_value);

    let text = vec![
        Line::from(""),
        Line::from("Please confirm your configuration:"),
        Line::from(""),
        Line::from(vec![
            Span::styled("QMK Path:  ", Style::default().fg(Color::Cyan)),
            Span::raw(qmk_path),
        ]),
        Line::from(vec![
            Span::styled("Keyboard:  ", Style::default().fg(Color::Cyan)),
            Span::raw(keyboard),
        ]),
        Line::from(vec![
            Span::styled("Layout:    ", Style::default().fg(Color::Cyan)),
            Span::raw(layout),
        ]),
        Line::from(""),
        Line::from("Press Enter to save configuration, or Backspace to go back."),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title("Confirmation"));
    f.render_widget(paragraph, area);
}

fn render_instructions(f: &mut Frame, state: &OnboardingWizardState, area: Rect) {
    let instructions = match state.current_step {
        WizardStep::Welcome => "Enter: Continue  |  Esc: Exit",
        WizardStep::QmkPath => "Enter: Continue  |  Backspace: Delete  |  Esc: Back",
        WizardStep::KeyboardSelection => "Type to filter  |  ↑↓: Navigate  |  Enter: Select  |  Esc: Clear filter/Back",
        WizardStep::LayoutSelection => "↑↓: Navigate  |  Enter: Select  |  Esc: Back",
        WizardStep::Confirmation => "Enter: Save & Exit  |  Esc: Back",
    };

    let paragraph = Paragraph::new(instructions)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

/// Handles keyboard input for the onboarding wizard
pub fn handle_input(state: &mut OnboardingWizardState, key: KeyEvent) -> Result<bool> {
    match state.current_step {
        WizardStep::Welcome => match key.code {
            KeyCode::Enter => {
                state.next_step()?;
            }
            KeyCode::Esc => return Ok(true), // Exit
            _ => {}
        },
        WizardStep::QmkPath => match key.code {
            KeyCode::Enter => {
                state.next_step()?;
            }
            KeyCode::Backspace => {
                state.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                state.input_buffer.push(c);
            }
            KeyCode::Esc => {
                state.previous_step();
            }
            _ => {}
        },
        WizardStep::KeyboardSelection => match key.code {
            KeyCode::Up => {
                if state.keyboard_selected_index > 0 {
                    state.keyboard_selected_index -= 1;
                }
            }
            KeyCode::Down => {
                let filtered_count = state.get_filtered_keyboards().len();
                if state.keyboard_selected_index < filtered_count.saturating_sub(1) {
                    state.keyboard_selected_index += 1;
                }
            }
            KeyCode::Enter => {
                state.next_step()?;
            }
            KeyCode::Char(c) => {
                // Add character to filter
                state.keyboard_filter.push(c);
                // Reset selection to first item when filter changes
                state.keyboard_selected_index = 0;
            }
            KeyCode::Backspace => {
                // Remove character from filter
                state.keyboard_filter.pop();
                // Reset selection to first item when filter changes
                state.keyboard_selected_index = 0;
            }
            KeyCode::Esc => {
                // If filter is active, clear it; otherwise go back
                if !state.keyboard_filter.is_empty() {
                    state.keyboard_filter.clear();
                    state.keyboard_selected_index = 0;
                } else {
                    state.previous_step();
                }
            }
            _ => {}
        },
        WizardStep::LayoutSelection => match key.code {
            KeyCode::Up => {
                if state.layout_selected_index > 0 {
                    state.layout_selected_index -= 1;
                }
            }
            KeyCode::Down => {
                if state.layout_selected_index < state.available_layouts.len().saturating_sub(1) {
                    state.layout_selected_index += 1;
                }
            }
            KeyCode::Enter => {
                state.next_step()?;
            }
            KeyCode::Esc => {
                state.previous_step();
            }
            _ => {}
        },
        WizardStep::Confirmation => match key.code {
            KeyCode::Enter => {
                state.next_step()?;
                return Ok(true); // Complete and exit
            }
            KeyCode::Esc => {
                state.previous_step();
            }
            _ => {}
        },
    }

    Ok(false)
}
