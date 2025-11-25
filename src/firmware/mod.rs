//! Firmware generation and compilation.
//!
//! This module handles generating keymap.c and vial.json files,
//! as well as background compilation of QMK firmware.

pub mod builder;
pub mod generator;
pub mod validator;

// Re-export firmware types
pub use builder::{BuildState, BuildStatus};
pub use generator::FirmwareGenerator;
pub use validator::FirmwareValidator;
