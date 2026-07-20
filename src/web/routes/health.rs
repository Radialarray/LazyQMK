//! Health check and informational endpoints.

use axum::Json;

use super::super::dto::{EffectInfo, EffectsListResponse, HealthResponse};
use crate::models::RgbMatrixEffect;

/// GET /health - Health check endpoint.
pub(super) async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// GET /api/effects - List available RGB matrix effects.
pub(super) async fn list_effects() -> Json<EffectsListResponse> {
    let effects = RgbMatrixEffect::all()
        .iter()
        .map(|e| EffectInfo {
            id: format!("{:?}", e).to_lowercase(),
            name: e.display_name().to_string(),
        })
        .collect();

    Json(EffectsListResponse { effects })
}
