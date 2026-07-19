//! Firmware validation before generation.
//!
//! This module performs pre-generation validation to ensure the layout
//! can be successfully compiled into QMK firmware.
//!
//! Sub-modules:
//! - [`report`] — `ValidationReport`, `ValidationError`, `ValidationWarning`
//!   and their formatting impls.
//! - [`core`] — `FirmwareValidator` and the `validate()` entry point.

mod core;
mod report;

pub use core::FirmwareValidator;
pub use report::ValidationErrorKind;

