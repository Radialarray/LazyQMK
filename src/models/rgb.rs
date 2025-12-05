//! RGB color handling with hex parsing and serialization.

// Allow small types passed by reference for API consistency
#![allow(clippy::trivially_copy_pass_by_ref)]
// Allow intentional type casts for color math
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// RGB color value with hex string representation.
///
/// Represents a color using red, green, and blue channels (0-255 each).
/// Supports parsing from hex strings (#RRGGBB) and serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RgbColor {
    /// Red channel (0-255)
    pub r: u8,
    /// Green channel (0-255)
    pub g: u8,
    /// Blue channel (0-255)
    pub b: u8,
}

#[allow(dead_code)]
impl RgbColor {
    /// Creates a new `RgbColor` from individual channel values.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parses an `RgbColor` from a hex string.
    ///
    /// Supports formats: "#RRGGBB", "RRGGBB", "#rrggbb", "rrggbb"
    ///
    /// # Examples
    ///
    /// ```
    /// use keyboard_configurator::models::RgbColor;
    ///
    /// let color = RgbColor::from_hex("#FF0000").unwrap();
    /// assert_eq!(color, RgbColor::new(255, 0, 0));
    ///
    /// let color = RgbColor::from_hex("00FF00").unwrap();
    /// assert_eq!(color, RgbColor::new(0, 255, 0));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not a valid hex color format.
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim();
        let hex = hex.strip_prefix('#').unwrap_or(hex);

        if hex.len() != 6 {
            anyhow::bail!("Invalid hex color format '{hex}'. Expected 6 hex digits (RRGGBB)");
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .context(format!("Invalid red channel in hex color '{hex}'"))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .context(format!("Invalid green channel in hex color '{hex}'"))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .context(format!("Invalid blue channel in hex color '{hex}'"))?;

        Ok(Self::new(r, g, b))
    }

    /// Converts the color to a hex string in the format "#RRGGBB" (uppercase).
    ///
    /// # Examples
    ///
    /// ```
    /// use keyboard_configurator::models::RgbColor;
    ///
    /// let color = RgbColor::new(255, 0, 0);
    /// assert_eq!(color.to_hex(), "#FF0000");
    ///
    /// let color = RgbColor::new(0, 128, 255);
    /// assert_eq!(color.to_hex(), "#0080FF");
    /// ```
    #[must_use]
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Converts the color to a Ratatui Color for terminal rendering.
    #[cfg(feature = "ratatui")]
    #[must_use]
    pub const fn to_ratatui_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::Rgb(self.r, self.g, self.b)
    }

    /// Returns a dimmed version of the color at the given percentage.
    ///
    /// # Arguments
    ///
    /// * `percent` - Brightness percentage (0-100). 0 = black, 100 = original color.
    ///
    /// # Examples
    ///
    /// ```
    /// use keyboard_configurator::models::RgbColor;
    ///
    /// let color = RgbColor::new(200, 100, 50);
    /// let dimmed = color.dim(50); // 50% brightness
    /// assert_eq!(dimmed, RgbColor::new(100, 50, 25));
    /// ```
    #[must_use]
    pub const fn dim(&self, percent: u8) -> Self {
        let percent = if percent > 100 { 100 } else { percent };
        Self {
            r: (self.r as u16 * percent as u16 / 100) as u8,
            g: (self.g as u16 * percent as u16 / 100) as u8,
            b: (self.b as u16 * percent as u16 / 100) as u8,
        }
    }
}

impl fmt::Display for RgbColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Default for RgbColor {
    /// Default color is white (#FFFFFF).
    fn default() -> Self {
        Self::new(255, 255, 255)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex_valid() {
        let color = RgbColor::from_hex("#FF0000").unwrap();
        assert_eq!(color, RgbColor::new(255, 0, 0));

        let color = RgbColor::from_hex("00FF00").unwrap();
        assert_eq!(color, RgbColor::new(0, 255, 0));

        let color = RgbColor::from_hex("#0000ff").unwrap();
        assert_eq!(color, RgbColor::new(0, 0, 255));

        let color = RgbColor::from_hex("  #FFFFFF  ").unwrap();
        assert_eq!(color, RgbColor::new(255, 255, 255));
    }

    #[test]
    fn test_from_hex_invalid() {
        assert!(RgbColor::from_hex("#FFF").is_err());
        assert!(RgbColor::from_hex("#FFFFFFF").is_err());
        assert!(RgbColor::from_hex("GGGGGG").is_err());
        assert!(RgbColor::from_hex("").is_err());
        assert!(RgbColor::from_hex("#").is_err());
    }

    #[test]
    fn test_to_hex() {
        let color = RgbColor::new(255, 0, 0);
        assert_eq!(color.to_hex(), "#FF0000");

        let color = RgbColor::new(0, 128, 255);
        assert_eq!(color.to_hex(), "#0080FF");

        let color = RgbColor::new(0, 0, 0);
        assert_eq!(color.to_hex(), "#000000");
    }

    #[test]
    fn test_roundtrip() {
        let original = RgbColor::new(123, 45, 67);
        let hex = original.to_hex();
        let parsed = RgbColor::from_hex(&hex).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_default() {
        let color = RgbColor::default();
        assert_eq!(color, RgbColor::new(255, 255, 255));
    }
}
