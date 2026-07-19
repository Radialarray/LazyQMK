//! RGB Overlay Ripple settings.

use crate::models::{Layout, RgbColor};

/// Parses Ripple Overlay enabled/disabled.
pub(super) fn try_parse_ripple_overlay_enabled(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Overlay**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Ripple Overlay**:")
        .unwrap()
        .trim()
        .to_lowercase();
    layout.rgb_overlay_ripple.enabled = matches!(value.as_str(), "on" | "true" | "yes" | "enabled");
    true
}

/// Parses Max Ripples (count).
pub(super) fn try_parse_max_ripples(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Max Ripples**:") {
        return false;
    }
    let value = line.strip_prefix("**Max Ripples**:").unwrap().trim();
    if let Ok(count) = value.parse::<u8>() {
        layout.rgb_overlay_ripple.max_ripples = count;
    }
    true
}

/// Parses Ripple Duration (ms).
pub(super) fn try_parse_ripple_duration(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Duration**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Ripple Duration**:")
        .unwrap()
        .trim()
        .trim_end_matches("ms")
        .trim();
    if let Ok(duration) = value.parse::<u16>() {
        layout.rgb_overlay_ripple.duration_ms = duration;
    }
    true
}

/// Parses Ripple Speed (0–255).
pub(super) fn try_parse_ripple_speed(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Speed**:") {
        return false;
    }
    let value = line.strip_prefix("**Ripple Speed**:").unwrap().trim();
    if let Ok(speed) = value.parse::<u8>() {
        layout.rgb_overlay_ripple.speed = speed;
    }
    true
}

/// Parses Ripple Band Width (0–255).
pub(super) fn try_parse_ripple_band_width(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Band Width**:") {
        return false;
    }
    let value = line.strip_prefix("**Ripple Band Width**:").unwrap().trim();
    if let Ok(width) = value.parse::<u8>() {
        layout.rgb_overlay_ripple.band_width = width;
    }
    true
}

/// Parses Ripple Amplitude (percent).
pub(super) fn try_parse_ripple_amplitude(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Amplitude**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Ripple Amplitude**:")
        .unwrap()
        .trim()
        .trim_end_matches('%')
        .trim();
    if let Ok(amp) = value.parse::<u8>() {
        layout.rgb_overlay_ripple.amplitude_pct = amp;
    }
    true
}

/// Parses Ripple Color Mode.
pub(super) fn try_parse_ripple_color_mode(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Color Mode**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Ripple Color Mode**:")
        .unwrap()
        .trim()
        .to_lowercase();
    layout.rgb_overlay_ripple.color_mode = match value.as_str() {
        "fixed" | "fixed color" => crate::models::RippleColorMode::Fixed,
        "key based" | "key-based" | "key color" => crate::models::RippleColorMode::KeyBased,
        "hue shift" | "hue-shift" => crate::models::RippleColorMode::HueShift,
        _ => crate::models::RippleColorMode::Fixed,
    };
    true
}

/// Parses Ripple Fixed Color (hex).
pub(super) fn try_parse_ripple_fixed_color(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Fixed Color**:") {
        return false;
    }
    let color_str = line.strip_prefix("**Ripple Fixed Color**:").unwrap().trim();
    if let Ok(color) = RgbColor::from_hex(color_str) {
        layout.rgb_overlay_ripple.fixed_color = color;
    }
    true
}

/// Parses Ripple Hue Shift (degrees).
pub(super) fn try_parse_ripple_hue_shift(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Hue Shift**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Ripple Hue Shift**:")
        .unwrap()
        .trim()
        .trim_end_matches('°')
        .trim();
    if let Ok(shift) = value.parse::<i16>() {
        layout.rgb_overlay_ripple.hue_shift_deg = shift;
    }
    true
}

/// Parses Ripple Trigger on Press (on/off).
pub(super) fn try_parse_ripple_trigger_on_press(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Trigger on Press**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Ripple Trigger on Press**:")
        .unwrap()
        .trim()
        .to_lowercase();
    layout.rgb_overlay_ripple.trigger_on_press =
        matches!(value.as_str(), "on" | "true" | "yes" | "enabled");
    true
}

/// Parses Ripple Trigger on Release (on/off).
pub(super) fn try_parse_ripple_trigger_on_release(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Trigger on Release**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Ripple Trigger on Release**:")
        .unwrap()
        .trim()
        .to_lowercase();
    layout.rgb_overlay_ripple.trigger_on_release =
        matches!(value.as_str(), "on" | "true" | "yes" | "enabled");
    true
}

/// Parses Ripple Ignore Transparent (on/off).
pub(super) fn try_parse_ripple_ignore_transparent(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Ignore Transparent**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Ripple Ignore Transparent**:")
        .unwrap()
        .trim()
        .to_lowercase();
    layout.rgb_overlay_ripple.ignore_transparent =
        matches!(value.as_str(), "on" | "true" | "yes" | "enabled");
    true
}

/// Parses Ripple Ignore Modifiers (on/off).
pub(super) fn try_parse_ripple_ignore_modifiers(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Ignore Modifiers**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Ripple Ignore Modifiers**:")
        .unwrap()
        .trim()
        .to_lowercase();
    layout.rgb_overlay_ripple.ignore_modifiers =
        matches!(value.as_str(), "on" | "true" | "yes" | "enabled");
    true
}

/// Parses Ripple Ignore Layer Switch (on/off).
pub(super) fn try_parse_ripple_ignore_layer_switch(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Ignore Layer Switch**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Ripple Ignore Layer Switch**:")
        .unwrap()
        .trim()
        .to_lowercase();
    layout.rgb_overlay_ripple.ignore_layer_switch =
        matches!(value.as_str(), "on" | "true" | "yes" | "enabled");
    true
}

/// Parses Ripple Wave Count.
pub(super) fn try_parse_ripple_wave_count(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Wave Count**:") {
        return false;
    }
    let value = line.strip_prefix("**Ripple Wave Count**:").unwrap().trim();
    if let Ok(count) = value.parse::<u8>() {
        layout.rgb_overlay_ripple.wave_count = count;
    }
    true
}

/// Parses Ripple Wave Delay (ms).
pub(super) fn try_parse_ripple_wave_delay(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Wave Delay**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Ripple Wave Delay**:")
        .unwrap()
        .trim()
        .trim_end_matches("ms")
        .trim();
    if let Ok(delay) = value.parse::<u16>() {
        layout.rgb_overlay_ripple.wave_delay_ms = delay;
    }
    true
}

/// Parses Ripple Key Action Palette (PaletteFX palette name, "none", or "default").
pub(super) fn try_parse_ripple_key_action_palette(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Ripple Key Action Palette**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Ripple Key Action Palette**:")
        .unwrap()
        .trim();
    if value.eq_ignore_ascii_case("none") || value.eq_ignore_ascii_case("default") {
        layout.rgb_overlay_ripple.key_action_palette = None;
    } else if let Some(palette) = crate::models::PaletteFxPalette::from_name(value) {
        layout.rgb_overlay_ripple.key_action_palette = Some(palette);
    }
    true
}
