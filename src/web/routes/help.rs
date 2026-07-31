//! Help endpoint backed by `src/data/help.toml`.
//!
//! Exposes the same TOML-defined keybinding registry that the TUI's
//! `help_overlay` and `status_bar` consume (src/tui/dialog/help_registry.rs).
//! Lets the WebUI surface keyboard shortcuts via a `?`-key overlay without
//! hardcoding labels.

use axum::{extract::State, Json};
use serde::Serialize;
use std::collections::BTreeMap;

use crate::tui::dialog::help_registry::HelpRegistry;

use super::super::error::AppError;
use super::super::AppState;

/// Single keybinding entry.
#[derive(Debug, Serialize)]
pub struct HelpBindingDto {
    /// Primary key combo (e.g. "Ctrl+S", "?").
    pub keys: Vec<String>,
    /// Alternate key combos (e.g. vim-style "h j k l" for arrows).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_keys: Option<Vec<String>>,
    /// Action label.
    pub action: String,
    /// Optional short hint shown in status bar.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// Lower number = higher priority.
    pub priority: u32,
}

/// Context grouping related keybindings (e.g. main view, color picker).
#[derive(Debug, Serialize)]
pub struct HelpContextDto {
    /// Identifier (e.g. "main", "keycode_picker").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of the context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Bindings, sorted by priority then key.
    pub bindings: Vec<HelpBindingDto>,
}

/// Top-level response shape.
#[derive(Debug, Serialize)]
pub struct HelpResponse {
    /// App-level metadata.
    pub app_name: String,
    /// Version string from help.toml `[meta]` if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// All keybinding contexts, sorted by id.
    pub contexts: Vec<HelpContextDto>,
}

/// GET /api/help
pub(super) async fn get_help(State(_state): State<AppState>) -> Result<Json<HelpResponse>, AppError> {
    let registry = HelpRegistry::load().map_err(|e| {
        AppError::with_details(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to parse help.toml",
            Some(e.to_string()),
        )
    })?;

    let mut contexts: BTreeMap<String, HelpContextDto> = BTreeMap::new();
    for context_id in registry.context_names() {
        let id_str = context_id.clone();
        let Some(context) = registry.get_context(&id_str) else {
            continue;
        };
        let bindings: Vec<HelpBindingDto> = context
            .bindings
            .iter()
            .map(|b| {
                let alt_keys = if b.alt_keys.is_empty() {
                    None
                } else {
                    Some(b.alt_keys.clone())
                };
                HelpBindingDto {
                    keys: b.keys.clone(),
                    alt_keys,
                    action: b.action.clone(),
                    hint: b.hint.clone(),
                    priority: b.priority,
                }
            })
            .collect();

        contexts.insert(
            id_str.clone(),
            HelpContextDto {
                id: id_str,
                name: context.name.clone(),
                description: Some(context.description.clone()),
                bindings,
            },
        );
    }

    Ok(Json(HelpResponse {
        app_name: registry.app_name().to_string(),
        version: None,
        contexts: contexts.into_values().collect(),
    }))
}