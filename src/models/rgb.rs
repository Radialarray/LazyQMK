//! RGB color handling with hex parsing and serialization.

// Allow small types passed by reference for API consistency
#![allow(clippy::trivially_copy_pass_by_ref)]
// Allow intentional type casts for color math
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
// Allow float comparisons in HSV conversion (standard algorithms)
#![allow(clippy::float_cmp)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// RGB color value with hex string representation.
///
/// Represents a color using red, green, and blue channels (0-255 each).
/// Supports parsing from hex strings (#RRGGBB) and serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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
    /// use lazyqmk::models::RgbColor;
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
    /// use lazyqmk::models::RgbColor;
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
    /// use lazyqmk::models::RgbColor;
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

    /// Converts the RGB color to HSV (Hue, Saturation, Value) color space.
    ///
    /// # Returns
    ///
    /// A tuple `(h, s, v)` where:
    /// - `h` (Hue): 0.0-360.0 degrees (0.0 for grayscale)
    /// - `s` (Saturation): 0.0-1.0
    /// - `v` (Value/Brightness): 0.0-1.0
    ///
    /// # Examples
    ///
    /// ```
    /// use lazyqmk::models::RgbColor;
    ///
    /// let red = RgbColor::new(255, 0, 0);
    /// let (h, s, v) = red.to_hsv();
    /// assert!((h - 0.0).abs() < 0.01);
    /// assert!((s - 1.0).abs() < 0.01);
    /// assert!((v - 1.0).abs() < 0.01);
    /// ```
    #[must_use]
    #[allow(clippy::many_single_char_names)] // Standard RGB/HSV color model uses single-char names
    pub fn to_hsv(&self) -> (f32, f32, f32) {
        let r = f32::from(self.r) / 255.0;
        let g = f32::from(self.g) / 255.0;
        let b = f32::from(self.b) / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        // Value is the maximum of RGB
        let v = max;

        // Saturation
        let s = if max == 0.0 { 0.0 } else { delta / max };

        // Hue
        let h = if delta == 0.0 {
            0.0 // Grayscale, hue is undefined
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        // Normalize hue to 0-360 range
        let h = if h < 0.0 { h + 360.0 } else { h };

        (h, s, v)
    }

    /// Creates an `RgbColor` from HSV (Hue, Saturation, Value) color space.
    ///
    /// # Arguments
    ///
    /// * `h` - Hue in degrees (0.0-360.0, will be clamped)
    /// * `s` - Saturation (0.0-1.0, will be clamped)
    /// * `v` - Value/Brightness (0.0-1.0, will be clamped)
    ///
    /// # Examples
    ///
    /// ```
    /// use lazyqmk::models::RgbColor;
    ///
    /// let red = RgbColor::from_hsv(0.0, 1.0, 1.0);
    /// assert_eq!(red, RgbColor::new(255, 0, 0));
    ///
    /// let green = RgbColor::from_hsv(120.0, 1.0, 1.0);
    /// assert_eq!(green, RgbColor::new(0, 255, 0));
    /// ```
    #[must_use]
    #[allow(clippy::many_single_char_names)] // Standard RGB/HSV color model uses single-char names
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        // Clamp inputs
        let h = h.clamp(0.0, 360.0);
        let s = s.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let c = v * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
        let m = v - c;

        let (r, g, b) = if h_prime < 1.0 {
            (c, x, 0.0)
        } else if h_prime < 2.0 {
            (x, c, 0.0)
        } else if h_prime < 3.0 {
            (0.0, c, x)
        } else if h_prime < 4.0 {
            (0.0, x, c)
        } else if h_prime < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self {
            r: ((r + m) * 255.0).round().clamp(0.0, 255.0) as u8,
            g: ((g + m) * 255.0).round().clamp(0.0, 255.0) as u8,
            b: ((b + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        }
    }

    /// Returns a color with adjusted saturation.
    ///
    /// # Arguments
    ///
    /// * `percent` - Saturation percentage (0-200). 0 = grayscale, 100 = original, 200 = max saturation.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazyqmk::models::RgbColor;
    ///
    /// let color = RgbColor::new(200, 100, 100);
    /// let saturated = color.saturate(200); // Double saturation
    /// let desaturated = color.saturate(50); // Half saturation
    /// let gray = color.saturate(0); // Remove all saturation
    /// ```
    #[must_use]
    pub fn saturate(&self, percent: u8) -> Self {
        let (h, s, v) = self.to_hsv();
        let new_s = (s * f32::from(percent) / 100.0).min(1.0);
        Self::from_hsv(h, new_s, v)
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

    // HSV conversion tests

    #[test]
    fn test_rgb_to_hsv_primary_colors() {
        // Red
        let red = RgbColor::new(255, 0, 0);
        let (h, s, v) = red.to_hsv();
        assert!((h - 0.0).abs() < 0.01);
        assert!((s - 1.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);

        // Green
        let green = RgbColor::new(0, 255, 0);
        let (h, s, v) = green.to_hsv();
        assert!((h - 120.0).abs() < 0.01);
        assert!((s - 1.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);

        // Blue
        let blue = RgbColor::new(0, 0, 255);
        let (h, s, v) = blue.to_hsv();
        assert!((h - 240.0).abs() < 0.01);
        assert!((s - 1.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_rgb_to_hsv_grayscale() {
        // Black
        let black = RgbColor::new(0, 0, 0);
        let (h, s, v) = black.to_hsv();
        assert_eq!(h, 0.0);
        assert_eq!(s, 0.0);
        assert_eq!(v, 0.0);

        // White
        let white = RgbColor::new(255, 255, 255);
        let (h, s, v) = white.to_hsv();
        assert_eq!(h, 0.0);
        assert_eq!(s, 0.0);
        assert!((v - 1.0).abs() < 0.01);

        // Gray
        let gray = RgbColor::new(128, 128, 128);
        let (h, s, v) = gray.to_hsv();
        assert_eq!(h, 0.0);
        assert_eq!(s, 0.0);
        assert!((v - 0.502).abs() < 0.01); // 128/255 ≈ 0.502
    }

    #[test]
    fn test_hsv_to_rgb_primary_colors() {
        // Red
        let red = RgbColor::from_hsv(0.0, 1.0, 1.0);
        assert_eq!(red, RgbColor::new(255, 0, 0));

        // Green
        let green = RgbColor::from_hsv(120.0, 1.0, 1.0);
        assert_eq!(green, RgbColor::new(0, 255, 0));

        // Blue
        let blue = RgbColor::from_hsv(240.0, 1.0, 1.0);
        assert_eq!(blue, RgbColor::new(0, 0, 255));
    }

    #[test]
    fn test_hsv_to_rgb_grayscale() {
        // Black
        let black = RgbColor::from_hsv(0.0, 0.0, 0.0);
        assert_eq!(black, RgbColor::new(0, 0, 0));

        // White
        let white = RgbColor::from_hsv(0.0, 0.0, 1.0);
        assert_eq!(white, RgbColor::new(255, 255, 255));

        // Gray (hue doesn't matter for grayscale)
        let gray = RgbColor::from_hsv(180.0, 0.0, 0.5);
        assert_eq!(gray, RgbColor::new(128, 128, 128));
    }

    #[test]
    fn test_hsv_roundtrip() {
        // Test various colors round-trip through HSV
        let colors = vec![
            RgbColor::new(255, 0, 0),    // Red
            RgbColor::new(0, 255, 0),    // Green
            RgbColor::new(0, 0, 255),    // Blue
            RgbColor::new(255, 255, 0),  // Yellow
            RgbColor::new(255, 0, 255),  // Magenta
            RgbColor::new(0, 255, 255),  // Cyan
            RgbColor::new(128, 64, 192), // Purple-ish
            RgbColor::new(200, 100, 50), // Orange-ish
        ];

        for color in colors {
            let (h, s, v) = color.to_hsv();
            let converted = RgbColor::from_hsv(h, s, v);
            // Allow small rounding errors (±1 per channel)
            assert!(
                (i16::from(color.r) - i16::from(converted.r)).abs() <= 1,
                "Red channel mismatch: {} vs {}",
                color.r,
                converted.r
            );
            assert!(
                (i16::from(color.g) - i16::from(converted.g)).abs() <= 1,
                "Green channel mismatch: {} vs {}",
                color.g,
                converted.g
            );
            assert!(
                (i16::from(color.b) - i16::from(converted.b)).abs() <= 1,
                "Blue channel mismatch: {} vs {}",
                color.b,
                converted.b
            );
        }
    }

    #[test]
    fn test_hsv_clamping() {
        // Test that out-of-range HSV values are clamped
        let color = RgbColor::from_hsv(400.0, 1.5, 1.5);
        // Should clamp to (360.0, 1.0, 1.0) which is red
        assert_eq!(color, RgbColor::new(255, 0, 0));

        let color = RgbColor::from_hsv(-10.0, -0.5, -0.5);
        // Should clamp to (0.0, 0.0, 0.0) which is black
        assert_eq!(color, RgbColor::new(0, 0, 0));
    }

    #[test]
    fn test_saturate_same_color() {
        let color = RgbColor::new(200, 100, 50);
        let same = color.saturate(100);
        // 100% should return approximately the same color
        assert!(
            (i16::from(color.r) - i16::from(same.r)).abs() <= 1,
            "Red channel should be unchanged"
        );
        assert!(
            (i16::from(color.g) - i16::from(same.g)).abs() <= 1,
            "Green channel should be unchanged"
        );
        assert!(
            (i16::from(color.b) - i16::from(same.b)).abs() <= 1,
            "Blue channel should be unchanged"
        );
    }

    #[test]
    fn test_saturate_grayscale() {
        let color = RgbColor::new(200, 100, 50);
        let gray = color.saturate(0);
        // 0% saturation should produce grayscale (all channels equal)
        assert_eq!(gray.r, gray.g);
        assert_eq!(gray.g, gray.b);
    }

    #[test]
    fn test_saturate_increase() {
        let color = RgbColor::new(200, 150, 150);
        let saturated = color.saturate(200);

        // More saturated color should have greater difference between max and min channels
        let original_range = color.r.max(color.g).max(color.b) - color.r.min(color.g).min(color.b);
        let saturated_range = saturated.r.max(saturated.g).max(saturated.b)
            - saturated.r.min(saturated.g).min(saturated.b);

        assert!(
            saturated_range >= original_range,
            "Saturated color should have greater channel range"
        );
    }

    #[test]
    fn test_saturate_clamp_at_max() {
        // Start with a fully saturated color
        let red = RgbColor::new(255, 0, 0);
        let (_, s, _) = red.to_hsv();
        assert!((s - 1.0).abs() < 0.01, "Red should be fully saturated");

        // Trying to saturate beyond 100% should clamp at maximum
        let more_saturated = red.saturate(200);
        let (_, s2, _) = more_saturated.to_hsv();
        assert!(
            (s2 - 1.0).abs() < 0.01,
            "Saturation should be clamped at 1.0"
        );
    }

    #[test]
    fn test_saturate_edge_cases() {
        // Black remains black regardless of saturation
        let black = RgbColor::new(0, 0, 0);
        assert_eq!(black.saturate(0), black);
        assert_eq!(black.saturate(100), black);
        assert_eq!(black.saturate(200), black);

        // White remains white regardless of saturation
        let white = RgbColor::new(255, 255, 255);
        assert_eq!(white.saturate(0), white);
        assert_eq!(white.saturate(100), white);
        assert_eq!(white.saturate(200), white);
    }
}
