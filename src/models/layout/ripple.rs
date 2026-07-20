//! RGB overlay ripple settings and color mode.

use serde::{Deserialize, Serialize};

use super::PaletteFxPalette;
use crate::models::RgbColor;

/// Color mode for ripple overlay effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RippleColorMode {
    /// Use a fixed color for all ripples
    #[default]
    #[serde(rename = "fixed")]
    Fixed,
    /// Use the key's base color (from layer colors)
    #[serde(rename = "key_based")]
    KeyBased,
    /// Shift the hue by a fixed amount from the key's base color
    #[serde(rename = "hue_shift")]
    HueShift,
}

impl RippleColorMode {
    /// Returns all available color modes
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Fixed, Self::KeyBased, Self::HueShift]
    }

    /// Returns a human-readable display name
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Fixed => "Fixed Color",
            Self::KeyBased => "Key Color",
            Self::HueShift => "Hue Shift",
        }
    }

    /// Returns a description of the color mode
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Fixed => "Use the same color for all ripples",
            Self::KeyBased => "Use each key's base color from layer settings",
            Self::HueShift => "Shift hue from key's base color by fixed degrees",
        }
    }
}

/// Configuration for RGB overlay ripple effects.
///
/// Ripples are triggered on keypress and rendered as an additive overlay
/// on top of the base TUI layer colors using `rgb_matrix_indicators_advanced_user`.
#[allow(clippy::struct_excessive_bools)] // Configuration flags are naturally bools
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbOverlayRippleSettings {
    /// Whether ripple overlay is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Maximum number of concurrent ripples (1-8)
    /// Default: 4
    #[serde(default = "default_max_ripples")]
    pub max_ripples: u8,

    /// Duration of each ripple in milliseconds
    /// Default: 1500ms
    #[serde(default = "default_ripple_duration_ms")]
    pub duration_ms: u16,

    /// Speed multiplier (0-255, higher = faster expansion in 2D pixel space)
    /// Default: 200
    #[serde(default = "default_ripple_speed")]
    pub speed: u8,

    /// Band width in pixel distance units (2D coordinate space)
    /// Default: 30
    #[serde(default = "default_ripple_band_width")]
    pub band_width: u8,

    /// Amplitude as percentage of base brightness (0-100)
    /// Default: 50
    #[serde(default = "default_ripple_amplitude_pct")]
    pub amplitude_pct: u8,

    /// Number of concentric waves per keypress (1-5)
    /// Default: 1 (single wave)
    #[serde(default = "default_ripple_wave_count")]
    pub wave_count: u8,

    /// Delay between consecutive waves in milliseconds (50-500)
    /// Default: 100ms
    #[serde(default = "default_ripple_wave_delay_ms")]
    pub wave_delay_ms: u16,

    /// Color mode for ripples
    #[serde(default)]
    pub color_mode: RippleColorMode,

    /// Fixed color (used when `color_mode` = Fixed)
    #[serde(default = "default_ripple_fixed_color")]
    pub fixed_color: RgbColor,

    /// Hue shift in degrees (used when `color_mode` = `HueShift`)
    /// Default: 60 (complementary color)
    #[serde(default = "default_ripple_hue_shift")]
    pub hue_shift_deg: i16,

    /// Trigger on key press
    #[serde(default = "default_true")]
    pub trigger_on_press: bool,

    /// Trigger on key release
    #[serde(default)]
    pub trigger_on_release: bool,

    /// Ignore transparent keys (`KC_TRNS`)
    #[serde(default = "default_true")]
    pub ignore_transparent: bool,

    /// Ignore modifier keys
    #[serde(default)]
    pub ignore_modifiers: bool,

    /// Ignore layer switch keys
    #[serde(default)]
    pub ignore_layer_switch: bool,

    /// `PaletteFX` palette to use for key-action reactive bursts.
    /// When `Some`, overrides the current palette with a specific one.
    /// Only effective when `palette_fx.enabled` is true.
    /// Default: `None` (use the current active palette).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_action_palette: Option<PaletteFxPalette>,
}

const fn default_max_ripples() -> u8 {
    4
}

const fn default_ripple_duration_ms() -> u16 {
    1500
}

const fn default_ripple_speed() -> u8 {
    200
}

const fn default_ripple_band_width() -> u8 {
    30
}

const fn default_ripple_amplitude_pct() -> u8 {
    50
}

fn default_ripple_fixed_color() -> RgbColor {
    RgbColor::new(0, 255, 255) // Cyan
}

const fn default_ripple_hue_shift() -> i16 {
    60
}

const fn default_ripple_wave_count() -> u8 {
    1
}

const fn default_ripple_wave_delay_ms() -> u16 {
    100
}

const fn default_true() -> bool {
    true
}

impl Default for RgbOverlayRippleSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            max_ripples: 4,
            duration_ms: 1500,
            speed: 200,
            band_width: 30,
            amplitude_pct: 50,
            wave_count: 1,
            wave_delay_ms: 100,
            color_mode: RippleColorMode::Fixed,
            fixed_color: RgbColor::new(0, 255, 255),
            hue_shift_deg: 60,
            trigger_on_press: true,
            trigger_on_release: false,
            ignore_transparent: true,
            ignore_modifiers: false,
            ignore_layer_switch: false,
            key_action_palette: None,
        }
    }
}

impl RgbOverlayRippleSettings {
    /// Validates the settings.
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.max_ripples == 0 || self.max_ripples > 8 {
            anyhow::bail!("max_ripples must be between 1 and 8");
        }
        if self.duration_ms == 0 {
            anyhow::bail!("duration_ms must be greater than 0");
        }
        if self.speed == 0 {
            anyhow::bail!("speed must be greater than 0");
        }
        if self.band_width == 0 {
            anyhow::bail!("band_width must be greater than 0");
        }
        if self.amplitude_pct > 100 {
            anyhow::bail!("amplitude_pct must be between 0 and 100");
        }
        if self.hue_shift_deg < -180 || self.hue_shift_deg > 180 {
            anyhow::bail!("hue_shift_deg must be between -180 and 180");
        }
        if self.wave_count == 0 || self.wave_count > 5 {
            anyhow::bail!("wave_count must be between 1 and 5");
        }
        if self.wave_delay_ms < 50 || self.wave_delay_ms > 500 {
            anyhow::bail!("wave_delay_ms must be between 50 and 500");
        }
        Ok(())
    }

    /// Checks if any settings differ from defaults.
    #[must_use]
    pub fn has_custom_settings(&self) -> bool {
        let defaults = Self::default();
        self.enabled != defaults.enabled
            || self.max_ripples != defaults.max_ripples
            || self.duration_ms != defaults.duration_ms
            || self.speed != defaults.speed
            || self.band_width != defaults.band_width
            || self.amplitude_pct != defaults.amplitude_pct
            || self.color_mode != defaults.color_mode
            || self.fixed_color != defaults.fixed_color
            || self.hue_shift_deg != defaults.hue_shift_deg
            || self.trigger_on_press != defaults.trigger_on_press
            || self.trigger_on_release != defaults.trigger_on_release
            || self.ignore_transparent != defaults.ignore_transparent
            || self.ignore_modifiers != defaults.ignore_modifiers
            || self.ignore_layer_switch != defaults.ignore_layer_switch
            || self.key_action_palette != defaults.key_action_palette
            || self.wave_count != defaults.wave_count
            || self.wave_delay_ms != defaults.wave_delay_ms
    }
}
