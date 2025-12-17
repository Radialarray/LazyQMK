//! End-to-end tests for `lazyqmk keycodes` command.

use std::process::Command;

/// Path to the lazyqmk binary
fn lazyqmk_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lazyqmk")
}

// ============================================================================
// List All Keycodes Tests
// ============================================================================

#[test]
fn test_keycodes_list_all() {
    let output = Command::new(lazyqmk_bin())
        .args(["keycodes"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Listing all keycodes should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain some keycodes in human-readable format
    assert!(!stdout.is_empty(), "Output should not be empty");
    // Common keycodes should be present
    assert!(
        stdout.contains("KC_A") || stdout.contains("KC_ESC") || stdout.contains('A'),
        "Output should contain common keycodes"
    );
}

#[test]
fn test_keycodes_list_all_json() {
    let output = Command::new(lazyqmk_bin())
        .args(["keycodes", "--json"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    // Validate JSON structure
    assert!(result["keycodes"].is_array(), "Should have keycodes array");
    assert!(result["count"].is_number(), "Should have count field");

    let count = result["count"].as_u64().unwrap();
    assert!(count > 0, "Should have at least one keycode");

    let keycodes = result["keycodes"]
        .as_array()
        .expect("Should have keycodes array");
    // Most implementations should have 800+ keycodes
    assert!(keycodes.len() > 100, "Should have many keycodes");
}

#[test]
fn test_keycodes_json_structure() {
    let output = Command::new(lazyqmk_bin())
        .args(["keycodes", "--json"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    // Each keycode should have required fields
    if let Some(keycodes) = result["keycodes"].as_array() {
        for keycode in keycodes.iter().take(5) {
            assert!(keycode["code"].is_string(), "Keycode should have code");
            assert!(keycode["label"].is_string(), "Keycode should have label");
            assert!(keycode["category"].is_string(), "Keycode should have category");
            // description is optional
        }
    }
}

// ============================================================================
// Filter by Category Tests
// ============================================================================

#[test]
fn test_keycodes_filter_basic_category() {
    let output = Command::new(lazyqmk_bin())
        .args(["keycodes", "--category", "basic"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Filtering by category should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "Output should not be empty");
    // Should contain keycodes from basic category
    assert!(
        stdout.contains("KC_") || stdout.contains("basic"),
        "Output should contain basic keycodes"
    );
}

#[test]
fn test_keycodes_filter_category_json() {
    let output = Command::new(lazyqmk_bin())
        .args(["keycodes", "--category", "modifiers", "--json"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    assert!(result["keycodes"].is_array());
    let keycodes = result["keycodes"]
        .as_array()
        .expect("Should have keycodes array");
    assert!(!keycodes.is_empty(), "Should have modifier keycodes");

    // All returned keycodes should be in the modifiers category
    for keycode in keycodes {
        assert_eq!(
            keycode["category"].as_str().unwrap(),
            "modifiers",
            "All returned keycodes should be in modifiers category"
        );
    }
}

#[test]
fn test_keycodes_filter_navigation_category() {
    let output = Command::new(lazyqmk_bin())
        .args(["keycodes", "--category", "navigation", "--json"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    let keycodes = result["keycodes"]
        .as_array()
        .expect("Should have keycodes array");
    // Navigation category should have arrow keys, etc.
    assert!(!keycodes.is_empty(), "Should have navigation keycodes");
}

#[test]
fn test_keycodes_invalid_category_fails() {
    let output = Command::new(lazyqmk_bin())
        .args(["keycodes", "--category", "invalid_category_xyz"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Invalid category should fail with exit code 1"
    );
}

// ============================================================================
// Output Format Tests
// ============================================================================

#[test]
fn test_keycodes_table_format_readable() {
    let output = Command::new(lazyqmk_bin())
        .args(["keycodes", "--category", "basic"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should be human-readable table format
    // Typically contains columns or aligned text
    assert!(!stdout.is_empty(), "Output should be readable");
}

#[test]
fn test_keycodes_json_valid_parseable() {
    let output = Command::new(lazyqmk_bin())
        .args(["keycodes", "--json"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should be valid JSON
    let _result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");
}

// ============================================================================
// Keycode Existence Tests
// ============================================================================

#[test]
fn test_keycodes_contains_common_keys() {
    let output = Command::new(lazyqmk_bin())
        .args(["keycodes", "--json"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    let keycodes = result["keycodes"]
        .as_array()
        .expect("Should have keycodes array");
    let codes: Vec<String> = keycodes
        .iter()
        .filter_map(|kc| kc["code"].as_str().map(String::from))
        .collect();

    // Check for common keycodes
    assert!(
        codes.iter().any(|c| c.contains("KC_A")),
        "Should contain letter keycodes"
    );
    assert!(
        codes.iter().any(|c| c.contains("KC_ESC")),
        "Should contain escape key"
    );
}
