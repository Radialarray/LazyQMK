//! Settings manager for global layout settings.
//!
//! Provides a UI for configuring layout-wide settings like inactive key behavior
//! and tap-hold timing configuration.
//! Accessible via Shift+S shortcut.

use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::models::{
    IdleEffectSettings, RgbBrightness, RgbOverlayRippleSettings, TapHoldSettings,
    UncoloredKeyBehavior,
};

use super::Theme;

mod input;
mod render_editor;
mod render_main;
mod render_selector;
mod state;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RgbSubgroup {
    Core,
    Idle,
    Ripple,
    PaletteFx,
}

impl RgbSubgroup {
    const fn display_name(self) -> &'static str {
        match self {
            Self::Core => "Core lighting",
            Self::Idle => "Idle lighting",
            Self::Ripple => "Press ripple",
            Self::PaletteFx => "PaletteFX effects",
        }
    }
}

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
            Self::Paths => "Setup & folders",
            Self::Build => "Keyboard & build output",
            Self::Ui => "Editor behavior",
            Self::Rgb => "Lighting behavior",
            Self::TapHold => "Tap-hold tuning",
            Self::Combos => "Combos & quick actions",
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
    /// Ripple color mode (Fixed, `KeyBased`, `HueShift`)
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
    /// `PaletteFX` palette for key-action reactive bursts
    OverlayRippleKeyActionPalette,
    /// Number of waves per keypress (1-5)
    OverlayRippleWaveCount,
    /// Delay between waves in milliseconds (50-500)
    OverlayRippleWaveDelay,

    // === PaletteFX Settings (Per-Layout) ===
    /// `PaletteFX` master switch
    PaletteFxEnabled,
    /// `PaletteFX` default effect
    PaletteFxDefaultEffect,
    /// `PaletteFX` default palette
    PaletteFxDefaultPalette,
    /// Enable all `PaletteFX` effects at compile time
    PaletteFxEnableAllEffects,
    /// Enable all `PaletteFX` palettes at compile time
    PaletteFxEnableAllPalettes,

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
            Self::OverlayRippleKeyActionPalette,
            Self::OverlayRippleWaveCount,
            Self::OverlayRippleWaveDelay,
            // PaletteFX (Per-Layout)
            Self::PaletteFxEnabled,
            Self::PaletteFxDefaultEffect,
            Self::PaletteFxDefaultPalette,
            Self::PaletteFxEnableAllEffects,
            Self::PaletteFxEnableAllPalettes,
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
            | Self::OverlayRippleIgnoreLayerSwitch
            | Self::OverlayRippleKeyActionPalette
            | Self::OverlayRippleWaveCount
            | Self::OverlayRippleWaveDelay
            | Self::PaletteFxEnabled
            | Self::PaletteFxDefaultEffect
            | Self::PaletteFxDefaultPalette
            | Self::PaletteFxEnableAllEffects
            | Self::PaletteFxEnableAllPalettes => SettingGroup::Rgb,
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

    #[must_use]
    const fn rgb_subgroup(self) -> Option<RgbSubgroup> {
        match self {
            Self::RgbEnabled
            | Self::RgbBrightness
            | Self::RgbSaturation
            | Self::RgbMatrixSpeed
            | Self::RgbTimeout
            | Self::UncoloredKeyBehavior => Some(RgbSubgroup::Core),
            Self::IdleEffectEnabled
            | Self::IdleTimeout
            | Self::IdleEffectDuration
            | Self::IdleEffectMode => Some(RgbSubgroup::Idle),
            Self::OverlayRippleEnabled
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
            | Self::OverlayRippleIgnoreLayerSwitch
            | Self::OverlayRippleKeyActionPalette
            | Self::OverlayRippleWaveCount
            | Self::OverlayRippleWaveDelay => Some(RgbSubgroup::Ripple),
            Self::PaletteFxEnabled
            | Self::PaletteFxDefaultEffect
            | Self::PaletteFxDefaultPalette
            | Self::PaletteFxEnableAllEffects
            | Self::PaletteFxEnableAllPalettes => Some(RgbSubgroup::PaletteFx),
            _ => None,
        }
    }

    /// Returns a human-readable name for this setting.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::QmkFirmwarePath => "QMK Firmware Folder",
            Self::Keyboard => "Keyboard",
            Self::LayoutVariant => "Layout Variant",
            Self::KeymapName => "Keymap Name",
            Self::OutputFormat => "Output Format",
            Self::OutputDir => "Build Output Folder",
            Self::ShowHelpOnStartup => "Show Help on Startup",
            Self::ThemeMode => "Theme Mode",
            Self::KeyboardScale => "Keyboard Scale",
            Self::RgbEnabled => "Lighting Enabled",
            Self::RgbBrightness => "Lighting Brightness",
            Self::RgbSaturation => "RGB Saturation",
            Self::RgbMatrixSpeed => "RGB Matrix Speed",
            Self::RgbTimeout => "Lighting Timeout",
            Self::IdleEffectEnabled => "Idle Lighting Enabled",
            Self::IdleTimeout => "Idle Wait Time",
            Self::IdleEffectDuration => "Idle Effect Length",
            Self::IdleEffectMode => "Idle Effect",
            Self::UncoloredKeyBehavior => "Uncolored Key Brightness",
            Self::OverlayRippleEnabled => "Press Ripple Enabled",
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
            Self::OverlayRippleKeyActionPalette => "Reactive Key Palette",
            Self::OverlayRippleWaveCount => "Ripple Waves per Key",
            Self::OverlayRippleWaveDelay => "Delay Between Waves",
            Self::PaletteFxEnabled => "PaletteFX Effects",
            Self::PaletteFxDefaultEffect => "PaletteFX Default Effect",
            Self::PaletteFxDefaultPalette => "PaletteFX Default Palette",
            Self::PaletteFxEnableAllEffects => "PaletteFX All Effects",
            Self::PaletteFxEnableAllPalettes => "PaletteFX All Palettes",
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
            Self::QmkFirmwarePath => {
                "Folder containing your local QMK checkout. Needed for keyboard info and builds."
            }
            Self::Keyboard => "Keyboard this layout targets when you generate or build firmware.",
            Self::LayoutVariant => {
                "Physical layout variant used by this keyboard, such as LAYOUT_split_3x6_3."
            }
            Self::KeymapName => "Name used for generated keymap files, such as default or mymap.",
            Self::OutputFormat => {
                "Firmware file type to export after build, such as uf2, hex, or bin."
            }
            Self::OutputDir => "Folder where built firmware files should be written.",
            Self::ShowHelpOnStartup => "Display help overlay when application starts",
            Self::ThemeMode => "Color theme: Auto (follow OS), Dark, or Light",
            Self::KeyboardScale => "Keyboard display size: 1.0 = default, 0.5 = half, 2.0 = double",
            Self::RgbEnabled => "Turn all keyboard lighting on or off.",
            Self::RgbBrightness => "Overall keyboard lighting brightness (0-100%).",
            Self::RgbSaturation => {
                "Saturation multiplier for all LEDs (0=Grayscale, 100=Normal, 200=Maximum)"
            }
            Self::RgbMatrixSpeed => {
                "Animation speed for RGB effects (0=Slowest, 127=Default, 255=Fastest)"
            }
            Self::RgbTimeout => "Turn lighting off after inactivity. Use 0 to keep it on.",
            Self::IdleEffectEnabled => "Play a temporary lighting effect before full idle timeout.",
            Self::IdleTimeout => "How long to wait before idle lighting begins. Use 0 to disable.",
            Self::IdleEffectDuration => {
                "How long idle lighting runs before lights turn off. Use 0 for immediate off."
            }
            Self::IdleEffectMode => "Lighting animation used while keyboard is idle.",
            Self::UncoloredKeyBehavior => {
                "Brightness for keys without individual/category colors (0=Off, 100=Full)"
            }
            Self::OverlayRippleEnabled => "Show ripple feedback on key press and/or release.",
            Self::OverlayRippleMaxRipples => "Maximum number of concurrent ripples (1-8)",
            Self::OverlayRippleDuration => "How long each ripple lasts in milliseconds",
            Self::OverlayRippleSpeed => {
                "Expansion speed in physical LED coordinate space (1-255, higher = faster)"
            }
            Self::OverlayRippleBandWidth => "Width of ripple band in physical distance units",
            Self::OverlayRippleAmplitude => "Brightness boost as percentage of base (0-100%)",
            Self::OverlayRippleColorMode => {
                "How to determine ripple colors (Fixed, Key Color, Hue Shift)"
            }
            Self::OverlayRippleFixedColor => "Color to use when color mode is Fixed Color",
            Self::OverlayRippleHueShift => {
                "Hue shift in degrees when mode is Hue Shift (-180 to 180)"
            }
            Self::OverlayRippleTriggerPress => "Trigger ripple effect on key press events",
            Self::OverlayRippleTriggerRelease => "Trigger ripple effect on key release events",
            Self::OverlayRippleIgnoreTransparent => {
                "Don't trigger ripples on transparent keys (KC_TRNS)"
            }
            Self::OverlayRippleIgnoreModifiers => "Don't trigger ripples on modifier keys",
            Self::OverlayRippleIgnoreLayerSwitch => "Don't trigger ripples on layer switch keys",
            Self::OverlayRippleKeyActionPalette => "PaletteFX palette for key-action reactive bursts. Requires PaletteFX enabled.",
            Self::OverlayRippleWaveCount => "Number of concentric waves per keypress (1-5). Higher = richer cascading effect.",
            Self::OverlayRippleWaveDelay => "Delay between consecutive waves in milliseconds (50-500ms).",
            Self::PaletteFxEnabled => "Enable PaletteFX community module effects instead of custom ripple overlay.",
            Self::PaletteFxDefaultEffect => "Default PaletteFX effect shown on startup. User can cycle with RM_NEXT/RM_PREV.",
            Self::PaletteFxDefaultPalette => "Color palette for PaletteFX effects (16 curated palettes available).",
            Self::PaletteFxEnableAllEffects => "Include all 6 PaletteFX effects in firmware (Gradient, Flow, Ripple, Sparkle, Vortex, Reactive).",
            Self::PaletteFxEnableAllPalettes => "Include all 16 PaletteFX palettes in firmware.",
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
        /// Suggested default value
        default: u16,
    },
    /// Editing a signed numeric value
    EditingSignedNumeric {
        /// Which setting is being edited
        setting: SettingItem,
        /// Current value as string for editing
        value: String,
        /// Minimum allowed value
        min: i16,
        /// Maximum allowed value
        max: i16,
        /// Suggested default value
        default: i16,
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
    /// Selecting `PaletteFX` effect
    SelectingPaletteFxEffect {
        /// Currently highlighted option index
        selected_option: usize,
    },
    /// Selecting `PaletteFX` palette
    SelectingPaletteFxPalette {
        /// Currently highlighted option index
        selected_option: usize,
    },
    /// Selecting key-action reactive palette
    SelectingKeyActionPalette {
        /// Currently highlighted option index
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

/// Events emitted by the `SettingsManager` component
#[derive(Debug, Clone)]
pub enum SettingsManagerEvent {
    /// Settings were updated (settings applied and dialog should close)
    SettingsUpdated,
    /// User cancelled without making changes
    Cancelled,
    /// Component closed naturally
    #[allow(dead_code)] // bin/lib split: variant in SettingsManagerEvent (handlers use it)
    Closed,
}

/// Context data needed for `SettingsManager` to render and handle input
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

/// `SettingsManager` component that implements the Component trait
///
/// This wraps `SettingsManagerState` to provide a self-contained component
/// that handles its own input and rendering. Due to the complexity of settings
/// (needing access to both config and layout data), this component needs context
/// passed through render and `handle_input` methods.
#[derive(Debug, Clone)]
pub struct SettingsManager {
    /// Internal state
    state: SettingsManagerState,
}

impl SettingsManager {
    /// Create a new `SettingsManager`
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
            ManagerMode::EditingSignedNumeric { .. } => self.handle_signed_numeric_editing(key),
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
            ManagerMode::SelectingPaletteFxEffect { .. } => {
                self.handle_palette_fx_effect_selection(key)
            }
            ManagerMode::SelectingPaletteFxPalette { .. } => {
                self.handle_palette_fx_palette_selection(key)
            }
            ManagerMode::SelectingKeyActionPalette { .. } => {
                self.handle_key_action_palette_selection(key)
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
        render_main::render_settings_manager(
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
