//! Tap dance documentation generator for layout exports.
//!
//! Generates markdown documentation for all tap dance actions defined in a layout,
//! including single tap, double tap, and hold actions, with a list of keys using each.

use crate::keycode_db::KeycodeDb;
use crate::models::Layout;
use std::fmt::Write;

/// Generates tap dance documentation section for layout exports.
///
/// Produces markdown with:
/// - Each tap dance action with its name and keycodes
/// - Single tap, double tap (if defined), and hold (if defined) actions
/// - List of keys using each tap dance (layer and position)
///
/// Returns empty string if layout has no tap dances defined.
///
/// # Example Output
///
/// ```markdown
/// ## Tap Dance Actions
///
/// ### TD(0): quote_tap_dance
/// - **Single Tap:** KC_QUOT (')
/// - **Double Tap:** KC_DQUO (")
///
/// ### TD(1): bracket_tap_dance
/// - **Single Tap:** KC_LBRC ([)
/// - **Double Tap:** KC_RBRC (])
/// - **Hold:** KC_LSFT (Shift)
///
/// **Keys Using Tap Dance:**
/// - Layer 0, Position (2,3): TD(0)
/// - Layer 1, Position (1,5): TD(1)
/// ```
pub fn generate_tap_dance_docs(layout: &Layout, keycode_db: &KeycodeDb) -> String {
    if layout.tap_dances.is_empty() {
        return String::new();
    }

    let mut output = String::new();

    output.push_str("## Tap Dance Actions\n\n");

    // Generate each tap dance definition
    for (index, tap_dance) in layout.tap_dances.iter().enumerate() {
        let _ = writeln!(output, "### TD({}): {}", index, tap_dance.name);

        // Single tap action
        let single_display = format_keycode(&tap_dance.single_tap, keycode_db);
        let _ = writeln!(output, "- **Single Tap:** {}", single_display);

        // Double tap action (if present)
        if let Some(double_tap) = &tap_dance.double_tap {
            let double_display = format_keycode(double_tap, keycode_db);
            let _ = writeln!(output, "- **Double Tap:** {}", double_display);
        }

        // Hold action (if present)
        if let Some(hold) = &tap_dance.hold {
            let hold_display = format_keycode(hold, keycode_db);
            let _ = writeln!(output, "- **Hold:** {}", hold_display);
        }

        output.push('\n');
    }

    // Generate "Keys Using Tap Dance" section
    let mut key_references = Vec::new();

    // Scan all layers and keys for TD() references
    for layer in &layout.layers {
        for key in &layer.keys {
            // Check if keycode is a tap dance reference
            if key.keycode.starts_with("TD(") && key.keycode.ends_with(')') {
                key_references.push((
                    layer.number,
                    key.position.row,
                    key.position.col,
                    key.keycode.clone(),
                ));
            }
        }
    }

    // Only show "Keys Using Tap Dance" section if there are references
    if !key_references.is_empty() {
        output.push_str("**Keys Using Tap Dance:**\n");

        for (layer_num, row, col, keycode) in key_references {
            let _ = writeln!(
                output,
                "- Layer {}, Position ({},{}): {}",
                layer_num, row, col, keycode
            );
        }
    }

    output
}

/// Formats a keycode with its display name.
///
/// If the keycode is found in the database, returns "CODE (Display Name)".
/// Otherwise, returns just the "CODE".
fn format_keycode(keycode: &str, keycode_db: &KeycodeDb) -> String {
    if let Some(kc_def) = keycode_db.get(keycode) {
        format!("{} ({})", keycode, kc_def.name)
    } else {
        keycode.to_string()
    }
}

#[cfg(test)]
mod tests;

