//! Combo code generation for QMK keymap.c.
//!
//! Emits QMK combo arrays and `process_combo_event` handler for two-key hold
//! actions. Combos are base-layer only and require holding both keys for the
//! configured duration.

use anyhow::{anyhow, Result};

use super::FirmwareGenerator;

/// Generates combo code if enabled.
///
/// Emits QMK combo arrays and `process_combo_event` handler for two-key hold actions.
/// Combos are base-layer only and require holding both keys for the configured duration.
#[allow(clippy::unnecessary_wraps)]
pub fn generate(gen: &FirmwareGenerator) -> Result<String> {
    // Collect only real (non-placeholder) combos so that gaps created when
    // non-contiguous combo indices are parsed (e.g. only Combo 2 and Combo 3
    // defined) do not produce phantom COMBO_0 entries in the generated C.
    let real_combos: Vec<_> = gen
        .layout
        .combo_settings
        .combos
        .iter()
        .filter(|c| !c.placeholder)
        .collect();

    // Only generate if combos are enabled and at least one real combo is defined
    if !gen.layout.combo_settings.enabled || real_combos.is_empty() {
        return Ok(String::new());
    }

    let mut code = String::new();

    code.push_str("#ifdef COMBO_ENABLE\n");
    code.push('\n');
    code.push_str("// Combo Configuration\n");
    code.push('\n');

    // Generate combo enum
    code.push_str("enum combo_events {\n");
    for (idx, _combo) in real_combos.iter().enumerate() {
        code.push_str(&format!("    COMBO_{}", idx));
        if idx < real_combos.len() - 1 {
            code.push_str(",\n");
        } else {
            code.push('\n');
        }
    }
    code.push_str("};\n");
    code.push('\n');

    // Generate combo key arrays.
    // combo.key1 / combo.key2 are stored as **visual** positions (set from
    // `state.selected_position` in the TUI and parsed directly from the
    // markdown without any coordinate conversion). Pass them straight to
    // `get_key()` which also operates on visual positions.
    for (idx, combo) in real_combos.iter().enumerate() {
        // Get keycodes for the two positions from base layer (layer 0).
        // A keyboard layout must always have a base layer; if it doesn't,
        // we cannot generate combo code that references it.
        let base_layer = gen.layout.get_layer(0).ok_or_else(|| {
            anyhow!(
                "Cannot generate firmware for layout '{}': missing base layer (layer 0). \
                 A keyboard layout must always contain a base layer.",
                gen.layout.metadata.name
            )
        })?;

        let key1 = base_layer
            .get_key(combo.key1)
            .map(|k| k.keycode.clone())
            .unwrap_or_else(|| "KC_NO".to_string());
        let key2 = base_layer
            .get_key(combo.key2)
            .map(|k| k.keycode.clone())
            .unwrap_or_else(|| "KC_NO".to_string());

        code.push_str(&format!(
            "const uint16_t PROGMEM combo_{}_keys[] = {{{}, {}, COMBO_END}};\n",
            idx, key1, key2
        ));
    }
    code.push('\n');

    // Generate combo array
    code.push_str("combo_t key_combos[] = {\n");
    for (idx, _combo) in real_combos.iter().enumerate() {
        code.push_str(&format!(
            "    [COMBO_{}] = COMBO_ACTION(combo_{}_keys),\n",
            idx, idx
        ));
    }
    code.push_str("};\n");
    code.push('\n');

    // Generate combo state tracking for hold durations
    code.push_str("// Combo hold state tracking\n");
    code.push_str("static struct {\n");
    code.push_str("    uint16_t timer;\n");
    code.push_str("    bool active;\n");
    code.push_str(&format!("}} combo_state[{}];\n", real_combos.len()));
    code.push('\n');

    // Generate process_combo_event handler
    code.push_str("void process_combo_event(uint16_t combo_index, bool pressed) {\n");
    code.push_str("    // Only activate combos on base layer (layer 0)\n");
    code.push_str("    if (get_highest_layer(layer_state) != 0) {\n");
    code.push_str("        return;\n");
    code.push('\n');
    code.push_str("    if (pressed) {\n");
    code.push_str("        // Start hold timer\n");
    code.push_str("        combo_state[combo_index].timer = timer_read();\n");
    code.push_str("        combo_state[combo_index].active = true;\n");
    code.push_str("    } else {\n");
    code.push_str("        // Check if hold duration was met\n");
    code.push_str("        if (combo_state[combo_index].active) {\n");
    code.push_str(
        "            uint16_t elapsed = timer_elapsed(combo_state[combo_index].timer);\n",
    );
    code.push_str("            \n");
    code.push_str("            switch (combo_index) {\n");

    // Generate cases for each combo
    for (idx, combo) in real_combos.iter().enumerate() {
        code.push_str(&format!("                case COMBO_{}:\n", idx));
        code.push_str(&format!(
            "                    if (elapsed >= {}) {{\n",
            combo.hold_duration_ms
        ));

        // Generate action based on combo action type
        match combo.action {
            crate::models::ComboAction::DisableEffects => {
                code.push_str("#ifdef RGB_MATRIX_ENABLE\n");
                code.push_str("                        // Disable RGB effects, revert to TUI layer colors\n");
                if gen.layout_has_custom_colors() {
                    code.push_str("                        rgb_matrix_mode_noeeprom(RGB_MATRIX_TUI_LAYER_COLORS);\n");
                } else {
                    code.push_str("                        rgb_matrix_mode_noeeprom(RGB_MATRIX_SOLID_COLOR);\n");
                }
                code.push_str("#endif\n");
            }
            crate::models::ComboAction::DisableLighting => {
                code.push_str("#ifdef RGB_MATRIX_ENABLE\n");
                code.push_str("                        // Toggle RGB lighting on/off\n");
                code.push_str("                        if (rgb_matrix_is_enabled()) {\n");
                code.push_str("                            rgb_matrix_disable_noeeprom();\n");
                code.push_str("                        } else {\n");
                code.push_str("                            rgb_matrix_enable_noeeprom();\n");
                code.push_str("                        }\n");
                code.push_str("#endif\n");
            }
            crate::models::ComboAction::Bootloader => {
                code.push_str("                        // Enter bootloader mode\n");
                code.push_str("                        reset_keyboard();\n");
            }
        }

        code.push_str("                    }\n");
        code.push_str("                    break;\n");
    }

    code.push_str("            }\n");
    code.push_str("            \n");
    code.push_str("            combo_state[combo_index].active = false;\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n");
    code.push('\n');

    code.push_str("#endif // COMBO_ENABLE\n");

    Ok(code)
}