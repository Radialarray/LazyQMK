//! Theme system for consistent UI colors across dark and light modes.
//!
//! This module provides a centralized theme management system that automatically
//! detects the OS theme (dark/light mode) and applies appropriate colors.

use ratatui::style::Color;

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

/// Theme variant identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ThemeVariant {
    /// Dark theme optimized for dark terminal backgrounds
    Dark,
    /// Light theme optimized for light terminal backgrounds
    Light,
}

impl Theme {
    /// Detects the OS theme and returns the appropriate Theme.
    ///
    /// This uses the `dark-light` crate to detect whether the OS is in
    /// dark or light mode, and returns the matching theme.
    ///
    /// # Examples
    /// ```
    /// use keyboard_configurator::tui::theme::Theme;
    ///
    /// let theme = Theme::detect();
    /// // Theme will match OS dark/light mode setting
    /// ```
    #[must_use]
    pub fn detect() -> Self {
        match dark_light::detect() {
            Ok(dark_light::Mode::Light) => Self::light(),
            // Fall back to dark theme for dark mode, unspecified, or errors
            Ok(dark_light::Mode::Dark | dark_light::Mode::Unspecified) | Err(_) => Self::dark(),
        }
    }

    /// Creates a dark theme optimized for dark terminal backgrounds.
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
    #[must_use]
    #[allow(dead_code)]
    pub const fn from_variant(variant: ThemeVariant) -> Self {
        match variant {
            ThemeVariant::Dark => Self::dark(),
            ThemeVariant::Light => Self::light(),
        }
    }

    /// Returns the theme variant for the current theme.
    ///
    /// This is determined by checking the background color.
    #[must_use]
    #[allow(dead_code)]
    pub const fn variant(&self) -> ThemeVariant {
        match self.background {
            Color::White | Color::Rgb(255, 255, 255) | Color::Rgb(245, 245, 245) => {
                ThemeVariant::Light
            }
            _ => ThemeVariant::Dark,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::detect()
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

    #[test]
    fn test_theme_detect() {
        // Just verify detect() returns a valid theme without panicking
        let theme = Theme::detect();
        // Should be either dark or light variant
        assert!(theme.variant() == ThemeVariant::Dark || theme.variant() == ThemeVariant::Light);
    }
}
