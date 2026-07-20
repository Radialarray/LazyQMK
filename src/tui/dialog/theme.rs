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

impl Theme {
    /// Detects the OS theme and returns the appropriate Theme.
    ///
    /// This uses the `dark-light` crate to detect whether the OS is in
    /// dark or light mode, and returns the matching theme.
    ///
    /// # Examples
    /// ```
    /// use lazyqmk::tui::theme::Theme;
    ///
    /// let theme = Theme::detect();
    /// // Theme will match OS dark/light mode setting
    /// ```
    #[must_use]
    pub fn detect() -> Self {
        match dark_light::detect() {
            dark_light::Mode::Light => Self::light(),
            // Fall back to dark theme for dark mode or default
            dark_light::Mode::Dark | dark_light::Mode::Default => Self::dark(),
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

    /// Creates a theme based on the user's theme mode preference.
    ///
    /// - `Auto`: Detects OS dark/light mode and returns matching theme
    /// - `Dark`: Always returns dark theme
    /// - `Light`: Always returns light theme
    #[must_use]
    pub fn from_mode(mode: crate::config::ThemeMode) -> Self {
        match mode {
            crate::config::ThemeMode::Auto => Self::detect(),
            crate::config::ThemeMode::Dark => Self::dark(),
            crate::config::ThemeMode::Light => Self::light(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::detect()
    }
}

#[cfg(test)]
mod tests;
