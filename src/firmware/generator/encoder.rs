//! Conditional encoder map generation for QMK keymap.c.
//!
//! Emits the `#ifdef ENCODER_MAP_ENABLE ... #endif` block listing per-layer
//! encoder bindings. When ENCODER_MAP_ENABLE is not defined (e.g. keyboards
//! without encoders), the entire block is dropped from the firmware.

use anyhow::Result;

use super::FirmwareGenerator;

/// Generates a conditional `encoder_map` wrapped in #ifdef `ENCODER_MAP_ENABLE`.
///
/// This allows the keymap to work both with and without encoders enabled.
/// When `ENCODER_MAP_ENABLE` is defined in rules.mk, this `encoder_map` will be included.
#[allow(clippy::unnecessary_wraps)]
pub fn generate(gen: &FirmwareGenerator) -> Result<String> {
    let mut code = String::new();

    // Get encoder count from keyboard geometry (0 if not specified)
    let encoder_count = gen.geometry.encoder_count as usize;

    code.push_str("#ifdef ENCODER_MAP_ENABLE\n");
    code.push_str("const uint16_t PROGMEM encoder_map[][NUM_ENCODERS][NUM_DIRECTIONS] = {\n");

    // Generate encoder bindings for each layer
    // Default encoder actions cycle through: RGB effect, hue, brightness, saturation
    let default_encoder_bindings = [
        ("RM_NEXT", "RM_PREV"), // Encoder 0: RGB effect
        ("RM_HUEU", "RM_HUED"), // Encoder 1: RGB hue
        ("RM_VALU", "RM_VALD"), // Encoder 2: RGB brightness
        ("RM_SATU", "RM_SATD"), // Encoder 3: RGB saturation
        ("KC_VOLU", "KC_VOLD"), // Encoder 4+: Volume (fallback for extra encoders)
    ];

    for (layer_idx, _layer) in gen.layout.layers.iter().enumerate() {
        code.push_str(&format!("    [{layer_idx}] = {{\n"));

        // Generate encoder bindings based on actual encoder count
        for enc_idx in 0..encoder_count {
            let (ccw, cw) = if enc_idx < default_encoder_bindings.len() {
                default_encoder_bindings[enc_idx]
            } else {
                // Fallback for any additional encoders beyond our defaults
                default_encoder_bindings[default_encoder_bindings.len() - 1]
            };
            code.push_str(&format!("        ENCODER_CCW_CW({ccw}, {cw}),\n"));
        }

        code.push_str("    }");

        if layer_idx < gen.layout.layers.len() - 1 {
            code.push_str(",\n");
        } else {
            code.push_str(",\n");
        }
    }

    code.push_str("};\n");
    code.push_str("#endif\n");

    Ok(code)
}