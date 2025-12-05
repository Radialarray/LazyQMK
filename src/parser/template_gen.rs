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

    Ok(output)
}

/// Generates YAML frontmatter from metadata.
fn generate_frontmatter(layout: &Layout) -> Result<String> {
    let yaml =
        serde_yaml::to_string(&layout.metadata).context("Failed to serialize metadata to YAML")?;

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
                key.description.as_ref().map(|desc| {
                    (layer_idx, key.position.row, key.position.col, desc.clone())
                })
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
    use crate::models::{HoldDecisionMode, RgbBrightness, TapHoldPreset, TapHoldSettings, UncoloredKeyBehavior};

    let default_uncolored = UncoloredKeyBehavior::default();
    let default_tap_hold = TapHoldSettings::default();

    // Check if we have any non-default settings
    let has_rgb_settings = !layout.rgb_enabled || layout.rgb_brightness != RgbBrightness::default() || layout.rgb_timeout_ms > 0;
    let has_uncolored_setting = layout.uncolored_key_behavior != default_uncolored;
    let has_tap_hold_settings = layout.tap_hold_settings != default_tap_hold;

    if !has_rgb_settings && !has_uncolored_setting && !has_tap_hold_settings {
        return None;
    }

    let mut output = String::from("## Settings\n\n");

    // Write RGB settings
    if !layout.rgb_enabled {
        output.push_str("**RGB Enabled**: Off\n");
    }
    if layout.rgb_brightness != RgbBrightness::default() {
        output.push_str(&format!("**RGB Brightness**: {}%\n", layout.rgb_brightness.as_percent()));
    }

    // Write uncolored_key_behavior if not default
    // Only write if not default (100%)
    if layout.uncolored_key_behavior.as_percent() != 100 {
        output.push_str(&format!("**Uncolored Key Brightness**: {}%\n", layout.uncolored_key_behavior.as_percent()));
    }

    // Write RGB timeout if set
    if layout.rgb_timeout_ms > 0 {
        // Convert milliseconds to a human-readable format
        let timeout_ms = layout.rgb_timeout_ms;
        if timeout_ms >= 60000 && timeout_ms % 60000 == 0 {
            // Whole minutes
            output.push_str(&format!("**RGB Timeout**: {} min\n", timeout_ms / 60000));
        } else if timeout_ms >= 1000 && timeout_ms % 1000 == 0 {
            // Whole seconds
            output.push_str(&format!("**RGB Timeout**: {} sec\n", timeout_ms / 1000));
        } else {
            // Milliseconds
            output.push_str(&format!("**RGB Timeout**: {timeout_ms}ms\n"));
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
            output.push_str(&format!("**Tapping Toggle**: {} taps\n", ths.tapping_toggle));
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
    use crate::models::{Category, ColorPalette, KeyDefinition, Layer, LayoutMetadata, Position, RgbColor};
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
            default_color: ColorPalette::load().unwrap_or_default().default_layer_color(),
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
            rgb_timeout_ms: 0,
            uncolored_key_behavior: crate::models::UncoloredKeyBehavior::default(),
            tap_hold_settings: crate::models::TapHoldSettings::default(),
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
    }

    #[test]
    fn test_tap_hold_settings_round_trip() {
        use crate::models::{HoldDecisionMode, TapHoldPreset, TapHoldSettings};

        let mut layout = create_test_layout();

        // Test with HomeRowMods preset
        layout.tap_hold_settings = TapHoldSettings::from_preset(TapHoldPreset::HomeRowMods);
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with HomeRowMods:\n{markdown}");
        assert!(markdown.contains("## Settings"));
        assert!(markdown.contains("**Tap-Hold Preset**: Home Row Mods"));
        assert!(markdown.contains("**Tapping Term**: 175ms"));
        assert!(markdown.contains("**Retro Tapping**: On"));
        assert!(markdown.contains("**Flow Tap Term**: 150ms"));
        assert!(markdown.contains("**Chordal Hold**: On"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.tap_hold_settings.preset, TapHoldPreset::HomeRowMods);
        assert_eq!(parsed.tap_hold_settings.tapping_term, 175);
        assert!(parsed.tap_hold_settings.retro_tapping);
        assert_eq!(parsed.tap_hold_settings.flow_tap_term, Some(150));
        assert!(parsed.tap_hold_settings.chordal_hold);

        // Test with custom settings
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
        println!("Generated markdown with Custom:\n{markdown}");
        assert!(markdown.contains("**Tap-Hold Preset**: Custom"));
        assert!(markdown.contains("**Tapping Term**: 180ms"));
        assert!(markdown.contains("**Quick Tap Term**: 100ms"));
        assert!(markdown.contains("**Hold Mode**: Hold On Other Key Press"));
        assert!(markdown.contains("**Retro Tapping**: On"));
        assert!(markdown.contains("**Tapping Toggle**: 3 taps"));
        assert!(markdown.contains("**Flow Tap Term**: 120ms"));
        assert!(markdown.contains("**Chordal Hold**: On"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.tap_hold_settings.preset, TapHoldPreset::Custom);
        assert_eq!(parsed.tap_hold_settings.tapping_term, 180);
        assert_eq!(parsed.tap_hold_settings.quick_tap_term, Some(100));
        assert_eq!(
            parsed.tap_hold_settings.hold_mode,
            HoldDecisionMode::HoldOnOtherKeyPress
        );
        assert!(parsed.tap_hold_settings.retro_tapping);
        assert_eq!(parsed.tap_hold_settings.tapping_toggle, 3);
        assert_eq!(parsed.tap_hold_settings.flow_tap_term, Some(120));
        assert!(parsed.tap_hold_settings.chordal_hold);

        // Test with default settings - should NOT write tap-hold settings
        layout.tap_hold_settings = TapHoldSettings::default();
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with defaults:\n{markdown}");
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
}
