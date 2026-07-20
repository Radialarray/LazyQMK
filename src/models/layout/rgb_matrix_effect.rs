//! Standard RGB Matrix effect modes available in QMK.

use serde::{Deserialize, Serialize};


/// Standard RGB Matrix effect modes available in QMK.
///
/// These correspond to QMK's `RGB_MATRIX`_* modes and are used for idle effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RgbMatrixEffect {
    /// Solid color (no animation)
    #[default]
    #[serde(rename = "solid_color")]
    SolidColor,

    /// Breathing animation
    #[serde(rename = "breathing")]
    Breathing,

    /// Rainbow moving chevron
    #[serde(rename = "rainbow_moving_chevron")]
    RainbowMovingChevron,

    /// Cycle all LEDs through hue
    #[serde(rename = "cycle_all")]
    CycleAll,

    /// Cycle left to right
    #[serde(rename = "cycle_left_right")]
    CycleLeftRight,

    /// Cycle up and down
    #[serde(rename = "cycle_up_down")]
    CycleUpDown,

    /// Rainbow beacon animation
    #[serde(rename = "rainbow_beacon")]
    RainbowBeacon,

    /// Rainbow pinwheels
    #[serde(rename = "rainbow_pinwheels")]
    RainbowPinwheels,

    /// Jellybean raindrops
    #[serde(rename = "jellybean_raindrops")]
    JellybeanRaindrops,
}

impl RgbMatrixEffect {
    /// Returns all available effects.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::SolidColor,
            Self::Breathing,
            Self::RainbowMovingChevron,
            Self::CycleAll,
            Self::CycleLeftRight,
            Self::CycleUpDown,
            Self::RainbowBeacon,
            Self::RainbowPinwheels,
            Self::JellybeanRaindrops,
        ]
    }

    /// Returns a human-readable name for this effect.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::SolidColor => "Solid Color",
            Self::Breathing => "Breathing",
            Self::RainbowMovingChevron => "Rainbow Moving Chevron",
            Self::CycleAll => "Cycle All",
            Self::CycleLeftRight => "Cycle Left/Right",
            Self::CycleUpDown => "Cycle Up/Down",
            Self::RainbowBeacon => "Rainbow Beacon",
            Self::RainbowPinwheels => "Rainbow Pinwheels",
            Self::JellybeanRaindrops => "Jellybean Raindrops",
        }
    }

    /// Returns the QMK `RGB_MATRIX`_* mode identifier for code generation.
    ///
    /// These map to the `RGB_MATRIX`_* enum values defined in QMK's `rgb_matrix_types.h`.
    /// The mode IDs are used in firmware to set the RGB effect mode.
    #[must_use]
    pub const fn qmk_mode_name(&self) -> &'static str {
        match self {
            Self::SolidColor => "RGB_MATRIX_SOLID_COLOR",
            Self::Breathing => "RGB_MATRIX_BREATHING",
            Self::RainbowMovingChevron => "RGB_MATRIX_RAINBOW_MOVING_CHEVRON",
            Self::CycleAll => "RGB_MATRIX_CYCLE_ALL",
            Self::CycleLeftRight => "RGB_MATRIX_CYCLE_LEFT_RIGHT",
            Self::CycleUpDown => "RGB_MATRIX_CYCLE_UP_DOWN",
            Self::RainbowBeacon => "RGB_MATRIX_RAINBOW_BEACON",
            Self::RainbowPinwheels => "RGB_MATRIX_RAINBOW_PINWHEELS",
            Self::JellybeanRaindrops => "RGB_MATRIX_JELLYBEAN_RAINDROPS",
        }
    }

    /// Parses an effect from a string name (case-insensitive).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let name_lower = name.to_lowercase().replace([' ', '_', '-'], "");
        match name_lower.as_str() {
            "solidcolor" | "solid" => Some(Self::SolidColor),
            "breathing" | "breath" => Some(Self::Breathing),
            "rainbowmovingchevron" | "chevron" => Some(Self::RainbowMovingChevron),
            "cycleall" | "cycle" => Some(Self::CycleAll),
            "cycleleftright" | "leftright" => Some(Self::CycleLeftRight),
            "cycleupdown" | "updown" => Some(Self::CycleUpDown),
            "rainbowbeacon" | "beacon" => Some(Self::RainbowBeacon),
            "rainbowpinwheels" | "pinwheels" => Some(Self::RainbowPinwheels),
            "jellybeanraindrops" | "raindrops" | "jellybean" => Some(Self::JellybeanRaindrops),
            _ => None,
        }
    }
}
