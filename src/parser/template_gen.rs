//! Markdown layout file generation (serialization).
//!
//! This module handles generating human-readable Markdown files from Layout structures,
//! with atomic file writes for safety.

// Allow format! appended to String - more readable for template generation
#![allow(clippy::format_push_string)]
// Allow intentional type casts
#![allow(clippy::cast_possible_truncation)]

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

    if !has_rgb_settings
        && !has_uncolored_setting
        && !has_idle_settings
        && !has_ripple_settings
        && !has_tap_hold_settings
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
                RippleColorMode::Fixed => "Fixed",
                RippleColorMode::KeyBased => "Key Based",
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
mod tests {
    use super::*;
    use crate::models::{
        Category, ColorPalette, KeyDefinition, Layer, LayoutMetadata, Position, RgbColor,
    };
    use crate::parser::layout::parse_markdown_layout_str;
    use chrono::Utc;

    fn create_test_layout() -> Layout {
        let metadata = LayoutMetadata {
            name: "Test Layout".to_string(),
            description: "A test layout".to_string(),
            author: "test".to_string(),
            created: Utc::now(),
            modified: Utc::now(),
            tags: vec!["test".to_string()],
            is_template: false,
            version: "1.0".to_string(),
            layout_variant: None,
            keyboard: None,
            keymap_name: None,
            output_format: None,
        };

        let mut layer = Layer {
            id: "test-layer-0".to_string(),
            number: 0,
            name: "Base".to_string(),
            default_color: ColorPalette::load()
                .unwrap_or_default()
                .default_layer_color(),
            category_id: None,
            keys: vec![],
            layer_colors_enabled: true,
        };

        // Add some keys
        layer.keys.push(KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_A".to_string(),
            label: None,
            color_override: None,
            category_id: None,
            combo_participant: false,
            description: None,
        });

        layer.keys.push(KeyDefinition {
            position: Position { row: 0, col: 1 },
            keycode: "KC_B".to_string(),
            label: None,
            color_override: Some(RgbColor::new(255, 0, 0)),
            category_id: None,
            combo_participant: false,
            description: None,
        });

        let category = Category {
            id: "navigation".to_string(),
            name: "Navigation".to_string(),
            color: RgbColor::new(0, 0, 255),
        };

        Layout {
            metadata,
            layers: vec![layer],
            categories: vec![category],
            rgb_enabled: true,
            rgb_brightness: crate::models::RgbBrightness::default(),
            rgb_saturation: crate::models::RgbSaturation::default(),
            rgb_matrix_default_speed: 127,
            rgb_timeout_ms: 0,
            uncolored_key_behavior: crate::models::UncoloredKeyBehavior::default(),
            idle_effect_settings: crate::models::IdleEffectSettings::default(),
            rgb_overlay_ripple: crate::models::RgbOverlayRippleSettings::default(),
            tap_hold_settings: crate::models::TapHoldSettings::default(),
            tap_dances: vec![],
        }
    }

    #[test]
    fn test_generate_frontmatter() {
        let layout = create_test_layout();
        let frontmatter = generate_frontmatter(&layout).unwrap();

        println!("Generated frontmatter:\n{frontmatter}");

        assert!(frontmatter.starts_with("---\n"));
        assert!(frontmatter.ends_with("---\n"));
        // YAML may use single quotes or no quotes depending on content
        assert!(frontmatter.contains("name:") && frontmatter.contains("Test Layout"));
        assert!(frontmatter.contains("version:") && frontmatter.contains("1.0"));
    }

    #[test]
    fn test_serialize_keycode_syntax() {
        // Basic keycode
        let key = KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_A".to_string(),
            label: None,
            color_override: None,
            category_id: None,
            combo_participant: false,
            description: None,
        };
        assert_eq!(serialize_keycode_syntax(&key), "KC_A");

        // With color override
        let key_with_color = KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_A".to_string(),
            label: None,
            color_override: Some(RgbColor::new(255, 0, 0)),
            category_id: None,
            combo_participant: false,
            description: None,
        };
        assert_eq!(serialize_keycode_syntax(&key_with_color), "KC_A{#FF0000}");

        // With category
        let key_with_category = KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_LEFT".to_string(),
            label: None,
            color_override: None,
            category_id: Some("navigation".to_string()),
            combo_participant: false,
            description: None,
        };
        assert_eq!(
            serialize_keycode_syntax(&key_with_category),
            "KC_LEFT@navigation"
        );

        // With both
        let key_with_both = KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_A".to_string(),
            label: None,
            color_override: Some(RgbColor::new(0, 255, 0)),
            category_id: Some("symbols".to_string()),
            combo_participant: false,
            description: None,
        };
        assert_eq!(
            serialize_keycode_syntax(&key_with_both),
            "KC_A{#00FF00}@symbols"
        );
    }

    #[test]
    fn test_generate_categories() {
        let layout = create_test_layout();
        let categories_section = generate_categories(&layout);

        assert!(categories_section.contains("## Categories"));
        assert!(categories_section.contains("- navigation: Navigation (#0000FF)"));
    }

    #[test]
    fn test_round_trip() {
        let layout = create_test_layout();

        // Generate markdown
        let markdown = generate_markdown(&layout).unwrap();

        println!("Generated markdown:\n{markdown}");

        // Parse it back
        let parsed_layout = parse_markdown_layout_str(&markdown).unwrap();

        // Verify key data is preserved
        assert_eq!(parsed_layout.metadata.name, layout.metadata.name);
        assert_eq!(parsed_layout.layers.len(), layout.layers.len());
        println!("Original categories: {}", layout.categories.len());
        println!("Parsed categories: {}", parsed_layout.categories.len());
        assert_eq!(parsed_layout.categories.len(), layout.categories.len());
        assert_eq!(
            parsed_layout.layers[0].keys.len(),
            layout.layers[0].keys.len()
        );
    }

    #[test]
    fn test_settings_round_trip() {
        use crate::models::UncoloredKeyBehavior;

        let mut layout = create_test_layout();

        // Test with Off setting (0%)
        layout.uncolored_key_behavior = UncoloredKeyBehavior::from(0);
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with Off:\n{markdown}");
        assert!(markdown.contains("**Uncolored Key Brightness**: 0%"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.uncolored_key_behavior.as_percent(), 0);

        // Test with 50% brightness
        layout.uncolored_key_behavior = UncoloredKeyBehavior::from(50);
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with 50% brightness:\n{markdown}");
        assert!(markdown.contains("**Uncolored Key Brightness**: 50%"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.uncolored_key_behavior.as_percent(), 50);

        // Test RGB brightness
        layout.rgb_brightness = crate::models::RgbBrightness::from(50);
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with 50% brightness:\n{markdown}");
        assert!(markdown.contains("**RGB Brightness**: 50%"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.rgb_brightness.as_percent(), 50);

        // Test RGB saturation
        layout.rgb_saturation = crate::models::RgbSaturation::from(75);
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with 75% saturation:\n{markdown}");
        assert!(markdown.contains("**RGB Saturation**: 75%"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.rgb_saturation.as_percent(), 75);

        // Test RGB Matrix Speed
        layout.rgb_matrix_default_speed = 200;
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with RGB speed 200:\n{markdown}");
        assert!(markdown.contains("**RGB Matrix Speed**: 200"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.rgb_matrix_default_speed, 200);
    }

    mod test_helpers {
        use crate::models::{HoldDecisionMode, TapHoldPreset, TapHoldSettings};

        pub fn assert_home_row_mods_markdown(markdown: &str) {
            assert!(markdown.contains("## Settings"));
            assert!(markdown.contains("**Tap-Hold Preset**: Home Row Mods"));
            assert!(markdown.contains("**Tapping Term**: 175ms"));
            assert!(markdown.contains("**Retro Tapping**: On"));
            assert!(markdown.contains("**Flow Tap Term**: 150ms"));
            assert!(markdown.contains("**Chordal Hold**: On"));
        }

        pub fn assert_home_row_mods_settings(settings: &TapHoldSettings) {
            assert_eq!(settings.preset, TapHoldPreset::HomeRowMods);
            assert_eq!(settings.tapping_term, 175);
            assert!(settings.retro_tapping);
            assert_eq!(settings.flow_tap_term, Some(150));
            assert!(settings.chordal_hold);
        }

        pub fn assert_custom_settings_markdown(markdown: &str) {
            assert!(markdown.contains("**Tap-Hold Preset**: Custom"));
            assert!(markdown.contains("**Tapping Term**: 180ms"));
            assert!(markdown.contains("**Quick Tap Term**: 100ms"));
            assert!(markdown.contains("**Hold Mode**: Hold On Other Key Press"));
            assert!(markdown.contains("**Retro Tapping**: On"));
            assert!(markdown.contains("**Tapping Toggle**: 3 taps"));
            assert!(markdown.contains("**Flow Tap Term**: 120ms"));
            assert!(markdown.contains("**Chordal Hold**: On"));
        }

        pub fn assert_custom_settings(settings: &TapHoldSettings) {
            assert_eq!(settings.preset, TapHoldPreset::Custom);
            assert_eq!(settings.tapping_term, 180);
            assert_eq!(settings.quick_tap_term, Some(100));
            assert_eq!(settings.hold_mode, HoldDecisionMode::HoldOnOtherKeyPress);
            assert!(settings.retro_tapping);
            assert_eq!(settings.tapping_toggle, 3);
            assert_eq!(settings.flow_tap_term, Some(120));
            assert!(settings.chordal_hold);
        }
    }

    #[test]
    fn test_tap_hold_home_row_mods_preset_round_trip() {
        use crate::models::{TapHoldPreset, TapHoldSettings};
        use test_helpers::*;

        let mut layout = create_test_layout();
        layout.tap_hold_settings = TapHoldSettings::from_preset(TapHoldPreset::HomeRowMods);

        let markdown = generate_markdown(&layout).unwrap();
        assert_home_row_mods_markdown(&markdown);

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_home_row_mods_settings(&parsed.tap_hold_settings);
    }

    #[test]
    fn test_tap_hold_custom_settings_round_trip() {
        use crate::models::{HoldDecisionMode, TapHoldPreset, TapHoldSettings};
        use test_helpers::*;

        let mut layout = create_test_layout();
        layout.tap_hold_settings = TapHoldSettings {
            tapping_term: 180,
            quick_tap_term: Some(100),
            hold_mode: HoldDecisionMode::HoldOnOtherKeyPress,
            retro_tapping: true,
            tapping_toggle: 3,
            flow_tap_term: Some(120),
            chordal_hold: true,
            preset: TapHoldPreset::Custom,
        };

        let markdown = generate_markdown(&layout).unwrap();
        assert_custom_settings_markdown(&markdown);

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_custom_settings(&parsed.tap_hold_settings);
    }

    #[test]
    fn test_tap_hold_default_settings_not_written() {
        use crate::models::TapHoldSettings;

        let mut layout = create_test_layout();
        layout.tap_hold_settings = TapHoldSettings::default();

        let markdown = generate_markdown(&layout).unwrap();
        assert!(!markdown.contains("Tap-Hold"));
        assert!(!markdown.contains("Tapping Term"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.tap_hold_settings, TapHoldSettings::default());
    }

    #[test]
    fn test_tap_hold_responsive_preset_round_trip() {
        use crate::models::{HoldDecisionMode, TapHoldPreset, TapHoldSettings};

        let mut layout = create_test_layout();

        // Test with Responsive preset
        layout.tap_hold_settings = TapHoldSettings::from_preset(TapHoldPreset::Responsive);
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with Responsive:\n{markdown}");
        assert!(markdown.contains("**Tap-Hold Preset**: Responsive"));
        assert!(markdown.contains("**Tapping Term**: 150ms"));
        assert!(markdown.contains("**Quick Tap Term**: 100ms"));
        assert!(markdown.contains("**Hold Mode**: Hold On Other Key Press"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.tap_hold_settings.preset, TapHoldPreset::Responsive);
        assert_eq!(parsed.tap_hold_settings.tapping_term, 150);
        assert_eq!(parsed.tap_hold_settings.quick_tap_term, Some(100));
        assert_eq!(
            parsed.tap_hold_settings.hold_mode,
            HoldDecisionMode::HoldOnOtherKeyPress
        );
    }

    #[test]
    fn test_tap_hold_deliberate_preset_round_trip() {
        use crate::models::{TapHoldPreset, TapHoldSettings};

        let mut layout = create_test_layout();

        // Test with Deliberate preset
        layout.tap_hold_settings = TapHoldSettings::from_preset(TapHoldPreset::Deliberate);
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with Deliberate:\n{markdown}");
        assert!(markdown.contains("**Tap-Hold Preset**: Deliberate"));
        assert!(markdown.contains("**Tapping Term**: 250ms"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.tap_hold_settings.preset, TapHoldPreset::Deliberate);
        assert_eq!(parsed.tap_hold_settings.tapping_term, 250);
    }

    #[test]
    fn test_key_descriptions_round_trip() {
        let mut layout = create_test_layout();

        // Add descriptions to some keys
        layout.layers[0].keys[0].description = Some("Primary thumb key".to_string());
        layout.layers[0].keys[1].description = Some("Secondary action key".to_string());

        // Generate markdown
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with descriptions:\n{markdown}");

        // Verify the descriptions section is present
        assert!(markdown.contains("## Key Descriptions"));
        assert!(markdown.contains("- 0:0:0: Primary thumb key"));
        assert!(markdown.contains("- 0:0:1: Secondary action key"));

        // Parse it back
        let parsed = parse_markdown_layout_str(&markdown).unwrap();

        // Verify descriptions are preserved
        assert_eq!(
            parsed.layers[0].keys[0].description,
            Some("Primary thumb key".to_string())
        );
        assert_eq!(
            parsed.layers[0].keys[1].description,
            Some("Secondary action key".to_string())
        );
    }

    #[test]
    fn test_key_descriptions_no_section_when_empty() {
        let layout = create_test_layout();

        // No descriptions - should not have descriptions section
        let markdown = generate_markdown(&layout).unwrap();
        assert!(!markdown.contains("## Key Descriptions"));
    }

    // === Idle Effect Settings Tests ===

    #[test]
    fn test_idle_effect_default_not_written() {
        let layout = create_test_layout();
        let markdown = generate_markdown(&layout).unwrap();

        // Default settings should not be written
        assert!(!markdown.contains("Idle Effect"));
        assert!(!markdown.contains("Idle Timeout"));
        assert!(!markdown.contains("Idle Effect Duration"));
        assert!(!markdown.contains("Idle Effect Mode"));
    }

    #[test]
    fn test_idle_effect_disabled() {
        let mut layout = create_test_layout();
        layout.idle_effect_settings = crate::models::IdleEffectSettings {
            enabled: false,
            ..crate::models::IdleEffectSettings::default()
        };

        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Effect**: Off"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert!(!parsed.idle_effect_settings.enabled);
    }

    #[test]
    fn test_idle_effect_timeout_formats() {
        let mut layout = create_test_layout();

        // Test minutes format
        layout.idle_effect_settings.idle_timeout_ms = 120_000; // 2 minutes
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Timeout**: 2 min"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.idle_effect_settings.idle_timeout_ms, 120_000);

        // Test seconds format
        layout.idle_effect_settings.idle_timeout_ms = 45_000; // 45 seconds
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Timeout**: 45 sec"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.idle_effect_settings.idle_timeout_ms, 45_000);

        // Test milliseconds format
        layout.idle_effect_settings.idle_timeout_ms = 12_345; // Odd milliseconds
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Timeout**: 12345ms"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.idle_effect_settings.idle_timeout_ms, 12_345);

        // Test zero (disabled)
        layout.idle_effect_settings.idle_timeout_ms = 0;
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Timeout**: 0"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.idle_effect_settings.idle_timeout_ms, 0);
    }

    #[test]
    fn test_idle_effect_duration_formats() {
        let mut layout = create_test_layout();

        // Test minutes format
        layout.idle_effect_settings.idle_effect_duration_ms = 600_000; // 10 minutes
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Effect Duration**: 10 min"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.idle_effect_settings.idle_effect_duration_ms, 600_000);

        // Test seconds format
        layout.idle_effect_settings.idle_effect_duration_ms = 90_000; // 90 seconds
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Effect Duration**: 90 sec"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.idle_effect_settings.idle_effect_duration_ms, 90_000);

        // Test zero (immediate off)
        layout.idle_effect_settings.idle_effect_duration_ms = 0;
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Effect Duration**: 0"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.idle_effect_settings.idle_effect_duration_ms, 0);
    }

    #[test]
    fn test_idle_effect_mode_round_trip() {
        use crate::models::RgbMatrixEffect;

        let mut layout = create_test_layout();

        // Test various effect modes (skip Breathing which is the default)
        let effects = [
            RgbMatrixEffect::RainbowMovingChevron,
            RgbMatrixEffect::CycleAll,
            RgbMatrixEffect::JellybeanRaindrops,
        ];

        for effect in effects {
            layout.idle_effect_settings.idle_effect_mode = effect;
            let markdown = generate_markdown(&layout).unwrap();
            assert!(
                markdown.contains("**Idle Effect Mode**:"),
                "Missing Idle Effect Mode for {}",
                effect.display_name()
            );
            assert!(
                markdown.contains(effect.display_name()),
                "Missing effect name {}",
                effect.display_name()
            );

            let parsed = parse_markdown_layout_str(&markdown).unwrap();
            assert_eq!(parsed.idle_effect_settings.idle_effect_mode, effect);
        }
    }

    #[test]
    fn test_idle_effect_complete_settings() {
        use crate::models::RgbMatrixEffect;

        let mut layout = create_test_layout();
        layout.idle_effect_settings = crate::models::IdleEffectSettings {
            enabled: true,
            idle_timeout_ms: 30_000,          // 30 seconds
            idle_effect_duration_ms: 180_000, // 3 minutes
            idle_effect_mode: RgbMatrixEffect::RainbowBeacon,
        };

        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown:\n{markdown}");

        assert!(markdown.contains("**Idle Timeout**: 30 sec"));
        assert!(markdown.contains("**Idle Effect Duration**: 3 min"));
        assert!(markdown.contains("**Idle Effect Mode**: Rainbow Beacon"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert!(parsed.idle_effect_settings.enabled);
        assert_eq!(parsed.idle_effect_settings.idle_timeout_ms, 30_000);
        assert_eq!(parsed.idle_effect_settings.idle_effect_duration_ms, 180_000);
        assert_eq!(
            parsed.idle_effect_settings.idle_effect_mode,
            RgbMatrixEffect::RainbowBeacon
        );
    }

    // === RGB Overlay Ripple Settings Tests ===

    #[test]
    fn test_ripple_overlay_default_not_written() {
        let layout = create_test_layout();
        let markdown = generate_markdown(&layout).unwrap();

        // Default settings should not be written
        assert!(!markdown.contains("Ripple Overlay"));
        assert!(!markdown.contains("Max Ripples"));
        assert!(!markdown.contains("Ripple Duration"));
    }

    #[test]
    fn test_ripple_overlay_enabled() {
        let mut layout = create_test_layout();
        layout.rgb_overlay_ripple = crate::models::RgbOverlayRippleSettings {
            enabled: true,
            ..crate::models::RgbOverlayRippleSettings::default()
        };

        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown:\n{markdown}");
        assert!(markdown.contains("**Ripple Overlay**: On"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert!(parsed.rgb_overlay_ripple.enabled);
    }

    #[test]
    fn test_ripple_overlay_basic_settings_round_trip() {
        let mut layout = create_test_layout();
        layout.rgb_overlay_ripple.enabled = true;
        layout.rgb_overlay_ripple.max_ripples = 6;
        layout.rgb_overlay_ripple.duration_ms = 750;
        layout.rgb_overlay_ripple.speed = 200;
        layout.rgb_overlay_ripple.band_width = 5;
        layout.rgb_overlay_ripple.amplitude_pct = 75;

        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown:\n{markdown}");

        assert!(markdown.contains("**Ripple Overlay**: On"));
        assert!(markdown.contains("**Max Ripples**: 6"));
        assert!(markdown.contains("**Ripple Duration**: 750ms"));
        assert!(markdown.contains("**Ripple Speed**: 200"));
        assert!(markdown.contains("**Ripple Band Width**: 5"));
        assert!(markdown.contains("**Ripple Amplitude**: 75%"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert!(parsed.rgb_overlay_ripple.enabled);
        assert_eq!(parsed.rgb_overlay_ripple.max_ripples, 6);
        assert_eq!(parsed.rgb_overlay_ripple.duration_ms, 750);
        assert_eq!(parsed.rgb_overlay_ripple.speed, 200);
        assert_eq!(parsed.rgb_overlay_ripple.band_width, 5);
        assert_eq!(parsed.rgb_overlay_ripple.amplitude_pct, 75);
    }

    #[test]
    fn test_ripple_overlay_color_modes_round_trip() {
        use crate::models::{RgbColor, RippleColorMode};

        let mut layout = create_test_layout();
        layout.rgb_overlay_ripple.enabled = true;

        // Test Fixed mode with custom color
        layout.rgb_overlay_ripple.color_mode = RippleColorMode::Fixed;
        layout.rgb_overlay_ripple.fixed_color = RgbColor::new(255, 0, 255); // Magenta
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Ripple Fixed Color**: #FF00FF"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.rgb_overlay_ripple.color_mode, RippleColorMode::Fixed);
        assert_eq!(
            parsed.rgb_overlay_ripple.fixed_color,
            RgbColor::new(255, 0, 255)
        );

        // Test KeyBased mode
        layout.rgb_overlay_ripple.color_mode = RippleColorMode::KeyBased;
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Ripple Color Mode**: Key Based"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(
            parsed.rgb_overlay_ripple.color_mode,
            RippleColorMode::KeyBased
        );

        // Test HueShift mode with custom shift
        layout.rgb_overlay_ripple.color_mode = RippleColorMode::HueShift;
        layout.rgb_overlay_ripple.hue_shift_deg = 120;
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Ripple Color Mode**: Hue Shift"));
        assert!(markdown.contains("**Ripple Hue Shift**: 120°"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(
            parsed.rgb_overlay_ripple.color_mode,
            RippleColorMode::HueShift
        );
        assert_eq!(parsed.rgb_overlay_ripple.hue_shift_deg, 120);
    }

    #[test]
    fn test_ripple_overlay_trigger_settings_round_trip() {
        let mut layout = create_test_layout();
        layout.rgb_overlay_ripple.enabled = true;
        layout.rgb_overlay_ripple.trigger_on_press = false;
        layout.rgb_overlay_ripple.trigger_on_release = true;

        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown:\n{markdown}");

        assert!(markdown.contains("**Ripple Trigger on Press**: Off"));
        assert!(markdown.contains("**Ripple Trigger on Release**: On"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert!(!parsed.rgb_overlay_ripple.trigger_on_press);
        assert!(parsed.rgb_overlay_ripple.trigger_on_release);
    }

    #[test]
    fn test_ripple_overlay_ignore_settings_round_trip() {
        let mut layout = create_test_layout();
        layout.rgb_overlay_ripple.enabled = true;
        layout.rgb_overlay_ripple.ignore_transparent = false;
        layout.rgb_overlay_ripple.ignore_modifiers = true;
        layout.rgb_overlay_ripple.ignore_layer_switch = true;

        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown:\n{markdown}");

        assert!(markdown.contains("**Ripple Ignore Transparent**: Off"));
        assert!(markdown.contains("**Ripple Ignore Modifiers**: On"));
        assert!(markdown.contains("**Ripple Ignore Layer Switch**: On"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert!(!parsed.rgb_overlay_ripple.ignore_transparent);
        assert!(parsed.rgb_overlay_ripple.ignore_modifiers);
        assert!(parsed.rgb_overlay_ripple.ignore_layer_switch);
    }

    #[test]
    fn test_ripple_overlay_complete_settings() {
        use crate::models::{RgbColor, RippleColorMode};

        let mut layout = create_test_layout();
        layout.rgb_overlay_ripple = crate::models::RgbOverlayRippleSettings {
            enabled: true,
            max_ripples: 8,
            duration_ms: 1000,
            speed: 255,
            band_width: 4,
            amplitude_pct: 100,
            color_mode: RippleColorMode::HueShift,
            fixed_color: RgbColor::new(255, 255, 0), // Yellow
            hue_shift_deg: -90,
            trigger_on_press: true,
            trigger_on_release: true,
            ignore_transparent: false,
            ignore_modifiers: true,
            ignore_layer_switch: true,
        };

        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown:\n{markdown}");

        assert!(markdown.contains("**Ripple Overlay**: On"));
        assert!(markdown.contains("**Max Ripples**: 8"));
        assert!(markdown.contains("**Ripple Duration**: 1000ms"));
        assert!(markdown.contains("**Ripple Speed**: 255"));
        assert!(markdown.contains("**Ripple Band Width**: 4"));
        assert!(markdown.contains("**Ripple Amplitude**: 100%"));
        assert!(markdown.contains("**Ripple Color Mode**: Hue Shift"));
        assert!(markdown.contains("**Ripple Fixed Color**: #FFFF00"));
        assert!(markdown.contains("**Ripple Hue Shift**: -90°"));
        assert!(markdown.contains("**Ripple Trigger on Release**: On"));
        assert!(markdown.contains("**Ripple Ignore Transparent**: Off"));
        assert!(markdown.contains("**Ripple Ignore Modifiers**: On"));
        assert!(markdown.contains("**Ripple Ignore Layer Switch**: On"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        let rip = &parsed.rgb_overlay_ripple;
        assert!(rip.enabled);
        assert_eq!(rip.max_ripples, 8);
        assert_eq!(rip.duration_ms, 1000);
        assert_eq!(rip.speed, 255);
        assert_eq!(rip.band_width, 4);
        assert_eq!(rip.amplitude_pct, 100);
        assert_eq!(rip.color_mode, RippleColorMode::HueShift);
        assert_eq!(rip.fixed_color, RgbColor::new(255, 255, 0));
        assert_eq!(rip.hue_shift_deg, -90);
        assert!(rip.trigger_on_press);
        assert!(rip.trigger_on_release);
        assert!(!rip.ignore_transparent);
        assert!(rip.ignore_modifiers);
        assert!(rip.ignore_layer_switch);
    }
}
