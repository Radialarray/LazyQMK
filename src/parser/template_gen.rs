//! Markdown layout file generation (serialization).
//!
//! This module handles generating human-readable Markdown files from Layout structures,
//! with atomic file writes for safety.

// Allow format! appended to String - more readable for template generation
#![allow(clippy::format_push_string)]
// Functions kept for migration support (markdown output → JSON)
#![allow(dead_code)]

use crate::models::Layout;
use anyhow::{Context, Result};
use std::path::Path;

/// Generates a Markdown layout file from a Layout structure.
///
/// This performs an atomic write using a temp file + rename pattern to ensure
/// the file is never left in a corrupted state.
///
/// # Errors
///
/// Returns errors for:
/// - File I/O failures
/// - Permission issues
/// - Atomic rename failures
pub fn save_markdown_layout(layout: &Layout, path: &Path) -> Result<()> {
    let markdown = generate_markdown(layout)?;
    atomic_write(path, &markdown)
}

/// Generates Markdown content from a Layout.
pub fn generate_markdown(layout: &Layout) -> Result<String> {
    let mut output = String::new();

    // Generate frontmatter
    output.push_str(&generate_frontmatter(layout)?);
    output.push('\n');

    // Generate title
    output.push_str(&format!("# {}\n\n", layout.metadata.name));

    // Generate layers
    for layer in &layout.layers {
        output.push_str(&generate_layer(layer)?);
        output.push('\n');
    }

    // Generate key descriptions section if any exist
    if let Some(descriptions_section) = generate_key_descriptions(layout) {
        output.push_str("---\n\n");
        output.push_str(&descriptions_section);
    }

    // Generate categories section if any exist
    if !layout.categories.is_empty() {
        // Add separator only if descriptions weren't written
        if has_key_descriptions(layout) {
            output.push('\n');
        } else {
            output.push_str("---\n\n");
        }
        output.push_str(&generate_categories(layout));
    }

    // Generate settings section if any non-default settings exist
    if let Some(settings_section) = generate_settings(layout) {
        // Add separator if neither descriptions nor categories were written
        if !has_key_descriptions(layout) && layout.categories.is_empty() {
            output.push_str("---\n\n");
        } else {
            output.push('\n');
        }
        output.push_str(&settings_section);
    }

    // Generate tap dances section if any exist
    if !layout.tap_dances.is_empty() {
        // Add separator if nothing else was written after layers
        if !has_key_descriptions(layout)
            && layout.categories.is_empty()
            && generate_settings(layout).is_none()
        {
            output.push_str("---\n\n");
        } else {
            output.push('\n');
        }
        output.push_str(&generate_tap_dances(layout));
    }

    Ok(output)
}

/// Generates YAML frontmatter from metadata.
fn generate_frontmatter(layout: &Layout) -> Result<String> {
    let yaml =
        serde_yml::to_string(&layout.metadata).context("Failed to serialize metadata to YAML")?;

    Ok(format!("---\n{yaml}---\n"))
}

/// Generates a layer section with header, properties, and table.
fn generate_layer(layer: &crate::models::Layer) -> Result<String> {
    let mut output = String::new();

    // Layer header: ## Layer N: Name
    output.push_str(&format!("## Layer {}: {}\n", layer.number, layer.name));

    // Layer ID: **ID**: uuid (always write to preserve references)
    output.push_str(&format!("**ID**: {}\n", layer.id));

    // Layer color: **Color**: #RRGGBB
    output.push_str(&format!("**Color**: {}\n", layer.default_color.to_hex()));

    // Optional layer category
    if let Some(cat_id) = &layer.category_id {
        output.push_str(&format!("**Category**: {cat_id}\n"));
    }

    // Layer colors enabled (only write if false, since true is the default)
    if !layer.layer_colors_enabled {
        output.push_str("**Layer Colors**: false\n");
    }

    output.push('\n');

    // Generate table
    output.push_str(&generate_table(layer)?);

    Ok(output)
}

/// Generates a Markdown table for a layer's keys.
#[allow(clippy::unnecessary_wraps)]
fn generate_table(layer: &crate::models::Layer) -> Result<String> {
    use std::collections::HashMap;

    if layer.keys.is_empty() {
        return Ok(String::new());
    }

    // Group keys by row
    let mut rows: HashMap<u8, Vec<_>> = HashMap::new();
    let mut max_col = 0;

    for key in &layer.keys {
        let row = key.position.row;
        let col = key.position.col;

        max_col = max_col.max(col);
        rows.entry(row).or_default().push(key);
    }

    let num_cols = (max_col + 1) as usize;
    let mut row_nums: Vec<_> = rows.keys().copied().collect();
    row_nums.sort_unstable();

    let mut output = String::new();

    // Generate header row
    output.push('|');
    for col in 0..num_cols {
        output.push_str(&format!(" C{col} |"));
    }
    output.push('\n');

    // Generate separator row
    output.push('|');
    for _ in 0..num_cols {
        output.push_str("------|");
    }
    output.push('\n');

    // Generate data rows
    for row_num in row_nums {
        output.push('|');
        let row_keys = rows.get(&row_num).unwrap();

        // Create a map for quick lookup by column
        let mut col_map: HashMap<u8, &crate::models::KeyDefinition> = HashMap::new();
        for key in row_keys {
            col_map.insert(key.position.col, key);
        }

        for col in 0..num_cols {
            if let Some(key) = col_map.get(&(col as u8)) {
                output.push(' ');
                output.push_str(&serialize_keycode_syntax(key));
                output.push_str(" |");
            } else {
                output.push_str("  |"); // Empty cell
            }
        }
        output.push('\n');
    }

    output.push('\n');
    Ok(output)
}

/// Serializes a keycode with optional color and category syntax.
///
/// Formats:
/// - `KC_X` - basic keycode
/// - `KC_X{#RRGGBB}` - with color override
/// - `KC_X@category-id` - with category
/// - `KC_X{#RRGGBB}@category-id` - with both
fn serialize_keycode_syntax(key: &crate::models::KeyDefinition) -> String {
    let mut result = key.keycode.clone();

    // Add color override if present
    if let Some(color) = key.color_override {
        result.push_str(&format!("{{{}}}", color.to_hex()));
    }

    // Add category if present
    if let Some(cat_id) = &key.category_id {
        result.push_str(&format!("@{cat_id}"));
    }

    result
}

/// Generates the categories section.
fn generate_categories(layout: &Layout) -> String {
    let mut output = String::from("## Categories\n\n");

    for category in &layout.categories {
        output.push_str(&format!(
            "- {}: {} ({})\n",
            category.id,
            category.name,
            category.color.to_hex()
        ));
    }

    output
}

/// Checks if any keys in the layout have descriptions.
fn has_key_descriptions(layout: &Layout) -> bool {
    layout
        .layers
        .iter()
        .any(|layer| layer.keys.iter().any(|key| key.description.is_some()))
}

/// Generates the key descriptions section.
/// Format: `- layer:row:col: description text`
fn generate_key_descriptions(layout: &Layout) -> Option<String> {
    // Collect all keys with descriptions
    let descriptions: Vec<_> = layout
        .layers
        .iter()
        .enumerate()
        .flat_map(|(layer_idx, layer)| {
            layer.keys.iter().filter_map(move |key| {
                key.description
                    .as_ref()
                    .map(|desc| (layer_idx, key.position.row, key.position.col, desc.clone()))
            })
        })
        .collect();

    if descriptions.is_empty() {
        return None;
    }

    let mut output = String::from("## Key Descriptions\n\n");

    for (layer_idx, row, col, description) in descriptions {
        output.push_str(&format!("- {layer_idx}:{row}:{col}: {description}\n"));
    }

    Some(output)
}

/// Generates the settings section.
/// Only writes non-default settings to keep files clean.
fn generate_settings(layout: &Layout) -> Option<String> {
    use crate::models::{
        HoldDecisionMode, RgbBrightness, RgbSaturation, TapHoldPreset, TapHoldSettings,
        UncoloredKeyBehavior,
    };

    let default_uncolored = UncoloredKeyBehavior::default();
    let default_tap_hold = TapHoldSettings::default();

    // Check if we have any non-default settings
    let has_rgb_settings = !layout.rgb_enabled
        || layout.rgb_brightness != RgbBrightness::default()
        || layout.rgb_saturation != RgbSaturation::default()
        || layout.rgb_matrix_default_speed != 127
        || layout.rgb_timeout_ms > 0;
    let has_uncolored_setting = layout.uncolored_key_behavior != default_uncolored;
    let has_idle_settings = layout.idle_effect_settings.has_custom_settings();
    let has_ripple_settings = layout.rgb_overlay_ripple.has_custom_settings();
    let has_tap_hold_settings = layout.tap_hold_settings != default_tap_hold;
    let has_combo_settings = layout.combo_settings.has_custom_settings();

    if !has_rgb_settings
        && !has_uncolored_setting
        && !has_idle_settings
        && !has_ripple_settings
        && !has_tap_hold_settings
        && !has_combo_settings
    {
        return None;
    }

    let mut output = String::from("## Settings\n\n");

    // Write RGB settings
    if !layout.rgb_enabled {
        output.push_str("**RGB Enabled**: Off\n");
    }
    if layout.rgb_brightness != RgbBrightness::default() {
        output.push_str(&format!(
            "**RGB Brightness**: {}%\n",
            layout.rgb_brightness.as_percent()
        ));
    }
    if layout.rgb_saturation != RgbSaturation::default() {
        output.push_str(&format!(
            "**RGB Saturation**: {}%\n",
            layout.rgb_saturation.as_percent()
        ));
    }
    if layout.rgb_matrix_default_speed != 127 {
        output.push_str(&format!(
            "**RGB Matrix Speed**: {}\n",
            layout.rgb_matrix_default_speed
        ));
    }

    // Write uncolored_key_behavior if not default
    // Only write if not default (100%)
    if layout.uncolored_key_behavior.as_percent() != 100 {
        output.push_str(&format!(
            "**Uncolored Key Brightness**: {}%\n",
            layout.uncolored_key_behavior.as_percent()
        ));
    }

    // Write RGB timeout if set
    if layout.rgb_timeout_ms > 0 {
        // Convert milliseconds to a human-readable format
        let timeout_ms = layout.rgb_timeout_ms;
        if timeout_ms >= 60000 && timeout_ms.is_multiple_of(60000) {
            // Whole minutes
            output.push_str(&format!("**RGB Timeout**: {} min\n", timeout_ms / 60000));
        } else if timeout_ms >= 1000 && timeout_ms.is_multiple_of(1000) {
            // Whole seconds
            output.push_str(&format!("**RGB Timeout**: {} sec\n", timeout_ms / 1000));
        } else {
            // Milliseconds
            output.push_str(&format!("**RGB Timeout**: {timeout_ms}ms\n"));
        }
    }

    // Write idle effect settings if any are non-default
    if layout.idle_effect_settings.has_custom_settings() {
        let ies = &layout.idle_effect_settings;
        let defaults = crate::models::IdleEffectSettings::default();

        // Write enabled/disabled if not default
        if ies.enabled != defaults.enabled {
            let value = if ies.enabled { "On" } else { "Off" };
            output.push_str(&format!("**Idle Effect**: {value}\n"));
        }

        // Write idle timeout if not default
        if ies.idle_timeout_ms != defaults.idle_timeout_ms {
            let timeout_ms = ies.idle_timeout_ms;
            if timeout_ms == 0 {
                output.push_str("**Idle Timeout**: 0\n");
            } else if timeout_ms >= 60000 && timeout_ms.is_multiple_of(60000) {
                // Whole minutes
                output.push_str(&format!("**Idle Timeout**: {} min\n", timeout_ms / 60000));
            } else if timeout_ms >= 1000 && timeout_ms.is_multiple_of(1000) {
                // Whole seconds
                output.push_str(&format!("**Idle Timeout**: {} sec\n", timeout_ms / 1000));
            } else {
                // Milliseconds
                output.push_str(&format!("**Idle Timeout**: {timeout_ms}ms\n"));
            }
        }

        // Write idle effect duration if not default
        if ies.idle_effect_duration_ms != defaults.idle_effect_duration_ms {
            let duration_ms = ies.idle_effect_duration_ms;
            if duration_ms == 0 {
                output.push_str("**Idle Effect Duration**: 0\n");
            } else if duration_ms >= 60000 && duration_ms.is_multiple_of(60000) {
                // Whole minutes
                output.push_str(&format!(
                    "**Idle Effect Duration**: {} min\n",
                    duration_ms / 60000
                ));
            } else if duration_ms >= 1000 && duration_ms.is_multiple_of(1000) {
                // Whole seconds
                output.push_str(&format!(
                    "**Idle Effect Duration**: {} sec\n",
                    duration_ms / 1000
                ));
            } else {
                // Milliseconds
                output.push_str(&format!("**Idle Effect Duration**: {duration_ms}ms\n"));
            }
        }

        // Write idle effect mode if not default
        if ies.idle_effect_mode != defaults.idle_effect_mode {
            output.push_str(&format!(
                "**Idle Effect Mode**: {}\n",
                ies.idle_effect_mode.display_name()
            ));
        }
    }

    // Write ripple overlay settings if any are non-default
    if layout.rgb_overlay_ripple.has_custom_settings() {
        use crate::models::RippleColorMode;
        let rip = &layout.rgb_overlay_ripple;
        let defaults = crate::models::RgbOverlayRippleSettings::default();

        // Write enabled/disabled if not default
        if rip.enabled != defaults.enabled {
            let value = if rip.enabled { "On" } else { "Off" };
            output.push_str(&format!("**Ripple Overlay**: {value}\n"));
        }

        // Write max ripples if not default
        if rip.max_ripples != defaults.max_ripples {
            output.push_str(&format!("**Max Ripples**: {}\n", rip.max_ripples));
        }

        // Write duration if not default
        if rip.duration_ms != defaults.duration_ms {
            output.push_str(&format!("**Ripple Duration**: {}ms\n", rip.duration_ms));
        }

        // Write speed if not default
        if rip.speed != defaults.speed {
            output.push_str(&format!("**Ripple Speed**: {}\n", rip.speed));
        }

        // Write band width if not default
        if rip.band_width != defaults.band_width {
            output.push_str(&format!("**Ripple Band Width**: {}\n", rip.band_width));
        }

        // Write amplitude if not default
        if rip.amplitude_pct != defaults.amplitude_pct {
            output.push_str(&format!("**Ripple Amplitude**: {}%\n", rip.amplitude_pct));
        }

        // Write color mode if not default
        if rip.color_mode != defaults.color_mode {
            let mode_name = match rip.color_mode {
                RippleColorMode::Fixed => "Fixed Color",
                RippleColorMode::KeyBased => "Key Color",
                RippleColorMode::HueShift => "Hue Shift",
            };
            output.push_str(&format!("**Ripple Color Mode**: {mode_name}\n"));
        }

        // Write fixed color if not default
        if rip.fixed_color != defaults.fixed_color {
            output.push_str(&format!(
                "**Ripple Fixed Color**: {}\n",
                rip.fixed_color.to_hex()
            ));
        }

        // Write hue shift if not default
        if rip.hue_shift_deg != defaults.hue_shift_deg {
            output.push_str(&format!("**Ripple Hue Shift**: {}°\n", rip.hue_shift_deg));
        }

        // Write trigger on press if not default
        if rip.trigger_on_press != defaults.trigger_on_press {
            let value = if rip.trigger_on_press { "On" } else { "Off" };
            output.push_str(&format!("**Ripple Trigger on Press**: {value}\n"));
        }

        // Write trigger on release if not default
        if rip.trigger_on_release != defaults.trigger_on_release {
            let value = if rip.trigger_on_release { "On" } else { "Off" };
            output.push_str(&format!("**Ripple Trigger on Release**: {value}\n"));
        }

        // Write ignore transparent if not default
        if rip.ignore_transparent != defaults.ignore_transparent {
            let value = if rip.ignore_transparent { "On" } else { "Off" };
            output.push_str(&format!("**Ripple Ignore Transparent**: {value}\n"));
        }

        // Write ignore modifiers if not default
        if rip.ignore_modifiers != defaults.ignore_modifiers {
            let value = if rip.ignore_modifiers { "On" } else { "Off" };
            output.push_str(&format!("**Ripple Ignore Modifiers**: {value}\n"));
        }

        // Write ignore layer switch if not default
        if rip.ignore_layer_switch != defaults.ignore_layer_switch {
            let value = if rip.ignore_layer_switch { "On" } else { "Off" };
            output.push_str(&format!("**Ripple Ignore Layer Switch**: {value}\n"));
        }

        // Write wave count if not default
        if rip.wave_count != defaults.wave_count {
            output.push_str(&format!("**Ripple Wave Count**: {}\n", rip.wave_count));
        }

        // Write wave delay if not default
        if rip.wave_delay_ms != defaults.wave_delay_ms {
            output.push_str(&format!("**Ripple Wave Delay**: {}ms\n", rip.wave_delay_ms));
        }

        // Write key action palette if set
        if let Some(palette) = &rip.key_action_palette {
            let palette_name = palette.display_name();
            output.push_str(&format!("**Ripple Key Action Palette**: {palette_name}\n"));
        }
    }

    // Write tap-hold settings if any are non-default
    if has_tap_hold_settings {
        let ths = &layout.tap_hold_settings;

        // Always write preset if not Default
        if ths.preset != TapHoldPreset::Default {
            let preset_name = match ths.preset {
                TapHoldPreset::Default => "Default",
                TapHoldPreset::HomeRowMods => "Home Row Mods",
                TapHoldPreset::Responsive => "Responsive",
                TapHoldPreset::Deliberate => "Deliberate",
                TapHoldPreset::Custom => "Custom",
            };
            output.push_str(&format!("**Tap-Hold Preset**: {preset_name}\n"));
        }

        // Write individual settings that differ from default
        if ths.tapping_term != default_tap_hold.tapping_term {
            output.push_str(&format!("**Tapping Term**: {}ms\n", ths.tapping_term));
        }

        if ths.quick_tap_term != default_tap_hold.quick_tap_term {
            match ths.quick_tap_term {
                Some(term) => output.push_str(&format!("**Quick Tap Term**: {term}ms\n")),
                None => output.push_str("**Quick Tap Term**: Auto\n"),
            }
        }

        if ths.hold_mode != default_tap_hold.hold_mode {
            let mode_name = match ths.hold_mode {
                HoldDecisionMode::Default => "Default",
                HoldDecisionMode::PermissiveHold => "Permissive Hold",
                HoldDecisionMode::HoldOnOtherKeyPress => "Hold On Other Key Press",
            };
            output.push_str(&format!("**Hold Mode**: {mode_name}\n"));
        }

        if ths.retro_tapping != default_tap_hold.retro_tapping {
            let value = if ths.retro_tapping { "On" } else { "Off" };
            output.push_str(&format!("**Retro Tapping**: {value}\n"));
        }

        if ths.tapping_toggle != default_tap_hold.tapping_toggle {
            output.push_str(&format!(
                "**Tapping Toggle**: {} taps\n",
                ths.tapping_toggle
            ));
        }

        if ths.flow_tap_term != default_tap_hold.flow_tap_term {
            match ths.flow_tap_term {
                Some(term) => output.push_str(&format!("**Flow Tap Term**: {term}ms\n")),
                None => output.push_str("**Flow Tap Term**: Disabled\n"),
            }
        }

        if ths.chordal_hold != default_tap_hold.chordal_hold {
            let value = if ths.chordal_hold { "On" } else { "Off" };
            output.push_str(&format!("**Chordal Hold**: {value}\n"));
        }
    }

    // Write combo settings if any are non-default
    if has_combo_settings {
        let cs = &layout.combo_settings;

        // Write enabled/disabled
        if cs.enabled {
            output.push_str("**Combos**: On\n");
        }

        // Write individual combo definitions
        for (idx, combo) in cs.combos.iter().enumerate() {
            let combo_num = idx + 1;
            let action_name = combo.action.display_name();

            // Format: **Combo N**: (row1,col1)+(row2,col2) → Action [duration]
            output.push_str(&format!(
                "**Combo {}**: ({},{})+({},{}) → {} [{}ms]\n",
                combo_num,
                combo.key1.row,
                combo.key1.col,
                combo.key2.row,
                combo.key2.col,
                action_name,
                combo.hold_duration_ms
            ));
        }
    }

    Some(output)
}

/// Generates the tap dances section.
fn generate_tap_dances(layout: &Layout) -> String {
    let mut output = String::from("## Tap Dances\n\n");

    for td in &layout.tap_dances {
        output.push_str(&format!("- **{}**:\n", td.name));
        output.push_str(&format!("  - Single Tap: {}\n", td.single_tap));
        if let Some(ref double) = td.double_tap {
            output.push_str(&format!("  - Double Tap: {double}\n"));
        }
        if let Some(ref hold) = td.hold {
            output.push_str(&format!("  - Hold: {hold}\n"));
        }
    }

    output
}

/// Performs an atomic file write using temp file + rename pattern.
///
/// This ensures the target file is never left in a corrupted state:
/// 1. Write to temporary file
/// 2. Verify write success
/// 3. Atomic rename to target path
fn atomic_write(path: &Path, content: &str) -> Result<()> {
    // Create temporary file path
    let temp_path = path.with_extension("md.tmp");

    // Write to temp file
    std::fs::write(&temp_path, content)
        .with_context(|| format!("Failed to write to temporary file: {}", temp_path.display()))?;

    // Atomic rename
    std::fs::rename(&temp_path, path)
        .with_context(|| format!("Failed to rename temporary file to: {}", path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests;

