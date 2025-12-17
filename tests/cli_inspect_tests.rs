//! End-to-end tests for `lazyqmk inspect` command.
#![allow(unused_variables)] // Temp dirs must be kept alive even if not directly accessed

use std::process::Command;

mod fixtures;

use fixtures::*;

/// Path to the lazyqmk binary
fn lazyqmk_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lazyqmk")
}

#[test]
fn test_inspect_metadata_json() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "inspect",
            "--layout",
            layout_path.to_str().unwrap(),
            "--section",
            "metadata",
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    // Validate structure
    assert_eq!(result["name"], "Test Layout");
    assert_eq!(result["author"], "Test Suite");
    assert_eq!(result["keyboard"], "test_keyboard");
    assert_eq!(result["layout_variant"], "LAYOUT_test");
    assert!(result["created"].is_string());
    assert!(result["modified"].is_string());
    assert!(result["tags"].is_array());
}

#[test]
fn test_inspect_metadata_plain() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "inspect",
            "--layout",
            layout_path.to_str().unwrap(),
            "--section",
            "metadata",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Metadata:"));
    assert!(stdout.contains("Name:"));
    assert!(stdout.contains("Test Layout"));
    assert!(stdout.contains("Author:"));
    assert!(stdout.contains("Test Suite"));
    assert!(stdout.contains("Keyboard:"));
    assert!(stdout.contains("test_keyboard"));
    assert!(stdout.contains("Layout Variant:"));
    assert!(stdout.contains("LAYOUT_test"));
    assert!(stdout.contains("Created:"));
    assert!(stdout.contains("Modified:"));
}

#[test]
fn test_inspect_layers_json() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "inspect",
            "--layout",
            layout_path.to_str().unwrap(),
            "--section",
            "layers",
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("Should parse JSON");

    // Validate structure
    assert_eq!(result["count"], 2, "Should have 2 layers");
    let layers = result["layers"].as_array().expect("Should have layers array");
    assert_eq!(layers.len(), 2);

    // Check first layer
    assert_eq!(layers[0]["number"], 0);
    assert_eq!(layers[0]["name"], "Base");
    assert_eq!(layers[0]["key_count"], 6);

    // Check second layer
    assert_eq!(layers[1]["number"], 1);
    assert_eq!(layers[1]["name"], "Function");
    assert_eq!(layers[1]["key_count"], 6);
}

#[test]
fn test_inspect_layers_plain() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "inspect",
            "--layout",
            layout_path.to_str().unwrap(),
            "--section",
            "layers",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Layers (2 total):"));
    assert!(stdout.contains("[0] Base (6 keys)"));
    assert!(stdout.contains("[1] Function (6 keys)"));
}

#[test]
fn test_inspect_categories_json() {
    let layout = test_layout_with_categories();
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "inspect",
            "--layout",
            layout_path.to_str().unwrap(),
            "--section",
            "categories",
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("Should parse JSON");

    // Validate structure
    assert!(result["count"].as_u64().unwrap() > 0);
    let categories = result["categories"]
        .as_array()
        .expect("Should have categories array");
    assert!(!categories.is_empty());

    // Check first category has required fields
    assert!(categories[0]["id"].is_string());
    assert!(categories[0]["name"].is_string());
    assert!(categories[0]["color"].is_string());
    assert!(categories[0]["color"]
        .as_str()
        .unwrap()
        .starts_with('#'));
}

#[test]
fn test_inspect_tap_dances_json() {
    let layout = test_layout_with_tap_dances();
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "inspect",
            "--layout",
            layout_path.to_str().unwrap(),
            "--section",
            "tap-dances",
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("Should parse JSON");

    // Validate structure
    assert_eq!(result["count"], 2, "Should have 2 tap dances");
    let tap_dances = result["tap_dances"]
        .as_array()
        .expect("Should have tap_dances array");
    assert_eq!(tap_dances.len(), 2);

    // Note: Tap dances are auto-created from TD() references when layout is loaded,
    // so they will have KC_NO placeholders. The names are extracted from the keycodes.
    // Order may vary based on hash set iteration, so we just check structure and presence.
    
    // Find the tap dances by name (order not guaranteed)
    let td_names: Vec<&str> = tap_dances
        .iter()
        .filter_map(|td| td["name"].as_str())
        .collect();
    
    assert!(td_names.contains(&"esc_caps"), "Should have esc_caps tap dance");
    assert!(td_names.contains(&"shift_ctrl"), "Should have shift_ctrl tap dance");
    
    // Check that each tap dance has required fields
    for td in tap_dances {
        assert!(td["name"].is_string());
        assert!(td["single_tap"].is_string());
        assert!(td["type"].is_string());
        // double_tap and hold may be null for auto-created placeholders
    }
}

#[test]
fn test_inspect_tap_dances_plain() {
    let layout = test_layout_with_tap_dances();
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "inspect",
            "--layout",
            layout_path.to_str().unwrap(),
            "--section",
            "tap-dances",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tap Dances (2 total):"));
    assert!(stdout.contains("esc_caps"));
    assert!(stdout.contains("shift_ctrl"));
    // Note: Auto-created tap dances will have KC_NO placeholders
    assert!(stdout.contains("Single:"));
}

#[test]
fn test_inspect_settings_json() {
    let layout = test_layout_with_idle_effect(true);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "inspect",
            "--layout",
            layout_path.to_str().unwrap(),
            "--section",
            "settings",
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("Should parse JSON");

    // Validate structure
    assert!(result["rgb_enabled"].is_boolean());
    assert!(result["rgb_brightness"].is_number());
    assert!(result["rgb_timeout_ms"].is_number());
    assert!(result["idle_effect_enabled"].is_boolean());
    assert!(result["idle_effect_timeout_ms"].is_number());
    assert!(result["idle_effect_duration_ms"].is_number());
    assert!(result["idle_effect_mode"].is_string());
}

#[test]
fn test_inspect_settings_plain() {
    let layout = test_layout_with_idle_effect(true);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "inspect",
            "--layout",
            layout_path.to_str().unwrap(),
            "--section",
            "settings",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("RGB Settings:"));
    assert!(stdout.contains("Enabled:"));
    assert!(stdout.contains("Brightness:"));
    assert!(stdout.contains("Timeout:"));
    assert!(stdout.contains("Idle Effect Settings:"));
}

#[test]
fn test_inspect_invalid_section() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "inspect",
            "--layout",
            layout_path.to_str().unwrap(),
            "--section",
            "invalid_section",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Invalid section should exit with code 1"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid section"),
        "Should mention invalid section"
    );
}

#[test]
fn test_inspect_nonexistent_file() {
    let output = Command::new(lazyqmk_bin())
        .args([
            "inspect",
            "--layout",
            "/tmp/nonexistent_layout_file_xyz.md",
            "--section",
            "metadata",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Nonexistent file should exit with code 2"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error:"),
        "Should have error message on stderr"
    );
}
