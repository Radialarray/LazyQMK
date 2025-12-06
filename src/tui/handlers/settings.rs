//! Settings manager input handler

use anyhow::Result;
use crossterm::event;

use crate::models::{HoldDecisionMode, RgbBrightness, TapHoldPreset, UncoloredKeyBehavior};
use crate::tui::settings_manager::{ManagerMode, SettingItem, SettingsManagerContext, SettingsManagerEvent};
use crate::tui::{ActiveComponent, AppState, PopupType};

/// Handle input for settings manager using Component trait pattern
pub fn handle_settings_manager_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Check if we're in browsing mode and handle Enter key specially
    let is_browsing = if let Some(ActiveComponent::SettingsManager(ref manager)) = state.active_component {
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

/// Handle Enter key while browsing settings (triggers editing or opens pickers)
fn handle_browsing_enter(state: &mut AppState) -> Result<bool> {
    // Get the selected setting from the manager's state
    let selected_idx = if let Some(ActiveComponent::SettingsManager(ref manager)) = state.active_component {
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
                    manager.state_mut().start_selecting_tap_hold_preset(
                        state.layout.tap_hold_settings.preset,
                    );
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
                    manager.state_mut().start_editing_numeric(*setting, current, 0, 500);
                }
                SettingItem::HoldMode => {
                    manager.state_mut().start_selecting_hold_mode(
                        state.layout.tap_hold_settings.hold_mode,
                    );
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
                    manager.state_mut().start_editing_numeric(*setting, current, 0, 300);
                }
                SettingItem::ChordalHold => {
                    manager.state_mut().start_toggling_boolean(
                        *setting,
                        state.layout.tap_hold_settings.chordal_hold,
                    );
                }
                SettingItem::RgbEnabled => {
                    manager.state_mut().start_toggling_boolean(*setting, state.layout.rgb_enabled);
                }
                SettingItem::RgbBrightness => {
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        u16::from(state.layout.rgb_brightness.as_percent()),
                        0,
                        100,
                    );
                }
                SettingItem::RgbTimeout => {
                    let current_secs = (state.layout.rgb_timeout_ms / 1000) as u16;
                    manager.state_mut().start_editing_numeric(
                        *setting,
                        current_secs,
                        0,
                        600,
                    );
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
                    manager.state_mut().start_toggling_boolean(
                        *setting,
                        state.config.ui.show_help_on_startup,
                    );
                }
            }
            state.set_status("Select option with ↑↓, Enter to apply");
        }
    }
    Ok(false)
}

/// Handle settings manager events
fn handle_settings_manager_event(state: &mut AppState, event: SettingsManagerEvent) -> Result<bool> {
    match event {
        SettingsManagerEvent::SettingsUpdated => {
            // Apply the settings from the manager's current state
            apply_settings(state)?;
            
            // Return to browsing mode
            if let Some(ActiveComponent::SettingsManager(ref mut manager)) = state.active_component {
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
                        state.set_status(format!("Tap-hold preset set to: {}", preset.display_name()));
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
            crate::tui::settings_manager::ManagerMode::EditingPath { setting, .. } => {
                if let Some(value) = manager_state.get_string_value() {
                    apply_path_setting(state, *setting, value.to_string())?;
                }
            }
            _ => {}
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
            } else if value >= 60 && value % 60 == 0 {
                format!("{} min", value / 60)
            } else {
                format!("{value} sec")
            };
            state.set_status(format!("RGB timeout set to: {display}"));
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
