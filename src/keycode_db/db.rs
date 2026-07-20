//! `KeycodeDb` struct + impl — the main keycode database.

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;

use super::{
    CategoriesIndex, CategoryFile, KeycodeCategory, KeycodeDefinition, KeycodeParam,
    LanguageDefinition, LanguageFile, LanguageKeycodes, TapHoldInfo, TapHoldType,
};
use crate::keycode_db::KeycodeDb;

#[allow(dead_code)] // bin/lib split: heavily used by tests + CLI subcommands
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
            ("tap_dance", include_str!("categories/tap_dance.json")),
        ];

        for (cat_id, json_data) in category_files {
            let cat_file: CategoryFile = serde_json::from_str(json_data)
                .with_context(|| format!("Failed to parse {cat_id}.json"))?;
            all_keycodes.extend(cat_file.keycodes);
        }

        let mut lookup = HashMap::new();
        let mut patterns = Vec::new();

        // Build lookup table for base keycodes
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

        // Load language-specific keycodes and merge them into the database so
        // validation recognizes DE_/FR_/… codes.
        let languages = Self::load_languages()?;
        let mut categories = index.categories;

        for lang in &languages {
            // Add a synthetic category for the language (helps picker grouping)
            categories.push(KeycodeCategory {
                id: format!("lang_{}", lang.language.id),
                name: lang.language.name.clone(),
                description: lang
                    .language
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("{} layout", lang.language.name)),
            });

            // Insert language keycodes into lookup and master list
            for kc in &lang.keycodes {
                let idx = all_keycodes.len();
                lookup.insert(kc.code.clone(), idx);
                all_keycodes.push(kc.clone());
            }
        }

        Ok(Self {
            keycodes: all_keycodes,
            categories,
            lookup,
            patterns,
            languages,
        })
    }

    /// Loads language-specific keycode files.
    fn load_languages() -> Result<Vec<LanguageKeycodes>> {
        // Include all language files at compile time
        let language_files: &[(&str, &str)] = &[
            ("german", include_str!("languages/german.json")),
            ("german_mac", include_str!("languages/german_mac.json")),
            ("french", include_str!("languages/french.json")),
            ("french_mac", include_str!("languages/french_mac.json")),
            ("spanish", include_str!("languages/spanish.json")),
            ("italian", include_str!("languages/italian.json")),
            ("uk", include_str!("languages/uk.json")),
            ("swedish", include_str!("languages/swedish.json")),
            ("norwegian", include_str!("languages/norwegian.json")),
            ("danish", include_str!("languages/danish.json")),
        ];

        let mut languages = Vec::new();

        for (lang_id, json_data) in language_files {
            let lang_file: LanguageFile = serde_json::from_str(json_data)
                .with_context(|| format!("Failed to parse languages/{lang_id}.json"))?;

            let language = LanguageDefinition {
                id: lang_file.language.id,
                name: lang_file.language.name,
                description: lang_file.language.description,
                prefix: lang_file.language.prefix.clone(),
                header: lang_file.language.header,
            };

            // Convert language keycodes to KeycodeDefinition with a synthetic category
            let keycodes: Vec<KeycodeDefinition> = lang_file
                .keycodes
                .into_iter()
                .map(|kc| KeycodeDefinition {
                    code: kc.code,
                    name: kc.name,
                    category: format!("lang_{}", lang_id),
                    description: kc.description,
                    pattern: None,
                    aliases: Vec::new(),
                    params: Vec::new(),
                })
                .collect();

            languages.push(LanguageKeycodes { language, keycodes });
        }

        Ok(languages)
    }

    /// Validates a keycode against the database.
    ///
    /// Returns true if the keycode exists or matches a pattern (e.g., MO(5)).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lazyqmk::keycode_db::KeycodeDb;
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
    /// use lazyqmk::keycode_db::KeycodeDb;
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
        results.sort_by_key(|b| std::cmp::Reverse(b.1));

        results.into_iter().map(|(keycode, _)| keycode).collect()
    }

    /// Searches for keycodes within a specific category.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lazyqmk::keycode_db::KeycodeDb;
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

    /// Gets all available languages.
    #[must_use]
    pub fn languages(&self) -> Vec<&LanguageDefinition> {
        self.languages.iter().map(|l| &l.language).collect()
    }

    /// Gets a language by ID.
    #[must_use]
    pub fn get_language(&self, id: &str) -> Option<&LanguageDefinition> {
        self.languages
            .iter()
            .find(|l| l.language.id == id)
            .map(|l| &l.language)
    }

    /// Gets all keycodes for a specific language.
    #[must_use]
    pub fn get_language_keycodes(&self, language_id: &str) -> Vec<&KeycodeDefinition> {
        self.languages
            .iter()
            .find(|l| l.language.id == language_id)
            .map(|l| l.keycodes.iter().collect())
            .unwrap_or_default()
    }

    /// Gets the total number of languages.
    #[must_use]
    pub fn language_count(&self) -> usize {
        self.languages.len()
    }

    /// Searches for keycodes within a specific language.
    #[must_use]
    pub fn search_in_language(&self, query: &str, language_id: &str) -> Vec<&KeycodeDefinition> {
        let keycodes = self.get_language_keycodes(language_id);
        if query.is_empty() {
            return keycodes;
        }

        let query_lower = query.to_lowercase();
        let mut results: Vec<(&KeycodeDefinition, i32)> = keycodes
            .into_iter()
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
        results.sort_by_key(|b| std::cmp::Reverse(b.1));

        results.into_iter().map(|(keycode, _)| keycode).collect()
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

    /// Parses a tap dance keycode and extracts the tap dance name.
    ///
    /// Returns the tap dance name if the keycode matches the `TD(name)` pattern.
    ///
    /// # Examples
    /// ```
    /// use lazyqmk::keycode_db::KeycodeDb;
    ///
    /// let db = KeycodeDb::load().unwrap();
    /// assert_eq!(db.parse_tap_dance_keycode("TD(esc_caps)"), Some("esc_caps".to_string()));
    /// assert_eq!(db.parse_tap_dance_keycode("TD(shift_123)"), Some("shift_123".to_string()));
    /// assert_eq!(db.parse_tap_dance_keycode("KC_A"), None);
    /// ```
    #[must_use]
    pub fn parse_tap_dance_keycode(&self, keycode: &str) -> Option<String> {
        // TD(name) pattern
        if keycode.starts_with("TD(") && keycode.ends_with(')') {
            let inner = &keycode[3..keycode.len() - 1];
            let name = inner.trim();

            // Validate it's a non-empty, valid C identifier
            if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return Some(name.to_string());
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
mod tests;
