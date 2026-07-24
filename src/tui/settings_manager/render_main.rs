//! Main rendering functions for the settings manager dialog.
//!
//! Contains the top-level render dispatcher, the settings list view,
//! and the setting value display helper.

use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::models::{
    IdleEffectSettings, RgbBrightness, RgbOverlayRippleSettings, TapHoldSettings,
    UncoloredKeyBehavior,
};

use super::render_editor::{
    render_boolean_toggle, render_numeric_editor, render_path_editor, render_signed_numeric_editor,
    render_string_editor,
};
use super::render_selector::{
    render_combo_action_selector, render_hold_mode_selector, render_idle_effect_mode_selector,
    render_key_action_palette_selector, render_key_position_selector,
    render_output_format_selector, render_palette_fx_effect_selector,
    render_palette_fx_palette_selector, render_ripple_color_mode_selector,
    render_tap_hold_preset_selector, render_theme_mode_selector,
};
use super::{ManagerMode, SettingGroup, SettingItem, SettingsManagerState};
use crate::tui::{popup_border_style, popup_title, PopupType, Theme};

/// Render the settings manager dialog
pub(super) fn render_settings_manager(
    f: &mut Frame,
    area: Rect,
    state: &SettingsManagerState,
    rgb_enabled: bool,
    rgb_brightness: RgbBrightness,
    rgb_timeout_ms: u32,
    uncolored_key_behavior: UncoloredKeyBehavior,
    idle_effect_settings: &IdleEffectSettings,
    overlay_ripple_settings: &RgbOverlayRippleSettings,
    tap_hold_settings: &TapHoldSettings,
    config: &crate::config::Config,
    layout: &crate::models::Layout,
    theme: &Theme,
) {
    // Center the dialog (80% width, 80% height)
    let dialog_width = (area.width * 80) / 100;
    let dialog_height = (area.height * 80) / 100;
    let dialog_x = (area.width - dialog_width) / 2;
    let dialog_y = (area.height - dialog_height) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the background area first
    f.render_widget(Clear, dialog_area);

    // Background block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(popup_border_style(&PopupType::SettingsManager, theme))
        .title(popup_title(&PopupType::SettingsManager, "Shift+S"))
        .style(Style::default().bg(theme.background));

    f.render_widget(block, dialog_area);

    // Inner area for content
    let inner_area = Rect {
        x: dialog_area.x + 2,
        y: dialog_area.y + 1,
        width: dialog_area.width.saturating_sub(4),
        height: dialog_area.height.saturating_sub(2),
    };

    match &state.mode {
        ManagerMode::Browsing => {
            render_settings_list(
                f,
                inner_area,
                state,
                rgb_enabled,
                rgb_brightness,
                rgb_timeout_ms,
                uncolored_key_behavior,
                idle_effect_settings,
                overlay_ripple_settings,
                tap_hold_settings,
                config,
                layout,
                theme,
            );
        }
        ManagerMode::SelectingTapHoldPreset { selected_option } => {
            render_tap_hold_preset_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::SelectingHoldMode { selected_option } => {
            render_hold_mode_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::EditingNumeric {
            setting,
            value,
            min,
            max,
            default,
        } => {
            render_numeric_editor(f, inner_area, *setting, value, *min, *max, *default, theme);
        }
        ManagerMode::EditingSignedNumeric {
            setting,
            value,
            min,
            max,
            default,
        } => {
            render_signed_numeric_editor(
                f, inner_area, *setting, value, *min, *max, *default, theme,
            );
        }
        ManagerMode::TogglingBoolean { setting, value } => {
            render_boolean_toggle(f, inner_area, *setting, *value, theme);
        }
        ManagerMode::EditingString { setting, value } => {
            render_string_editor(f, inner_area, *setting, value, theme);
        }
        ManagerMode::SelectingOutputFormat { selected_option } => {
            render_output_format_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::SelectingThemeMode { selected_option } => {
            render_theme_mode_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::EditingPath { setting, value } => {
            render_path_editor(f, inner_area, *setting, value, theme);
        }
        ManagerMode::SelectingIdleEffectMode { selected_option } => {
            render_idle_effect_mode_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::SelectingRippleColorMode { selected_option } => {
            render_ripple_color_mode_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::SelectingPaletteFxEffect { selected_option } => {
            render_palette_fx_effect_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::SelectingPaletteFxPalette { selected_option } => {
            render_palette_fx_palette_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::SelectingKeyActionPalette { selected_option } => {
            render_key_action_palette_selector(f, inner_area, *selected_option, theme);
        }
        ManagerMode::SelectingKeyPosition {
            setting,
            instruction,
        } => {
            render_key_position_selector(f, inner_area, *setting, instruction, theme);
        }
        ManagerMode::SelectingAction { idx, current } => {
            render_combo_action_selector(f, inner_area, *idx, current, theme);
        }
    }
}

/// Render the list of settings
fn render_settings_list(
    f: &mut Frame,
    area: Rect,
    state: &SettingsManagerState,
    rgb_enabled: bool,
    rgb_brightness: RgbBrightness,
    rgb_timeout_ms: u32,
    uncolored_key_behavior: UncoloredKeyBehavior,
    idle_effect_settings: &IdleEffectSettings,
    overlay_ripple_settings: &RgbOverlayRippleSettings,
    tap_hold_settings: &TapHoldSettings,
    config: &crate::config::Config,
    layout: &crate::models::Layout,
    theme: &Theme,
) {
    // Split area for task summary, list and help text
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Task summary
            Constraint::Min(5),    // Settings list
            Constraint::Length(5), // Help text
        ])
        .split(area);

    let selected_setting = SettingItem::all(layout).get(state.selected).copied();
    let selected_group = selected_setting.map(|setting| setting.group());
    let subgroup_summary = selected_setting
        .and_then(SettingItem::rgb_subgroup)
        .map(|subgroup| format!(" • Subsection: {}", subgroup.display_name()));
    let summary = selected_group.map_or_else(
        || "Choose a setting to edit.".to_string(),
        |group| {
            let scope = if group.is_global() {
                "Saved in config.toml"
            } else {
                "Saved in current layout"
            };
            format!(
                "Task area: {} • {}{}",
                group.display_name(),
                scope,
                subgroup_summary.unwrap_or_default()
            )
        },
    );
    let summary_widget = Paragraph::new(summary).block(
        Block::default()
            .borders(Borders::ALL)
            .title("What this section controls")
            .style(Style::default().bg(theme.background)),
    );
    f.render_widget(summary_widget, chunks[0]);

    // Build settings list with group headers
    let settings = SettingItem::all(layout);
    let selected_desc = settings
        .get(state.selected)
        .map_or_else(String::new, SettingItem::description);
    let mut items: Vec<ListItem> = Vec::new();
    let mut current_group: Option<SettingGroup> = None;
    let mut current_rgb_subgroup: Option<super::RgbSubgroup> = None;
    let mut display_index = 0;

    for setting in settings {
        // Add group header if group changes
        let group = setting.group();
        if current_group != Some(group) {
            if current_group.is_some() {
                // Add spacing between groups
                items.push(ListItem::new(Line::from("")));
            }
            items.push(ListItem::new(Line::from(vec![Span::styled(
                format!("── {} ──", group.display_name()),
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )])));
            current_group = Some(group);
            current_rgb_subgroup = None;
        }

        if group == SettingGroup::Rgb {
            let subgroup = setting.rgb_subgroup();
            if subgroup != current_rgb_subgroup {
                if current_rgb_subgroup.is_some() {
                    items.push(ListItem::new(Line::from("")));
                }

                if let Some(subgroup) = subgroup {
                    items.push(ListItem::new(Line::from(vec![Span::styled(
                        format!("  {}", subgroup.display_name()),
                        Style::default()
                            .fg(theme.text_secondary)
                            .add_modifier(Modifier::BOLD),
                    )])));
                }

                current_rgb_subgroup = subgroup;
            }
        }

        let style = if display_index == state.selected {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text)
        };

        // Get current value for this setting
        let value = get_setting_value_display(
            setting,
            rgb_enabled,
            rgb_brightness,
            rgb_timeout_ms,
            uncolored_key_behavior,
            idle_effect_settings,
            overlay_ripple_settings,
            tap_hold_settings,
            config,
            Some(layout),
        );

        let marker = if display_index == state.selected {
            "▶ "
        } else {
            "  "
        };

        // Show indicator for global settings (stored in config.toml)
        let scope_indicator = if setting.is_global() {
            Span::styled("[G] ", Style::default().fg(theme.text_muted))
        } else {
            Span::styled("[L] ", Style::default().fg(theme.text_muted))
        };

        let content = Line::from(vec![
            Span::styled(marker, Style::default().fg(theme.primary)),
            scope_indicator,
            Span::styled(setting.display_name(), style),
            Span::styled(": ", Style::default().fg(theme.text_muted)),
            Span::styled(value, Style::default().fg(theme.success)),
        ]);

        items.push(ListItem::new(content));
        display_index += 1;
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Settings"))
        .highlight_style(Style::default().bg(theme.surface));

    f.render_widget(list, chunks[1]);

    // Show description of selected setting (computed before consuming settings)
    let description = selected_desc;

    // Render help text
    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            description,
            Style::default().fg(theme.text_muted),
        )]),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(theme.primary)),
            Span::raw(": Navigate  "),
            Span::styled("Enter", Style::default().fg(theme.primary)),
            Span::raw(": Change  "),
            Span::styled("Esc", Style::default().fg(theme.primary)),
            Span::raw(": Close"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Left);

    f.render_widget(help, chunks[2]);
}

/// Get display string for a setting value
pub(super) fn get_setting_value_display(
    setting: SettingItem,
    rgb_enabled: bool,
    rgb_brightness: RgbBrightness,
    rgb_timeout_ms: u32,
    uncolored_key_behavior: UncoloredKeyBehavior,
    idle_effect_settings: &IdleEffectSettings,
    overlay_ripple_settings: &RgbOverlayRippleSettings,
    tap_hold: &TapHoldSettings,
    config: &crate::config::Config,
    layout: Option<&crate::models::Layout>,
) -> String {
    match setting {
        // Global: Paths
        SettingItem::QmkFirmwarePath => config
            .paths
            .qmk_firmware
            .as_ref()
            .map_or_else(|| "<not set>".to_string(), |p| p.display().to_string()),
        // Per-Layout: Build settings (now in layout metadata)
        SettingItem::Keyboard => layout
            .as_ref()
            .and_then(|l| l.metadata.keyboard.clone())
            .unwrap_or_else(|| "<not set>".to_string()),
        SettingItem::LayoutVariant => layout
            .as_ref()
            .and_then(|l| l.metadata.layout_variant.clone())
            .unwrap_or_else(|| "<not set>".to_string()),
        SettingItem::KeymapName => layout
            .as_ref()
            .and_then(|l| l.metadata.keymap_name.clone())
            .unwrap_or_else(|| "<not set>".to_string()),
        SettingItem::OutputFormat => layout
            .as_ref()
            .and_then(|l| l.metadata.output_format.clone())
            .unwrap_or_else(|| "<not set>".to_string()),
        SettingItem::OutputDir => config.build.output_dir.display().to_string(),
        // Global: UI
        SettingItem::ShowHelpOnStartup => if config.ui.show_help_on_startup {
            "On"
        } else {
            "Off"
        }
        .to_string(),
        SettingItem::ThemeMode => match config.ui.theme_mode {
            crate::config::ThemeMode::Auto => "Auto".to_string(),
            crate::config::ThemeMode::Dark => "Dark".to_string(),
            crate::config::ThemeMode::Light => "Light".to_string(),
        },
        SettingItem::KeyboardScale => format!("{:.0}%", config.ui.keyboard_scale * 100.0),
        // Per-Layout: RGB
        SettingItem::RgbEnabled => if rgb_enabled { "On" } else { "Off" }.to_string(),
        SettingItem::RgbBrightness => format!("{}%", rgb_brightness.as_percent()),
        SettingItem::RgbSaturation => {
            let saturation = layout
                .as_ref()
                .map(|l| l.rgb_saturation.as_percent())
                .unwrap_or(100);
            format!("{}%", saturation)
        }
        SettingItem::RgbMatrixSpeed => {
            let speed = layout
                .as_ref()
                .map(|l| l.rgb_matrix_default_speed)
                .unwrap_or(127);
            format!("{}", speed)
        }
        SettingItem::RgbTimeout => {
            if rgb_timeout_ms == 0 {
                "Disabled".to_string()
            } else if rgb_timeout_ms >= 60000 && rgb_timeout_ms.is_multiple_of(60000) {
                format!("{} min", rgb_timeout_ms / 60000)
            } else if rgb_timeout_ms >= 1000 && rgb_timeout_ms.is_multiple_of(1000) {
                format!("{} sec", rgb_timeout_ms / 1000)
            } else {
                format!("{rgb_timeout_ms}ms")
            }
        }
        SettingItem::IdleEffectEnabled => if idle_effect_settings.enabled {
            "On"
        } else {
            "Off"
        }
        .to_string(),
        SettingItem::IdleTimeout => {
            let idle_timeout_ms = idle_effect_settings.idle_timeout_ms;
            if idle_timeout_ms == 0 {
                "Disabled".to_string()
            } else if idle_timeout_ms >= 60000 && idle_timeout_ms.is_multiple_of(60000) {
                format!("{} min", idle_timeout_ms / 60000)
            } else if idle_timeout_ms >= 1000 && idle_timeout_ms.is_multiple_of(1000) {
                format!("{} sec", idle_timeout_ms / 1000)
            } else {
                format!("{idle_timeout_ms}ms")
            }
        }
        SettingItem::IdleEffectDuration => {
            let duration_ms = idle_effect_settings.idle_effect_duration_ms;
            if duration_ms == 0 {
                "Disabled".to_string()
            } else if duration_ms >= 60000 && duration_ms.is_multiple_of(60000) {
                format!("{} min", duration_ms / 60000)
            } else if duration_ms >= 1000 && duration_ms.is_multiple_of(1000) {
                format!("{} sec", duration_ms / 1000)
            } else {
                format!("{duration_ms}ms")
            }
        }
        SettingItem::IdleEffectMode => idle_effect_settings
            .idle_effect_mode
            .display_name()
            .to_string(),
        SettingItem::UncoloredKeyBehavior => format!("{}%", uncolored_key_behavior.as_percent()),
        // Per-Layout: Overlay Ripple
        SettingItem::OverlayRippleEnabled => if overlay_ripple_settings.enabled {
            "On"
        } else {
            "Off"
        }
        .to_string(),
        SettingItem::OverlayRippleMaxRipples => format!("{}", overlay_ripple_settings.max_ripples),
        SettingItem::OverlayRippleDuration => format!("{}ms", overlay_ripple_settings.duration_ms),
        SettingItem::OverlayRippleSpeed => format!("{}", overlay_ripple_settings.speed),
        SettingItem::OverlayRippleBandWidth => format!("{}", overlay_ripple_settings.band_width),
        SettingItem::OverlayRippleAmplitude => {
            format!("{}%", overlay_ripple_settings.amplitude_pct)
        }
        SettingItem::OverlayRippleColorMode => overlay_ripple_settings
            .color_mode
            .display_name()
            .to_string(),
        SettingItem::OverlayRippleFixedColor => overlay_ripple_settings.fixed_color.to_hex(),
        SettingItem::OverlayRippleHueShift => format!("{}°", overlay_ripple_settings.hue_shift_deg),
        SettingItem::OverlayRippleTriggerPress => if overlay_ripple_settings.trigger_on_press {
            "On"
        } else {
            "Off"
        }
        .to_string(),
        SettingItem::OverlayRippleTriggerRelease => if overlay_ripple_settings.trigger_on_release {
            "On"
        } else {
            "Off"
        }
        .to_string(),
        SettingItem::OverlayRippleIgnoreTransparent => {
            if overlay_ripple_settings.ignore_transparent {
                "On"
            } else {
                "Off"
            }
            .to_string()
        }
        SettingItem::OverlayRippleIgnoreModifiers => if overlay_ripple_settings.ignore_modifiers {
            "On"
        } else {
            "Off"
        }
        .to_string(),
        SettingItem::OverlayRippleIgnoreLayerSwitch => {
            if overlay_ripple_settings.ignore_layer_switch {
                "On"
            } else {
                "Off"
            }
            .to_string()
        }
        SettingItem::OverlayRippleKeyActionPalette => overlay_ripple_settings
            .key_action_palette
            .map_or_else(|| "Default".to_string(), |p| p.display_name().to_string()),
        SettingItem::OverlayRippleWaveCount => format!("{}", overlay_ripple_settings.wave_count),
        SettingItem::OverlayRippleWaveDelay => {
            format!("{}ms", overlay_ripple_settings.wave_delay_ms)
        }
        // Per-Layout: Tap-Hold
        SettingItem::TapHoldPreset => tap_hold.preset.display_name().to_string(),
        SettingItem::TappingTerm => format!("{}ms", tap_hold.tapping_term),
        SettingItem::QuickTapTerm => match tap_hold.quick_tap_term {
            Some(term) => format!("{term}ms"),
            None => "Auto".to_string(),
        },
        SettingItem::HoldMode => tap_hold.hold_mode.display_name().to_string(),
        SettingItem::RetroTapping => if tap_hold.retro_tapping { "On" } else { "Off" }.to_string(),
        SettingItem::TappingToggle => format!("{} taps", tap_hold.tapping_toggle),
        SettingItem::FlowTapTerm => match tap_hold.flow_tap_term {
            Some(term) => format!("{term}ms"),
            None => "Disabled".to_string(),
        },
        SettingItem::ChordalHold => if tap_hold.chordal_hold { "On" } else { "Off" }.to_string(),
        // Per-Layout: Combo Settings
        SettingItem::CombosEnabled => {
            if let Some(layout) = layout {
                if layout.combo_settings.enabled {
                    "On"
                } else {
                    "Off"
                }
                .to_string()
            } else {
                "Off".to_string()
            }
        }
        SettingItem::AddCombo => "<add new combo>".to_string(),
        SettingItem::RemoveCombo(idx) => layout
            .and_then(|l| l.combo_settings.combos.get(idx))
            .map_or_else(
                || "<not set>".to_string(),
                |_| "<press Enter to remove>".to_string(),
            ),
        SettingItem::ComboKey1(idx) => layout
            .and_then(|l| l.combo_settings.combos.get(idx))
            .map_or_else(
                || "<not set>".to_string(),
                |c| format!("({}, {})", c.key1.row, c.key1.col),
            ),
        SettingItem::ComboKey2(idx) => layout
            .and_then(|l| l.combo_settings.combos.get(idx))
            .map_or_else(
                || "<not set>".to_string(),
                |c| format!("({}, {})", c.key2.row, c.key2.col),
            ),
        SettingItem::ComboHoldDuration(idx) => layout
            .and_then(|l| l.combo_settings.combos.get(idx))
            .map_or_else(
                || "500ms".to_string(),
                |c| format!("{}ms", c.hold_duration_ms),
            ),
        SettingItem::ComboAction(idx) => layout
            .and_then(|l| l.combo_settings.combos.get(idx))
            .map_or_else(
                || "<not set>".to_string(),
                |c| c.action.display_name().to_string(),
            ),
        // Per-Layout: PaletteFX
        SettingItem::PaletteFxEnabled => layout
            .map(|l| if l.palette_fx.enabled { "On" } else { "Off" })
            .unwrap_or("Off")
            .to_string(),
        SettingItem::PaletteFxDefaultEffect => layout
            .map(|l| l.palette_fx.default_effect.display_name().to_string())
            .unwrap_or_default(),
        SettingItem::PaletteFxDefaultPalette => layout
            .map(|l| l.palette_fx.default_palette.display_name().to_string())
            .unwrap_or_default(),
        SettingItem::PaletteFxEnableAllEffects => layout
            .map(|l| {
                if l.palette_fx.enable_all_effects {
                    "On"
                } else {
                    "Off"
                }
            })
            .unwrap_or("Off")
            .to_string(),
        SettingItem::PaletteFxEnableAllPalettes => layout
            .map(|l| {
                if l.palette_fx.enable_all_palettes {
                    "On"
                } else {
                    "Off"
                }
            })
            .unwrap_or("Off")
            .to_string(),
    }
}
