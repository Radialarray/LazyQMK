//! RGB color handling with hex parsing and serialization.

// Allow small types passed by reference for API consistency
#![allow(clippy::trivially_copy_pass_by_ref)]
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
    #[allow(dead_code)] // bin/lib split: ratatui feature unused by web/bin builds
    #[must_use]
    #[allow(dead_code)] // bin/lib split: ratatui conversion (used only with ratatui feature; tests in lib)
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
mod tests;
