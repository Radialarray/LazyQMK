//! RGB saturation level (0-200%).

use serde::{Deserialize, Serialize};

/// RGB saturation level (0-200%).
///
/// Controls the global saturation multiplier for all RGB LEDs.
/// 0 = fully desaturated (grayscale), 100 = original colors, 200 = maximum saturation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbSaturation(u8);

impl RgbSaturation {
    /// Creates a new saturation value (0-200).
    ///
    /// # Panics
    /// Panics if value > 200
    #[must_use]
    pub const fn new(value: u8) -> Self {
        assert!(value <= 200, "Saturation must be 0-200");
        Self(value)
    }

    /// Returns the saturation as a percentage (0-200).
    #[must_use]
    pub const fn as_percent(&self) -> u8 {
        self.0
    }

    /// Neutral saturation (100%) - no change to colors.
    pub const NEUTRAL: Self = Self(100);
}

impl Default for RgbSaturation {
    fn default() -> Self {
        Self::NEUTRAL
    }
}

impl From<u8> for RgbSaturation {
    fn from(value: u8) -> Self {
        Self::new(value.min(200))
    }
}
