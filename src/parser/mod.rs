//! Parsing and serialization for various file formats.
//!
//! This module handles reading and writing keyboard layouts from Markdown,
//! parsing QMK info.json files, and generating firmware configuration files.

pub mod keyboard_json;

// Re-export commonly used functions
pub use keyboard_json::{
    build_keyboard_geometry, extract_layout_definition, extract_layout_names,
    parse_keyboard_info_json, scan_keyboards,
};
