//! PaletteFX settings.

use crate::models::Layout;

/// Parses PaletteFX enabled/disabled.
pub(super) fn try_parse_palette_fx_enabled(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**PaletteFX**:") {
        return false;
    }
    let value = line
        .strip_prefix("**PaletteFX**:")
        .unwrap()
        .trim()
        .to_lowercase();
    layout.palette_fx.enabled = matches!(value.as_str(), "on" | "true" | "yes" | "enabled");
    true
}

/// Parses PaletteFX Default Effect.
pub(super) fn try_parse_palette_fx_default_effect(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**PaletteFX Default Effect**:") {
        return false;
    }
    let value = line
        .strip_prefix("**PaletteFX Default Effect**:")
        .unwrap()
        .trim();
    if let Some(effect) = crate::models::PaletteFxEffect::from_name(value) {
        layout.palette_fx.default_effect = effect;
    }
    true
}

/// Parses PaletteFX Default Palette.
pub(super) fn try_parse_palette_fx_default_palette(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**PaletteFX Default Palette**:") {
        return false;
    }
    let value = line
        .strip_prefix("**PaletteFX Default Palette**:")
        .unwrap()
        .trim();
    if let Some(palette) = crate::models::PaletteFxPalette::from_name(value) {
        layout.palette_fx.default_palette = palette;
    }
    true
}

/// Parses PaletteFX All Effects.
pub(super) fn try_parse_palette_fx_all_effects(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**PaletteFX All Effects**:") {
        return false;
    }
    let value = line
        .strip_prefix("**PaletteFX All Effects**:")
        .unwrap()
        .trim()
        .to_lowercase();
    layout.palette_fx.enable_all_effects =
        matches!(value.as_str(), "on" | "true" | "yes" | "enabled");
    true
}

/// Parses PaletteFX All Palettes.
pub(super) fn try_parse_palette_fx_all_palettes(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**PaletteFX All Palettes**:") {
        return false;
    }
    let value = line
        .strip_prefix("**PaletteFX All Palettes**:")
        .unwrap()
        .trim()
        .to_lowercase();
    layout.palette_fx.enable_all_palettes =
        matches!(value.as_str(), "on" | "true" | "yes" | "enabled");
    true
}
