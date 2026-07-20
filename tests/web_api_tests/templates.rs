use super::fixtures::{test_layout_basic, write_layout_file};
use super::helpers::*;

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
    let template_filename = format!("{}.json", unique_name.to_lowercase().replace(' ', "_"));
    let template_path = template_dir.join(&template_filename);
    write_layout_file(&template, &template_path).expect("Failed to write template");

    // Also create a non-template layout in workspace (should not appear in templates)
    let layout = test_layout_basic(2, 3);
    let layout_path = temp_dir.path().join("regular_layout.json");
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
    let template_filename = format!("{}.json", unique_name.to_lowercase().replace(' ', "_"));
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

    let (status, json) = get_json(&app, "/api/templates/nonexistent_template.json").await;

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
    let filename = format!("{}.json", unique_name.to_lowercase().replace(' ', "_"));
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
    let source_filename = "source_for_template.json";
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
    let expected_filename = format!("{}.json", expected_filename);

    // Cleanup
    cleanup_template(&template_dir, &expected_filename);

    assert_eq!(status, StatusCode::OK, "Response: {:?}", json);
    assert_eq!(json["name"], unique_name);
    assert!(std::path::Path::new(json["filename"].as_str().unwrap())
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json")));
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
        "/api/layouts/nonexistent_layout.json/save-as-template",
        request,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}
