//! Settings section dispatcher.
//!
//! The `## Settings` section in a layout markdown file contains a series of
//! `**Setting Name**: value` lines. Each setting is parsed by a dedicated
//! helper in a sibling file (one per logical group: general, rgb, tap_hold,
//! idle, ripple, palette_fx, combos).
//!
//! Each helper returns `true` if it recognized and consumed the line; the
//! dispatcher advances `line_num` once per iteration regardless of which (if
//! any) helper handled the line. This preserves the original incremental
//! parsing semantics — no setting depends on the order of arms in the
//! dispatcher, and unrecognized lines are silently ignored (the loop still
//! terminates on the next `##` or `---` boundary).

mod combos;
mod general;
mod idle;
mod palette_fx;
mod rgb;
mod ripple;
mod tap_hold;

use crate::models::{Layout, TapHoldPreset};
use anyhow::Result;

/// Parses the settings section.
#[allow(clippy::cognitive_complexity, clippy::unnecessary_wraps)]
pub(super) fn parse_settings(
    lines: &[&str],
    start_line: usize,
    layout: &mut Layout,
) -> Result<usize> {
    let mut line_num = start_line + 1; // Skip "## Settings" header

    // Track if a preset was explicitly specified in the file
    let mut explicit_preset: Option<TapHoldPreset> = None;

    while line_num < lines.len() {
        let line = lines[line_num].trim();

        // Skip empty lines
        if line.is_empty() {
            line_num += 1;
            continue;
        }

        // Stop at next section
        if line.starts_with("##") || line.starts_with("---") {
            break;
        }

        // Try each setting group in turn. Each helper returns `true` if it
        // recognized the line. Order does not matter for correctness — every
        // setting has a unique key prefix so exactly one helper matches.
        if let Some(preset) = tap_hold::try_parse_tap_hold_preset(line, layout) {
            explicit_preset = Some(preset);
        } else if tap_hold::try_parse_tapping_term(line, layout) {
            // handled
        } else if tap_hold::try_parse_quick_tap_term(line, layout) {
            // handled
        } else if tap_hold::try_parse_hold_mode(line, layout) {
            // handled
        } else if tap_hold::try_parse_retro_tapping(line, layout) {
            // handled
        } else if tap_hold::try_parse_tapping_toggle(line, layout) {
            // handled
        } else if tap_hold::try_parse_flow_tap_term(line, layout) {
            // handled
        } else if tap_hold::try_parse_chordal_hold(line, layout) {
            // handled
        } else if general::try_parse_uncolored_key_behavior(line, layout) {
            // handled
        } else if general::try_parse_rgb_enabled(line, layout) {
            // handled
        } else if general::try_parse_rgb_timeout(line, layout) {
            // handled
        } else if rgb::try_parse_rgb_brightness(line, layout) {
            // handled
        } else if rgb::try_parse_rgb_saturation(line, layout) {
            // handled
        } else if rgb::try_parse_rgb_matrix_speed(line, layout) {
            // handled
        } else if idle::try_parse_idle_effect_enabled(line, layout) {
            // handled
        } else if idle::try_parse_idle_timeout(line, layout) {
            // handled
        } else if idle::try_parse_idle_effect_duration(line, layout) {
            // handled
        } else if idle::try_parse_idle_effect_mode(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_overlay_enabled(line, layout) {
            // handled
        } else if ripple::try_parse_max_ripples(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_duration(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_speed(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_band_width(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_amplitude(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_color_mode(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_fixed_color(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_hue_shift(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_trigger_on_press(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_trigger_on_release(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_ignore_transparent(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_ignore_modifiers(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_ignore_layer_switch(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_wave_count(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_wave_delay(line, layout) {
            // handled
        } else if ripple::try_parse_ripple_key_action_palette(line, layout) {
            // handled
        } else if palette_fx::try_parse_palette_fx_enabled(line, layout) {
            // handled
        } else if palette_fx::try_parse_palette_fx_default_effect(line, layout) {
            // handled
        } else if palette_fx::try_parse_palette_fx_default_palette(line, layout) {
            // handled
        } else if palette_fx::try_parse_palette_fx_all_effects(line, layout) {
            // handled
        } else if palette_fx::try_parse_palette_fx_all_palettes(line, layout) {
            // handled
        } else if combos::try_parse_combos_enabled(line, layout) {
            // handled
        } else if combos::try_parse_combo_definition(line, layout) {
            // handled
        }

        line_num += 1;
    }

    // If an explicit preset was specified, ensure it's preserved
    // (individual setting parsing may have changed it via mark_custom in older code)
    if let Some(preset) = explicit_preset {
        layout.tap_hold_settings.preset = preset;
    }

    Ok(line_num)
}
