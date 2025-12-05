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

use crate::models::{
    HoldDecisionMode, RgbBrightness, TapHoldPreset, TapHoldSettings, UncoloredKeyBehavior,
};

use super::Theme;

/// Setting group for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingGroup {
    // === Global Settings (stored in config.toml) ===
    /// Path configuration settings
    Paths,
    /// Firmware build settings
    Build,
    /// UI preferences
    Ui,

    // === Per-Layout Settings (stored in layout .md file) ===
    /// General layout settings
    General,
    /// RGB lighting settings
    Rgb,
    /// Tap-hold timing and behavior settings
    TapHold,
}

impl SettingGroup {
    /// Returns all groups in display order.
    /// Note: Kept for API completeness - useful for iterating over all groups.
    #[allow(dead_code)]
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            // Global settings first
            Self::Paths,
            Self::Build,
            Self::Ui,
            // Per-layout settings
            Self::General,
            Self::Rgb,
            Self::TapHold,
        ]
    }

    /// Returns display name.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Paths => "Paths [Global]",
            Self::Build => "Build [Global]",
            Self::Ui => "UI [Global]",
            Self::General => "General [Layout]",
            Self::Rgb => "RGB Lighting [Layout]",
            Self::TapHold => "Tap-Hold [Layout]",
        }
    }

    /// Returns whether this is a global setting (stored in config.toml)
    #[must_use]
    pub const fn is_global(&self) -> bool {
        matches!(self, Self::Paths | Self::Build | Self::Ui)
    }
}

/// Available settings that can be configured
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingItem {
    // === Paths Settings (Global) ===
    /// QMK firmware directory path
    QmkFirmwarePath,

    // === Build Settings (Global) ===
    /// Target keyboard
    Keyboard,
    /// Layout variant
    LayoutVariant,
    /// Keymap name
    KeymapName,
    /// Output format (uf2, hex, bin)
    OutputFormat,
    /// Build output directory
    OutputDir,

    // === UI Settings (Global) ===
    /// Display help on startup
    ShowHelpOnStartup,

    // === RGB Settings (Per-Layout) ===
    /// Master switch for all RGB LEDs
    RgbEnabled,
    /// Global RGB brightness (0-100%)
    RgbBrightness,
    /// RGB Matrix timeout (auto-off after inactivity)
    RgbTimeout,
    /// Brightness for keys without individual/category colors (0-100%)
    UncoloredKeyBehavior,

    // === Tap-Hold Settings (Per-Layout) ===
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
    /// Number of taps to toggle layer with `TT()`
    TappingToggle,
    /// Flow tap timing for home-row mod optimization
    FlowTapTerm,
    /// Use opposite-hand rule for tap-hold decision
    ChordalHold,
}

impl SettingItem {
    /// Returns all settings in a single flat list.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            // Paths (Global)
            Self::QmkFirmwarePath,
            // Build (Global)
            Self::Keyboard,
            Self::LayoutVariant,
            Self::KeymapName,
            Self::OutputFormat,
            Self::OutputDir,
            // UI (Global)
            Self::ShowHelpOnStartup,
            // RGB (Per-Layout)
            Self::RgbEnabled,
            Self::RgbBrightness,
            Self::RgbTimeout,
            Self::UncoloredKeyBehavior,
            // Tap-Hold (Per-Layout)
            Self::TapHoldPreset,
            Self::TappingTerm,
            Self::QuickTapTerm,
            Self::HoldMode,
            Self::RetroTapping,
            Self::TappingToggle,
            Self::FlowTapTerm,
            Self::ChordalHold,
        ]
    }

    /// Returns which group this setting belongs to.
    #[must_use]
    pub const fn group(&self) -> SettingGroup {
        match self {
            Self::QmkFirmwarePath => SettingGroup::Paths,
            Self::Keyboard
            | Self::LayoutVariant
            | Self::KeymapName
            | Self::OutputFormat
            | Self::OutputDir => SettingGroup::Build,
            Self::ShowHelpOnStartup => SettingGroup::Ui,
            Self::RgbEnabled
            | Self::RgbBrightness
            | Self::RgbTimeout
            | Self::UncoloredKeyBehavior => SettingGroup::Rgb,
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
            Self::QmkFirmwarePath => "QMK Firmware Path",
            Self::Keyboard => "Keyboard",
            Self::LayoutVariant => "Layout Variant",
            Self::KeymapName => "Keymap Name",
            Self::OutputFormat => "Output Format",
            Self::OutputDir => "Output Directory",
            Self::ShowHelpOnStartup => "Show Help on Startup",
            Self::RgbEnabled => "RGB Master Switch",
            Self::RgbBrightness => "RGB Brightness",
            Self::RgbTimeout => "RGB Timeout",
            Self::UncoloredKeyBehavior => "Uncolored Key Brightness",
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
            Self::QmkFirmwarePath => "Path to QMK firmware directory (required for builds)",
            Self::Keyboard => "Target keyboard for firmware builds",
            Self::LayoutVariant => "Physical layout variant (e.g., LAYOUT_split_3x6_3)",
            Self::KeymapName => "Name of the keymap (e.g., 'default', 'mymap')",
            Self::OutputFormat => "Firmware output format: uf2, hex, or bin",
            Self::OutputDir => "Directory where built firmware will be saved",
            Self::ShowHelpOnStartup => "Display help overlay when application starts",
            Self::RgbEnabled => "Turn all RGB LEDs on or off",
            Self::RgbBrightness => "Global brightness multiplier for all LEDs (0-100%)",
            Self::RgbTimeout => "Auto-off RGB after inactivity (0 = disabled)",
            Self::UncoloredKeyBehavior => {
                "Brightness for keys without individual/category colors (0=Off, 100=Full)"
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

    /// Returns whether this is a global setting (stored in config.toml)
    #[must_use]
    pub const fn is_global(&self) -> bool {
        self.group().is_global()
    }
}

/// Manager mode - determines what operation is being performed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerMode {
    /// Browsing settings (default mode)
    Browsing,
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
    /// Editing a numeric value (`tapping_term`, `quick_tap_term`, etc.)
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
    /// Toggling a boolean value (`retro_tapping`, `chordal_hold`)
    TogglingBoolean {
        /// Which setting is being toggled
        setting: SettingItem,
        /// Current value
        value: bool,
    },
    /// Editing a string value (keymap name, etc.)
    EditingString {
        /// Which setting is being edited
        setting: SettingItem,
        /// Current value
        value: String,
    },
    /// Selecting output format (uf2, hex, bin)
    SelectingOutputFormat {
        /// Currently highlighted option index
        selected_option: usize,
    },
    /// Editing a path (QMK path, output dir)
    EditingPath {
        /// Which setting is being edited
        setting: SettingItem,
        /// Current value
        value: String,
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
    pub const fn select_previous(&mut self, setting_count: usize) {
        if setting_count > 0 {
            if self.selected > 0 {
                self.selected -= 1;
            } else {
                self.selected = setting_count - 1;
            }
        }
    }

    /// Move selection down
    pub const fn select_next(&mut self, setting_count: usize) {
        if setting_count > 0 {
            self.selected = (self.selected + 1) % setting_count;
        }
    }

    /// Start selecting uncolored key behavior
    // Removed: UncoloredKeyBehavior is now a simple percentage, not a selector

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
    pub fn start_editing_numeric(
        &mut self,
        setting: SettingItem,
        current: u16,
        min: u16,
        max: u16,
    ) {
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
    pub const fn option_previous(&mut self, option_count: usize) {
        match &mut self.mode {
            ManagerMode::SelectingTapHoldPreset { selected_option }
            | ManagerMode::SelectingHoldMode { selected_option }
            | ManagerMode::SelectingOutputFormat { selected_option } => {
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
    pub const fn option_next(&mut self, option_count: usize) {
        match &mut self.mode {
            ManagerMode::SelectingTapHoldPreset { selected_option }
            | ManagerMode::SelectingHoldMode { selected_option }
            | ManagerMode::SelectingOutputFormat { selected_option } => {
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
    pub const fn get_selected_option(&self) -> Option<usize> {
        match &self.mode {
            ManagerMode::SelectingTapHoldPreset { selected_option }
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
    pub const fn get_boolean_value(&self) -> Option<bool> {
        if let ManagerMode::TogglingBoolean { value, .. } = &self.mode {
            Some(*value)
        } else {
            None
        }
    }

    /// Start editing a string value
    pub fn start_editing_string(&mut self, setting: SettingItem, current: String) {
        self.mode = ManagerMode::EditingString {
            setting,
            value: current,
        };
    }

    /// Start editing a path value
    pub fn start_editing_path(&mut self, setting: SettingItem, current: String) {
        self.mode = ManagerMode::EditingPath {
            setting,
            value: current,
        };
    }

    /// Start selecting output format
    pub fn start_selecting_output_format(&mut self, selected: usize) {
        self.mode = ManagerMode::SelectingOutputFormat {
            selected_option: selected,
        };
    }

    /// Handle character input for string/path editing
    pub fn handle_string_char_input(&mut self, c: char) {
        match &mut self.mode {
            ManagerMode::EditingString { value, .. } | ManagerMode::EditingPath { value, .. } => {
                value.push(c);
            }
            _ => {}
        }
    }

    /// Handle backspace for string/path editing
    pub fn handle_string_backspace(&mut self) {
        match &mut self.mode {
            ManagerMode::EditingString { value, .. } | ManagerMode::EditingPath { value, .. } => {
                value.pop();
            }
            _ => {}
        }
    }

    /// Get the current string value being edited
    #[must_use]
    pub fn get_string_value(&self) -> Option<&str> {
        match &self.mode {
            ManagerMode::EditingString { value, .. } | ManagerMode::EditingPath { value, .. } => {
                Some(value)
            }
            _ => None,
        }
    }

    /// Get the selected output format index
    #[must_use]
    pub const fn get_output_format_selected(&self) -> Option<usize> {
        if let ManagerMode::SelectingOutputFormat { selected_option } = &self.mode {
            Some(*selected_option)
        } else {
            None
        }
    }

    /// Cancel current operation and return to browsing
    pub fn cancel(&mut self) {
        self.mode = ManagerMode::Browsing;
    }

    /// Check if we're in browsing mode.
    /// Note: Kept for API completeness - pattern matching on mode is preferred in practice.
    #[allow(dead_code)]
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
    rgb_enabled: bool,
    rgb_brightness: RgbBrightness,
    rgb_timeout_ms: u32,
    uncolored_key_behavior: UncoloredKeyBehavior,
    tap_hold_settings: &TapHoldSettings,
    config: &crate::config::Config,
    layout: &crate::models::Layout,
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
            render_settings_list(
                f,
                inner_area,
                state,
                rgb_enabled,
                rgb_brightness,
                rgb_timeout_ms,
                uncolored_key_behavior,
                tap_hold_settings,
                config,
                layout,
                theme,
            );
        }
        ManagerMode::SelectingTapHoldPreset { selected_option } => {
            render_tap_hold_preset_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::SelectingHoldMode { selected_option } => {
            render_hold_mode_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::EditingNumeric {
            setting,
            value,
            min,
            max,
        } => {
            render_numeric_editor(f, inner_area, *setting, value, *min, *max, theme);
        }
        ManagerMode::TogglingBoolean { setting, value } => {
            render_boolean_toggle(f, inner_area, *setting, *value, theme);
        }
        ManagerMode::EditingString { setting, value } => {
            render_string_editor(f, inner_area, *setting, value, theme);
        }
        ManagerMode::SelectingOutputFormat { selected_option } => {
            render_output_format_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::EditingPath { setting, value } => {
            render_path_editor(f, inner_area, *setting, value, theme);
        }
    }
}

/// Render the list of settings
fn render_settings_list(
    f: &mut Frame,
    area: Rect,
    state: &SettingsManagerState,
    rgb_enabled: bool,
    rgb_brightness: RgbBrightness,
    rgb_timeout_ms: u32,
    uncolored_key_behavior: UncoloredKeyBehavior,
    tap_hold_settings: &TapHoldSettings,
    config: &crate::config::Config,
    layout: &crate::models::Layout,
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
        let value = get_setting_value_display(
            *setting,
            rgb_enabled,
            rgb_brightness,
            rgb_timeout_ms,
            uncolored_key_behavior,
            tap_hold_settings,
            config,
            Some(layout),
        );

        let marker = if display_index == state.selected {
            "▶ "
        } else {
            "  "
        };

        // Show indicator for global settings (stored in config.toml)
        let scope_indicator = if setting.is_global() {
            Span::styled("[G] ", Style::default().fg(theme.text_muted))
        } else {
            Span::styled("[L] ", Style::default().fg(theme.text_muted))
        };

        let content = Line::from(vec![
            Span::styled(marker, Style::default().fg(theme.primary)),
            scope_indicator,
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
    let description = selected_setting.map_or("", SettingItem::description);

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
    rgb_enabled: bool,
    rgb_brightness: RgbBrightness,
    rgb_timeout_ms: u32,
    uncolored_key_behavior: UncoloredKeyBehavior,
    tap_hold: &TapHoldSettings,
    config: &crate::config::Config,
    layout: Option<&crate::models::Layout>,
) -> String {
    match setting {
        // Global: Paths
        SettingItem::QmkFirmwarePath => config
            .paths
            .qmk_firmware
            .as_ref()
            .map_or_else(|| "<not set>".to_string(), |p| p.display().to_string()),
        // Per-Layout: Build settings (now in layout metadata)
        SettingItem::Keyboard => layout
            .as_ref()
            .and_then(|l| l.metadata.keyboard.clone())
            .unwrap_or_else(|| "<not set>".to_string()),
        SettingItem::LayoutVariant => layout
            .as_ref()
            .and_then(|l| l.metadata.layout_variant.clone())
            .unwrap_or_else(|| "<not set>".to_string()),
        SettingItem::KeymapName => layout
            .as_ref()
            .and_then(|l| l.metadata.keymap_name.clone())
            .unwrap_or_else(|| "<not set>".to_string()),
        SettingItem::OutputFormat => layout
            .as_ref()
            .and_then(|l| l.metadata.output_format.clone())
            .unwrap_or_else(|| "<not set>".to_string()),
        SettingItem::OutputDir => config.build.output_dir.display().to_string(),
        // Global: UI
        SettingItem::ShowHelpOnStartup => if config.ui.show_help_on_startup {
            "On"
        } else {
            "Off"
        }
        .to_string(),
        // Per-Layout: RGB
        SettingItem::RgbEnabled => if rgb_enabled { "On" } else { "Off" }.to_string(),
        SettingItem::RgbBrightness => format!("{}%", rgb_brightness.as_percent()),
        SettingItem::RgbTimeout => {
            if rgb_timeout_ms == 0 {
                "Disabled".to_string()
            } else if rgb_timeout_ms >= 60000 && rgb_timeout_ms % 60000 == 0 {
                format!("{} min", rgb_timeout_ms / 60000)
            } else if rgb_timeout_ms >= 1000 && rgb_timeout_ms % 1000 == 0 {
                format!("{} sec", rgb_timeout_ms / 1000)
            } else {
                format!("{rgb_timeout_ms}ms")
            }
        }
        SettingItem::UncoloredKeyBehavior => format!("{}%", uncolored_key_behavior.as_percent()),
        // Per-Layout: Tap-Hold
        SettingItem::TapHoldPreset => tap_hold.preset.display_name().to_string(),
        SettingItem::TappingTerm => format!("{}ms", tap_hold.tapping_term),
        SettingItem::QuickTapTerm => match tap_hold.quick_tap_term {
            Some(term) => format!("{term}ms"),
            None => "Auto".to_string(),
        },
        SettingItem::HoldMode => tap_hold.hold_mode.display_name().to_string(),
        SettingItem::RetroTapping => if tap_hold.retro_tapping { "On" } else { "Off" }.to_string(),
        SettingItem::TappingToggle => format!("{} taps", tap_hold.tapping_toggle),
        SettingItem::FlowTapTerm => match tap_hold.flow_tap_term {
            Some(term) => format!("{term}ms"),
            None => "Disabled".to_string(),
        },
        SettingItem::ChordalHold => if tap_hold.chordal_hold { "On" } else { "Off" }.to_string(),
    }
}

/// Render inactive behavior selector
// Removed: UncoloredKeyBehavior is now a simple percentage, not a selector

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
fn render_enum_selector<S1: AsRef<str>, S2: AsRef<str>>(
    f: &mut Frame,
    area: Rect,
    title: &str,
    options: &[(S1, S2)],
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
                Span::styled(name.as_ref(), style),
                Span::styled(" - ", Style::default().fg(theme.text_muted)),
                Span::styled(desc.as_ref(), Style::default().fg(theme.text_muted)),
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

/// Render numeric editor for integer values
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
            Constraint::Length(3), // Input field
            Constraint::Min(2),    // Description + range
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

    // Input field with cursor
    let display_value = format!("{value}▌");
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title("Value");
    let input_text = Paragraph::new(display_value)
        .style(Style::default().fg(theme.text))
        .block(input_block);
    f.render_widget(input_text, chunks[1]);

    // Description and range
    let desc = vec![
        Line::from(setting.description()),
        Line::from(""),
        Line::from(Span::styled(
            format!("Range: {min} to {max}"),
            Style::default().fg(theme.text_muted),
        )),
    ];

    let desc_text = Paragraph::new(desc)
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(Style::default().fg(theme.text_muted));
    f.render_widget(desc_text, chunks[2]);

    // Help text
    let help = vec![
        Line::from(vec![
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

    f.render_widget(help_widget, chunks[3]);
}

/// Render string editor (for keymap name, etc.)
fn render_string_editor(
    f: &mut Frame,
    area: Rect,
    setting: SettingItem,
    value: &str,
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Length(3), // Input field
            Constraint::Min(2),    // Description
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

    // Input field with cursor
    let display_value = format!("{value}▌");
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title("Value");
    let input_text = Paragraph::new(display_value)
        .style(Style::default().fg(theme.text))
        .block(input_block);
    f.render_widget(input_text, chunks[1]);

    // Description
    let desc_text = Paragraph::new(setting.description())
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(Style::default().fg(theme.text_muted));
    f.render_widget(desc_text, chunks[2]);

    // Help text
    let help = vec![
        Line::from(""),
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

    f.render_widget(help_widget, chunks[3]);
}

/// Render output format selector
fn render_output_format_selector(f: &mut Frame, area: Rect, selected_option: usize, theme: &Theme) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(5),    // Options
            Constraint::Length(4), // Help
        ])
        .split(area);

    // Title
    let title_text = Paragraph::new("Output Format")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title_text, chunks[0]);

    // Options
    let options = ["uf2", "hex", "bin"];
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, label)| {
            let selected = idx == selected_option;
            let style = if selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let marker = if selected { "▶ " } else { "  " };
            let description = match *label {
                "uf2" => " (USB Flashing Format - RP2040, etc.)",
                "hex" => " (Intel HEX - AVR, etc.)",
                "bin" => " (Raw binary)",
                _ => "",
            };

            ListItem::new(Line::from(vec![
                Span::styled(marker, Style::default().fg(theme.primary)),
                Span::styled(*label, style),
                Span::styled(description, Style::default().fg(theme.text_muted)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Format"))
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

/// Render path editor (for QMK path, output directory, etc.)
fn render_path_editor(f: &mut Frame, area: Rect, setting: SettingItem, value: &str, theme: &Theme) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Length(3), // Input field
            Constraint::Min(2),    // Description + current path
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

    // Input field with cursor
    let display_value = format!("{value}▌");
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title("Path");
    let input_text = Paragraph::new(display_value)
        .style(Style::default().fg(theme.text))
        .block(input_block);
    f.render_widget(input_text, chunks[1]);

    // Description and path validation hint
    let path_exists = std::path::Path::new(value).exists();
    let status_line = if value.is_empty() {
        Line::from(Span::styled(
            "Enter a path",
            Style::default().fg(theme.text_muted),
        ))
    } else if path_exists {
        Line::from(Span::styled(
            "✓ Path exists",
            Style::default().fg(theme.primary),
        ))
    } else {
        Line::from(Span::styled(
            "⚠ Path does not exist (will be created if needed)",
            Style::default().fg(theme.warning),
        ))
    };

    let desc = vec![
        Line::from(setting.description()),
        Line::from(""),
        status_line,
    ];

    let desc_text = Paragraph::new(desc)
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(Style::default().fg(theme.text_muted));
    f.render_widget(desc_text, chunks[2]);

    // Help text
    let help = vec![
        Line::from(""),
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

    f.render_widget(help_widget, chunks[3]);
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
