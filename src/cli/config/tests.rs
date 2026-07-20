//! Tests for config.
//!
//! Auto-extracted from config.rs.

use super::*;

use super::*;

#[test]
fn test_theme_mode_parsing() {
    assert_eq!(
        match "auto" {
            "auto" => ThemeMode::Auto,
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::Auto,
        },
        ThemeMode::Auto
    );
    assert_eq!(
        match "light" {
            "auto" => ThemeMode::Auto,
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::Auto,
        },
        ThemeMode::Light
    );
    assert_eq!(
        match "dark" {
            "auto" => ThemeMode::Auto,
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::Auto,
        },
        ThemeMode::Dark
    );
}
