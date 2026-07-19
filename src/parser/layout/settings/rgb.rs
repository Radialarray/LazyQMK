//! Basic RGB settings: brightness, saturation, matrix default speed.

use crate::models::Layout;

/// Parses RGB Brightness (percent).
pub(super) fn try_parse_rgb_brightness(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**RGB Brightness**:") {
        return false;
    }
    let value = line
        .strip_prefix("**RGB Brightness**:")
        .unwrap()
        .trim()
        .trim_end_matches('%');
    if let Ok(percent) = value.parse::<u8>() {
        layout.rgb_brightness = crate::models::RgbBrightness::from(percent);
    }
    true
}

/// Parses RGB Saturation (percent).
pub(super) fn try_parse_rgb_saturation(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**RGB Saturation**:") {
        return false;
    }
    let value = line
        .strip_prefix("**RGB Saturation**:")
        .unwrap()
        .trim()
        .trim_end_matches('%');
    if let Ok(percent) = value.parse::<u8>() {
        layout.rgb_saturation = crate::models::RgbSaturation::from(percent);
    }
    true
}

/// Parses RGB Matrix Default Speed (0–255).
pub(super) fn try_parse_rgb_matrix_speed(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**RGB Matrix Speed**:") {
        return false;
    }
    let value = line.strip_prefix("**RGB Matrix Speed**:").unwrap().trim();
    if let Ok(speed) = value.parse::<u8>() {
        layout.rgb_matrix_default_speed = speed;
    }
    true
}
