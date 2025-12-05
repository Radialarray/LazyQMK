//! Service layer for business logic.
//!
//! This module contains services that encapsulate complex business logic
//! and coordinate between different parts of the application.

pub mod geometry;
pub mod layouts;

// Re-export commonly used types and functions
pub use geometry::{
    build_geometry_for_layout, build_minimal_geometry, extract_base_keyboard, GeometryContext,
    GeometryResult,
};
pub use layouts::LayoutService;
