//! Color palette data structures for the enhanced color picker.
//!
//! This module provides a curated color palette based on Tailwind CSS colors,
//! with 12 base colors and 9 shades each.

use serde::Deserialize;

use super::RgbColor;

/// A complete color palette with multiple base colors.
#[derive(Debug, Clone, Deserialize)]
pub struct ColorPalette {
    /// The list of base colors in the palette.
    pub colors: Vec<PaletteColor>,
}

/// A single base color with multiple shades.
#[derive(Debug, Clone, Deserialize)]
pub struct PaletteColor {
    /// Display name of the color (e.g., "Red", "Blue").
    pub name: String,
    /// Shades from light (50) to dark (800).
    pub shades: Vec<Shade>,
}

/// A single shade of a color.
#[derive(Debug, Clone, Deserialize)]
pub struct Shade {
    /// Shade level (50, 100, 200, 300, 400, 500, 600, 700, 800).
    pub level: u16,
    /// Hex color code (e.g., "#EF4444").
    pub hex: String,
    /// Red component (0-255).
    pub r: u8,
    /// Green component (0-255).
    pub g: u8,
    /// Blue component (0-255).
    pub b: u8,
}

impl Shade {
    /// Convert this shade to an `RgbColor`.
    #[must_use]
    pub const fn to_rgb(&self) -> RgbColor {
        RgbColor::new(self.r, self.g, self.b)
    }
}

impl PaletteColor {
    /// Get the "primary" shade (500 level, or middle shade).
    #[must_use]
    pub fn primary_shade(&self) -> Option<&Shade> {
        self.shades
            .iter()
            .find(|s| s.level == 500)
            .or_else(|| self.shades.get(self.shades.len() / 2))
    }

    /// Get a shade by index (0-8).
    #[must_use]
    pub fn shade_at(&self, index: usize) -> Option<&Shade> {
        self.shades.get(index)
    }

    /// Get the number of shades.
    #[must_use]
    pub const fn shade_count(&self) -> usize {
        self.shades.len()
    }
}

impl ColorPalette {
    /// Load the color palette from embedded JSON data.
    ///
    /// # Errors
    /// Returns an error if the JSON data cannot be parsed.
    pub fn load() -> anyhow::Result<Self> {
        let json_data = include_str!("../data/color_palette.json");
        let palette: Self = serde_json::from_str(json_data)?;
        Ok(palette)
    }

    /// Get a color by name (case-insensitive).
    #[must_use]
    pub fn get_color(&self, name: &str) -> Option<&PaletteColor> {
        let name_lower = name.to_lowercase();
        self.colors
            .iter()
            .find(|c| c.name.to_lowercase() == name_lower)
    }

    /// Get a specific shade by color name and level.
    ///
    /// # Example
    /// ```no_run
    /// use lazyqmk::models::ColorPalette;
    /// let palette = ColorPalette::load().unwrap();
    /// let gray_500 = palette.get_shade("Gray", 500);
    /// ```
    #[must_use]
    pub fn get_shade(&self, color_name: &str, level: u16) -> Option<&Shade> {
        self.get_color(color_name)?
            .shades
            .iter()
            .find(|s| s.level == level)
    }

    /// Get the default layer color (Gray-500).
    ///
    /// This is the standard gray used for layers that don't have
    /// a custom color assigned. Returns (107, 114, 128) if palette
    /// loads correctly, or a fallback gray if not.
    #[must_use]
    pub fn default_layer_color(&self) -> RgbColor {
        self.get_shade("Gray", 500)
            .map(Shade::to_rgb)
            .unwrap_or_else(|| RgbColor::new(107, 114, 128))
    }

    /// Get a color by index.
    #[must_use]
    pub fn color_at(&self, index: usize) -> Option<&PaletteColor> {
        self.colors.get(index)
    }

    /// Get the number of base colors.
    #[must_use]
    pub const fn color_count(&self) -> usize {
        self.colors.len()
    }

    /// Get the number of columns for display (4 colors per row).
    #[must_use]
    pub const fn columns(&self) -> usize {
        4
    }

    /// Get the number of rows for display.
    #[must_use]
    pub const fn rows(&self) -> usize {
        self.colors.len().div_ceil(self.columns())
    }
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::load().unwrap_or_else(|_| Self { colors: Vec::new() })
    }
}

#[cfg(test)]
mod tests;

