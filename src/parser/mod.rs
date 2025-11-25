//! Parsing and serialization for various file formats.
//!
//! This module handles reading and writing keyboard layouts from Markdown,
//! parsing QMK info.json files, and generating firmware configuration files.

pub mod keyboard_json;
pub mod layout;
pub mod template_gen;

// Re-export commonly used functions
pub use layout::parse_markdown_layout;
pub use template_gen::save_markdown_layout;
