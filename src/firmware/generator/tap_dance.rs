//! Tap dance code generation for QMK keymap.c.
//!
//! Generates the tap dance enum, finished()/reset() helper functions for
//! 3-way tap dances, the `tap_dance_actions[]` array, and a keycode
//! post-processor that converts `TD(name)` references to `TD(TD_NAME)`.

use super::FirmwareGenerator;

/// Generates tap dance enum definition.
///
/// Creates `enum tap_dance_ids { TD_NAME1, TD_NAME2, ... };`
/// Names are sorted alphabetically for stable ordering.
pub fn generate_enum(gen: &FirmwareGenerator) -> String {
    if gen.layout.tap_dances.is_empty() {
        return String::new();
    }

    let mut code = String::new();
    code.push_str("enum tap_dance_ids {\n");

    // Sort tap dances by name for stable enum ordering
    let mut sorted_tds: Vec<_> = gen.layout.tap_dances.iter().collect();
    sorted_tds.sort_by_key(|td| &td.name);

    for (idx, td) in sorted_tds.iter().enumerate() {
        let enum_name = format!("TD_{}", td.name.to_uppercase());
        code.push_str(&format!("    {}", enum_name));
        if idx < sorted_tds.len() - 1 {
            code.push(',');
        }
        code.push('\n');
    }

    code.push_str("};\n");
    code
}

/// Generates helper functions for 3-way tap dances (with hold behavior).
///
/// For each tap dance that has a hold action, generates finished/reset callback functions.
pub fn generate_helpers(gen: &FirmwareGenerator) -> String {
    let mut code = String::new();

    // Sort tap dances by name for consistent output
    let mut sorted_tds: Vec<_> = gen.layout.tap_dances.iter().collect();
    sorted_tds.sort_by_key(|td| &td.name);

    for td in sorted_tds {
        // Only generate helper functions for 3-way tap dances.
        // A 2-way-hold tap dance (single + hold, no double_tap) is valid
        // by the model but doesn't fit the finished()/reset() helpers
        // which dispatch on count == 2.
        if !td.is_three_way() {
            continue;
        }

        let name_lower = td.name.to_lowercase();
        let single_tap = &td.single_tap;
        // SAFETY: is_three_way() above guarantees both are Some
        let double_tap = td
            .double_tap
            .as_ref()
            .expect("is_three_way implies double_tap is Some");
        let hold = td.hold.as_ref().expect("is_three_way implies hold is Some");

        // Generate finished function
        code.push_str(&format!(
            "void td_{}_finished(tap_dance_state_t *state, void *user_data) {{\n",
            name_lower
        ));
        code.push_str("    if (state->count == 1) {\n");
        code.push_str("        if (state->interrupted || !state->pressed) {\n");
        code.push_str(&format!("            register_code16({});\n", single_tap));
        code.push_str("        } else {\n");
        code.push_str(&format!("            register_code16({});\n", hold));
        code.push_str("        }\n");
        code.push_str("    } else if (state->count == 2) {\n");
        code.push_str(&format!("        register_code16({});\n", double_tap));
        code.push_str("    }\n");
        code.push_str("}\n\n");

        // Generate reset function
        code.push_str(&format!(
            "void td_{}_reset(tap_dance_state_t *state, void *user_data) {{\n",
            name_lower
        ));
        code.push_str("    if (state->count == 1) {\n");
        code.push_str(&format!("        unregister_code16({});\n", single_tap));
        code.push_str(&format!("        unregister_code16({});\n", hold));
        code.push_str("    } else if (state->count == 2) {\n");
        code.push_str(&format!("        unregister_code16({});\n", double_tap));
        code.push_str("    }\n");
        code.push_str("}\n\n");
    }

    code
}

/// Generates tap dance actions array.
///
/// Creates `tap_dance_action_t tap_dance_actions[] = { ... };`
/// Uses `ACTION_TAP_DANCE_DOUBLE` for 2-way, `ACTION_TAP_DANCE_FN_ADVANCED` for 3-way.
pub fn generate_actions(gen: &FirmwareGenerator) -> String {
    if gen.layout.tap_dances.is_empty() {
        return String::new();
    }

    let mut code = String::new();
    code.push_str("tap_dance_action_t tap_dance_actions[] = {\n");

    // Sort tap dances by name to match enum ordering
    let mut sorted_tds: Vec<_> = gen.layout.tap_dances.iter().collect();
    sorted_tds.sort_by_key(|td| &td.name);

    for (idx, td) in sorted_tds.iter().enumerate() {
        let enum_name = format!("TD_{}", td.name.to_uppercase());

        let action = if td.has_hold() {
            // 3-way tap dance: ACTION_TAP_DANCE_FN_ADVANCED(NULL, finished, reset)
            let name_lower = td.name.to_lowercase();
            format!(
                "ACTION_TAP_DANCE_FN_ADVANCED(NULL, td_{}_finished, td_{}_reset)",
                name_lower, name_lower
            )
        } else if let Some(double) = &td.double_tap {
            // 2-way tap dance: ACTION_TAP_DANCE_DOUBLE(single, double)
            let single = &td.single_tap;
            format!("ACTION_TAP_DANCE_DOUBLE({}, {})", single, double)
        } else {
            // Single-tap-only fallback: treat as double with same keycode to avoid helper fns
            let single = &td.single_tap;
            format!("ACTION_TAP_DANCE_DOUBLE({}, {})", single, single)
        };

        code.push_str(&format!("    [{}] = {}", enum_name, action));
        if idx < sorted_tds.len() - 1 {
            code.push(',');
        }
        code.push('\n');
    }

    code.push_str("};\n");
    code
}
