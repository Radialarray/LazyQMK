use super::fixtures::{test_layout_basic, write_layout_file};
use super::helpers::*;

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
    let layout_path = temp_dir.path().join("new_layout.json");
    assert!(layout_path.exists());
}

#[tokio::test]
async fn test_create_layout_already_exists() {
    let (state, temp_dir) = create_test_state_with_qmk();

    // Create an existing file
    let existing_path = temp_dir.path().join("existing_layout.json");
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
    let layout_path = temp_dir.path().join("switch_test.json");
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
    let layout_path = temp_dir.path().join("no_keyboard.json");
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
    let source_filename = "source_for_conflict.json";
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
    let expected_filename = format!("{}.json", expected_filename);
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
    let template_filename = format!("{}.json", unique_name.to_lowercase().replace(' ', "_"));
    let template_path = template_dir.join(&template_filename);
    write_layout_file(&template, &template_path).expect("Failed to write template");

    let app = create_router(state);

    let target_filename = format!("new_from_template_{}.json", std::process::id());
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
        "target_filename": "new_layout.json"
    });

    let (status, json) = post_json(
        &app,
        "/api/templates/nonexistent_template.json/apply",
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
    let template_filename = format!("{}.json", unique_name.to_lowercase().replace(' ', "_"));
    let template_path = template_dir.join(&template_filename);
    write_layout_file(&template, &template_path).expect("Failed to write template");

    // Pre-create a layout at the target path
    let target_filename = "existing_layout.json";
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
        "target_filename": "new_layout.json"
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
    let template_filename = format!("{}.json", unique_name.to_lowercase().replace(' ', "_"));
    let template_path = template_dir.join(&template_filename);
    write_layout_file(&template, &template_path).expect("Failed to write template");

    let app = create_router(state);

    // Path traversal in target filename
    let request = json!({
        "target_filename": "../evil_layout.json"
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
    let source_filename = "source_for_empty_name.json";
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
    let source_filename = "source_for_long_name.json";
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
    let source_filename = "source_for_non_ascii_tag.json";
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
    let source_filename = "source_for_uppercase_tag.json";
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
    let source_filename = "source_for_empty_tag.json";
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
