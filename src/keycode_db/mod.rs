//! QMK keycode database and validation.
//!
//! This module provides access to the embedded keycode database,
//! validation functions, and fuzzy search capabilities.

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
}

/// Database schema from keycodes.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeycodeDatabase {
    version: String,
    categories: Vec<KeycodeCategory>,
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

#[allow(dead_code)]
impl KeycodeDb {
    /// Loads the keycode database from the embedded JSON file.
    pub fn load() -> Result<Self> {
        let json_data = include_str!("keycodes.json");
        let db: KeycodeDatabase =
            serde_json::from_str(json_data).context("Failed to parse embedded keycodes.json")?;

        let mut lookup = HashMap::new();
        let mut patterns = Vec::new();

        // Build lookup table
        for (idx, keycode) in db.keycodes.iter().enumerate() {
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
            keycodes: db.keycodes,
            categories: db.categories,
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
    /// use keyboard_tui::keycode_db::KeycodeDb;
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
    /// use keyboard_tui::keycode_db::KeycodeDb;
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
    /// use keyboard_tui::keycode_db::KeycodeDb;
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
}
