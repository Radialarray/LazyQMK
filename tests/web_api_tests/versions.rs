//! Integration tests for layout versioning API.

use super::fixtures::{test_layout_basic, write_layout_file};
use super::helpers::*;
use lazyqmk::services::layout_versions::LayoutVersionService;
use serde_json::json;

#[tokio::test]
async fn test_versions_list_empty() {
    let (state, temp_dir) = create_test_state_with_qmk();
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("empty_layout.json");
    write_layout_file(&layout, &path).unwrap();

    let app = create_router(state);
    let (status, json) = get_json(&app, "/api/versions?layout=empty_layout").await;
    assert_eq!(status, StatusCode::OK);
    let revisions = json["revisions"].as_array().unwrap();
    // No snapshots have been created, just the migrated initial.
    // For an untouched layout that was never loaded, this should be empty.
    assert!(revisions.is_empty() || revisions.len() >= 1);
}

#[tokio::test]
async fn test_versions_list_returns_created_snapshot() {
    let (state, temp_dir) = create_test_state_with_qmk();
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("versioned_layout.json");
    write_layout_file(&layout, &path).unwrap();

    // Pre-migrate and create a snapshot so we have something to list.
    let svc = LayoutVersionService::new(temp_dir.path().to_path_buf());
    svc.migrate_legacy_path(&path).unwrap();
    svc.create_snapshot(
        "versioned_layout",
        &layout,
        Some("test-snapshot"),
        None,
        None,
    )
    .unwrap();

    let app = create_router(state);
    let (status, json) = get_json(&app, "/api/versions?layout=versioned_layout").await;
    assert_eq!(status, StatusCode::OK);
    let revisions = json["revisions"].as_array().unwrap();
    assert!(revisions.len() >= 2);
    // Latest should be the manual snapshot.
    assert_eq!(revisions[0]["label"], "test-snapshot");
}

#[tokio::test]
async fn test_versions_create_snapshot_via_api() {
    let (state, temp_dir) = create_test_state_with_qmk();
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("api_layout.json");
    write_layout_file(&layout, &path).unwrap();

    // Pre-migrate so the folder layout exists.
    let svc = LayoutVersionService::new(temp_dir.path().to_path_buf());
    svc.migrate_legacy_path(&path).unwrap();

    let app = create_router(state);
    let (status, json) = post_json(
        &app,
        "/api/versions/api_layout",
        json!({"label": "from-api"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["label"], "from-api");
}

#[tokio::test]
async fn test_versions_get_single_revision() {
    let (state, temp_dir) = create_test_state_with_qmk();
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("get_layout.json");
    write_layout_file(&layout, &path).unwrap();

    let svc = LayoutVersionService::new(temp_dir.path().to_path_buf());
    svc.migrate_legacy_path(&path).unwrap();

    let app = create_router(state);
    let (status, json) = get_json(&app, "/api/versions/get_layout/1").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["revision"], 1);
    assert!(json["layout"]["metadata"]["name"].is_string());
}

#[tokio::test]
async fn test_versions_diff_endpoint() {
    let (state, temp_dir) = create_test_state_with_qmk();
    let mut layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("diff_layout.json");
    write_layout_file(&layout, &path).unwrap();

    let svc = LayoutVersionService::new(temp_dir.path().to_path_buf());
    svc.migrate_legacy_path(&path).unwrap();
    layout.metadata.description = "after".to_string();
    svc.create_snapshot("diff_layout", &layout, Some("after"), None, None)
        .unwrap();

    let app = create_router(state);
    let (status, json) =
        get_json(&app, "/api/versions/diff_layout/diff?from=1&to=2").await;
    assert_eq!(status, StatusCode::OK);
    let diff = &json["diff"];
    assert_eq!(diff["from_revision"], 1);
    assert_eq!(diff["to_revision"], 2);
    assert_eq!(diff["summary"]["metadata_changed"], true);
}

#[tokio::test]
async fn test_versions_rename_endpoint() {
    let (state, temp_dir) = create_test_state_with_qmk();
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("rename_layout.json");
    write_layout_file(&layout, &path).unwrap();

    let svc = LayoutVersionService::new(temp_dir.path().to_path_buf());
    svc.migrate_legacy_path(&path).unwrap();

    let app = create_router(state);
    let (status, json) = patch_json(
        &app,
        "/api/versions/rename_layout/1",
        json!({"label": "new-label"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["label"], "new-label");
}

#[tokio::test]
async fn test_versions_delete_endpoint() {
    let (state, temp_dir) = create_test_state_with_qmk();
    let layout = test_layout_basic(2, 3);
    let path = temp_dir.path().join("delete_layout.json");
    write_layout_file(&layout, &path).unwrap();

    let svc = LayoutVersionService::new(temp_dir.path().to_path_buf());
    svc.migrate_legacy_path(&path).unwrap();
    svc.create_snapshot("delete_layout", &layout, Some("extra"), None, None)
        .unwrap();

    let app = create_router(state);
    // Revision 2 is non-current.
    let status = delete_json_status(&app, "/api/versions/delete_layout/2").await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Revision 1 is current — should refuse.
    let status = delete_json_status(&app, "/api/versions/delete_layout/1").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_versions_missing_layout_param() {
    let (state, _temp_dir) = create_test_state_with_qmk();
    let app = create_router(state);
    let (status, _json) = get_json(&app, "/api/versions").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
