//! Keyboard TUI Library
//!
//! This library provides core functionality for the Keyboard TUI application,
//! including parsing QMK info.json files, managing keyboard layouts, and
//! generating firmware code.

// Module declarations
pub mod config;
pub mod firmware;
pub mod keycode_db;
pub mod models;
pub mod parser;
pub mod tui;
