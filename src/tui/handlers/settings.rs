//! Settings manager input handler

use anyhow::Result;
use crossterm::event::{self, KeyCode};

use crate::models::{HoldDecisionMode, RgbBrightness, TapHoldPreset, UncoloredKeyBehavior};
use crate::tui::settings_manager::{ManagerMode, SettingItem};
use crate::tui::{AppState, PopupType};

/// Handle input for settings manager
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
pub fn handle_settings_manager_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match &state.settings_manager_state.mode.clone() {
        ManagerMode::Browsing => match key.code {
            KeyCode::Esc => {
                state.active_popup = None;
                state.set_status("Settings closed");
                Ok(false)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = SettingItem::all().len();
                state.settings_manager_state.select_previous(count);
                Ok(false)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = SettingItem::all().len();
                state.settings_manager_state.select_next(count);
                Ok(false)
            }
            KeyCode::Enter => {
                // Start editing the selected setting
                let settings = SettingItem::all();
                if let Some(setting) = settings.get(state.settings_manager_state.selected) {
                    match setting {
                        SettingItem::UncoloredKeyBehavior => {
                            state.settings_manager_state.start_editing_numeric(
                                *setting,
                                state.layout.uncolored_key_behavior.as_percent() as u16,
                                0,
                                100, // 0=Off, 1-99=Dim, 100=Full
                            );
                        }
                        SettingItem::TapHoldPreset => {
                            state
                                .settings_manager_state
                                .start_selecting_tap_hold_preset(
                                    state.layout.tap_hold_settings.preset,
                                );
                        }
                        SettingItem::TappingTerm => {
                            state.settings_manager_state.start_editing_numeric(
                                *setting,
                                state.layout.tap_hold_settings.tapping_term,
                                100,
                                500,
                            );
                        }
                        SettingItem::QuickTapTerm => {
                            let current =
                                state.layout.tap_hold_settings.quick_tap_term.unwrap_or(0);
                            state
                                .settings_manager_state
                                .start_editing_numeric(*setting, current, 0, 500);
                        }
                        SettingItem::HoldMode => {
                            state.settings_manager_state.start_selecting_hold_mode(
                                state.layout.tap_hold_settings.hold_mode,
                            );
                        }
                        SettingItem::RetroTapping => {
                            state.settings_manager_state.start_toggling_boolean(
                                *setting,
                                state.layout.tap_hold_settings.retro_tapping,
                            );
                        }
                        SettingItem::TappingToggle => {
                            state.settings_manager_state.start_editing_numeric(
                                *setting,
                                u16::from(state.layout.tap_hold_settings.tapping_toggle),
                                1,
                                10,
                            );
                        }
                        SettingItem::FlowTapTerm => {
                            let current = state.layout.tap_hold_settings.flow_tap_term.unwrap_or(0);
                            state
                                .settings_manager_state
                                .start_editing_numeric(*setting, current, 0, 300);
                        }
                        SettingItem::ChordalHold => {
                            state.settings_manager_state.start_toggling_boolean(
                                *setting,
                                state.layout.tap_hold_settings.chordal_hold,
                            );
                        }
                        // RGB Settings
                        SettingItem::RgbEnabled => {
                            state
                                .settings_manager_state
                                .start_toggling_boolean(*setting, state.layout.rgb_enabled);
                        }
                        SettingItem::RgbBrightness => {
                            state.settings_manager_state.start_editing_numeric(
                                *setting,
                                u16::from(state.layout.rgb_brightness.as_percent()),
                                0,
                                100, // 0-100%
                            );
                        }
                        SettingItem::RgbTimeout => {
                            // RGB timeout is stored as ms but we edit in seconds for usability
                            // Max 10 minutes = 600000ms, stored as u32 but edited as u16 seconds
                            let current_secs = (state.layout.rgb_timeout_ms / 1000) as u16;
                            state.settings_manager_state.start_editing_numeric(
                                *setting,
                                current_secs,
                                0,
                                600, // Max 10 minutes in seconds
                            );
                        }

                        // Global settings
                        SettingItem::QmkFirmwarePath => {
                            state.settings_manager_state.start_editing_path(
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
                            match crate::tui::onboarding_wizard::OnboardingWizardState::new_for_keyboard_selection(&qmk_path) {
                                Ok(wizard_state) => {
                                    state.wizard_state = wizard_state;
                                    state.active_popup = Some(PopupType::SetupWizard);
                                    state.settings_manager_state.cancel();
                                    state.set_status("Select keyboard - Type to filter, Enter to select");
                                }
                                Err(e) => {
                                    state.set_error(format!("Failed to scan keyboards: {e}"));
                                }
                            }
                        }
                        SettingItem::LayoutVariant => {
                            // Trigger layout variant picker - same as Ctrl+Y
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

                            if let Err(e) = state
                                .layout_picker_state
                                .load_layouts(&qmk_path, &base_keyboard)
                            {
                                state.set_error(format!("Failed to load layouts: {e}"));
                                return Ok(false);
                            }

                            state.active_popup = Some(PopupType::LayoutPicker);
                            state.set_status("Select layout variant - ↑↓: Navigate, Enter: Select");
                        }
                        SettingItem::KeymapName => {
                            let keymap = state
                                .layout
                                .metadata
                                .keymap_name
                                .clone()
                                .unwrap_or_default();
                            state
                                .settings_manager_state
                                .start_editing_string(*setting, keymap);
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
                            state
                                .settings_manager_state
                                .start_selecting_output_format(selected);
                        }
                        SettingItem::OutputDir => {
                            state.settings_manager_state.start_editing_path(
                                *setting,
                                state.config.build.output_dir.to_string_lossy().to_string(),
                            );
                        }
                        SettingItem::ShowHelpOnStartup => {
                            state.settings_manager_state.start_toggling_boolean(
                                *setting,
                                state.config.ui.show_help_on_startup,
                            );
                        }
                    }
                    state.set_status("Select option with ↑↓, Enter to apply");
                }
                Ok(false)
            }
            _ => Ok(false),
        },

        ManagerMode::SelectingTapHoldPreset { .. } => match key.code {
            KeyCode::Esc => {
                state.settings_manager_state.cancel();
                state.set_status("Cancelled");
                Ok(false)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = TapHoldPreset::all().len();
                state.settings_manager_state.option_previous(count);
                Ok(false)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = TapHoldPreset::all().len();
                state.settings_manager_state.option_next(count);
                Ok(false)
            }
            KeyCode::Enter => {
                if let Some(selected_idx) = state.settings_manager_state.get_selected_option() {
                    if let Some(&preset) = TapHoldPreset::all().get(selected_idx) {
                        state.layout.tap_hold_settings.apply_preset(preset);
                        state.mark_dirty();
                        state.settings_manager_state.cancel();
                        state.set_status(format!(
                            "Tap-hold preset set to: {}",
                            preset.display_name()
                        ));
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        },
        ManagerMode::SelectingHoldMode { .. } => match key.code {
            KeyCode::Esc => {
                state.settings_manager_state.cancel();
                state.set_status("Cancelled");
                Ok(false)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = HoldDecisionMode::all().len();
                state.settings_manager_state.option_previous(count);
                Ok(false)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = HoldDecisionMode::all().len();
                state.settings_manager_state.option_next(count);
                Ok(false)
            }
            KeyCode::Enter => {
                if let Some(selected_idx) = state.settings_manager_state.get_selected_option() {
                    if let Some(&mode) = HoldDecisionMode::all().get(selected_idx) {
                        state.layout.tap_hold_settings.hold_mode = mode;
                        state.layout.tap_hold_settings.mark_custom();
                        state.mark_dirty();
                        state.settings_manager_state.cancel();
                        state.set_status(format!("Hold mode set to: {}", mode.display_name()));
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        },
        ManagerMode::EditingNumeric { setting, .. } => {
            let setting = *setting;
            match key.code {
                KeyCode::Esc => {
                    state.settings_manager_state.cancel();
                    state.set_status("Cancelled");
                    Ok(false)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state.settings_manager_state.increment_numeric(10);
                    Ok(false)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.settings_manager_state.decrement_numeric(10);
                    Ok(false)
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    state.settings_manager_state.handle_char_input(c);
                    Ok(false)
                }
                KeyCode::Backspace => {
                    state.settings_manager_state.handle_backspace();
                    Ok(false)
                }
                KeyCode::Enter => {
                    if let Some(value) = state.settings_manager_state.get_numeric_value() {
                        apply_numeric_setting(state, setting, value);
                        state.mark_dirty();
                        state.settings_manager_state.cancel();
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::TogglingBoolean { setting, .. } => {
            let setting = *setting;
            match key.code {
                KeyCode::Esc => {
                    state.settings_manager_state.cancel();
                    state.set_status("Cancelled");
                    Ok(false)
                }
                KeyCode::Up | KeyCode::Down | KeyCode::Char('k') | KeyCode::Char('j') => {
                    state.settings_manager_state.option_previous(2);
                    Ok(false)
                }
                KeyCode::Enter => {
                    if let Some(value) = state.settings_manager_state.get_boolean_value() {
                        apply_boolean_setting(state, setting, value);
                        state.mark_dirty();
                        state.settings_manager_state.cancel();
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::EditingString { setting, .. } => {
            let setting = *setting;
            match key.code {
                KeyCode::Esc => {
                    state.settings_manager_state.cancel();
                    state.set_status("Cancelled");
                    Ok(false)
                }
                KeyCode::Char(c) => {
                    state.settings_manager_state.handle_string_char_input(c);
                    Ok(false)
                }
                KeyCode::Backspace => {
                    state.settings_manager_state.handle_string_backspace();
                    Ok(false)
                }
                KeyCode::Enter => {
                    if let Some(value) = state.settings_manager_state.get_string_value() {
                        apply_string_setting(state, setting, value.to_string())?;
                        state.settings_manager_state.cancel();
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::SelectingOutputFormat { .. } => match key.code {
            KeyCode::Esc => {
                state.settings_manager_state.cancel();
                state.set_status("Cancelled");
                Ok(false)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                state.settings_manager_state.option_previous(3);
                Ok(false)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.settings_manager_state.option_next(3);
                Ok(false)
            }
            KeyCode::Enter => {
                if let Some(selected_idx) =
                    state.settings_manager_state.get_output_format_selected()
                {
                    let format = match selected_idx {
                        0 => "uf2",
                        1 => "hex",
                        2 => "bin",
                        _ => "uf2",
                    };
                    state.layout.metadata.output_format = Some(format.to_string());
                    state.layout.metadata.touch();
                    state.set_status(format!("Output format set to: {format}"));
                    state.settings_manager_state.cancel();
                }
                Ok(false)
            }
            _ => Ok(false),
        },
        ManagerMode::EditingPath { setting, .. } => {
            let setting = *setting;
            match key.code {
                KeyCode::Esc => {
                    state.settings_manager_state.cancel();
                    state.set_status("Cancelled");
                    Ok(false)
                }
                KeyCode::Char(c) => {
                    state.settings_manager_state.handle_string_char_input(c);
                    Ok(false)
                }
                KeyCode::Backspace => {
                    state.settings_manager_state.handle_string_backspace();
                    Ok(false)
                }
                KeyCode::Enter => {
                    if let Some(value) = state.settings_manager_state.get_string_value() {
                        apply_path_setting(state, setting, value.to_string())?;
                        state.settings_manager_state.cancel();
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
    }
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
