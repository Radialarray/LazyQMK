//! Onboarding wizard for first-time setup and configuration.
//!
//! This module implements a step-by-step wizard to guide users through
//! initial configuration: QMK path, keyboard selection, layout variant,
//! output paths, and layout file settings.

// Allow small types passed by reference for API consistency
#![allow(clippy::trivially_copy_pass_by_ref)]

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
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
    /// Enter layout file name
    LayoutName,
    /// Enter firmware output path
    OutputPath,
    /// Confirmation and save
    Confirmation,
}

impl WizardStep {
    /// Gets the next step in the wizard
    #[must_use]
    pub const fn next(&self) -> Option<Self> {
        match self {
            Self::Welcome => Some(Self::QmkPath),
            Self::QmkPath => Some(Self::KeyboardSelection),
            Self::KeyboardSelection => Some(Self::LayoutSelection),
            Self::LayoutSelection => Some(Self::LayoutName),
            Self::LayoutName => Some(Self::OutputPath),
            Self::OutputPath => Some(Self::Confirmation),
            Self::Confirmation => None,
        }
    }

    /// Gets the previous step in the wizard
    #[must_use]
    pub const fn previous(&self) -> Option<Self> {
        match self {
            Self::Welcome => None,
            Self::QmkPath => Some(Self::Welcome),
            Self::KeyboardSelection => Some(Self::QmkPath),
            Self::LayoutSelection => Some(Self::KeyboardSelection),
            Self::LayoutName => Some(Self::LayoutSelection),
            Self::OutputPath => Some(Self::LayoutName),
            Self::Confirmation => Some(Self::OutputPath),
        }
    }

    /// Gets the step title
    #[must_use]
    pub const fn title(&self) -> &'static str {
        match self {
            Self::Welcome => "Welcome to Keyboard TUI",
            Self::QmkPath => "QMK Firmware Path",
            Self::KeyboardSelection => "Select Keyboard",
            Self::LayoutSelection => "Select Layout",
            Self::LayoutName => "Layout File Name",
            Self::OutputPath => "Firmware Output Path",
            Self::Confirmation => "Confirm Configuration",
        }
    }

    /// Gets the step number (1-based)
    #[must_use]
    pub const fn step_number(&self) -> usize {
        match self {
            Self::Welcome => 1,
            Self::QmkPath => 2,
            Self::KeyboardSelection => 3,
            Self::LayoutSelection => 4,
            Self::LayoutName => 5,
            Self::OutputPath => 6,
            Self::Confirmation => 7,
        }
    }

    /// Gets the total number of steps
    #[must_use]
    pub const fn total_steps() -> usize {
        7
    }
}

/// Focus state for keyboard selection step
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardSelectionFocus {
    /// Focus is on the filter input field
    FilterInput,
    /// Focus is on the keyboard list
    List,
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
    /// Focus state for keyboard selection (filter input vs list)
    pub keyboard_selection_focus: KeyboardSelectionFocus,
    /// List of available layouts (populated after keyboard is selected)
    pub available_layouts: Vec<String>,
    /// Selected layout index in list
    pub layout_selected_index: usize,
    /// Error message to display
    pub error_message: Option<String>,
    /// Whether the wizard is complete
    pub is_complete: bool,
    /// Whether this is a keyboard-only change (skip other config steps)
    pub keyboard_change_only: bool,
}

impl OnboardingWizardState {
    /// Creates a new onboarding wizard state
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_step: WizardStep::Welcome,
            inputs: HashMap::new(),
            input_buffer: String::new(),
            available_keyboards: Vec::new(),
            keyboard_filter: String::new(),
            keyboard_selected_index: 0,
            keyboard_selection_focus: KeyboardSelectionFocus::FilterInput,
            available_layouts: Vec::new(),
            layout_selected_index: 0,
            error_message: None,
            is_complete: false,
            keyboard_change_only: false,
        }
    }

    /// Creates a wizard state starting at keyboard selection step.
    /// 
    /// This is used when changing keyboard from settings - skips QMK path setup
    /// and completes after layout selection (skips output path, format, etc.).
    /// 
    /// # Arguments
    /// * `qmk_path` - The already-configured QMK firmware path
    /// 
    /// # Returns
    /// * `Ok(Self)` - Wizard state ready for keyboard selection
    /// * `Err` - If keyboard scanning fails
    pub fn new_for_keyboard_selection(qmk_path: &std::path::Path) -> Result<Self> {
        let keyboards = scan_keyboards(qmk_path)?;
        
        let mut inputs = HashMap::new();
        inputs.insert("qmk_path".to_string(), qmk_path.to_string_lossy().to_string());
        
        Ok(Self {
            current_step: WizardStep::KeyboardSelection,
            inputs,
            input_buffer: String::new(),
            available_keyboards: keyboards,
            keyboard_filter: String::new(),
            keyboard_selected_index: 0,
            keyboard_selection_focus: KeyboardSelectionFocus::FilterInput,
            available_layouts: Vec::new(),
            layout_selected_index: 0,
            error_message: None,
            is_complete: false,
            keyboard_change_only: true,
        })
    }

    /// Creates a wizard state for creating a new layout.
    /// 
    /// This starts at keyboard selection but goes through all steps
    /// (keyboard, layout, layout name, output path, confirmation).
    /// Used when "Create New Layout" is selected from the layout picker.
    /// 
    /// # Arguments
    /// * `config` - The existing configuration with QMK path already set
    /// 
    /// # Returns
    /// * `Ok(Self)` - Wizard state ready for keyboard selection
    /// * `Err` - If keyboard scanning fails
    pub fn new_for_new_layout(config: &Config) -> Result<Self> {
        let qmk_path = config.paths.qmk_firmware.as_ref()
            .ok_or_else(|| anyhow::anyhow!("QMK firmware path not configured"))?;
        
        let keyboards = scan_keyboards(qmk_path)?;
        
        let mut inputs = HashMap::new();
        inputs.insert("qmk_path".to_string(), qmk_path.to_string_lossy().to_string());
        // Pre-populate output path from existing config
        inputs.insert("output_path".to_string(), config.build.output_dir.display().to_string());
        
        Ok(Self {
            current_step: WizardStep::KeyboardSelection,
            inputs,
            input_buffer: String::new(),
            available_keyboards: keyboards,
            keyboard_filter: String::new(),
            keyboard_selected_index: 0,
            keyboard_selection_focus: KeyboardSelectionFocus::FilterInput,
            available_layouts: Vec::new(),
            layout_selected_index: 0,
            error_message: None,
            is_complete: false,
            keyboard_change_only: false, // Go through all steps
        })
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
    #[allow(clippy::too_many_lines)]
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
                
                // If this is a keyboard-only change, we're done
                if self.keyboard_change_only {
                    self.is_complete = true;
                    return Ok(());
                }
                
                // Pre-populate layout name from keyboard if not already set
                if !self.inputs.contains_key("layout_name") {
                    let keyboard = self.inputs.get("keyboard").unwrap();
                    let default_name = keyboard.split('/').next_back().unwrap_or(keyboard);
                    self.input_buffer = format!("{default_name}_layout");
                }
                
                self.current_step = WizardStep::LayoutName;
            }
            WizardStep::LayoutName => {
                // Save layout name
                if self.input_buffer.is_empty() {
                    self.error_message = Some("Layout name cannot be empty".to_string());
                    return Ok(());
                }
                
                self.inputs.insert("layout_name".to_string(), self.input_buffer.clone());
                self.input_buffer.clear();
                
                // Pre-populate output path with default if not already set
                if !self.inputs.contains_key("output_path") {
                    if let Ok(default_dir) = Config::config_dir() {
                        self.input_buffer = default_dir.join("builds").display().to_string();
                    }
                }
                
                self.current_step = WizardStep::OutputPath;
            }
            WizardStep::OutputPath => {
                // Save output path
                if self.input_buffer.is_empty() {
                    self.error_message = Some("Output path cannot be empty".to_string());
                    return Ok(());
                }
                
                let output_path = PathBuf::from(&self.input_buffer);
                
                // Validate parent directory exists or can be created
                if let Some(parent) = output_path.parent() {
                    if !parent.exists() {
                        self.error_message = Some(format!(
                            "Parent directory does not exist: {}",
                            parent.display()
                        ));
                        return Ok(());
                    }
                }
                
                self.inputs.insert("output_path".to_string(), self.input_buffer.clone());
                self.input_buffer.clear();
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

            // Restore layout name when going back
            if self.current_step == WizardStep::LayoutName {
                if let Some(layout_name) = self.inputs.get("layout_name") {
                    self.input_buffer = layout_name.clone();
                }
            }

            // Restore output path when going back
            if self.current_step == WizardStep::OutputPath {
                if let Some(output_path) = self.inputs.get("output_path") {
                    self.input_buffer = output_path.clone();
                }
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

        // Note: keyboard and layout are now stored per-layout in metadata, not in config
        // The wizard collects these for creating a new layout template

        if let Some(output_path) = self.inputs.get("output_path") {
            config.build.output_dir = PathBuf::from(output_path);
        }

        Ok(config)
    }

    /// Pre-populates the wizard with values from an existing config.
    ///
    /// This is used when opening the wizard from the TUI with Ctrl+W
    /// to allow editing the current configuration.
    #[must_use] pub fn from_config(config: &Config) -> Self {
        let mut wizard = Self::new();

        // Pre-populate QMK path
        if let Some(qmk_path) = &config.paths.qmk_firmware {
            wizard.inputs.insert("qmk_path".to_string(), qmk_path.display().to_string());
            wizard.input_buffer = qmk_path.display().to_string();
        }

        // Note: keyboard and layout are now per-layout in metadata, not in config
        // The wizard is for initial setup only

        // Pre-populate output path
        wizard.inputs.insert("output_path".to_string(), config.build.output_dir.display().to_string());

        wizard
    }
}

impl Default for OnboardingWizardState {
    fn default() -> Self {
        Self::new()
    }
}

/// Renders the onboarding wizard
pub fn render(f: &mut Frame, state: &OnboardingWizardState, theme: &crate::tui::theme::Theme) {
    let size = f.size();

    // Clear the background area first
    f.render_widget(Clear, size);

    // Render opaque background for entire wizard
    let background = Block::default()
        .style(Style::default().bg(theme.background));
    f.render_widget(background, size);

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
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).style(Style::default().bg(theme.background)));
    f.render_widget(title, vertical_chunks[0]);

    // Render content based on current step
    match state.current_step {
        WizardStep::Welcome => render_welcome(f, vertical_chunks[1], theme),
        WizardStep::QmkPath => render_qmk_path_input(f, state, vertical_chunks[1], theme),
        WizardStep::KeyboardSelection => render_keyboard_selection(f, state, vertical_chunks[1], theme),
        WizardStep::LayoutSelection => render_layout_selection(f, state, vertical_chunks[1], theme),
        WizardStep::LayoutName => render_layout_name_input(f, state, vertical_chunks[1], theme),
        WizardStep::OutputPath => render_output_path_input(f, state, vertical_chunks[1], theme),
        WizardStep::Confirmation => render_confirmation(f, state, vertical_chunks[1], theme),
    }

    // Render instructions
    render_instructions(f, state, vertical_chunks[2], theme);

    // Render error message if any
    if let Some(error) = &state.error_message {
        let error_widget = Paragraph::new(error.as_str())
            .style(Style::default().fg(theme.error))
            .alignment(Alignment::Center);
        f.render_widget(error_widget, vertical_chunks[3]);
    }
}

fn render_welcome(f: &mut Frame, area: Rect, theme: &crate::tui::theme::Theme) {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Welcome to Keyboard TUI!",
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "This wizard will help you set up your configuration.",
            Style::default().fg(theme.text),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "You will need:",
            Style::default().fg(theme.text),
        )),
        Line::from(Span::styled(
            "  - Path to your QMK firmware directory",
            Style::default().fg(theme.text),
        )),
        Line::from(Span::styled(
            "  - Knowledge of which keyboard you want to configure",
            Style::default().fg(theme.text),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to continue...",
            Style::default().fg(theme.text_muted),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text))
        .block(Block::default().borders(Borders::ALL).style(Style::default().fg(theme.primary).bg(theme.background)));
    f.render_widget(paragraph, area);
}

fn render_qmk_path_input(f: &mut Frame, state: &OnboardingWizardState, area: Rect, theme: &crate::tui::theme::Theme) {
    let text = vec![
        Line::from(""),
        Line::from("Enter the path to your QMK firmware directory:"),
        Line::from(""),
        Line::from(Span::styled(
            format!("> {}_", state.input_buffer),
            Style::default().fg(theme.accent),
        )),
        Line::from(""),
        Line::from("Example: /home/user/qmk_firmware"),
        Line::from("         C:\\Users\\user\\qmk_firmware"),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Left)
        .style(Style::default().fg(theme.text))
        .block(Block::default().borders(Borders::ALL).title("QMK Path").style(Style::default().fg(theme.primary).bg(theme.background)));
    f.render_widget(paragraph, area);
}

fn render_keyboard_selection(f: &mut Frame, state: &OnboardingWizardState, area: Rect, theme: &crate::tui::theme::Theme) {
    // Split area into filter input and list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter input
            Constraint::Min(5),    // Keyboard list
            Constraint::Length(2), // Help text
        ])
        .split(area);

    // Render filter input with focus indicator
    let filter_focused = state.keyboard_selection_focus == KeyboardSelectionFocus::FilterInput;
    let filter_title = if filter_focused {
        "Search [FOCUSED - Type to filter]"
    } else {
        "Search"
    };
    let filter_border_style = if filter_focused {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.primary)
    };
    
    let filter_input = Paragraph::new(format!("Filter: {}_", state.keyboard_filter))
        .style(Style::default().fg(if filter_focused { theme.accent } else { theme.text }))
        .block(Block::default().borders(Borders::ALL).title(filter_title).style(filter_border_style.bg(theme.background)));
    f.render_widget(filter_input, chunks[0]);

    // Get filtered keyboards
    let filtered_keyboards = state.get_filtered_keyboards();

    let keyboards: Vec<ListItem> = filtered_keyboards
        .iter()
        .enumerate()
        .map(|(i, kb)| {
            let style = if i == state.keyboard_selected_index {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            ListItem::new(kb.as_str()).style(style)
        })
        .collect();

    // List with focus indicator
    let list_focused = state.keyboard_selection_focus == KeyboardSelectionFocus::List;
    let list_title = if list_focused {
        format!(
            "Keyboards [FOCUSED] ({} of {} total)",
            filtered_keyboards.len(),
            state.available_keyboards.len()
        )
    } else {
        format!(
            "Available Keyboards ({} of {} total)",
            filtered_keyboards.len(),
            state.available_keyboards.len()
        )
    };
    let list_border_style = if list_focused {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.primary)
    };

    let list = List::new(keyboards)
        .block(Block::default().borders(Borders::ALL).title(list_title).style(list_border_style.bg(theme.background)))
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, chunks[1]);

    // Help text
    let help_text = if filter_focused {
        "Tab: Switch to list | Enter: Select (if 1 result) | Esc: Clear filter or back"
    } else {
        "↑↓/jk: Navigate | Enter: Select | Tab: Back to filter | Esc: Back to filter"
    };
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

fn render_layout_selection(f: &mut Frame, state: &OnboardingWizardState, area: Rect, theme: &crate::tui::theme::Theme) {
    let layouts: Vec<ListItem> = state
        .available_layouts
        .iter()
        .enumerate()
        .map(|(i, layout)| {
            let style = if i == state.layout_selected_index {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
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
        )).style(Style::default().fg(theme.primary).bg(theme.background)))
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

fn render_layout_name_input(f: &mut Frame, state: &OnboardingWizardState, area: Rect, theme: &crate::tui::theme::Theme) {
    let text = vec![
        Line::from(""),
        Line::from("Enter a name for your layout file:"),
        Line::from(""),
        Line::from(Span::styled(
            format!("> {}_", state.input_buffer),
            Style::default().fg(theme.accent),
        )),
        Line::from(""),
        Line::from("This will be used as the filename (e.g., my_layout.md)"),
        Line::from("The file will be saved in your layouts directory."),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Left)
        .style(Style::default().fg(theme.text))
        .block(Block::default().borders(Borders::ALL).title("Layout Name").style(Style::default().fg(theme.primary).bg(theme.background)));
    f.render_widget(paragraph, area);
}

fn render_output_path_input(f: &mut Frame, state: &OnboardingWizardState, area: Rect, theme: &crate::tui::theme::Theme) {
    let text = vec![
        Line::from(""),
        Line::from("Enter the path where firmware files should be copied:"),
        Line::from(""),
        Line::from(Span::styled(
            format!("> {}_", state.input_buffer),
            Style::default().fg(theme.accent),
        )),
        Line::from(""),
        Line::from("After building, firmware files (.uf2, .hex) will be copied here."),
        Line::from("This makes it easy to flash your keyboard."),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Left)
        .style(Style::default().fg(theme.text))
        .block(Block::default().borders(Borders::ALL).title("Firmware Output Path").style(Style::default().fg(theme.primary).bg(theme.background)));
    f.render_widget(paragraph, area);
}

fn render_confirmation(f: &mut Frame, state: &OnboardingWizardState, area: Rect, theme: &crate::tui::theme::Theme) {
    let default_value = "<not set>".to_string();
    let qmk_path = state.inputs.get("qmk_path").unwrap_or(&default_value);
    let keyboard = state.inputs.get("keyboard").unwrap_or(&default_value);
    let layout = state.inputs.get("layout").unwrap_or(&default_value);
    let layout_name = state.inputs.get("layout_name").unwrap_or(&default_value);
    let output_path = state.inputs.get("output_path").unwrap_or(&default_value);

    let text = vec![
        Line::from(""),
        Line::from("Please confirm your configuration:"),
        Line::from(""),
        Line::from(vec![
            Span::styled("QMK Path:     ", Style::default().fg(theme.primary)),
            Span::raw(qmk_path),
        ]),
        Line::from(vec![
            Span::styled("Keyboard:     ", Style::default().fg(theme.primary)),
            Span::raw(keyboard),
        ]),
        Line::from(vec![
            Span::styled("Layout:       ", Style::default().fg(theme.primary)),
            Span::raw(layout),
        ]),
        Line::from(vec![
            Span::styled("Layout Name:  ", Style::default().fg(theme.primary)),
            Span::raw(layout_name),
        ]),
        Line::from(vec![
            Span::styled("Output Path:  ", Style::default().fg(theme.primary)),
            Span::raw(output_path),
        ]),
        Line::from(""),
        Line::from("Press Enter to save configuration, or Esc to go back."),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Left)
        .style(Style::default().fg(theme.text))
        .block(Block::default().borders(Borders::ALL).title("Confirmation").style(Style::default().fg(theme.primary).bg(theme.background)));
    f.render_widget(paragraph, area);
}

fn render_instructions(f: &mut Frame, state: &OnboardingWizardState, area: Rect, theme: &crate::tui::theme::Theme) {
    let step_info = format!(
        "Step {} of {}",
        state.current_step.step_number(),
        WizardStep::total_steps()
    );
    
    let instructions = match state.current_step {
        WizardStep::Welcome => "Enter: Continue  |  Esc: Exit",
        WizardStep::QmkPath 
        | WizardStep::LayoutName 
        | WizardStep::OutputPath => "Enter: Continue  |  Backspace: Delete  |  Esc: Back",
        WizardStep::KeyboardSelection => {
            "Type to filter  |  ↑↓: Navigate  |  Enter: Select  |  Esc: Clear filter/Back"
        }
        WizardStep::LayoutSelection => "↑↓: Navigate  |  Enter: Select  |  Esc: Back",
        WizardStep::Confirmation => "Enter: Save & Exit  |  Esc: Back",
    };

    let text = vec![
        Line::from(step_info),
        Line::from(instructions),
    ];

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).style(Style::default().fg(theme.primary).bg(theme.background)));
    f.render_widget(paragraph, area);
}

/// Handles keyboard input for the onboarding wizard
#[allow(clippy::too_many_lines)]
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
        WizardStep::KeyboardSelection => match state.keyboard_selection_focus {
            KeyboardSelectionFocus::FilterInput => match key.code {
                KeyCode::Tab => {
                    // Switch focus to list
                    state.keyboard_selection_focus = KeyboardSelectionFocus::List;
                }
                KeyCode::Enter => {
                    // Enter from filter: if only one result, select it; otherwise switch to list
                    let filtered = state.get_filtered_keyboards();
                    if filtered.len() == 1 {
                        state.next_step()?;
                    } else if !filtered.is_empty() {
                        state.keyboard_selection_focus = KeyboardSelectionFocus::List;
                    }
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
                    // If filter is active, clear it
                    if !state.keyboard_filter.is_empty() {
                        state.keyboard_filter.clear();
                        state.keyboard_selected_index = 0;
                    } else if state.keyboard_change_only {
                        // In keyboard_change_only mode, Esc exits the wizard
                        return Ok(true);
                    } else {
                        state.previous_step();
                    }
                }
                _ => {}
            },
            KeyboardSelectionFocus::List => match key.code {
                KeyCode::Tab => {
                    // Switch focus back to filter
                    state.keyboard_selection_focus = KeyboardSelectionFocus::FilterInput;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if state.keyboard_selected_index > 0 {
                        state.keyboard_selected_index -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let filtered_count = state.get_filtered_keyboards().len();
                    if state.keyboard_selected_index < filtered_count.saturating_sub(1) {
                        state.keyboard_selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    state.next_step()?;
                }
                KeyCode::Esc => {
                    // Esc from list: switch back to filter
                    state.keyboard_selection_focus = KeyboardSelectionFocus::FilterInput;
                }
                _ => {}
            },
        },
        WizardStep::LayoutSelection => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if state.layout_selected_index > 0 {
                    state.layout_selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if state.layout_selected_index < state.available_layouts.len().saturating_sub(1) {
                    state.layout_selected_index += 1;
                }
            }
            KeyCode::Enter => {
                state.next_step()?;
                // If keyboard_change_only mode completed, signal exit
                if state.is_complete {
                    return Ok(true);
                }
            }
            KeyCode::Esc => {
                // In keyboard_change_only mode, Esc exits the wizard
                if state.keyboard_change_only {
                    return Ok(true);
                }
                state.previous_step();
            }
            _ => {}
        },
        WizardStep::LayoutName => match key.code {
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
        WizardStep::OutputPath => match key.code {
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
