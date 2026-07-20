//! Tests for color_palette.
//!
//! Auto-extracted from color_palette.rs.

use super::*;

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

#[test]
fn test_get_color_by_name() {
    let palette = ColorPalette::load().expect("Failed to load palette");

    let gray = palette.get_color("Gray").expect("Gray should exist");
    assert_eq!(gray.name, "Gray");

    // Case-insensitive
    let blue = palette.get_color("blue").expect("blue should find Blue");
    assert_eq!(blue.name, "Blue");

    assert!(palette.get_color("nonexistent").is_none());
}

#[test]
fn test_get_shade() {
    let palette = ColorPalette::load().expect("Failed to load palette");

    let gray_500 = palette
        .get_shade("Gray", 500)
        .expect("Gray-500 should exist");
    assert_eq!(gray_500.level, 500);
    assert_eq!(gray_500.r, 107);
    assert_eq!(gray_500.g, 114);
    assert_eq!(gray_500.b, 128);

    assert!(palette.get_shade("Gray", 999).is_none());
    assert!(palette.get_shade("nonexistent", 500).is_none());
}

#[test]
fn test_default_layer_color() {
    let palette = ColorPalette::load().expect("Failed to load palette");
    let default = palette.default_layer_color();

    // Should be Gray-500
    assert_eq!(default.r, 107);
    assert_eq!(default.g, 114);
    assert_eq!(default.b, 128);
}
