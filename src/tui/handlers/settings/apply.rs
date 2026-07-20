//! Apply settings from the manager state to the app state.

use anyhow::Result;

use crate::models::{
    HoldDecisionMode, PaletteFxEffect, PaletteFxPalette, RgbBrightness, RgbMatrixEffect,
    RgbSaturation, RippleColorMode, TapHoldPreset, UncoloredKeyBehavior,
};
use crate::tui::settings_manager::SettingItem;
use crate::tui::{ActiveComponent, AppState};

/// Apply settings from the manager state to the app state
pub(super) fn apply_settings(state: &mut AppState) -> Result<()> {
    // Get the manager's state to extract current values
    if let Some(ActiveComponent::SettingsManager(ref manager)) = state.active_component {
        let manager_state = manager.state();

        // Extract the setting being edited based on mode
        match &manager_state.mode {
            crate::tui::settings_manager::ManagerMode::SelectingTapHoldPreset { .. } => {
                if let Some(selected_idx) = manager_state.get_selected_option() {
                    if let Some(&preset) = TapHoldPreset::all().get(selected_idx) {
                        state.layout.tap_hold_settings.apply_preset(preset);
                        state.mark_dirty();
                        state.set_status(format!(
                            "Tap-hold preset set to: {}",
                            preset.display_name()
                        ));
                    }
                }
            }
            crate::tui::settings_manager::ManagerMode::SelectingHoldMode { .. } => {
                if let Some(selected_idx) = manager_state.get_selected_option() {
                    if let Some(&mode) = HoldDecisionMode::all().get(selected_idx) {
                        state.layout.tap_hold_settings.hold_mode = mode;
                        state.layout.tap_hold_settings.mark_custom();
                        state.mark_dirty();
                        state.set_status(format!("Hold mode set to: {}", mode.display_name()));
                    }
                }
            }
            crate::tui::settings_manager::ManagerMode::EditingNumeric { setting, .. } => {
                if let Some(value) = manager_state.get_numeric_value() {
                    apply_numeric_setting(state, *setting, value);
                    state.mark_dirty();
                }
            }
            crate::tui::settings_manager::ManagerMode::EditingSignedNumeric { setting, .. } => {
                if let Some(value) = manager_state.get_signed_numeric_value() {
                    apply_signed_numeric_setting(state, *setting, value);
                    state.mark_dirty();
                }
            }
            crate::tui::settings_manager::ManagerMode::TogglingBoolean { setting, .. } => {
                if let Some(value) = manager_state.get_boolean_value() {
                    apply_boolean_setting(state, *setting, value);
                    state.mark_dirty();
                }
            }
            crate::tui::settings_manager::ManagerMode::EditingString { setting, .. } => {
                if let Some(value) = manager_state.get_string_value() {
                    apply_string_setting(state, *setting, value.to_string())?;
                }
            }
            crate::tui::settings_manager::ManagerMode::SelectingOutputFormat { .. } => {
                if let Some(selected_idx) = manager_state.get_output_format_selected() {
                    let format = match selected_idx {
                        0 => "uf2",
                        1 => "hex",
                        2 => "bin",
                        _ => "uf2",
                    };
                    state.layout.metadata.output_format = Some(format.to_string());
                    state.layout.metadata.touch();
                    state.set_status(format!("Output format set to: {format}"));
                }
            }
            crate::tui::settings_manager::ManagerMode::SelectingThemeMode { .. } => {
                if let Some(selected_idx) = manager_state.get_selected_option() {
                    let theme_mode = match selected_idx {
                        1 => crate::config::ThemeMode::Dark,
                        2 => crate::config::ThemeMode::Light,
                        _ => crate::config::ThemeMode::Auto,
                    };
                    state.config.ui.theme_mode = theme_mode;
                    if let Err(e) = state.config.save() {
                        state.set_status(format!("Failed to save config: {e}"));
                    } else {
                        state.set_status(format!(
                            "Theme mode set to: {}",
                            theme_mode_display(theme_mode)
                        ));
                    }
                }
            }
            crate::tui::settings_manager::ManagerMode::EditingPath { setting, .. } => {
                if let Some(value) = manager_state.get_string_value() {
                    apply_path_setting(state, *setting, value.to_string())?;
                }
            }
            crate::tui::settings_manager::ManagerMode::SelectingIdleEffectMode { .. } => {
                if let Some(selected_idx) = manager_state.get_selected_option() {
                    if let Some(&mode) = RgbMatrixEffect::all().get(selected_idx) {
                        state.layout.idle_effect_settings.idle_effect_mode = mode;
                        state.mark_dirty();
                        state.set_status(format!(
                            "Idle effect mode set to: {}",
                            mode.display_name()
                        ));
                    }
                }
            }
            crate::tui::settings_manager::ManagerMode::SelectingRippleColorMode { .. } => {
                if let Some(selected_idx) = manager_state.get_selected_option() {
                    if let Some(&mode) = RippleColorMode::all().get(selected_idx) {
                        state.layout.rgb_overlay_ripple.color_mode = mode;
                        state.mark_dirty();
                        state.set_status(format!(
                            "Ripple color mode set to: {}",
                            mode.display_name()
                        ));
                    }
                }
            }
            crate::tui::settings_manager::ManagerMode::SelectingPaletteFxEffect { .. } => {
                if let Some(selected_idx) = manager_state.get_selected_option() {
                    if let Some(&effect) = PaletteFxEffect::all().get(selected_idx) {
                        state.layout.palette_fx.default_effect = effect;
                        state.mark_dirty();
                        state.set_status(format!(
                            "PaletteFX effect set to: {}",
                            effect.display_name()
                        ));
                    }
                }
            }
            crate::tui::settings_manager::ManagerMode::SelectingPaletteFxPalette { .. } => {
                if let Some(selected_idx) = manager_state.get_selected_option() {
                    if let Some(&palette) = PaletteFxPalette::all().get(selected_idx) {
                        state.layout.palette_fx.default_palette = palette;
                        state.mark_dirty();
                        state.set_status(format!(
                            "PaletteFX palette set to: {}",
                            palette.display_name()
                        ));
                    }
                }
            }
            crate::tui::settings_manager::ManagerMode::SelectingKeyActionPalette { .. } => {
                if let Some(selected_idx) = manager_state.get_selected_option() {
                    if selected_idx == 0 {
                        // "Default" - use current active palette
                        state.layout.rgb_overlay_ripple.key_action_palette = None;
                        state.mark_dirty();
                        state
                            .set_status("Key action palette set to: Default (current)".to_string());
                    } else if let Some(&palette) = PaletteFxPalette::all().get(selected_idx - 1) {
                        state.layout.rgb_overlay_ripple.key_action_palette = Some(palette);
                        state.mark_dirty();
                        state.set_status(format!(
                            "Key action palette set to: {}",
                            palette.display_name()
                        ));
                    }
                }
            }
            crate::tui::settings_manager::ManagerMode::SelectingKeyPosition { setting, .. } => {
                // Key position was selected - apply it to the appropriate combo
                apply_combo_key_position(state, *setting);
            }
            crate::tui::settings_manager::ManagerMode::Browsing => {}
        }
    }
    Ok(())
}

/// Apply a numeric setting value
fn apply_numeric_setting(state: &mut AppState, setting: SettingItem, value: u16) {
    match setting {
        SettingItem::TappingTerm => {
            state.layout.tap_hold_settings.tapping_term = value;
            state.layout.tap_hold_settings.mark_custom();
            state.set_status(format!("Tapping term set to: {value}ms"));
        }
        SettingItem::QuickTapTerm => {
            state.layout.tap_hold_settings.quick_tap_term =
                if value == 0 { None } else { Some(value) };
            state.layout.tap_hold_settings.mark_custom();
            let display = if value == 0 {
                "Auto".to_string()
            } else {
                format!("{value}ms")
            };
            state.set_status(format!("Quick tap term set to: {display}"));
        }
        SettingItem::TappingToggle => {
            state.layout.tap_hold_settings.tapping_toggle = value as u8;
            state.layout.tap_hold_settings.mark_custom();
            state.set_status(format!("Tapping toggle set to: {value} taps"));
        }
        SettingItem::FlowTapTerm => {
            state.layout.tap_hold_settings.flow_tap_term =
                if value == 0 { None } else { Some(value) };
            state.layout.tap_hold_settings.mark_custom();
            let display = if value == 0 {
                "Disabled".to_string()
            } else {
                format!("{value}ms")
            };
            state.set_status(format!("Flow tap term set to: {display}"));
        }
        SettingItem::RgbBrightness => {
            state.layout.rgb_brightness = RgbBrightness::from(value as u8);
            state.set_status(format!("RGB brightness set to: {value}%"));
        }
        SettingItem::RgbSaturation => {
            state.layout.rgb_saturation = RgbSaturation::from(value as u8);
            state.set_status(format!("RGB saturation set to: {value}%"));
        }
        SettingItem::RgbMatrixSpeed => {
            state.layout.rgb_matrix_default_speed = value as u8;
            state.dirty = true;
            state.set_status(format!("RGB Matrix speed set to: {value}"));
        }
        SettingItem::UncoloredKeyBehavior => {
            state.layout.uncolored_key_behavior = UncoloredKeyBehavior::from(value as u8);
            let description = match value {
                0 => "Off (Black)",
                100 => "Show Color",
                n => {
                    return state
                        .set_status(format!("Uncolored key brightness set to: {n}% (Dimmed)"))
                }
            };
            state.set_status(format!("Uncolored key brightness set to: {description}"));
        }
        SettingItem::RgbTimeout => {
            // value is in seconds, convert to milliseconds for storage
            state.layout.rgb_timeout_ms = u32::from(value) * 1000;
            let display = if value == 0 {
                "Disabled".to_string()
            } else if value >= 60 && value.is_multiple_of(60) {
                format!("{} min", value / 60)
            } else {
                format!("{value} sec")
            };
            state.set_status(format!("RGB timeout set to: {display}"));
        }
        SettingItem::KeyboardScale => {
            // value is percentage (100 = 100%), convert to multiplier
            let scale = (value as f32) / 100.0;
            state.config.ui.keyboard_scale = scale;
            if let Err(e) = state.config.save() {
                state.set_status(format!("Failed to save config: {e}"));
            } else {
                state.set_status(format!("Keyboard scale set to: {:.0}%", scale * 100.0));
            }
        }
        SettingItem::IdleTimeout => {
            // value is in seconds, convert to milliseconds for storage
            state.layout.idle_effect_settings.idle_timeout_ms = u32::from(value) * 1000;
            let display = if value == 0 {
                "Disabled".to_string()
            } else if value >= 60 && value.is_multiple_of(60) {
                format!("{} min", value / 60)
            } else {
                format!("{value} sec")
            };
            state.set_status(format!("Idle timeout set to: {display}"));
        }
        SettingItem::IdleEffectDuration => {
            // value is in seconds, convert to milliseconds for storage
            state.layout.idle_effect_settings.idle_effect_duration_ms = u32::from(value) * 1000;
            let display = if value == 0 {
                "Disabled".to_string()
            } else if value >= 60 && value.is_multiple_of(60) {
                format!("{} min", value / 60)
            } else {
                format!("{value} sec")
            };
            state.set_status(format!("Idle effect duration set to: {display}"));
        }
        SettingItem::OverlayRippleMaxRipples => {
            state.layout.rgb_overlay_ripple.max_ripples = value as u8;
            state.set_status(format!("Overlay ripple max ripples set to: {value}"));
        }
        SettingItem::OverlayRippleDuration => {
            state.layout.rgb_overlay_ripple.duration_ms = value;
            state.set_status(format!("Overlay ripple duration set to: {value}ms"));
        }
        SettingItem::OverlayRippleSpeed => {
            state.layout.rgb_overlay_ripple.speed = value as u8;
            state.set_status(format!("Overlay ripple speed set to: {value}"));
        }
        SettingItem::OverlayRippleBandWidth => {
            state.layout.rgb_overlay_ripple.band_width = value as u8;
            state.set_status(format!("Overlay ripple band width set to: {value}"));
        }
        SettingItem::OverlayRippleAmplitude => {
            state.layout.rgb_overlay_ripple.amplitude_pct = value as u8;
            state.set_status(format!("Overlay ripple amplitude set to: {value}%"));
        }
        SettingItem::OverlayRippleWaveCount => {
            state.layout.rgb_overlay_ripple.wave_count = value as u8;
            state.set_status(format!("Overlay ripple waves per key set to: {value}"));
        }
        SettingItem::OverlayRippleWaveDelay => {
            state.layout.rgb_overlay_ripple.wave_delay_ms = value;
            state.set_status(format!("Overlay ripple wave delay set to: {value}ms"));
        }
        SettingItem::OverlayRippleHueShift => {
            state.layout.rgb_overlay_ripple.hue_shift_deg = value as i16;
            state.set_status(format!("Overlay ripple hue shift set to: {value}°"));
        }
        // Combo Settings - Hold Durations
        SettingItem::Combo1HoldDuration => {
            update_combo_hold_duration(state, 0, value);
        }
        SettingItem::Combo2HoldDuration => {
            update_combo_hold_duration(state, 1, value);
        }
        SettingItem::Combo3HoldDuration => {
            update_combo_hold_duration(state, 2, value);
        }
        _ => {}
    }
}

/// Apply a signed numeric setting value
fn apply_signed_numeric_setting(state: &mut AppState, setting: SettingItem, value: i16) {
    if setting == SettingItem::OverlayRippleHueShift {
        state.layout.rgb_overlay_ripple.hue_shift_deg = value;
        state.set_status(format!("Overlay ripple hue shift set to: {value}°"));
    }
}

/// Apply a boolean setting value
fn apply_boolean_setting(state: &mut AppState, setting: SettingItem, value: bool) {
    match setting {
        SettingItem::RetroTapping => {
            state.layout.tap_hold_settings.retro_tapping = value;
            state.layout.tap_hold_settings.mark_custom();
            let display = if value { "On" } else { "Off" };
            state.set_status(format!("Retro tapping set to: {display}"));
        }
        SettingItem::ChordalHold => {
            state.layout.tap_hold_settings.chordal_hold = value;
            state.layout.tap_hold_settings.mark_custom();
            let display = if value { "On" } else { "Off" };
            state.set_status(format!("Chordal hold set to: {display}"));
        }
        SettingItem::RgbEnabled => {
            state.layout.rgb_enabled = value;
            let display = if value { "On" } else { "Off" };
            state.set_status(format!("RGB master switch set to: {display}"));
        }
        SettingItem::ShowHelpOnStartup => {
            state.config.ui.show_help_on_startup = value;
            if let Err(e) = state.config.save() {
                state.set_status(format!("Failed to save config: {e}"));
            } else {
                let display = if value { "On" } else { "Off" };
                state.set_status(format!("Show help on startup set to: {display}"));
            }
        }
        SettingItem::IdleEffectEnabled => {
            state.layout.idle_effect_settings.enabled = value;
            let display = if value { "On" } else { "Off" };
            state.set_status(format!("Idle effect enabled set to: {display}"));
        }
        SettingItem::OverlayRippleEnabled => {
            state.layout.rgb_overlay_ripple.enabled = value;
            let display = if value { "On" } else { "Off" };
            state.set_status(format!("Overlay ripple enabled set to: {display}"));
        }
        SettingItem::OverlayRippleTriggerPress => {
            state.layout.rgb_overlay_ripple.trigger_on_press = value;
            let display = if value { "On" } else { "Off" };
            state.set_status(format!("Overlay ripple trigger on press set to: {display}"));
        }
        SettingItem::OverlayRippleTriggerRelease => {
            state.layout.rgb_overlay_ripple.trigger_on_release = value;
            let display = if value { "On" } else { "Off" };
            state.set_status(format!(
                "Overlay ripple trigger on release set to: {display}"
            ));
        }
        SettingItem::OverlayRippleIgnoreModifiers => {
            state.layout.rgb_overlay_ripple.ignore_modifiers = value;
            let display = if value { "On" } else { "Off" };
            state.set_status(format!("Overlay ripple ignore modifiers set to: {display}"));
        }
        SettingItem::OverlayRippleIgnoreLayerSwitch => {
            state.layout.rgb_overlay_ripple.ignore_layer_switch = value;
            let display = if value { "On" } else { "Off" };
            state.set_status(format!(
                "Overlay ripple ignore layer switch keys set to: {display}"
            ));
        }
        SettingItem::OverlayRippleIgnoreTransparent => {
            state.layout.rgb_overlay_ripple.ignore_transparent = value;
            let display = if value { "On" } else { "Off" };
            state.set_status(format!(
                "Overlay ripple ignore transparent set to: {display}"
            ));
        }
        // Combo Settings
        SettingItem::CombosEnabled => {
            state.layout.combo_settings.enabled = value;
            let display = if value { "On" } else { "Off" };
            state.set_status(format!("Combos enabled set to: {display}"));
        }
        // PaletteFX Settings
        SettingItem::PaletteFxEnabled => {
            state.layout.palette_fx.enabled = value;
            state.mark_dirty();
            if value {
                state.set_status("PaletteFX enabled");
            } else {
                state.set_status("PaletteFX disabled");
            }
        }
        SettingItem::PaletteFxEnableAllEffects => {
            state.layout.palette_fx.enable_all_effects = value;
            state.mark_dirty();
            if value {
                state.set_status("PaletteFX all effects enabled");
            } else {
                state.set_status("PaletteFX all effects disabled");
            }
        }
        SettingItem::PaletteFxEnableAllPalettes => {
            state.layout.palette_fx.enable_all_palettes = value;
            state.mark_dirty();
            if value {
                state.set_status("PaletteFX all palettes enabled");
            } else {
                state.set_status("PaletteFX all palettes disabled");
            }
        }
        _ => {}
    }
}

/// Apply a string setting value
fn apply_string_setting(state: &mut AppState, setting: SettingItem, value: String) -> Result<()> {
    match setting {
        SettingItem::KeymapName => {
            let keymap = if value.is_empty() {
                "default".to_string()
            } else {
                value
            };
            state.layout.metadata.keymap_name = Some(keymap.clone());
            state.layout.metadata.touch();
            state.set_status(format!("Keymap name set to: {keymap}"));
        }
        _ => {}
    }
    Ok(())
}

/// Apply a path setting value
fn apply_path_setting(state: &mut AppState, setting: SettingItem, value: String) -> Result<()> {
    match setting {
        SettingItem::QmkFirmwarePath => {
            state.config.paths.qmk_firmware = if value.is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(&value))
            };
            if let Err(e) = state.config.save() {
                state.set_status(format!("Failed to save config: {e}"));
            } else {
                state.set_status(format!(
                    "QMK firmware path set to: {}",
                    if value.is_empty() {
                        "(not set)"
                    } else {
                        &value
                    }
                ));
            }
        }
        SettingItem::OutputDir => {
            state.config.build.output_dir = std::path::PathBuf::from(&value);
            if let Err(e) = state.config.save() {
                state.set_status(format!("Failed to save config: {e}"));
            } else {
                state.set_status(format!("Output directory set to: {value}"));
            }
        }
        _ => {}
    }
    Ok(())
}

/// Helper function to display theme mode name
fn theme_mode_display(mode: crate::config::ThemeMode) -> &'static str {
    match mode {
        crate::config::ThemeMode::Auto => "Auto",
        crate::config::ThemeMode::Dark => "Dark",
        crate::config::ThemeMode::Light => "Light",
    }
}

/// Apply a key position to a combo setting
pub(super) fn apply_combo_key_position(state: &mut AppState, setting: SettingItem) {
    use crate::models::{ComboAction, ComboDefinition, Position};

    let position = state.selected_position;

    // Determine which combo and which key
    let (combo_idx, is_key1) = match setting {
        SettingItem::Combo1Key1 => (0, true),
        SettingItem::Combo1Key2 => (0, false),
        SettingItem::Combo2Key1 => (1, true),
        SettingItem::Combo2Key2 => (1, false),
        SettingItem::Combo3Key1 => (2, true),
        SettingItem::Combo3Key2 => (2, false),
        _ => return,
    };

    // Ensure combo exists, create if needed
    while state.layout.combo_settings.combos.len() <= combo_idx {
        let action = match combo_idx {
            0 => ComboAction::DisableEffects,
            1 => ComboAction::DisableLighting,
            2 => ComboAction::Bootloader,
            _ => ComboAction::DisableEffects,
        };
        // Create a new combo with placeholder position
        let combo = ComboDefinition::new(
            Position { row: 0, col: 0 },
            Position { row: 0, col: 0 },
            action,
        );
        state.layout.combo_settings.combos.push(combo);
    }

    // Update the specified key
    if let Some(combo) = state.layout.combo_settings.combos.get_mut(combo_idx) {
        if is_key1 {
            combo.key1 = position;
        } else {
            combo.key2 = position;
        }
        state.mark_dirty();
        state.set_status(format!(
            "{} set to position ({}, {})",
            setting.display_name(),
            position.row,
            position.col
        ));
    }
}

/// Update hold duration for a combo
fn update_combo_hold_duration(state: &mut AppState, combo_idx: usize, duration_ms: u16) {
    use crate::models::{ComboAction, ComboDefinition, Position};

    // Ensure combo exists
    while state.layout.combo_settings.combos.len() <= combo_idx {
        let action = match combo_idx {
            0 => ComboAction::DisableEffects,
            1 => ComboAction::DisableLighting,
            2 => ComboAction::Bootloader,
            _ => ComboAction::DisableEffects,
        };
        let combo = ComboDefinition::new(
            Position { row: 0, col: 0 },
            Position { row: 0, col: 0 },
            action,
        );
        state.layout.combo_settings.combos.push(combo);
    }

    if let Some(combo) = state.layout.combo_settings.combos.get_mut(combo_idx) {
        combo.hold_duration_ms = duration_ms;
        state.set_status(format!(
            "Combo {} hold duration set to: {}ms",
            combo_idx + 1,
            duration_ms
        ));
    }
}
