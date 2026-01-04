//! Export functionality for keyboard layouts.
//!
//! This module provides tools to export keyboard layout configurations in various formats,
//! currently focused on generating markdown documentation with visual representations
//! and configuration summaries.

pub mod color_legend;
pub mod keyboard_renderer;
pub mod layer_navigation;
pub mod settings_summary;
pub mod tap_dance_docs;

pub use color_legend::generate_color_legend;
pub use keyboard_renderer::render_layer_diagram;
pub use layer_navigation::generate_layer_navigation;
pub use settings_summary::generate_settings_summary;
pub use tap_dance_docs::generate_tap_dance_docs;
