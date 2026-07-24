//! Tests for the settings manager.

use crate::models::{
    IdleEffectSettings, RgbBrightness, RgbMatrixEffect, RgbOverlayRippleSettings, TapHoldSettings,
    UncoloredKeyBehavior,
};

use super::render_main::get_setting_value_display;
use super::{ManagerMode, SettingItem, SettingsManagerState};

#[test]
fn test_numeric_editor_reset_to_default() {
    let mut state = SettingsManagerState::new();
    state.start_editing_numeric(SettingItem::TappingTerm, 275, 100, 500, 200);

    state.reset_numeric_to_default();

    assert_eq!(state.get_numeric_value(), Some(200));
}

#[test]
fn test_signed_numeric_editor_reset_to_default() {
    let mut state = SettingsManagerState::new();
    state.start_editing_signed_numeric(SettingItem::OverlayRippleHueShift, -45, -180, 180, 60);

    state.reset_signed_numeric_to_default();

    assert_eq!(state.get_signed_numeric_value(), Some(60));
}

#[test]
fn test_start_editing_numeric_stores_default() {
    let mut state = SettingsManagerState::new();
    state.start_editing_numeric(SettingItem::TappingToggle, 7, 1, 10, 5);

    assert!(matches!(
        state.mode,
        ManagerMode::EditingNumeric { default: 5, .. }
    ));
}

#[test]
fn test_setting_item_all_includes_idle_effect_settings() {
    let layout = crate::models::Layout::new("test").unwrap();
    let all_settings = SettingItem::all(&layout);

    // Verify idle effect settings are present
    assert!(all_settings.contains(&SettingItem::IdleEffectEnabled));
    assert!(all_settings.contains(&SettingItem::IdleTimeout));
    assert!(all_settings.contains(&SettingItem::IdleEffectDuration));
    assert!(all_settings.contains(&SettingItem::IdleEffectMode));
}

#[test]
fn test_idle_effect_settings_belong_to_rgb_group() {
    assert_eq!(
        SettingItem::IdleEffectEnabled.group(),
        super::SettingGroup::Rgb
    );
    assert_eq!(SettingItem::IdleTimeout.group(), super::SettingGroup::Rgb);
    assert_eq!(
        SettingItem::IdleEffectDuration.group(),
        super::SettingGroup::Rgb
    );
    assert_eq!(
        SettingItem::IdleEffectMode.group(),
        super::SettingGroup::Rgb
    );
}

#[test]
fn test_idle_effect_settings_have_display_names() {
    assert_eq!(
        SettingItem::IdleEffectEnabled.display_name(),
        "Idle Lighting Enabled"
    );
    assert_eq!(SettingItem::IdleTimeout.display_name(), "Idle Wait Time");
    assert_eq!(
        SettingItem::IdleEffectDuration.display_name(),
        "Idle Effect Length"
    );
    assert_eq!(SettingItem::IdleEffectMode.display_name(), "Idle Effect");
}

#[test]
fn test_idle_effect_settings_have_descriptions() {
    let desc = SettingItem::IdleEffectEnabled.description();
    assert!(!desc.is_empty());
    assert!(desc.contains("idle"));

    let desc = SettingItem::IdleTimeout.description();
    assert!(!desc.is_empty());

    let desc = SettingItem::IdleEffectDuration.description();
    assert!(!desc.is_empty());

    let desc = SettingItem::IdleEffectMode.description();
    assert!(!desc.is_empty());
}

#[test]
fn test_get_setting_value_display_idle_effect_enabled() {
    let idle_settings = IdleEffectSettings {
        enabled: true,
        ..Default::default()
    };

    let display = get_setting_value_display(
        SettingItem::IdleEffectEnabled,
        true,
        RgbBrightness::from(100),
        0,
        UncoloredKeyBehavior::from(100),
        &idle_settings,
        &RgbOverlayRippleSettings::default(),
        &TapHoldSettings::default(),
        &crate::config::Config::default(),
        None,
    );

    assert_eq!(display, "On");

    let idle_settings = IdleEffectSettings {
        enabled: false,
        ..Default::default()
    };

    let display = get_setting_value_display(
        SettingItem::IdleEffectEnabled,
        true,
        RgbBrightness::from(100),
        0,
        UncoloredKeyBehavior::from(100),
        &idle_settings,
        &RgbOverlayRippleSettings::default(),
        &TapHoldSettings::default(),
        &crate::config::Config::default(),
        None,
    );

    assert_eq!(display, "Off");
}

#[test]
fn test_get_setting_value_display_idle_timeout() {
    let idle_settings = IdleEffectSettings {
        idle_timeout_ms: 0,
        ..Default::default()
    };

    let display = get_setting_value_display(
        SettingItem::IdleTimeout,
        true,
        RgbBrightness::from(100),
        0,
        UncoloredKeyBehavior::from(100),
        &idle_settings,
        &RgbOverlayRippleSettings::default(),
        &TapHoldSettings::default(),
        &crate::config::Config::default(),
        None,
    );

    assert_eq!(display, "Disabled");

    let idle_settings = IdleEffectSettings {
        idle_timeout_ms: 60_000,
        ..Default::default()
    };

    let display = get_setting_value_display(
        SettingItem::IdleTimeout,
        true,
        RgbBrightness::from(100),
        0,
        UncoloredKeyBehavior::from(100),
        &idle_settings,
        &RgbOverlayRippleSettings::default(),
        &TapHoldSettings::default(),
        &crate::config::Config::default(),
        None,
    );

    assert_eq!(display, "1 min");

    let idle_settings = IdleEffectSettings {
        idle_timeout_ms: 30_000,
        ..Default::default()
    };

    let display = get_setting_value_display(
        SettingItem::IdleTimeout,
        true,
        RgbBrightness::from(100),
        0,
        UncoloredKeyBehavior::from(100),
        &idle_settings,
        &RgbOverlayRippleSettings::default(),
        &TapHoldSettings::default(),
        &crate::config::Config::default(),
        None,
    );

    assert_eq!(display, "30 sec");
}

#[test]
fn test_get_setting_value_display_idle_effect_mode() {
    let idle_settings = IdleEffectSettings {
        idle_effect_mode: RgbMatrixEffect::Breathing,
        ..Default::default()
    };

    let display = get_setting_value_display(
        SettingItem::IdleEffectMode,
        true,
        RgbBrightness::from(100),
        0,
        UncoloredKeyBehavior::from(100),
        &idle_settings,
        &RgbOverlayRippleSettings::default(),
        &TapHoldSettings::default(),
        &crate::config::Config::default(),
        None,
    );

    assert_eq!(display, "Breathing");

    let idle_settings = IdleEffectSettings {
        idle_effect_mode: RgbMatrixEffect::RainbowBeacon,
        ..Default::default()
    };

    let display = get_setting_value_display(
        SettingItem::IdleEffectMode,
        true,
        RgbBrightness::from(100),
        0,
        UncoloredKeyBehavior::from(100),
        &idle_settings,
        &RgbOverlayRippleSettings::default(),
        &TapHoldSettings::default(),
        &crate::config::Config::default(),
        None,
    );

    assert_eq!(display, "Rainbow Beacon");
}
