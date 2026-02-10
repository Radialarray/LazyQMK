//! Integration tests for the LazyQMK Web API.
//!
//! These tests require the `web` feature to be enabled:
//! ```bash
//! cargo test --features web web_api
//! ```

#![cfg(feature = "web")]

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::fs;
use tempfile::TempDir;
use tower::ServiceExt;

use lazyqmk::config::{BuildConfig, Config, PathConfig, UiConfig};
use lazyqmk::web::{create_router, AppState};

mod fixtures;
use fixtures::{test_layout_basic, write_layout_file};

/// Creates a test layout marked as a template.
fn test_template_basic(rows: usize, cols: usize, name: &str) -> lazyqmk::models::Layout {
    let mut layout = test_layout_basic(rows, cols);
    layout.metadata.is_template = true;
    layout.metadata.name = name.to_string();
    layout.metadata.description = format!("Template: {}", name);
    layout.metadata.tags = vec!["template".to_string(), "test".to_string()];
    layout
}

/// Creates a test AppState with a temporary workspace.
fn create_test_state() -> (AppState, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let config = Config {
        paths: PathConfig { qmk_firmware: None },
        build: BuildConfig {
            output_dir: temp_dir.path().to_path_buf(),
        },
        ui: UiConfig::default(),
    };

    let state =
        AppState::new(config, temp_dir.path().to_path_buf()).expect("Failed to create app state");

    (state, temp_dir)
}

/// Creates a test AppState with a mock QMK firmware directory.
fn create_test_state_with_qmk() -> (AppState, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let qmk_path = temp_dir.path().join("qmk_firmware");

    // Create minimal QMK structure
    let keyboard_dir = qmk_path.join("keyboards").join("test_keyboard");
    fs::create_dir_all(&keyboard_dir).expect("Failed to create keyboards dir");

    // Create Makefile (required for validation)
    fs::write(qmk_path.join("Makefile"), "# Test Makefile\n").expect("Failed to write Makefile");

    // Create minimal info.json
    let info_json = json!({
        "keyboard_name": "test_keyboard",
        "manufacturer": "Test",
        "maintainer": "test",
        "processor": "atmega32u4",
        "bootloader": "atmel-dfu",
        "usb": {
            "vid": "0xFEED",
            "pid": "0x0000",
            "device_version": "1.0.0"
        },
        "matrix_pins": {
            "cols": ["F0", "F1", "F2"],
            "rows": ["D0", "D1"]
        },
        "diode_direction": "COL2ROW",
        "layouts": {
            "LAYOUT_test": {
                "layout": [
                    {"matrix": [0, 0], "x": 0, "y": 0},
                    {"matrix": [0, 1], "x": 1, "y": 0},
                    {"matrix": [0, 2], "x": 2, "y": 0},
                    {"matrix": [1, 0], "x": 0, "y": 1},
                    {"matrix": [1, 1], "x": 1, "y": 1},
                    {"matrix": [1, 2], "x": 2, "y": 1}
                ]
            }
        }
    });

    fs::write(
        keyboard_dir.join("info.json"),
        serde_json::to_string_pretty(&info_json).unwrap(),
    )
    .expect("Failed to write info.json");

    let config = Config {
        paths: PathConfig {
            qmk_firmware: Some(qmk_path),
        },
        build: BuildConfig {
            output_dir: temp_dir.path().to_path_buf(),
        },
        ui: UiConfig::default(),
    };

    let state =
        AppState::new(config, temp_dir.path().to_path_buf()).expect("Failed to create app state");

    (state, temp_dir)
}

/// Helper to make a GET request and get the response body as JSON.
async fn get_json(app: &axum::Router, uri: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);

    (status, json)
}

/// Helper to make a PUT request with JSON body.
async fn put_json(app: &axum::Router, uri: &str, body: Value) -> StatusCode {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(uri)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    response.status()
}

/// Helper to make a POST request with JSON body and get the response.
async fn post_json(app: &axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);

    (status, json)
}

// ============================================================================
// Health Check Tests
// ============================================================================

#[tokio::test]
async fn test_health_check() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/health").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "healthy");
    assert!(json["version"].is_string());
}

// ============================================================================
// Layout Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_list_layouts_empty() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/layouts").await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["layouts"].is_array());
    assert!(json["layouts"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_list_layouts_with_files() {
    let (state, temp_dir) = create_test_state();

    // Create a test layout file
    let layout = test_layout_basic(2, 3);
    let layout_path = temp_dir.path().join("test_layout.md");
    write_layout_file(&layout, &layout_path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/layouts").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["layouts"].as_array().unwrap().len(), 1);
    assert_eq!(json["layouts"][0]["filename"], "test_layout.md");
    assert_eq!(json["layouts"][0]["name"], "Test Layout");
}

#[tokio::test]
async fn test_get_layout_success() {
    let (state, temp_dir) = create_test_state();

    // Create a test layout file
    let layout = test_layout_basic(2, 3);
    let layout_path = temp_dir.path().join("my_layout.md");
    write_layout_file(&layout, &layout_path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/layouts/my_layout.md").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["metadata"]["name"], "Test Layout");
    assert_eq!(json["layers"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_layout_not_found() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/layouts/nonexistent.md").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_get_layout_path_traversal_rejected() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    // URL-encoded path traversal: %2e%2e = ".."
    let (status, json) = get_json(&app, "/api/layouts/..%2F..%2Fetc%2Fpasswd").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("path traversal not allowed"));
}

#[tokio::test]
async fn test_save_layout_success() {
    let (state, temp_dir) = create_test_state();
    let app = create_router(state);

    // Create a layout to save
    let layout = test_layout_basic(2, 3);
    let layout_json: Value = serde_json::to_value(&layout).unwrap();

    let status = put_json(&app, "/api/layouts/new_layout.md", layout_json).await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify file was created
    let saved_path = temp_dir.path().join("new_layout.md");
    assert!(saved_path.exists());
}

#[tokio::test]
async fn test_save_layout_path_traversal_rejected() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let layout = test_layout_basic(2, 3);
    let layout_json: Value = serde_json::to_value(&layout).unwrap();

    // URL-encoded path traversal: %2F = "/"
    let status = put_json(&app, "/api/layouts/..%2Fevil.md", layout_json).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ============================================================================
// Keycode Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_list_keycodes_all() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keycodes").await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["keycodes"].is_array());
    assert!(json["total"].as_u64().unwrap() > 100);
}

#[tokio::test]
async fn test_list_keycodes_search() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keycodes?search=KC_A").await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["keycodes"].is_array());
    // Should find KC_A at minimum
    let keycodes = json["keycodes"].as_array().unwrap();
    assert!(keycodes.iter().any(|kc| kc["code"] == "KC_A"));
}

#[tokio::test]
async fn test_list_keycodes_by_category() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keycodes?category=navigation").await;

    assert_eq!(status, StatusCode::OK);
    let keycodes = json["keycodes"].as_array().unwrap();
    // All returned keycodes should be in navigation category
    assert!(keycodes.iter().all(|kc| kc["category"] == "navigation"));
}

#[tokio::test]
async fn test_list_categories() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keycodes/categories").await;

    assert_eq!(status, StatusCode::OK);
    let categories = json["categories"].as_array().unwrap();
    assert!(!categories.is_empty());

    // Should have common categories
    let category_ids: Vec<&str> = categories
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert!(category_ids.contains(&"basic"));
    assert!(category_ids.contains(&"navigation"));
    assert!(category_ids.contains(&"layers"));
}

// ============================================================================
// Config Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_get_config() {
    let (state, temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/config").await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["output_dir"].is_string());
    // workspace_root should be returned and match the temp directory we created
    assert!(json["workspace_root"].is_string());
    let workspace_root = json["workspace_root"].as_str().unwrap();
    assert_eq!(workspace_root, temp_dir.path().to_str().unwrap());
}

#[tokio::test]
async fn test_get_config_returns_workspace_root() {
    let (state, temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/config").await;

    assert_eq!(status, StatusCode::OK);

    // Verify workspace_root is in the response
    assert!(
        json.get("workspace_root").is_some(),
        "ConfigResponse should include workspace_root field"
    );

    let workspace_root = json["workspace_root"].as_str().unwrap();

    // In test context, workspace_root should be the temp directory path
    assert!(!workspace_root.is_empty());
    assert_eq!(workspace_root, temp_dir.path().to_str().unwrap());
}

#[tokio::test]
async fn test_layouts_found_in_workspace_root() {
    let (state, temp_dir) = create_test_state();

    // Create a test layout file in the workspace directory
    let layout = test_layout_basic(2, 3);
    let layout_path = temp_dir.path().join("workspace_test_layout.md");
    write_layout_file(&layout, &layout_path).expect("Failed to write layout");

    let app = create_router(state);

    // Verify the layout appears in /api/layouts
    let (status, json) = get_json(&app, "/api/layouts").await;

    assert_eq!(status, StatusCode::OK);
    let layouts = json["layouts"].as_array().unwrap();
    assert!(
        layouts
            .iter()
            .any(|l| l["filename"] == "workspace_test_layout.md"),
        "Layout should be found in workspace root"
    );
}

// ============================================================================
// Geometry Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_get_geometry_success() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keyboards/test_keyboard/geometry/LAYOUT_test").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["keyboard"], "test_keyboard");
    assert_eq!(json["layout"], "LAYOUT_test");
    assert_eq!(json["keys"].as_array().unwrap().len(), 6);
    assert_eq!(json["matrix_rows"], 2);
    assert_eq!(json["matrix_cols"], 3);
}

#[tokio::test]
async fn test_get_geometry_no_qmk() {
    let (state, _temp_dir) = create_test_state(); // No QMK path configured
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keyboards/test_keyboard/geometry/LAYOUT_test").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("not configured"));
}

#[tokio::test]
async fn test_get_geometry_keyboard_not_found() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keyboards/nonexistent/geometry/LAYOUT_test").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("Failed to parse"));
}

#[tokio::test]
async fn test_get_geometry_layout_not_found() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let (status, json) =
        get_json(&app, "/api/keyboards/test_keyboard/geometry/INVALID_LAYOUT").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_get_geometry_path_traversal_rejected() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    // URL-encoded path traversal: %2F = "/", %2e = "."
    let (status, json) = get_json(
        &app,
        "/api/keyboards/..%2F..%2Fetc%2Fpasswd/geometry/LAYOUT",
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("path traversal not allowed"));
}

// ============================================================================
// Template Endpoint Tests
// ============================================================================

/// Creates a test state with a custom template directory for isolated tests.
/// Returns the state, temp workspace dir, and a path to use for templates.
fn create_test_state_with_template_dir() -> (AppState, TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Use the platform config directory for templates (what the endpoints use)
    let template_dir = lazyqmk::config::Config::config_dir()
        .expect("Failed to get config dir")
        .join("templates");

    // Ensure template directory exists
    fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let config = Config {
        paths: PathConfig { qmk_firmware: None },
        build: BuildConfig {
            output_dir: temp_dir.path().to_path_buf(),
        },
        ui: UiConfig::default(),
    };

    let state =
        AppState::new(config, temp_dir.path().to_path_buf()).expect("Failed to create app state");

    (state, temp_dir, template_dir)
}

/// Helper to clean up a specific template file after test.
fn cleanup_template(template_dir: &std::path::Path, filename: &str) {
    let path = template_dir.join(filename);
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}

#[tokio::test]
async fn test_list_templates_empty_or_existing() {
    // Note: We can't guarantee empty since the config dir persists between tests.
    // Instead, verify the endpoint returns a valid response with templates array.
    let (state, _temp_dir, _template_dir) = create_test_state_with_template_dir();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/templates").await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["templates"].is_array());
}

#[tokio::test]
async fn test_list_templates_finds_saved_template() {
    let (state, temp_dir, template_dir) = create_test_state_with_template_dir();

    // Create a template in the template directory with unique name
    let unique_name = format!("test_list_template_{}", std::process::id());
    let template = test_template_basic(2, 3, &unique_name);
    let template_filename = format!("{}.md", unique_name.to_lowercase().replace(' ', "_"));
    let template_path = template_dir.join(&template_filename);
    write_layout_file(&template, &template_path).expect("Failed to write template");

    // Also create a non-template layout in workspace (should not appear in templates)
    let layout = test_layout_basic(2, 3);
    let layout_path = temp_dir.path().join("regular_layout.md");
    write_layout_file(&layout, &layout_path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/templates").await;

    // Cleanup before assertions (in case of failure)
    cleanup_template(&template_dir, &template_filename);

    assert_eq!(status, StatusCode::OK);
    let templates = json["templates"].as_array().unwrap();

    // Should find our template
    let found = templates
        .iter()
        .any(|t| t["name"].as_str() == Some(&unique_name));
    assert!(found, "Template '{}' not found in list", unique_name);

    // Verify template has expected fields
    let our_template = templates
        .iter()
        .find(|t| t["name"].as_str() == Some(&unique_name))
        .unwrap();
    assert!(our_template["filename"].is_string());
    assert!(our_template["description"].is_string());
    assert!(our_template["layer_count"].is_number());
    assert!(our_template["tags"].is_array());
}

#[tokio::test]
async fn test_get_template_success() {
    let (state, _temp_dir, template_dir) = create_test_state_with_template_dir();

    // Create a template
    let unique_name = format!("test_get_template_{}", std::process::id());
    let template = test_template_basic(2, 3, &unique_name);
    let template_filename = format!("{}.md", unique_name.to_lowercase().replace(' ', "_"));
    let template_path = template_dir.join(&template_filename);
    write_layout_file(&template, &template_path).expect("Failed to write template");

    let app = create_router(state);

    let (status, json) = get_json(&app, &format!("/api/templates/{}", template_filename)).await;

    // Cleanup
    cleanup_template(&template_dir, &template_filename);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["metadata"]["name"], unique_name);
    assert_eq!(json["metadata"]["is_template"], true);
    assert_eq!(json["layers"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_template_not_found() {
    let (state, _temp_dir, _template_dir) = create_test_state_with_template_dir();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/templates/nonexistent_template.md").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_get_template_path_traversal_rejected() {
    let (state, _temp_dir, _template_dir) = create_test_state_with_template_dir();
    let app = create_router(state);

    // URL-encoded path traversal
    let (status, json) = get_json(&app, "/api/templates/..%2F..%2Fetc%2Fpasswd").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("path traversal not allowed"));
}

#[tokio::test]
async fn test_get_template_rejects_non_template() {
    let (state, _temp_dir, template_dir) = create_test_state_with_template_dir();

    // Create a non-template layout in template directory
    let unique_name = format!("test_non_template_{}", std::process::id());
    let mut layout = test_layout_basic(2, 3);
    layout.metadata.is_template = false; // Explicitly not a template
    let filename = format!("{}.md", unique_name.to_lowercase().replace(' ', "_"));
    let path = template_dir.join(&filename);
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = get_json(&app, &format!("/api/templates/{}", filename)).await;

    // Cleanup
    cleanup_template(&template_dir, &filename);

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("not a template"));
}

#[tokio::test]
async fn test_save_as_template_success() {
    let (state, temp_dir, template_dir) = create_test_state_with_template_dir();

    // Create a source layout in workspace
    let layout = test_layout_basic(2, 3);
    let source_filename = "source_for_template.md";
    let source_path = temp_dir.path().join(source_filename);
    write_layout_file(&layout, &source_path).expect("Failed to write source layout");

    let app = create_router(state);

    let unique_name = format!("My Test Template {}", std::process::id());
    let request = json!({
        "name": unique_name,
        "tags": ["custom", "test"]
    });

    let (status, json) = post_json(
        &app,
        &format!("/api/layouts/{}/save-as-template", source_filename),
        request,
    )
    .await;

    // Generate expected filename (same logic as sanitize_template_filename)
    let expected_filename: String = unique_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    let expected_filename = format!("{}.md", expected_filename);

    // Cleanup
    cleanup_template(&template_dir, &expected_filename);

    assert_eq!(status, StatusCode::OK, "Response: {:?}", json);
    assert_eq!(json["name"], unique_name);
    assert!(std::path::Path::new(json["filename"].as_str().unwrap())
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md")));
    assert_eq!(json["layer_count"], 2);
    assert!(json["tags"].as_array().unwrap().contains(&json!("custom")));
}

#[tokio::test]
async fn test_save_as_template_source_not_found() {
    let (state, _temp_dir, _template_dir) = create_test_state_with_template_dir();
    let app = create_router(state);

    let request = json!({
        "name": "Template From Missing",
        "tags": []
    });

    let (status, json) = post_json(
        &app,
        "/api/layouts/nonexistent_layout.md/save-as-template",
        request,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

// ============================================================================
// Keyboard & Setup Wizard Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_list_keyboards_no_qmk_path() {
    // Create state without QMK path configured
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keyboards").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("QMK firmware path not configured"));
}

#[tokio::test]
async fn test_list_keyboards_with_qmk() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keyboards").await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["keyboards"].is_array());
    let keyboards = json["keyboards"].as_array().unwrap();
    // Should find test_keyboard from our mock QMK setup
    assert!(!keyboards.is_empty());
    assert!(keyboards.iter().any(|k| k["path"] == "test_keyboard"));
}

#[tokio::test]
async fn test_list_keyboard_layouts() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keyboards/test_keyboard/layouts").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["keyboard"], "test_keyboard");
    assert!(json["variants"].is_array());
    let variants = json["variants"].as_array().unwrap();
    // Should find LAYOUT_test from our mock info.json
    assert!(!variants.is_empty());
    assert!(variants.iter().any(|v| v["name"] == "LAYOUT_test"));
    // Check key count
    let layout_test = variants
        .iter()
        .find(|v| v["name"] == "LAYOUT_test")
        .unwrap();
    assert_eq!(layout_test["key_count"], 6);
}

#[tokio::test]
async fn test_list_keyboard_layouts_not_found() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keyboards/nonexistent_keyboard/layouts").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().is_some());
}

#[tokio::test]
async fn test_list_keyboard_layouts_path_traversal() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    // Path traversal should be rejected with 400 before keyboard lookup
    let (status, json) = get_json(&app, "/api/keyboards/..%2F..%2Fetc/layouts").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("Invalid keyboard path"));
}

#[tokio::test]
async fn test_create_layout() {
    let (state, temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let request = json!({
        "filename": "new_layout",
        "name": "My New Layout",
        "keyboard": "test_keyboard",
        "layout_variant": "LAYOUT_test",
        "description": "A test layout",
        "author": "Test Author"
    });

    let (status, json) = post_json(&app, "/api/layouts", request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["metadata"]["name"], "My New Layout");
    assert_eq!(json["metadata"]["keyboard"], "test_keyboard");
    assert_eq!(json["metadata"]["layout_variant"], "LAYOUT_test");
    assert_eq!(json["metadata"]["description"], "A test layout");
    assert_eq!(json["metadata"]["author"], "Test Author");

    // Verify file was created
    let layout_path = temp_dir.path().join("new_layout.md");
    assert!(layout_path.exists());
}

#[tokio::test]
async fn test_create_layout_already_exists() {
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create an existing file
    let existing_path = temp_dir.path().join("existing_layout.md");
    std::fs::write(&existing_path, "# Existing").expect("Failed to write file");

    let app = create_router(state);

    let request = json!({
        "filename": "existing_layout",
        "name": "New Layout",
        "keyboard": "test_keyboard",
        "layout_variant": "LAYOUT_test"
    });

    let (status, json) = post_json(&app, "/api/layouts", request).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(json["error"].as_str().unwrap().contains("already exists"));
}

#[tokio::test]
async fn test_create_layout_invalid_keyboard() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let request = json!({
        "filename": "new_layout",
        "name": "My New Layout",
        "keyboard": "nonexistent_keyboard",
        "layout_variant": "LAYOUT_test"
    });

    let (status, _json) = post_json(&app, "/api/layouts", request).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_layout_invalid_variant() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let request = json!({
        "filename": "new_layout",
        "name": "My New Layout",
        "keyboard": "test_keyboard",
        "layout_variant": "NONEXISTENT_LAYOUT"
    });

    let (status, json) = post_json(&app, "/api/layouts", request).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("Layout variant"));
}

#[tokio::test]
async fn test_switch_layout_variant() {
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create a layout file with a keyboard set
    let mut layout = test_layout_basic(2, 3);
    layout.metadata.keyboard = Some("test_keyboard".to_string());
    layout.metadata.layout_variant = Some("LAYOUT_test".to_string());
    let layout_path = temp_dir.path().join("switch_test.md");
    write_layout_file(&layout, &layout_path).expect("Failed to write layout");

    let app = create_router(state);

    let request = json!({
        "layout_variant": "LAYOUT_test"
    });

    let (status, json) = post_json(&app, "/api/layouts/switch_test/switch-variant", request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["layout"].is_object());
    assert!(json["keys_added"].is_number());
    assert!(json["keys_removed"].is_number());
}

#[tokio::test]
async fn test_switch_layout_variant_not_found() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let request = json!({
        "layout_variant": "LAYOUT_test"
    });

    let (status, json) = post_json(&app, "/api/layouts/nonexistent/switch-variant", request).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_switch_layout_variant_no_keyboard() {
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create a layout file without keyboard set
    let mut layout = test_layout_basic(2, 3);
    layout.metadata.keyboard = None; // Clear the keyboard
    layout.metadata.layout_variant = None; // Clear the variant
    let layout_path = temp_dir.path().join("no_keyboard.md");
    write_layout_file(&layout, &layout_path).expect("Failed to write layout");

    let app = create_router(state);

    let request = json!({
        "layout_variant": "LAYOUT_test"
    });

    let (status, json) = post_json(&app, "/api/layouts/no_keyboard/switch-variant", request).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("no keyboard defined"));
}

#[tokio::test]
async fn test_save_as_template_conflict() {
    let (state, temp_dir, template_dir) = create_test_state_with_template_dir();

    // Create a source layout
    let layout = test_layout_basic(2, 3);
    let source_filename = "source_for_conflict.md";
    let source_path = temp_dir.path().join(source_filename);
    write_layout_file(&layout, &source_path).expect("Failed to write source layout");

    // Pre-create a template with the same name
    let unique_name = format!("conflict_template_{}", std::process::id());
    let existing_template = test_template_basic(2, 3, &unique_name);
    let expected_filename: String = unique_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    let expected_filename = format!("{}.md", expected_filename);
    let template_path = template_dir.join(&expected_filename);
    write_layout_file(&existing_template, &template_path)
        .expect("Failed to write existing template");

    let app = create_router(state);

    let request = json!({
        "name": unique_name,
        "tags": []
    });

    let (status, json) = post_json(
        &app,
        &format!("/api/layouts/{}/save-as-template", source_filename),
        request,
    )
    .await;

    // Cleanup
    cleanup_template(&template_dir, &expected_filename);

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(json["error"].as_str().unwrap().contains("already exists"));
}

#[tokio::test]
async fn test_save_as_template_path_traversal_rejected() {
    let (state, _temp_dir, _template_dir) = create_test_state_with_template_dir();
    let app = create_router(state);

    let request = json!({
        "name": "Evil Template",
        "tags": []
    });

    // URL-encoded path traversal
    let (status, json) = post_json(
        &app,
        "/api/layouts/..%2F..%2Fetc%2Fpasswd/save-as-template",
        request,
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("path traversal not allowed"));
}

#[tokio::test]
async fn test_apply_template_success() {
    let (state, temp_dir, template_dir) = create_test_state_with_template_dir();

    // Create a template
    let unique_name = format!("apply_template_{}", std::process::id());
    let template = test_template_basic(2, 3, &unique_name);
    let template_filename = format!("{}.md", unique_name.to_lowercase().replace(' ', "_"));
    let template_path = template_dir.join(&template_filename);
    write_layout_file(&template, &template_path).expect("Failed to write template");

    let app = create_router(state);

    let target_filename = format!("new_from_template_{}.md", std::process::id());
    let request = json!({
        "target_filename": target_filename
    });

    let (status, json) = post_json(
        &app,
        &format!("/api/templates/{}/apply", template_filename),
        request,
    )
    .await;

    // Cleanup
    cleanup_template(&template_dir, &template_filename);
    let target_path = temp_dir.path().join(&target_filename);
    if target_path.exists() {
        let _ = fs::remove_file(&target_path);
    }

    assert_eq!(status, StatusCode::OK, "Response: {:?}", json);
    // Applied layout should NOT be a template
    assert_eq!(json["metadata"]["is_template"], false);
    assert_eq!(json["layers"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_apply_template_not_found() {
    let (state, _temp_dir, _template_dir) = create_test_state_with_template_dir();
    let app = create_router(state);

    let request = json!({
        "target_filename": "new_layout.md"
    });

    let (status, json) = post_json(
        &app,
        "/api/templates/nonexistent_template.md/apply",
        request,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_apply_template_target_exists_conflict() {
    let (state, temp_dir, template_dir) = create_test_state_with_template_dir();

    // Create a template
    let unique_name = format!("apply_conflict_template_{}", std::process::id());
    let template = test_template_basic(2, 3, &unique_name);
    let template_filename = format!("{}.md", unique_name.to_lowercase().replace(' ', "_"));
    let template_path = template_dir.join(&template_filename);
    write_layout_file(&template, &template_path).expect("Failed to write template");

    // Pre-create a layout at the target path
    let target_filename = "existing_layout.md";
    let existing_layout = test_layout_basic(2, 3);
    let target_path = temp_dir.path().join(target_filename);
    write_layout_file(&existing_layout, &target_path).expect("Failed to write existing layout");

    let app = create_router(state);

    let request = json!({
        "target_filename": target_filename
    });

    let (status, json) = post_json(
        &app,
        &format!("/api/templates/{}/apply", template_filename),
        request,
    )
    .await;

    // Cleanup
    cleanup_template(&template_dir, &template_filename);

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(json["error"].as_str().unwrap().contains("already exists"));
}

#[tokio::test]
async fn test_apply_template_path_traversal_rejected() {
    let (state, _temp_dir, _template_dir) = create_test_state_with_template_dir();
    let app = create_router(state);

    let request = json!({
        "target_filename": "new_layout.md"
    });

    // URL-encoded path traversal in template filename
    let (status, json) =
        post_json(&app, "/api/templates/..%2F..%2Fetc%2Fpasswd/apply", request).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("path traversal not allowed"));
}

#[tokio::test]
async fn test_apply_template_target_path_traversal_rejected() {
    let (state, _temp_dir, template_dir) = create_test_state_with_template_dir();

    // Create a valid template
    let unique_name = format!("target_traversal_template_{}", std::process::id());
    let template = test_template_basic(2, 3, &unique_name);
    let template_filename = format!("{}.md", unique_name.to_lowercase().replace(' ', "_"));
    let template_path = template_dir.join(&template_filename);
    write_layout_file(&template, &template_path).expect("Failed to write template");

    let app = create_router(state);

    // Path traversal in target filename
    let request = json!({
        "target_filename": "../evil_layout.md"
    });

    let (status, json) = post_json(
        &app,
        &format!("/api/templates/{}/apply", template_filename),
        request,
    )
    .await;

    // Cleanup
    cleanup_template(&template_dir, &template_filename);

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("path traversal not allowed"));
}

#[tokio::test]
async fn test_save_as_template_validates_name_empty() {
    let (state, temp_dir, _template_dir) = create_test_state_with_template_dir();

    // Create a source layout
    let layout = test_layout_basic(2, 3);
    let source_filename = "source_for_empty_name.md";
    let source_path = temp_dir.path().join(source_filename);
    write_layout_file(&layout, &source_path).expect("Failed to write source layout");

    let app = create_router(state);

    let request = json!({
        "name": "",
        "tags": []
    });

    let (status, json) = post_json(
        &app,
        &format!("/api/layouts/{}/save-as-template", source_filename),
        request,
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("cannot be empty"));
}

#[tokio::test]
async fn test_save_as_template_validates_name_too_long() {
    let (state, temp_dir, _template_dir) = create_test_state_with_template_dir();

    // Create a source layout
    let layout = test_layout_basic(2, 3);
    let source_filename = "source_for_long_name.md";
    let source_path = temp_dir.path().join(source_filename);
    write_layout_file(&layout, &source_path).expect("Failed to write source layout");

    let app = create_router(state);

    // Name with 101 bytes (exceeds 100-byte limit)
    let long_name = "a".repeat(101);
    let request = json!({
        "name": long_name,
        "tags": []
    });

    let (status, json) = post_json(
        &app,
        &format!("/api/layouts/{}/save-as-template", source_filename),
        request,
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("exceeds maximum length"));
    assert!(json["error"].as_str().unwrap().contains("101"));
}

#[tokio::test]
async fn test_save_as_template_validates_tag_non_ascii() {
    let (state, temp_dir, _template_dir) = create_test_state_with_template_dir();

    // Create a source layout
    let layout = test_layout_basic(2, 3);
    let source_filename = "source_for_non_ascii_tag.md";
    let source_path = temp_dir.path().join(source_filename);
    write_layout_file(&layout, &source_path).expect("Failed to write source layout");

    let app = create_router(state);

    let request = json!({
        "name": "Valid Template Name",
        "tags": ["valid", "café", "test"]
    });

    let (status, json) = post_json(
        &app,
        &format!("/api/layouts/{}/save-as-template", source_filename),
        request,
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("café"));
    assert!(json["error"].as_str().unwrap().contains("lowercase ASCII"));
}

#[tokio::test]
async fn test_save_as_template_validates_tag_uppercase() {
    let (state, temp_dir, _template_dir) = create_test_state_with_template_dir();

    // Create a source layout
    let layout = test_layout_basic(2, 3);
    let source_filename = "source_for_uppercase_tag.md";
    let source_path = temp_dir.path().join(source_filename);
    write_layout_file(&layout, &source_path).expect("Failed to write source layout");

    let app = create_router(state);

    let request = json!({
        "name": "Valid Template Name",
        "tags": ["valid", "UPPERCASE", "test"]
    });

    let (status, json) = post_json(
        &app,
        &format!("/api/layouts/{}/save-as-template", source_filename),
        request,
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("UPPERCASE"));
}

#[tokio::test]
async fn test_save_as_template_validates_tag_empty() {
    let (state, temp_dir, _template_dir) = create_test_state_with_template_dir();

    // Create a source layout
    let layout = test_layout_basic(2, 3);
    let source_filename = "source_for_empty_tag.md";
    let source_path = temp_dir.path().join(source_filename);
    write_layout_file(&layout, &source_path).expect("Failed to write source layout");

    let app = create_router(state);

    let request = json!({
        "name": "Valid Template Name",
        "tags": ["valid", "", "test"]
    });

    let (status, json) = post_json(
        &app,
        &format!("/api/layouts/{}/save-as-template", source_filename),
        request,
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("cannot be empty"));
}

// ============================================================================
// Build Job Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_list_build_jobs_empty() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/build/jobs").await;

    assert_eq!(status, StatusCode::OK);
    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_start_build_missing_layout() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let request = json!({
        "layout_filename": "nonexistent.md"
    });

    let (status, json) = post_json(&app, "/api/build/start", request).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_start_build_no_keyboard_in_layout() {
    let (state, temp_dir) = create_test_state();

    // Create a layout without keyboard metadata
    let mut layout = test_layout_basic(2, 3);
    layout.metadata.keyboard = None; // Remove keyboard field
    let filename = "no_keyboard_layout.md";
    let path = temp_dir.path().join(filename);
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    let request = json!({
        "layout_filename": filename
    });

    let (status, json) = post_json(&app, "/api/build/start", request).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("no keyboard defined"));
}

#[tokio::test]
async fn test_get_build_job_not_found() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/build/jobs/nonexistent-job-id").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_get_build_logs_not_found() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/build/jobs/nonexistent-job-id/logs").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_cancel_build_job_not_found() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) =
        post_json(&app, "/api/build/jobs/nonexistent-job-id/cancel", json!({})).await;

    assert_eq!(status, StatusCode::OK);
    assert!(!json["success"].as_bool().unwrap());
    assert!(json["message"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_start_build_no_qmk_path() {
    let (state, temp_dir) = create_test_state();

    // Create a layout with keyboard metadata
    let mut layout = test_layout_basic(2, 3);
    layout.metadata.keyboard = Some("test_keyboard".to_string());
    layout.metadata.keymap_name = Some("default".to_string());

    let filename = "with_keyboard_layout.md";
    let path = temp_dir.path().join(filename);
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    let request = json!({
        "layout_filename": filename
    });

    let (status, json) = post_json(&app, "/api/build/start", request).await;

    // Should fail because QMK path is not configured
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("QMK firmware path not configured"));
}

// ============================================================================
// Preflight Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_preflight_first_run() {
    // Empty workspace, no QMK config = first run
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/preflight").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["qmk_configured"], false);
    assert_eq!(json["has_layouts"], false);
    assert_eq!(json["first_run"], true);
    assert!(json["qmk_firmware_path"].is_null());
}

#[tokio::test]
async fn test_preflight_with_qmk_configured() {
    // QMK configured but no layouts
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/preflight").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["qmk_configured"], true);
    assert_eq!(json["has_layouts"], false);
    assert_eq!(json["first_run"], false); // QMK configured means not first run
    assert!(json["qmk_firmware_path"].is_string());
}

#[tokio::test]
async fn test_preflight_with_layouts() {
    // Has layouts but no QMK config
    let (state, temp_dir) = create_test_state();

    // Create a layout file
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("test_layout.md");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/preflight").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["qmk_configured"], false);
    assert_eq!(json["has_layouts"], true);
    assert_eq!(json["first_run"], false); // Has layouts means not first run
}

#[tokio::test]
async fn test_preflight_returning_user() {
    // Has both QMK config and layouts = returning user
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create a layout file
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("my_layout.md");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/preflight").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["qmk_configured"], true);
    assert_eq!(json["has_layouts"], true);
    assert_eq!(json["first_run"], false);
}

// ============================================================================
// Generate Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_generate_firmware_missing_layout() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let (status, json) = post_json(&app, "/api/layouts/nonexistent.md/generate", json!({})).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_generate_firmware_path_traversal_rejected() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    // Test path traversal in filename (URL encoding prevents router normalization)
    let (status, json) = post_json(&app, "/api/layouts/..%2Fsecret.md/generate", json!({})).await;

    // Should be rejected as bad request due to path traversal detection
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("path traversal"));
}

#[tokio::test]
async fn test_generate_firmware_no_keyboard() {
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create a layout file without keyboard defined
    let mut layout = test_layout_basic(2, 3);
    layout.metadata.keyboard = None;
    let path = temp_dir.path().join("no_keyboard.md");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = post_json(&app, "/api/layouts/no_keyboard.md/generate", json!({})).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("no keyboard"));
}

#[tokio::test]
async fn test_generate_firmware_no_layout_variant() {
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create a layout file without layout_variant defined
    let mut layout = test_layout_basic(2, 3);
    layout.metadata.layout_variant = None;
    let path = temp_dir.path().join("no_variant.md");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = post_json(&app, "/api/layouts/no_variant.md/generate", json!({})).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("no layout variant"));
}

#[tokio::test]
async fn test_generate_firmware_starts_job() {
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create a layout file
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("test_layout.md");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = post_json(&app, "/api/layouts/test_layout.md/generate", json!({})).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "started");
    assert!(json["job"]["id"].is_string());
    assert!(json["job"]["download_url"].is_string());
    assert_eq!(json["job"]["status"], "pending");
    assert_eq!(json["job"]["layout_filename"], "test_layout.md");
}

#[tokio::test]
async fn test_generate_job_status() {
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create a layout file
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("test_layout.md");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    // Start a job
    let (status, json) = post_json(&app, "/api/layouts/test_layout.md/generate", json!({})).await;
    assert_eq!(status, StatusCode::OK);

    let job_id = json["job"]["id"].as_str().unwrap();

    // Get job status
    let (status, json) = get_json(&app, &format!("/api/generate/jobs/{job_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["job"]["id"].is_string());
}

#[tokio::test]
async fn test_generate_job_not_found() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/generate/jobs/nonexistent-job-id").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_generate_job_logs() {
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create a layout file
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("test_layout.md");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    // Start a job
    let (status, json) = post_json(&app, "/api/layouts/test_layout.md/generate", json!({})).await;
    assert_eq!(status, StatusCode::OK);

    let job_id = json["job"]["id"].as_str().unwrap();

    // Get job logs
    let (status, json) = get_json(&app, &format!("/api/generate/jobs/{job_id}/logs")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["job_id"], job_id);
    assert!(json["logs"].is_array());
}

#[tokio::test]
async fn test_generate_job_cancel() {
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create a layout file
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("test_layout.md");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    // Start a job
    let (status, json) = post_json(&app, "/api/layouts/test_layout.md/generate", json!({})).await;
    assert_eq!(status, StatusCode::OK);

    let job_id = json["job"]["id"].as_str().unwrap();

    // Cancel the job
    let (status, json) = post_json(
        &app,
        &format!("/api/generate/jobs/{job_id}/cancel"),
        json!({}),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    // Job may have already completed by now, so accept either success or failure
    assert!(json["success"].is_boolean());
}

#[tokio::test]
async fn test_generate_jobs_list() {
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create a layout file
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("test_layout.md");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    // Start a job
    let _ = post_json(&app, "/api/layouts/test_layout.md/generate", json!({})).await;

    // List jobs
    let (status, json) = get_json(&app, "/api/generate/jobs").await;

    assert_eq!(status, StatusCode::OK);
    assert!(json.is_array());
    assert!(!json.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_generate_download_job_not_found() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/generate/jobs/nonexistent-job-id/download").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

/// Test that GET /api/layouts/{filename} returns enriched key data with visual_index,
/// matrix_position, and led_index fields needed by the frontend.
#[tokio::test]
async fn test_get_layout_returns_enriched_key_data() {
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create a layout file
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("test_layout.md");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    // GET the layout
    let (status, json) = get_json(&app, "/api/layouts/test_layout.md").await;

    assert_eq!(status, StatusCode::OK);

    // Verify layout structure
    assert!(json["metadata"].is_object());
    assert!(json["layers"].is_array());

    // Get first layer
    let layers = json["layers"].as_array().unwrap();
    assert!(!layers.is_empty());

    let first_layer = &layers[0];
    let keys = first_layer["keys"].as_array().unwrap();
    assert!(!keys.is_empty());

    // Verify each key has the required enriched fields
    for key in keys {
        // Required fields for frontend compatibility
        assert!(key["keycode"].is_string(), "Missing keycode field");
        assert!(
            key["visual_index"].is_number(),
            "Missing visual_index field"
        );
        assert!(
            key["matrix_position"].is_array(),
            "Missing matrix_position field"
        );
        assert!(key["led_index"].is_number(), "Missing led_index field");
        assert!(key["position"].is_object(), "Missing position field");

        // Verify matrix_position is [row, col] array
        let matrix_pos = key["matrix_position"].as_array().unwrap();
        assert_eq!(matrix_pos.len(), 2, "matrix_position should be [row, col]");

        // Verify position has row and col fields
        assert!(
            key["position"]["row"].is_number(),
            "position.row should be a number"
        );
        assert!(
            key["position"]["col"].is_number(),
            "position.col should be a number"
        );
    }
}

#[tokio::test]
async fn test_swap_keys_swaps_keycodes_and_colors() {
    let (state, temp_dir) = create_test_state();
    let workspace = temp_dir.path();

    // Create test layout with two distinct keys
    let layout_content = r#"---
name: Swap Test
description: Test layout for swap
author: Test
created: 2024-01-01T00:00:00Z
modified: 2024-01-01T00:00:00Z
tags: []
is_template: false
version: '1.0'
keyboard: test
layout_variant: LAYOUT_test
keymap_name: test
output_format: hex
---

# Swap Test

## Layer 0: Base
**ID**: test-layer
**Color**: #FFFFFF

| C0 | C1 |
|------|------|
| KC_Q{#FF0000} | KC_W{#00FF00} |
"#;

    fs::write(workspace.join("test_swap.md"), layout_content).unwrap();

    let app = create_router(state);

    // Get initial layout to verify starting state
    let (status, initial_json) = get_json(&app, "/api/layouts/test_swap.md").await;
    assert_eq!(status, StatusCode::OK);

    let initial_keys = initial_json["layers"][0]["keys"].as_array().unwrap();
    let key0_initial = &initial_keys[0];
    let key1_initial = &initial_keys[1];

    assert_eq!(key0_initial["keycode"].as_str().unwrap(), "KC_Q");
    assert_eq!(key1_initial["keycode"].as_str().unwrap(), "KC_W");
    assert_eq!(key0_initial["color_override"]["r"].as_u64().unwrap(), 255);
    assert_eq!(key0_initial["color_override"]["g"].as_u64().unwrap(), 0);
    assert_eq!(key0_initial["color_override"]["b"].as_u64().unwrap(), 0);
    assert_eq!(key1_initial["color_override"]["r"].as_u64().unwrap(), 0);
    assert_eq!(key1_initial["color_override"]["g"].as_u64().unwrap(), 255);
    assert_eq!(key1_initial["color_override"]["b"].as_u64().unwrap(), 0);

    // Swap keys at positions (0,0) and (0,1)
    let swap_request = json!({
        "layer": 0,
        "first_position": { "row": 0, "col": 0 },
        "second_position": { "row": 0, "col": 1 }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/layouts/test_swap.md/swap-keys")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&swap_request).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Get layout again to verify swap
    let (status, swapped_json) = get_json(&app, "/api/layouts/test_swap.md").await;
    assert_eq!(status, StatusCode::OK);

    let swapped_keys = swapped_json["layers"][0]["keys"].as_array().unwrap();
    let key0_swapped = &swapped_keys[0];
    let key1_swapped = &swapped_keys[1];

    // Verify keycodes swapped
    assert_eq!(
        key0_swapped["keycode"].as_str().unwrap(),
        "KC_W",
        "Key at position 0 should now have KC_W keycode"
    );
    assert_eq!(
        key1_swapped["keycode"].as_str().unwrap(),
        "KC_Q",
        "Key at position 1 should now have KC_Q keycode"
    );

    // Verify colors swapped (green to key0, red to key1)
    assert_eq!(
        key0_swapped["color_override"]["r"].as_u64().unwrap(),
        0,
        "Key at position 0 should now have green color (r=0)"
    );
    assert_eq!(
        key0_swapped["color_override"]["g"].as_u64().unwrap(),
        255,
        "Key at position 0 should now have green color (g=255)"
    );
    assert_eq!(
        key0_swapped["color_override"]["b"].as_u64().unwrap(),
        0,
        "Key at position 0 should now have green color (b=0)"
    );
    assert_eq!(
        key1_swapped["color_override"]["r"].as_u64().unwrap(),
        255,
        "Key at position 1 should now have red color (r=255)"
    );
    assert_eq!(
        key1_swapped["color_override"]["g"].as_u64().unwrap(),
        0,
        "Key at position 1 should now have red color (g=0)"
    );
    assert_eq!(
        key1_swapped["color_override"]["b"].as_u64().unwrap(),
        0,
        "Key at position 1 should now have red color (b=0)"
    );

    // Verify the file on disk was actually updated
    let file_content = fs::read_to_string(workspace.join("test_swap.md")).unwrap();
    assert!(
        file_content.contains("KC_W{#00FF00} | KC_Q{#FF0000}"),
        "Layout file should have swapped keys"
    );
}
