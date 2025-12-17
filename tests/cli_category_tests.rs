//! End-to-end tests for `lazyqmk category` commands.

use std::process::Command;
use serde::{Deserialize, Serialize};

mod fixtures;
use fixtures::*;

#[derive(Debug, Deserialize, Serialize)]
struct CategoryItem {
    id: String,
    name: String,
    color: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ListCategoriesResponse {
    categories: Vec<CategoryItem>,
    count: usize,
}

/// Path to the lazyqmk binary (set by cargo at compile time)
fn lazyqmk_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lazyqmk")
}

// ============================================================================
// List Command Tests
// ============================================================================

#[test]
fn test_category_list_empty_layout() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "list",
            "--layout",
            layout_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Empty layout should list successfully. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No categories") || stdout.is_empty(),
        "Output should indicate no categories"
    );
}

#[test]
fn test_category_list_with_categories() {
    let layout = test_layout_with_categories();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "list",
            "--layout",
            layout_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("navigation") || stdout.contains("Navigation"),
        "Output should contain navigation category"
    );
    assert!(
        stdout.contains("numbers") || stdout.contains("Numbers"),
        "Output should contain numbers category"
    );
}

#[test]
fn test_category_list_json_format() {
    let layout = test_layout_with_categories();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
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

    assert!(result["categories"].is_array(), "Should have categories array");
    assert!(result["count"].is_number(), "Should have count field");
    assert_eq!(result["count"].as_u64().unwrap(), 2, "Should have 2 categories");
}

#[test]
fn test_category_list_json_empty() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
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

    assert_eq!(result["count"].as_u64().unwrap(), 0, "Should have 0 categories");
    assert_eq!(
        result["categories"].as_array().unwrap().len(),
        0,
        "Categories array should be empty"
    );
}

// ============================================================================
// Add Command Tests
// ============================================================================

#[test]
fn test_category_add_valid() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "add",
            "--layout",
            layout_path.to_str().unwrap(),
            "--id",
            "test-cat",
            "--name",
            "Test Category",
            "--color",
            "#FF0000",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Adding valid category should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify it was added
    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "list",
            "--layout",
            layout_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    assert_eq!(
        result["count"].as_u64().unwrap(),
        1,
        "Should have 1 category after add"
    );
}

#[test]
fn test_category_add_duplicate_id_fails() {
    let mut layout = test_layout_basic(2, 3);
    let cat = lazyqmk::models::Category::new("test-cat", "Test", lazyqmk::models::RgbColor::new(255, 0, 0)).unwrap();
    layout.categories.push(cat);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "add",
            "--layout",
            layout_path.to_str().unwrap(),
            "--id",
            "test-cat",
            "--name",
            "Duplicate",
            "--color",
            "#00FF00",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Duplicate category ID should fail with exit code 1"
    );
}

#[test]
fn test_category_add_invalid_hex_color() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "add",
            "--layout",
            layout_path.to_str().unwrap(),
            "--id",
            "test-cat",
            "--name",
            "Test",
            "--color",
            "NOT_A_COLOR",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Invalid color should fail with exit code 1"
    );
}

#[test]
fn test_category_add_invalid_id_format() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "add",
            "--layout",
            layout_path.to_str().unwrap(),
            "--id",
            "Invalid ID!",
            "--name",
            "Test",
            "--color",
            "#FF0000",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Invalid ID format should fail with exit code 1"
    );
}

#[test]
fn test_category_add_short_hex_color() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "add",
            "--layout",
            layout_path.to_str().unwrap(),
            "--id",
            "test-cat",
            "--name",
            "Test",
            "--color",
            "#F0A",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0), "Short hex format should work. stderr: {}", String::from_utf8_lossy(&output.stderr));
}

// ============================================================================
// Delete Command Tests
// ============================================================================

#[test]
fn test_category_delete_unused() {
    let mut layout = test_layout_basic(2, 3);
    let cat = lazyqmk::models::Category::new("test-cat", "Test", lazyqmk::models::RgbColor::new(255, 0, 0)).unwrap();
    layout.categories.push(cat);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "delete",
            "--layout",
            layout_path.to_str().unwrap(),
            "--id",
            "test-cat",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Deleting unused category should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify it was deleted
    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "list",
            "--layout",
            layout_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    let response: ListCategoriesResponse =
        serde_json::from_slice(&output.stdout).expect("Invalid JSON");
    assert_eq!(response.count, 0, "Category should be deleted");
}

#[test]
fn test_category_delete_used_without_force_fails() {
    let layout = test_layout_with_categories();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "delete",
            "--layout",
            layout_path.to_str().unwrap(),
            "--id",
            "navigation",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Deleting used category without force should fail"
    );
}

#[test]
fn test_category_delete_used_with_force() {
    let layout = test_layout_with_categories();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "delete",
            "--layout",
            layout_path.to_str().unwrap(),
            "--id",
            "navigation",
            "--force",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Deleting used category with force should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify references were cleared
    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "list",
            "--layout",
            layout_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    assert_eq!(
        result["count"].as_u64().unwrap(),
        1,
        "Should have 1 category remaining"
    );
}

#[test]
fn test_category_delete_nonexistent_fails() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "delete",
            "--layout",
            layout_path.to_str().unwrap(),
            "--id",
            "nonexistent",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Deleting nonexistent category should fail"
    );
}

// ============================================================================
// File Error Tests
// ============================================================================

#[test]
fn test_category_list_nonexistent_file() {
    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "list",
            "--layout",
            "/nonexistent/layout.md",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Nonexistent file should exit with code 2"
    );
}

#[test]
fn test_category_add_nonexistent_file() {
    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "add",
            "--layout",
            "/nonexistent/layout.md",
            "--id",
            "test",
            "--name",
            "Test",
            "--color",
            "#FF0000",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Nonexistent file should exit with code 2"
    );
}

#[test]
fn test_category_delete_nonexistent_file() {
    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "delete",
            "--layout",
            "/nonexistent/layout.md",
            "--id",
            "test",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Nonexistent file should exit with code 2"
    );
}

// ============================================================================
// JSON Output Validation Tests
// ============================================================================

#[test]
fn test_category_list_json_structure() {
    let layout = test_layout_with_categories();
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "category",
            "list",
            "--layout",
            layout_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    // Validate schema
    assert!(result["categories"].is_array());
    assert!(result["count"].is_number());

    // Validate each category has required fields
    if let Some(categories) = result["categories"].as_array() {
        for cat in categories {
            assert!(cat["id"].is_string(), "Category should have id");
            assert!(cat["name"].is_string(), "Category should have name");
            assert!(cat["color"].is_string(), "Category should have color");
        }
    }
}
