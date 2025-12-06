//! Service layer for business logic.
//!
//! This module contains services that encapsulate complex business logic
//! and coordinate between different parts of the application.

pub mod geometry;
pub mod layouts;

// Re-export commonly used types and functions
pub use layouts::LayoutService;
