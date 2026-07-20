use super::helpers::*;

#[tokio::test]
async fn test_health_check() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/health").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "healthy");
    assert!(json["version"].is_string());
}
