//! Keycode display helpers shared across TUI and export rendering.
//!
//! These functions are NOT feature-gated (used by both `lazyqmk` binary
//! and the web feature) and are the single source of truth for the
//! short-form keycode display strings.
//!
//! The richer `keycode_db::display::format_modifier_long` (full
//! "Left Control" style) lives in the web-only `display` module.

/// Strip the `KC_` prefix from a simple keycode.
///
/// `"KC_A"` → `"A"`, `"KC_ENTER"` → `"ENTER"`, `"MO(1)"` → `"MO(1)"`.
#[must_use]
pub fn strip_kc_prefix(keycode: &str) -> String {
    keycode.strip_prefix("KC_").unwrap_or(keycode).to_string()
}

/// Format a modifier mask for compact display.
///
/// `"MOD_LCTL"` → `"C"`, `"MOD_LCTL | MOD_LSFT"` → `"CS"`,
/// `"MOD_LCTL | MOD_LSFT | MOD_LALT | MOD_LGUI"` → `"CSAG"`.
/// Falls back to the first 3 chars if no recognized modifier is found.
#[must_use]
pub fn format_modifier(mod_str: &str) -> String {
    let mut result = String::new();

    if mod_str.contains("LCTL") || mod_str.contains("RCTL") || mod_str.contains("MOD_CTL") {
        result.push('C');
    }
    if mod_str.contains("LSFT") || mod_str.contains("RSFT") || mod_str.contains("MOD_SFT") {
        result.push('S');
    }
    if mod_str.contains("LALT") || mod_str.contains("RALT") || mod_str.contains("MOD_ALT") {
        result.push('A');
    }
    if mod_str.contains("LGUI") || mod_str.contains("RGUI") || mod_str.contains("MOD_GUI") {
        result.push('G');
    }
    if mod_str.contains("MEH") {
        return "MEH".to_string();
    }
    if mod_str.contains("HYPR") {
        return "HYP".to_string();
    }

    if result.is_empty() {
        mod_str.chars().take(3).collect()
    } else {
        result
    }
}

