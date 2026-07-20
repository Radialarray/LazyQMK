//! PaletteFX community module types — effects, palettes, and settings.

use serde::{Deserialize, Serialize};

/// `PaletteFX` effect types (community module by getreuer).
///
/// These replace our custom ripple overlay with professional-quality
/// palette-based RGB matrix effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PaletteFxEffect {
    /// Gradient: vertical color gradient
    #[default]
    #[serde(rename = "gradient")]
    Gradient,
    /// Flow: animated wave patterns
    #[serde(rename = "flow")]
    Flow,
    /// Ripple: circular rings emanating from random points
    #[serde(rename = "ripple")]
    Ripple,
    /// Sparkle: LEDs sparkle with pseudorandom phase
    #[serde(rename = "sparkle")]
    Sparkle,
    /// Vortex: spinning vortex centered on keyboard
    #[serde(rename = "vortex")]
    Vortex,
    /// Reactive: responds to key presses
    #[serde(rename = "reactive")]
    Reactive,
}

impl PaletteFxEffect {
    /// Returns all available effects.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Gradient,
            Self::Flow,
            Self::Ripple,
            Self::Sparkle,
            Self::Vortex,
            Self::Reactive,
        ]
    }

    /// Returns a human-readable display name.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Gradient => "Gradient",
            Self::Flow => "Flow",
            Self::Ripple => "Ripple",
            Self::Sparkle => "Sparkle",
            Self::Vortex => "Vortex",
            Self::Reactive => "Reactive",
        }
    }

    /// Returns the QMK `PALETTEFX_*` enum value for code generation.
    #[must_use]
    pub const fn qmk_mode_name(&self) -> &'static str {
        match self {
            Self::Gradient => "RGB_MATRIX_COMMUNITY_MODULE_PALETTEFX_GRADIENT",
            Self::Flow => "RGB_MATRIX_COMMUNITY_MODULE_PALETTEFX_FLOW",
            Self::Ripple => "RGB_MATRIX_COMMUNITY_MODULE_PALETTEFX_RIPPLE",
            Self::Sparkle => "RGB_MATRIX_COMMUNITY_MODULE_PALETTEFX_SPARKLE",
            Self::Vortex => "RGB_MATRIX_COMMUNITY_MODULE_PALETTEFX_VORTEX",
            Self::Reactive => "RGB_MATRIX_COMMUNITY_MODULE_PALETTEFX_REACTIVE",
        }
    }

    /// Returns the config.h define name for enabling this effect.
    #[must_use]
    pub const fn enable_define(&self) -> &'static str {
        match self {
            Self::Gradient => "PALETTEFX_GRADIENT_ENABLE",
            Self::Flow => "PALETTEFX_FLOW_ENABLE",
            Self::Ripple => "PALETTEFX_RIPPLE_ENABLE",
            Self::Sparkle => "PALETTEFX_SPARKLE_ENABLE",
            Self::Vortex => "PALETTEFX_VORTEX_ENABLE",
            Self::Reactive => "PALETTEFX_REACTIVE_ENABLE",
        }
    }

    /// Parses an effect from a string name (case-insensitive).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let name_lower = name.to_lowercase().replace([' ', '_', '-'], "");
        match name_lower.as_str() {
            "gradient" => Some(Self::Gradient),
            "flow" => Some(Self::Flow),
            "ripple" => Some(Self::Ripple),
            "sparkle" => Some(Self::Sparkle),
            "vortex" => Some(Self::Vortex),
            "reactive" => Some(Self::Reactive),
            _ => None,
        }
    }
}

/// `PaletteFX` palette options.
///
/// Palettes are color gradients sampled from 16 HSV color stops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PaletteFxPalette {
    /// Afterburn: fiery oranges/reds
    #[default]
    #[serde(rename = "afterburn")]
    Afterburn,
    /// Amber: warm amber tones
    #[serde(rename = "amber")]
    Amber,
    /// Bad Wolf: dark theme-inspired
    #[serde(rename = "bad_wolf")]
    BadWolf,
    /// Carnival: vibrant rainbow
    #[serde(rename = "carnival")]
    Carnival,
    /// Classic: traditional color wheel
    #[serde(rename = "classic")]
    Classic,
    /// Dracula: dark purple/pink theme
    #[serde(rename = "dracula")]
    Dracula,
    /// Groovy: retro warm tones
    #[serde(rename = "groovy")]
    Groovy,
    /// Not Pink: pink/magenta tones
    #[serde(rename = "not_pink")]
    NotPink,
    /// Phosphor: green/teal glow
    #[serde(rename = "phosphor")]
    Phosphor,
    /// Polarized: blue/cyan tones
    #[serde(rename = "polarized")]
    Polarized,
    /// Rose Gold: warm rose tones
    #[serde(rename = "rose_gold")]
    RoseGold,
    /// Sport: athletic team colors
    #[serde(rename = "sport")]
    Sport,
    /// Synthwave: retrowave purple/pink/cyan
    #[serde(rename = "synthwave")]
    Synthwave,
    /// Thermal: heat map colors
    #[serde(rename = "thermal")]
    Thermal,
    /// Viridis: perceptually uniform green/blue/purple
    #[serde(rename = "viridis")]
    Viridis,
    /// Watermelon: pink/green tones
    #[serde(rename = "watermelon")]
    Watermelon,
}

impl PaletteFxPalette {
    /// Returns all available palettes.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Afterburn,
            Self::Amber,
            Self::BadWolf,
            Self::Carnival,
            Self::Classic,
            Self::Dracula,
            Self::Groovy,
            Self::NotPink,
            Self::Phosphor,
            Self::Polarized,
            Self::RoseGold,
            Self::Sport,
            Self::Synthwave,
            Self::Thermal,
            Self::Viridis,
            Self::Watermelon,
        ]
    }

    /// Returns a human-readable display name.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Afterburn => "Afterburn",
            Self::Amber => "Amber",
            Self::BadWolf => "Bad Wolf",
            Self::Carnival => "Carnival",
            Self::Classic => "Classic",
            Self::Dracula => "Dracula",
            Self::Groovy => "Groovy",
            Self::NotPink => "Not Pink",
            Self::Phosphor => "Phosphor",
            Self::Polarized => "Polarized",
            Self::RoseGold => "Rose Gold",
            Self::Sport => "Sport",
            Self::Synthwave => "Synthwave",
            Self::Thermal => "Thermal",
            Self::Viridis => "Viridis",
            Self::Watermelon => "Watermelon",
        }
    }

    /// Returns the config.h define name for enabling this palette.
    #[must_use]
    pub const fn enable_define(&self) -> &'static str {
        match self {
            Self::Afterburn => "PALETTEFX_AFTERBURN_ENABLE",
            Self::Amber => "PALETTEFX_AMBER_ENABLE",
            Self::BadWolf => "PALETTEFX_BADWOLF_ENABLE",
            Self::Carnival => "PALETTEFX_CARNIVAL_ENABLE",
            Self::Classic => "PALETTEFX_CLASSIC_ENABLE",
            Self::Dracula => "PALETTEFX_DRACULA_ENABLE",
            Self::Groovy => "PALETTEFX_GROOVY_ENABLE",
            Self::NotPink => "PALETTEFX_NOTPINK_ENABLE",
            Self::Phosphor => "PALETTEFX_PHOSPHOR_ENABLE",
            Self::Polarized => "PALETTEFX_POLARIZED_ENABLE",
            Self::RoseGold => "PALETTEFX_ROSEGOLD_ENABLE",
            Self::Sport => "PALETTEFX_SPORT_ENABLE",
            Self::Synthwave => "PALETTEFX_SYNTHWAVE_ENABLE",
            Self::Thermal => "PALETTEFX_THERMAL_ENABLE",
            Self::Viridis => "PALETTEFX_VIRIDIS_ENABLE",
            Self::Watermelon => "PALETTEFX_WATERMELON_ENABLE",
        }
    }

    /// Parses a palette from a string name (case-insensitive).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let name_lower = name.to_lowercase().replace([' ', '_', '-'], "");
        match name_lower.as_str() {
            "afterburn" => Some(Self::Afterburn),
            "amber" => Some(Self::Amber),
            "badwolf" => Some(Self::BadWolf),
            "carnival" => Some(Self::Carnival),
            "classic" => Some(Self::Classic),
            "dracula" => Some(Self::Dracula),
            "groovy" => Some(Self::Groovy),
            "notpink" => Some(Self::NotPink),
            "phosphor" => Some(Self::Phosphor),
            "polarized" => Some(Self::Polarized),
            "rosegold" => Some(Self::RoseGold),
            "sport" => Some(Self::Sport),
            "synthwave" => Some(Self::Synthwave),
            "thermal" => Some(Self::Thermal),
            "viridis" => Some(Self::Viridis),
            "watermelon" => Some(Self::Watermelon),
            _ => None,
        }
    }
}

/// Configuration for `PaletteFX` module integration.
///
/// When enabled, replaces the custom ripple overlay with the `PaletteFX`
/// community module's professional-quality effects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaletteFxSettings {
    /// Master switch for `PaletteFX` integration
    #[serde(default)]
    pub enabled: bool,

    /// Default effect to use (applied at startup)
    #[serde(default)]
    pub default_effect: PaletteFxEffect,

    /// Default palette to use (applied at startup)
    #[serde(default)]
    pub default_palette: PaletteFxPalette,

    /// Whether to enable all effects (vs individual selection)
    #[serde(default = "default_true")]
    pub enable_all_effects: bool,

    /// Whether to enable all palettes (vs individual selection)
    #[serde(default = "default_true")]
    pub enable_all_palettes: bool,
}

const fn default_true() -> bool {
    true
}

impl Default for PaletteFxSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            default_effect: PaletteFxEffect::Flow,
            default_palette: PaletteFxPalette::Synthwave,
            enable_all_effects: true,
            enable_all_palettes: true,
        }
    }
}
