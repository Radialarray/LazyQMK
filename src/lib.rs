//! LazyQMK Library
//!
//! This library provides core functionality for the LazyQMK application,
//! including parsing QMK info.json files, managing keyboard layouts, and
//! generating firmware code.

// Allow intentional type casts for terminal coordinates and QMK data structures
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_wrap)]

// Module declarations
pub mod app;
pub mod branding;
pub mod config;
pub mod constants;
pub mod firmware;
pub mod keycode_db;
pub mod models;
pub mod parser;
pub mod services;
pub mod shortcuts;
pub mod tui;
