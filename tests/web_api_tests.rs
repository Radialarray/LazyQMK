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
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/config").await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["output_dir"].is_string());
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
