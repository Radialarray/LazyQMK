//! End-to-end tests for `lazyqmk tap-dance` command.

use std::fs;
use std::process::Command;

mod fixtures;
use fixtures::*;

/// Path to the lazyqmk binary
fn lazyqmk_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lazyqmk")
}

// ============================================================================
// tap-dance list
// ============================================================================

#[test]
fn test_tap_dance_list_empty() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args(["tap-dance", "list", "--layout", layout_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Empty output expected when no tap dances
    assert_eq!(stdout.trim(), "", "Should have no output for empty list");
}

#[test]
fn test_tap_dance_list_with_dances() {
    let layout = test_layout_with_tap_dances();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args(["tap-dance", "list", "--layout", layout_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should show both tap dances
    assert!(stdout.contains("esc_caps"), "Should list esc_caps");
    assert!(stdout.contains("shift_ctrl"), "Should list shift_ctrl");
    
    // Should show action details
    assert!(stdout.contains("single=KC_ESC"), "Should show single tap");
    assert!(stdout.contains("double=KC_CAPS"), "Should show double tap");
    assert!(stdout.contains("hold=KC_LCTL"), "Should show hold for 3-way");
}

#[test]
fn test_tap_dance_list_json() {
    let layout = test_layout_with_tap_dances();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "list",
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
    assert_eq!(result["count"], 2, "Should have 2 tap dances");
    
    let tap_dances = result["tap_dances"].as_array().expect("Should be array");
    assert_eq!(tap_dances.len(), 2);

    // Check first tap dance (2-way)
    let td1 = &tap_dances[0];
    assert_eq!(td1["name"], "esc_caps");
    assert_eq!(td1["single_tap"], "KC_ESC");
    assert_eq!(td1["double_tap"], "KC_CAPS");
    assert_eq!(td1["hold"], serde_json::Value::Null);
    assert_eq!(td1["type"], "two_way");

    // Check second tap dance (3-way)
    let td2 = &tap_dances[1];
    assert_eq!(td2["name"], "shift_ctrl");
    assert_eq!(td2["single_tap"], "KC_LSFT");
    assert_eq!(td2["double_tap"], "KC_CAPS");
    assert_eq!(td2["hold"], "KC_LCTL");
    assert_eq!(td2["type"], "three_way");
}

// ============================================================================
// tap-dance add
// ============================================================================

#[test]
fn test_tap_dance_add_two_way() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "add",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "tab_esc",
            "--single",
            "KC_TAB",
            "--double",
            "KC_ESC",
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
    assert!(stdout.contains("Successfully added"), "Should confirm success");
    assert!(stdout.contains("tab_esc"), "Should mention tap dance name");

    // Verify it was added to the file
    let content = fs::read_to_string(&layout_path).expect("Should read file");
    assert!(content.contains("## Tap Dances"), "Should have tap dance section");
    assert!(content.contains("**tab_esc**"), "Should have tap dance in file");
    assert!(content.contains("Single Tap: KC_TAB"), "Should have single tap");
    assert!(content.contains("Double Tap: KC_ESC"), "Should have double tap");
}

#[test]
fn test_tap_dance_add_three_way() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "add",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "space_layer",
            "--single",
            "KC_SPC",
            "--double",
            "KC_ENT",
            "--hold",
            "MO(1)",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    // Verify it was added with all three actions
    let content = fs::read_to_string(&layout_path).expect("Should read file");
    assert!(content.contains("**space_layer**"));
    assert!(content.contains("Single Tap: KC_SPC"));
    assert!(content.contains("Double Tap: KC_ENT"));
    assert!(content.contains("Hold: MO(1)"));
}

#[test]
fn test_tap_dance_add_duplicate_name() {
    let layout = test_layout_with_tap_dances();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    // Try to add a tap dance with existing name
    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "add",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "esc_caps", // Already exists
            "--single",
            "KC_A",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Should fail with validation error"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already exists") || stderr.contains("duplicate"),
        "Should mention duplicate name"
    );
}

#[test]
fn test_tap_dance_add_invalid_name() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    // Try to add a tap dance with invalid C identifier
    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "add",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "invalid-name!", // Contains invalid characters
            "--single",
            "KC_A",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Should fail with validation error"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("alphanumeric") || stderr.contains("valid") || stderr.contains("identifier"),
        "Should mention invalid name format. stderr: {}",
        stderr
    );
}

#[test]
fn test_tap_dance_add_and_verify() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    // Add a tap dance
    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "add",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "test_td",
            "--single",
            "KC_A",
            "--double",
            "KC_B",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    // List to verify it's there
    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "list",
            "--layout",
            layout_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(result["count"], 1);
    assert_eq!(result["tap_dances"][0]["name"], "test_td");
}

#[test]
fn test_tap_dance_add_preserves_formatting() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    // Read original content
    // Add a tap dance
    Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "add",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "test_td",
            "--single",
            "KC_A",
        ])
        .output()
        .expect("Failed to execute command");

    // Read modified content
    let modified = fs::read_to_string(&layout_path).expect("Should read file");

    // Should preserve metadata
    assert!(modified.contains("name: Test Layout"), "Should preserve layout name");
    assert!(modified.contains("keyboard: test_keyboard"), "Should preserve keyboard");
    
    // Should add tap dance section
    assert!(modified.contains("## Tap Dances"), "Should have tap dance section");
    assert!(modified.contains("**test_td**"), "Should have new tap dance");
}

// ============================================================================
// tap-dance delete
// ============================================================================

#[test]
fn test_tap_dance_delete_unused() {
    let mut layout = test_layout_with_tap_dances();
    // Remove references but keep definitions
    layout.layers[0].keys[0].keycode = "KC_A".to_string();
    layout.layers[0].keys[1].keycode = "KC_B".to_string();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "delete",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "esc_caps",
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
    assert!(stdout.contains("Successfully deleted"), "Should confirm deletion");

    // Verify it was removed
    let content = fs::read_to_string(&layout_path).expect("Should read file");
    assert!(!content.contains("name: esc_caps"), "Should not have tap dance in file");
}

#[test]
fn test_tap_dance_delete_not_found() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "delete",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "nonexistent",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Should fail when tap dance not found"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("nonexistent"),
        "Should mention tap dance not found"
    );
}

#[test]
fn test_tap_dance_delete_referenced() {
    let layout = test_layout_with_tap_dances();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    // Try to delete a tap dance that's in use
    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "delete",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "esc_caps",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Should fail when tap dance is referenced"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("referenced") || stderr.contains("in use") || stderr.contains("--force"),
        "Should mention references and --force option. stderr: {}",
        stderr
    );
}

#[test]
fn test_tap_dance_delete_referenced_force() {
    let layout = test_layout_with_tap_dances();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    // Delete with --force should succeed
    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "delete",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "esc_caps",
            "--force",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Should succeed with --force. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Successfully deleted"), "Should confirm deletion");
    assert!(
        stdout.contains("replaced") || stdout.contains("KC_TRNS"),
        "Should mention reference replacement"
    );

    // Verify definition removed and references replaced
    let content = fs::read_to_string(&layout_path).expect("Should read file");
    assert!(!content.contains("**esc_caps**"), "Definition should be removed");
    assert!(!content.contains("TD(esc_caps)"), "References should be replaced");
}

#[test]
fn test_tap_dance_delete_and_verify() {
    let mut layout = test_layout_with_tap_dances();
    // Remove references
    layout.layers[0].keys[0].keycode = "KC_A".to_string();
    layout.layers[0].keys[1].keycode = "KC_B".to_string();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    // Delete a tap dance
    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "delete",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "esc_caps",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    // List to verify it's gone
    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "list",
            "--layout",
            layout_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should only have one remaining (shift_ctrl)
    assert_eq!(result["count"], 1);
    assert_eq!(result["tap_dances"][0]["name"], "shift_ctrl");
}

// ============================================================================
// tap-dance validate
// ============================================================================

#[test]
fn test_tap_dance_validate_all_valid() {
    let layout = test_layout_with_tap_dances();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "validate",
            "--layout",
            layout_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Should succeed when all valid. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✓") || stdout.contains("valid") || stdout.contains("passed"),
        "Should indicate validation passed"
    );
}

#[test]
fn test_tap_dance_validate_orphaned_ref() {
    // Note: The system auto-creates placeholder tap dances for TD() references,
    // so "orphaned" references actually become "unused" definitions with KC_NO placeholders.
    // This test verifies that auto-created placeholders are detected properly.
    
    let mut layout = test_layout_basic(2, 3);
    // Add reference - this will auto-create a placeholder definition when loaded
    layout.layers[0].keys[0].keycode = "TD(undefined_td)".to_string();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "validate",
            "--layout",
            layout_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // Should pass validation because auto-create makes a placeholder
    assert_eq!(
        output.status.code(),
        Some(0),
        "Should pass (auto-created placeholder). stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show as valid since the reference exists (auto-created)
    assert!(
        stdout.contains("✓") || stdout.contains("valid"),
        "Should validate successfully with auto-created placeholder"
    );
}

#[test]
fn test_tap_dance_validate_unused_definition() {
    let mut layout = test_layout_with_tap_dances();
    // Remove all references but keep definitions
    layout.layers[0].keys[0].keycode = "KC_A".to_string();
    layout.layers[0].keys[1].keycode = "KC_B".to_string();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "validate",
            "--layout",
            layout_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // Unused definitions are warnings, not errors (exit 0)
    assert_eq!(
        output.status.code(),
        Some(0),
        "Should succeed (warnings only). stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("unused") || stdout.contains("never used") || stdout.contains("⚠"),
        "Should mention unused definitions"
    );
}

#[test]
fn test_tap_dance_validate_json() {
    let layout = test_layout_with_tap_dances();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "validate",
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
    assert!(result["valid"].is_boolean(), "valid should be boolean");
    assert_eq!(result["valid"], true, "Should be valid");
    
    assert!(result["orphaned"].is_array(), "orphaned should be array");
    assert_eq!(
        result["orphaned"].as_array().unwrap().len(),
        0,
        "Should have no orphaned refs"
    );
    
    assert!(result["unused"].is_array(), "unused should be array");
    assert_eq!(
        result["unused"].as_array().unwrap().len(),
        0,
        "Should have no unused definitions"
    );
}

// ============================================================================
// End-to-end flows
// ============================================================================

#[test]
fn test_tap_dance_full_workflow() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    // 1. Add a tap dance
    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "add",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "workflow_td",
            "--single",
            "KC_A",
            "--double",
            "KC_B",
        ])
        .output()
        .expect("Failed to execute command");
    assert_eq!(output.status.code(), Some(0), "Add should succeed");

    // 2. Validate (should pass with warning about unused)
    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "validate",
            "--layout",
            layout_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");
    assert_eq!(output.status.code(), Some(0), "Validate should pass");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("unused") || stdout.contains("never used"),
        "Should warn about unused definition"
    );

    // 3. List (should show 1 tap dance)
    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "list",
            "--layout",
            layout_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute command");
    assert_eq!(output.status.code(), Some(0), "List should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(result["count"], 1);

    // 4. Delete
    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "delete",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "workflow_td",
        ])
        .output()
        .expect("Failed to execute command");
    assert_eq!(output.status.code(), Some(0), "Delete should succeed");

    // 5. Validate again (should pass with no warnings)
    let output = Command::new(lazyqmk_bin())
        .args([
            "tap-dance",
            "validate",
            "--layout",
            layout_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");
    assert_eq!(output.status.code(), Some(0), "Final validate should pass");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✓") || stdout.contains("valid"),
        "Should show clean validation"
    );
}

#[test]
#[ignore = "Requires QMK CLI and full firmware generation pipeline"]
fn test_tap_dance_add_use_generate() {
    // This test would:
    // 1. Add a tap dance definition
    // 2. Use it in a key (manually modify layout file)
    // 3. Run generate command
    // 4. Verify generated firmware has tap dance code
    // Marked as ignore since it requires QMK setup
}
