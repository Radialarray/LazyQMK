//! Parsing and serialization for various file formats.
//!
//! This module handles reading and writing keyboard layouts in JSON format
//! (primary, since 0.22.0) and legacy Markdown format (for migration).
//! Also parses QMK info.json files and generates firmware configuration.

pub mod json_serde;
pub mod keyboard_json;
pub mod layout;
pub mod template_gen;

// Re-export commonly used functions
pub use json_serde::{parse_json_layout, save_json_layout};

// Legacy markdown parser (kept for .md → .json migration)
#[allow(unused_imports)] // bin/lib split: lib tests use this
pub use layout::parse_markdown_layout;
#[allow(unused_imports)] // bin/lib split: lib tests use this
pub use template_gen::save_markdown_layout;
