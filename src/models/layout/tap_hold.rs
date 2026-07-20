//! Tap-hold settings — how QMK distinguishes tap from hold.

use serde::{Deserialize, Serialize};

/// How QMK decides between tap and hold when other keys are involved.
///
/// These modes affect what happens when you press a tap-hold key and then
/// press another key before releasing the first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HoldDecisionMode {
    /// Only timing-based: hold if key held longer than `tapping_term`.
    /// Most conservative, requires deliberate holds.
    #[default]
    Default,

    /// Hold when another key is tapped (pressed and released) during hold.
    /// Pattern: A↓ B↓ B↑ A↑ (nested) triggers hold.
    /// Good for home-row mods with fast typing.
    PermissiveHold,

    /// Hold when another key is pressed during hold.
    /// Pattern: A↓ B↓ triggers hold immediately.
    /// Most aggressive, fastest hold detection.
    HoldOnOtherKeyPress,
}

impl HoldDecisionMode {
    /// Returns all available modes.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Default,
            Self::PermissiveHold,
            Self::HoldOnOtherKeyPress,
        ]
    }

    /// Returns a human-readable name for this mode.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Default => "Default (Timing Only)",
            Self::PermissiveHold => "Permissive Hold",
            Self::HoldOnOtherKeyPress => "Hold On Other Key",
        }
    }

    /// Returns a description of this mode.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Default => "Hold only when held past tapping term",
            Self::PermissiveHold => "Hold when another key is tapped during hold",
            Self::HoldOnOtherKeyPress => "Hold when another key is pressed during hold",
        }
    }

    /// Returns the QMK #define name for this mode (if not default).
    #[must_use]
    pub const fn config_define(&self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::PermissiveHold => Some("PERMISSIVE_HOLD"),
            Self::HoldOnOtherKeyPress => Some("HOLD_ON_OTHER_KEY_PRESS"),
        }
    }
}

/// Preset configurations for tap-hold behavior.
///
/// These provide sensible defaults for common use cases while allowing
/// full customization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TapHoldPreset {
    /// QMK defaults - conservative timing, suitable for beginners
    #[default]
    Default,

    /// Optimized for home row mods - lower tapping term, flow tap, chordal hold
    HomeRowMods,

    /// Very responsive - quick hold detection for gaming or fast modifier access
    Responsive,

    /// Deliberate - higher tapping term, requires intentional holds
    Deliberate,

    /// User has customized values
    Custom,
}

impl TapHoldPreset {
    /// Returns all available presets.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Default,
            Self::HomeRowMods,
            Self::Responsive,
            Self::Deliberate,
            Self::Custom,
        ]
    }

    /// Returns a human-readable name for this preset.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::HomeRowMods => "Home Row Mods",
            Self::Responsive => "Responsive",
            Self::Deliberate => "Deliberate",
            Self::Custom => "Custom",
        }
    }

    /// Returns a description of this preset.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Default => "QMK defaults, conservative timing",
            Self::HomeRowMods => "Optimized for home row modifiers",
            Self::Responsive => "Fast hold detection for gaming",
            Self::Deliberate => "Requires intentional holds",
            Self::Custom => "User-defined settings",
        }
    }

    /// Returns the settings for this preset.
    #[must_use]
    pub fn settings(&self) -> TapHoldSettings {
        match self {
            Self::Default | Self::Custom => TapHoldSettings::default(),
            Self::HomeRowMods => TapHoldSettings {
                tapping_term: 175,
                quick_tap_term: Some(120),
                hold_mode: HoldDecisionMode::PermissiveHold,
                retro_tapping: true,
                tapping_toggle: 5,
                flow_tap_term: Some(150),
                chordal_hold: true,
                preset: Self::HomeRowMods,
            },
            Self::Responsive => TapHoldSettings {
                tapping_term: 150,
                quick_tap_term: Some(100),
                hold_mode: HoldDecisionMode::HoldOnOtherKeyPress,
                retro_tapping: false,
                tapping_toggle: 5,
                flow_tap_term: None,
                chordal_hold: false,
                preset: Self::Responsive,
            },
            Self::Deliberate => TapHoldSettings {
                tapping_term: 250,
                quick_tap_term: None,
                hold_mode: HoldDecisionMode::Default,
                retro_tapping: false,
                tapping_toggle: 5,
                flow_tap_term: None,
                chordal_hold: false,
                preset: Self::Deliberate,
            },
        }
    }
}

/// Configuration for tap-hold behavior (LT, MT, TT, etc.).
///
/// These settings control how QMK distinguishes between tap and hold actions
/// for dual-function keys like Layer-Tap and Mod-Tap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TapHoldSettings {
    // === Core Timing ===
    /// Time in milliseconds to distinguish tap from hold.
    /// If a key is held longer than this, it's a hold; otherwise it's a tap.
    /// Range: 100-500ms, Default: 200ms
    pub tapping_term: u16,

    /// Time window for tap-then-hold to trigger auto-repeat instead of hold.
    /// If you tap and then hold within this time, it repeats the tap action.
    /// None = same as `tapping_term` (QMK default behavior).
    /// Range: 0-500ms
    pub quick_tap_term: Option<u16>,

    // === Decision Mode ===
    /// How to decide between tap and hold when other keys are involved.
    pub hold_mode: HoldDecisionMode,

    // === Special Behaviors ===
    /// Send tap keycode even if held past tapping term (when no other key pressed).
    /// Useful for home-row mods to avoid accidental modifiers.
    pub retro_tapping: bool,

    /// Number of taps required to toggle layer with `TT()` keys.
    /// Range: 1-10, Default: 5
    pub tapping_toggle: u8,

    // === Home Row Mod Optimizations ===
    /// Flow Tap: time window where rapid typing triggers tap, not hold.
    /// Helps prevent accidental modifiers during fast typing.
    /// None = disabled, Some(ms) = enabled with specified term.
    /// Range: 0-300ms
    pub flow_tap_term: Option<u16>,

    /// Chordal Hold: use opposite-hand rule for tap-hold decision.
    /// Same-hand key presses favor tap, opposite-hand presses favor hold.
    /// Excellent for home-row mods.
    pub chordal_hold: bool,

    // === Preset Tracking ===
    /// Which preset these settings are based on (Custom if modified).
    #[serde(default)]
    pub preset: TapHoldPreset,
}

impl Default for TapHoldSettings {
    fn default() -> Self {
        Self {
            tapping_term: 200,
            quick_tap_term: None,
            hold_mode: HoldDecisionMode::Default,
            retro_tapping: false,
            tapping_toggle: 5,
            flow_tap_term: None,
            chordal_hold: false,
            preset: TapHoldPreset::Default,
        }
    }
}

impl TapHoldSettings {
    /// Creates settings from a preset.
    #[must_use]
    pub fn from_preset(preset: TapHoldPreset) -> Self {
        preset.settings()
    }

    /// Applies a preset, updating all values.
    pub fn apply_preset(&mut self, preset: TapHoldPreset) {
        *self = Self::from_preset(preset);
    }

    /// Marks settings as custom (called when any value is manually changed).
    pub const fn mark_custom(&mut self) {
        self.preset = TapHoldPreset::Custom;
    }

    /// Checks if `tapping_term` differs from QMK default.
    #[must_use]
    pub const fn has_custom_tapping_term(&self) -> bool {
        self.tapping_term != 200
    }

    /// Checks if `quick_tap_term` is explicitly set.
    #[must_use]
    pub const fn has_custom_quick_tap_term(&self) -> bool {
        self.quick_tap_term.is_some()
    }

    /// Checks if any non-default settings are configured.
    #[must_use]
    pub fn has_custom_settings(&self) -> bool {
        self.has_custom_tapping_term()
            || self.has_custom_quick_tap_term()
            || self.hold_mode != HoldDecisionMode::Default
            || self.retro_tapping
            || self.tapping_toggle != 5
            || self.flow_tap_term.is_some()
            || self.chordal_hold
    }

    /// Validates settings are within acceptable ranges.
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.tapping_term < 50 || self.tapping_term > 1000 {
            anyhow::bail!("Tapping term must be between 50 and 1000ms");
        }
        if let Some(qt) = self.quick_tap_term {
            if qt > 1000 {
                anyhow::bail!("Quick tap term must be at most 1000ms");
            }
        }
        if self.tapping_toggle < 1 || self.tapping_toggle > 10 {
            anyhow::bail!("Tapping toggle must be between 1 and 10");
        }
        if let Some(ft) = self.flow_tap_term {
            if ft > 500 {
                anyhow::bail!("Flow tap term must be at most 500ms");
            }
        }
        Ok(())
    }
}
