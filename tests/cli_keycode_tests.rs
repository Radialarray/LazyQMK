//! End-to-end tests for `lazyqmk keycode` command.
#![allow(unused_variables)] // Temp dirs must be kept alive even if not directly accessed

use std::process::Command;

mod fixtures;

use fixtures::*;

/// Path to the lazyqmk binary
fn lazyqmk_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lazyqmk")
}

#[test]
fn test_keycode_resolve_lt_json() {
    let layout = test_layout_with_layer_refs();
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    // Get the UUID of layer 1 (Function layer)
    let layer_uuid = &layout.layers[1].id;
    let expr = format!("LT(@{}, KC_SPC)", layer_uuid);

    let output = Command::new(lazyqmk_bin())
        .args([
            "keycode",
            "--layout",
            layout_path.to_str().unwrap(),
            "--expr",
            &expr,
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Should resolve successfully. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("Should parse JSON");

    // Validate structure
    assert_eq!(result["input"], expr);
    assert_eq!(result["resolved"], "LT(1, KC_SPC)");
    assert_eq!(result["layer_name"], "Function");
    assert_eq!(result["valid"], true);
}

#[test]
fn test_keycode_resolve_lt_plain() {
    let layout = test_layout_with_layer_refs();
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let layer_uuid = &layout.layers[1].id;
    let expr = format!("LT(@{}, KC_SPC)", layer_uuid);

    let output = Command::new(lazyqmk_bin())
        .args([
            "keycode",
            "--layout",
            layout_path.to_str().unwrap(),
            "--expr",
            &expr,
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Input:"));
    assert!(stdout.contains("Resolved: LT(1, KC_SPC)"));
    assert!(stdout.contains("Layer:    Function"));
    assert!(stdout.contains("✓ Valid"));
}

#[test]
fn test_keycode_resolve_mo() {
    let layout = test_layout_with_layer_refs();
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let layer_uuid = &layout.layers[1].id;
    let expr = format!("MO(@{})", layer_uuid);

    let output = Command::new(lazyqmk_bin())
        .args([
            "keycode",
            "--layout",
            layout_path.to_str().unwrap(),
            "--expr",
            &expr,
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("Should parse JSON");

    assert_eq!(result["resolved"], "MO(1)");
    assert_eq!(result["layer_name"], "Function");
    assert_eq!(result["valid"], true);
}

#[test]
fn test_keycode_resolve_tg() {
    let layout = test_layout_with_layer_refs();
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let layer_uuid = &layout.layers[1].id;
    let expr = format!("TG(@{})", layer_uuid);

    let output = Command::new(lazyqmk_bin())
        .args([
            "keycode",
            "--layout",
            layout_path.to_str().unwrap(),
            "--expr",
            &expr,
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("Should parse JSON");

    assert_eq!(result["resolved"], "TG(1)");
    assert_eq!(result["layer_name"], "Function");
    assert_eq!(result["valid"], true);
}

#[test]
fn test_keycode_resolve_lm() {
    let layout = test_layout_with_layer_refs();
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let layer_uuid = &layout.layers[1].id;
    let expr = format!("LM(@{}, MOD_LSFT)", layer_uuid);

    let output = Command::new(lazyqmk_bin())
        .args([
            "keycode",
            "--layout",
            layout_path.to_str().unwrap(),
            "--expr",
            &expr,
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("Should parse JSON");

    assert_eq!(result["resolved"], "LM(1, MOD_LSFT)");
    assert_eq!(result["layer_name"], "Function");
    assert_eq!(result["valid"], true);
}

#[test]
fn test_keycode_resolve_invalid_uuid() {
    let layout = test_layout_with_layer_refs();
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let expr = "MO(@invalid-uuid-that-does-not-exist)";

    let output = Command::new(lazyqmk_bin())
        .args([
            "keycode",
            "--layout",
            layout_path.to_str().unwrap(),
            "--expr",
            expr,
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Invalid UUID should exit with code 1"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error:"),
        "Should have error message on stderr"
    );
}

#[test]
fn test_keycode_passthrough_simple() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let expr = "KC_A";

    let output = Command::new(lazyqmk_bin())
        .args([
            "keycode",
            "--layout",
            layout_path.to_str().unwrap(),
            "--expr",
            expr,
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Simple keycode should pass through"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("Should parse JSON");

    assert_eq!(result["input"], "KC_A");
    assert_eq!(result["resolved"], "KC_A");
    assert!(result["layer_name"].is_null());
    assert_eq!(result["valid"], true);
}

#[test]
fn test_keycode_passthrough_complex() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    let expr = "LSFT(KC_A)";

    let output = Command::new(lazyqmk_bin())
        .args([
            "keycode",
            "--layout",
            layout_path.to_str().unwrap(),
            "--expr",
            expr,
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Complex keycode without UUID should pass through"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("Should parse JSON");

    assert_eq!(result["input"], "LSFT(KC_A)");
    assert_eq!(result["resolved"], "LSFT(KC_A)");
    assert!(result["layer_name"].is_null());
    assert_eq!(result["valid"], true);
}

#[test]
fn test_keycode_nonexistent_file() {
    let output = Command::new(lazyqmk_bin())
        .args([
            "keycode",
            "--layout",
            "/tmp/nonexistent_layout_xyz.md",
            "--expr",
            "KC_A",
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

#[test]
fn test_keycode_resolve_layer_0() {
    let layout = test_layout_with_layer_refs();
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);

    // Test resolving to layer 0 (Base)
    let layer_uuid = &layout.layers[0].id;
    let expr = format!("MO(@{})", layer_uuid);

    let output = Command::new(lazyqmk_bin())
        .args([
            "keycode",
            "--layout",
            layout_path.to_str().unwrap(),
            "--expr",
            &expr,
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("Should parse JSON");

    assert_eq!(result["resolved"], "MO(0)");
    assert_eq!(result["layer_name"], "Base");
    assert_eq!(result["valid"], true);
}
