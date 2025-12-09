//! Service layer for business logic.
//!
//! This module contains services that encapsulate complex business logic
//! and coordinate between different parts of the application.

pub mod geometry;
pub mod layouts;

// Re-export GeometryService if it exists, otherwise just re-export the module
// pub use geometry::GeometryService;
pub use layouts::LayoutService;
