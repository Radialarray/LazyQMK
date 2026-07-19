//! Markdown layout file parsing and generation.
//!
//! This module handles parsing keyboard layouts from human-readable Markdown files
//! and generating them back for saving. The format uses YAML frontmatter for metadata
//! and Markdown tables for key assignments.
//!
//! The parser is organised by **phase**, with one module per section type:
//!
//! - [`metadata`] — YAML frontmatter (layout name, author, tags, …)
//! - [`layers`] — `## Layer N: Name` blocks with key tables
//! - [`categories`] — `## Categories` section
//! - [`settings`] — `## Settings` section (split into one file per group)
//! - [`key_descriptions`] — `## Key Descriptions` section
//! - [`tap_dances`] — `## Tap Dances` section
//!
//! The dispatch happens in [`parse_content`], which is invoked by the public
//! entry points [`parse_markdown_layout`] and [`parse_markdown_layout_str`].

mod categories;
mod key_descriptions;
mod layers;
mod metadata;
mod settings;
mod tap_dances;

use crate::constants::APP_BINARY_NAME;
use crate::models::Layout;
use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

/// Cached regex for validating layout tag identifiers (lowercase, digits, hyphens).
pub(super) fn tag_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-z0-9-]+$").unwrap())
}

/// Cached regex for parsing `## Layer N: Name` headers.
pub(super) fn layer_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^##\s+Layer\s+(\d+):\s+(.+)$").unwrap())
}

/// Cached regex for parsing keycode cell syntax (with optional color and category).
pub(super) fn keycode_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^([A-Z_][A-Z_0-9]*(?:\([^)]*\))?)(?:\{(#[0-9A-Fa-f]{6})\})?(?:@([a-z][a-z0-9-]*))?\s*$",
        )
        .unwrap()
    })
}

/// Cached regex for parsing category lines: `- id: Name (#RRGGBB)`.
pub(super) fn category_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^-\s+([a-z][a-z0-9-]*):\s+(.+?)\s+\(#([0-9A-Fa-f]{6})\)$").unwrap()
    })
}

/// Cached regex for parsing combo lines.
pub(super) fn combo_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^\*\*Combo\s+(\d+)\*\*:\s*\((\d+),(\d+)\)\s*\+\s*\((\d+),(\d+)\)\s*→\s*(.+?)\s*(?:\[(\d+)ms\])?$",
        )
        .unwrap()
    })
}

/// Cached regex for parsing key description lines: `- layer:row:col: description`.
pub(super) fn desc_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^-\s+(\d+):(\d+):(\d+):\s+(.+)$").unwrap())
}

/// Parses a Markdown layout file into a Layout structure.
///
/// # File Format
///
/// ```markdown
/// ---
/// name: "Layout Name"
/// description: "Description"
/// author: "Author"
/// created: "2024-01-15T10:30:00Z"
/// modified: "2024-01-20T15:45:00Z"
/// tags: ["tag1", "tag2"]
/// is_template: false
/// version: "1.0"
/// ---
///
/// # Layout Title
///
/// ## Layer 0: Base
/// **Color**: #808080
/// **Category**: optional-category-id
///
/// | KC_TAB | KC_Q | ... |
/// |--------|------|-----|
/// | KC_A   | KC_S | ... |
///
/// ## Categories
///
/// - category-id: Category Name (#RRGGBB)
/// ```
///
/// # Errors
///
/// Returns errors for:
/// - File not found
/// - Invalid YAML frontmatter
/// - Malformed layer headers
/// - Invalid table structure
/// - Invalid keycodes or color syntax
pub fn parse_markdown_layout(path: &Path) -> Result<Layout> {
    // Check if file exists first to provide better error message
    if !path.exists() {
        anyhow::bail!(
            "Layout file not found: {}\n\n\
             Please check the file path and try again.\n\
             If you need help getting started, run: {} --init",
            path.display(),
            APP_BINARY_NAME
        );
    }

    // Check if it's a file (not a directory)
    if !path.is_file() {
        anyhow::bail!(
            "Path is not a file: {}\n\n\
            Please provide a path to a Markdown (.md) file.",
            path.display()
        );
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read layout file: {}", path.display()))?;

    parse_markdown_layout_str(&content)
        .with_context(|| format!("Failed to parse layout file: {}", path.display()))
}

/// Parses a Markdown layout from a string.
pub fn parse_markdown_layout_str(content: &str) -> Result<Layout> {
    let lines: Vec<&str> = content.lines().collect();

    // Parse frontmatter
    let (metadata, content_start) = metadata::parse_frontmatter(&lines)?;

    // Create layout
    let mut layout = Layout {
        metadata,
        layers: Vec::new(),
        categories: Vec::new(),
        rgb_enabled: true,
        rgb_brightness: crate::models::RgbBrightness::default(),
        rgb_saturation: crate::models::RgbSaturation::default(),
        rgb_matrix_default_speed: 127,
        rgb_timeout_ms: 0,
        uncolored_key_behavior: crate::models::UncoloredKeyBehavior::default(),
        idle_effect_settings: crate::models::IdleEffectSettings::default(),
        rgb_overlay_ripple: crate::models::RgbOverlayRippleSettings::default(),
        palette_fx: crate::models::PaletteFxSettings::default(),
        tap_hold_settings: crate::models::TapHoldSettings::default(),
        combo_settings: crate::models::ComboSettings::default(),
        tap_dances: Vec::new(),
    };

    // Parse content (layers and categories)
    parse_content(&lines[content_start..], &mut layout)?;

    // Auto-create missing tap dance definitions for any TD() references
    layout.auto_create_tap_dances();

    // Validate the parsed layout
    layout.validate()?;

    Ok(layout)
}

/// Parses the content section (layers and categories).
fn parse_content(lines: &[&str], layout: &mut Layout) -> Result<()> {
    let mut line_num = 0;

    while line_num < lines.len() {
        let line = lines[line_num].trim();

        // Skip empty lines and main title
        if line.is_empty() || line.starts_with("# ") {
            line_num += 1;
            continue;
        }

        // Check for layer header (## Layer N: Name)
        if line.starts_with("## Layer ") {
            line_num = layers::parse_layer(lines, line_num, layout)
                .with_context(|| format!("Error parsing layer at line {}", line_num + 1))?;
            continue;
        }

        // Check for categories section (## Categories)
        if line == "## Categories" {
            line_num = categories::parse_categories(lines, line_num, layout)
                .with_context(|| format!("Error parsing categories at line {}", line_num + 1))?;
            continue;
        }

        // Check for settings section (## Settings)
        if line == "## Settings" {
            line_num = settings::parse_settings(lines, line_num, layout)
                .with_context(|| format!("Error parsing settings at line {}", line_num + 1))?;
            continue;
        }

        // Check for key descriptions section (## Key Descriptions)
        if line == "## Key Descriptions" {
            line_num = key_descriptions::parse_key_descriptions(lines, line_num, layout)
                .with_context(|| {
                    format!("Error parsing key descriptions at line {}", line_num + 1)
                })?;
            continue;
        }

        // Check for tap dances section (## Tap Dances)
        if line == "## Tap Dances" {
            line_num = tap_dances::parse_tap_dances(lines, line_num, layout)
                .with_context(|| format!("Error parsing tap dances at line {}", line_num + 1))?;
            continue;
        }

        line_num += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Position, RgbColor};

    #[test]
    fn test_parse_frontmatter() {
        let lines = vec![
            "---",
            "name: \"Test Layout\"",
            "description: \"A test layout\"",
            "author: \"test\"",
            "created: \"2024-01-15T10:30:00Z\"",
            "modified: \"2024-01-20T15:45:00Z\"",
            "tags: [\"test\", \"example\"]",
            "is_template: false",
            "version: \"1.0\"",
            "---",
            "",
            "# Content starts here",
        ];

        let (metadata, content_start) = metadata::parse_frontmatter(&lines).unwrap();
        assert_eq!(metadata.name, "Test Layout");
        assert_eq!(metadata.description, "A test layout");
        assert_eq!(metadata.author, "test");
        assert_eq!(metadata.tags, vec!["test", "example"]);
        assert!(!metadata.is_template);
        assert_eq!(metadata.version, "1.0");
        assert_eq!(content_start, 10);
    }

    #[test]
    fn test_parse_keycode_syntax() {
        // Basic keycode
        let key = layers::parse_keycode_syntax("KC_A", 0, 0).unwrap();
        assert_eq!(key.keycode, "KC_A");
        assert_eq!(key.position, Position::new(0, 0));
        assert_eq!(key.color_override, None);
        assert_eq!(key.category_id, None);

        // With color override
        let key = layers::parse_keycode_syntax("KC_A{#FF0000}", 0, 1).unwrap();
        assert_eq!(key.keycode, "KC_A");
        assert_eq!(key.color_override, Some(RgbColor::new(255, 0, 0)));

        // With category
        let key = layers::parse_keycode_syntax("KC_LEFT@navigation", 1, 0).unwrap();
        assert_eq!(key.keycode, "KC_LEFT");
        assert_eq!(key.category_id, Some("navigation".to_string()));

        // With both
        let key = layers::parse_keycode_syntax("KC_A{#00FF00}@symbols", 1, 1).unwrap();
        assert_eq!(key.keycode, "KC_A");
        assert_eq!(key.color_override, Some(RgbColor::new(0, 255, 0)));
        assert_eq!(key.category_id, Some("symbols".to_string()));
    }

    #[test]
    fn test_parse_complete_layout() {
        let content = r#"---
name: "Test Layout"
description: "A test"
author: "test"
created: "2024-01-15T10:30:00Z"
modified: "2024-01-20T15:45:00Z"
tags: ["test"]
is_template: false
version: "1.0"
---

# Test Layout

## Layer 0: Base
**Color**: #808080

| C0   | C1   |
|------|------|
| KC_A | KC_B |
| KC_C | KC_D |

## Categories

- navigation: Navigation (#0000FF)
"#;

        let layout = parse_markdown_layout_str(content).unwrap();
        assert_eq!(layout.metadata.name, "Test Layout");
        assert_eq!(layout.layers.len(), 1);
        assert_eq!(layout.layers[0].keys.len(), 4);
        println!("Parsed {} categories", layout.categories.len());
        for cat in &layout.categories {
            println!("Category: {} - {} - {:?}", cat.id, cat.name, cat.color);
        }
        assert_eq!(layout.categories.len(), 1);
    }

    #[test]
    fn test_parse_split_keyboard_with_gap() {
        use crate::models::Position;

        // Test that split keyboards with empty gap columns preserve correct key positions
        // This is critical: the gap between left and right halves must not shift columns
        let content = r#"---
name: "Split Layout Test"
description: "Tests gap handling"
author: "test"
created: "2024-01-15T10:30:00Z"
modified: "2024-01-20T15:45:00Z"
tags: []
is_template: false
version: "1.0"
---

# Split Layout

## Layer 0: Base
**Color**: #808080

| C0   | C1   |      | C3   | C4   |
|------|------|------|------|------|
| KC_A | KC_B |      | KC_C | KC_D |
| KC_E | KC_F |      | KC_G | KC_H |

"#;

        let layout = parse_markdown_layout_str(content).unwrap();
        assert_eq!(layout.layers.len(), 1);

        let layer = &layout.layers[0];
        // Should have 8 keys: 4 on left (cols 0,1), 4 on right (cols 3,4)
        assert_eq!(layer.keys.len(), 8);

        // Verify left side keys are at columns 0 and 1
        let key_a = layer
            .get_key(Position::new(0, 0))
            .expect("KC_A should be at row 0, col 0");
        assert_eq!(key_a.keycode, "KC_A");

        let key_b = layer
            .get_key(Position::new(0, 1))
            .expect("KC_B should be at row 0, col 1");
        assert_eq!(key_b.keycode, "KC_B");

        // Verify right side keys are at columns 3 and 4 (NOT 2 and 3!)
        // This is the critical test - the gap at column 2 must be preserved

        let key_c = layer
            .get_key(Position::new(0, 3))
            .expect("KC_C should be at row 0, col 3");
        assert_eq!(key_c.keycode, "KC_C");

        let key_d = layer
            .get_key(Position::new(0, 4))
            .expect("KC_D should be at row 0, col 4");
        assert_eq!(key_d.keycode, "KC_D");

        // Verify second row maintains the same column structure
        let key_g = layer
            .get_key(Position::new(1, 3))
            .expect("KC_G should be at row 1, col 3");
        assert_eq!(key_g.keycode, "KC_G");

        let key_h = layer
            .get_key(Position::new(1, 4))
            .expect("KC_H should be at row 1, col 4");
        assert_eq!(key_h.keycode, "KC_H");

        // Verify there's no key at the gap column
        assert!(
            layer.get_key(Position::new(0, 2)).is_none(),
            "Column 2 should be empty (gap)"
        );
        assert!(
            layer.get_key(Position::new(1, 2)).is_none(),
            "Column 2 should be empty (gap)"
        );
    }
}

#[cfg(test)]
mod description_tests {
    use super::*;
    use crate::models::Position;

    #[test]
    fn test_parse_descriptions_from_real_format() {
        let content = r"---
name: test
description: ''
author: ''
created: 2025-12-03T17:29:20.830366Z
modified: 2025-12-03T19:58:22.195899Z
tags: []
is_template: false
version: '1.0'
---
# test

## Layer 0: Base
**ID**: test-id
**Color**: #808080

| C0        | C1   |
|-----------|------|
| LCG(KC_Q) | KC_Q |

---

## Key Descriptions

- 0:0:0: This is a test
- 0:0:1: Another test
";

        let layout = parse_markdown_layout_str(content).expect("Parse failed");

        println!("Layer 0 has {} keys", layout.layers[0].keys.len());
        for key in &layout.layers[0].keys {
            println!(
                "  {:?}: {} -> {:?}",
                key.position, key.keycode, key.description
            );
        }

        // Check that descriptions were parsed
        let key0 = layout.layers[0]
            .keys
            .iter()
            .find(|k| k.position == Position { row: 0, col: 0 });
        let key1 = layout.layers[0]
            .keys
            .iter()
            .find(|k| k.position == Position { row: 0, col: 1 });

        assert!(key0.is_some(), "Key at 0:0 not found");
        assert!(key1.is_some(), "Key at 0:1 not found");

        assert_eq!(
            key0.unwrap().description,
            Some("This is a test".to_string()),
            "Key 0:0:0 description mismatch"
        );
        assert_eq!(
            key1.unwrap().description,
            Some("Another test".to_string()),
            "Key 0:0:1 description mismatch"
        );
    }
}

#[cfg(test)]
mod tap_dance_parsing_tests {
    use super::*;
    use crate::models::{KeyDefinition, Layer, Position, RgbColor, TapDanceAction};
    use crate::parser::template_gen;

    #[test]
    fn test_parse_tap_dances_from_markdown() {
        // Create a simple layout with tap dances
        let mut layout = Layout::new("Test Layout").expect("Failed to create layout");
        layout.tap_dances = vec![
            TapDanceAction {
                name: "esc_caps".to_string(),
                single_tap: "KC_ESC".to_string(),
                double_tap: Some("KC_CAPS".to_string()),
                hold: None,
            },
            TapDanceAction {
                name: "shift_ctrl".to_string(),
                single_tap: "KC_LSFT".to_string(),
                double_tap: Some("KC_CAPS".to_string()),
                hold: Some("KC_LCTL".to_string()),
            },
        ];

        // Add a simple layer
        let mut layer =
            Layer::new(0, "Base", RgbColor::new(128, 128, 128)).expect("Failed to create layer");
        layer.keys = vec![
            KeyDefinition::new(Position::new(0, 0), "KC_A"),
            KeyDefinition::new(Position::new(0, 1), "KC_B"),
        ];
        layout.layers.push(layer);

        // Generate markdown from the layout
        let markdown =
            template_gen::generate_markdown(&layout).expect("Failed to generate markdown");

        println!("Generated markdown:\n{}", markdown);

        // Parse it back
        let parsed_layout = parse_markdown_layout_str(&markdown).expect("Parse failed");

        println!("Parsed {} tap dances", parsed_layout.tap_dances.len());
        for td in &parsed_layout.tap_dances {
            println!(
                "  - {}: single={}, double={:?}, hold={:?}",
                td.name, td.single_tap, td.double_tap, td.hold
            );
        }

        assert_eq!(
            parsed_layout.tap_dances.len(),
            2,
            "Should parse 2 tap dances"
        );

        let td1 = &parsed_layout.tap_dances[0];
        assert_eq!(td1.name, "esc_caps");
        assert_eq!(td1.single_tap, "KC_ESC");
        assert_eq!(td1.double_tap, Some("KC_CAPS".to_string()));
        assert_eq!(td1.hold, None);

        let td2 = &parsed_layout.tap_dances[1];
        assert_eq!(td2.name, "shift_ctrl");
        assert_eq!(td2.single_tap, "KC_LSFT");
        assert_eq!(td2.double_tap, Some("KC_CAPS".to_string()));
        assert_eq!(td2.hold, Some("KC_LCTL".to_string()));
    }
}

#[cfg(test)]
mod combo_parsing_tests {
    use super::*;
    use crate::models::{ComboAction, ComboDefinition, KeyDefinition, Layer, Position, RgbColor};
    use crate::parser::template_gen;

    #[test]
    fn test_parse_combos_from_markdown() {
        // Create a layout with combo settings
        let mut layout = Layout::new("Test Layout").expect("Failed to create layout");

        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_A"));
        layer.add_key(KeyDefinition::new(Position::new(0, 1), "KC_B"));
        layer.add_key(KeyDefinition::new(Position::new(1, 0), "KC_C"));
        layer.add_key(KeyDefinition::new(Position::new(1, 1), "KC_D"));
        layout.add_layer(layer).unwrap();

        // Add combo settings
        layout.combo_settings.enabled = true;
        layout
            .combo_settings
            .add_combo(ComboDefinition::new(
                Position::new(0, 0),
                Position::new(0, 1),
                ComboAction::DisableEffects,
            ))
            .unwrap();
        layout
            .combo_settings
            .add_combo(ComboDefinition::with_duration(
                Position::new(1, 0),
                Position::new(1, 1),
                ComboAction::Bootloader,
                750,
            ))
            .unwrap();

        // Generate markdown
        let markdown =
            template_gen::generate_markdown(&layout).expect("Failed to generate markdown");

        println!("Generated markdown:\n{}", markdown);

        // Verify settings section is present
        assert!(markdown.contains("## Settings"), "Settings section missing");
        assert!(
            markdown.contains("**Combos**: On"),
            "Combos enabled missing"
        );
        assert!(
            markdown.contains("**Combo 1**: (0,0)+(0,1) → Disable Effects [500ms]"),
            "Combo 1 definition missing"
        );
        assert!(
            markdown.contains("**Combo 2**: (1,0)+(1,1) → Bootloader [750ms]"),
            "Combo 2 definition missing"
        );

        // Parse it back
        let parsed_layout = parse_markdown_layout_str(&markdown).expect("Parse failed");

        // Verify combo settings were parsed correctly
        assert!(
            parsed_layout.combo_settings.enabled,
            "Combos not enabled after parse"
        );
        assert_eq!(
            parsed_layout.combo_settings.combos.len(),
            2,
            "Combo count mismatch"
        );

        let combo1 = &parsed_layout.combo_settings.combos[0];
        assert_eq!(combo1.key1, Position::new(0, 0));
        assert_eq!(combo1.key2, Position::new(0, 1));
        assert_eq!(combo1.action, ComboAction::DisableEffects);
        assert_eq!(combo1.hold_duration_ms, 500);

        let combo2 = &parsed_layout.combo_settings.combos[1];
        assert_eq!(combo2.key1, Position::new(1, 0));
        assert_eq!(combo2.key2, Position::new(1, 1));
        assert_eq!(combo2.action, ComboAction::Bootloader);
        assert_eq!(combo2.hold_duration_ms, 750);
    }

    #[test]
    fn test_combos_not_written_when_default() {
        use crate::models::{KeyDefinition, Layer, RgbColor};

        // Create layout with default combo settings (disabled)
        let mut layout = Layout::new("Test Layout").expect("Failed to create layout");

        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_A"));
        layout.add_layer(layer).unwrap();

        // Don't modify combo_settings - leave as default

        // Generate markdown
        let markdown =
            template_gen::generate_markdown(&layout).expect("Failed to generate markdown");

        // Settings section should not exist since everything is default
        assert!(
            !markdown.contains("**Combos**"),
            "Combos should not be written when default"
        );
    }
}
