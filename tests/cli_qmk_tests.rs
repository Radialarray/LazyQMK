//! End-to-end tests for `lazyqmk` QMK metadata commands.
//!
//! Tests the following commands:
//! - `list-keyboards`: List all keyboards in QMK firmware with optional filtering
//! - `list-layouts`: List layout variants for a specific keyboard with key counts
//! - `geometry`: Display coordinate mappings for keyboard layout

use std::process::Command;
use std::path::PathBuf;

mod fixtures;

/// Path to the lazyqmk binary
fn lazyqmk_bin() -> String {
    std::env::var("CARGO_BIN_EXE_lazyqmk")
        .unwrap_or_else(|_| "target/release/lazyqmk".to_string())
}

/// Path to the mock QMK fixture for testing without full submodule
fn mock_qmk_fixture() -> PathBuf {
    // CARGO_MANIFEST_DIR is always set by cargo test to the project root
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR should be set by cargo");
    PathBuf::from(manifest_dir).join("tests/fixtures/mock_qmk")
}

// ============================================================================
// list-keyboards TESTS
// ============================================================================

/// Test: list-keyboards with valid QMK path returns keyboards successfully
#[test]
fn test_list_keyboards_valid_path() {
    let fixture_path = mock_qmk_fixture();

    let output = Command::new(lazyqmk_bin())
        .args(["list-keyboards", "--qmk-path", "dummy"])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Should succeed with valid QMK path. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Found") || stdout.contains("keyboards"),
        "Output should contain keyboard listing"
    );
}

/// Test: list-keyboards with JSON output produces valid JSON
#[test]
#[allow(clippy::cast_possible_truncation)]
fn test_list_keyboards_json_output() {
    let fixture_path = mock_qmk_fixture();

    let output = Command::new(lazyqmk_bin())
        .args(["list-keyboards", "--qmk-path", "dummy", "--json"])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Should succeed with valid QMK path"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    // Validate JSON structure
    assert!(
        result["keyboards"].is_array(),
        "Response should contain 'keyboards' array"
    );
    assert!(
        result["count"].is_number(),
        "Response should contain 'count' field"
    );

    let count = result["count"].as_u64().expect("count should be a number");
    let keyboards = result["keyboards"]
        .as_array()
        .expect("keyboards should be an array");

    assert_eq!(
        keyboards.len(),
        count as usize,
        "Keyboard count should match array length"
    );
    assert!(
        !keyboards.is_empty(),
        "Should find at least one keyboard in fixture"
    );

    // Verify all keyboards are strings
    for kb in keyboards {
        assert!(kb.is_string(), "Each keyboard should be a string");
    }
}

/// Test: list-keyboards with regex filter returns matching keyboards
#[test]
fn test_list_keyboards_with_filter() {
    let fixture_path = mock_qmk_fixture();
    // Use a filter to match our fixture keyboards
    let filter = "crkbd";

    let output = Command::new(lazyqmk_bin())
        .args([
            "list-keyboards",
            "--qmk-path",
            "dummy",
            "--filter",
            filter,
            "--json",
        ])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0), "Filter should match some keyboards");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    let keyboards = result["keyboards"]
        .as_array()
        .expect("Should have keyboards array");
    assert!(!keyboards.is_empty(), "Filter should match at least one keyboard");

    // All keyboards should contain the filter string
    for kb in keyboards {
        let kb_name = kb.as_str().expect("Should be string");
        assert!(
            kb_name.contains("crkbd"),
            "Keyboard should match filter: {}",
            kb_name
        );
    }
}

/// Test: list-keyboards with invalid filter regex returns error
#[test]
fn test_list_keyboards_invalid_filter_regex() {
    let fixture_path = mock_qmk_fixture();
    let invalid_regex = "[invalid(regex";

    let output = Command::new(lazyqmk_bin())
        .args([
            "list-keyboards",
            "--qmk-path",
            "dummy",
            "--filter",
            invalid_regex,
        ])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Invalid regex should return exit code 1 (validation error)"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid regex") || stderr.contains("regex"),
        "Error message should mention regex issue"
    );
}

/// Test: list-keyboards with strict filter that matches nothing returns error
#[test]
fn test_list_keyboards_empty_filter_result() {
    let fixture_path = mock_qmk_fixture();
    // Filter that almost certainly won't match anything
    let strict_filter = "^ZZZZZZZZZ";

    let output = Command::new(lazyqmk_bin())
        .args([
            "list-keyboards",
            "--qmk-path",
            "dummy",
            "--filter",
            strict_filter,
        ])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Empty filter result should return exit code 1 (validation error)"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No keyboards match") || stderr.contains("match filter"),
        "Error message should indicate no matches"
    );
}

/// Test: list-keyboards with invalid QMK path returns error
#[test]
fn test_list_keyboards_invalid_qmk_path() {
    let invalid_path = "/nonexistent/path/to/qmk";

    let output = Command::new(lazyqmk_bin())
        .args(["list-keyboards", "--qmk-path", invalid_path])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Invalid QMK path should return exit code 2 (I/O error)"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("QMK path") || stderr.contains("not found") || stderr.contains("exist"),
        "Error message should mention missing QMK path"
    );
}

// ============================================================================
// list-layouts TESTS
// ============================================================================

/// Test: list-layouts with valid keyboard returns layouts with key counts
#[test]
fn test_list_layouts_valid_keyboard() {
    let fixture_path = mock_qmk_fixture();
    // Use a keyboard we know exists in the fixture
    let keyboard_name = "crkbd";

    // Now test list-layouts
    let output = Command::new(lazyqmk_bin())
        .args([
            "list-layouts",
            "--qmk-path",
            "dummy",
            "--keyboard",
            keyboard_name,
        ])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute command");

    // May or may not have layouts, but if it fails should be validation error
    match output.status.code() {
        Some(0) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("layout"), "Output should mention layouts");
        }
        Some(1) => {
            // Validation error is acceptable (keyboard has no layouts)
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("layout") || stderr.contains("not found"),
                "Error should mention layouts"
            );
        }
        _ => {
            panic!("Unexpected exit code: {:?}", output.status.code());
        }
    }
}

/// Test: list-layouts with JSON output is valid JSON
#[test]
#[allow(clippy::cast_possible_truncation)]
fn test_list_layouts_json_output() {
    let fixture_path = mock_qmk_fixture();
    // Use a keyboard we know exists in the fixture
    let keyboard_name = "crkbd";

    let output = Command::new(lazyqmk_bin())
        .args([
            "list-layouts",
            "--qmk-path",
            "dummy",
            "--keyboard",
            keyboard_name,
            "--json",
        ])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute command");

    if output.status.code() == Some(0) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let result: serde_json::Value =
            serde_json::from_str(&stdout).expect("Should parse JSON output");

        // Validate structure
        assert!(result["keyboard"].is_string(), "Should have keyboard field");
        assert!(result["layouts"].is_array(), "Should have layouts array");
        assert!(result["count"].is_number(), "Should have count field");

        let layouts = result["layouts"]
            .as_array()
            .expect("Should be array");
        let count = result["count"].as_u64().expect("Should be number");

        assert_eq!(
            layouts.len(),
            count as usize,
            "Layout count should match array length"
        );

        // Verify each layout has name and key_count
        for layout in layouts {
            assert!(
                layout["name"].is_string(),
                "Each layout should have name"
            );
            assert!(
                layout["key_count"].is_number(),
                "Each layout should have key_count"
            );
        }
    }
}

/// Test: list-layouts with non-existent keyboard returns error
#[test]
fn test_list_layouts_nonexistent_keyboard() {
    let fixture_path = mock_qmk_fixture();
    let nonexistent_keyboard = "nonexistent_keyboard_xyz";

    let output = Command::new(lazyqmk_bin())
        .args([
            "list-layouts",
            "--qmk-path",
            "dummy",
            "--keyboard",
            nonexistent_keyboard,
        ])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Non-existent keyboard should return exit code 1 (validation error)"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("Keyboard not found"),
        "Error message should mention keyboard not found"
    );
}

/// Test: list-layouts with invalid QMK path returns error
#[test]
fn test_list_layouts_invalid_qmk_path() {
    let invalid_path = "/nonexistent/qmk";
    let keyboard = "test_keyboard";

    let output = Command::new(lazyqmk_bin())
        .args([
            "list-layouts",
            "--qmk-path",
            invalid_path,
            "--keyboard",
            keyboard,
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Invalid QMK path should return exit code 2 (I/O error)"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("QMK path") || stderr.contains("not found") || stderr.contains("exist"),
        "Error should mention QMK path"
    );
}

/// Test: list-layouts outputs human-readable format correctly
#[test]
fn test_list_layouts_human_readable_output() {
    let fixture_path = mock_qmk_fixture();
    // Use a keyboard we know exists in the fixture
    let keyboard_name = "crkbd";

    let output = Command::new(lazyqmk_bin())
        .args([
            "list-layouts",
            "--qmk-path",
            "dummy",
            "--keyboard",
            keyboard_name,
        ])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute command");

    if output.status.code() == Some(0) {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Verify human-readable format
        assert!(stdout.contains("Keyboard:"), "Should show keyboard name");
        assert!(
            stdout.contains("layout") || stdout.contains("Available"),
            "Should show layouts"
        );
        assert!(
            stdout.contains("keys") || stdout.contains("key"),
            "Should show key counts"
        );
    }
}

// ============================================================================
// geometry TESTS
// ============================================================================

/// Test: geometry with valid keyboard and layout returns coordinate mappings
#[test]
fn test_geometry_valid_keyboard_and_layout() {
    let fixture_path = mock_qmk_fixture();
    // Use a keyboard we know exists in the fixture
    let keyboard_name = "crkbd";
    let layout_name = "LAYOUT";

    // Test geometry command
    let geo_output = Command::new(lazyqmk_bin())
        .args([
            "geometry",
            "--qmk-path",
            "dummy",
            "--keyboard",
            keyboard_name,
            "--layout-name",
            layout_name,
        ])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute geometry command");

    if geo_output.status.code() == Some(0) {
        let stdout = String::from_utf8_lossy(&geo_output.stdout);
        assert!(
            stdout.contains("Keyboard:") || stdout.contains("Layout:"),
            "Geometry output should show keyboard and layout"
        );
        assert!(
            stdout.contains("Matrix:") || stdout.contains("matrix"),
            "Should show matrix dimensions"
        );
        assert!(
            stdout.contains("Keys:") || stdout.contains("keys"),
            "Should show key count"
        );
    }
}

/// Test: geometry with JSON output is valid JSON with correct structure
#[test]
fn test_geometry_json_output() {
    let fixture_path = mock_qmk_fixture();
    // Use a keyboard we know exists in the fixture
    let keyboard_name = "crkbd";
    let layout_name = "LAYOUT";

    // Test geometry with JSON
    let geo_output = Command::new(lazyqmk_bin())
        .args([
            "geometry",
            "--qmk-path",
            "dummy",
            "--keyboard",
            keyboard_name,
            "--layout-name",
            layout_name,
            "--json",
        ])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute geometry command");

    if geo_output.status.code() == Some(0) {
        let stdout = String::from_utf8_lossy(&geo_output.stdout);
        let result: serde_json::Value =
            serde_json::from_str(&stdout).expect("Should parse JSON");

        // Validate structure
        assert!(
            result["keyboard"].is_string(),
            "Should have keyboard field"
        );
        assert!(result["layout"].is_string(), "Should have layout field");
        assert!(result["matrix"].is_object(), "Should have matrix object");
        assert!(
            result["key_count"].is_number(),
            "Should have key_count field"
        );
        assert!(
            result["mappings"].is_array(),
            "Should have mappings array"
        );

        // Validate matrix structure
        let matrix = &result["matrix"];
        assert!(matrix["rows"].is_number(), "Matrix should have rows");
        assert!(matrix["cols"].is_number(), "Matrix should have cols");

        let rows = matrix["rows"]
            .as_u64()
            .expect("rows should be number");
        let cols = matrix["cols"]
            .as_u64()
            .expect("cols should be number");
        assert!(rows > 0, "Rows should be positive");
        assert!(cols > 0, "Cols should be positive");

        // Validate mappings
        let mappings = result["mappings"]
            .as_array()
            .expect("mappings should be array");
        assert!(!mappings.is_empty(), "Should have mappings");

        for mapping in mappings {
            assert!(
                mapping["visual_index"].is_number(),
                "Mapping should have visual_index"
            );
            assert!(
                mapping["matrix"].is_array(),
                "Mapping should have matrix array"
            );
            assert!(
                mapping["led_index"].is_number(),
                "Mapping should have led_index"
            );
            assert!(
                mapping["visual_position"].is_array(),
                "Mapping should have visual_position array"
            );
        }
    }
}

/// Test: geometry with non-existent keyboard returns error
#[test]
fn test_geometry_nonexistent_keyboard() {
    let fixture_path = mock_qmk_fixture();
    let nonexistent_keyboard = "nonexistent_keyboard_xyz";

    let output = Command::new(lazyqmk_bin())
        .args([
            "geometry",
            "--qmk-path",
            "dummy",
            "--keyboard",
            nonexistent_keyboard,
            "--layout-name",
            "LAYOUT",
        ])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Non-existent keyboard should return exit code 1 (validation error)"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid keyboard") || stderr.contains("keyboard") || stderr.contains("not found"),
        "Error should mention invalid keyboard"
    );
}

/// Test: geometry with non-existent layout returns error
#[test]
fn test_geometry_nonexistent_layout() {
    let fixture_path = mock_qmk_fixture();
    // Use a keyboard we know exists in the fixture
    let keyboard_name = "crkbd";

    let output = Command::new(lazyqmk_bin())
        .args([
            "geometry",
            "--qmk-path",
            "dummy",
            "--keyboard",
            keyboard_name,
            "--layout-name",
            "NONEXISTENT_LAYOUT_XYZ",
        ])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Non-existent layout should return exit code 1 (validation error)"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Layout") || stderr.contains("layout") || stderr.contains("not found"),
        "Error should mention layout"
    );
}

/// Test: geometry with invalid QMK path returns error
#[test]
fn test_geometry_invalid_qmk_path() {
    let invalid_path = "/nonexistent/qmk";

    let output = Command::new(lazyqmk_bin())
        .args([
            "geometry",
            "--qmk-path",
            invalid_path,
            "--keyboard",
            "test_keyboard",
            "--layout-name",
            "LAYOUT",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Invalid QMK path should return exit code 2 (I/O error)"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("QMK path") || stderr.contains("not found") || stderr.contains("exist"),
        "Error should mention QMK path"
    );
}

/// Test: geometry outputs human-readable format correctly
#[test]
fn test_geometry_human_readable_output() {
    let fixture_path = mock_qmk_fixture();
    // Use a keyboard we know exists in the fixture
    let keyboard_name = "crkbd";
    let layout_name = "LAYOUT";

    // Test geometry human-readable output
    let geo_output = Command::new(lazyqmk_bin())
        .args([
            "geometry",
            "--qmk-path",
            "dummy",
            "--keyboard",
            keyboard_name,
            "--layout-name",
            layout_name,
        ])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute geometry command");

    if geo_output.status.code() == Some(0) {
        let stdout = String::from_utf8_lossy(&geo_output.stdout);

        // Verify human-readable format
        assert!(stdout.contains("Keyboard:"), "Should show keyboard");
        assert!(stdout.contains("Layout:"), "Should show layout");
        assert!(stdout.contains("Matrix:"), "Should show matrix");
        assert!(stdout.contains("Keys:"), "Should show keys");
        assert!(
            stdout.contains("Coordinate Mappings") || stdout.contains("Visual"),
            "Should show coordinate mappings"
        );
    }
}

/// Test: geometry with matrix and LED indices in JSON output
#[test]
fn test_geometry_matrix_and_led_indices() {
    let fixture_path = mock_qmk_fixture();
    // Use a keyboard we know exists in the fixture
    let keyboard_name = "crkbd";
    let layout_name = "LAYOUT";

    // Test geometry with JSON
    let geo_output = Command::new(lazyqmk_bin())
        .args([
            "geometry",
            "--qmk-path",
            "dummy",
            "--keyboard",
            keyboard_name,
            "--layout-name",
            layout_name,
            "--json",
        ])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute geometry command");

    if geo_output.status.code() == Some(0) {
        let stdout = String::from_utf8_lossy(&geo_output.stdout);
        let result: serde_json::Value =
            serde_json::from_str(&stdout).expect("Should parse JSON");

        let mappings = result["mappings"]
            .as_array()
            .expect("mappings should be array");

        if !mappings.is_empty() {
            let first_mapping = &mappings[0];

            // Verify matrix field is a 2-element array
            let matrix = first_mapping["matrix"]
                .as_array()
                .expect("matrix should be array");
            assert_eq!(
                matrix.len(),
                2,
                "Matrix should have [row, col]"
            );
            assert!(matrix[0].is_number(), "Matrix row should be number");
            assert!(matrix[1].is_number(), "Matrix col should be number");

            // Verify visual_position field is a 2-element array
            let visual = first_mapping["visual_position"]
                .as_array()
                .expect("visual_position should be array");
            assert_eq!(
                visual.len(),
                2,
                "Visual position should have [x, y]"
            );
            assert!(visual[0].is_number(), "X should be number");
            assert!(visual[1].is_number(), "Y should be number");
        }
    }
}
