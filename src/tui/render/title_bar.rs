//! Title bar rendering.
//!
//! Renders the top bar showing layout name, keyboard, layer info, mode, and dirty state.

use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app_state::AppState;
use crate::tui::app_state::SelectionMode;
use crate::tui::help_registry::HelpRegistry;
use crate::tui::popup_type::PopupVisualKind;

/// Render title bar with layout name and dirty indicator
pub(super) fn render_title_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let draft_state = if state.dirty { "Unsaved" } else { "Saved" };
    let mode = if let Some(active_popup) = &state.active_popup {
        match active_popup.visual_kind() {
            PopupVisualKind::Settings => "Settings",
            PopupVisualKind::Wizard => "Onboarding",
            kind => kind.mode_label(),
        }
    } else if let Some(selection_mode) = &state.selection_mode {
        match selection_mode {
            SelectionMode::Normal => "Selection",
            SelectionMode::Rectangle { .. } => "Rectangle select",
            SelectionMode::Swap { .. } => "Swap",
        }
    } else {
        "Key edit"
    };
    let keyboard = state
        .layout
        .metadata
        .keyboard
        .clone()
        .unwrap_or_else(|| "Keyboard not set".to_string());
    let layer_name = state
        .layout
        .layers
        .get(state.current_layer)
        .map_or("Unknown layer", |layer| layer.name.as_str());
    let build_summary = state.build_state.as_ref().and_then(|build| {
        if build.status == crate::firmware::BuildStatus::Idle {
            None
        } else if build.last_message.is_empty() {
            Some(build.status.to_string())
        } else {
            Some(format!("{} • {}", build.status, build.last_message))
        }
    });
    let title = format!(
        " {} | {} | {} | L{} {} [active] | {} ",
        HelpRegistry::default().app_name(),
        state.layout.metadata.name,
        keyboard,
        state.current_layer,
        layer_name,
        draft_state
    );
    let subtitle = if let Some(build_summary) = build_summary {
        format!(" Mode: {mode}  |  Build: {build_summary}")
    } else {
        format!(" Mode: {mode}")
    };

    let title_widget = Paragraph::new(vec![Line::from(title), Line::from(subtitle)])
        .style(
            Style::default()
                .fg(state.theme.primary)
                .bg(state.theme.background),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Editor ")
                .style(Style::default().bg(state.theme.background)),
        );

    f.render_widget(title_widget, area);
}
