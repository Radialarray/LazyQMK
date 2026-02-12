//! Dependency checking and environment validation.
//!
//! This module provides tools to check that the QMK development environment
//! is properly set up with all required external dependencies.

pub mod checker;
pub mod formatter;

// Re-export checker types
pub use checker::{DependencyChecker, DependencyStatus, ToolStatus};

// Re-export formatter types (allow unused for public API exports)
#[allow(unused_imports)]
pub use formatter::{DoctorFormatter, JsonDependency, JsonOutput, OutputFormat, Platform};
