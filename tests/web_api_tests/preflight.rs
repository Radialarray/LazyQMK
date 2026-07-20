use super::fixtures::{test_layout_basic, write_layout_file};
use super::helpers::*;

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
    let path = temp_dir.path().join("test_layout.json");
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
    let path = temp_dir.path().join("my_layout.json");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/preflight").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["qmk_configured"], true);
    assert_eq!(json["has_layouts"], true);
    assert_eq!(json["first_run"], false);
}
