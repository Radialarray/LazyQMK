//! Dependency checking and environment validation.
//!
//! This module provides tools to check that the QMK development environment
//! is properly set up with all required external dependencies.

pub mod checker;

// Re-export checker types
pub use checker::{DependencyChecker, DependencyStatus, ToolStatus};
