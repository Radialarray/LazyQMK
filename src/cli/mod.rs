//! CLI command handlers for LazyQMK.
//!
//! This module provides headless, scriptable access to LazyQMK's core functionality
//! for automation, testing, and CI/CD integration.

pub mod category;
pub mod common;
pub mod config;
pub mod doctor;
pub mod export;
pub mod generate;
pub mod help;
pub mod inspect;
pub mod keycode;
pub mod keycodes;
pub mod layer_refs;
pub mod qmk;
pub mod tap_dance;
pub mod template;
pub mod validate;

// Re-export types used by main.rs and tests
pub use category::CategoryArgs;
pub use common::ExitCode;
pub use config::ConfigArgs;
pub use doctor::DoctorArgs;
pub use export::ExportArgs;
pub use generate::GenerateArgs;
pub use help::HelpArgs;
pub use inspect::InspectArgs;
pub use keycode::KeycodeArgs;
pub use keycodes::KeycodesArgs;
pub use layer_refs::LayerRefsArgs;
pub use qmk::{GeometryArgs, ListKeyboardsArgs, ListLayoutsArgs};
pub use tap_dance::TapDanceArgs;
pub use template::TemplateArgs;
pub use validate::ValidateArgs;
