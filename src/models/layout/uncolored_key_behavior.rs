//! Brightness level for keys without individual or category color.

use serde::{Deserialize, Serialize};


/// Brightness level for keys without an individual or category color assignment.
///
/// This controls how keys are displayed when they only have layer-level colors
/// (layer category or layer default) but no individual override or key category.
///
/// - 0 = Off (black LEDs)
/// - 1-99 = Dim to that percentage
/// - 100 = Show full color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UncoloredKeyBehavior(u8);

impl UncoloredKeyBehavior {
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

    /// Full brightness - show resolved color (100%).
    pub const FULL: Self = Self(100);
}

impl Default for UncoloredKeyBehavior {
    fn default() -> Self {
        Self::FULL // Show full color by default
    }
}

impl From<u8> for UncoloredKeyBehavior {
    fn from(value: u8) -> Self {
        Self::new(value.min(100))
    }
}

