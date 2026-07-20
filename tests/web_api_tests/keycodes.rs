use super::helpers::*;

#[tokio::test]
async fn test_list_keycodes_all() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keycodes").await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["keycodes"].is_array());
    assert!(json["total"].as_u64().unwrap() > 100);
}

#[tokio::test]
async fn test_list_keycodes_search() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keycodes?search=KC_A").await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["keycodes"].is_array());
    // Should find KC_A at minimum
    let keycodes = json["keycodes"].as_array().unwrap();
    assert!(keycodes.iter().any(|kc| kc["code"] == "KC_A"));
}

#[tokio::test]
async fn test_list_keycodes_by_category() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keycodes?category=navigation").await;

    assert_eq!(status, StatusCode::OK);
    let keycodes = json["keycodes"].as_array().unwrap();
    // All returned keycodes should be in navigation category
    assert!(keycodes.iter().all(|kc| kc["category"] == "navigation"));
}

#[tokio::test]
async fn test_list_categories() {
    let (state, _temp_dir) = create_test_state();
    let app = create_router(state);

    let (status, json) = get_json(&app, "/api/keycodes/categories").await;

    assert_eq!(status, StatusCode::OK);
    let categories = json["categories"].as_array().unwrap();
    assert!(!categories.is_empty());

    // Should have common categories
    let category_ids: Vec<&str> = categories
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert!(category_ids.contains(&"basic"));
    assert!(category_ids.contains(&"navigation"));
    assert!(category_ids.contains(&"layers"));
}
