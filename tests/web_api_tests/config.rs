use super::fixtures::{test_layout_basic, write_layout_file};
use super::helpers::*;

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
    let layout_path = temp_dir.path().join("workspace_test_layout.json");
    write_layout_file(&layout, &layout_path).expect("Failed to write layout");

    let app = create_router(state);

    // Verify the layout appears in /api/layouts
    let (status, json) = get_json(&app, "/api/layouts").await;

    assert_eq!(status, StatusCode::OK);
    let layouts = json["layouts"].as_array().unwrap();
    assert!(
        layouts
            .iter()
            .any(|l| l["filename"] == "workspace_test_layout.json"),
        "Layout should be found in workspace root"
    );
}
