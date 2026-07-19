//! Keycode → display-text formatting.
//!
//! - `format_keycode` — full formatter: handles tap-hold split, mod-tap,
//!   layer taps, simple modifiers, basic keycodes.
//! - `strip_kc_prefix` — drops the `KC_` prefix from simple keycodes.
//! - `format_modifier` — `MOD_LCTL` → `Ctrl`, etc.

/// Handles tap-hold keys with split display (e.g., "LT(1, `KC_A`)" -> "L1 / A")
pub(super) fn format_keycode(keycode: &str) -> String {
    // Handle Layer Tap: LT(layer, keycode)
    if let Some(inner) = keycode.strip_prefix("LT(") {
        if let Some(args) = inner.strip_suffix(')') {
            let parts: Vec<&str> = args.split(',').map(str::trim).collect();
            if parts.len() == 2 {
                let layer = parts[0].trim_start_matches('@'); // Remove @ prefix
                let tap = strip_kc_prefix(parts[1]);
                return format!("L{} / {}", layer, tap);
            }
        }
    }

    // Handle Mod Tap: MT(mod, keycode)
    if let Some(inner) = keycode.strip_prefix("MT(") {
        if let Some(args) = inner.strip_suffix(')') {
            let parts: Vec<&str> = args.split(',').map(str::trim).collect();
            if parts.len() == 2 {
                let mod_display = format_modifier(parts[0]);
                let tap = strip_kc_prefix(parts[1]);
                return format!("{} / {}", mod_display, tap);
            }
        }
    }

    // Handle named mod-tap: LCTL_T(keycode), LSFT_T(keycode), etc.
    for (prefix, mod_name) in &[
        ("LCTL_T", "CTL"),
        ("LSFT_T", "SFT"),
        ("LALT_T", "ALT"),
        ("LGUI_T", "GUI"),
        ("RCTL_T", "CTL"),
        ("RSFT_T", "SFT"),
        ("RALT_T", "ALT"),
        ("RGUI_T", "GUI"),
    ] {
        if let Some(inner) = keycode.strip_prefix(prefix) {
            if let Some(tap) = inner.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                return format!("{} / {}", mod_name, strip_kc_prefix(tap));
            }
        }
    }

    // Handle Layer Mod: LM(layer, mod)
    if let Some(inner) = keycode.strip_prefix("LM(") {
        if let Some(args) = inner.strip_suffix(')') {
            let parts: Vec<&str> = args.split(',').map(str::trim).collect();
            if parts.len() == 2 {
                let layer = parts[0].trim_start_matches('@');
                let mod_display = format_modifier(parts[1]);
                return format!("L{}+{}", layer, mod_display);
            }
        }
    }

    // Handle MO (momentary layer)
    if let Some(inner) = keycode.strip_prefix("MO(") {
        if let Some(layer) = inner.strip_suffix(')') {
            let layer = layer.trim_start_matches('@');
            return format!("▼L{}", layer);
        }
    }

    // Handle TG (toggle layer)
    if let Some(inner) = keycode.strip_prefix("TG(") {
        if let Some(layer) = inner.strip_suffix(')') {
            let layer = layer.trim_start_matches('@');
            return format!("TG{}", layer);
        }
    }

    // Handle Tap Dance: TD(name)
    if let Some(inner) = keycode.strip_prefix("TD(") {
        if let Some(name) = inner.strip_suffix(')') {
            return format!("TD:{}", name);
        }
    }

    // Simple keycode: strip KC_ prefix
    strip_kc_prefix(keycode)
}

/// Strips the "KC_" prefix from a keycode.
pub(super) fn strip_kc_prefix(keycode: &str) -> String {
    keycode.strip_prefix("KC_").unwrap_or(keycode).to_string()
}

/// Formats a modifier string for compact display.
pub(super) fn format_modifier(mod_str: &str) -> String {
    let mut result = String::new();

    if mod_str.contains("LCTL") || mod_str.contains("RCTL") {
        result.push('C');
    }
    if mod_str.contains("LSFT") || mod_str.contains("RSFT") {
        result.push('S');
    }
    if mod_str.contains("LALT") || mod_str.contains("RALT") {
        result.push('A');
    }
    if mod_str.contains("LGUI") || mod_str.contains("RGUI") {
        result.push('G');
    }

    if result.is_empty() {
        // Fallback: take first 3 chars
        mod_str.chars().take(3).collect()
    } else {
        result
    }
}