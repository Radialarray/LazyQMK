//! Settings manager for global layout settings.
//!
//! Provides a UI for configuring layout-wide settings like inactive key behavior
//! and tap-hold timing configuration.
//! Accessible via Shift+S shortcut.

use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::models::{HoldDecisionMode, InactiveKeyBehavior, TapHoldPreset, TapHoldSettings};

use super::Theme;

/// Setting group for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingGroup {
    /// General layout settings
    General,
    /// Tap-hold timing and behavior settings
    TapHold,
}

impl SettingGroup {
    /// Returns all groups in display order.
    #[must_use]
    pub const fn all() -> &'static [SettingGroup] {
        &[SettingGroup::General, SettingGroup::TapHold]
    }

    /// Returns display name.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::General => "General",
            Self::TapHold => "Tap-Hold",
        }
    }
}

/// Available settings that can be configured
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingItem {
    // === General Settings ===
    /// Behavior for keys without color on current layer
    InactiveKeyBehavior,

    // === Tap-Hold Settings ===
    /// Preset for common tap-hold configurations
    TapHoldPreset,
    /// Base timing for tap vs hold decision
    TappingTerm,
    /// Quick tap window for auto-repeat
    QuickTapTerm,
    /// How to decide between tap and hold
    HoldMode,
    /// Whether to send tap on release after tapping term
    RetroTapping,
    /// Number of taps to toggle layer with TT()
    TappingToggle,
    /// Flow tap timing for home-row mod optimization
    FlowTapTerm,
    /// Use opposite-hand rule for tap-hold decision
    ChordalHold,
}

impl SettingItem {
    /// Returns all settings in a single flat list.
    #[must_use]
    pub const fn all() -> &'static [SettingItem] {
        &[
            SettingItem::InactiveKeyBehavior,
            SettingItem::TapHoldPreset,
            SettingItem::TappingTerm,
            SettingItem::QuickTapTerm,
            SettingItem::HoldMode,
            SettingItem::RetroTapping,
            SettingItem::TappingToggle,
            SettingItem::FlowTapTerm,
            SettingItem::ChordalHold,
        ]
    }

    /// Returns which group this setting belongs to.
    #[must_use]
    pub const fn group(&self) -> SettingGroup {
        match self {
            Self::InactiveKeyBehavior => SettingGroup::General,
            Self::TapHoldPreset
            | Self::TappingTerm
            | Self::QuickTapTerm
            | Self::HoldMode
            | Self::RetroTapping
            | Self::TappingToggle
            | Self::FlowTapTerm
            | Self::ChordalHold => SettingGroup::TapHold,
        }
    }

    /// Returns a human-readable name for this setting.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::InactiveKeyBehavior => "Inactive Key Behavior",
            Self::TapHoldPreset => "Preset",
            Self::TappingTerm => "Tapping Term",
            Self::QuickTapTerm => "Quick Tap Term",
            Self::HoldMode => "Hold Mode",
            Self::RetroTapping => "Retro Tapping",
            Self::TappingToggle => "Tapping Toggle",
            Self::FlowTapTerm => "Flow Tap Term",
            Self::ChordalHold => "Chordal Hold",
        }
    }

    /// Returns a description of this setting.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::InactiveKeyBehavior => {
                "How to display keys without a color on the current layer"
            }
            Self::TapHoldPreset => "Quick configuration preset for common use cases",
            Self::TappingTerm => "Milliseconds to distinguish tap from hold (100-500ms)",
            Self::QuickTapTerm => "Window for tap-then-hold to trigger auto-repeat",
            Self::HoldMode => "How to decide tap vs hold when other keys are pressed",
            Self::RetroTapping => "Send tap keycode even if held past tapping term",
            Self::TappingToggle => "Number of taps to toggle layer with TT() keys (1-10)",
            Self::FlowTapTerm => "Rapid typing window to prevent accidental modifiers",
            Self::ChordalHold => "Use opposite-hand rule for tap-hold (great for HRM)",
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
        /// Currently highlighted option index
        selected_option: usize,
    },
    /// Selecting a tap-hold preset
    SelectingTapHoldPreset {
        /// Currently highlighted option index
        selected_option: usize,
    },
    /// Selecting a hold mode
    SelectingHoldMode {
        /// Currently highlighted option index
        selected_option: usize,
    },
    /// Editing a numeric value (tapping_term, quick_tap_term, etc.)
    EditingNumeric {
        /// Which setting is being edited
        setting: SettingItem,
        /// Current value as string for editing
        value: String,
        /// Minimum allowed value
        min: u16,
        /// Maximum allowed value
        max: u16,
    },
    /// Toggling a boolean value (retro_tapping, chordal_hold)
    TogglingBoolean {
        /// Which setting is being toggled
        setting: SettingItem,
        /// Current value
        value: bool,
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
        let selected_option = InactiveKeyBehavior::all()
            .iter()
            .position(|&b| b == current)
            .unwrap_or(0);
        self.mode = ManagerMode::SelectingInactiveKeyBehavior { selected_option };
    }

    /// Start selecting tap-hold preset
    pub fn start_selecting_tap_hold_preset(&mut self, current: TapHoldPreset) {
        let selected_option = TapHoldPreset::all()
            .iter()
            .position(|&p| p == current)
            .unwrap_or(0);
        self.mode = ManagerMode::SelectingTapHoldPreset { selected_option };
    }

    /// Start selecting hold mode
    pub fn start_selecting_hold_mode(&mut self, current: HoldDecisionMode) {
        let selected_option = HoldDecisionMode::all()
            .iter()
            .position(|&m| m == current)
            .unwrap_or(0);
        self.mode = ManagerMode::SelectingHoldMode { selected_option };
    }

    /// Start editing a numeric value
    pub fn start_editing_numeric(&mut self, setting: SettingItem, current: u16, min: u16, max: u16) {
        self.mode = ManagerMode::EditingNumeric {
            setting,
            value: current.to_string(),
            min,
            max,
        };
    }

    /// Start toggling a boolean value
    pub fn start_toggling_boolean(&mut self, setting: SettingItem, current: bool) {
        self.mode = ManagerMode::TogglingBoolean {
            setting,
            value: current,
        };
    }

    /// Move option selection up (for enum selectors)
    pub fn option_previous(&mut self, option_count: usize) {
        match &mut self.mode {
            ManagerMode::SelectingInactiveKeyBehavior { selected_option }
            | ManagerMode::SelectingTapHoldPreset { selected_option }
            | ManagerMode::SelectingHoldMode { selected_option } => {
                if *selected_option > 0 {
                    *selected_option -= 1;
                } else {
                    *selected_option = option_count - 1;
                }
            }
            ManagerMode::TogglingBoolean { value, .. } => {
                *value = !*value;
            }
            _ => {}
        }
    }

    /// Move option selection down (for enum selectors)
    pub fn option_next(&mut self, option_count: usize) {
        match &mut self.mode {
            ManagerMode::SelectingInactiveKeyBehavior { selected_option }
            | ManagerMode::SelectingTapHoldPreset { selected_option }
            | ManagerMode::SelectingHoldMode { selected_option } => {
                *selected_option = (*selected_option + 1) % option_count;
            }
            ManagerMode::TogglingBoolean { value, .. } => {
                *value = !*value;
            }
            _ => {}
        }
    }

    /// Get the currently selected option index
    #[must_use]
    pub fn get_selected_option(&self) -> Option<usize> {
        match &self.mode {
            ManagerMode::SelectingInactiveKeyBehavior { selected_option }
            | ManagerMode::SelectingTapHoldPreset { selected_option }
            | ManagerMode::SelectingHoldMode { selected_option } => Some(*selected_option),
            _ => None,
        }
    }

    /// Handle character input for numeric editing
    pub fn handle_char_input(&mut self, c: char) {
        if let ManagerMode::EditingNumeric { value, max, .. } = &mut self.mode {
            if c.is_ascii_digit() {
                value.push(c);
                // Cap at max length to prevent overflow
                if value.len() > 4 {
                    value.pop();
                }
                // Cap at max value
                if let Ok(num) = value.parse::<u16>() {
                    if num > *max {
                        *value = max.to_string();
                    }
                }
            }
        }
    }

    /// Handle backspace for numeric editing
    pub fn handle_backspace(&mut self) {
        if let ManagerMode::EditingNumeric { value, .. } = &mut self.mode {
            value.pop();
        }
    }

    /// Increment numeric value
    pub fn increment_numeric(&mut self, step: u16) {
        if let ManagerMode::EditingNumeric { value, max, .. } = &mut self.mode {
            if let Ok(mut num) = value.parse::<u16>() {
                num = num.saturating_add(step).min(*max);
                *value = num.to_string();
            }
        }
    }

    /// Decrement numeric value
    pub fn decrement_numeric(&mut self, step: u16) {
        if let ManagerMode::EditingNumeric { value, min, .. } = &mut self.mode {
            if let Ok(mut num) = value.parse::<u16>() {
                num = num.saturating_sub(step).max(*min);
                *value = num.to_string();
            }
        }
    }

    /// Get the current numeric value being edited
    #[must_use]
    pub fn get_numeric_value(&self) -> Option<u16> {
        if let ManagerMode::EditingNumeric { value, min, .. } = &self.mode {
            value.parse().ok().or(Some(*min))
        } else {
            None
        }
    }

    /// Get the current boolean value being toggled
    #[must_use]
    pub fn get_boolean_value(&self) -> Option<bool> {
        if let ManagerMode::TogglingBoolean { value, .. } = &self.mode {
            Some(*value)
        } else {
            None
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
    tap_hold_settings: &TapHoldSettings,
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
            render_settings_list(f, inner_area, state, inactive_key_behavior, tap_hold_settings, theme);
        }
        ManagerMode::SelectingInactiveKeyBehavior { selected_option } => {
            render_inactive_behavior_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::SelectingTapHoldPreset { selected_option } => {
            render_tap_hold_preset_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::SelectingHoldMode { selected_option } => {
            render_hold_mode_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::EditingNumeric { setting, value, min, max } => {
            render_numeric_editor(f, inner_area, *setting, value, *min, *max, theme);
        }
        ManagerMode::TogglingBoolean { setting, value } => {
            render_boolean_toggle(f, inner_area, *setting, *value, theme);
        }
    }
}

/// Render the list of settings
fn render_settings_list(
    f: &mut Frame,
    area: Rect,
    state: &SettingsManagerState,
    inactive_key_behavior: InactiveKeyBehavior,
    tap_hold_settings: &TapHoldSettings,
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

    // Build settings list with group headers
    let settings = SettingItem::all();
    let mut items: Vec<ListItem> = Vec::new();
    let mut current_group: Option<SettingGroup> = None;
    let mut display_index = 0;

    for setting in settings {
        // Add group header if group changes
        let group = setting.group();
        if current_group != Some(group) {
            if current_group.is_some() {
                // Add spacing between groups
                items.push(ListItem::new(Line::from("")));
            }
            items.push(ListItem::new(Line::from(vec![Span::styled(
                format!("── {} ──", group.display_name()),
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )])));
            current_group = Some(group);
        }

        let style = if display_index == state.selected {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text)
        };

        // Get current value for this setting
        let value = get_setting_value_display(*setting, inactive_key_behavior, tap_hold_settings);

        let marker = if display_index == state.selected {
            "▶ "
        } else {
            "  "
        };

        let content = Line::from(vec![
            Span::styled(marker, Style::default().fg(theme.primary)),
            Span::styled(setting.display_name(), style),
            Span::styled(": ", Style::default().fg(theme.text_muted)),
            Span::styled(value, Style::default().fg(theme.success)),
        ]);

        items.push(ListItem::new(content));
        display_index += 1;
    }

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
        Line::from(vec![Span::styled(
            description,
            Style::default().fg(theme.text_muted),
        )]),
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

/// Get display string for a setting value
fn get_setting_value_display(
    setting: SettingItem,
    inactive_key_behavior: InactiveKeyBehavior,
    tap_hold: &TapHoldSettings,
) -> String {
    match setting {
        SettingItem::InactiveKeyBehavior => inactive_key_behavior.display_name().to_string(),
        SettingItem::TapHoldPreset => tap_hold.preset.display_name().to_string(),
        SettingItem::TappingTerm => format!("{}ms", tap_hold.tapping_term),
        SettingItem::QuickTapTerm => match tap_hold.quick_tap_term {
            Some(term) => format!("{}ms", term),
            None => "Auto".to_string(),
        },
        SettingItem::HoldMode => tap_hold.hold_mode.display_name().to_string(),
        SettingItem::RetroTapping => {
            if tap_hold.retro_tapping {
                "On"
            } else {
                "Off"
            }
            .to_string()
        }
        SettingItem::TappingToggle => format!("{} taps", tap_hold.tapping_toggle),
        SettingItem::FlowTapTerm => match tap_hold.flow_tap_term {
            Some(term) => format!("{}ms", term),
            None => "Disabled".to_string(),
        },
        SettingItem::ChordalHold => {
            if tap_hold.chordal_hold {
                "On"
            } else {
                "Off"
            }
            .to_string()
        }
    }
}

/// Render inactive behavior selector
fn render_inactive_behavior_selector(
    f: &mut Frame,
    area: Rect,
    selected: usize,
    theme: &Theme,
) {
    let options = InactiveKeyBehavior::all();
    render_enum_selector(
        f,
        area,
        "Inactive Key Behavior",
        options
            .iter()
            .map(|o| (o.display_name(), o.description()))
            .collect::<Vec<_>>()
            .as_slice(),
        selected,
        theme,
    );
}

/// Render tap-hold preset selector
fn render_tap_hold_preset_selector(f: &mut Frame, area: Rect, selected: usize, theme: &Theme) {
    let options = TapHoldPreset::all();
    render_enum_selector(
        f,
        area,
        "Tap-Hold Preset",
        options
            .iter()
            .map(|o| (o.display_name(), o.description()))
            .collect::<Vec<_>>()
            .as_slice(),
        selected,
        theme,
    );
}

/// Render hold mode selector
fn render_hold_mode_selector(f: &mut Frame, area: Rect, selected: usize, theme: &Theme) {
    let options = HoldDecisionMode::all();
    render_enum_selector(
        f,
        area,
        "Hold Mode",
        options
            .iter()
            .map(|o| (o.display_name(), o.description()))
            .collect::<Vec<_>>()
            .as_slice(),
        selected,
        theme,
    );
}

/// Generic enum selector renderer
fn render_enum_selector(
    f: &mut Frame,
    area: Rect,
    title: &str,
    options: &[(&str, &str)],
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
    let title_text = Paragraph::new(title).alignment(Alignment::Center).style(
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(title_text, chunks[0]);

    // Options list
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
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
                Span::styled(*name, style),
                Span::styled(" - ", Style::default().fg(theme.text_muted)),
                Span::styled(*desc, Style::default().fg(theme.text_muted)),
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

/// Render numeric value editor
fn render_numeric_editor(
    f: &mut Frame,
    area: Rect,
    setting: SettingItem,
    value: &str,
    min: u16,
    max: u16,
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(6),    // Value display
            Constraint::Length(5), // Help
        ])
        .split(area);

    // Title
    let title_text = Paragraph::new(setting.display_name())
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title_text, chunks[0]);

    // Value display
    let value_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Current value: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("{value}ms"),
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("Range: {min}ms - {max}ms"),
            Style::default().fg(theme.text_muted),
        )]),
    ];

    let value_paragraph = Paragraph::new(value_text)
        .block(Block::default().borders(Borders::ALL).title("Value"))
        .alignment(Alignment::Center);

    f.render_widget(value_paragraph, chunks[1]);

    // Help text
    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("0-9", Style::default().fg(theme.primary)),
            Span::raw(": Type value  "),
            Span::styled("↑/↓", Style::default().fg(theme.primary)),
            Span::raw(": ±10  "),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(theme.primary)),
            Span::raw(": Apply  "),
            Span::styled("Esc", Style::default().fg(theme.primary)),
            Span::raw(": Cancel  "),
            Span::styled("Backspace", Style::default().fg(theme.primary)),
            Span::raw(": Delete"),
        ]),
    ];

    let help_widget = Paragraph::new(help)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_muted));

    f.render_widget(help_widget, chunks[2]);
}

/// Render boolean toggle
fn render_boolean_toggle(
    f: &mut Frame,
    area: Rect,
    setting: SettingItem,
    value: bool,
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(5),    // Options
            Constraint::Length(4), // Help
        ])
        .split(area);

    // Title
    let title_text = Paragraph::new(setting.display_name())
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title_text, chunks[0]);

    // Options (On/Off)
    let items: Vec<ListItem> = [("On", true), ("Off", false)]
        .iter()
        .map(|(label, is_on)| {
            let selected = *is_on == value;
            let style = if selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let marker = if selected { "▶ " } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(marker, Style::default().fg(theme.primary)),
                Span::styled(*label, style),
            ]))
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
            Span::raw(": Toggle  "),
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
