use super::fixtures::{test_layout_basic, write_layout_file};
use super::helpers::*;

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
        "layout_filename": "nonexistent.json"
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
    let filename = "no_keyboard_layout.json";
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

    let filename = "with_keyboard_layout.json";
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
