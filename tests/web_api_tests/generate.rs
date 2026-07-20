use super::fixtures::{test_layout_basic, write_layout_file};
use super::helpers::*;

#[tokio::test]
async fn test_generate_firmware_missing_layout() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);

    let (status, json) = post_json(&app, "/api/layouts/nonexistent.json/generate", json!({})).await;

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
    let path = temp_dir.path().join("no_keyboard.json");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = post_json(&app, "/api/layouts/no_keyboard.json/generate", json!({})).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("no keyboard"));
}

#[tokio::test]
async fn test_generate_firmware_no_layout_variant() {
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create a layout file without layout_variant defined
    let mut layout = test_layout_basic(2, 3);
    layout.metadata.layout_variant = None;
    let path = temp_dir.path().join("no_variant.json");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = post_json(&app, "/api/layouts/no_variant.json/generate", json!({})).await;

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
    let path = temp_dir.path().join("test_layout.json");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    let (status, json) = post_json(&app, "/api/layouts/test_layout.json/generate", json!({})).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "started");
    assert!(json["job"]["id"].is_string());
    assert!(json["job"]["download_url"].is_string());
    assert_eq!(json["job"]["status"], "pending");
    assert_eq!(json["job"]["layout_filename"], "test_layout.json");
}

#[tokio::test]
async fn test_generate_job_status() {
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create a layout file
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("test_layout.json");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    // Start a job
    let (status, json) = post_json(&app, "/api/layouts/test_layout.json/generate", json!({})).await;
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
    let path = temp_dir.path().join("test_layout.json");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    // Start a job
    let (status, json) = post_json(&app, "/api/layouts/test_layout.json/generate", json!({})).await;
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
    let path = temp_dir.path().join("test_layout.json");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    // Start a job
    let (status, json) = post_json(&app, "/api/layouts/test_layout.json/generate", json!({})).await;
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
    let path = temp_dir.path().join("test_layout.json");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    // Start a job
    let _ = post_json(&app, "/api/layouts/test_layout.json/generate", json!({})).await;

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
    let path = temp_dir.path().join("test_layout.json");
    write_layout_file(&layout, &path).expect("Failed to write layout");

    let app = create_router(state);

    // GET the layout
    let (status, json) = get_json(&app, "/api/layouts/test_layout.json").await;

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

    // Create test layout with two distinct keys using the write helper
    let mut layout = test_layout_basic(2, 3);
    layout.metadata.name = "Swap Test".to_string();
    layout.metadata.description = "Test layout for swap".to_string();
    layout.metadata.author = "Test".to_string();
    layout.metadata.keyboard = Some("test".to_string());
    layout.metadata.layout_variant = Some("LAYOUT_test".to_string());

    // Set keycodes and color overrides on layer 0
    layout.layers[0].keys[0].keycode = "KC_Q".to_string();
    layout.layers[0].keys[0].color_override = Some(lazyqmk::models::RgbColor::new(255, 0, 0));
    layout.layers[0].keys[1].keycode = "KC_W".to_string();
    layout.layers[0].keys[1].color_override = Some(lazyqmk::models::RgbColor::new(0, 255, 0));

    let layout_path = workspace.join("test_swap.json");
    write_layout_file(&layout, &layout_path).expect("Failed to write layout");

    let app = create_router(state);

    // Get initial layout to verify starting state
    let (status, initial_json) = get_json(&app, "/api/layouts/test_swap.json").await;
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
        .uri("/api/layouts/test_swap.json/swap-keys")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&swap_request).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Get layout again to verify swap
    let (status, swapped_json) = get_json(&app, "/api/layouts/test_swap.json").await;
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

    // Verify the file on disk was actually updated (JSON format)
    let file_content = fs::read_to_string(workspace.join("test_swap.json")).unwrap();
    assert!(
        file_content.contains(r#""keycode": "KC_W""#),
        "Layout file should have KC_W (was swapped)"
    );
    assert!(
        file_content.contains(r#""keycode": "KC_Q""#),
        "Layout file should have KC_Q (was swapped)"
    );
}
