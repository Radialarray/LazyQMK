use super::fixtures::{test_layout_basic, write_layout_file};
use super::helpers::*;

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
    let layout_path = temp_dir.path().join("test_layout.json");
    write_layout_file(&layout, &layout_path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/layouts").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["layouts"].as_array().unwrap().len(), 1);
    assert_eq!(json["layouts"][0]["filename"], "test_layout.json");
    assert_eq!(json["layouts"][0]["name"], "Test Layout");
}

#[tokio::test]
async fn test_get_layout_success() {
    let (state, temp_dir) = create_test_state();

    // Create a test layout file
    let layout = test_layout_basic(2, 3);
    let layout_path = temp_dir.path().join("my_layout.json");
    write_layout_file(&layout, &layout_path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/layouts/my_layout.json").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["metadata"]["name"], "Test Layout");
    assert_eq!(json["layers"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_save_layout_success() {
    let (state, temp_dir) = create_test_state();
    let app = create_router(state);

    // Create a layout to save
    let layout = test_layout_basic(2, 3);
    let layout_json: Value = serde_json::to_value(&layout).unwrap();

    let status = put_json(&app, "/api/layouts/new_layout.json", layout_json).await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify file was created
    let saved_path = temp_dir.path().join("new_layout.json");
    assert!(saved_path.exists());
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
async fn test_save_layout_path_traversal_rejected() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let layout = test_layout_basic(2, 3);
    let layout_json: Value = serde_json::to_value(&layout).unwrap();

    // URL-encoded path traversal: %2F = "/"
    let status = put_json(&app, "/api/layouts/..%2Fevil.md", layout_json).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// Regression test: PUTting a layout with an out-of-range ripple `speed`
/// (the `u8` typed field in `RgbOverlayRippleSettingsDto`) must be rejected
/// with 422 instead of corrupting the layout. Mirrors the user-facing bug
/// where typing `300` in the Speed field bypassed the HTML `max="255"` hint
/// and produced a 422 from the server.
#[tokio::test]
async fn test_save_layout_rejects_out_of_range_ripple_speed() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let mut layout = test_layout_basic(2, 3);
    layout.rgb_overlay_ripple.enabled = true;
    // Bypass the model's `u8` constraint by mutating the JSON directly:
    // the model cannot hold 300, but the wire format can and the server
    // should reject it cleanly.
    let mut layout_json: Value = serde_json::to_value(&layout).unwrap();
    layout_json["rgb_overlay_ripple"]["speed"] = json!(300);

    let status = put_json(&app, "/api/layouts/bad_speed.json", layout_json).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
