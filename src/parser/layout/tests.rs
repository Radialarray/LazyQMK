//! Tests for parser::layout.
//!
//! Auto-extracted from src/parser/layout/mod.rs.
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

use super::*;

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
