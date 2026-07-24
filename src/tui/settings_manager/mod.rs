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
    /// Add a new combo entry to the layout
    AddCombo,
    /// Remove the combo at the given index (1-based display)
    RemoveCombo(usize),
    /// First key position for the combo at the given index
    ComboKey1(usize),
    /// Second key position for the combo at the given index
    ComboKey2(usize),
    /// Hold duration in milliseconds for the combo at the given index
    ComboHoldDuration(usize),
    /// Action performed by the combo at the given index
    ComboAction(usize),
}

impl SettingItem {
    /// Returns all settings in a single flat list, including dynamic combo entries
    /// derived from the provided layout.
    #[must_use]
    pub fn all(layout: &crate::models::Layout) -> Vec<Self> {
        let mut items = vec![
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
            Self::AddCombo,
        ];

        // Append per-combo entries (one block per defined combo)
        for idx in 0..layout.combo_settings.combos.len() {
            items.push(Self::ComboKey1(idx));
            items.push(Self::ComboKey2(idx));
            items.push(Self::ComboAction(idx));
            items.push(Self::ComboHoldDuration(idx));
            items.push(Self::RemoveCombo(idx));
        }

        items
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
            | Self::AddCombo
            | Self::RemoveCombo(_)
            | Self::ComboKey1(_)
            | Self::ComboKey2(_)
            | Self::ComboHoldDuration(_)
            | Self::ComboAction(_) => SettingGroup::Combos,
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
    pub fn display_name(&self) -> String {
        match *self {
            Self::QmkFirmwarePath => "QMK Firmware Folder".to_string(),
            Self::Keyboard => "Keyboard".to_string(),
            Self::LayoutVariant => "Layout Variant".to_string(),
            Self::KeymapName => "Keymap Name".to_string(),
            Self::OutputFormat => "Output Format".to_string(),
            Self::OutputDir => "Build Output Folder".to_string(),
            Self::ShowHelpOnStartup => "Show Help on Startup".to_string(),
            Self::ThemeMode => "Theme Mode".to_string(),
            Self::KeyboardScale => "Keyboard Scale".to_string(),
            Self::RgbEnabled => "Lighting Enabled".to_string(),
            Self::RgbBrightness => "Lighting Brightness".to_string(),
            Self::RgbSaturation => "RGB Saturation".to_string(),
            Self::RgbMatrixSpeed => "RGB Matrix Speed".to_string(),
            Self::RgbTimeout => "Lighting Timeout".to_string(),
            Self::IdleEffectEnabled => "Idle Lighting Enabled".to_string(),
            Self::IdleTimeout => "Idle Wait Time".to_string(),
            Self::IdleEffectDuration => "Idle Effect Length".to_string(),
            Self::IdleEffectMode => "Idle Effect".to_string(),
            Self::UncoloredKeyBehavior => "Uncolored Key Brightness".to_string(),
            Self::OverlayRippleEnabled => "Press Ripple Enabled".to_string(),
            Self::OverlayRippleMaxRipples => "Max Concurrent Ripples".to_string(),
            Self::OverlayRippleDuration => "Ripple Duration".to_string(),
            Self::OverlayRippleSpeed => "Ripple Speed".to_string(),
            Self::OverlayRippleBandWidth => "Ripple Band Width".to_string(),
            Self::OverlayRippleAmplitude => "Ripple Amplitude".to_string(),
            Self::OverlayRippleColorMode => "Ripple Color Mode".to_string(),
            Self::OverlayRippleFixedColor => "Ripple Fixed Color".to_string(),
            Self::OverlayRippleHueShift => "Ripple Hue Shift".to_string(),
            Self::OverlayRippleTriggerPress => "Trigger on Press".to_string(),
            Self::OverlayRippleTriggerRelease => "Trigger on Release".to_string(),
            Self::OverlayRippleIgnoreTransparent => "Ignore Transparent Keys".to_string(),
            Self::OverlayRippleIgnoreModifiers => "Ignore Modifier Keys".to_string(),
            Self::OverlayRippleIgnoreLayerSwitch => "Ignore Layer Switch Keys".to_string(),
            Self::OverlayRippleKeyActionPalette => "Reactive Key Palette".to_string(),
            Self::OverlayRippleWaveCount => "Ripple Waves per Key".to_string(),
            Self::OverlayRippleWaveDelay => "Delay Between Waves".to_string(),
            Self::PaletteFxEnabled => "PaletteFX Effects".to_string(),
            Self::PaletteFxDefaultEffect => "PaletteFX Default Effect".to_string(),
            Self::PaletteFxDefaultPalette => "PaletteFX Default Palette".to_string(),
            Self::PaletteFxEnableAllEffects => "PaletteFX All Effects".to_string(),
            Self::PaletteFxEnableAllPalettes => "PaletteFX All Palettes".to_string(),
            Self::TapHoldPreset => "Preset".to_string(),
            Self::TappingTerm => "Tapping Term".to_string(),
            Self::QuickTapTerm => "Quick Tap Term".to_string(),
            Self::HoldMode => "Hold Mode".to_string(),
            Self::RetroTapping => "Retro Tapping".to_string(),
            Self::TappingToggle => "Tapping Toggle".to_string(),
            Self::FlowTapTerm => "Flow Tap Term".to_string(),
            Self::ChordalHold => "Chordal Hold".to_string(),
            // Combo Settings
            Self::CombosEnabled => "Combos Enabled".to_string(),
            Self::AddCombo => "Add Combo".to_string(),
            Self::RemoveCombo(idx) => format!("Remove Combo {}", idx + 1),
            Self::ComboKey1(idx) => format!("Combo {} Key 1", idx + 1),
            Self::ComboKey2(idx) => format!("Combo {} Key 2", idx + 1),
            Self::ComboHoldDuration(idx) => format!("Combo {} Hold Duration", idx + 1),
            Self::ComboAction(idx) => format!("Combo {} Action", idx + 1),
        }
    }

    /// Returns a description of this setting.
    #[must_use]
    pub fn description(&self) -> String {
        match *self {
            Self::QmkFirmwarePath => {
                "Folder containing your local QMK checkout. Needed for keyboard info and builds."
                    .to_string()
            }
            Self::Keyboard => {
                "Keyboard this layout targets when you generate or build firmware.".to_string()
            }
            Self::LayoutVariant => {
                "Physical layout variant used by this keyboard, such as LAYOUT_split_3x6_3."
                    .to_string()
            }
            Self::KeymapName => {
                "Name used for generated keymap files, such as default or mymap.".to_string()
            }
            Self::OutputFormat => {
                "Firmware file type to export after build, such as uf2, hex, or bin.".to_string()
            }
            Self::OutputDir => "Folder where built firmware files should be written.".to_string(),
            Self::ShowHelpOnStartup => "Display help overlay when application starts".to_string(),
            Self::ThemeMode => "Color theme: Auto (follow OS), Dark, or Light".to_string(),
            Self::KeyboardScale => {
                "Keyboard display size: 1.0 = default, 0.5 = half, 2.0 = double".to_string()
            }
            Self::RgbEnabled => "Turn all keyboard lighting on or off.".to_string(),
            Self::RgbBrightness => "Overall keyboard lighting brightness (0-100%).".to_string(),
            Self::RgbSaturation => {
                "Saturation multiplier for all LEDs (0=Grayscale, 100=Normal, 200=Maximum)"
                    .to_string()
            }
            Self::RgbMatrixSpeed => {
                "Animation speed for RGB effects (0=Slowest, 127=Default, 255=Fastest)".to_string()
            }
            Self::RgbTimeout => "Turn lighting off after inactivity. Use 0 to keep it on.".to_string(),
            Self::IdleEffectEnabled => {
                "Play a temporary lighting effect before full idle timeout.".to_string()
            }
            Self::IdleTimeout => {
                "How long to wait before idle lighting begins. Use 0 to disable.".to_string()
            }
            Self::IdleEffectDuration => {
                "How long idle lighting runs before lights turn off. Use 0 for immediate off."
                    .to_string()
            }
            Self::IdleEffectMode => "Lighting animation used while keyboard is idle.".to_string(),
            Self::UncoloredKeyBehavior => {
                "Brightness for keys without individual/category colors (0=Off, 100=Full)"
                    .to_string()
            }
            Self::OverlayRippleEnabled => "Show ripple feedback on key press and/or release.".to_string(),
            Self::OverlayRippleMaxRipples => "Maximum number of concurrent ripples (1-8)".to_string(),
            Self::OverlayRippleDuration => "How long each ripple lasts in milliseconds".to_string(),
            Self::OverlayRippleSpeed => {
                "Expansion speed in physical LED coordinate space (1-255, higher = faster)"
                    .to_string()
            }
            Self::OverlayRippleBandWidth => "Width of ripple band in physical distance units".to_string(),
            Self::OverlayRippleAmplitude => "Brightness boost as percentage of base (0-100%)".to_string(),
            Self::OverlayRippleColorMode => {
                "How to determine ripple colors (Fixed, Key Color, Hue Shift)".to_string()
            }
            Self::OverlayRippleFixedColor => "Color to use when color mode is Fixed Color".to_string(),
            Self::OverlayRippleHueShift => {
                "Hue shift in degrees when mode is Hue Shift (-180 to 180)".to_string()
            }
            Self::OverlayRippleTriggerPress => "Trigger ripple effect on key press events".to_string(),
            Self::OverlayRippleTriggerRelease => "Trigger ripple effect on key release events".to_string(),
            Self::OverlayRippleIgnoreTransparent => {
                "Don't trigger ripples on transparent keys (KC_TRNS)".to_string()
            }
            Self::OverlayRippleIgnoreModifiers => "Don't trigger ripples on modifier keys".to_string(),
            Self::OverlayRippleIgnoreLayerSwitch => {
                "Don't trigger ripples on layer switch keys".to_string()
            }
            Self::OverlayRippleKeyActionPalette => {
                "PaletteFX palette for key-action reactive bursts. Requires PaletteFX enabled."
                    .to_string()
            }
            Self::OverlayRippleWaveCount => {
                "Number of concentric waves per keypress (1-5). Higher = richer cascading effect."
                    .to_string()
            }
            Self::OverlayRippleWaveDelay => {
                "Delay between consecutive waves in milliseconds (50-500ms).".to_string()
            }
            Self::PaletteFxEnabled => {
                "Enable PaletteFX community module effects instead of custom ripple overlay."
                    .to_string()
            }
            Self::PaletteFxDefaultEffect => {
                "Default PaletteFX effect shown on startup. User can cycle with RM_NEXT/RM_PREV."
                    .to_string()
            }
            Self::PaletteFxDefaultPalette => {
                "Color palette for PaletteFX effects (16 curated palettes available).".to_string()
            }
            Self::PaletteFxEnableAllEffects => {
                "Include all 6 PaletteFX effects in firmware (Gradient, Flow, Ripple, Sparkle, Vortex, Reactive)."
                    .to_string()
            }
            Self::PaletteFxEnableAllPalettes => {
                "Include all 16 PaletteFX palettes in firmware.".to_string()
            }
            Self::TapHoldPreset => "Quick configuration preset for common use cases".to_string(),
            Self::TappingTerm => "Milliseconds to distinguish tap from hold (100-500ms)".to_string(),
            Self::QuickTapTerm => "Window for tap-then-hold to trigger auto-repeat".to_string(),
            Self::HoldMode => "How to decide tap vs hold when other keys are pressed".to_string(),
            Self::RetroTapping => "Send tap keycode even if held past tapping term".to_string(),
            Self::TappingToggle => "Number of taps to toggle layer with TT() keys (1-10)".to_string(),
            Self::FlowTapTerm => "Rapid typing window to prevent accidental modifiers".to_string(),
            Self::ChordalHold => "Use opposite-hand rule for tap-hold (great for HRM)".to_string(),
            // Combo Settings
            Self::CombosEnabled => {
                "Master switch for combo feature (two-key hold actions)".to_string()
            }
            Self::AddCombo => {
                "Add a new combo entry (up to 32 combos supported)".to_string()
            }
            Self::RemoveCombo(idx) => format!("Remove Combo {} from the layout", idx + 1),
            Self::ComboKey1(idx) => format!("First key position for Combo {}", idx + 1),
            Self::ComboKey2(idx) => format!("Second key position for Combo {}", idx + 1),
            Self::ComboHoldDuration(idx) => {
                format!("Hold duration in milliseconds for Combo {} (50-2000ms)", idx + 1)
            }
            Self::ComboAction(idx) => format!("Action performed by Combo {}", idx + 1),
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
    /// Selecting the action for a combo entry
    SelectingAction {
        /// Index of the combo entry whose action is being configured
        idx: usize,
        /// Currently selected action
        current: crate::models::ComboAction,
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
            ManagerMode::SelectingAction { .. } => self.handle_action_selection(key),
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
