//! Tests for rgb.
//!
//! Auto-extracted from rgb.rs.

use super::*;

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
