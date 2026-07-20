//! Tests for help_overlay.
//!
//! Auto-extracted from help_overlay.rs.

use super::*;

use super::*;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// Helper to create a test terminal with given dimensions
fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).expect("Failed to create test terminal")
}

// =========================================================================
// HelpOverlayState unit tests
// =========================================================================

#[test]
fn test_help_overlay_state_initial() {
    let state = HelpOverlayState::new();
    assert_eq!(state.scroll_offset, 0, "Initial scroll should be at top");
}

#[test]
fn test_scroll_up_from_zero_stays_zero() {
    let mut state = HelpOverlayState::new();
    state.scroll_up();
    assert_eq!(state.scroll_offset, 0, "Scroll up from 0 should stay at 0");
}

#[test]
fn test_scroll_down_increments() {
    let mut state = HelpOverlayState::new();
    state.scroll_down();
    assert_eq!(
        state.scroll_offset, 1,
        "Scroll down should increment offset"
    );
    state.scroll_down();
    assert_eq!(state.scroll_offset, 2, "Scroll down again should increment");
}

#[test]
fn test_scroll_to_top_resets() {
    let mut state = HelpOverlayState::new();
    state.scroll_offset = 50;
    state.scroll_to_top();
    assert_eq!(state.scroll_offset, 0, "Scroll to top should reset to 0");
}

#[test]
fn test_scroll_to_bottom_sets_max() {
    let mut state = HelpOverlayState::new();
    state.scroll_to_bottom();
    assert_eq!(
        state.scroll_offset,
        usize::MAX,
        "Scroll to bottom should set to MAX"
    );
}

#[test]
fn test_page_up_page_down() {
    let mut state = HelpOverlayState::new();
    state.scroll_offset = 50;

    state.page_down(10);
    assert_eq!(state.scroll_offset, 60, "Page down by 10 should add 10");

    state.page_up(15);
    assert_eq!(state.scroll_offset, 45, "Page up by 15 should subtract 15");

    state.page_up(100);
    assert_eq!(state.scroll_offset, 0, "Page up beyond 0 should clamp to 0");
}

#[test]
fn test_compute_max_scroll() {
    // Content smaller than viewport: no scrolling possible
    assert_eq!(
        HelpOverlayState::compute_max_scroll(10, 20),
        0,
        "Content smaller than viewport should have max_scroll=0"
    );

    // Content equals viewport: no scrolling possible
    assert_eq!(
        HelpOverlayState::compute_max_scroll(20, 20),
        0,
        "Content equals viewport should have max_scroll=0"
    );

    // Content larger than viewport
    assert_eq!(
        HelpOverlayState::compute_max_scroll(100, 20),
        80,
        "100 lines with 20 visible should have max_scroll=80"
    );
}

#[test]
fn test_clamped_offset_within_bounds() {
    let mut state = HelpOverlayState::new();
    state.scroll_offset = 30;

    // 100 total lines, 20 visible => max_scroll = 80
    let clamped = state.clamped_offset(100, 20);
    assert_eq!(
        clamped, 30,
        "Offset 30 is within bounds, should be unchanged"
    );
}

#[test]
fn test_clamped_offset_exceeds_max() {
    let mut state = HelpOverlayState::new();
    state.scroll_offset = 95;

    // 100 total lines, 20 visible => max_scroll = 80
    let clamped = state.clamped_offset(100, 20);
    assert_eq!(clamped, 80, "Offset 95 should clamp to max_scroll=80");
}

#[test]
fn test_clamped_offset_at_usize_max() {
    let mut state = HelpOverlayState::new();
    state.scroll_to_bottom(); // Sets to usize::MAX

    // 100 total lines, 20 visible => max_scroll = 80
    let clamped = state.clamped_offset(100, 20);
    assert_eq!(clamped, 80, "usize::MAX should clamp to max_scroll=80");
}

#[test]
fn test_clamped_offset_content_fits_viewport() {
    let mut state = HelpOverlayState::new();
    state.scroll_offset = 10;

    // 15 total lines, 20 visible => max_scroll = 0 (content fits)
    let clamped = state.clamped_offset(15, 20);
    assert_eq!(clamped, 0, "When content fits viewport, offset should be 0");
}

// =========================================================================
// HelpOverlay Component tests
// =========================================================================

#[test]
fn test_help_overlay_new() {
    let overlay = HelpOverlay::new();
    assert_eq!(overlay.state.scroll_offset, 0);
}

#[test]
fn test_help_overlay_default() {
    let overlay = HelpOverlay::default();
    assert_eq!(overlay.state.scroll_offset, 0);
}

// =========================================================================
// Rendering tests using TestBackend
// =========================================================================

#[test]
fn test_render_normal_terminal_shows_content() {
    // Test that rendering on a normal-sized terminal shows content
    let mut terminal = create_test_terminal(80, 40);
    let overlay = HelpOverlay::new();
    let theme = Theme::default();

    terminal
        .draw(|frame| {
            let area = frame.area();
            use crate::tui::component::Component;
            overlay.render(frame, area, &theme);
        })
        .expect("Failed to render");

    // Get the buffer and check that title is present
    let buffer = terminal.backend().buffer();
    let content = buffer_to_string(buffer);

    assert!(
        content.contains("Help"),
        "Rendered content should contain 'Help' title. Got:\n{}",
        content
    );

    // Should contain at least one help entry (NAVIGATION section exists)
    assert!(
        content.contains("NAVIGATION") || content.contains("Navigate"),
        "Rendered content should contain help entries"
    );
}

#[test]
fn test_render_tiny_terminal_shows_something() {
    // Test that even a very small terminal doesn't panic and shows something
    let mut terminal = create_test_terminal(20, 10);
    let overlay = HelpOverlay::new();
    let theme = Theme::default();

    // This should not panic
    terminal
        .draw(|frame| {
            let area = frame.area();
            use crate::tui::component::Component;
            overlay.render(frame, area, &theme);
        })
        .expect("Failed to render on tiny terminal");

    // Get the buffer
    let buffer = terminal.backend().buffer();
    let content = buffer_to_string(buffer);

    // Should still show border or title fragment
    assert!(
        !content.trim().is_empty(),
        "Even tiny terminal should render something"
    );
}

#[test]
fn test_render_excessive_scroll_clamped() {
    // Test that excessive scroll offset is clamped and doesn't produce blank output
    let mut terminal = create_test_terminal(80, 40);
    let mut overlay = HelpOverlay::new();
    let theme = Theme::default();

    // Set scroll to an excessive value
    overlay.state.scroll_offset = 10000;

    terminal
        .draw(|frame| {
            let area = frame.area();
            use crate::tui::component::Component;
            overlay.render(frame, area, &theme);
        })
        .expect("Failed to render with excessive scroll");

    let buffer = terminal.backend().buffer();
    let content = buffer_to_string(buffer);

    // Should still have visible content (the footer at minimum)
    assert!(
        content.contains("Help") || content.contains("═"),
        "Excessive scroll should be clamped and still show content. Got:\n{}",
        content
    );
}

#[test]
fn test_render_at_scroll_bottom_shows_footer() {
    // Test that scrolling to bottom shows the footer
    let mut terminal = create_test_terminal(80, 40);
    let mut overlay = HelpOverlay::new();
    let theme = Theme::default();

    overlay.state.scroll_to_bottom();

    terminal
        .draw(|frame| {
            let area = frame.area();
            use crate::tui::component::Component;
            overlay.render(frame, area, &theme);
        })
        .expect("Failed to render at scroll bottom");

    let buffer = terminal.backend().buffer();
    let content = buffer_to_string(buffer);

    // Footer contains "Press '?' to close" text
    assert!(
        content.contains("build log")
            || content.contains("Templates")
            || content.contains("BUILD SYSTEM")
            || content.contains("─"),
        "Scroll to bottom should show footer content. Got:\n{}",
        content
    );
}

#[test]
fn test_render_zero_height_terminal() {
    // Edge case: terminal with 0 height should not panic
    let mut terminal = create_test_terminal(80, 0);
    let overlay = HelpOverlay::new();
    let theme = Theme::default();

    // This should not panic
    let result = terminal.draw(|frame| {
        let area = frame.area();
        use crate::tui::component::Component;
        overlay.render(frame, area, &theme);
    });

    assert!(
        result.is_ok(),
        "Rendering on 0-height terminal should not panic"
    );
}

// =========================================================================
// Input handling tests
// =========================================================================

#[test]
fn test_handle_input_close_keys() {
    use crate::tui::component::Component;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut overlay = HelpOverlay::new();

    // '?' should close
    let event = overlay.handle_input(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE));
    assert!(matches!(event, Some(HelpOverlayEvent::Closed)));

    // Esc should close
    let event = overlay.handle_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(matches!(event, Some(HelpOverlayEvent::Closed)));

    // 'q' should close
    let event = overlay.handle_input(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
    assert!(matches!(event, Some(HelpOverlayEvent::Closed)));
}

#[test]
fn test_handle_input_scroll_keys() {
    use crate::tui::component::Component;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut overlay = HelpOverlay::new();

    // Down arrow scrolls down
    let event = overlay.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    assert!(event.is_none());
    assert_eq!(overlay.state.scroll_offset, 1);

    // 'j' also scrolls down
    let event = overlay.handle_input(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
    assert!(event.is_none());
    assert_eq!(overlay.state.scroll_offset, 2);

    // Up arrow scrolls up
    let event = overlay.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    assert!(event.is_none());
    assert_eq!(overlay.state.scroll_offset, 1);

    // 'k' also scrolls up
    let event = overlay.handle_input(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
    assert!(event.is_none());
    assert_eq!(overlay.state.scroll_offset, 0);
}

#[test]
fn test_handle_input_home_end() {
    use crate::tui::component::Component;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut overlay = HelpOverlay::new();
    overlay.state.scroll_offset = 50;

    // Home goes to top
    let event = overlay.handle_input(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));
    assert!(event.is_none());
    assert_eq!(overlay.state.scroll_offset, 0);

    // End goes to bottom (sets to MAX, will be clamped at render)
    let event = overlay.handle_input(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
    assert!(event.is_none());
    assert_eq!(overlay.state.scroll_offset, usize::MAX);
}

// =========================================================================
// Content generation tests
// =========================================================================

#[test]
fn test_get_help_content_not_empty() {
    let theme = Theme::default();
    let content = HelpOverlayState::get_help_content(&theme);

    assert!(!content.is_empty(), "Help content should not be empty");
    assert!(
        content.len() > 10,
        "Help content should have multiple lines"
    );
}

#[test]
fn test_get_help_content_has_sections() {
    let theme = Theme::default();
    let content = HelpOverlayState::get_help_content(&theme);

    // Convert to string for easier checking
    let text: String = content
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.contains("MOVE AROUND"), "Should have movement section");
    assert!(
        text.contains("WORK WITH LAYERS"),
        "Should have layer task section"
    );
    assert!(text.contains("EDIT KEYS"), "Should have key task section");
}

// =========================================================================
// Helper functions
// =========================================================================

/// Convert a buffer to a single string for content checking
fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
    let mut result = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = &buffer[(x, y)];
            result.push_str(cell.symbol());
        }
        result.push('\n');
    }
    result
}
