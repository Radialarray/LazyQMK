//! Component trait pattern for TUI components.
//!
//! This module defines the traits and types used to implement self-contained,
//! testable TUI components that can handle their own input and rendering.

use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::models::RgbColor;
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

    /// Check if component should close.
    ///
    /// Returns `true` if the component has finished its work and should be closed.
    /// Default implementation returns `false`.
    fn should_close(&self) -> bool {
        false
    }
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

    /// Check if component should close.
    fn should_close(&self) -> bool {
        false
    }
}

/// Events that can be emitted by popup components.
///
/// These events are emitted by components and processed by the parent (AppState)
/// to update application state or trigger actions.
#[derive(Debug, Clone)]
pub enum ComponentEvent {
    // Selection events
    /// User selected a keycode
    KeycodeSelected(String),
    
    /// User selected a color
    ColorSelected(RgbColor),
    
    /// User cleared/reset a color
    ColorCleared,
    
    /// User selected a category
    CategorySelected(String),
    
    /// User selected a layer index
    LayerSelected(usize),
    
    /// User selected modifier keys
    ModifiersSelected(Vec<String>),

    // Action events
    /// User created a new layer
    LayerAdded {
        /// Index where layer was inserted
        index: usize,
        /// Name of the new layer
        name: String,
    },
    
    /// User deleted a layer
    LayerDeleted(usize),
    
    /// User renamed a layer
    LayerRenamed {
        /// Index of the renamed layer
        index: usize,
        /// New name
        name: String,
    },
    
    /// User reordered layers
    LayerReordered {
        /// Original index
        from: usize,
        /// New index
        to: usize,
    },
    
    /// User created a category
    CategoryAdded {
        /// Category ID
        id: String,
        /// Category name
        name: String,
        /// Category color
        color: RgbColor,
    },
    
    /// User deleted a category
    CategoryDeleted(String),
    
    /// User updated a category
    CategoryUpdated {
        /// Category ID
        id: String,
        /// New name (if changed)
        name: Option<String>,
        /// New color (if changed)
        color: Option<RgbColor>,
    },

    // State change events
    /// Layout metadata was updated
    MetadataUpdated {
        /// New keyboard path
        keyboard: Option<String>,
        /// New variant path
        variant: Option<String>,
        /// New layout name
        name: Option<String>,
        /// New author
        author: Option<String>,
    },
    
    /// Application settings were updated
    SettingsUpdated {
        /// New QMK firmware path
        qmk_firmware_path: Option<String>,
        /// New layouts directory
        layouts_dir: Option<String>,
        /// New theme name
        theme: Option<String>,
    },

    // Navigation events
    /// User loaded a template
    TemplateLoaded(std::path::PathBuf),
    
    /// User saved a template
    TemplateSaved {
        /// Path where template was saved
        path: std::path::PathBuf,
        /// Template name
        name: String,
    },
    
    /// User loaded a layout
    LayoutLoaded(std::path::PathBuf),
    
    /// User selected a keyboard
    KeyboardSelected {
        /// Keyboard path (e.g., "ferris/0_2")
        keyboard: String,
        /// Optional variant
        variant: Option<String>,
    },

    // Dismissal
    /// User cancelled without making changes
    Cancelled,
    
    /// Component closed naturally (e.g., help overlay dismissed)
    Closed,
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
    SingleKey,
    /// Setting category for multiple selected keys
    MultipleKeys,
}
