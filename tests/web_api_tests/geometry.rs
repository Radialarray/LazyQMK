use super::helpers::*;

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
