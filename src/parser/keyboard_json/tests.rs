//! Tests for keyboard_json.
//!
//! Auto-extracted from keyboard_json.rs.

use super::*;

use super::*;
use tempfile::TempDir;

fn create_test_info_json() -> String {
    r#"{
            "keyboard_name": "test_keyboard",
            "manufacturer": "Test Manufacturer",
            "layouts": {
                "LAYOUT": {
                    "layout": [
                        {"x": 0, "y": 0, "matrix": [0, 0]},
                        {"x": 1, "y": 0, "matrix": [0, 1]},
                        {"x": 2, "y": 0, "matrix": [0, 2]},
                        {"x": 0, "y": 1, "matrix": [1, 0]},
                        {"x": 1, "y": 1, "matrix": [1, 1]},
                        {"x": 2, "y": 1, "matrix": [1, 2]}
                    ]
                },
                "LAYOUT_split": {
                    "layout": [
                        {"x": 0, "y": 0, "matrix": [0, 0], "w": 1.5},
                        {"x": 1.5, "y": 0, "matrix": [0, 1]},
                        {"x": 5, "y": 0, "matrix": [4, 0]},
                        {"x": 6, "y": 0, "matrix": [4, 1]}
                    ]
                }
            }
        }"#
    .to_string()
}

#[test]
fn test_parse_info_json() {
    let temp_dir = TempDir::new().unwrap();
    let info_path = temp_dir.path().join("info.json");
    fs::write(&info_path, create_test_info_json()).unwrap();

    let info = parse_info_json(&info_path).unwrap();
    assert_eq!(info.keyboard_name, Some("test_keyboard".to_string()));
    assert_eq!(info.layouts.len(), 2);
    assert!(info.layouts.contains_key("LAYOUT"));
    assert!(info.layouts.contains_key("LAYOUT_split"));
}

#[test]
fn test_extract_layout_names() {
    let temp_dir = TempDir::new().unwrap();
    let info_path = temp_dir.path().join("info.json");
    fs::write(&info_path, create_test_info_json()).unwrap();

    let info = parse_info_json(&info_path).unwrap();
    let names = extract_layout_names(&info);

    assert_eq!(names.len(), 2);
    assert!(names.contains(&"LAYOUT".to_string()));
    assert!(names.contains(&"LAYOUT_split".to_string()));
}

#[test]
fn test_extract_layout_variants() {
    let temp_dir = TempDir::new().unwrap();
    let info_path = temp_dir.path().join("info.json");
    fs::write(&info_path, create_test_info_json()).unwrap();

    let info = parse_info_json(&info_path).unwrap();
    let variants = extract_layout_variants(&info);

    assert_eq!(variants.len(), 2);

    // Check LAYOUT variant
    let layout = variants.iter().find(|v| v.name == "LAYOUT").unwrap();
    assert_eq!(layout.key_count, 6);

    // Check LAYOUT_split variant
    let split = variants.iter().find(|v| v.name == "LAYOUT_split").unwrap();
    assert_eq!(split.key_count, 4);
}

#[test]
#[allow(clippy::float_cmp)]
fn test_extract_layout_definition() {
    let temp_dir = TempDir::new().unwrap();
    let info_path = temp_dir.path().join("info.json");
    fs::write(&info_path, create_test_info_json()).unwrap();

    let info = parse_info_json(&info_path).unwrap();

    let layout = extract_layout_definition(&info, "LAYOUT").unwrap();
    assert_eq!(layout.layout.len(), 6);

    let split_layout = extract_layout_definition(&info, "LAYOUT_split").unwrap();
    assert_eq!(split_layout.layout.len(), 4);
    assert_eq!(split_layout.layout[0].w, 1.5);
}

#[test]
#[allow(clippy::float_cmp)]
fn test_build_keyboard_geometry() {
    let temp_dir = TempDir::new().unwrap();
    let info_path = temp_dir.path().join("info.json");
    fs::write(&info_path, create_test_info_json()).unwrap();

    let info = parse_info_json(&info_path).unwrap();
    let geometry = build_keyboard_geometry(&info, "test_keyboard", "LAYOUT").unwrap();

    assert_eq!(geometry.keyboard_name, "test_keyboard");
    assert_eq!(geometry.layout_name, "LAYOUT");
    assert_eq!(geometry.matrix_rows, 2);
    assert_eq!(geometry.matrix_cols, 3);
    assert_eq!(geometry.keys.len(), 6);

    // Check first key
    assert_eq!(geometry.keys[0].matrix_position, (0, 0));
    assert_eq!(geometry.keys[0].led_index, 0);
    assert_eq!(geometry.keys[0].visual_x, 0.0);
    assert_eq!(geometry.keys[0].visual_y, 0.0);
}

#[test]
#[allow(clippy::float_cmp)]
fn test_build_keyboard_geometry_with_split_layout() {
    let temp_dir = TempDir::new().unwrap();
    let info_path = temp_dir.path().join("info.json");
    fs::write(&info_path, create_test_info_json()).unwrap();

    let info = parse_info_json(&info_path).unwrap();
    let geometry = build_keyboard_geometry(&info, "test_keyboard", "LAYOUT_split").unwrap();

    assert_eq!(geometry.matrix_rows, 5); // max row is 4, so rows = 5
    assert_eq!(geometry.matrix_cols, 2);
    assert_eq!(geometry.keys.len(), 4);

    // Check key with custom width
    assert_eq!(geometry.keys[0].width, 1.5);
}

#[test]
fn test_scan_keyboards_invalid_path() {
    let temp_dir = TempDir::new().unwrap();

    // Test with a path that has no keyboards directory
    let result = scan_keyboards(temp_dir.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("QMK keyboards directory not found"));
}

// Note: Testing scan_keyboards with actual QMK requires the QMK CLI to be installed
// and a valid QMK repository. See tests/qmk_info_json_tests.rs for integration tests
// that test against the actual QMK firmware submodule.
