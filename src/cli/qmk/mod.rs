//! QMK metadata CLI commands.
//!
//! Provides read-only commands for keyboard discovery and inspection,
//! without requiring the QMK CLI to be installed. Each command parses
//! `info.json` / `keyboard.json` directly.
//!
//! Sub-commands:
//! - [`list_keyboards`] — `qmk list-keyboards`
//! - [`list_layouts`] — `qmk list-layouts <keyboard>`
//! - [`geometry`] — `qmk geometry <keyboard> <layout>`

pub mod geometry;
pub mod list_keyboards;
pub mod list_layouts;

pub use geometry::GeometryArgs;
pub use list_keyboards::ListKeyboardsArgs;
pub use list_layouts::ListLayoutsArgs;
