//! Shared helper functions and re-exports for web API tests.
//!
//! Provides utility functions used across all topic modules.

// Re-exported types and modules for use by sibling topic modules.
// Each topic module does `use super::helpers::*;` to bring these into scope.
pub(super) use axum::body::Body;
pub(super) use axum::http::Request;
pub(super) use axum::http::StatusCode;
pub(super) use http_body_util::BodyExt;
pub(super) use serde_json::{json, Value};
pub(super) use std::fs;
pub(super) use tempfile::TempDir;
pub(super) use tower::ServiceExt;

pub(super) use lazyqmk::config::{BuildConfig, Config, PathConfig, UiConfig};
pub(super) use lazyqmk::web::{create_router, AppState};

// Re-export test_layout_basic from fixtures (needed by test_template_basic below)
pub(super) use super::fixtures::{test_layout_basic, write_layout_file};

/// Creates a test layout marked as a template.
pub(super) fn test_template_basic(rows: usize, cols: usize, name: &str) -> lazyqmk::models::Layout {
    let mut layout = test_layout_basic(rows, cols);
    layout.metadata.is_template = true;
    layout.metadata.name = name.to_string();
    layout.metadata.description = format!("Template: {}", name);
    layout.metadata.tags = vec!["template".to_string(), "test".to_string()];
    layout
}

/// Creates a test AppState with a temporary workspace.
pub(super) fn create_test_state() -> (AppState, TempDir) {
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
pub(super) fn create_test_state_with_qmk() -> (AppState, TempDir) {
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
pub(super) async fn get_json(app: &axum::Router, uri: &str) -> (StatusCode, Value) {
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
pub(super) async fn put_json(app: &axum::Router, uri: &str, body: Value) -> StatusCode {
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
pub(super) async fn post_json(app: &axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
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

/// Creates a test state with a custom template directory for isolated tests.
/// Returns the state, temp workspace dir, and a path to use for templates.
pub(super) fn create_test_state_with_template_dir() -> (AppState, TempDir, std::path::PathBuf) {
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
pub(super) fn cleanup_template(template_dir: &std::path::Path, filename: &str) {
    let path = template_dir.join(filename);
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}
