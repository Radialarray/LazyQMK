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
    /// Convert this shade to an RgbColor.
    #[must_use]
    pub const fn to_rgb(&self) -> RgbColor {
        RgbColor::new(self.r, self.g, self.b)
    }
}

impl PaletteColor {
    /// Get the "primary" shade (500 level, or middle shade).
    #[must_use]
    pub fn primary_shade(&self) -> Option<&Shade> {
        self.shades.iter().find(|s| s.level == 500)
            .or_else(|| self.shades.get(self.shades.len() / 2))
    }
    
    /// Get a shade by index (0-8).
    #[must_use]
    pub fn shade_at(&self, index: usize) -> Option<&Shade> {
        self.shades.get(index)
    }
    
    /// Get the number of shades.
    #[must_use]
    pub fn shade_count(&self) -> usize {
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
    
    /// Get a color by index.
    #[must_use]
    pub fn color_at(&self, index: usize) -> Option<&PaletteColor> {
        self.colors.get(index)
    }
    
    /// Get the number of base colors.
    #[must_use]
    pub fn color_count(&self) -> usize {
        self.colors.len()
    }
    
    /// Get the number of columns for display (4 colors per row).
    #[must_use]
    pub const fn columns(&self) -> usize {
        4
    }
    
    /// Get the number of rows for display.
    #[must_use]
    pub fn rows(&self) -> usize {
        (self.colors.len() + self.columns() - 1) / self.columns()
    }
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::load().unwrap_or_else(|_| Self { colors: Vec::new() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_palette() {
        let palette = ColorPalette::load().expect("Failed to load palette");
        assert_eq!(palette.color_count(), 12);
    }

    #[test]
    fn test_palette_colors() {
        let palette = ColorPalette::load().expect("Failed to load palette");
        
        // Check first color is Red
        let red = palette.color_at(0).expect("Red should exist");
        assert_eq!(red.name, "Red");
        assert_eq!(red.shade_count(), 9);
        
        // Check Red-500
        let red_500 = red.primary_shade().expect("Red-500 should exist");
        assert_eq!(red_500.level, 500);
        assert_eq!(red_500.hex, "#EF4444");
        assert_eq!(red_500.r, 239);
        assert_eq!(red_500.g, 68);
        assert_eq!(red_500.b, 68);
    }

    #[test]
    fn test_shade_to_rgb() {
        let palette = ColorPalette::load().expect("Failed to load palette");
        let blue = palette.color_at(7).expect("Blue should exist");
        let blue_500 = blue.primary_shade().expect("Blue-500 should exist");
        
        let rgb = blue_500.to_rgb();
        assert_eq!(rgb.r, 59);
        assert_eq!(rgb.g, 130);
        assert_eq!(rgb.b, 246);
    }

    #[test]
    fn test_palette_layout() {
        let palette = ColorPalette::load().expect("Failed to load palette");
        assert_eq!(palette.columns(), 4);
        assert_eq!(palette.rows(), 3); // 12 colors / 4 columns = 3 rows
    }
}
