//! Component trait pattern for TUI components.
//!
//! This module defines the traits and types used to implement self-contained,
//! testable TUI components that can handle their own input and rendering.

use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::tui::Theme;

/// A component that can be rendered and handle input.
///
/// Components are self-contained UI elements that manage their own state,
/// handle keyboard input, and can emit events to communicate with the parent.
pub trait Component {
    /// Event type this component can emit
    type Event;

    /// Handle keyboard input.
    ///
    /// Returns `Some(Event)` if the component wants to signal something to the parent.
    /// Returns `None` if input was handled internally without needing parent action.
    fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event>;

    /// Render the component.
    ///
    /// The component should render itself within the provided area.
    fn render(&self, f: &mut Frame, area: Rect, theme: &Theme);
}

/// Extended trait for components that need shared context.
///
/// Some components need read access to shared application data (like keyboard geometry,
/// keycode database, etc.). This trait allows components to receive that context.
pub trait ContextualComponent {
    /// The type of context this component needs
    type Context;
    
    /// Event type this component can emit
    type Event;

    /// Handle keyboard input with access to shared context.
    fn handle_input(&mut self, key: KeyEvent, context: &Self::Context) -> Option<Self::Event>;

    /// Render the component with access to shared context.
    fn render(&self, f: &mut Frame, area: Rect, theme: &Theme, context: &Self::Context);
}



/// Context that contains what the color picker is modifying
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPickerContext {
    /// Coloring an individual key
    IndividualKey,
    /// Setting layer default color
    LayerDefault,
    /// Setting category color
    Category,
}

/// Context that contains what the category picker is modifying
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryPickerContext {
    /// Setting category for a single key
    #[allow(dead_code)]
    SingleKey,
    /// Setting category for multiple selected keys
    #[allow(dead_code)]
    MultipleKeys,
}
