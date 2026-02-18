//! Settings manager for global layout settings.
//!
//! Provides a UI for configuring layout-wide settings like inactive key behavior
//! and tap-hold timing configuration.
//! Accessible via Shift+S shortcut.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::models::{
    HoldDecisionMode, IdleEffectSettings, RgbBrightness, RgbMatrixEffect, RgbOverlayRippleSettings,
    RippleColorMode, TapHoldPreset, TapHoldSettings, UncoloredKeyBehavior,
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
    #[allow(dead_code)] // Placeholder for future general settings UI
    General,
    /// RGB lighting settings
    Rgb,
    /// Tap-hold timing and behavior settings
    TapHold,
    /// Two-key hold combo settings
    Combos,
}

impl SettingGroup {
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
            Self::Combos => "Combos [Layout]",
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
    /// Theme mode (Auto, Dark, Light)
    ThemeMode,
    /// Unified keyboard scale factor
    KeyboardScale,

    // === RGB Settings (Per-Layout) ===
    /// Master switch for all RGB LEDs
    RgbEnabled,
    /// Global RGB brightness (0-100%)
    RgbBrightness,
    /// RGB color saturation (0-200%)
    RgbSaturation,
    /// RGB Matrix default animation speed (0-255)
    RgbMatrixSpeed,
    /// RGB Matrix timeout (auto-off after inactivity)
    RgbTimeout,
    /// Idle effect master switch (idle → effect → off)
    IdleEffectEnabled,
    /// Idle timeout before starting idle effect (ms)
    IdleTimeout,
    /// How long to run idle effect before turning off (ms)
    IdleEffectDuration,
    /// Idle effect mode (standard RGB effects)
    IdleEffectMode,
    /// Brightness for keys without individual/category colors (0-100%)
    UncoloredKeyBehavior,
    /// Overlay ripple master switch
    OverlayRippleEnabled,
    /// Maximum concurrent ripples (1-8)
    OverlayRippleMaxRipples,
    /// Ripple duration in milliseconds
    OverlayRippleDuration,
    /// Ripple expansion speed (0-255)
    OverlayRippleSpeed,
    /// Ripple band width in LED units
    OverlayRippleBandWidth,
    /// Ripple amplitude as percentage (0-100%)
    OverlayRippleAmplitude,
    /// Ripple color mode (Fixed, KeyBased, HueShift)
    OverlayRippleColorMode,
    /// Fixed color for ripples
    OverlayRippleFixedColor,
    /// Hue shift in degrees (-180 to 180)
    OverlayRippleHueShift,
    /// Trigger on key press
    OverlayRippleTriggerPress,
    /// Trigger on key release
    OverlayRippleTriggerRelease,
    /// Ignore transparent keys
    OverlayRippleIgnoreTransparent,
    /// Ignore modifier keys
    OverlayRippleIgnoreModifiers,
    /// Ignore layer switch keys
    OverlayRippleIgnoreLayerSwitch,

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

    // === Combo Settings (Per-Layout) ===
    /// Master switch for combo feature
    CombosEnabled,
    /// Configure Combo 1: Disable Effects - First key position
    Combo1Key1,
    /// Configure Combo 1: Disable Effects - Second key position
    Combo1Key2,
    /// Configure Combo 1: Disable Effects - Hold duration in milliseconds
    Combo1HoldDuration,
    /// Configure Combo 2: Disable Lighting - First key position
    Combo2Key1,
    /// Configure Combo 2: Disable Lighting - Second key position
    Combo2Key2,
    /// Configure Combo 2: Disable Lighting - Hold duration in milliseconds
    Combo2HoldDuration,
    /// Configure Combo 3: Bootloader - First key position
    Combo3Key1,
    /// Configure Combo 3: Bootloader - Second key position
    Combo3Key2,
    /// Configure Combo 3: Bootloader - Hold duration in milliseconds
    Combo3HoldDuration,
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
            Self::ThemeMode,
            Self::KeyboardScale,
            // RGB (Per-Layout)
            Self::RgbEnabled,
            Self::RgbBrightness,
            Self::RgbSaturation,
            Self::RgbMatrixSpeed,
            Self::RgbTimeout,
            Self::IdleEffectEnabled,
            Self::IdleTimeout,
            Self::IdleEffectDuration,
            Self::IdleEffectMode,
            Self::UncoloredKeyBehavior,
            Self::OverlayRippleEnabled,
            Self::OverlayRippleMaxRipples,
            Self::OverlayRippleDuration,
            Self::OverlayRippleSpeed,
            Self::OverlayRippleBandWidth,
            Self::OverlayRippleAmplitude,
            Self::OverlayRippleColorMode,
            Self::OverlayRippleFixedColor,
            Self::OverlayRippleHueShift,
            Self::OverlayRippleTriggerPress,
            Self::OverlayRippleTriggerRelease,
            Self::OverlayRippleIgnoreTransparent,
            Self::OverlayRippleIgnoreModifiers,
            Self::OverlayRippleIgnoreLayerSwitch,
            // Tap-Hold (Per-Layout)
            Self::TapHoldPreset,
            Self::TappingTerm,
            Self::QuickTapTerm,
            Self::HoldMode,
            Self::RetroTapping,
            Self::TappingToggle,
            Self::FlowTapTerm,
            Self::ChordalHold,
            // Combos (Per-Layout)
            Self::CombosEnabled,
            Self::Combo1Key1,
            Self::Combo1Key2,
            Self::Combo1HoldDuration,
            Self::Combo2Key1,
            Self::Combo2Key2,
            Self::Combo2HoldDuration,
            Self::Combo3Key1,
            Self::Combo3Key2,
            Self::Combo3HoldDuration,
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
            Self::ShowHelpOnStartup | Self::ThemeMode | Self::KeyboardScale => SettingGroup::Ui,
            Self::RgbEnabled
            | Self::RgbBrightness
            | Self::RgbSaturation
            | Self::RgbMatrixSpeed
            | Self::RgbTimeout
            | Self::IdleEffectEnabled
            | Self::IdleTimeout
            | Self::IdleEffectDuration
            | Self::IdleEffectMode
            | Self::UncoloredKeyBehavior
            | Self::OverlayRippleEnabled
            | Self::OverlayRippleMaxRipples
            | Self::OverlayRippleDuration
            | Self::OverlayRippleSpeed
            | Self::OverlayRippleBandWidth
            | Self::OverlayRippleAmplitude
            | Self::OverlayRippleColorMode
            | Self::OverlayRippleFixedColor
            | Self::OverlayRippleHueShift
            | Self::OverlayRippleTriggerPress
            | Self::OverlayRippleTriggerRelease
            | Self::OverlayRippleIgnoreTransparent
            | Self::OverlayRippleIgnoreModifiers
            | Self::OverlayRippleIgnoreLayerSwitch => SettingGroup::Rgb,
            Self::TapHoldPreset
            | Self::TappingTerm
            | Self::QuickTapTerm
            | Self::HoldMode
            | Self::RetroTapping
            | Self::TappingToggle
            | Self::FlowTapTerm
            | Self::ChordalHold => SettingGroup::TapHold,
            Self::CombosEnabled
            | Self::Combo1Key1
            | Self::Combo1Key2
            | Self::Combo1HoldDuration
            | Self::Combo2Key1
            | Self::Combo2Key2
            | Self::Combo2HoldDuration
            | Self::Combo3Key1
            | Self::Combo3Key2
            | Self::Combo3HoldDuration => SettingGroup::Combos,
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
            Self::ThemeMode => "Theme Mode",
            Self::KeyboardScale => "Keyboard Scale",
            Self::RgbEnabled => "RGB Master Switch",
            Self::RgbBrightness => "RGB Brightness",
            Self::RgbSaturation => "RGB Saturation",
            Self::RgbMatrixSpeed => "RGB Matrix Speed",
            Self::RgbTimeout => "RGB Timeout",
            Self::IdleEffectEnabled => "Idle Effect Enabled",
            Self::IdleTimeout => "Idle Timeout",
            Self::IdleEffectDuration => "Idle Effect Duration",
            Self::IdleEffectMode => "Idle Effect Mode",
            Self::UncoloredKeyBehavior => "Uncolored Key Brightness",
            Self::OverlayRippleEnabled => "Overlay Ripple Enabled",
            Self::OverlayRippleMaxRipples => "Max Concurrent Ripples",
            Self::OverlayRippleDuration => "Ripple Duration",
            Self::OverlayRippleSpeed => "Ripple Speed",
            Self::OverlayRippleBandWidth => "Ripple Band Width",
            Self::OverlayRippleAmplitude => "Ripple Amplitude",
            Self::OverlayRippleColorMode => "Ripple Color Mode",
            Self::OverlayRippleFixedColor => "Ripple Fixed Color",
            Self::OverlayRippleHueShift => "Ripple Hue Shift",
            Self::OverlayRippleTriggerPress => "Trigger on Press",
            Self::OverlayRippleTriggerRelease => "Trigger on Release",
            Self::OverlayRippleIgnoreTransparent => "Ignore Transparent Keys",
            Self::OverlayRippleIgnoreModifiers => "Ignore Modifier Keys",
            Self::OverlayRippleIgnoreLayerSwitch => "Ignore Layer Switch Keys",
            Self::TapHoldPreset => "Preset",
            Self::TappingTerm => "Tapping Term",
            Self::QuickTapTerm => "Quick Tap Term",
            Self::HoldMode => "Hold Mode",
            Self::RetroTapping => "Retro Tapping",
            Self::TappingToggle => "Tapping Toggle",
            Self::FlowTapTerm => "Flow Tap Term",
            Self::ChordalHold => "Chordal Hold",
            // Combo Settings
            Self::CombosEnabled => "Combos Enabled",
            Self::Combo1Key1 => "Combo 1 Key 1 (Disable Effects)",
            Self::Combo1Key2 => "Combo 1 Key 2 (Disable Effects)",
            Self::Combo1HoldDuration => "Combo 1 Hold Duration",
            Self::Combo2Key1 => "Combo 2 Key 1 (Disable Lighting)",
            Self::Combo2Key2 => "Combo 2 Key 2 (Disable Lighting)",
            Self::Combo2HoldDuration => "Combo 2 Hold Duration",
            Self::Combo3Key1 => "Combo 3 Key 1 (Bootloader)",
            Self::Combo3Key2 => "Combo 3 Key 2 (Bootloader)",
            Self::Combo3HoldDuration => "Combo 3 Hold Duration",
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
            Self::ThemeMode => "Color theme: Auto (follow OS), Dark, or Light",
            Self::KeyboardScale => "Keyboard display size: 1.0 = default, 0.5 = half, 2.0 = double",
            Self::RgbEnabled => "Turn all RGB LEDs on or off",
            Self::RgbBrightness => "Global brightness multiplier for all LEDs (0-100%)",
            Self::RgbSaturation => {
                "Saturation multiplier for all LEDs (0=Grayscale, 100=Normal, 200=Maximum)"
            }
            Self::RgbMatrixSpeed => {
                "Animation speed for RGB effects (0=Slowest, 127=Default, 255=Fastest)"
            }
            Self::RgbTimeout => "Auto-off RGB after inactivity (0 = disabled)",
            Self::IdleEffectEnabled => "Enable idle effect (triggers RGB animation before timeout)",
            Self::IdleTimeout => "Delay before starting idle effect (0 = disabled)",
            Self::IdleEffectDuration => {
                "How long to run idle effect before turning off (0 = immediate)"
            }
            Self::IdleEffectMode => "RGB animation effect to use during idle period",
            Self::UncoloredKeyBehavior => {
                "Brightness for keys without individual/category colors (0=Off, 100=Full)"
            }
            Self::OverlayRippleEnabled => "Enable ripple overlay on keypresses",
            Self::OverlayRippleMaxRipples => "Maximum number of concurrent ripples (1-8)",
            Self::OverlayRippleDuration => "How long each ripple lasts in milliseconds",
            Self::OverlayRippleSpeed => "Expansion speed multiplier (0-255, higher = faster)",
            Self::OverlayRippleBandWidth => "Width of ripple band in LED units",
            Self::OverlayRippleAmplitude => "Brightness boost as percentage of base (0-100%)",
            Self::OverlayRippleColorMode => {
                "How to determine ripple colors (Fixed, Key Color, Hue Shift)"
            }
            Self::OverlayRippleFixedColor => "Color to use when color mode is Fixed",
            Self::OverlayRippleHueShift => {
                "Hue shift in degrees when mode is Hue Shift (-180 to 180)"
            }
            Self::OverlayRippleTriggerPress => "Trigger ripple effect on key press",
            Self::OverlayRippleTriggerRelease => "Trigger ripple effect on key release",
            Self::OverlayRippleIgnoreTransparent => {
                "Don't trigger ripples on transparent keys (KC_TRNS)"
            }
            Self::OverlayRippleIgnoreModifiers => "Don't trigger ripples on modifier keys",
            Self::OverlayRippleIgnoreLayerSwitch => "Don't trigger ripples on layer switch keys",
            Self::TapHoldPreset => "Quick configuration preset for common use cases",
            Self::TappingTerm => "Milliseconds to distinguish tap from hold (100-500ms)",
            Self::QuickTapTerm => "Window for tap-then-hold to trigger auto-repeat",
            Self::HoldMode => "How to decide tap vs hold when other keys are pressed",
            Self::RetroTapping => "Send tap keycode even if held past tapping term",
            Self::TappingToggle => "Number of taps to toggle layer with TT() keys (1-10)",
            Self::FlowTapTerm => "Rapid typing window to prevent accidental modifiers",
            Self::ChordalHold => "Use opposite-hand rule for tap-hold (great for HRM)",
            // Combo Settings
            Self::CombosEnabled => "Master switch for combo feature (two-key hold actions)",
            Self::Combo1Key1 => "First key position for Combo 1 (Disable RGB Effects)",
            Self::Combo1Key2 => "Second key position for Combo 1 (Disable RGB Effects)",
            Self::Combo1HoldDuration => "Hold duration in milliseconds for Combo 1 (50-2000ms)",
            Self::Combo2Key1 => "First key position for Combo 2 (Disable All Lighting)",
            Self::Combo2Key2 => "Second key position for Combo 2 (Disable All Lighting)",
            Self::Combo2HoldDuration => "Hold duration in milliseconds for Combo 2 (50-2000ms)",
            Self::Combo3Key1 => "First key position for Combo 3 (Enter Bootloader)",
            Self::Combo3Key2 => "Second key position for Combo 3 (Enter Bootloader)",
            Self::Combo3HoldDuration => "Hold duration in milliseconds for Combo 3 (50-2000ms)",
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
    /// Selecting theme mode (Auto, Dark, Light)
    SelectingThemeMode {
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
    /// Selecting idle effect mode
    SelectingIdleEffectMode {
        /// Currently highlighted option index
        selected_option: usize,
    },
    /// Selecting ripple color mode
    SelectingRippleColorMode {
        /// Currently highlighted option index
        selected_option: usize,
    },
    /// Selecting a key position (for combo configuration)
    SelectingKeyPosition {
        /// Which setting is being configured
        setting: SettingItem,
        /// Instruction message to show
        instruction: String,
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
            | ManagerMode::SelectingOutputFormat { selected_option }
            | ManagerMode::SelectingThemeMode { selected_option }
            | ManagerMode::SelectingIdleEffectMode { selected_option }
            | ManagerMode::SelectingRippleColorMode { selected_option } => {
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
            | ManagerMode::SelectingRippleColorMode { selected_option } => {
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
            | ManagerMode::SelectingRippleColorMode { selected_option } => Some(*selected_option),
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

/// Events emitted by the SettingsManager component
#[derive(Debug, Clone)]
pub enum SettingsManagerEvent {
    /// Settings were updated (settings applied and dialog should close)
    SettingsUpdated,
    /// User cancelled without making changes
    Cancelled,
    /// Component closed naturally
    #[allow(dead_code)]
    Closed,
}

/// Context data needed for SettingsManager to render and handle input
#[derive(Debug, Clone)]
pub struct SettingsManagerContext {
    /// RGB enabled flag
    pub rgb_enabled: bool,
    /// RGB brightness
    pub rgb_brightness: RgbBrightness,
    /// RGB timeout in milliseconds
    pub rgb_timeout_ms: u32,
    /// Uncolored key behavior
    pub uncolored_key_behavior: UncoloredKeyBehavior,
    /// Idle effect settings
    pub idle_effect_settings: IdleEffectSettings,
    /// Overlay ripple settings
    pub overlay_ripple_settings: RgbOverlayRippleSettings,
    /// Tap-hold settings
    pub tap_hold_settings: TapHoldSettings,
    /// Application config
    pub config: crate::config::Config,
    /// Current layout (for layout-specific settings)
    pub layout: crate::models::Layout,
}

/// SettingsManager component that implements the Component trait
///
/// This wraps `SettingsManagerState` to provide a self-contained component
/// that handles its own input and rendering. Due to the complexity of settings
/// (needing access to both config and layout data), this component needs context
/// passed through render and handle_input methods.
#[derive(Debug, Clone)]
pub struct SettingsManager {
    /// Internal state
    state: SettingsManagerState,
}

impl SettingsManager {
    /// Create a new SettingsManager
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: SettingsManagerState::new(),
        }
    }

    /// Get a reference to the internal state (for backward compatibility)
    #[must_use]
    pub const fn state(&self) -> &SettingsManagerState {
        &self.state
    }

    /// Get a mutable reference to the internal state (for backward compatibility)
    pub fn state_mut(&mut self) -> &mut SettingsManagerState {
        &mut self.state
    }

    /// Handle input with access to context data
    ///
    /// Returns Some(Event) if the component wants to signal something to the parent
    pub fn handle_input_with_context(
        &mut self,
        key: KeyEvent,
        context: &SettingsManagerContext,
    ) -> Option<SettingsManagerEvent> {
        match &self.state.mode.clone() {
            ManagerMode::Browsing => self.handle_browsing_input(key, context),
            ManagerMode::SelectingTapHoldPreset { .. } => {
                self.handle_preset_selection(key, context)
            }
            ManagerMode::SelectingHoldMode { .. } => self.handle_hold_mode_selection(key, context),
            ManagerMode::EditingNumeric { .. } => self.handle_numeric_editing(key),
            ManagerMode::TogglingBoolean { .. } => self.handle_boolean_toggle(key),
            ManagerMode::EditingString { .. } => self.handle_string_editing(key),
            ManagerMode::SelectingOutputFormat { .. } => self.handle_output_format_selection(key),
            ManagerMode::SelectingThemeMode { .. } => self.handle_theme_mode_selection(key),
            ManagerMode::EditingPath { .. } => self.handle_path_editing(key),
            ManagerMode::SelectingIdleEffectMode { .. } => {
                self.handle_idle_effect_mode_selection(key)
            }
            ManagerMode::SelectingRippleColorMode { .. } => {
                self.handle_ripple_color_mode_selection(key)
            }
            ManagerMode::SelectingKeyPosition { .. } => {
                // Key position selection is handled by the parent (main app input handler)
                // because it needs access to keyboard navigation state
                None
            }
        }
    }

    /// Render with access to context data
    pub fn render_with_context(
        &self,
        f: &mut Frame,
        area: Rect,
        theme: &Theme,
        context: &SettingsManagerContext,
    ) {
        render_settings_manager(
            f,
            area,
            &self.state,
            context.rgb_enabled,
            context.rgb_brightness,
            context.rgb_timeout_ms,
            context.uncolored_key_behavior,
            &context.idle_effect_settings,
            &context.overlay_ripple_settings,
            &context.tap_hold_settings,
            &context.config,
            &context.layout,
            theme,
        );
    }

    // Input handling methods
    fn handle_browsing_input(
        &mut self,
        key: KeyEvent,
        _context: &SettingsManagerContext,
    ) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => Some(SettingsManagerEvent::Cancelled),
            KeyCode::Up | KeyCode::Char('k') => {
                let count = SettingItem::all().len();
                self.state.select_previous(count);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = SettingItem::all().len();
                self.state.select_next(count);
                None
            }
            KeyCode::Enter => {
                // Note: Actual setting editing is complex and involves opening
                // sub-dialogs or other popups. This is handled by the parent.
                // We just signal that Enter was pressed on a setting.
                None
            }
            _ => None,
        }
    }

    fn handle_preset_selection(
        &mut self,
        key: KeyEvent,
        _context: &SettingsManagerContext,
    ) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = TapHoldPreset::all().len();
                self.state.option_previous(count);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = TapHoldPreset::all().len();
                self.state.option_next(count);
                None
            }
            KeyCode::Enter => {
                // Signal that a selection was made
                // Parent will extract the value from state
                Some(SettingsManagerEvent::SettingsUpdated)
            }
            _ => None,
        }
    }

    fn handle_hold_mode_selection(
        &mut self,
        key: KeyEvent,
        _context: &SettingsManagerContext,
    ) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = HoldDecisionMode::all().len();
                self.state.option_previous(count);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = HoldDecisionMode::all().len();
                self.state.option_next(count);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    fn handle_numeric_editing(&mut self, key: KeyEvent) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.increment_numeric(10);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.decrement_numeric(10);
                None
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.state.handle_char_input(c);
                None
            }
            KeyCode::Backspace => {
                self.state.handle_backspace();
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    fn handle_boolean_toggle(&mut self, key: KeyEvent) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Down | KeyCode::Char('k') | KeyCode::Char('j') => {
                self.state.option_previous(2);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    fn handle_string_editing(&mut self, key: KeyEvent) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Char(c) => {
                self.state.handle_string_char_input(c);
                None
            }
            KeyCode::Backspace => {
                self.state.handle_string_backspace();
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    fn handle_output_format_selection(&mut self, key: KeyEvent) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.option_previous(3);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.option_next(3);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    fn handle_theme_mode_selection(&mut self, key: KeyEvent) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.option_previous(3);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.option_next(3);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    fn handle_path_editing(&mut self, key: KeyEvent) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Char(c) => {
                self.state.handle_string_char_input(c);
                None
            }
            KeyCode::Backspace => {
                self.state.handle_string_backspace();
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    fn handle_idle_effect_mode_selection(&mut self, key: KeyEvent) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = RgbMatrixEffect::all().len();
                self.state.option_previous(count);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = RgbMatrixEffect::all().len();
                self.state.option_next(count);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    fn handle_ripple_color_mode_selection(
        &mut self,
        key: KeyEvent,
    ) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = RippleColorMode::all().len();
                self.state.option_previous(count);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = RippleColorMode::all().len();
                self.state.option_next(count);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

// Note: SettingsManager does NOT implement the standard Component trait
// because it requires complex contextual data (Config + Layout) that doesn't
// fit the simple trait signature. Instead, it provides custom methods
// `handle_input_with_context` and `render_with_context`.
//
// This is an intentional design decision: Some components are complex enough
// that they need custom handling, and forcing them into a simple trait would
// be counterproductive.

/// Render the settings manager dialog
pub fn render_settings_manager(
    f: &mut Frame,
    area: Rect,
    state: &SettingsManagerState,
    rgb_enabled: bool,
    rgb_brightness: RgbBrightness,
    rgb_timeout_ms: u32,
    uncolored_key_behavior: UncoloredKeyBehavior,
    idle_effect_settings: &IdleEffectSettings,
    overlay_ripple_settings: &RgbOverlayRippleSettings,
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
                idle_effect_settings,
                overlay_ripple_settings,
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
        ManagerMode::SelectingThemeMode { selected_option } => {
            render_theme_mode_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::EditingPath { setting, value } => {
            render_path_editor(f, inner_area, *setting, value, theme);
        }
        ManagerMode::SelectingIdleEffectMode { selected_option } => {
            render_idle_effect_mode_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::SelectingRippleColorMode { selected_option } => {
            render_ripple_color_mode_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::SelectingKeyPosition {
            setting,
            instruction,
        } => {
            render_key_position_selector(f, inner_area, *setting, instruction, theme);
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
    idle_effect_settings: &IdleEffectSettings,
    overlay_ripple_settings: &RgbOverlayRippleSettings,
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
            idle_effect_settings,
            overlay_ripple_settings,
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
    idle_effect_settings: &IdleEffectSettings,
    overlay_ripple_settings: &RgbOverlayRippleSettings,
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
        SettingItem::ThemeMode => match config.ui.theme_mode {
            crate::config::ThemeMode::Auto => "Auto".to_string(),
            crate::config::ThemeMode::Dark => "Dark".to_string(),
            crate::config::ThemeMode::Light => "Light".to_string(),
        },
        SettingItem::KeyboardScale => format!("{:.0}%", config.ui.keyboard_scale * 100.0),
        // Per-Layout: RGB
        SettingItem::RgbEnabled => if rgb_enabled { "On" } else { "Off" }.to_string(),
        SettingItem::RgbBrightness => format!("{}%", rgb_brightness.as_percent()),
        SettingItem::RgbSaturation => {
            let saturation = layout
                .as_ref()
                .map(|l| l.rgb_saturation.as_percent())
                .unwrap_or(100);
            format!("{}%", saturation)
        }
        SettingItem::RgbMatrixSpeed => {
            let speed = layout
                .as_ref()
                .map(|l| l.rgb_matrix_default_speed)
                .unwrap_or(127);
            format!("{}", speed)
        }
        SettingItem::RgbTimeout => {
            if rgb_timeout_ms == 0 {
                "Disabled".to_string()
            } else if rgb_timeout_ms >= 60000 && rgb_timeout_ms.is_multiple_of(60000) {
                format!("{} min", rgb_timeout_ms / 60000)
            } else if rgb_timeout_ms >= 1000 && rgb_timeout_ms.is_multiple_of(1000) {
                format!("{} sec", rgb_timeout_ms / 1000)
            } else {
                format!("{rgb_timeout_ms}ms")
            }
        }
        SettingItem::IdleEffectEnabled => if idle_effect_settings.enabled {
            "On"
        } else {
            "Off"
        }
        .to_string(),
        SettingItem::IdleTimeout => {
            let idle_timeout_ms = idle_effect_settings.idle_timeout_ms;
            if idle_timeout_ms == 0 {
                "Disabled".to_string()
            } else if idle_timeout_ms >= 60000 && idle_timeout_ms.is_multiple_of(60000) {
                format!("{} min", idle_timeout_ms / 60000)
            } else if idle_timeout_ms >= 1000 && idle_timeout_ms.is_multiple_of(1000) {
                format!("{} sec", idle_timeout_ms / 1000)
            } else {
                format!("{idle_timeout_ms}ms")
            }
        }
        SettingItem::IdleEffectDuration => {
            let duration_ms = idle_effect_settings.idle_effect_duration_ms;
            if duration_ms == 0 {
                "Disabled".to_string()
            } else if duration_ms >= 60000 && duration_ms.is_multiple_of(60000) {
                format!("{} min", duration_ms / 60000)
            } else if duration_ms >= 1000 && duration_ms.is_multiple_of(1000) {
                format!("{} sec", duration_ms / 1000)
            } else {
                format!("{duration_ms}ms")
            }
        }
        SettingItem::IdleEffectMode => idle_effect_settings
            .idle_effect_mode
            .display_name()
            .to_string(),
        SettingItem::UncoloredKeyBehavior => format!("{}%", uncolored_key_behavior.as_percent()),
        // Per-Layout: Overlay Ripple
        SettingItem::OverlayRippleEnabled => if overlay_ripple_settings.enabled {
            "On"
        } else {
            "Off"
        }
        .to_string(),
        SettingItem::OverlayRippleMaxRipples => format!("{}", overlay_ripple_settings.max_ripples),
        SettingItem::OverlayRippleDuration => format!("{}ms", overlay_ripple_settings.duration_ms),
        SettingItem::OverlayRippleSpeed => format!("{}", overlay_ripple_settings.speed),
        SettingItem::OverlayRippleBandWidth => format!("{}", overlay_ripple_settings.band_width),
        SettingItem::OverlayRippleAmplitude => {
            format!("{}%", overlay_ripple_settings.amplitude_pct)
        }
        SettingItem::OverlayRippleColorMode => overlay_ripple_settings
            .color_mode
            .display_name()
            .to_string(),
        SettingItem::OverlayRippleFixedColor => overlay_ripple_settings.fixed_color.to_hex(),
        SettingItem::OverlayRippleHueShift => format!("{}°", overlay_ripple_settings.hue_shift_deg),
        SettingItem::OverlayRippleTriggerPress => if overlay_ripple_settings.trigger_on_press {
            "On"
        } else {
            "Off"
        }
        .to_string(),
        SettingItem::OverlayRippleTriggerRelease => if overlay_ripple_settings.trigger_on_release {
            "On"
        } else {
            "Off"
        }
        .to_string(),
        SettingItem::OverlayRippleIgnoreTransparent => {
            if overlay_ripple_settings.ignore_transparent {
                "On"
            } else {
                "Off"
            }
            .to_string()
        }
        SettingItem::OverlayRippleIgnoreModifiers => if overlay_ripple_settings.ignore_modifiers {
            "On"
        } else {
            "Off"
        }
        .to_string(),
        SettingItem::OverlayRippleIgnoreLayerSwitch => {
            if overlay_ripple_settings.ignore_layer_switch {
                "On"
            } else {
                "Off"
            }
            .to_string()
        }
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
        // Per-Layout: Combo Settings
        SettingItem::CombosEnabled => {
            if let Some(layout) = layout {
                if layout.combo_settings.enabled {
                    "On"
                } else {
                    "Off"
                }
                .to_string()
            } else {
                "Off".to_string()
            }
        }
        SettingItem::Combo1Key1 => layout
            .and_then(|l| l.combo_settings.combos.first())
            .map_or_else(
                || "<not set>".to_string(),
                |c| format!("({}, {})", c.key1.row, c.key1.col),
            ),
        SettingItem::Combo1Key2 => layout
            .and_then(|l| l.combo_settings.combos.first())
            .map_or_else(
                || "<not set>".to_string(),
                |c| format!("({}, {})", c.key2.row, c.key2.col),
            ),
        SettingItem::Combo1HoldDuration => layout
            .and_then(|l| l.combo_settings.combos.first())
            .map_or_else(
                || "500ms".to_string(),
                |c| format!("{}ms", c.hold_duration_ms),
            ),
        SettingItem::Combo2Key1 => layout
            .and_then(|l| l.combo_settings.combos.get(1))
            .map_or_else(
                || "<not set>".to_string(),
                |c| format!("({}, {})", c.key1.row, c.key1.col),
            ),
        SettingItem::Combo2Key2 => layout
            .and_then(|l| l.combo_settings.combos.get(1))
            .map_or_else(
                || "<not set>".to_string(),
                |c| format!("({}, {})", c.key2.row, c.key2.col),
            ),
        SettingItem::Combo2HoldDuration => layout
            .and_then(|l| l.combo_settings.combos.get(1))
            .map_or_else(
                || "500ms".to_string(),
                |c| format!("{}ms", c.hold_duration_ms),
            ),
        SettingItem::Combo3Key1 => layout
            .and_then(|l| l.combo_settings.combos.get(2))
            .map_or_else(
                || "<not set>".to_string(),
                |c| format!("({}, {})", c.key1.row, c.key1.col),
            ),
        SettingItem::Combo3Key2 => layout
            .and_then(|l| l.combo_settings.combos.get(2))
            .map_or_else(
                || "<not set>".to_string(),
                |c| format!("({}, {})", c.key2.row, c.key2.col),
            ),
        SettingItem::Combo3HoldDuration => layout
            .and_then(|l| l.combo_settings.combos.get(2))
            .map_or_else(
                || "500ms".to_string(),
                |c| format!("{}ms", c.hold_duration_ms),
            ),
    }
}

// /// Render inactive behavior selector
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

/// Render theme mode selector
fn render_theme_mode_selector(f: &mut Frame, area: Rect, selected_option: usize, theme: &Theme) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(5),    // Options
            Constraint::Length(4), // Help
        ])
        .split(area);

    // Title
    let title_text = Paragraph::new("Theme Mode")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title_text, chunks[0]);

    // Options
    let options = [
        ("Auto", "Follow OS dark/light mode setting"),
        ("Dark", "Always use dark theme"),
        ("Light", "Always use light theme"),
    ];
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, (label, description))| {
            let selected = idx == selected_option;
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
                Span::styled(" - ", Style::default().fg(theme.text_muted)),
                Span::styled(*description, Style::default().fg(theme.text_muted)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Theme"))
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

/// Render idle effect mode selector
fn render_idle_effect_mode_selector(f: &mut Frame, area: Rect, selected: usize, theme: &Theme) {
    let options = RgbMatrixEffect::all();
    render_enum_selector(
        f,
        area,
        "Idle Effect Mode",
        options
            .iter()
            .map(|o| (o.display_name(), "RGB animation during idle period"))
            .collect::<Vec<_>>()
            .as_slice(),
        selected,
        theme,
    );
}

/// Render ripple color mode selector
fn render_ripple_color_mode_selector(f: &mut Frame, area: Rect, selected: usize, theme: &Theme) {
    let options = RippleColorMode::all();
    render_enum_selector(
        f,
        area,
        "Ripple Color Mode",
        options
            .iter()
            .map(|o| (o.display_name(), o.description()))
            .collect::<Vec<_>>()
            .as_slice(),
        selected,
        theme,
    );
}

/// Render key position selector instruction
fn render_key_position_selector(
    f: &mut Frame,
    area: Rect,
    setting: SettingItem,
    instruction: &str,
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(3),    // Instructions
            Constraint::Length(4), // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled("Select Key Position: ", Style::default().fg(theme.primary)),
        Span::styled(setting.display_name(), Style::default().fg(theme.accent)),
    ])])
    .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            instruction,
            Style::default().fg(theme.text),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Use arrow keys to navigate the keyboard below",
            Style::default().fg(theme.text_muted),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Instructions")
            .style(Style::default().bg(theme.background)),
    );
    f.render_widget(instructions, chunks[1]);

    // Help
    let help = Paragraph::new(vec![Line::from(vec![
        Span::styled("Arrow Keys", Style::default().fg(theme.primary)),
        Span::raw(": Navigate  "),
        Span::styled("Enter", Style::default().fg(theme.primary)),
        Span::raw(": Select Key  "),
        Span::styled("Esc", Style::default().fg(theme.primary)),
        Span::raw(": Cancel"),
    ])])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Controls")
            .style(Style::default().bg(theme.background)),
    );
    f.render_widget(help, chunks[2]);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_item_all_includes_idle_effect_settings() {
        let all_settings = SettingItem::all();

        // Verify idle effect settings are present
        assert!(all_settings.contains(&SettingItem::IdleEffectEnabled));
        assert!(all_settings.contains(&SettingItem::IdleTimeout));
        assert!(all_settings.contains(&SettingItem::IdleEffectDuration));
        assert!(all_settings.contains(&SettingItem::IdleEffectMode));
    }

    #[test]
    fn test_idle_effect_settings_belong_to_rgb_group() {
        assert_eq!(SettingItem::IdleEffectEnabled.group(), SettingGroup::Rgb);
        assert_eq!(SettingItem::IdleTimeout.group(), SettingGroup::Rgb);
        assert_eq!(SettingItem::IdleEffectDuration.group(), SettingGroup::Rgb);
        assert_eq!(SettingItem::IdleEffectMode.group(), SettingGroup::Rgb);
    }

    #[test]
    fn test_idle_effect_settings_have_display_names() {
        assert_eq!(
            SettingItem::IdleEffectEnabled.display_name(),
            "Idle Effect Enabled"
        );
        assert_eq!(SettingItem::IdleTimeout.display_name(), "Idle Timeout");
        assert_eq!(
            SettingItem::IdleEffectDuration.display_name(),
            "Idle Effect Duration"
        );
        assert_eq!(
            SettingItem::IdleEffectMode.display_name(),
            "Idle Effect Mode"
        );
    }

    #[test]
    fn test_idle_effect_settings_have_descriptions() {
        let desc = SettingItem::IdleEffectEnabled.description();
        assert!(!desc.is_empty());
        assert!(desc.contains("idle"));

        let desc = SettingItem::IdleTimeout.description();
        assert!(!desc.is_empty());

        let desc = SettingItem::IdleEffectDuration.description();
        assert!(!desc.is_empty());

        let desc = SettingItem::IdleEffectMode.description();
        assert!(!desc.is_empty());
    }

    #[test]
    fn test_get_setting_value_display_idle_effect_enabled() {
        let idle_settings = IdleEffectSettings {
            enabled: true,
            ..Default::default()
        };

        let display = get_setting_value_display(
            SettingItem::IdleEffectEnabled,
            true,
            RgbBrightness::from(100),
            0,
            UncoloredKeyBehavior::from(100),
            &idle_settings,
            &RgbOverlayRippleSettings::default(),
            &TapHoldSettings::default(),
            &crate::config::Config::default(),
            None,
        );

        assert_eq!(display, "On");

        let idle_settings = IdleEffectSettings {
            enabled: false,
            ..Default::default()
        };

        let display = get_setting_value_display(
            SettingItem::IdleEffectEnabled,
            true,
            RgbBrightness::from(100),
            0,
            UncoloredKeyBehavior::from(100),
            &idle_settings,
            &RgbOverlayRippleSettings::default(),
            &TapHoldSettings::default(),
            &crate::config::Config::default(),
            None,
        );

        assert_eq!(display, "Off");
    }

    #[test]
    fn test_get_setting_value_display_idle_timeout() {
        let idle_settings = IdleEffectSettings {
            idle_timeout_ms: 0,
            ..Default::default()
        };

        let display = get_setting_value_display(
            SettingItem::IdleTimeout,
            true,
            RgbBrightness::from(100),
            0,
            UncoloredKeyBehavior::from(100),
            &idle_settings,
            &RgbOverlayRippleSettings::default(),
            &TapHoldSettings::default(),
            &crate::config::Config::default(),
            None,
        );

        assert_eq!(display, "Disabled");

        let idle_settings = IdleEffectSettings {
            idle_timeout_ms: 60_000,
            ..Default::default()
        };

        let display = get_setting_value_display(
            SettingItem::IdleTimeout,
            true,
            RgbBrightness::from(100),
            0,
            UncoloredKeyBehavior::from(100),
            &idle_settings,
            &RgbOverlayRippleSettings::default(),
            &TapHoldSettings::default(),
            &crate::config::Config::default(),
            None,
        );

        assert_eq!(display, "1 min");

        let idle_settings = IdleEffectSettings {
            idle_timeout_ms: 30_000,
            ..Default::default()
        };

        let display = get_setting_value_display(
            SettingItem::IdleTimeout,
            true,
            RgbBrightness::from(100),
            0,
            UncoloredKeyBehavior::from(100),
            &idle_settings,
            &RgbOverlayRippleSettings::default(),
            &TapHoldSettings::default(),
            &crate::config::Config::default(),
            None,
        );

        assert_eq!(display, "30 sec");
    }

    #[test]
    fn test_get_setting_value_display_idle_effect_mode() {
        let idle_settings = IdleEffectSettings {
            idle_effect_mode: RgbMatrixEffect::Breathing,
            ..Default::default()
        };

        let display = get_setting_value_display(
            SettingItem::IdleEffectMode,
            true,
            RgbBrightness::from(100),
            0,
            UncoloredKeyBehavior::from(100),
            &idle_settings,
            &RgbOverlayRippleSettings::default(),
            &TapHoldSettings::default(),
            &crate::config::Config::default(),
            None,
        );

        assert_eq!(display, "Breathing");

        let idle_settings = IdleEffectSettings {
            idle_effect_mode: RgbMatrixEffect::RainbowBeacon,
            ..Default::default()
        };

        let display = get_setting_value_display(
            SettingItem::IdleEffectMode,
            true,
            RgbBrightness::from(100),
            0,
            UncoloredKeyBehavior::from(100),
            &idle_settings,
            &RgbOverlayRippleSettings::default(),
            &TapHoldSettings::default(),
            &crate::config::Config::default(),
            None,
        );

        assert_eq!(display, "Rainbow Beacon");
    }
}
