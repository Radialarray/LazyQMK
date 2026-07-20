//! Integration tests for the LazyQMK Web API.
//!
//! These tests require the `web` feature to be enabled:
//! ```bash
//! cargo test --features web web_api
//! ```

#![cfg(feature = "web")]

mod fixtures;

#[path = "web_api_tests/build.rs"]
mod build;
#[path = "web_api_tests/config.rs"]
mod config;
#[path = "web_api_tests/generate.rs"]
mod generate;
#[path = "web_api_tests/geometry.rs"]
mod geometry;
#[path = "web_api_tests/health.rs"]
mod health;
#[path = "web_api_tests/helpers.rs"]
mod helpers;
#[path = "web_api_tests/keyboard_wizard.rs"]
mod keyboard_wizard;
#[path = "web_api_tests/keycodes.rs"]
mod keycodes;
#[path = "web_api_tests/layouts.rs"]
mod layouts;
#[path = "web_api_tests/preflight.rs"]
mod preflight;
#[path = "web_api_tests/templates.rs"]
mod templates;
