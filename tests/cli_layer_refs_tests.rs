//! End-to-end tests for `lazyqmk layer-refs` command.

use std::process::Command;
use lazyqmk::models::{Layer, Layout, RgbColor};

mod fixtures;
use fixtures::*;

/// Path to the lazyqmk binary
fn lazyqmk_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lazyqmk")
}

/// Helper to create a layout with specific layer references
fn test_layout_with_custom_refs() -> Layout {
    let mut layout = test_layout_basic(2, 3);
    
    // Layer 0 has references to layer 1
    let layer1_id = layout.layers[1].id.clone();
    layout.layers[0].keys[0].keycode = format!("MO(@{})", layer1_id);
    layout.layers[0].keys[1].keycode = format!("LT(@{}, KC_SPC)", layer1_id);
    
    layout
}

/// Helper to create a layout with transparency conflicts
fn test_layout_with_transparency_conflict() -> Layout {
    let mut layout = test_layout_basic(2, 3);
    
    // Layer 0 has MO(1) at position (0,0)
    let layer1_id = layout.layers[1].id.clone();
    layout.layers[0].keys[0].keycode = format!("MO(@{})", layer1_id);
    
    // Layer 1 has non-transparent key at same position (conflict!)
    layout.layers[1].keys[0].keycode = "KC_A".to_string();
    
    layout
}

/// Helper to create a layout with self-reference
fn test_layout_with_self_reference() -> Layout {
    let mut layout = test_layout_basic(2, 3);
    
    // Layer 0 references itself
    let layer0_id = layout.layers[0].id.clone();
    layout.layers[0].keys[0].keycode = format!("TG(@{})", layer0_id);
    
    layout
}

/// Helper to create a layout with all reference types
fn test_layout_with_all_ref_types() -> Layout {
    // Create a larger layout to accommodate all reference types (need at least 8 keys)
    let mut layout = test_layout_basic(3, 3); // 3x3 = 9 keys per layer
    
    // Ensure we have at least 3 layers
    if layout.layers.len() < 3 {
        // Create layer 2 with keys matching the geometry
        let mut layer2 = Layer::new(2, "Adjust", RgbColor::new(255, 0, 255)).unwrap();
        for row in 0..3 {
            for col in 0..3 {
                layer2.add_key(lazyqmk::models::KeyDefinition::new(
                    lazyqmk::models::Position::new(row, col),
                    "KC_TRNS"
                ));
            }
        }
        layout.layers.push(layer2);
    }
    
    let layer1_id = layout.layers[1].id.clone();
    let layer2_id = layout.layers[2].id.clone();
    
    // Add various reference types from layer 0
    // We have 9 keys (0,0) to (2,2), so we can use indices 0-7
    if layout.layers[0].keys.len() >= 8 {
        layout.layers[0].keys[0].keycode = format!("MO(@{})", layer1_id);
        layout.layers[0].keys[1].keycode = format!("LT(@{}, KC_SPC)", layer1_id);
        layout.layers[0].keys[2].keycode = format!("TG(@{})", layer1_id);
        layout.layers[0].keys[3].keycode = format!("OSL(@{})", layer2_id);
        layout.layers[0].keys[4].keycode = format!("TO(@{})", layer2_id);
        layout.layers[0].keys[5].keycode = format!("TT(@{})", layer2_id);
        layout.layers[0].keys[6].keycode = format!("DF(@{})", layer2_id);
        layout.layers[0].keys[7].keycode = format!("LM(@{}, MOD_LSFT)", layer2_id);
    }
    
    layout
}

// ============================================================================
// Basic functionality
// ============================================================================

#[test]
fn test_layer_refs_no_refs() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args(["layer-refs", "--layout", layout_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should show both layers
    assert!(stdout.contains("Layer 0: Base"), "Should show layer 0");
    assert!(stdout.contains("Layer 1: Function"), "Should show layer 1");
    
    // Should indicate no references
    assert!(
        stdout.contains("No inbound references"),
        "Should indicate no references"
    );
}

#[test]
fn test_layer_refs_simple_mo() {
    let mut layout = test_layout_basic(2, 3);
    let layer1_id = layout.layers[1].id.clone();
    layout.layers[0].keys[0].keycode = format!("MO(@{})", layer1_id);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args(["layer-refs", "--layout", layout_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Layer 1 should show inbound reference
    assert!(stdout.contains("Layer 1: Function"), "Should show layer 1");
    assert!(stdout.contains("Inbound References:"), "Should have references section");
    assert!(
        stdout.contains("Momentary") || stdout.contains("MO"),
        "Should mention MO reference type"
    );
    assert!(stdout.contains("Layer 0"), "Should mention source layer");
}

#[test]
fn test_layer_refs_multiple_refs() {
    let mut layout = test_layout_basic(2, 3);
    let layer1_id = layout.layers[1].id.clone();
    
    // Add multiple references to layer 1 from layer 0
    layout.layers[0].keys[0].keycode = format!("MO(@{})", layer1_id);
    layout.layers[0].keys[1].keycode = format!("TG(@{})", layer1_id);
    layout.layers[0].keys[2].keycode = format!("LT(@{}, KC_SPC)", layer1_id);
    
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args(["layer-refs", "--layout", layout_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should show all three references
    let reference_count = stdout.matches("Layer 0").count();
    assert!(
        reference_count >= 3,
        "Should show at least 3 references to layer 0, found {}",
        reference_count
    );
    
    // Should show different reference types
    assert!(stdout.contains("Momentary") || stdout.contains("MO"));
    assert!(stdout.contains("Toggle") || stdout.contains("TG"));
    assert!(stdout.contains("Tap-Hold") || stdout.contains("LT"));
}

#[test]
fn test_layer_refs_self_reference() {
    let layout = test_layout_with_self_reference();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args(["layer-refs", "--layout", layout_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Layer 0 should show reference to itself
    assert!(stdout.contains("Layer 0: Base"));
    assert!(stdout.contains("Inbound References:"));
    // The reference should be from Layer 0 to Layer 0
    let layer0_section = stdout.split("Layer 1:").next().unwrap();
    assert!(
        layer0_section.contains("Layer 0") && layer0_section.contains("Toggle"),
        "Should show self-reference"
    );
}

#[test]
fn test_layer_refs_json_output() {
    let layout = test_layout_with_custom_refs();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "layer-refs",
            "--layout",
            layout_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    // Validate structure
    assert!(result["layers"].is_array(), "layers should be array");
    let layers = result["layers"].as_array().unwrap();
    assert_eq!(layers.len(), 2, "Should have 2 layers");

    // Check layer structure
    for layer in layers {
        assert!(layer["number"].is_number(), "number should be number");
        assert!(layer["name"].is_string(), "name should be string");
        assert!(layer["inbound_refs"].is_array(), "inbound_refs should be array");
        assert!(layer["warnings"].is_array(), "warnings should be array");
    }

    // Layer 1 should have inbound references
    let layer1 = &layers[1];
    let refs = layer1["inbound_refs"].as_array().unwrap();
    assert!(!refs.is_empty(), "Layer 1 should have inbound references");

    // Check reference structure
    let ref0 = &refs[0];
    assert!(ref0["from_layer"].is_number(), "from_layer should be number");
    assert!(ref0["position"].is_object(), "position should be object");
    assert!(ref0["position"]["row"].is_number(), "row should be number");
    assert!(ref0["position"]["col"].is_number(), "col should be number");
    assert!(ref0["kind"].is_string(), "kind should be string");
    assert!(ref0["keycode"].is_string(), "keycode should be string");
}

// ============================================================================
// Transparency warnings
// ============================================================================

#[test]
fn test_layer_refs_transparency_conflict() {
    let layout = test_layout_with_transparency_conflict();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args(["layer-refs", "--layout", layout_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should show warning for layer 1
    assert!(stdout.contains("Warning"), "Should have warnings section");
    assert!(
        stdout.contains("Non-transparent") || stdout.contains("conflicts"),
        "Should mention transparency conflict"
    );
    assert!(stdout.contains("KC_A"), "Should mention conflicting keycode");
}

#[test]
fn test_layer_refs_transparent_key_no_warning() {
    let mut layout = test_layout_basic(2, 3);
    let layer1_id = layout.layers[1].id.clone();
    
    // Add MO reference
    layout.layers[0].keys[0].keycode = format!("MO(@{})", layer1_id);
    
    // Ensure target key is transparent (it should be KC_TRNS in layer 1 position 0,0)
    // Actually, let's explicitly set it
    layout.layers[1].keys[0].keycode = "KC_TRNS".to_string();
    
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args(["layer-refs", "--layout", layout_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should NOT have warnings because key is transparent
    let layer1_section = if let Some(start) = stdout.find("Layer 1:") {
        if let Some(end) = stdout[start..].find("Layer") {
            &stdout[start..start + end]
        } else {
            &stdout[start..]
        }
    } else {
        ""
    };
    
    assert!(
        !layer1_section.contains("Warning"),
        "Should not have warnings for transparent keys"
    );
}

#[test]
fn test_layer_refs_non_hold_like_no_warning() {
    let mut layout = test_layout_basic(2, 3);
    let layer1_id = layout.layers[1].id.clone();
    
    // Add TG (toggle) reference - NOT hold-like
    layout.layers[0].keys[0].keycode = format!("TG(@{})", layer1_id);
    
    // Target key is non-transparent
    layout.layers[1].keys[0].keycode = "KC_A".to_string();
    
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args(["layer-refs", "--layout", layout_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should NOT have warnings because TG is not hold-like
    let layer1_section = if let Some(start) = stdout.find("Layer 1:") {
        if let Some(end) = stdout[start..].find("\n\n") {
            &stdout[start..start + end]
        } else {
            &stdout[start..]
        }
    } else {
        ""
    };
    
    assert!(
        !layer1_section.contains("Warning"),
        "Should not warn for non-hold-like references"
    );
}

#[test]
fn test_layer_refs_multiple_conflicts() {
    let mut layout = test_layout_basic(2, 3);
    let layer1_id = layout.layers[1].id.clone();
    
    // Add multiple hold-like references
    layout.layers[0].keys[0].keycode = format!("MO(@{})", layer1_id);
    layout.layers[0].keys[1].keycode = format!("LT(@{}, KC_SPC)", layer1_id);
    
    // Both target keys are non-transparent
    layout.layers[1].keys[0].keycode = "KC_A".to_string();
    layout.layers[1].keys[1].keycode = "KC_B".to_string();
    
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args(["layer-refs", "--layout", layout_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should show multiple warnings
    let warning_count = stdout.matches("Non-transparent").count();
    assert!(
        warning_count >= 2,
        "Should show at least 2 warnings, found {}",
        warning_count
    );
    assert!(stdout.contains("KC_A"), "Should mention first conflict");
    assert!(stdout.contains("KC_B"), "Should mention second conflict");
}

// ============================================================================
// Reference types
// ============================================================================

#[test]
fn test_layer_refs_all_types() {
    let layout = test_layout_with_all_ref_types();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args(["layer-refs", "--layout", layout_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should show various reference types
    assert!(
        stdout.contains("Momentary") || stdout.contains("MO"),
        "Should show Momentary"
    );
    assert!(
        stdout.contains("Tap-Hold") || stdout.contains("LT"),
        "Should show Tap-Hold"
    );
    assert!(stdout.contains("Toggle") || stdout.contains("TG"), "Should show Toggle");
    assert!(
        stdout.contains("One-Shot") || stdout.contains("OSL"),
        "Should show One-Shot"
    );
    assert!(stdout.contains("Switch") || stdout.contains("TO"), "Should show Switch");
    assert!(
        stdout.contains("Tap-Toggle") || stdout.contains("TT"),
        "Should show Tap-Toggle"
    );
    assert!(
        stdout.contains("Default") || stdout.contains("DF"),
        "Should show Default Set"
    );
    assert!(
        stdout.contains("Layer-Mod") || stdout.contains("LM"),
        "Should show Layer-Mod"
    );
}

#[test]
fn test_layer_refs_hold_like_only() {
    let mut layout = test_layout_basic(2, 3);
    let layer1_id = layout.layers[1].id.clone();
    
    // Add hold-like references: MO, LT, TT, LM
    layout.layers[0].keys[0].keycode = format!("MO(@{})", layer1_id);
    layout.layers[0].keys[1].keycode = format!("LT(@{}, KC_SPC)", layer1_id);
    layout.layers[0].keys[2].keycode = format!("TT(@{})", layer1_id);
    
    // Add non-hold-like: TG, OSL, TO, DF
    layout.layers[0].keys[3].keycode = format!("TG(@{})", layer1_id);
    layout.layers[0].keys[4].keycode = format!("OSL(@{})", layer1_id);
    
    // Make all target keys non-transparent
    for key in &mut layout.layers[1].keys {
        key.keycode = "KC_X".to_string();
    }
    
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args(["layer-refs", "--layout", layout_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should have warnings for hold-like positions (0,0), (0,1), (0,2)
    // but NOT for (0,3) and (0,4)
    assert!(stdout.contains("Warning"), "Should have warnings");
    
    // Count warnings - should be 3 (for MO, LT, TT)
    let warning_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("Position") && line.contains("conflicts"))
        .collect();
    
    assert!(
        warning_lines.len() >= 3,
        "Should have at least 3 warnings for hold-like refs, found {}",
        warning_lines.len()
    );
}

#[test]
fn test_layer_refs_json_with_warnings() {
    let layout = test_layout_with_transparency_conflict();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "layer-refs",
            "--layout",
            layout_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let layers = result["layers"].as_array().unwrap();
    let layer1 = &layers[1];
    
    // Layer 1 should have warnings
    let warnings = layer1["warnings"].as_array().unwrap();
    assert!(!warnings.is_empty(), "Should have warnings");
    
    // Check warning structure
    let warning0 = &warnings[0];
    assert!(warning0["position"].is_object(), "Should have position");
    assert!(warning0["message"].is_string(), "Should have message");
    
    let message = warning0["message"].as_str().unwrap();
    assert!(
        message.contains("Non-transparent") || message.contains("conflicts"),
        "Warning should mention conflict"
    );
}
