//! Input handling dispatch for the TUI.
//!
//! Routes keyboard events to the appropriate handler based on current state.

mod dispatch;

pub use dispatch::handle_key_event;
