//! State methods for SettingsManagerState.

use crate::models::{
    HoldDecisionMode, PaletteFxEffect, PaletteFxPalette, RgbMatrixEffect, RippleColorMode,
    TapHoldPreset,
};

use super::{ManagerMode, SettingItem, SettingsManagerState};

impl SettingsManagerState {
    /// Create a new settings manager state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected: 0,
            mode: ManagerMode::Browsing,
        }
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

    // /// Start selecting uncolored key behavior
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
        default: u16,
    ) {
        self.mode = ManagerMode::EditingNumeric {
            setting,
            value: current.to_string(),
            min,
            max,
            default,
        };
    }

    /// Start editing a signed numeric value
    pub fn start_editing_signed_numeric(
        &mut self,
        setting: SettingItem,
        current: i16,
        min: i16,
        max: i16,
        default: i16,
    ) {
        self.mode = ManagerMode::EditingSignedNumeric {
            setting,
            value: current.to_string(),
            min,
            max,
            default,
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
            | ManagerMode::SelectingOutputFormat { selected_option }
            | ManagerMode::SelectingThemeMode { selected_option }
            | ManagerMode::SelectingIdleEffectMode { selected_option }
            | ManagerMode::SelectingRippleColorMode { selected_option }
            | ManagerMode::SelectingPaletteFxEffect { selected_option }
            | ManagerMode::SelectingPaletteFxPalette { selected_option }
            | ManagerMode::SelectingKeyActionPalette { selected_option } => {
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
            | ManagerMode::SelectingOutputFormat { selected_option }
            | ManagerMode::SelectingThemeMode { selected_option }
            | ManagerMode::SelectingIdleEffectMode { selected_option }
            | ManagerMode::SelectingRippleColorMode { selected_option }
            | ManagerMode::SelectingPaletteFxEffect { selected_option }
            | ManagerMode::SelectingPaletteFxPalette { selected_option }
            | ManagerMode::SelectingKeyActionPalette { selected_option } => {
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
            | ManagerMode::SelectingHoldMode { selected_option }
            | ManagerMode::SelectingOutputFormat { selected_option }
            | ManagerMode::SelectingThemeMode { selected_option }
            | ManagerMode::SelectingIdleEffectMode { selected_option }
            | ManagerMode::SelectingRippleColorMode { selected_option }
            | ManagerMode::SelectingPaletteFxEffect { selected_option }
            | ManagerMode::SelectingPaletteFxPalette { selected_option }
            | ManagerMode::SelectingKeyActionPalette { selected_option } => Some(*selected_option),
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

    /// Handle character input for signed numeric editing
    pub fn handle_signed_char_input(&mut self, c: char) {
        if let ManagerMode::EditingSignedNumeric {
            value, min, max, ..
        } = &mut self.mode
        {
            if c.is_ascii_digit() {
                value.push(c);
            } else if c == '-' && value.is_empty() && *min < 0 {
                value.push(c);
            } else {
                return;
            }

            if value.len() > 5 {
                value.pop();
                return;
            }

            if let Ok(num) = value.parse::<i16>() {
                if num > *max {
                    *value = max.to_string();
                } else if num < *min {
                    *value = min.to_string();
                }
            }
        }
    }

    /// Handle backspace for numeric editing
    pub fn handle_backspace(&mut self) {
        match &mut self.mode {
            ManagerMode::EditingNumeric { value, .. }
            | ManagerMode::EditingSignedNumeric { value, .. } => {
                value.pop();
            }
            _ => {}
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

    /// Reset numeric editor to suggested default value.
    pub fn reset_numeric_to_default(&mut self) {
        if let ManagerMode::EditingNumeric { value, default, .. } = &mut self.mode {
            *value = default.to_string();
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

    /// Increment signed numeric value
    pub fn increment_signed_numeric(&mut self, step: i16) {
        if let ManagerMode::EditingSignedNumeric { value, max, .. } = &mut self.mode {
            if let Ok(mut num) = value.parse::<i16>() {
                num = num.saturating_add(step).min(*max);
                *value = num.to_string();
            }
        }
    }

    /// Decrement signed numeric value
    pub fn decrement_signed_numeric(&mut self, step: i16) {
        if let ManagerMode::EditingSignedNumeric { value, min, .. } = &mut self.mode {
            if let Ok(mut num) = value.parse::<i16>() {
                num = num.saturating_sub(step).max(*min);
                *value = num.to_string();
            }
        }
    }

    /// Reset signed numeric editor to suggested default value.
    pub fn reset_signed_numeric_to_default(&mut self) {
        if let ManagerMode::EditingSignedNumeric { value, default, .. } = &mut self.mode {
            *value = default.to_string();
        }
    }

    /// Get current signed numeric value being edited
    #[must_use]
    pub fn get_signed_numeric_value(&self) -> Option<i16> {
        if let ManagerMode::EditingSignedNumeric { value, min, .. } = &self.mode {
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

    /// Start selecting theme mode
    pub fn start_selecting_theme_mode(&mut self, selected: usize) {
        self.mode = ManagerMode::SelectingThemeMode {
            selected_option: selected,
        };
    }

    /// Start selecting idle effect mode
    pub fn start_selecting_idle_effect_mode(&mut self, current: RgbMatrixEffect) {
        let selected_option = RgbMatrixEffect::all()
            .iter()
            .position(|&e| e == current)
            .unwrap_or(0);
        self.mode = ManagerMode::SelectingIdleEffectMode { selected_option };
    }

    /// Start selecting ripple color mode
    pub fn start_selecting_ripple_color_mode(&mut self, current: RippleColorMode) {
        let selected_option = RippleColorMode::all()
            .iter()
            .position(|&m| m == current)
            .unwrap_or(0);
        self.mode = ManagerMode::SelectingRippleColorMode { selected_option };
    }

    /// Start selecting `PaletteFX` effect
    pub fn start_selecting_palette_fx_effect(&mut self, current: PaletteFxEffect) {
        let selected_option = PaletteFxEffect::all()
            .iter()
            .position(|&e| e == current)
            .unwrap_or(0);
        self.mode = ManagerMode::SelectingPaletteFxEffect { selected_option };
    }

    /// Start selecting `PaletteFX` palette
    pub fn start_selecting_palette_fx_palette(&mut self, current: PaletteFxPalette) {
        let selected_option = PaletteFxPalette::all()
            .iter()
            .position(|&p| p == current)
            .unwrap_or(0);
        self.mode = ManagerMode::SelectingPaletteFxPalette { selected_option };
    }

    /// Start selecting the key-action reactive palette (with "Default" as first option)
    pub fn start_selecting_key_action_palette(&mut self, current: Option<PaletteFxPalette>) {
        // Option 0 = "Default" (use current palette)
        // Options 1.. = specific palettes
        let selected_option = current.map_or(0, |p| {
            PaletteFxPalette::all()
                .iter()
                .position(|&pal| pal == p)
                .map_or(0, |i| i + 1)
        });
        self.mode = ManagerMode::SelectingKeyActionPalette { selected_option };
    }

    /// Start selecting a key position (for combo configuration)
    pub fn start_selecting_key_position(&mut self, setting: SettingItem, instruction: String) {
        self.mode = ManagerMode::SelectingKeyPosition {
            setting,
            instruction,
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
}

impl Default for SettingsManagerState {
    fn default() -> Self {
        Self::new()
    }
}
