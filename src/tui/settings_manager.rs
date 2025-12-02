//! Settings manager for global layout settings.
//!
//! Provides a UI for configuring layout-wide settings like inactive key behavior.
//! Accessible via Shift+S shortcut.

use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::models::InactiveKeyBehavior;

use super::Theme;

/// Available settings that can be configured
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingItem {
    /// Behavior for keys without color on current layer
    InactiveKeyBehavior,
}

impl SettingItem {
    /// Returns all available settings.
    #[must_use]
    pub const fn all() -> &'static [SettingItem] {
        &[SettingItem::InactiveKeyBehavior]
    }

    /// Returns a human-readable name for this setting.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::InactiveKeyBehavior => "Inactive Key Behavior",
        }
    }

    /// Returns a description of this setting.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::InactiveKeyBehavior => "How to display keys without a color on the current layer",
        }
    }
}

/// Manager mode - determines what operation is being performed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerMode {
    /// Browsing settings (default mode)
    Browsing,
    /// Selecting a value for inactive key behavior
    SelectingInactiveKeyBehavior {
        /// Currently highlighted option
        selected_option: usize,
    },
}

/// State for the settings manager dialog
#[derive(Debug, Clone)]
pub struct SettingsManagerState {
    /// Currently selected setting index
    pub selected: usize,
    /// Current operation mode
    pub mode: ManagerMode,
}

impl SettingsManagerState {
    /// Create a new settings manager state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected: 0,
            mode: ManagerMode::Browsing,
        }
    }

    /// Reset to default state
    pub fn reset(&mut self) {
        self.selected = 0;
        self.mode = ManagerMode::Browsing;
    }

    /// Move selection up
    pub fn select_previous(&mut self, setting_count: usize) {
        if setting_count > 0 {
            if self.selected > 0 {
                self.selected -= 1;
            } else {
                self.selected = setting_count - 1;
            }
        }
    }

    /// Move selection down
    pub fn select_next(&mut self, setting_count: usize) {
        if setting_count > 0 {
            self.selected = (self.selected + 1) % setting_count;
        }
    }

    /// Start selecting inactive key behavior
    pub fn start_selecting_inactive_behavior(&mut self, current: InactiveKeyBehavior) {
        // Find the index of the current value
        let selected_option = InactiveKeyBehavior::all()
            .iter()
            .position(|&b| b == current)
            .unwrap_or(0);
        
        self.mode = ManagerMode::SelectingInactiveKeyBehavior { selected_option };
    }

    /// Move option selection up
    pub fn option_previous(&mut self, option_count: usize) {
        if let ManagerMode::SelectingInactiveKeyBehavior { selected_option } = &mut self.mode {
            if *selected_option > 0 {
                *selected_option -= 1;
            } else {
                *selected_option = option_count - 1;
            }
        }
    }

    /// Move option selection down
    pub fn option_next(&mut self, option_count: usize) {
        if let ManagerMode::SelectingInactiveKeyBehavior { selected_option } = &mut self.mode {
            *selected_option = (*selected_option + 1) % option_count;
        }
    }

    /// Get the currently selected option index
    #[must_use]
    pub fn get_selected_option(&self) -> Option<usize> {
        match &self.mode {
            ManagerMode::SelectingInactiveKeyBehavior { selected_option } => Some(*selected_option),
            _ => None,
        }
    }

    /// Cancel current operation and return to browsing
    pub fn cancel(&mut self) {
        self.mode = ManagerMode::Browsing;
    }

    /// Check if we're in browsing mode
    #[must_use]
    pub const fn is_browsing(&self) -> bool {
        matches!(self.mode, ManagerMode::Browsing)
    }
}

impl Default for SettingsManagerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the settings manager dialog
pub fn render_settings_manager(
    f: &mut Frame,
    area: Rect,
    state: &SettingsManagerState,
    inactive_key_behavior: InactiveKeyBehavior,
    theme: &Theme,
) {
    // Center the dialog (80% width, 80% height)
    let dialog_width = (area.width * 80) / 100;
    let dialog_height = (area.height * 80) / 100;
    let dialog_x = (area.width - dialog_width) / 2;
    let dialog_y = (area.height - dialog_height) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the background area first
    f.render_widget(Clear, dialog_area);

    // Background block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Settings (Shift+S) ")
        .style(Style::default().bg(theme.background));

    f.render_widget(block, dialog_area);

    // Inner area for content
    let inner_area = Rect {
        x: dialog_area.x + 2,
        y: dialog_area.y + 1,
        width: dialog_area.width.saturating_sub(4),
        height: dialog_area.height.saturating_sub(2),
    };

    match &state.mode {
        ManagerMode::Browsing => {
            render_settings_list(f, inner_area, state, inactive_key_behavior, theme);
        }
        ManagerMode::SelectingInactiveKeyBehavior { selected_option } => {
            render_option_selector(
                f,
                inner_area,
                "Inactive Key Behavior",
                InactiveKeyBehavior::all(),
                *selected_option,
                theme,
            );
        }
    }
}

/// Render the list of settings
fn render_settings_list(
    f: &mut Frame,
    area: Rect,
    state: &SettingsManagerState,
    inactive_key_behavior: InactiveKeyBehavior,
    theme: &Theme,
) {
    // Split area for list and help text
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Settings list
            Constraint::Length(5), // Help text
        ])
        .split(area);

    // Render settings list
    let settings = SettingItem::all();
    let items: Vec<ListItem> = settings
        .iter()
        .enumerate()
        .map(|(i, setting)| {
            let style = if i == state.selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            // Get current value for this setting
            let value = match setting {
                SettingItem::InactiveKeyBehavior => inactive_key_behavior.display_name(),
            };

            let content = Line::from(vec![
                Span::styled(setting.display_name(), style),
                Span::styled(": ", Style::default().fg(theme.text_muted)),
                Span::styled(value, Style::default().fg(theme.success)),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Settings"))
        .highlight_style(Style::default().bg(theme.surface));

    f.render_widget(list, chunks[0]);

    // Show description of selected setting
    let selected_setting = settings.get(state.selected);
    let description = selected_setting
        .map(SettingItem::description)
        .unwrap_or("");

    // Render help text
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(description, Style::default().fg(theme.text_muted)),
        ]),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(theme.primary)),
            Span::raw(": Navigate  "),
            Span::styled("Enter", Style::default().fg(theme.primary)),
            Span::raw(": Change  "),
            Span::styled("Esc", Style::default().fg(theme.primary)),
            Span::raw(": Close"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Left);

    f.render_widget(help, chunks[1]);
}

/// Render option selector for a setting
fn render_option_selector(
    f: &mut Frame,
    area: Rect,
    title: &str,
    options: &[InactiveKeyBehavior],
    selected: usize,
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(5),    // Options list
            Constraint::Length(4), // Help
        ])
        .split(area);

    // Title
    let title_text = Paragraph::new(title)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title_text, chunks[0]);

    // Options list
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, option)| {
            let style = if i == selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let marker = if i == selected { "▶ " } else { "  " };

            let content = Line::from(vec![
                Span::styled(marker, Style::default().fg(theme.primary)),
                Span::styled(option.display_name(), style),
                Span::styled(" - ", Style::default().fg(theme.text_muted)),
                Span::styled(option.description(), Style::default().fg(theme.text_muted)),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Options"))
        .highlight_style(Style::default().bg(theme.surface));

    f.render_widget(list, chunks[1]);

    // Help text
    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(theme.primary)),
            Span::raw(": Select  "),
            Span::styled("Enter", Style::default().fg(theme.primary)),
            Span::raw(": Apply  "),
            Span::styled("Esc", Style::default().fg(theme.primary)),
            Span::raw(": Cancel"),
        ]),
    ];

    let help_widget = Paragraph::new(help)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_muted));

    f.render_widget(help_widget, chunks[2]);
}
