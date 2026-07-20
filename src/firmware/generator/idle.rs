//! Idle effect state machine code generation.
//!
//! Emits C code that manages idle timeout and transitions between ACTIVE,
//! IDLE_EFFECT, and OFF states. Tracks activity via `timer_read/timer_elapsed`
//! and switches RGB effects accordingly. Integrates with ripple overlay when
//! both features are enabled (forward declaration + ripple trigger call inside
//! `process_record_user`).

use anyhow::Result;

use super::FirmwareGenerator;

/// Generates idle effect state machine code if enabled.
///
/// Emits C code to manage idle timeout and transition between ACTIVE, `IDLE_EFFECT`, and OFF states.
/// The code tracks activity using `timer_read/timer_elapsed` and switches RGB effects accordingly.
#[allow(clippy::unnecessary_wraps)]
pub fn generate(gen: &FirmwareGenerator) -> Result<String> {
    // Only generate if idle effect is enabled and keyboard has RGB
    if !gen.layout.idle_effect_settings.enabled || !gen.geometry.has_rgb_matrix() {
        return Ok(String::new());
    }

    // Check if ripple is also enabled (need to add ripple trigger).
    // Mirror the same guard used in generate_ripple_overlay_code so that
    // has_ripple is only true when LQMK_RIPPLE_OVERLAY_ENABLED will actually
    // be #defined (i.e., ripple enabled AND keyboard has RGB matrix).
    // Ripple overlay works independently of PaletteFX — PaletteFX is only
    // used as an idle screensaver, not a replacement for keypress feedback.
    let has_ripple = gen.layout.rgb_overlay_ripple.enabled && gen.geometry.has_rgb_matrix();

    let mut code = String::new();

    // Bootloader combo state (file scope, works regardless of idle/ripple)
    code.push_str("// Bootloader combo: Q+R (left) or U+P (right) held for 1500ms\n");
    code.push_str("static uint32_t bootloader_combo_timer = 0;\n");
    code.push_str("static bool bootloader_combo_active = false;\n");
    code.push('\n');

    code.push_str("#ifdef RGB_MATRIX_ENABLE\n");
    code.push_str("#ifdef LQMK_IDLE_TIMEOUT_MS\n");
    code.push('\n');
    code.push_str("// Idle Effect State Machine\n");
    code.push_str("typedef enum {\n");
    code.push_str("    IDLE_STATE_ACTIVE,\n");
    code.push_str("    IDLE_STATE_IDLE_EFFECT,\n");
    code.push_str("    IDLE_STATE_OFF\n");
    code.push_str("} idle_state_t;\n");
    code.push('\n');
    code.push_str("static idle_state_t idle_state = IDLE_STATE_ACTIVE;\n");
    code.push_str("static uint32_t last_activity_time = 0;\n");
    code.push('\n');

    // Matrix scan hook to check idle timeout
    code.push_str("void matrix_scan_user(void) {\n");
    code.push_str("    uint32_t elapsed = timer_elapsed32(last_activity_time);\n");
    code.push('\n');
    code.push_str("    switch (idle_state) {\n");
    code.push_str("        case IDLE_STATE_ACTIVE:\n");
    code.push_str("            if (elapsed >= LQMK_IDLE_TIMEOUT_MS) {\n");
    code.push_str("                // Transition to idle effect\n");
    code.push_str("                rgb_matrix_mode_noeeprom(LQMK_IDLE_EFFECT_MODE);\n");
    code.push_str("                idle_state = IDLE_STATE_IDLE_EFFECT;\n");
    code.push_str("            }\n");
    code.push_str("            break;\n");
    code.push('\n');
    code.push_str("        case IDLE_STATE_IDLE_EFFECT:\n");
    code.push_str(
        "            if (elapsed >= LQMK_IDLE_TIMEOUT_MS + LQMK_IDLE_EFFECT_DURATION_MS) {\n",
    );
    code.push_str("                // Transition to off\n");
    code.push_str("                rgb_matrix_disable_noeeprom();\n");
    code.push_str("                idle_state = IDLE_STATE_OFF;\n");
    code.push_str("            }\n");
    code.push_str("            break;\n");
    code.push('\n');
    code.push_str("        case IDLE_STATE_OFF:\n");
    code.push_str("            // Stay off until activity\n");
    code.push_str("            break;\n");
    code.push_str("    }\n");
    code.push_str("}\n");
    code.push('\n');

    // Forward declaration for ripple trigger (defined below in LQMK_RIPPLE_OVERLAY_ENABLED block)
    if has_ripple {
        code.push_str("#ifdef LQMK_RIPPLE_OVERLAY_ENABLED\n");
        code.push_str(
            "static bool lazyqmk_ripple_trigger(uint16_t keycode, keyrecord_t *record);\n",
        );
        code.push_str("#endif\n");
        code.push('\n');
    }

    // Process record hook to reset on activity
    code.push_str("bool process_record_user(uint16_t keycode, keyrecord_t *record) {\n");

    // Add ripple trigger integration if both features are enabled
    if has_ripple {
        code.push_str("    bool ripple_triggered = false;\n");
        code.push_str("#ifdef LQMK_RIPPLE_OVERLAY_ENABLED\n");
        code.push_str("    // Trigger ripple effect on matching key events\n");
        code.push_str("    ripple_triggered = lazyqmk_ripple_trigger(keycode, record);\n");
        code.push_str("#endif\n");
        code.push('\n');
    }

    // Bootloader combo detection: Q+R (left) or U+P (right) held for 1500ms
    // Extracts the base keycode from QK_MODS, QK_LAYER_TAP, QK_MOD_TAP wrappers
    // so combos work even when the physical keys are wrapped in LT/MT/MOD macros.
    code.push_str("    // Bootloader combo: Q+R (left) or U+P (right) held for 1500ms\n");
    code.push_str("    {\n");
    code.push_str("        // Resolve the basic keycode (handle LT/MT/MOD wrappers)\n");
    code.push_str("        uint16_t base_kc = keycode;\n");
    code.push_str("        if (IS_QK_MODS(keycode)) {\n");
    code.push_str("            base_kc = QK_MODS_GET_BASIC_KEYCODE(keycode);\n");
    code.push_str("        } else if (IS_QK_LAYER_TAP(keycode)) {\n");
    code.push_str("            base_kc = QK_LAYER_TAP_GET_TAP_KEYCODE(keycode);\n");
    code.push_str("        } else if (IS_QK_MOD_TAP(keycode)) {\n");
    code.push_str("            base_kc = QK_MOD_TAP_GET_TAP_KEYCODE(keycode);\n");
    code.push_str("        }\n");
    code.push_str("        bool is_combo_key = (base_kc == KC_Q || base_kc == KC_R\n");
    code.push_str("                          || base_kc == KC_U || base_kc == KC_P);\n");
    code.push_str("        if (is_combo_key) {\n");
    code.push_str("            static bool q_held = false;\n");
    code.push_str("            static bool r_held = false;\n");
    code.push_str("            static bool u_held = false;\n");
    code.push_str("            static bool p_held = false;\n");
    code.push('\n');
    code.push_str("            if (record->event.pressed) {\n");
    code.push_str("                if (base_kc == KC_Q) q_held = true;\n");
    code.push_str("                if (base_kc == KC_R) r_held = true;\n");
    code.push_str("                if (base_kc == KC_U) u_held = true;\n");
    code.push_str("                if (base_kc == KC_P) p_held = true;\n");
    code.push_str("            } else {\n");
    code.push_str("                if (base_kc == KC_Q) q_held = false;\n");
    code.push_str("                if (base_kc == KC_R) r_held = false;\n");
    code.push_str("                if (base_kc == KC_U) u_held = false;\n");
    code.push_str("                if (base_kc == KC_P) p_held = false;\n");
    code.push_str("            }\n");
    code.push('\n');
    code.push_str("            // Check if either pair is complete\n");
    code.push_str("            bool pair_active = (q_held && r_held) || (u_held && p_held);\n");
    code.push_str("            if (pair_active) {\n");
    code.push_str("                if (!bootloader_combo_active) {\n");
    code.push_str("                    bootloader_combo_timer = timer_read32();\n");
    code.push_str("                    bootloader_combo_active = true;\n");
    code.push_str("                }\n");
    code.push_str("            } else {\n");
    code.push_str("                bootloader_combo_active = false;\n");
    code.push_str("            }\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push('\n');

    code.push_str("    if (record->event.pressed");
    if has_ripple {
        code.push_str(" || ripple_triggered");
    }
    code.push_str(") {\n");
    code.push_str("        // Reset activity timer\n");
    code.push_str("        last_activity_time = timer_read32();\n");
    code.push('\n');

    code.push_str("        if (idle_state != IDLE_STATE_ACTIVE) {\n");
    code.push_str("            // Re-enable RGB if it was disabled\n");
    code.push_str("            if (idle_state == IDLE_STATE_OFF) {\n");
    code.push_str("                rgb_matrix_enable_noeeprom();\n");
    code.push_str("            }\n");
    code.push('\n');

    // Restore the appropriate mode on keypress
    // When PaletteFX is enabled, we still restore TUI layer colors — PaletteFX
    // is only used as idle screensaver, not the default operating mode.
    if gen.layout_has_custom_colors() {
        code.push_str("            // Restore TUI layer colors mode\n");
        code.push_str("            rgb_matrix_mode_noeeprom(RGB_MATRIX_TUI_LAYER_COLORS);\n");
    } else {
        code.push_str("            // Restore default RGB mode\n");
        code.push_str("            rgb_matrix_mode_noeeprom(RGB_MATRIX_DEFAULT_MODE);\n");
    }

    code.push_str("            idle_state = IDLE_STATE_ACTIVE;\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push('\n');
    code.push_str("    return true;\n");
    code.push_str("}\n");
    code.push('\n');

    // Keyboard post init hook to initialize timer
    code.push_str("void keyboard_post_init_user(void) {\n");
    code.push_str("    last_activity_time = timer_read32();\n");
    code.push_str("}\n");
    code.push('\n');

    code.push_str("#endif // LQMK_IDLE_TIMEOUT_MS\n");
    code.push_str("#endif // RGB_MATRIX_ENABLE\n");

    Ok(code)
}
