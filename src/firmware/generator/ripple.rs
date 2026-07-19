//! RGB overlay ripple code generation.
//!
//! Emits C code that manages ripple effects triggered by keypresses via
//! `rgb_matrix_indicators_advanced_user`, layering on top of TUI layer colors.
//!
//! Ripple overlay works independently of `PaletteFX` — `PaletteFX` is only
//! used as an idle screensaver, not as a keypress effect.

use anyhow::Result;

use super::FirmwareGenerator;

/// Generates RGB overlay ripple code if enabled.
///
/// Emits C code to manage ripple effects triggered by keypresses using
/// `rgb_matrix_indicators_advanced_user` for overlay on top of TUI layer colors.
///
/// NOTE: Ripple overlay works independently of `PaletteFX` — `PaletteFX` is
/// only used as an idle screensaver effect.
#[allow(clippy::too_many_lines)]
pub fn generate(gen: &FirmwareGenerator) -> Result<String> {
    // Only generate if ripple is enabled and keyboard has RGB.
    // NOTE: PaletteFX does NOT replace key-action effects — it is an idle
    // screensaver only. Key-action ripple overlay runs independently.
    if !gen.layout.rgb_overlay_ripple.enabled || !gen.geometry.has_rgb_matrix() {
        return Ok(String::new());
    }

    let settings = &gen.layout.rgb_overlay_ripple;
    settings.validate()?;

    let has_palettefx = gen.layout.palette_fx.enabled;
    let color_mode = settings.color_mode;

    let mut code = String::new();

    code.push_str("#ifdef RGB_MATRIX_ENABLE\n");
    code.push_str("#ifdef LQMK_RIPPLE_OVERLAY_ENABLED\n");
    code.push('\n');
    // The reactive key-action burst is a CUSTOM implementation. It started
    // as a port of PaletteFX's Reactive algorithm (radial bump with
    // amplitude curve) but diverged to fit the overlay use case:
    //
    //   1. EXPANDING WAVEFRONT RING: the radius grows from 0 outward over
    //      the ripple duration, producing a moving wavefront rather than
    //      a static glow.
    //   2. ADDITIVE OVERLAY: instead of replacing LED colors, the burst
    //      is added on top of the TUI base color, preserving the per-key
    //      layer colors. PaletteFX's Reactive REPLACES the LED color
    //      and would clobber the TUI colors.
    //   3. PER-KEY COLOR: each ripple uses the originating key's TUI
    //      color (via lazyqmk_ripple_base_color) so different keys
    //      produce visually distinct bursts. PaletteFX uses one global
    //      palette color.
    //   4. MULTI-RIPPLE: each in-range ripple contributes its own
    //      scaled color to the LED, so pressing multiple keys in
    //      quick succession produces multiple distinct visible bursts.
    //   5. MATRIX-POSITION DISTANCE: distance is computed in matrix
    //      (row, col) space using lazyqmk_led_to_matrix_row/col,
    //      which is more reliable than g_led_config.point (which can
    //      be uninitialised for some LEDs on certain keyboards).
    //
    // The mathematical shape (radial falloff with amplitude envelope)
    // is similar to PaletteFX Reactive, but everything else is custom.
    //
    // PaletteFX is still used by LazyQMK for the IDLE SCREENSAVER
    // (see generate_idle_effect_code), not for this key-action burst.
    code.push_str("// Reactive key-action overlay (custom implementation)\n");
    code.push_str("// Expanding wavefront ring, additive on TUI colors, per-key color.\n");
    code.push('\n');
    code.push_str("// Forward declaration from QMK (used for per-key layer resolution)\n");
    code.push_str("uint8_t layer_switch_get_layer(keypos_t key);\n");
    code.push('\n');

    // Ripple state structure
    code.push_str("typedef struct {\n");
    code.push_str("    uint8_t led_index;\n");
    code.push_str("    uint8_t row;\n");
    code.push_str("    uint8_t col;\n");
    code.push_str("    uint32_t start_time;\n");
    code.push_str("    uint32_t trigger_delay_ms;\n");
    code.push_str("    bool active;\n");
    code.push_str("} ripple_t;\n");
    code.push('\n');

    // Ripple state array
    code.push_str("static ripple_t ripples[LQMK_RIPPLE_MAX_RIPPLES] = {0};\n");
    code.push('\n');

    // Helper: Add new ripple (find empty slot or replace oldest)
    // delay_ms: stagger offset for multi-wave cascading (0 = immediate)
    code.push_str(
        "static void lazyqmk_ripple_add(uint8_t led_index, uint8_t row, uint8_t col, uint32_t delay_ms) {\n",
    );
    code.push_str("    // Find an empty slot or the oldest ripple\n");
    code.push_str("    uint8_t oldest_idx = 0;\n");
    code.push_str("    uint32_t oldest_time = ripples[0].start_time;\n");
    code.push('\n');
    code.push_str("    for (uint8_t i = 0; i < LQMK_RIPPLE_MAX_RIPPLES; i++) {\n");
    code.push_str("        if (!ripples[i].active) {\n");
    code.push_str("            ripples[i].led_index = led_index;\n");
    code.push_str("            ripples[i].row = row;\n");
    code.push_str("            ripples[i].col = col;\n");
    code.push_str("            ripples[i].start_time = timer_read32();\n");
    code.push_str("            ripples[i].trigger_delay_ms = delay_ms;\n");
    code.push_str("            ripples[i].active = true;\n");
    code.push_str("            return;\n");
    code.push_str("        }\n");
    code.push_str("        if (ripples[i].start_time < oldest_time) {\n");
    code.push_str("            oldest_time = ripples[i].start_time;\n");
    code.push_str("            oldest_idx = i;\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push('\n');
    code.push_str("    // Replace oldest if no empty slot\n");
    code.push_str("    ripples[oldest_idx].led_index = led_index;\n");
    code.push_str("    ripples[oldest_idx].row = row;\n");
    code.push_str("    ripples[oldest_idx].col = col;\n");
    code.push_str("    ripples[oldest_idx].start_time = timer_read32();\n");
    code.push_str("    ripples[oldest_idx].trigger_delay_ms = delay_ms;\n");
    code.push_str("    ripples[oldest_idx].active = true;\n");
    code.push_str("}\n");
    code.push('\n');

    // Helper: Map matrix position to LED index
    code.push_str("static uint8_t lazyqmk_matrix_to_led(uint8_t row, uint8_t col) {\n");
    code.push_str("    for (uint8_t i = 0; i < RGB_MATRIX_LED_COUNT; i++) {\n");
    code.push_str("        if (g_led_config.matrix_co[row][col] == i) {\n");
    code.push_str("            return i;\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("    // Fallback to center LED if mapping not found\n");
    code.push_str("    return RGB_MATRIX_LED_COUNT / 2;\n");
    code.push_str("}\n");
    code.push('\n');

    // LED->matrix position lookup for per-key layer resolution
    let led_to_matrix = &gen.mapping.led_to_matrix;
    let led_count = gen.mapping.key_count();
    let mut led_row_vals = String::new();
    let mut led_col_vals = String::new();
    for (i, &(row, col)) in led_to_matrix.iter().enumerate() {
        if i > 0 {
            led_row_vals.push_str(", ");
            led_col_vals.push_str(", ");
        }
        led_row_vals.push_str(&format!("{}", row));
        led_col_vals.push_str(&format!("{}", col));
    }
    code.push_str("// LED index -> matrix position mapping (for per-key layer resolution)\n");
    code.push_str(&format!(
        "const uint8_t PROGMEM lazyqmk_led_to_matrix_row[{led_count}] = {{ {led_row_vals} }};\n"
    ));
    code.push_str(&format!(
        "const uint8_t PROGMEM lazyqmk_led_to_matrix_col[{led_count}] = {{ {led_col_vals} }};\n"
    ));
    code.push('\n');

    // Base color lookup (per-key layer colors or fallback)
    // Always generated: used by the originating-key ripple path when
    // background lighting is on, and by the non-palettefx color modes.
    code.push_str("static RGB lazyqmk_ripple_base_color(uint8_t led_index) {\n");
    code.push_str("    RGB color = {0, 0, 0};\n");
    code.push('\n');
    code.push_str("#ifdef LAYER_BASE_COLORS_LAYER_COUNT\n");
    code.push_str("    uint8_t row = pgm_read_byte(&lazyqmk_led_to_matrix_row[led_index]);\n");
    code.push_str("    uint8_t col = pgm_read_byte(&lazyqmk_led_to_matrix_col[led_index]);\n");
    code.push_str("    keypos_t key = { .row = row, .col = col };\n");
    code.push_str("    uint8_t layer = layer_switch_get_layer(key);\n");
    code.push_str("    if (layer < layer_base_colors_layer_count) {\n");
    code.push_str(
        "        color.r = pgm_read_byte(&layer_base_colors[layer][led_index][0]);\n",
    );
    code.push_str(
        "        color.g = pgm_read_byte(&layer_base_colors[layer][led_index][1]);\n",
    );
    code.push_str(
        "        color.b = pgm_read_byte(&layer_base_colors[layer][led_index][2]);\n",
    );
    code.push_str("        return color;\n");
    code.push_str("    }\n");
    code.push_str("#endif\n");
    code.push('\n');
    code.push_str("    rgb_t matrix_rgb = hsv_to_rgb(rgb_matrix_get_hsv());\n");
    code.push_str("    color.r = matrix_rgb.r;\n");
    code.push_str("    color.g = matrix_rgb.g;\n");
    code.push_str("    color.b = matrix_rgb.b;\n");
    code.push_str("    return color;\n");
    code.push_str("}\n");
    code.push('\n');

    // Reactive apply: compute the burst contribution for a single LED.
    // Uses an EXPANDING RING algorithm: the wavefront moves outward
    // from the pressed key over time. Each active ripple contributes
    // its own color to LEDs near its current wavefront, so multiple
    // simultaneous ripples produce multiple distinct bursts.
    code.push_str("static void lazyqmk_reactive_apply(uint8_t led_index) {\n");
    code.push_str("    uint32_t now = timer_read32();\n");
    code.push_str("    // Accumulate color contribution from each ripple in range.\n");
    code.push_str("    uint8_t contrib_r = 0, contrib_g = 0, contrib_b = 0;\n");
    code.push('\n');
    code.push_str("    for (uint8_t i = 0; i < LQMK_RIPPLE_MAX_RIPPLES; i++) {\n");
    code.push_str("        if (!ripples[i].active) continue;\n");
    code.push('\n');
    code.push_str("        uint32_t elapsed_ms = now - ripples[i].start_time;\n");
    code.push_str("        // Respect trigger delay for multi-wave cascading\n");
    code.push_str("        if (elapsed_ms < ripples[i].trigger_delay_ms) continue;\n");
    code.push_str(
        "        uint32_t effective_elapsed = elapsed_ms - ripples[i].trigger_delay_ms;\n",
    );
    code.push_str("        if (effective_elapsed >= LQMK_RIPPLE_DURATION_MS) {\n");
    code.push_str("            ripples[i].active = false;\n");
    code.push_str("            continue;\n");
    code.push_str("        }\n");
    code.push('\n');
    // Compute the current ring radius (in matrix-space distance).
    // The wavefront expands from 0 to LQMK_RIPPLE_MAX_RADIUS over the
    // duration. At duration/2, the ring is at max radius and at peak
    // intensity. After that, it fades and the ring continues outward.
    // Speed-aware expansion: radius grows from 0 to LQMK_RIPPLE_MAX_RADIUS
    // over LQMK_RIPPLE_DURATION_MS. LQMK_RIPPLE_SPEED accelerates (higher = faster).
    // At default speed=200 this is identity: (elapsed * max * 200) / (dur * 200).
    code.push_str("        uint32_t scaled = (uint32_t)effective_elapsed * LQMK_RIPPLE_MAX_RADIUS * LQMK_RIPPLE_SPEED;\n");
    code.push_str(
        "        uint8_t radius = (uint8_t)(scaled / LQMK_RIPPLE_SCALED_DURATION);\n",
    );
    code.push_str("        // Intensity envelope: triangle ramp 0→255→0 over duration\n");
    code.push_str("        uint8_t amp = (uint8_t)scale16by8(\n");
    code.push_str("            effective_elapsed < (LQMK_RIPPLE_DURATION_MS / 2)\n");
    code.push_str("                ? (uint16_t)(effective_elapsed * 2)\n");
    code.push_str(
        "                : (uint16_t)((LQMK_RIPPLE_DURATION_MS - effective_elapsed) * 2),\n",
    );
    code.push_str("            LQMK_RIPPLE_AMP_SCALE);\n");
    code.push('\n');
    code.push_str("        // Compute distance using MATRIX positions\n");
    code.push_str(
        "        uint8_t led_row = pgm_read_byte(&lazyqmk_led_to_matrix_row[led_index]);\n",
    );
    code.push_str(
        "        uint8_t led_col = pgm_read_byte(&lazyqmk_led_to_matrix_col[led_index]);\n",
    );
    code.push_str("        int8_t drow = (int8_t)led_row - (int8_t)ripples[i].row;\n");
    code.push_str("        int8_t dcol = (int8_t)led_col - (int8_t)ripples[i].col;\n");
    code.push_str("        if (drow < 0) drow = -drow;\n");
    code.push_str("        if (dcol < 0) dcol = -dcol;\n");
    code.push('\n');
    code.push_str("        // Skip if outside the maximum ring area\n");
    code.push_str("        if (drow > LQMK_RIPPLE_MAX_RADIUS + 1 || dcol > LQMK_RIPPLE_MAX_RADIUS + 1) continue;\n");
    code.push_str("        uint8_t dist = (uint8_t)(drow + dcol);\n");
    code.push('\n');
    // Soft pulse: brightness peaks at the wavefront, fades smoothly
    // both inward (toward center) and outward (toward edge).
    // FADE_WIDTH controls the gradient width — wider = smoother, narrower = sharper.
    code.push_str("        uint8_t bump;\n");
    code.push_str("        if (dist <= radius) {\n");
    code.push_str("            // Inside the wavefront: gradient fading toward center\n");
    code.push_str("            uint8_t inner_dist = radius - dist;\n");
    code.push_str("            if (inner_dist > LQMK_RIPPLE_FADE_WIDTH) continue;\n");
    code.push_str("            bump = scale8((uint8_t)(255 - 255 * inner_dist / LQMK_RIPPLE_FADE_WIDTH), amp);\n");
    code.push_str("        } else {\n");
    code.push_str("            // Outside the wavefront: gradient fading outward\n");
    code.push_str("            uint8_t outer_dist = dist - radius;\n");
    code.push_str("            if (outer_dist > LQMK_RIPPLE_FADE_WIDTH) continue;\n");
    code.push_str("            bump = scale8((uint8_t)(255 - 255 * outer_dist / LQMK_RIPPLE_FADE_WIDTH), amp);\n");
    code.push_str("        }\n");
    code.push_str("        if (bump == 0) continue;\n");
    code.push('\n');
    // For each ripple in range, accumulate ITS color contribution.
    code.push_str("        {\n");
    code.push_str("            RGB c = lazyqmk_ripple_base_color(ripples[i].led_index);\n");
    code.push_str("            contrib_r = qadd8(contrib_r, scale8(c.r, bump));\n");
    code.push_str("            contrib_g = qadd8(contrib_g, scale8(c.g, bump));\n");
    code.push_str("            contrib_b = qadd8(contrib_b, scale8(c.b, bump));\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push('\n');
    code.push_str("    // Nothing to render\n");
    code.push_str("    if (contrib_r == 0 && contrib_g == 0 && contrib_b == 0) return;\n");
    code.push('\n');

    // Color application: two paths depending on whether TUI background
    // lighting is configured. If background lighting is on, the per-LED
    // contribution (sum of all in-range ripples' colors scaled by their
    // local bump) is ADDED on top of the TUI color. If no background
    // lighting, the per-LED contribution REPLACES the LED color with
    // the PaletteFX palette (or configured color mode).
    let has_background_lighting = gen.layout_has_custom_colors();
    if has_background_lighting {
        // Background lighting is on: LAYER the ripple on top of the
        // TUI color. The contribution is the sum of in-range ripples'
        // colors, each scaled by its own local bump.
        code.push_str("    // Read the TUI base color for the current LED\n");
        code.push_str("    RGB base = lazyqmk_ripple_base_color(led_index);\n");
        code.push_str("    // Additive contribution (multi-ripple aware)\n");
        code.push_str("    base.r = qadd8(base.r, contrib_r);\n");
        code.push_str("    base.g = qadd8(base.g, contrib_g);\n");
        code.push_str("    base.b = qadd8(base.b, contrib_b);\n");
        code.push_str("    rgb_matrix_set_color(led_index, base.r, base.g, base.b);\n");
    } else if has_palettefx {
        // No background lighting: use PaletteFX palette colors at full intensity
        code.push_str("    // Use PaletteFX palette lookup for rich gradient colors\n");
        if let Some(ref palette) = settings.key_action_palette {
            let index = *palette as u8;
            code.push_str(&format!(
                "    const uint16_t* palette = palettefx_get_palette_data_by_index({index});  // {name}\n",
                name = palette.display_name()
            ));
        } else {
            code.push_str(
                "    const uint16_t* palette = palettefx_get_palette_data();  // current palette\n",
            );
        }
        code.push_str("    uint8_t brightness = rgb_matrix_get_val();\n");
        code.push_str("    hsv_t hsv = palettefx_interp_color(palette, brightness);\n");
        code.push_str("    if (brightness < 32) {\n");
        code.push_str("        hsv.v = scale8(hsv.v, (uint8_t)(64 + 6 * brightness));\n");
        code.push_str("    }\n");
        code.push_str("    rgb_t contrib = hsv_to_rgb(hsv);\n");
        code.push_str(
            "    rgb_matrix_set_color(led_index, contrib.r, contrib.g, contrib.b);\n",
        );
    } else {
        // No PaletteFX and no background lighting: use configured color mode
        code.push_str("    RGB base = lazyqmk_ripple_base_color(led_index);\n");
        code.push_str("    uint8_t brightness = rgb_matrix_get_val();\n");
        code.push_str("    uint8_t contrib_r, contrib_g, contrib_b;\n");
        code.push('\n');

        match color_mode {
            crate::models::layout::RippleColorMode::Fixed => {
                let color = &settings.fixed_color;
                code.push_str("    // Fixed color mode: use the configured fixed color\n");
                code.push_str(&format!(
                    "    contrib_r = scale8({}, brightness);\n",
                    color.r
                ));
                code.push_str(&format!(
                    "    contrib_g = scale8({}, brightness);\n",
                    color.g
                ));
                code.push_str(&format!(
                    "    contrib_b = scale8({}, brightness);\n",
                    color.b
                ));
            }
            crate::models::layout::RippleColorMode::KeyBased => {
                code.push_str(
                    "    // Key color mode: use the trigger key's resolved base color\n",
                );
                code.push_str("    {\n");
                code.push_str("        RGB key_color = {0, 0, 0};\n");
                code.push_str(
                    "        for (uint8_t i = 0; i < LQMK_RIPPLE_MAX_RIPPLES; i++) {\n",
                );
                code.push_str("            if (ripples[i].active) {\n");
                code.push_str("                key_color = lazyqmk_ripple_base_color(ripples[i].led_index);\n");
                code.push_str("            }\n");
                code.push_str("        }\n");
                code.push_str("        contrib_r = scale8(key_color.r, brightness);\n");
                code.push_str("        contrib_g = scale8(key_color.g, brightness);\n");
                code.push_str("        contrib_b = scale8(key_color.b, brightness);\n");
                code.push_str("    }\n");
            }
            crate::models::layout::RippleColorMode::HueShift => {
                let hue_shift_steps = (i32::from(settings.hue_shift_deg) * 256) / 360;
                code.push_str(
                    "    // Hue shift mode: shift the base color's hue by configured degrees\n",
                );
                code.push_str("    {\n");
                code.push_str("        hsv_t hsv = rgb_to_hsv(base);\n");
                code.push_str(&format!(
                    "        int16_t shifted_hue = (int16_t)hsv.h + {};\n",
                    hue_shift_steps
                ));
                code.push_str("        while (shifted_hue < 0) shifted_hue += 256;\n");
                code.push_str("        while (shifted_hue >= 256) shifted_hue -= 256;\n");
                code.push_str("        hsv.h = (uint8_t)shifted_hue;\n");
                code.push_str("        rgb_t shifted = hsv_to_rgb(hsv);\n");
                code.push_str("        contrib_r = scale8(shifted.r, brightness);\n");
                code.push_str("        contrib_g = scale8(shifted.g, brightness);\n");
                code.push_str("        contrib_b = scale8(shifted.b, brightness);\n");
                code.push_str("    }\n");
            }
        }

        code.push_str(
            "    rgb_matrix_set_color(led_index, contrib_r, contrib_g, contrib_b);\n",
        );
    }
    code.push_str("}\n");
    code.push('\n');

    // Helper: Trigger ripple on keypress
    code.push_str(
        "static bool lazyqmk_ripple_trigger(uint16_t keycode, keyrecord_t *record) {\n",
    );

    // Add filters
    if settings.ignore_transparent {
        code.push_str("    if (keycode == KC_TRNS) return false;\n");
    }
    if settings.ignore_modifiers {
        code.push_str("    if (IS_MODIFIER_KEYCODE(keycode)) return false;\n");
    }
    if settings.ignore_layer_switch {
        code.push_str("    if (IS_LAYER_SWITCH_KEYCODE(keycode)) return false;\n");
    }

    code.push_str("    // Get LED index from matrix position\n");
    code.push_str("    uint8_t led_index = lazyqmk_matrix_to_led(record->event.key.row, record->event.key.col);\n");
    code.push('\n');

    // Check trigger conditions
    let needs_press_check = settings.trigger_on_press || settings.trigger_on_release;
    if needs_press_check {
        code.push_str("    bool should_trigger = false;\n");
        if settings.trigger_on_press {
            code.push_str("    if (record->event.pressed && LQMK_RIPPLE_TRIGGER_ON_PRESS) should_trigger = true;\n");
        }
        if settings.trigger_on_release {
            code.push_str("    if (!record->event.pressed && LQMK_RIPPLE_TRIGGER_ON_RELEASE) should_trigger = true;\n");
        }
        code.push_str("    if (should_trigger) {\n");
        code.push_str("        // Spawn concentric waves with staggered delays\n");
        code.push_str("        for (uint8_t w = 0; w < LQMK_RIPPLE_WAVE_COUNT; w++) {\n");
        code.push_str("            lazyqmk_ripple_add(led_index, record->event.key.row, record->event.key.col,\n");
        code.push_str(
            "                               (uint32_t)w * LQMK_RIPPLE_WAVE_DELAY_MS);\n",
        );
        code.push_str("        }\n");
        code.push_str("        return true;\n");
        code.push_str("    }\n");
    }

    code.push_str("    return false;\n");
    code.push_str("}\n");
    code.push('\n');

    // RGB Matrix advanced indicators hook — applies the reactive overlay
    // on top of whatever the current RGB matrix effect rendered.
    // NOTE: intentionally NOT weak — must override QMK's weak default.
    code.push_str(
        "bool rgb_matrix_indicators_advanced_user(uint8_t led_min, uint8_t led_max) {\n",
    );
    code.push_str("    for (uint8_t i = led_min; i < led_max; i++) {\n");
    code.push_str("        lazyqmk_reactive_apply(i);\n");
    code.push_str("    }\n");
    code.push_str("    return false;\n");
    code.push_str("}\n");
    code.push('\n');

    // Process record hook integration
    // Check if idle effect is enabled to determine if we need to wrap or create new hook
    let has_idle_effect = gen.layout.idle_effect_settings.enabled;

    if !has_idle_effect {
        // No idle effect, generate standalone process_record_user for ripple
        code.push_str("bool process_record_user(uint16_t keycode, keyrecord_t *record) {\n");
        code.push_str("    lazyqmk_ripple_trigger(keycode, record);\n");
        code.push_str("    return true;\n");
        code.push_str("}\n");
        code.push('\n');
    }
    // If idle effect is enabled, the ripple trigger is already integrated in idle effect code

    code.push_str("#endif // LQMK_RIPPLE_OVERLAY_ENABLED\n");
    code.push_str("#endif // RGB_MATRIX_ENABLE\n");

    Ok(code)
}