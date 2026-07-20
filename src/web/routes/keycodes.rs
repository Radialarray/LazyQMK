//! Keycode search and category listing endpoints.

use axum::{
    extract::{Query, State},
    Json,
};

use super::super::dto::{
    CategoryInfo, CategoryListResponse, KeycodeInfo, KeycodeListResponse, KeycodeQuery,
};
use super::super::AppState;

/// GET /api/keycodes - Query keycode database.
pub(super) async fn list_keycodes(
    State(state): State<AppState>,
    Query(query): Query<KeycodeQuery>,
) -> Json<KeycodeListResponse> {
    let search = query.search.as_deref().unwrap_or("");

    let keycodes: Vec<KeycodeInfo> = match &query.category {
        Some(cat) => state
            .keycode_db
            .search_in_category(search, cat)
            .into_iter()
            .map(KeycodeInfo::from)
            .collect(),
        None => state
            .keycode_db
            .search(search)
            .into_iter()
            .map(KeycodeInfo::from)
            .collect(),
    };

    let total = keycodes.len();
    Json(KeycodeListResponse { keycodes, total })
}

/// GET /api/keycodes/categories - List keycode categories.
pub(super) async fn list_categories(State(state): State<AppState>) -> Json<CategoryListResponse> {
    let categories = state
        .keycode_db
        .categories()
        .iter()
        .map(CategoryInfo::from)
        .collect();

    Json(CategoryListResponse { categories })
}
