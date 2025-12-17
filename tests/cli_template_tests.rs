//! End-to-end tests for `lazyqmk template` commands.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

mod fixtures;
use fixtures::*;

// Mutex to ensure template tests don't run in parallel
static TEMPLATE_TEST_LOCK: Mutex<()> = Mutex::new(());

/// Path to the lazyqmk binary
fn lazyqmk_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lazyqmk")
}

/// Get the template directory path
fn get_template_dir() -> PathBuf {
    let config_dir = dirs::config_dir().expect("Failed to get config directory");
    config_dir.join("LazyQMK").join("templates")
}

/// Clean up all templates in the template directory to ensure test isolation
fn cleanup_templates() {
    let template_dir = get_template_dir();
    if template_dir.exists() {
        // Remove all .md files in the template directory
        if let Ok(entries) = fs::read_dir(&template_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }
}

// ============================================================================
// List Command Tests
// ============================================================================

#[test]
fn test_template_list_empty_directory() {
    let output = Command::new(lazyqmk_bin())
        .args(["template", "list"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Listing empty templates should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No templates") || stdout.is_empty() || stdout.contains('0'),
        "Output should indicate no templates"
    );
}

#[test]
fn test_template_list_json_empty() {
    let _lock = TEMPLATE_TEST_LOCK.lock().unwrap();
    cleanup_templates();

    let output = Command::new(lazyqmk_bin())
        .args(["template", "list", "--json"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    assert!(result["templates"].is_array(), "Should have templates array");
    assert_eq!(
        result["count"].as_u64().unwrap(),
        0,
        "Should have 0 templates"
    );
}

#[test]
fn test_template_list_json_structure() {
    let output = Command::new(lazyqmk_bin())
        .args(["template", "list", "--json"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    // Validate schema
    assert!(result["templates"].is_array());
    assert!(result["count"].is_number());

    // Each template should have required fields
    if let Some(templates) = result["templates"].as_array() {
        for template in templates {
            assert!(template["name"].is_string(), "Template should have name");
            assert!(template["file"].is_string(), "Template should have file");
            assert!(template["tags"].is_array(), "Template should have tags");
            assert!(template["author"].is_string(), "Template should have author");
            assert!(template["created"].is_string(), "Template should have created");
        }
    }
}

// ============================================================================
// Save Command Tests
// ============================================================================

#[test]
fn test_template_save_valid_layout() {
    let _lock = TEMPLATE_TEST_LOCK.lock().unwrap();
    cleanup_templates();

    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "template",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "my_test_template",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Saving template should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify template was created by listing
    let output = Command::new(lazyqmk_bin())
        .args(["template", "list", "--json"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    let templates = result["templates"]
        .as_array()
        .expect("Should have templates array");
    assert!(
        templates
            .iter()
            .any(|t| t["name"].as_str() == Some("my_test_template")),
        "Template should be in list"
    );
}

#[test]
fn test_template_save_with_tags() {
    let _lock = TEMPLATE_TEST_LOCK.lock().unwrap();
    cleanup_templates();

    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args([
            "template",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "tagged_template",
            "--tags",
            "corne,42-key,ergonomic",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Saving template with tags should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = Command::new(lazyqmk_bin())
        .args(["template", "list", "--json"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    let templates = result["templates"]
        .as_array()
        .expect("Should have templates array");
    let template = templates
        .iter()
        .find(|t| t["name"].as_str() == Some("tagged_template"))
        .expect("Template should exist");

    let tags = template["tags"]
        .as_array()
        .expect("Template should have tags array");
    assert_eq!(tags.len(), 3, "Template should have 3 tags");
}

#[test]
fn test_template_save_duplicate_name_fails() {
    let _lock = TEMPLATE_TEST_LOCK.lock().unwrap();
    cleanup_templates();

    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    // Save first template
    let output1 = Command::new(lazyqmk_bin())
        .args([
            "template",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "duplicate_test",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output1.status.code(), Some(0));

    // Try to save with same name
    let output2 = Command::new(lazyqmk_bin())
        .args([
            "template",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "duplicate_test",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output2.status.code(),
        Some(1),
        "Duplicate template name should fail"
    );
}

#[test]
fn test_template_save_invalid_layout_file() {
    let output = Command::new(lazyqmk_bin())
        .args([
            "template",
            "save",
            "--layout",
            "/nonexistent/layout.md",
            "--name",
            "test_template",
        ])
        .output()
        .expect("Failed to execute command");

    // Accept either exit code 1 (validation) or 2 (I/O) - depends on how command reports missing file
    assert!(
        output.status.code() == Some(1) || output.status.code() == Some(2),
        "Invalid layout file should fail with exit code 1 or 2, got {:?}",
        output.status.code()
    );
}

// ============================================================================
// Apply Command Tests
// ============================================================================

#[test]
fn test_template_apply_existing_template() {
    let _lock = TEMPLATE_TEST_LOCK.lock().unwrap();
    cleanup_templates();

    let layout = test_layout_basic(2, 3);
    let (layout_path, _layout_temp) = create_temp_layout_file(&layout);

    // First, save a template
    let output = Command::new(lazyqmk_bin())
        .args([
            "template",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "apply_test_template",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    // Now apply it
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("applied_layout.md");

    let output = Command::new(lazyqmk_bin())
        .args([
            "template",
            "apply",
            "--name",
            "apply_test_template",
            "--out",
            output_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Applying template should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output file was created
    assert!(output_path.exists(), "Output file should be created");

    // Verify it's a valid layout file
    let content = fs::read_to_string(&output_path).expect("Failed to read output file");
    assert!(!content.is_empty(), "Output file should not be empty");
}

#[test]
fn test_template_apply_nonexistent_template() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.md");

    let output = Command::new(lazyqmk_bin())
        .args([
            "template",
            "apply",
            "--name",
            "nonexistent_template",
            "--out",
            output_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Nonexistent template should fail"
    );
}

#[test]
fn test_template_apply_output_file_exists_fails() {
    let _lock = TEMPLATE_TEST_LOCK.lock().unwrap();
    cleanup_templates();

    let layout = test_layout_basic(2, 3);
    let (layout_path, _layout_temp) = create_temp_layout_file(&layout);

    // Save a template
    let output = Command::new(lazyqmk_bin())
        .args([
            "template",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
            "--name",
            "overwrite_test_template",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    // Create output file that already exists
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("existing.md");
    fs::write(&output_path, "existing content").expect("Failed to create existing file");

    // Try to apply (should fail because file exists)
    let output = Command::new(lazyqmk_bin())
        .args([
            "template",
            "apply",
            "--name",
            "overwrite_test_template",
            "--out",
            output_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // Should fail because output file already exists
    assert_eq!(
        output.status.code(),
        Some(1),
        "Applying to existing file should fail"
    );
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_template_save_and_list_multiple() {
    let _lock = TEMPLATE_TEST_LOCK.lock().unwrap();
    cleanup_templates();

    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    // Save multiple templates
    for i in 0..3 {
        let name = format!("template_{}", i);
        let output = Command::new(lazyqmk_bin())
            .args([
                "template",
                "save",
                "--layout",
                layout_path.to_str().unwrap(),
                "--name",
                &name,
            ])
            .output()
            .expect("Failed to execute command");

        assert_eq!(
            output.status.code(),
            Some(0),
            "Saving template {} should succeed",
            i
        );
    }

    // List all templates
    let output = Command::new(lazyqmk_bin())
        .args(["template", "list", "--json"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    assert_eq!(
        result["count"].as_u64().unwrap(),
        3,
        "Should have exactly 3 templates"
    );
}
