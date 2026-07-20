//! Tests for theme.
//!
//! Auto-extracted from theme.rs.

use super::*;

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
fn test_theme_constructors() {
    // Test that dark() and light() create distinct themes
    let dark = Theme::dark();
    let light = Theme::light();
    assert_ne!(dark, light);
    assert_ne!(dark.background, light.background);
    assert_ne!(dark.text, light.text);
}

#[test]
fn test_theme_variants() {
    // Test that dark and light themes have expected characteristics
    let dark = Theme::dark();
    assert_eq!(dark.background, Color::Black);
    assert_eq!(dark.text, Color::White);

    let light = Theme::light();
    assert_eq!(light.background, Color::White);
    assert_eq!(light.text, Color::Black);
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
    let _theme = Theme::detect();
    // Test passes if detect() doesn't panic
}
