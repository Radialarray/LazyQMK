//! Main content area rendering.
//!
//! Renders the keyboard widget in the central content area.

use ratatui::{layout::Rect, Frame};

use crate::tui::app_state::AppState;
use crate::tui::keyboard::KeyboardWidget;

/// Render main content (keyboard widget)
pub(super) fn render_main_content(f: &mut Frame, area: Rect, state: &AppState) {
    KeyboardWidget::render(f, area, state);
}
