//! QMK keycode database and validation.
//!
//! This module provides access to the embedded keycode database,
//! validation functions, and fuzzy search capabilities.
//!
//! Sub-modules:
//! - [`db`] — `KeycodeDb` struct and its large impl block (load, search,
//!   param/prefix/parse helpers, language accessors).
//! - [`display`] — display metadata for the web Key Details panel
//!   (cfg-gated on `web` feature).

#![allow(clippy::doc_link_with_quotes)]

mod db;

#[cfg(feature = "web")]
mod display;

// Re-exports for web feature - used by web::mod.rs but may appear unused
// when compiling the main binary with web feature enabled.
#[cfg(feature = "web")]
#[allow(unused_imports)] // cfg-gated; bin doesn't link web feature path
pub use display::{
    ActionKind, KeyDetailAction, KeyDisplay, KeyDisplayMetadata, TapDanceDisplayInfo,
};


use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;





/// Category of keycodes for organization in the picker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeycodeCategory {
    /// Category ID (e.g., "basic", "navigation")
    pub id: String,
    /// Display name (e.g., "Basic Keys", "Navigation")
    pub name: String,
    /// Description of what keys are in this category
    pub description: String,
}

/// Type of parameter a keycode expects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    /// Needs a basic keycode (opens keycode picker)
    Keycode,
    /// Needs a layer selection (opens layer picker)
    Layer,
    /// Needs modifier selection (opens modifier picker)
    Modifier,
    /// Needs a tap dance action selection (opens tap dance picker)
    #[serde(rename = "tapdance")]
    TapDance,
}

/// Parameter definition for parameterized keycodes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeycodeParam {
    /// Type of parameter
    #[serde(rename = "type")]
    pub param_type: ParamType,
    /// Parameter name (for display)
    pub name: String,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

/// Individual keycode definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeycodeDefinition {
    /// QMK keycode (e.g., "`KC_A`", "MO(1)")
    pub code: String,
    /// Display name (e.g., "A", "Momentary Layer 1")
    pub name: String,
    /// Category ID
    pub category: String,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
    /// Optional regex pattern for validation (e.g., "MO\\((\\d+)\\)")
    #[serde(default)]
    pub pattern: Option<String>,
    /// Alternative keycode names/aliases
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Parameters this keycode requires (for parameterized keycodes)
    #[serde(default)]
    pub params: Vec<KeycodeParam>,
}

/// Categories index file schema (categories.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CategoriesIndex {
    version: String,
    categories: Vec<KeycodeCategory>,
}

/// Category file schema (categories/*.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CategoryFile {
    category: KeycodeCategory,
    keycodes: Vec<KeycodeDefinition>,
}

/// Language definition with metadata about the keyboard layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageDefinition {
    /// Language ID (e.g., "german", "french")
    pub id: String,
    /// Display name (e.g., "German", "French")
    pub name: String,
    /// Description of the layout
    #[serde(default)]
    pub description: Option<String>,
    /// Keycode prefix (e.g., "DE_", "FR_")
    pub prefix: String,
    /// QMK header file path (e.g., "keymap_extras/keymap_german.h")
    pub header: String,
}

/// Language file schema (languages/*.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LanguageFile {
    language: LanguageFileMeta,
    keycodes: Vec<LanguageKeycode>,
}

/// Language metadata in individual language files
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LanguageFileMeta {
    id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    prefix: String,
    header: String,
}

/// Keycode definition in language files (simplified, no category needed)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LanguageKeycode {
    code: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
}

/// Stored language keycodes with metadata
#[derive(Debug, Clone)]
pub struct LanguageKeycodes {
    /// Language metadata
    pub language: LanguageDefinition,
    /// Keycodes for this language
    pub keycodes: Vec<KeycodeDefinition>,
}

/// QMK keycode database with fast lookup and search capabilities.
///
/// The database is embedded in the binary at compile time and loaded
/// lazily on first access. It provides O(1) keycode validation and
/// fuzzy search for the keycode picker.
#[derive(Debug, Clone)]
pub struct KeycodeDb {
    /// All keycode definitions
    keycodes: Vec<KeycodeDefinition>,
    /// Category definitions
    categories: Vec<KeycodeCategory>,
    /// Fast lookup by keycode string
    lookup: HashMap<String, usize>,
    /// Compiled regex patterns for parameterized keycodes (MO(n), TG(n), etc.)
    patterns: Vec<(String, Regex)>,
    /// Language-specific keycodes (loaded separately from main categories)
    languages: Vec<LanguageKeycodes>,
}

/// Type of tap-hold keycode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapHoldType {
    /// LT(layer, keycode) - Layer Tap
    LayerTap,
    /// MT(mod, keycode) - Custom Mod Tap
    ModTap,
    /// Named mod-tap like `LCTL_T(keycode)`
    ModTapNamed,
    /// LM(layer, mod) - Layer Mod
    LayerMod,
    /// `SH_T(keycode)` - Swap Hands Tap
    SwapHands,
}

/// Information about a parsed tap-hold keycode
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TapHoldInfo {
    /// Type of tap-hold
    pub tap_hold_type: TapHoldType,
    /// The prefix (e.g., "LT", "MT", "`LCTL_T`")
    pub prefix: String,
    /// First argument (layer for LT/LM, modifier for MT, keycode for others)
    pub arg1: String,
    /// Second argument if any (keycode for LT/MT, modifier for LM)
    pub arg2: Option<String>,
}
