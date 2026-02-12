//! Dependency checking and environment validation.
//!
//! This module provides tools to check that the QMK development environment
//! is properly set up with all required external dependencies.

pub mod checker;
pub mod formatter;

// Re-export checker types
pub use checker::{DependencyChecker, DependencyStatus, ToolStatus};

// Re-export formatter types
pub use formatter::{DoctorFormatter, JsonDependency, JsonOutput, OutputFormat, Platform};
