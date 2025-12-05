//! QMK keycode database and validation.
//!
//! This module provides access to the embedded keycode database,
//! validation functions, and fuzzy search capabilities.

#![allow(clippy::doc_link_with_quotes)]

use anyhow::{Context, Result};
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

#[allow(dead_code)]
impl KeycodeDb {
    /// Loads the keycode database from embedded category files.
    pub fn load() -> Result<Self> {
        // Load categories index
        let categories_json = include_str!("categories.json");
        let index: CategoriesIndex =
            serde_json::from_str(categories_json).context("Failed to parse categories.json")?;

        // Load each category file
        let mut all_keycodes = Vec::new();

        // Include all category files at compile time
        let category_files: &[(&str, &str)] = &[
            ("basic", include_str!("categories/basic.json")),
            ("symbols", include_str!("categories/symbols.json")),
            ("shifted", include_str!("categories/shifted.json")),
            ("navigation", include_str!("categories/navigation.json")),
            ("function", include_str!("categories/function.json")),
            ("numpad", include_str!("categories/numpad.json")),
            ("modifiers", include_str!("categories/modifiers.json")),
            ("mod_combo", include_str!("categories/mod_combo.json")),
            ("mod_tap", include_str!("categories/mod_tap.json")),
            ("layers", include_str!("categories/layers.json")),
            ("one_shot", include_str!("categories/one_shot.json")),
            ("mouse", include_str!("categories/mouse.json")),
            ("media", include_str!("categories/media.json")),
            ("rgb", include_str!("categories/rgb.json")),
            ("backlight", include_str!("categories/backlight.json")),
            ("audio", include_str!("categories/audio.json")),
            ("system", include_str!("categories/system.json")),
            (
                "international",
                include_str!("categories/international.json"),
            ),
            ("advanced", include_str!("categories/advanced.json")),
            ("magic", include_str!("categories/magic.json")),
        ];

        for (cat_id, json_data) in category_files {
            let cat_file: CategoryFile = serde_json::from_str(json_data)
                .with_context(|| format!("Failed to parse {cat_id}.json"))?;
            all_keycodes.extend(cat_file.keycodes);
        }

        let mut lookup = HashMap::new();
        let mut patterns = Vec::new();

        // Build lookup table
        for (idx, keycode) in all_keycodes.iter().enumerate() {
            lookup.insert(keycode.code.clone(), idx);

            // Add aliases to lookup
            for alias in &keycode.aliases {
                lookup.insert(alias.clone(), idx);
            }

            // Compile regex patterns
            if let Some(pattern) = &keycode.pattern {
                if let Ok(regex) = Regex::new(pattern) {
                    patterns.push((keycode.category.clone(), regex));
                }
            }
        }

        Ok(Self {
            keycodes: all_keycodes,
            categories: index.categories,
            lookup,
            patterns,
        })
    }

    /// Validates a keycode against the database.
    ///
    /// Returns true if the keycode exists or matches a pattern (e.g., MO(5)).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyboard_configurator::keycode_db::KeycodeDb;
    ///
    /// let db = KeycodeDb::load().unwrap();
    /// assert!(db.is_valid("KC_A"));
    /// assert!(db.is_valid("KC_TRNS"));
    /// assert!(db.is_valid("MO(5)"));
    /// assert!(!db.is_valid("INVALID_KEY"));
    /// ```
    #[must_use]
    pub fn is_valid(&self, keycode: &str) -> bool {
        // Check direct lookup first (O(1))
        if self.lookup.contains_key(keycode) {
            return true;
        }

        // Check pattern matches (for MO(n), TG(n), etc.)
        for (_category, regex) in &self.patterns {
            if regex.is_match(keycode) {
                return true;
            }
        }

        false
    }

    /// Gets a keycode definition by code.
    #[must_use]
    pub fn get(&self, keycode: &str) -> Option<&KeycodeDefinition> {
        let idx = self.lookup.get(keycode)?;
        self.keycodes.get(*idx)
    }

    /// Searches for keycodes by fuzzy matching the code, name, or description.
    ///
    /// Returns keycodes where the query appears as a substring (case-insensitive)
    /// in the code, name, or description. Results are sorted by relevance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyboard_configurator::keycode_db::KeycodeDb;
    ///
    /// let db = KeycodeDb::load().unwrap();
    /// let results = db.search("arr");
    /// // Returns KC_LEFT, KC_RIGHT, KC_UP, KC_DOWN (arrow keys)
    /// ```
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<&KeycodeDefinition> {
        if query.is_empty() {
            return self.keycodes.iter().collect();
        }

        let query_lower = query.to_lowercase();
        let mut results: Vec<(&KeycodeDefinition, i32)> = self
            .keycodes
            .iter()
            .filter_map(|keycode| {
                let code_lower = keycode.code.to_lowercase();
                let name_lower = keycode.name.to_lowercase();
                let desc_lower = keycode
                    .description
                    .as_ref()
                    .map(|d| d.to_lowercase())
                    .unwrap_or_default();

                // Exact match (highest priority)
                if code_lower == query_lower || name_lower == query_lower {
                    return Some((keycode, 100));
                }

                // Starts with query (high priority)
                if code_lower.starts_with(&query_lower) || name_lower.starts_with(&query_lower) {
                    return Some((keycode, 50));
                }

                // Contains query in code or name (medium priority)
                if code_lower.contains(&query_lower) || name_lower.contains(&query_lower) {
                    return Some((keycode, 10));
                }

                // Contains query in description (lower priority)
                if desc_lower.contains(&query_lower) {
                    return Some((keycode, 5));
                }

                None
            })
            .collect();

        // Sort by relevance (descending)
        results.sort_by(|a, b| b.1.cmp(&a.1));

        results.into_iter().map(|(keycode, _)| keycode).collect()
    }

    /// Searches for keycodes within a specific category.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyboard_configurator::keycode_db::KeycodeDb;
    ///
    /// let db = KeycodeDb::load().unwrap();
    /// let nav_keys = db.search_in_category("", "navigation");
    /// // Returns all navigation keys
    /// ```
    #[must_use]
    pub fn search_in_category(&self, query: &str, category_id: &str) -> Vec<&KeycodeDefinition> {
        self.search(query)
            .into_iter()
            .filter(|k| k.category == category_id)
            .collect()
    }

    /// Gets all keycodes in a category.
    #[must_use]
    pub fn get_category_keycodes(&self, category_id: &str) -> Vec<&KeycodeDefinition> {
        self.keycodes
            .iter()
            .filter(|k| k.category == category_id)
            .collect()
    }

    /// Gets all categories.
    #[must_use]
    pub fn categories(&self) -> &[KeycodeCategory] {
        &self.categories
    }

    /// Gets a category by ID.
    #[must_use]
    pub fn get_category(&self, id: &str) -> Option<&KeycodeCategory> {
        self.categories.iter().find(|c| c.id == id)
    }

    /// Gets the total number of keycodes.
    #[must_use]
    pub const fn keycode_count(&self) -> usize {
        self.keycodes.len()
    }

    /// Gets the total number of categories.
    #[must_use]
    pub const fn category_count(&self) -> usize {
        self.categories.len()
    }

    /// Check if a keycode is parameterized (requires additional input).
    #[must_use]
    pub fn is_parameterized(&self, code: &str) -> bool {
        self.get(code).is_some_and(|kc| !kc.params.is_empty())
    }

    /// Get the parameters for a keycode, if any.
    #[must_use]
    pub fn get_params(&self, code: &str) -> Option<&[KeycodeParam]> {
        self.get(code)
            .filter(|kc| !kc.params.is_empty())
            .map(|kc| kc.params.as_slice())
    }

    /// Get the prefix (code without parentheses) for a parameterized keycode.
    /// E.g., "`LCG()`" -> "LCG", "`LCTL_T()`" -> "`LCTL_T`"
    #[must_use]
    pub fn get_prefix(code: &str) -> Option<&str> {
        code.strip_suffix("()")
    }

    /// Get display abbreviation for a mod-tap prefix.
    ///
    /// Returns a short display name suitable for showing on a key.
    /// E.g., "`LCTL_T`" -> "CTL", "`MEH_T`" -> "MEH", "`LGUI_T`" -> "GUI"
    #[must_use]
    pub fn get_mod_tap_display(&self, prefix: &str) -> Option<&'static str> {
        // Map mod-tap prefixes to their short display names
        // This uses static strings to match the KeycodeDb patterns
        match prefix {
            "LCTL_T" | "RCTL_T" | "CTL_T" => Some("CTL"),
            "LSFT_T" | "RSFT_T" | "SFT_T" => Some("SFT"),
            "LALT_T" | "RALT_T" | "ALT_T" | "LOPT_T" | "ROPT_T" | "OPT_T" => Some("ALT"),
            "LGUI_T" | "RGUI_T" | "GUI_T" | "LCMD_T" | "RCMD_T" | "CMD_T" | "LWIN_T" | "RWIN_T"
            | "WIN_T" => Some("GUI"),
            "LSG_T" | "RSG_T" | "SGUI_T" => Some("S+G"),
            "LCA_T" | "RCA_T" => Some("C+A"),
            "LCS_T" | "RCS_T" => Some("C+S"),
            "LCAG_T" | "RCAG_T" => Some("CAG"),
            "LSA_T" | "RSA_T" | "SAGR_T" => Some("S+A"),
            "LSAG_T" | "RSAG_T" => Some("SAG"),
            "MEH_T" => Some("MEH"),
            "HYPR_T" | "ALL_T" => Some("HYP"),
            _ => None,
        }
    }

    /// Parse a keycode to extract tap-hold information if applicable.
    ///
    /// Returns `Some(TapHoldInfo)` if the keycode is a tap-hold type,
    /// `None` otherwise.
    #[must_use]
    pub fn parse_tap_hold(&self, keycode: &str) -> Option<TapHoldInfo> {
        // LT(layer, keycode) - Layer Tap
        if keycode.starts_with("LT(") && keycode.ends_with(')') {
            let inner = &keycode[3..keycode.len() - 1];
            if let Some((layer, tap)) = inner.split_once(',') {
                return Some(TapHoldInfo {
                    tap_hold_type: TapHoldType::LayerTap,
                    prefix: "LT".to_string(),
                    arg1: layer.trim().to_string(),
                    arg2: Some(tap.trim().to_string()),
                });
            }
        }

        // MT(mod, keycode) - Custom Mod Tap
        if keycode.starts_with("MT(") && keycode.ends_with(')') {
            let inner = &keycode[3..keycode.len() - 1];
            if let Some((modifier, tap)) = inner.split_once(',') {
                return Some(TapHoldInfo {
                    tap_hold_type: TapHoldType::ModTap,
                    prefix: "MT".to_string(),
                    arg1: modifier.trim().to_string(),
                    arg2: Some(tap.trim().to_string()),
                });
            }
        }

        // LM(layer, mod) - Layer Mod
        if keycode.starts_with("LM(") && keycode.ends_with(')') {
            let inner = &keycode[3..keycode.len() - 1];
            if let Some((layer, modifier)) = inner.split_once(',') {
                return Some(TapHoldInfo {
                    tap_hold_type: TapHoldType::LayerMod,
                    prefix: "LM".to_string(),
                    arg1: layer.trim().to_string(),
                    arg2: Some(modifier.trim().to_string()),
                });
            }
        }

        // SH_T(keycode) - Swap Hands Tap
        if keycode.starts_with("SH_T(") && keycode.ends_with(')') {
            let tap = &keycode[5..keycode.len() - 1];
            return Some(TapHoldInfo {
                tap_hold_type: TapHoldType::SwapHands,
                prefix: "SH_T".to_string(),
                arg1: tap.trim().to_string(),
                arg2: None,
            });
        }

        // Check mod-tap category keycodes (LCTL_T, LSFT_T, etc.)
        for kc in self.get_category_keycodes("mod_tap") {
            // Skip MT() which we already handled
            if kc.code == "MT()" {
                continue;
            }

            // Get the prefix without ()
            if let Some(prefix) = kc.code.strip_suffix("()") {
                let full_prefix = format!("{prefix}(");
                if keycode.starts_with(&full_prefix) && keycode.ends_with(')') {
                    let tap = &keycode[full_prefix.len()..keycode.len() - 1];
                    return Some(TapHoldInfo {
                        tap_hold_type: TapHoldType::ModTapNamed,
                        prefix: prefix.to_string(),
                        arg1: tap.trim().to_string(),
                        arg2: None,
                    });
                }
            }
        }

        None
    }

    /// Check if a keycode is a layer-switching keycode.
    ///
    /// This dynamically checks against all keycodes in the "layers" category
    /// that have a layer parameter.
    #[must_use]
    pub fn is_layer_keycode(&self, keycode: &str) -> bool {
        // Check against patterns from the layers category
        for kc in self.get_category_keycodes("layers") {
            // Skip KC_TRNS, KC_NO and similar non-parameterized layer keycodes
            if kc.params.is_empty() {
                continue;
            }

            // Check if keycode matches this layer keycode's pattern
            if let Some(pattern) = &kc.pattern {
                if let Ok(regex) = Regex::new(pattern) {
                    if regex.is_match(keycode) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Parse a layer keycode to extract (prefix, `layer_ref`, suffix).
    ///
    /// For simple keycodes like MO(1), returns ("MO", "1", "")
    /// For compound keycodes like LT(1, `KC_A`), returns ("LT", "1", ", `KC_A`)")
    #[must_use]
    pub fn parse_layer_keycode(&self, keycode: &str) -> Option<(String, String, String)> {
        // Try to match against layers category keycodes
        for kc in self.get_category_keycodes("layers") {
            if kc.params.is_empty() {
                continue;
            }

            // Get the prefix from the code (e.g., "MO" from "MO()")
            let prefix = kc.code.strip_suffix("()")?;
            let prefix_with_paren = format!("{prefix}(");

            if keycode.starts_with(&prefix_with_paren) && keycode.ends_with(')') {
                let inner = &keycode[prefix_with_paren.len()..keycode.len() - 1];

                // Check if this is a compound keycode (has comma)
                if kc.params.len() > 1 {
                    // LT(layer, kc) or LM(layer, mod)
                    if let Some(comma_pos) = inner.find(',') {
                        let layer_ref = inner[..comma_pos].trim().to_string();
                        let suffix = format!(", {})", inner[comma_pos + 1..].trim());
                        return Some((prefix.to_string(), layer_ref, suffix));
                    }
                } else {
                    // Simple layer keycode: MO(layer), TG(layer), etc.
                    return Some((prefix.to_string(), inner.trim().to_string(), String::new()));
                }
            }
        }
        None
    }

    /// Get the list of simple layer keycode prefixes (single parameter).
    /// E.g., ["MO", "TG", "TO", "DF", "OSL", "TT", "PDF"]
    #[must_use]
    pub fn get_simple_layer_prefixes(&self) -> Vec<&str> {
        self.get_category_keycodes("layers")
            .iter()
            .filter(|kc| kc.params.len() == 1)
            .filter_map(|kc| kc.code.strip_suffix("()"))
            .collect()
    }

    /// Get the list of compound layer keycode prefixes (multiple parameters).
    /// E.g., ["LT", "LM"]
    #[must_use]
    pub fn get_compound_layer_prefixes(&self) -> Vec<&str> {
        self.get_category_keycodes("layers")
            .iter()
            .filter(|kc| kc.params.len() > 1)
            .filter_map(|kc| kc.code.strip_suffix("()"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_db() -> KeycodeDb {
        KeycodeDb::load().expect("Failed to load keycode database")
    }

    #[test]
    fn test_load_database() {
        let db = get_test_db();
        assert!(db.keycode_count() > 100);
        assert!(db.category_count() > 5);
    }

    #[test]
    fn test_is_valid_basic_keys() {
        let db = get_test_db();
        assert!(db.is_valid("KC_A"));
        assert!(db.is_valid("KC_B"));
        assert!(db.is_valid("KC_1"));
        assert!(db.is_valid("KC_ENT"));
        assert!(db.is_valid("KC_ENTER")); // Alias
    }

    #[test]
    fn test_is_valid_special_keys() {
        let db = get_test_db();
        assert!(db.is_valid("KC_TRNS"));
        assert!(db.is_valid("KC_TRANSPARENT")); // Alias
        assert!(db.is_valid("KC_NO"));
    }

    #[test]
    fn test_is_valid_layer_switching() {
        let db = get_test_db();
        assert!(db.is_valid("MO(0)"));
        assert!(db.is_valid("MO(1)"));
        assert!(db.is_valid("MO(5)")); // Pattern match
        assert!(db.is_valid("TG(0)"));
        assert!(db.is_valid("TO(3)"));
    }

    #[test]
    fn test_is_valid_invalid_keys() {
        let db = get_test_db();
        assert!(!db.is_valid("INVALID_KEY"));
        assert!(!db.is_valid("KC_FOO"));
        assert!(!db.is_valid(""));
    }

    #[test]
    fn test_get_keycode() {
        let db = get_test_db();
        let keycode = db.get("KC_A").unwrap();
        assert_eq!(keycode.code, "KC_A");
        assert_eq!(keycode.name, "A");
        assert_eq!(keycode.category, "basic");
    }

    #[test]
    fn test_get_keycode_by_alias() {
        let db = get_test_db();
        let keycode = db.get("KC_ENTER").unwrap();
        assert_eq!(keycode.code, "KC_ENT");
        assert_eq!(keycode.name, "Enter");
    }

    #[test]
    fn test_search_empty_query() {
        let db = get_test_db();
        let results = db.search("");
        assert_eq!(results.len(), db.keycode_count());
    }

    #[test]
    fn test_search_exact_match() {
        let db = get_test_db();
        let results = db.search("KC_A");
        assert!(!results.is_empty());
        assert_eq!(results[0].code, "KC_A");
    }

    #[test]
    fn test_search_partial_match() {
        let db = get_test_db();
        let results = db.search("arr");
        // Should match arrow keys (KC_LEFT, KC_RIGHT, KC_UP, KC_DOWN)
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .any(|k| k.name.to_lowercase().contains("arrow")));
    }

    #[test]
    fn test_search_case_insensitive() {
        let db = get_test_db();
        let results_upper = db.search("ENTER");
        let results_lower = db.search("enter");
        assert_eq!(results_upper.len(), results_lower.len());
        assert!(!results_upper.is_empty());
    }

    #[test]
    fn test_search_in_category() {
        let db = get_test_db();
        let nav_keys = db.search_in_category("", "navigation");
        assert!(!nav_keys.is_empty());
        assert!(nav_keys.iter().all(|k| k.category == "navigation"));
    }

    #[test]
    fn test_get_category_keycodes() {
        let db = get_test_db();
        let function_keys = db.get_category_keycodes("function");
        assert!(!function_keys.is_empty());
        assert!(function_keys.iter().any(|k| k.code == "KC_F1"));
        assert!(function_keys.iter().any(|k| k.code == "KC_F12"));
    }

    #[test]
    fn test_get_category() {
        let db = get_test_db();
        let category = db.get_category("basic").unwrap();
        assert_eq!(category.id, "basic");
        assert_eq!(category.name, "Basic");
    }

    #[test]
    fn test_categories() {
        let db = get_test_db();
        let categories = db.categories();
        assert!(categories.len() >= 8);
        assert!(categories.iter().any(|c| c.id == "basic"));
        assert!(categories.iter().any(|c| c.id == "navigation"));
        assert!(categories.iter().any(|c| c.id == "media"));
    }

    #[test]
    fn test_is_parameterized() {
        let db = get_test_db();
        // Layer keycodes are parameterized
        assert!(db.is_parameterized("MO()"));
        assert!(db.is_parameterized("LT()"));
        assert!(db.is_parameterized("LM()"));
        // Modifier wrappers are parameterized
        assert!(db.is_parameterized("LCG()"));
        assert!(db.is_parameterized("LCTL()"));
        // Mod-taps are parameterized
        assert!(db.is_parameterized("LCTL_T()"));
        // Basic keys are NOT parameterized
        assert!(!db.is_parameterized("KC_A"));
        assert!(!db.is_parameterized("KC_ENT"));
    }

    #[test]
    fn test_get_params_layer_only() {
        let db = get_test_db();
        let params = db.get_params("MO()").unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].param_type, ParamType::Layer);
    }

    #[test]
    fn test_get_params_layer_tap() {
        let db = get_test_db();
        let params = db.get_params("LT()").unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].param_type, ParamType::Layer);
        assert_eq!(params[1].param_type, ParamType::Keycode);
    }

    #[test]
    fn test_get_params_layer_mod() {
        let db = get_test_db();
        let params = db.get_params("LM()").unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].param_type, ParamType::Layer);
        assert_eq!(params[1].param_type, ParamType::Modifier);
    }

    #[test]
    fn test_get_params_modifier_wrapper() {
        let db = get_test_db();
        let params = db.get_params("LCG()").unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].param_type, ParamType::Keycode);
    }

    #[test]
    fn test_get_params_mod_tap() {
        let db = get_test_db();
        let params = db.get_params("LCTL_T()").unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].param_type, ParamType::Keycode);
    }

    #[test]
    fn test_get_params_custom_mod_tap() {
        let db = get_test_db();
        let params = db.get_params("MT()").unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].param_type, ParamType::Modifier);
        assert_eq!(params[1].param_type, ParamType::Keycode);
    }

    #[test]
    fn test_get_prefix() {
        assert_eq!(KeycodeDb::get_prefix("LCG()"), Some("LCG"));
        assert_eq!(KeycodeDb::get_prefix("LCTL_T()"), Some("LCTL_T"));
        assert_eq!(KeycodeDb::get_prefix("MO()"), Some("MO"));
        assert_eq!(KeycodeDb::get_prefix("KC_A"), None);
    }

    #[test]
    fn test_get_mod_tap_display() {
        let db = get_test_db();
        // Basic modifiers
        assert_eq!(db.get_mod_tap_display("LCTL_T"), Some("CTL"));
        assert_eq!(db.get_mod_tap_display("RCTL_T"), Some("CTL"));
        assert_eq!(db.get_mod_tap_display("LSFT_T"), Some("SFT"));
        assert_eq!(db.get_mod_tap_display("LALT_T"), Some("ALT"));
        assert_eq!(db.get_mod_tap_display("LGUI_T"), Some("GUI"));
        // Combo modifiers
        assert_eq!(db.get_mod_tap_display("MEH_T"), Some("MEH"));
        assert_eq!(db.get_mod_tap_display("HYPR_T"), Some("HYP"));
        assert_eq!(db.get_mod_tap_display("LCAG_T"), Some("CAG"));
        // Aliases
        assert_eq!(db.get_mod_tap_display("CMD_T"), Some("GUI"));
        assert_eq!(db.get_mod_tap_display("OPT_T"), Some("ALT"));
        // Not a mod-tap
        assert_eq!(db.get_mod_tap_display("KC_A"), None);
    }

    #[test]
    fn test_parse_tap_hold_layer_tap() {
        let db = get_test_db();
        let info = db.parse_tap_hold("LT(1, KC_A)").unwrap();
        assert_eq!(info.tap_hold_type, TapHoldType::LayerTap);
        assert_eq!(info.prefix, "LT");
        assert_eq!(info.arg1, "1");
        assert_eq!(info.arg2, Some("KC_A".to_string()));
    }

    #[test]
    fn test_parse_tap_hold_mod_tap() {
        let db = get_test_db();
        let info = db.parse_tap_hold("MT(MOD_LCTL, KC_A)").unwrap();
        assert_eq!(info.tap_hold_type, TapHoldType::ModTap);
        assert_eq!(info.prefix, "MT");
        assert_eq!(info.arg1, "MOD_LCTL");
        assert_eq!(info.arg2, Some("KC_A".to_string()));
    }

    #[test]
    fn test_parse_tap_hold_mod_tap_named() {
        let db = get_test_db();
        let info = db.parse_tap_hold("LCTL_T(KC_A)").unwrap();
        assert_eq!(info.tap_hold_type, TapHoldType::ModTapNamed);
        assert_eq!(info.prefix, "LCTL_T");
        assert_eq!(info.arg1, "KC_A");
        assert_eq!(info.arg2, None);
    }

    #[test]
    fn test_parse_tap_hold_layer_mod() {
        let db = get_test_db();
        let info = db.parse_tap_hold("LM(1, MOD_LCTL)").unwrap();
        assert_eq!(info.tap_hold_type, TapHoldType::LayerMod);
        assert_eq!(info.prefix, "LM");
        assert_eq!(info.arg1, "1");
        assert_eq!(info.arg2, Some("MOD_LCTL".to_string()));
    }

    #[test]
    fn test_parse_tap_hold_swap_hands() {
        let db = get_test_db();
        let info = db.parse_tap_hold("SH_T(KC_A)").unwrap();
        assert_eq!(info.tap_hold_type, TapHoldType::SwapHands);
        assert_eq!(info.prefix, "SH_T");
        assert_eq!(info.arg1, "KC_A");
        assert_eq!(info.arg2, None);
    }

    #[test]
    fn test_parse_tap_hold_not_tap_hold() {
        let db = get_test_db();
        assert!(db.parse_tap_hold("KC_A").is_none());
        assert!(db.parse_tap_hold("MO(1)").is_none());
        assert!(db.parse_tap_hold("TG(2)").is_none());
    }

    #[test]
    fn test_is_layer_keycode() {
        let db = get_test_db();
        // Simple layer keycodes
        assert!(db.is_layer_keycode("MO(1)"));
        assert!(db.is_layer_keycode("TG(2)"));
        assert!(db.is_layer_keycode("TO(0)"));
        assert!(db.is_layer_keycode("DF(1)"));
        assert!(db.is_layer_keycode("OSL(3)"));
        assert!(db.is_layer_keycode("TT(2)"));
        assert!(db.is_layer_keycode("PDF(0)"));
        // Compound layer keycodes
        assert!(db.is_layer_keycode("LT(1, KC_A)"));
        assert!(db.is_layer_keycode("LM(2, MOD_LCTL)"));
        // UUID references
        assert!(db.is_layer_keycode("MO(@abc-123)"));
        assert!(db.is_layer_keycode("LT(@layer-id, KC_SPC)"));
        // Not layer keycodes
        assert!(!db.is_layer_keycode("KC_A"));
        assert!(!db.is_layer_keycode("LCTL_T(KC_A)"));
        assert!(!db.is_layer_keycode("KC_TRNS")); // Non-parameterized
    }

    #[test]
    fn test_parse_layer_keycode_simple() {
        let db = get_test_db();
        let (prefix, layer_ref, suffix) = db.parse_layer_keycode("MO(1)").unwrap();
        assert_eq!(prefix, "MO");
        assert_eq!(layer_ref, "1");
        assert_eq!(suffix, "");
    }

    #[test]
    fn test_parse_layer_keycode_with_uuid() {
        let db = get_test_db();
        let (prefix, layer_ref, suffix) = db.parse_layer_keycode("TG(@abc-123)").unwrap();
        assert_eq!(prefix, "TG");
        assert_eq!(layer_ref, "@abc-123");
        assert_eq!(suffix, "");
    }

    #[test]
    fn test_parse_layer_keycode_compound() {
        let db = get_test_db();
        let (prefix, layer_ref, suffix) = db.parse_layer_keycode("LT(1, KC_A)").unwrap();
        assert_eq!(prefix, "LT");
        assert_eq!(layer_ref, "1");
        assert_eq!(suffix, ", KC_A)");
    }

    #[test]
    fn test_parse_layer_keycode_lm() {
        let db = get_test_db();
        let (prefix, layer_ref, suffix) =
            db.parse_layer_keycode("LM(@layer-id, MOD_LSFT)").unwrap();
        assert_eq!(prefix, "LM");
        assert_eq!(layer_ref, "@layer-id");
        assert_eq!(suffix, ", MOD_LSFT)");
    }

    #[test]
    fn test_parse_layer_keycode_not_layer() {
        let db = get_test_db();
        assert!(db.parse_layer_keycode("KC_A").is_none());
        assert!(db.parse_layer_keycode("LCTL_T(KC_A)").is_none());
    }

    #[test]
    fn test_get_simple_layer_prefixes() {
        let db = get_test_db();
        let prefixes = db.get_simple_layer_prefixes();
        assert!(prefixes.contains(&"MO"));
        assert!(prefixes.contains(&"TG"));
        assert!(prefixes.contains(&"TO"));
        assert!(prefixes.contains(&"DF"));
        assert!(prefixes.contains(&"OSL"));
        assert!(prefixes.contains(&"TT"));
        assert!(prefixes.contains(&"PDF"));
        // LT and LM are compound, not simple
        assert!(!prefixes.contains(&"LT"));
        assert!(!prefixes.contains(&"LM"));
    }

    #[test]
    fn test_get_compound_layer_prefixes() {
        let db = get_test_db();
        let prefixes = db.get_compound_layer_prefixes();
        assert!(prefixes.contains(&"LT"));
        assert!(prefixes.contains(&"LM"));
        // Simple layer keycodes should not be here
        assert!(!prefixes.contains(&"MO"));
        assert!(!prefixes.contains(&"TG"));
    }
}
