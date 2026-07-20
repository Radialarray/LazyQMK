//! Handle Enter key while browsing settings (triggers editing or opens pickers).

use anyhow::Result;

use crate::tui::settings_manager::SettingItem;
use crate::tui::{ActiveComponent, AppState, PopupType};

/// Handle Enter key while browsing settings (triggers editing or opens pickers)
pub(super) fn handle_browsing_enter(state: &mut AppState) -> Result<bool> {
    // Get the selected setting from the manager's state
    let selected_idx =
        if let Some(ActiveComponent::SettingsManager(ref manager)) = state.active_component {
            manager.state().selected
        } else {
            return Ok(false);
        };

    let settings = SettingItem::all();
    if let Some(setting) = settings.get(selected_idx) {
        // Update the manager's state to start editing
        if let Some(ActiveComponent::SettingsManager(ref mut manager)) = state.active_component {
            match setting {
                SettingItem::UncoloredKeyBehavior => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        state.layout.uncolored_key_behavior.as_percent() as u16,
                        0,
                        100,
                        100,
                    );
                }
                SettingItem::TapHoldPreset => {
                    manager
                        .state_mut()
                        .start_selecting_tap_hold_preset(state.layout.tap_hold_settings.preset);
                }
                SettingItem::TappingTerm => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        state.layout.tap_hold_settings.tapping_term,
                        100,
                        500,
                        200,
                    );
                }
                SettingItem::QuickTapTerm => {
                    let current = state
                        .layout
                        .tap_hold_settings
                        .quick_tap_term
                        .unwrap_or(state.layout.tap_hold_settings.tapping_term);
                    manager
                        .state_mut()
                        .start_editing_numeric(*setting, current, 0, 500, 200);
                }
                SettingItem::HoldMode => {
                    manager
                        .state_mut()
                        .start_selecting_hold_mode(state.layout.tap_hold_settings.hold_mode);
                }
                SettingItem::RetroTapping => {
                    manager.state_mut().start_toggling_boolean(
                        *setting,
                        state.layout.tap_hold_settings.retro_tapping,
                    );
                }
                SettingItem::TappingToggle => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.tap_hold_settings.tapping_toggle),
                        1,
                        10,
                        5,
                    );
                }
                SettingItem::FlowTapTerm => {
                    let current = state
                        .layout
                        .tap_hold_settings
                        .flow_tap_term
                        .unwrap_or(state.layout.tap_hold_settings.tapping_term);
                    manager
                        .state_mut()
                        .start_editing_numeric(*setting, current, 0, 300, 150);
                }
                SettingItem::ChordalHold => {
                    manager.state_mut().start_toggling_boolean(
                        *setting,
                        state.layout.tap_hold_settings.chordal_hold,
                    );
                }
                SettingItem::RgbEnabled => {
                    manager
                        .state_mut()
                        .start_toggling_boolean(*setting, state.layout.rgb_enabled);
                }
                SettingItem::RgbBrightness => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.rgb_brightness.as_percent()),
                        0,
                        100,
                        100,
                    );
                }
                SettingItem::RgbSaturation => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.rgb_saturation.as_percent()),
                        0,
                        200,
                        100,
                    );
                }
                SettingItem::RgbMatrixSpeed => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.rgb_matrix_default_speed),
                        0,
                        255,
                        127,
                    );
                }
                SettingItem::RgbTimeout => {
                    let current_secs = (state.layout.rgb_timeout_ms / 1000) as u16;
                    manager
                        .state_mut()
                        .start_editing_numeric(*setting, current_secs, 0, 600, 0);
                }
                SettingItem::QmkFirmwarePath => {
                    manager.state_mut().start_editing_path(
                        *setting,
                        state
                            .config
                            .paths
                            .qmk_firmware
                            .clone()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default(),
                    );
                }
                SettingItem::Keyboard => {
                    // Check if QMK path is configured
                    let qmk_path = if let Some(path) = &state.config.paths.qmk_firmware {
                        path.clone()
                    } else {
                        state.set_error("QMK firmware path not configured. Set it first.");
                        return Ok(false);
                    };

                    // Start wizard directly at keyboard selection step
                    // This closes the settings manager temporarily
                    match crate::tui::onboarding_wizard::OnboardingWizardState::new_for_keyboard_selection(&qmk_path) {
                        Ok(wizard_state) => {
                            state.wizard_state = wizard_state;
                            state.active_component = None;
                            state.active_popup = Some(PopupType::SetupWizard);
                            state.set_status("Select keyboard - Type to filter, Enter to select");
                        }
                        Err(e) => {
                            state.set_error(format!("Failed to scan keyboards: {e}"));
                        }
                    }
                }
                SettingItem::LayoutVariant => {
                    // Mark that we came from settings so we return there
                    state.return_to_settings_after_picker = true;

                    // Load available layouts for current keyboard
                    let qmk_path = if let Some(path) = &state.config.paths.qmk_firmware {
                        path.clone()
                    } else {
                        state.set_error("QMK firmware path not configured");
                        return Ok(false);
                    };

                    // Extract base keyboard path from layout metadata
                    let keyboard = state.layout.metadata.keyboard.as_deref().unwrap_or("");
                    let base_keyboard = AppState::extract_base_keyboard(keyboard);

                    if let Err(e) = state.open_layout_variant_picker(&qmk_path, &base_keyboard) {
                        state.set_error(format!("Failed to load layouts: {e}"));
                        return Ok(false);
                    }

                    state.set_status(
                        "Select layout variant - ↑↓: Navigate, Enter: Apply, Esc: Cancel",
                    );
                }
                SettingItem::KeymapName => {
                    let keymap = state
                        .layout
                        .metadata
                        .keymap_name
                        .clone()
                        .unwrap_or_default();
                    manager.state_mut().start_editing_string(*setting, keymap);
                }
                SettingItem::OutputFormat => {
                    let current_format = state
                        .layout
                        .metadata
                        .output_format
                        .as_deref()
                        .unwrap_or("uf2");
                    let selected = match current_format {
                        "hex" => 1,
                        "bin" => 2,
                        _ => 0, // uf2
                    };
                    manager.state_mut().start_selecting_output_format(selected);
                }
                SettingItem::OutputDir => {
                    manager.state_mut().start_editing_path(
                        *setting,
                        state.config.build.output_dir.to_string_lossy().to_string(),
                    );
                }
                SettingItem::ShowHelpOnStartup => {
                    manager
                        .state_mut()
                        .start_toggling_boolean(*setting, state.config.ui.show_help_on_startup);
                }
                SettingItem::ThemeMode => {
                    let selected = match state.config.ui.theme_mode {
                        crate::config::ThemeMode::Dark => 1,
                        crate::config::ThemeMode::Light => 2,
                        crate::config::ThemeMode::Auto => 0,
                    };
                    manager.state_mut().start_selecting_theme_mode(selected);
                }
                SettingItem::KeyboardScale => {
                    // Scale is stored as a multiplier (1.0 = 100%)
                    // UI shows as percentage (100 = 100%)
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        (state.config.ui.keyboard_scale * 100.0) as u16,
                        25,  // 25% minimum
                        200, // 200% maximum
                        100,
                    );
                }
                SettingItem::IdleEffectEnabled => {
                    manager.state_mut().start_toggling_boolean(
                        *setting,
                        state.layout.idle_effect_settings.enabled,
                    );
                }
                SettingItem::IdleTimeout => {
                    let current_secs =
                        (state.layout.idle_effect_settings.idle_timeout_ms / 1000) as u16;
                    manager
                        .state_mut()
                        .start_editing_numeric(*setting, current_secs, 0, 3600, 60);
                }
                SettingItem::IdleEffectDuration => {
                    let current_secs =
                        (state.layout.idle_effect_settings.idle_effect_duration_ms / 1000) as u16;
                    manager
                        .state_mut()
                        .start_editing_numeric(*setting, current_secs, 0, 3600, 300);
                }
                SettingItem::IdleEffectMode => {
                    manager.state_mut().start_selecting_idle_effect_mode(
                        state.layout.idle_effect_settings.idle_effect_mode,
                    );
                }
                SettingItem::OverlayRippleEnabled => {
                    manager
                        .state_mut()
                        .start_toggling_boolean(*setting, state.layout.rgb_overlay_ripple.enabled);
                }
                SettingItem::OverlayRippleMaxRipples => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.rgb_overlay_ripple.max_ripples),
                        1,
                        8,
                        4,
                    );
                }
                SettingItem::OverlayRippleDuration => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        state.layout.rgb_overlay_ripple.duration_ms,
                        100,
                        5000,
                        1500,
                    );
                }
                SettingItem::OverlayRippleSpeed => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.rgb_overlay_ripple.speed),
                        1,
                        255,
                        64,
                    );
                }
                SettingItem::OverlayRippleBandWidth => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.rgb_overlay_ripple.band_width),
                        1,
                        255,
                        30,
                    );
                }
                SettingItem::OverlayRippleAmplitude => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.rgb_overlay_ripple.amplitude_pct),
                        0,
                        100,
                        50,
                    );
                }
                SettingItem::OverlayRippleColorMode => {
                    manager.state_mut().start_selecting_ripple_color_mode(
                        state.layout.rgb_overlay_ripple.color_mode,
                    );
                }
                SettingItem::OverlayRippleFixedColor => {
                    state.return_to_settings_after_picker = true;
                    state.open_color_picker(
                        crate::tui::component::ColorPickerContext::OverlayRippleFixedColor,
                        state.layout.rgb_overlay_ripple.fixed_color,
                    );
                    state.set_status("Adjust ripple fixed color - Enter to apply");
                    return Ok(false);
                }
                SettingItem::OverlayRippleHueShift => {
                    manager.state_mut().start_editing_signed_numeric(
                        *setting,
                        state.layout.rgb_overlay_ripple.hue_shift_deg,
                        -180,
                        180,
                        60,
                    );
                }
                SettingItem::OverlayRippleTriggerPress => {
                    manager.state_mut().start_toggling_boolean(
                        *setting,
                        state.layout.rgb_overlay_ripple.trigger_on_press,
                    );
                }
                SettingItem::OverlayRippleTriggerRelease => {
                    manager.state_mut().start_toggling_boolean(
                        *setting,
                        state.layout.rgb_overlay_ripple.trigger_on_release,
                    );
                }
                SettingItem::OverlayRippleIgnoreModifiers => {
                    manager.state_mut().start_toggling_boolean(
                        *setting,
                        state.layout.rgb_overlay_ripple.ignore_modifiers,
                    );
                }
                SettingItem::OverlayRippleIgnoreLayerSwitch => {
                    manager.state_mut().start_toggling_boolean(
                        *setting,
                        state.layout.rgb_overlay_ripple.ignore_layer_switch,
                    );
                }
                SettingItem::OverlayRippleKeyActionPalette => {
                    manager.state_mut().start_selecting_key_action_palette(
                        state.layout.rgb_overlay_ripple.key_action_palette,
                    );
                }
                SettingItem::OverlayRippleIgnoreTransparent => {
                    manager.state_mut().start_toggling_boolean(
                        *setting,
                        state.layout.rgb_overlay_ripple.ignore_transparent,
                    );
                }
                SettingItem::OverlayRippleWaveCount => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.rgb_overlay_ripple.wave_count),
                        1,
                        5,
                        1,
                    );
                }
                SettingItem::OverlayRippleWaveDelay => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        state.layout.rgb_overlay_ripple.wave_delay_ms,
                        50,
                        500,
                        100,
                    );
                }
                // PaletteFX Settings
                SettingItem::PaletteFxEnabled => {
                    manager
                        .state_mut()
                        .start_toggling_boolean(*setting, state.layout.palette_fx.enabled);
                }
                SettingItem::PaletteFxDefaultEffect => {
                    manager
                        .state_mut()
                        .start_selecting_palette_fx_effect(state.layout.palette_fx.default_effect);
                }
                SettingItem::PaletteFxDefaultPalette => {
                    manager.state_mut().start_selecting_palette_fx_palette(
                        state.layout.palette_fx.default_palette,
                    );
                }
                SettingItem::PaletteFxEnableAllEffects => {
                    manager.state_mut().start_toggling_boolean(
                        *setting,
                        state.layout.palette_fx.enable_all_effects,
                    );
                }
                SettingItem::PaletteFxEnableAllPalettes => {
                    manager.state_mut().start_toggling_boolean(
                        *setting,
                        state.layout.palette_fx.enable_all_palettes,
                    );
                }
                // Combo Settings
                SettingItem::CombosEnabled => {
                    manager
                        .state_mut()
                        .start_toggling_boolean(*setting, state.layout.combo_settings.enabled);
                }
                SettingItem::Combo1Key1
                | SettingItem::Combo1Key2
                | SettingItem::Combo2Key1
                | SettingItem::Combo2Key2
                | SettingItem::Combo3Key1
                | SettingItem::Combo3Key2 => {
                    // Signal to parent to enter key selection mode
                    // The parent will handle the actual key navigation
                    manager.state_mut().start_selecting_key_position(
                        *setting,
                        format!(
                            "Navigate keyboard and press Enter to select {}",
                            setting.display_name()
                        ),
                    );
                    state.set_status("Navigate to key position and press Enter to select");
                }
                SettingItem::Combo1HoldDuration => {
                    let current = state
                        .layout
                        .combo_settings
                        .combos
                        .first()
                        .map(|c| c.hold_duration_ms)
                        .unwrap_or(500);
                    manager
                        .state_mut()
                        .start_editing_numeric(*setting, current, 50, 2000, 500);
                }
                SettingItem::Combo2HoldDuration => {
                    let current = state
                        .layout
                        .combo_settings
                        .combos
                        .get(1)
                        .map(|c| c.hold_duration_ms)
                        .unwrap_or(500);
                    manager
                        .state_mut()
                        .start_editing_numeric(*setting, current, 50, 2000, 500);
                }
                SettingItem::Combo3HoldDuration => {
                    let current = state
                        .layout
                        .combo_settings
                        .combos
                        .get(2)
                        .map(|c| c.hold_duration_ms)
                        .unwrap_or(500);
                    manager
                        .state_mut()
                        .start_editing_numeric(*setting, current, 50, 2000, 500);
                }
            }
            state.set_status("Select option with ↑↓, Enter to apply");
        }
    }
    Ok(false)
}
