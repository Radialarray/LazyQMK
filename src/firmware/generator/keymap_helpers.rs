//! Keymap helper functions.
//!
//! These support `generate_keymap_c` with header detection, layer-key layout
//! ordering, color resolution, and the RGB matrix base color table. Each
//! helper is exposed as a free function taking `&FirmwareGenerator`.

use anyhow::{Context, Result};
use std::collections::HashSet;

use super::FirmwareGenerator;

/// Scans all keycodes in the layout and returns the set of additional
/// header files that need to be included for language-specific keycodes.
///
/// This detects keycodes like `DE_Y`, `DE_UDIA` that require language-specific
/// headers from QMK's `keymap_extras` directory.
///
/// Language prefix-to-header mappings are loaded dynamically from the `KeycodeDb`,
/// supporting all languages defined in the keycode database.
pub fn detect_required_headers(gen: &FirmwareGenerator) -> Vec<String> {
    let mut headers = HashSet::new();

    // Build prefix-to-header mapping from language database
    let languages = gen.keycode_db.languages();
    let prefix_headers: Vec<(&str, &str)> = languages
        .iter()
        .map(|lang| (lang.prefix.as_str(), lang.header.as_str()))
        .collect();

    for layer in &gen.layout.layers {
        for key in &layer.keys {
            let keycode = &key.keycode;
            for (prefix, header) in &prefix_headers {
                // Check if keycode starts with prefix (e.g., "DE_Y")
                // or contains it in parameterized form (e.g., "LT(1, DE_Y)")
                if keycode.starts_with(prefix) || keycode.contains(prefix) {
                    headers.insert((*header).to_string());
                    break;
                }
            }
        }
    }

    headers.into_iter().collect()
}

/// Generates keycodes for a layer ordered by layout index.
///
/// This is the critical transformation: visual position → matrix → layout order.
/// The layout order matches the info.json layout array, which is what QMK's
/// LAYOUT macro expects.
///
/// Layer references (e.g., `MO(@uuid)`) are resolved to numeric indices
/// (e.g., `MO(1)`) for firmware compatibility.
pub fn generate_layer_keys_by_layout(
    gen: &FirmwareGenerator,
    layer: &crate::models::layer::Layer,
) -> Result<Vec<String>> {
    let key_count = gen.mapping.key_count();
    let mut keys_by_layout = vec![String::from("KC_NO"); key_count];

    // Map each key to its layout position
    for key in &layer.keys {
        let visual_pos = key.position;

        // Visual → Layout index
        let layout_idx = gen
            .mapping
            .visual_to_layout_index(visual_pos.row, visual_pos.col)
            .with_context(|| {
                format!(
                    "Failed to map visual position ({}, {}) to layout index",
                    visual_pos.row, visual_pos.col
                )
            })?;

        // Resolve layer references in keycode (e.g., MO(@uuid) -> MO(1))
        let resolved_keycode = resolve_keycode(gen, &key.keycode);

        // Process tap dance keycodes (e.g., TD(name) -> TD(TD_NAME))
        let processed_keycode = tap_dance::process_keycode(gen, &resolved_keycode);

        // Store keycode at layout position
        keys_by_layout[layout_idx as usize] = processed_keycode;
    }

    Ok(keys_by_layout)
}

/// Resolves a keycode, converting layer references to numeric indices.
///
/// If the keycode contains a layer reference like `MO(@uuid)`, it will be
/// resolved to the numeric index like `MO(1)`. If resolution fails (e.g.,
/// the referenced layer no longer exists), the original keycode is returned.
pub fn resolve_keycode(gen: &FirmwareGenerator, keycode: &str) -> String {
    // Try to resolve layer references using the keycode database
    if let Some(resolved) = gen.layout.resolve_layer_keycode(keycode, gen.keycode_db) {
        resolved
    } else {
        // Not a layer keycode or resolution failed - use as-is
        keycode.to_string()
    }
}

/// Generates resolved colors for a layer ordered by LED index.
///
/// Colors use the same visual -> LED mapping for LED-based features
/// and honor the layout's four-level color priority system and the
/// `inactive_key_behavior` setting.
///
/// If the layer has `colors_enabled = false`, returns all black (off) colors.
pub fn generate_layer_colors_by_led(
    gen: &FirmwareGenerator,
    layer_idx: usize,
) -> Result<Vec<crate::models::RgbColor>> {
    let led_count = gen.mapping.key_count();
    let mut colors_by_led = vec![crate::models::RgbColor::default(); led_count];

    let layer = gen
        .layout
        .get_layer(layer_idx)
        .with_context(|| format!("Invalid layer index {layer_idx}"))?;

    // If layer colors are disabled for this layer, return all black (LEDs off)
    if !layer.layer_colors_enabled {
        return Ok(vec![crate::models::RgbColor::new(0, 0, 0); led_count]);
    }

    // Map each key's resolved color to its LED position
    // Uses resolve_display_color which considers inactive_key_behavior
    for key in &layer.keys {
        let visual_pos = key.position;

        let led_idx = gen
            .mapping
            .visual_to_led_index(visual_pos.row, visual_pos.col)
            .with_context(|| {
                format!(
                    "Failed to map visual position ({}, {}) to LED index",
                    visual_pos.row, visual_pos.col
                )
            })?;

        // Use resolve_display_color to respect inactive_key_behavior
        let (color, _is_key_specific) = gen.layout.resolve_display_color(layer_idx, key);
        // Apply RGB settings (brightness and master switch)
        let final_color = gen.layout.apply_rgb_settings(color);
        colors_by_led[led_idx as usize] = final_color;
    }

    Ok(colors_by_led)
}

/// Returns true if the layout uses any custom color semantics.
///
/// This treats the layout as "colored" if:
/// - Any layer default color differs from the global default (white), or
/// - Any layer has a category, or
/// - Any key has a color override or category, or
/// - The layout defines any categories at all.
pub fn layout_has_custom_colors(gen: &FirmwareGenerator) -> bool {
    if !gen.layout.categories.is_empty() {
        return true;
    }

    let default_color = crate::models::RgbColor::default();

    for layer in &gen.layout.layers {
        if layer.default_color != default_color {
            return true;
        }

        if layer.category_id.is_some() {
            return true;
        }

        for key in &layer.keys {
            if key.color_override.is_some() || key.category_id.is_some() {
                return true;
            }
        }
    }

    false
}

/// Generates an RGB matrix base color table in C when RGB is present.
///
/// The table layout is:
/// `const uint8_t PROGMEM layer_base_colors[NUM_LAYERS][RGB_MATRIX_LED_COUNT][3]`.
pub fn generate_rgb_matrix_color_table(gen: &FirmwareGenerator) -> Result<String> {
    // If the keyboard has no RGB matrix (no keys), emit an empty string
    // to avoid unused data in non-RGB builds.
    if !gen.geometry.has_rgb_matrix() {
        return Ok(String::new());
    }

    let mut code = String::new();
    let layer_count = gen.layout.layers.len();
    let led_count = gen.mapping.key_count();

    code.push_str("#ifdef RGB_MATRIX_ENABLE\n");
    code.push_str(&format!(
        "const uint8_t PROGMEM layer_base_colors[{layer_count}][{led_count}][3] = {{\n"
    ));

    for layer_idx in 0..layer_count {
        let colors = gen.generate_layer_colors_by_led(layer_idx)?;
        code.push_str("    {\n");

        for (led_idx, color) in colors.iter().enumerate() {
            code.push_str(&format!(
                "        {{{:3}, {:3}, {:3}}}",
                color.r, color.g, color.b
            ));

            if led_idx < colors.len() - 1 {
                code.push(',');
            }

            code.push('\n');
        }

        code.push_str("    }");
        if layer_idx < layer_count - 1 {
            code.push_str(",\n");
        } else {
            code.push('\n');
        }
    }

    code.push_str("};\n");
    code.push_str(&format!(
        "const uint8_t PROGMEM layer_base_colors_layer_count = {layer_count};\n"
    ));
    code.push_str("#endif\n");

    Ok(code)
}

/// Tap dance keycode post-processing helper.
pub mod tap_dance {
    use super::FirmwareGenerator;

    /// Processes a keycode, converting TD(name) references to `TD(TD_NAME_UPPER)`.
    ///
    /// Validates that the referenced tap dance exists in the layout.
    pub fn process_keycode(gen: &FirmwareGenerator, keycode: &str) -> String {
        // Try to parse as tap dance keycode
        if let Some(name) = gen.keycode_db.parse_tap_dance_keycode(keycode) {
            // Validate that this tap dance exists in the layout
            if gen.layout.get_tap_dance(&name).is_some() {
                // Convert TD(name) -> TD(TD_NAME)
                let enum_name = format!("TD_{}", name.to_uppercase());
                return format!("TD({})", enum_name);
            }
            // If tap dance doesn't exist, return original (will be caught by validator)
        }

        // Not a tap dance keycode, return unchanged
        keycode.to_string()
    }
}
