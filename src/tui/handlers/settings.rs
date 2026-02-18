//! Settings manager input handler

use anyhow::Result;
use crossterm::event;

use crate::models::{
    HoldDecisionMode, RgbBrightness, RgbMatrixEffect, RgbSaturation, RippleColorMode,
    TapHoldPreset, UncoloredKeyBehavior,
};
use crate::tui::settings_manager::{
    ManagerMode, SettingItem, SettingsManagerContext, SettingsManagerEvent,
};
use crate::tui::{ActiveComponent, AppState, PopupType};

/// Handle input for settings manager using Component trait pattern
pub fn handle_settings_manager_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Check if we're in key position selection mode
    let is_selecting_key_position =
        if let Some(ActiveComponent::SettingsManager(ref manager)) = state.active_component {
            matches!(
                manager.state().mode,
                ManagerMode::SelectingKeyPosition { .. }
            )
        } else {
            false
        };

    // Handle key position selection mode separately
    if is_selecting_key_position {
        return handle_key_position_selection(state, key);
    }

    // Check if we're in browsing mode and handle Enter key specially
    let is_browsing =
        if let Some(ActiveComponent::SettingsManager(ref manager)) = state.active_component {
            matches!(manager.state().mode, ManagerMode::Browsing)
        } else {
            false
        };

    // Handle Enter in browsing mode - triggers editing submodes or opens nested pickers
    if is_browsing && key.code == event::KeyCode::Enter {
        return handle_browsing_enter(state);
    }

    // Extract the component and handle input with context
    if let Some(ActiveComponent::SettingsManager(ref mut manager)) = state.active_component {
        // Build context from current state
        let context = SettingsManagerContext {
            rgb_enabled: state.layout.rgb_enabled,
            rgb_brightness: state.layout.rgb_brightness,
            rgb_timeout_ms: state.layout.rgb_timeout_ms,
            uncolored_key_behavior: state.layout.uncolored_key_behavior,
            idle_effect_settings: state.layout.idle_effect_settings.clone(),
            overlay_ripple_settings: state.layout.rgb_overlay_ripple.clone(),
            tap_hold_settings: state.layout.tap_hold_settings.clone(),
            config: state.config.clone(),
            layout: state.layout.clone(),
        };

        // Handle input and check for events
        if let Some(event) = manager.handle_input_with_context(key, &context) {
            return handle_settings_manager_event(state, event);
        }
    }
    Ok(false)
}

/// Handle key position selection mode
fn handle_key_position_selection(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        event::KeyCode::Esc => {
            // Cancel selection and return to browsing
            if let Some(ActiveComponent::SettingsManager(ref mut manager)) = state.active_component
            {
                manager.state_mut().cancel();
                state.set_status("Key position selection cancelled");
            }
            Ok(false)
        }
        event::KeyCode::Enter => {
            // Apply the selected position
            if let Some(ActiveComponent::SettingsManager(ref manager)) = state.active_component {
                if let ManagerMode::SelectingKeyPosition { setting, .. } = &manager.state().mode {
                    let setting = *setting;
                    // Apply the key position
                    apply_combo_key_position(state, setting);
                }
            }
            // Return to browsing mode
            if let Some(ActiveComponent::SettingsManager(ref mut manager)) = state.active_component
            {
                manager.state_mut().cancel();
            }
            Ok(false)
        }
        event::KeyCode::Up | event::KeyCode::Char('k') => {
            // Navigate keyboard up
            if let Some(new_pos) = state.mapping.find_position_up(state.selected_position) {
                state.selected_position = new_pos;
            }
            Ok(false)
        }
        event::KeyCode::Down | event::KeyCode::Char('j') => {
            // Navigate keyboard down
            if let Some(new_pos) = state.mapping.find_position_down(state.selected_position) {
                state.selected_position = new_pos;
            }
            Ok(false)
        }
        event::KeyCode::Left | event::KeyCode::Char('h') => {
            // Navigate keyboard left
            if let Some(new_pos) = state.mapping.find_position_left(state.selected_position) {
                state.selected_position = new_pos;
            }
            Ok(false)
        }
        event::KeyCode::Right | event::KeyCode::Char('l') => {
            // Navigate keyboard right
            if let Some(new_pos) = state.mapping.find_position_right(state.selected_position) {
                state.selected_position = new_pos;
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle Enter key while browsing settings (triggers editing or opens pickers)
fn handle_browsing_enter(state: &mut AppState) -> Result<bool> {
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
                    );
                }
                SettingItem::QuickTapTerm => {
                    let current = state.layout.tap_hold_settings.quick_tap_term.unwrap_or(0);
                    manager
                        .state_mut()
                        .start_editing_numeric(*setting, current, 0, 500);
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
                    );
                }
                SettingItem::FlowTapTerm => {
                    let current = state.layout.tap_hold_settings.flow_tap_term.unwrap_or(0);
                    manager
                        .state_mut()
                        .start_editing_numeric(*setting, current, 0, 300);
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
                    );
                }
                SettingItem::RgbSaturation => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.rgb_saturation.as_percent()),
                        0,
                        200,
                    );
                }
                SettingItem::RgbMatrixSpeed => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.rgb_matrix_default_speed),
                        0,
                        255,
                    );
                }
                SettingItem::RgbTimeout => {
                    let current_secs = (state.layout.rgb_timeout_ms / 1000) as u16;
                    manager
                        .state_mut()
                        .start_editing_numeric(*setting, current_secs, 0, 600);
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

                    state.set_status("Select layout variant - ↑↓: Navigate, Enter: Select");
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
                        .start_editing_numeric(*setting, current_secs, 0, 3600);
                }
                SettingItem::IdleEffectDuration => {
                    let current_secs =
                        (state.layout.idle_effect_settings.idle_effect_duration_ms / 1000) as u16;
                    manager
                        .state_mut()
                        .start_editing_numeric(*setting, current_secs, 0, 3600);
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
                    );
                }
                SettingItem::OverlayRippleDuration => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        state.layout.rgb_overlay_ripple.duration_ms,
                        100,
                        5000,
                    );
                }
                SettingItem::OverlayRippleSpeed => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.rgb_overlay_ripple.speed),
                        1,
                        255,
                    );
                }
                SettingItem::OverlayRippleBandWidth => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.rgb_overlay_ripple.band_width),
                        1,
                        20,
                    );
                }
                SettingItem::OverlayRippleAmplitude => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.rgb_overlay_ripple.amplitude_pct),
                        0,
                        100,
                    );
                }
                SettingItem::OverlayRippleColorMode => {
                    manager.state_mut().start_selecting_ripple_color_mode(
                        state.layout.rgb_overlay_ripple.color_mode,
                    );
                }
                // Note: OverlayRippleFixedColor is not editable via simple numeric input
                // Would require a full color picker UI component
                SettingItem::OverlayRippleFixedColor => {
                    state.set_status(
                        "Fixed color editing requires color picker (not yet implemented)",
                    );
                    return Ok(false);
                }
                SettingItem::OverlayRippleHueShift => {
                    // hue_shift_deg is i16, but we need to convert for u16 editor
                    let current = if state.layout.rgb_overlay_ripple.hue_shift_deg < 0 {
                        0
                    } else {
                        state.layout.rgb_overlay_ripple.hue_shift_deg as u16
                    };
                    manager
                        .state_mut()
                        .start_editing_numeric(*setting, current, 0, 359);
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
                SettingItem::OverlayRippleIgnoreTransparent => {
                    manager.state_mut().start_toggling_boolean(
                        *setting,
                        state.layout.rgb_overlay_ripple.ignore_transparent,
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
                        .start_editing_numeric(*setting, current, 50, 2000);
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
                        .start_editing_numeric(*setting, current, 50, 2000);
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
                        .start_editing_numeric(*setting, current, 50, 2000);
                }
            }
            state.set_status("Select option with ↑↓, Enter to apply");
        }
    }
    Ok(false)
}

/// Handle settings manager events
fn handle_settings_manager_event(
    state: &mut AppState,
    event: SettingsManagerEvent,
) -> Result<bool> {
    match event {
        SettingsManagerEvent::SettingsUpdated => {
            // Apply the settings from the manager's current state
            apply_settings(state)?;

            // Return to browsing mode
            if let Some(ActiveComponent::SettingsManager(ref mut manager)) = state.active_component
            {
                manager.state_mut().cancel();
            }

            Ok(false)
        }
        SettingsManagerEvent::Cancelled => {
            state.close_component();
            state.set_status("Settings closed");
            Ok(false)
        }
        SettingsManagerEvent::Closed => {
            state.close_component();
            state.set_status("Settings closed");
            Ok(false)
        }
    }
}

/// Apply settings from the manager state to the app state
fn apply_settings(state: &mut AppState) -> Result<()> {
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
fn apply_combo_key_position(state: &mut AppState, setting: SettingItem) {
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
