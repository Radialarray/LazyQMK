//! Main TUI event loop.
//!
//! Provides `run_tui` which drives rendering and input dispatch.

use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use crate::tui::app_state::AppState;
use crate::tui::input::handle_key_event;
use crate::tui::render::render;
use crate::tui::theme::Theme;

/// Main event loop
pub fn run_tui(
    state: &mut AppState,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<()> {
    loop {
        // Apply theme based on user preference (Auto detects OS, Dark/Light are explicit)
        state.theme = Theme::from_mode(state.config.ui.theme_mode);

        // Decrement flash highlight counter
        if let Some((layer, pos, frames)) = state.flash_highlight {
            if frames > 1 {
                state.flash_highlight = Some((layer, pos, frames - 1));
            } else {
                state.flash_highlight = None;
            }
        }

        // Render current state
        terminal.draw(|f| render(f, state))?;

        // Poll for events with 100ms timeout
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if handle_key_event(state, key)? {
                        break;
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal resized, will re-render on next loop
                }
                // Ignore focus/mouse/paste — not used by this app
                _ => {}
            }
        }

        // Poll build state for updates
        if let Some(build_state) = &mut state.build_state {
            if build_state.poll() {
                // Build message received, will update on next render
            }
        }

        // Check if should quit
        if state.should_quit {
            break;
        }
    }

    Ok(())
}
