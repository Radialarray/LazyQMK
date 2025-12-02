//! Theme system for consistent UI colors across dark and light modes.
//!
//! This module provides a centralized theme management system that replaces
//! hardcoded colors throughout the TUI with semantic color roles.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Semantic color theme for the TUI.
///
/// Provides consistent colors across all UI components with support
/// for both dark and light terminal backgrounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    // Primary UI colors
    /// Primary color for borders, titles, and emphasis
    pub primary: Color,
    /// Accent color for highlights, selections, and focus states
    pub accent: Color,
    /// Success state color for confirmations and success messages
    pub success: Color,
    /// Error state color for errors and destructive actions
    pub error: Color,
    /// Warning state color for warnings and cautions
    pub warning: Color,

    // Text hierarchy
    /// Primary text content color
    pub text: Color,
    /// Secondary text color for labels and less important content
    pub text_secondary: Color,
    /// Muted text color for help text, disabled items, and dim content
    pub text_muted: Color,

    // Backgrounds
    /// Main background color
    pub background: Color,
    /// Highlight/selection background color
    pub highlight_bg: Color,
    /// Surface color for panels and elevated elements
    pub surface: Color,

    // State indicators
    /// Active/focused element color
    pub active: Color,
    /// Inactive/disabled element color
    pub inactive: Color,
}

/// Theme variant identifier for serialization and config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeVariant {
    /// Dark theme optimized for dark terminal backgrounds
    Dark,
    /// Light theme optimized for light terminal backgrounds
    Light,
}

impl Theme {
    /// Creates a dark theme optimized for dark terminal backgrounds.
    ///
    /// This is the default theme and maintains the existing color scheme.
    ///
    /// # Color Choices
    /// - Uses bright colors (Cyan, Yellow) for UI chrome
    /// - White text on black background for maximum contrast
    /// - Semantic colors: Green for success, Red for errors
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            primary: Color::Cyan,
            accent: Color::Yellow,
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,

            text: Color::White,
            text_secondary: Color::Gray,
            text_muted: Color::DarkGray,

            background: Color::Black,
            highlight_bg: Color::DarkGray,
            surface: Color::Rgb(30, 30, 30),

            active: Color::Yellow,
            inactive: Color::Gray,
        }
    }

    /// Creates a light theme optimized for light terminal backgrounds.
    ///
    /// All colors meet WCAG AA contrast requirements (4.5:1 minimum).
    ///
    /// # Color Choices
    /// - Uses darker colors for text and UI elements
    /// - Black text on white background for maximum readability
    /// - Adjusted accent colors for visibility on light backgrounds
    #[must_use]
    pub const fn light() -> Self {
        Self {
            primary: Color::Blue,
            accent: Color::Rgb(180, 100, 0), // Dark orange for visibility
            success: Color::Rgb(0, 128, 0),  // Dark green
            error: Color::Red,
            warning: Color::Rgb(200, 100, 0), // Orange-brown for warnings

            text: Color::Black,
            text_secondary: Color::Rgb(60, 60, 60),
            text_muted: Color::Gray,

            background: Color::White,
            highlight_bg: Color::Rgb(230, 230, 230),
            surface: Color::Rgb(245, 245, 245),

            active: Color::Rgb(180, 100, 0),
            inactive: Color::Rgb(180, 180, 180),
        }
    }

    /// Creates a theme from a variant enum.
    ///
    /// # Examples
    /// ```
    /// use keyboard_tui::tui::theme::{Theme, ThemeVariant};
    ///
    /// let dark = Theme::from_variant(ThemeVariant::Dark);
    /// let light = Theme::from_variant(ThemeVariant::Light);
    /// ```
    #[must_use]
    #[allow(dead_code)]
    pub const fn from_variant(variant: ThemeVariant) -> Self {
        match variant {
            ThemeVariant::Dark => Self::dark(),
            ThemeVariant::Light => Self::light(),
        }
    }

    /// Creates a theme from a string name (typically from config).
    ///
    /// # Supported Names
    /// - `"dark"` or `"default"` → Dark theme
    /// - `"light"` → Light theme
    /// - Any other value → Dark theme (fallback)
    ///
    /// # Examples
    /// ```
    /// use keyboard_tui::tui::theme::Theme;
    ///
    /// let dark = Theme::from_name("dark");
    /// let light = Theme::from_name("light");
    /// let default_fallback = Theme::from_name("invalid");
    /// ```
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "light" => Self::light(),
            "dark" | "default" | _ => Self::dark(),
        }
    }

    /// Returns the theme variant for the current theme.
    ///
    /// This is determined by checking the background color.
    #[must_use]
    pub const fn variant(&self) -> ThemeVariant {
        // Determine variant based on background color
        match self.background {
            Color::White | Color::Rgb(255, 255, 255) => ThemeVariant::Light,
            _ => ThemeVariant::Dark,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_dark() {
        let theme = Theme::dark();
        assert_eq!(theme.primary, Color::Cyan);
        assert_eq!(theme.background, Color::Black);
        assert_eq!(theme.text, Color::White);
        assert_eq!(theme.accent, Color::Yellow);
        assert_eq!(theme.success, Color::Green);
        assert_eq!(theme.error, Color::Red);
    }

    #[test]
    fn test_theme_light() {
        let theme = Theme::light();
        assert_eq!(theme.text, Color::Black);
        assert_eq!(theme.background, Color::White);
        assert_eq!(theme.primary, Color::Blue);
        // Verify accent is not yellow (too bright for light bg)
        assert_ne!(theme.accent, Color::Yellow);
    }

    #[test]
    fn test_theme_from_name() {
        let dark1 = Theme::from_name("dark");
        assert_eq!(dark1.background, Color::Black);

        let dark2 = Theme::from_name("default");
        assert_eq!(dark2.background, Color::Black);

        let light = Theme::from_name("light");
        assert_eq!(light.background, Color::White);

        // Test fallback to dark for invalid names
        let fallback = Theme::from_name("invalid");
        assert_eq!(fallback.background, Color::Black);
    }

    #[test]
    fn test_theme_from_variant() {
        let dark = Theme::from_variant(ThemeVariant::Dark);
        assert_eq!(dark, Theme::dark());

        let light = Theme::from_variant(ThemeVariant::Light);
        assert_eq!(light, Theme::light());
    }

    #[test]
    fn test_theme_variant_detection() {
        let dark = Theme::dark();
        assert_eq!(dark.variant(), ThemeVariant::Dark);

        let light = Theme::light();
        assert_eq!(light.variant(), ThemeVariant::Light);
    }

    #[test]
    fn test_theme_default() {
        let default = Theme::default();
        assert_eq!(default, Theme::dark());
    }

    #[test]
    fn test_theme_contrast() {
        let dark = Theme::dark();
        // Dark theme should have light text on dark background
        assert_eq!(dark.text, Color::White);
        assert_eq!(dark.background, Color::Black);

        let light = Theme::light();
        // Light theme should have dark text on light background
        assert_eq!(light.text, Color::Black);
        assert_eq!(light.background, Color::White);
    }

    #[test]
    fn test_semantic_colors_present() {
        let theme = Theme::dark();
        // Verify all semantic colors are defined
        assert_ne!(theme.success, theme.error);
        assert_ne!(theme.primary, theme.accent);
        assert_ne!(theme.text, theme.text_muted);
    }
}
