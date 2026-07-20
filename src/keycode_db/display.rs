//! Keycode display parsing for web/TUI rendering.
//!
//! This module provides structured keycode breakdown for multi-action keys
//! (tap-hold, layer-tap, mod-tap, tap-dance, etc.) suitable for rendering
//! in both TUI and web interfaces.
//!
//! Note: This module is only compiled with the `web` feature. The helper
//! functions are used by `get_display_metadata` which is called from the
//! web API handler. Clippy may report dead code when checking the main
//! binary because it doesn't use the web module directly.

// Allow dead_code because these functions are used by get_display_metadata
// which is called from web::mod.rs, but clippy checking the main binary
// doesn't see this usage path.
#![allow(dead_code)]

use serde::Serialize;

use super::KeycodeDb;

/// Display labels for a key, with up to 3 parts.
///
/// - `primary`: Main label (always present, e.g., "A", "ESC", "L1")
/// - `secondary`: Secondary action (optional, e.g., hold action for mod-tap)
/// - `tertiary`: Third action (optional, e.g., for tap-dance with 3 actions)
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyDisplay {
    /// Primary/main label for the key (short form for in-key display)
    pub primary: String,
    /// Secondary label (e.g., hold action) - optional
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<String>,
    /// Tertiary label (e.g., double-tap for tap-dance) - optional
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tertiary: Option<String>,
}

/// Type of action in a multi-action keycode
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    /// Single key press
    Tap,
    /// Hold action
    Hold,
    /// Double tap action (tap dance)
    DoubleTap,
    /// Layer switch
    Layer,
    /// Modifier
    Modifier,
    /// Simple keycode with no multi-action behavior
    Simple,
}

/// Detailed description of a single action within a keycode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyDetailAction {
    /// Type of action
    pub kind: ActionKind,
    /// Raw keycode or parameter (e.g., "`KC_A`", "1", "`MOD_LCTL`")
    pub code: String,
    /// Human-readable description
    pub description: String,
}

/// Complete key display metadata including short labels and full details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyDisplayMetadata {
    /// Short labels for in-key display
    pub display: KeyDisplay,
    /// Full action breakdown for Key Details panel
    pub details: Vec<KeyDetailAction>,
}

impl KeycodeDb {
    /// Parses a keycode and returns display metadata for rendering.
    ///
    /// This reuses the same parsing logic as the TUI but returns structured
    /// data suitable for the web frontend's Key Details panel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lazyqmk::keycode_db::KeycodeDb;
    /// use std::collections::HashMap;
    ///
    /// let db = KeycodeDb::load().unwrap();
    ///
    /// // Simple key
    /// let meta = db.get_display_metadata("KC_A", None, None);
    /// assert_eq!(meta.display.primary, "A");
    ///
    /// // Layer tap
    /// let meta = db.get_display_metadata("LT(1, KC_ESC)", None, None);
    /// assert_eq!(meta.display.primary, "ESC");
    /// assert_eq!(meta.display.secondary, Some("L1".to_string()));
    /// ```
    #[must_use]
    pub fn get_display_metadata(
        &self,
        keycode: &str,
        tap_dance_info: Option<&TapDanceDisplayInfo>,
        layer_id_to_number: Option<&std::collections::HashMap<String, u8>>,
    ) -> KeyDisplayMetadata {
        // Handle special cases
        if keycode.is_empty() || keycode == "KC_NO" || keycode == "XXXXXXX" {
            return KeyDisplayMetadata {
                display: KeyDisplay {
                    primary: String::new(),
                    secondary: None,
                    tertiary: None,
                },
                details: vec![KeyDetailAction {
                    kind: ActionKind::Simple,
                    code: keycode.to_string(),
                    description: "No action".to_string(),
                }],
            };
        }

        if keycode == "KC_TRNS" || keycode == "_______" {
            return KeyDisplayMetadata {
                display: KeyDisplay {
                    primary: "▽".to_string(),
                    secondary: None,
                    tertiary: None,
                },
                details: vec![KeyDetailAction {
                    kind: ActionKind::Simple,
                    code: keycode.to_string(),
                    description: "Transparent (uses key from lower layer)".to_string(),
                }],
            };
        }

        // Handle Layer Tap: LT(layer, keycode)
        if let Some(inner) = keycode.strip_prefix("LT(") {
            if let Some(args) = inner.strip_suffix(')') {
                let parts: Vec<&str> = args.split(',').map(str::trim).collect();
                if parts.len() == 2 {
                    let layer = resolve_layer_reference(parts[0], layer_id_to_number);
                    let tap_code = parts[1];
                    let tap_label = strip_kc_prefix(tap_code);

                    return KeyDisplayMetadata {
                        display: KeyDisplay {
                            primary: tap_label,
                            secondary: Some(format!("L{layer}")),
                            tertiary: None,
                        },
                        details: vec![
                            KeyDetailAction {
                                kind: ActionKind::Tap,
                                code: tap_code.to_string(),
                                description: format!(
                                    "Tap: {}",
                                    self.get_keycode_description(tap_code)
                                ),
                            },
                            KeyDetailAction {
                                kind: ActionKind::Hold,
                                code: format!("Layer {layer}"),
                                description: format!("Hold: Activate layer {layer}"),
                            },
                        ],
                    };
                }
            }
        }

        // Handle MT(mod, keycode) - Custom Mod Tap
        if let Some(inner) = keycode.strip_prefix("MT(") {
            if let Some(args) = inner.strip_suffix(')') {
                let parts: Vec<&str> = args.split(',').map(str::trim).collect();
                if parts.len() == 2 {
                    let modifier = parts[0];
                    let tap_code = parts[1];
                    let tap_label = strip_kc_prefix(tap_code);
                    let mod_display = format_modifier(modifier);

                    return KeyDisplayMetadata {
                        display: KeyDisplay {
                            primary: tap_label,
                            secondary: Some(mod_display),
                            tertiary: None,
                        },
                        details: vec![
                            KeyDetailAction {
                                kind: ActionKind::Tap,
                                code: tap_code.to_string(),
                                description: format!(
                                    "Tap: {}",
                                    self.get_keycode_description(tap_code)
                                ),
                            },
                            KeyDetailAction {
                                kind: ActionKind::Hold,
                                code: modifier.to_string(),
                                description: format!("Hold: {}", format_modifier_long(modifier)),
                            },
                        ],
                    };
                }
            }
        }

        // Handle named mod-tap: LCTL_T(keycode), LSFT_T(keycode), etc.
        for (prefix, mod_short, mod_long) in MOD_TAP_VARIANTS {
            if let Some(inner) = keycode.strip_prefix(prefix) {
                if let Some(tap_code) = inner.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                    let tap_label = strip_kc_prefix(tap_code);

                    return KeyDisplayMetadata {
                        display: KeyDisplay {
                            primary: tap_label,
                            secondary: Some((*mod_short).to_string()),
                            tertiary: None,
                        },
                        details: vec![
                            KeyDetailAction {
                                kind: ActionKind::Tap,
                                code: tap_code.to_string(),
                                description: format!(
                                    "Tap: {}",
                                    self.get_keycode_description(tap_code)
                                ),
                            },
                            KeyDetailAction {
                                kind: ActionKind::Hold,
                                code: (*prefix).to_string(),
                                description: format!("Hold: {}", mod_long),
                            },
                        ],
                    };
                }
            }
        }

        // Handle LM(layer, mod) - Layer Mod
        if let Some(inner) = keycode.strip_prefix("LM(") {
            if let Some(args) = inner.strip_suffix(')') {
                let parts: Vec<&str> = args.split(',').map(str::trim).collect();
                if parts.len() == 2 {
                    let layer = resolve_layer_reference(parts[0], layer_id_to_number);
                    let modifier = parts[1];
                    let mod_display = format_modifier(modifier);

                    return KeyDisplayMetadata {
                        display: KeyDisplay {
                            primary: format!("L{layer}"),
                            secondary: Some(mod_display),
                            tertiary: None,
                        },
                        details: vec![
                            KeyDetailAction {
                                kind: ActionKind::Layer,
                                code: format!("Layer {layer}"),
                                description: format!("Activate layer {layer} with modifier"),
                            },
                            KeyDetailAction {
                                kind: ActionKind::Modifier,
                                code: modifier.to_string(),
                                description: format!(
                                    "Modifier: {}",
                                    format_modifier_long(modifier)
                                ),
                            },
                        ],
                    };
                }
            }
        }

        // Handle MO (momentary layer)
        if let Some(inner) = keycode.strip_prefix("MO(") {
            if let Some(layer_ref) = inner.strip_suffix(')') {
                let layer = resolve_layer_reference(layer_ref, layer_id_to_number);
                return KeyDisplayMetadata {
                    display: KeyDisplay {
                        primary: format!("▼L{layer}"),
                        secondary: None,
                        tertiary: None,
                    },
                    details: vec![KeyDetailAction {
                        kind: ActionKind::Layer,
                        code: format!("Layer {layer}"),
                        description: format!("Momentary: Activate layer {layer} while held"),
                    }],
                };
            }
        }

        // Handle TG (toggle layer)
        if let Some(inner) = keycode.strip_prefix("TG(") {
            if let Some(layer_ref) = inner.strip_suffix(')') {
                let layer = resolve_layer_reference(layer_ref, layer_id_to_number);
                return KeyDisplayMetadata {
                    display: KeyDisplay {
                        primary: format!("TG{layer}"),
                        secondary: None,
                        tertiary: None,
                    },
                    details: vec![KeyDetailAction {
                        kind: ActionKind::Layer,
                        code: format!("Layer {layer}"),
                        description: format!("Toggle: Switch layer {layer} on/off"),
                    }],
                };
            }
        }

        // Handle TO (switch to layer)
        if let Some(inner) = keycode.strip_prefix("TO(") {
            if let Some(layer_ref) = inner.strip_suffix(')') {
                let layer = resolve_layer_reference(layer_ref, layer_id_to_number);
                return KeyDisplayMetadata {
                    display: KeyDisplay {
                        primary: format!("TO{layer}"),
                        secondary: None,
                        tertiary: None,
                    },
                    details: vec![KeyDetailAction {
                        kind: ActionKind::Layer,
                        code: format!("Layer {layer}"),
                        description: format!("Switch: Go to layer {layer}"),
                    }],
                };
            }
        }

        // Handle OSL (one-shot layer)
        if let Some(inner) = keycode.strip_prefix("OSL(") {
            if let Some(layer_ref) = inner.strip_suffix(')') {
                let layer = resolve_layer_reference(layer_ref, layer_id_to_number);
                return KeyDisplayMetadata {
                    display: KeyDisplay {
                        primary: format!("OS{layer}"),
                        secondary: None,
                        tertiary: None,
                    },
                    details: vec![KeyDetailAction {
                        kind: ActionKind::Layer,
                        code: format!("Layer {layer}"),
                        description: format!("One-shot: Activate layer {layer} for next key"),
                    }],
                };
            }
        }

        // Handle DF (set default layer)
        if let Some(inner) = keycode.strip_prefix("DF(") {
            if let Some(layer_ref) = inner.strip_suffix(')') {
                let layer = resolve_layer_reference(layer_ref, layer_id_to_number);
                return KeyDisplayMetadata {
                    display: KeyDisplay {
                        primary: format!("DF{layer}"),
                        secondary: None,
                        tertiary: None,
                    },
                    details: vec![KeyDetailAction {
                        kind: ActionKind::Layer,
                        code: format!("Layer {layer}"),
                        description: format!("Default: Set layer {layer} as default"),
                    }],
                };
            }
        }

        // Handle TT (layer tap-toggle)
        if let Some(inner) = keycode.strip_prefix("TT(") {
            if let Some(layer_ref) = inner.strip_suffix(')') {
                let layer = resolve_layer_reference(layer_ref, layer_id_to_number);
                return KeyDisplayMetadata {
                    display: KeyDisplay {
                        primary: format!("TT{layer}"),
                        secondary: None,
                        tertiary: None,
                    },
                    details: vec![KeyDetailAction {
                        kind: ActionKind::Layer,
                        code: format!("Layer {layer}"),
                        description: format!(
                            "Tap-Toggle: Tap to toggle layer {layer}, hold to momentarily activate"
                        ),
                    }],
                };
            }
        }

        // Handle PDF (per-layer default layer)
        if let Some(inner) = keycode.strip_prefix("PDF(") {
            if let Some(layer_ref) = inner.strip_suffix(')') {
                let layer = resolve_layer_reference(layer_ref, layer_id_to_number);
                return KeyDisplayMetadata {
                    display: KeyDisplay {
                        primary: format!("PDF{layer}"),
                        secondary: None,
                        tertiary: None,
                    },
                    details: vec![KeyDetailAction {
                        kind: ActionKind::Layer,
                        code: format!("Layer {layer}"),
                        description: format!(
                            "Per-layer Default: Set layer {layer} as per-layer default"
                        ),
                    }],
                };
            }
        }

        // Handle TD (tap dance)
        if let Some(inner) = keycode.strip_prefix("TD(") {
            if let Some(name) = inner.strip_suffix(')') {
                // Use provided tap dance info if available
                if let Some(td_info) = tap_dance_info {
                    let mut details = vec![KeyDetailAction {
                        kind: ActionKind::Tap,
                        code: td_info.single_tap.clone(),
                        description: format!(
                            "Tap: {}",
                            self.get_keycode_description(&td_info.single_tap)
                        ),
                    }];

                    let mut secondary = None;
                    let mut tertiary = None;

                    if let Some(double_tap) = &td_info.double_tap {
                        details.push(KeyDetailAction {
                            kind: ActionKind::DoubleTap,
                            code: double_tap.clone(),
                            description: format!(
                                "Double tap: {}",
                                self.get_keycode_description(double_tap)
                            ),
                        });
                        secondary = Some(strip_kc_prefix(double_tap));
                    }

                    if let Some(hold) = &td_info.hold {
                        details.push(KeyDetailAction {
                            kind: ActionKind::Hold,
                            code: hold.clone(),
                            description: format!("Hold: {}", self.get_keycode_description(hold)),
                        });
                        if secondary.is_none() {
                            secondary = Some(strip_kc_prefix(hold));
                        } else {
                            tertiary = Some(strip_kc_prefix(hold));
                        }
                    }

                    return KeyDisplayMetadata {
                        display: KeyDisplay {
                            primary: strip_kc_prefix(&td_info.single_tap),
                            secondary,
                            tertiary,
                        },
                        details,
                    };
                }

                // Fallback: no tap dance info provided
                return KeyDisplayMetadata {
                    display: KeyDisplay {
                        primary: format!("TD:{name}"),
                        secondary: None,
                        tertiary: None,
                    },
                    details: vec![KeyDetailAction {
                        kind: ActionKind::Tap,
                        code: keycode.to_string(),
                        description: format!("Tap Dance: {name}"),
                    }],
                };
            }
        }

        // Handle one-shot modifiers
        if let Some(inner) = keycode.strip_prefix("OSM(") {
            if let Some(modifier) = inner.strip_suffix(')') {
                let mod_display = format_modifier(modifier);
                return KeyDisplayMetadata {
                    display: KeyDisplay {
                        primary: format!("OS{mod_display}"),
                        secondary: None,
                        tertiary: None,
                    },
                    details: vec![KeyDetailAction {
                        kind: ActionKind::Modifier,
                        code: modifier.to_string(),
                        description: format!(
                            "One-shot: {} for next key",
                            format_modifier_long(modifier)
                        ),
                    }],
                };
            }
        }

        // Handle modifier wrappers: LCTL(kc), LSFT(kc), etc.
        for (prefix, mod_short, mod_long) in MODIFIER_WRAPPERS {
            if let Some(inner) = keycode.strip_prefix(prefix) {
                if let Some(inner_code) = inner.strip_prefix('(').and_then(|s| s.strip_suffix(')'))
                {
                    let inner_label = strip_kc_prefix(inner_code);
                    return KeyDisplayMetadata {
                        display: KeyDisplay {
                            primary: format!("{mod_short}+{inner_label}"),
                            secondary: None,
                            tertiary: None,
                        },
                        details: vec![KeyDetailAction {
                            kind: ActionKind::Simple,
                            code: keycode.to_string(),
                            description: format!(
                                "{} + {}",
                                mod_long,
                                self.get_keycode_description(inner_code)
                            ),
                        }],
                    };
                }
            }
        }

        // Simple keycode: look up in database
        let label = strip_kc_prefix(keycode);
        let description = self.get_keycode_description(keycode);

        KeyDisplayMetadata {
            display: KeyDisplay {
                primary: label,
                secondary: None,
                tertiary: None,
            },
            details: vec![KeyDetailAction {
                kind: ActionKind::Simple,
                code: keycode.to_string(),
                description,
            }],
        }
    }

    /// Gets a human-readable description for a keycode.
    fn get_keycode_description(&self, keycode: &str) -> String {
        if let Some(kc_def) = self.get(keycode) {
            if let Some(desc) = &kc_def.description {
                format!("{} - {}", kc_def.name, desc)
            } else {
                kc_def.name.clone()
            }
        } else {
            strip_kc_prefix(keycode)
        }
    }
}

/// Tap dance display information provided by the layout.
#[derive(Debug, Clone)]
pub struct TapDanceDisplayInfo {
    /// Single tap keycode
    pub single_tap: String,
    /// Optional double tap keycode
    pub double_tap: Option<String>,
    /// Optional hold keycode
    pub hold: Option<String>,
}

/// Resolves a layer reference (which may be a UUID with @ prefix or a numeric string) to a numeric layer number.
///
/// # Examples
/// - "@uuid-123" → looks up in `layer_id_to_number` map, returns "0" if found for layer 0
/// - "1" → returns "1" (already numeric)
/// - "invalid" → returns "invalid" (fallback)
fn resolve_layer_reference(
    layer_ref: &str,
    layer_id_to_number: Option<&std::collections::HashMap<String, u8>>,
) -> String {
    // If it starts with @, it's a UUID reference
    if let Some(uuid) = layer_ref.strip_prefix('@') {
        if let Some(map) = layer_id_to_number {
            if let Some(&number) = map.get(uuid) {
                return number.to_string();
            }
        }
        // If we can't resolve, strip the @ and show the UUID (shouldn't happen in prod)
        return uuid.to_string();
    }
    // Otherwise, it's already numeric or another format
    layer_ref.to_string()
}

/// Strips the "KC_" prefix from a keycode.
fn strip_kc_prefix(keycode: &str) -> String {
    keycode.strip_prefix("KC_").unwrap_or(keycode).to_string()
}

/// Formats a modifier string for compact display.
fn format_modifier(mod_str: &str) -> String {
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
        // Fallback: take first 3 chars
        mod_str.chars().take(3).collect()
    } else {
        result
    }
}

/// Formats a modifier string for full description.
fn format_modifier_long(mod_str: &str) -> String {
    let mut parts = Vec::new();

    if mod_str.contains("LCTL") || mod_str.contains("MOD_LCTL") {
        parts.push("Left Control");
    } else if mod_str.contains("RCTL") || mod_str.contains("MOD_RCTL") {
        parts.push("Right Control");
    } else if mod_str.contains("CTL") || mod_str.contains("MOD_CTL") {
        parts.push("Control");
    }

    if mod_str.contains("LSFT") || mod_str.contains("MOD_LSFT") {
        parts.push("Left Shift");
    } else if mod_str.contains("RSFT") || mod_str.contains("MOD_RSFT") {
        parts.push("Right Shift");
    } else if mod_str.contains("SFT") || mod_str.contains("MOD_SFT") {
        parts.push("Shift");
    }

    if mod_str.contains("LALT") || mod_str.contains("MOD_LALT") {
        parts.push("Left Alt");
    } else if mod_str.contains("RALT") || mod_str.contains("MOD_RALT") {
        parts.push("Right Alt");
    } else if mod_str.contains("ALT") || mod_str.contains("MOD_ALT") {
        parts.push("Alt");
    }

    if mod_str.contains("LGUI") || mod_str.contains("MOD_LGUI") {
        parts.push("Left GUI");
    } else if mod_str.contains("RGUI") || mod_str.contains("MOD_RGUI") {
        parts.push("Right GUI");
    } else if mod_str.contains("GUI") || mod_str.contains("MOD_GUI") {
        parts.push("GUI");
    }

    if mod_str.contains("MEH") {
        return "Meh (Ctrl+Shift+Alt)".to_string();
    }
    if mod_str.contains("HYPR") {
        return "Hyper (Ctrl+Shift+Alt+GUI)".to_string();
    }

    if parts.is_empty() {
        mod_str.to_string()
    } else {
        parts.join(" + ")
    }
}

/// Mod-tap variant prefixes with short and long descriptions.
const MOD_TAP_VARIANTS: &[(&str, &str, &str)] = &[
    ("LCTL_T", "CTL", "Left Control"),
    ("RCTL_T", "CTL", "Right Control"),
    ("CTL_T", "CTL", "Control"),
    ("LSFT_T", "SFT", "Left Shift"),
    ("RSFT_T", "SFT", "Right Shift"),
    ("SFT_T", "SFT", "Shift"),
    ("LALT_T", "ALT", "Left Alt"),
    ("RALT_T", "ALT", "Right Alt"),
    ("ALT_T", "ALT", "Alt"),
    ("LOPT_T", "OPT", "Left Option"),
    ("ROPT_T", "OPT", "Right Option"),
    ("OPT_T", "OPT", "Option"),
    ("LGUI_T", "GUI", "Left GUI"),
    ("RGUI_T", "GUI", "Right GUI"),
    ("GUI_T", "GUI", "GUI"),
    ("LCMD_T", "CMD", "Left Command"),
    ("RCMD_T", "CMD", "Right Command"),
    ("CMD_T", "CMD", "Command"),
    ("LWIN_T", "WIN", "Left Windows"),
    ("RWIN_T", "WIN", "Right Windows"),
    ("WIN_T", "WIN", "Windows"),
    ("MEH_T", "MEH", "Meh (Ctrl+Shift+Alt)"),
    ("HYPR_T", "HYP", "Hyper (Ctrl+Shift+Alt+GUI)"),
    ("ALL_T", "ALL", "Hyper (Ctrl+Shift+Alt+GUI)"),
    ("LSG_T", "S+G", "Left Shift + GUI"),
    ("RSG_T", "S+G", "Right Shift + GUI"),
    ("LCA_T", "C+A", "Left Ctrl + Alt"),
    ("RCA_T", "C+A", "Right Ctrl + Alt"),
    ("LCS_T", "C+S", "Left Ctrl + Shift"),
    ("RCS_T", "C+S", "Right Ctrl + Shift"),
    ("LCAG_T", "CAG", "Ctrl + Alt + GUI"),
    ("RCAG_T", "CAG", "Ctrl + Alt + GUI"),
    ("LSA_T", "S+A", "Left Shift + Alt"),
    ("RSA_T", "S+A", "Right Shift + Alt"),
    ("SAGR_T", "S+A", "Shift + AltGr"),
];

/// Modifier wrapper prefixes with short and long descriptions.
const MODIFIER_WRAPPERS: &[(&str, &str, &str)] = &[
    ("LCTL", "C", "Control"),
    ("RCTL", "C", "Control"),
    ("LSFT", "S", "Shift"),
    ("RSFT", "S", "Shift"),
    ("LALT", "A", "Alt"),
    ("RALT", "A", "Alt"),
    ("LGUI", "G", "GUI"),
    ("RGUI", "G", "GUI"),
    ("LCMD", "⌘", "Command"),
    ("RCMD", "⌘", "Command"),
    ("LCG", "C+G", "Control + GUI"),
    ("RCG", "C+G", "Control + GUI"),
    ("LCA", "C+A", "Control + Alt"),
    ("RCA", "C+A", "Control + Alt"),
    ("LSG", "S+G", "Shift + GUI"),
    ("RSG", "S+G", "Shift + GUI"),
    ("LSA", "S+A", "Shift + Alt"),
    ("RSA", "S+A", "Shift + Alt"),
    ("MEH", "MEH", "Meh (Ctrl+Shift+Alt)"),
    ("HYPR", "HYP", "Hyper (Ctrl+Shift+Alt+GUI)"),
];

#[cfg(test)]
mod tests;
