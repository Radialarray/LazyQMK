//! LazyQMK Library
//!
//! This library provides core functionality for the LazyQMK application,
//! including parsing QMK info.json files, managing keyboard layouts, and
//! generating firmware code.

// Crate-wide clippy allows for numeric casts.
//
// Justification: cast sites are scattered across the entire crate
// (parser/, models/, export/, web/, tui/, firmware/) — 29+ cast sites
// were previously surfaced as warnings when these allows were absent.
// The casts are intentional in their contexts (terminal coordinates,
// QMK data structures, layout indices) and are bounded by external
// constraints (terminal dimensions, QMK matrix/LED counts).
//
// Per-function scoping would require touching every cast site across
// the crate, with no behavior change. The blanket crate-level allow is
// the pragmatic choice here; tighter scoping has been applied to the
// inner modules that no longer need it (parser/, tui/, models/rgb.rs,
// models/keyboard_geometry.rs, models/visual_layout_mapping.rs,
// models/layout.rs all had their own file-level cast allows removed).
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_wrap)]

// Module declarations
pub mod app;
pub mod branding;
pub mod cli;
pub mod config;
pub mod constants;
pub mod doctor;
pub mod export;
pub mod firmware;
pub mod keycode_db;
pub mod models;
pub mod parser;
pub mod services;
pub mod shortcuts;
pub mod tui;

#[cfg(feature = "web")]
pub mod web;
