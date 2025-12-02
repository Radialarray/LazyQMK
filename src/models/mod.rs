//! Data models for keyboard layouts, layers, and configuration.
//!
//! This module contains all the core data structures used throughout the application.
//! Models are designed to be independent of UI and business logic.

pub mod category;
pub mod keyboard_geometry;
pub mod layer;
pub mod layout;
pub mod rgb;
pub mod visual_layout_mapping;

// Re-export all model types
pub use category::Category;
pub use keyboard_geometry::{KeyGeometry, KeyboardGeometry};
pub use layer::{KeyDefinition, Layer, Position};
pub use layout::{InactiveKeyBehavior, Layout, LayoutMetadata};
pub use rgb::RgbColor;
pub use visual_layout_mapping::VisualLayoutMapping;
