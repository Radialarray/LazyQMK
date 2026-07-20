//! RGB brightness level (0-100%).

use serde::{Deserialize, Serialize};


/// RGB brightness level (0-100%).
///
/// Controls the global brightness multiplier for all RGB LEDs.
/// 0 = LEDs off, 100 = full brightness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbBrightness(u8);

impl RgbBrightness {
    /// Creates a new brightness value (0-100).
    ///
    /// # Panics
    /// Panics if value > 100
    #[must_use]
    pub const fn new(value: u8) -> Self {
        assert!(value <= 100, "Brightness must be 0-100");
        Self(value)
    }

    /// Returns the brightness as a percentage (0-100).
    #[must_use]
    pub const fn as_percent(&self) -> u8 {
        self.0
    }

    /// Full brightness (100%).
    pub const FULL: Self = Self(100);
}

impl Default for RgbBrightness {
    fn default() -> Self {
        Self::FULL
    }
}

impl From<u8> for RgbBrightness {
    fn from(value: u8) -> Self {
        Self::new(value.min(100))
    }
}
