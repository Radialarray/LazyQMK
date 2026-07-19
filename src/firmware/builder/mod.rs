//! Background firmware compilation with progress tracking.
//!
//! This module handles spawning background threads to compile QMK firmware
//! and reporting progress via message channels.
//!
//! Sub-modules:
//! - [`state`] — `BuildStatus`, `LogLevel`, `BuildState` types and the
//!   `BuildState` impl that drives the build lifecycle.
//! - [`build`] — low-level helpers (`run_build`, `find_firmware_file`,
//!   `enhance_qmk_error`) used by `BuildState`.

mod build;
mod state;

pub use state::{BuildState, BuildStatus, LogLevel};

