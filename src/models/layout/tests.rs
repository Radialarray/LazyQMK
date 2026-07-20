//! Tests for all layout types.

use super::*;
use crate::models::layer::{KeyDefinition, Layer, Position};
use crate::models::{Category, RgbColor};

#[test]
fn test_layout_metadata_new() {
    let metadata = LayoutMetadata::new("Test Layout").unwrap();
    assert_eq!(metadata.name, "Test Layout");
    assert!(metadata.description.is_empty());
    assert!(metadata.author.is_empty());
    assert!(metadata.tags.is_empty());
    assert!(!metadata.is_template);
    assert_eq!(metadata.version, "1.0");
}

#[test]
fn test_layout_metadata_validate_name() {
    assert!(LayoutMetadata::new("Valid Name").is_ok());
    assert!(LayoutMetadata::new("").is_err());
    assert!(LayoutMetadata::new("a".repeat(101)).is_err());
}

#[test]
fn test_layout_metadata_add_tag() {
    let mut metadata = LayoutMetadata::new("Test").unwrap();
    metadata.add_tag("programming").unwrap();
    metadata.add_tag("vim").unwrap();

    assert_eq!(metadata.tags, vec!["programming", "vim"]);

    // Duplicate tag should not be added
    metadata.add_tag("programming").unwrap();
    assert_eq!(metadata.tags, vec!["programming", "vim"]);
}

#[test]
fn test_layout_new() {
    let layout = Layout::new("Test Layout").unwrap();
    assert_eq!(layout.metadata.name, "Test Layout");
    assert!(layout.layers.is_empty());
    assert!(layout.categories.is_empty());
}

#[test]
fn test_layout_add_layer() {
    let mut layout = Layout::new("Test").unwrap();
    let layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();

    assert!(layout.add_layer(layer0).is_ok());
    assert!(layout.add_layer(layer1).is_ok());
    assert_eq!(layout.layers.len(), 2);
}

#[test]
fn test_layout_add_layer_sequential_validation() {
    let mut layout = Layout::new("Test").unwrap();
    let layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    let layer2 = Layer::new(2, "Skip", RgbColor::new(0, 255, 0)).unwrap();

    assert!(layout.add_layer(layer0).is_ok());
    assert!(layout.add_layer(layer2).is_err()); // Should fail - not sequential
}

#[test]
fn test_layout_add_category() {
    let mut layout = Layout::new("Test").unwrap();
    let category = Category::new("navigation", "Navigation", RgbColor::new(0, 255, 0)).unwrap();

    assert!(layout.add_category(category).is_ok());
    assert_eq!(layout.categories.len(), 1);
}

#[test]
fn test_layout_add_category_duplicate() {
    let mut layout = Layout::new("Test").unwrap();
    let category1 = Category::new("navigation", "Navigation", RgbColor::new(0, 255, 0)).unwrap();
    let category2 = Category::new("navigation", "Nav Keys", RgbColor::new(255, 0, 0)).unwrap();

    assert!(layout.add_category(category1).is_ok());
    assert!(layout.add_category(category2).is_err()); // Duplicate ID
}

#[test]
fn test_ripple_settings_validate_rejects_zero_duration() {
    let mut settings = RgbOverlayRippleSettings::default();
    settings.duration_ms = 0;

    assert!(settings.validate().is_err());
}

#[test]
fn test_ripple_settings_validate_rejects_zero_band_width() {
    let mut settings = RgbOverlayRippleSettings::default();
    settings.band_width = 0;

    assert!(settings.validate().is_err());
}

#[test]
fn test_ripple_settings_validate_rejects_zero_speed() {
    let mut settings = RgbOverlayRippleSettings::default();
    settings.speed = 0;

    assert!(settings.validate().is_err());
}

#[test]
fn test_layout_validate_rejects_invalid_ripple_settings() {
    let mut layout = Layout::new("Test").unwrap();
    let mut layer = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_A"));
    layout.add_layer(layer).unwrap();
    layout.rgb_overlay_ripple.duration_ms = 0;

    assert!(layout.validate().is_err());
}

#[test]
fn test_layout_resolve_key_color() {
    let mut layout = Layout::new("Test").unwrap();
    let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();

    // Test 1: Individual override (highest priority)
    let key_with_override =
        KeyDefinition::new(Position::new(0, 0), "KC_A").with_color(RgbColor::new(255, 0, 0));
    layer.add_key(key_with_override.clone());

    layout.add_layer(layer).unwrap();

    let color = layout.resolve_key_color(0, &key_with_override);
    assert_eq!(color, RgbColor::new(255, 0, 0));

    // Test 2: Key category (second priority)
    let category = Category::new("navigation", "Navigation", RgbColor::new(0, 255, 0)).unwrap();
    layout.add_category(category).unwrap();

    let key_with_category =
        KeyDefinition::new(Position::new(0, 1), "KC_B").with_category("navigation");
    layout
        .get_layer_mut(0)
        .unwrap()
        .add_key(key_with_category.clone());

    let color = layout.resolve_key_color(0, &key_with_category);
    assert_eq!(color, RgbColor::new(0, 255, 0));

    // Test 3: Layer default (fallback)
    let key_default = KeyDefinition::new(Position::new(0, 2), "KC_C");
    layout
        .get_layer_mut(0)
        .unwrap()
        .add_key(key_default.clone());

    let color = layout.resolve_key_color(0, &key_default);
    assert_eq!(color, RgbColor::new(255, 255, 255));
}

#[test]
fn test_layout_validate() {
    let mut layout = Layout::new("Test").unwrap();

    // Empty layout should fail
    assert!(layout.validate().is_err());

    // Add a layer with keys
    let mut layer = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_A"));
    layer.add_key(KeyDefinition::new(Position::new(0, 1), "KC_B"));
    layout.add_layer(layer).unwrap();

    // Should pass now
    assert!(layout.validate().is_ok());

    // Add another layer with different key count
    let mut layer2 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
    layer2.add_key(KeyDefinition::new(Position::new(0, 0), "KC_1"));
    layout.add_layer(layer2).unwrap();

    // Should fail - mismatched key counts
    assert!(layout.validate().is_err());
}

// === Tap-Hold Settings Tests ===

#[test]
fn test_tap_hold_settings_default() {
    let settings = TapHoldSettings::default();
    assert_eq!(settings.tapping_term, 200);
    assert_eq!(settings.quick_tap_term, None);
    assert_eq!(settings.hold_mode, HoldDecisionMode::Default);
    assert!(!settings.retro_tapping);
    assert_eq!(settings.tapping_toggle, 5);
    assert_eq!(settings.flow_tap_term, None);
    assert!(!settings.chordal_hold);
    assert_eq!(settings.preset, TapHoldPreset::Default);
}

#[test]
fn test_tap_hold_preset_home_row_mods() {
    let settings = TapHoldPreset::HomeRowMods.settings();
    assert_eq!(settings.tapping_term, 175);
    assert_eq!(settings.quick_tap_term, Some(120));
    assert_eq!(settings.hold_mode, HoldDecisionMode::PermissiveHold);
    assert!(settings.retro_tapping);
    assert_eq!(settings.flow_tap_term, Some(150));
    assert!(settings.chordal_hold);
    assert_eq!(settings.preset, TapHoldPreset::HomeRowMods);
}

#[test]
fn test_tap_hold_preset_responsive() {
    let settings = TapHoldPreset::Responsive.settings();
    assert_eq!(settings.tapping_term, 150);
    assert_eq!(settings.quick_tap_term, Some(100));
    assert_eq!(settings.hold_mode, HoldDecisionMode::HoldOnOtherKeyPress);
    assert!(!settings.retro_tapping);
    assert_eq!(settings.flow_tap_term, None);
    assert!(!settings.chordal_hold);
}

#[test]
fn test_tap_hold_preset_deliberate() {
    let settings = TapHoldPreset::Deliberate.settings();
    assert_eq!(settings.tapping_term, 250);
    assert_eq!(settings.quick_tap_term, None);
    assert_eq!(settings.hold_mode, HoldDecisionMode::Default);
}

#[test]
fn test_tap_hold_settings_apply_preset() {
    let mut settings = TapHoldSettings::default();
    settings.apply_preset(TapHoldPreset::HomeRowMods);

    assert_eq!(settings.tapping_term, 175);
    assert!(settings.chordal_hold);
    assert_eq!(settings.preset, TapHoldPreset::HomeRowMods);
}

#[test]
fn test_tap_hold_settings_mark_custom() {
    let mut settings = TapHoldPreset::HomeRowMods.settings();
    assert_eq!(settings.preset, TapHoldPreset::HomeRowMods);

    settings.mark_custom();
    assert_eq!(settings.preset, TapHoldPreset::Custom);
}

#[test]
fn test_tap_hold_settings_has_custom_settings() {
    let default_settings = TapHoldSettings::default();
    assert!(!default_settings.has_custom_settings());

    let custom = TapHoldSettings {
        tapping_term: 180,
        ..TapHoldSettings::default()
    };
    assert!(custom.has_custom_settings());

    let with_retro = TapHoldSettings {
        retro_tapping: true,
        ..TapHoldSettings::default()
    };
    assert!(with_retro.has_custom_settings());
}

#[test]
fn test_tap_hold_settings_validation() {
    let mut settings = TapHoldSettings::default();
    assert!(settings.validate().is_ok());

    // Invalid tapping term (too low)
    settings.tapping_term = 10;
    assert!(settings.validate().is_err());

    // Invalid tapping term (too high)
    settings.tapping_term = 2000;
    assert!(settings.validate().is_err());

    // Valid tapping term
    settings.tapping_term = 200;
    assert!(settings.validate().is_ok());

    // Invalid tapping toggle
    settings.tapping_toggle = 0;
    assert!(settings.validate().is_err());

    settings.tapping_toggle = 5;
    assert!(settings.validate().is_ok());
}

#[test]
fn test_hold_decision_mode_config_define() {
    assert_eq!(HoldDecisionMode::Default.config_define(), None);
    assert_eq!(
        HoldDecisionMode::PermissiveHold.config_define(),
        Some("PERMISSIVE_HOLD")
    );
    assert_eq!(
        HoldDecisionMode::HoldOnOtherKeyPress.config_define(),
        Some("HOLD_ON_OTHER_KEY_PRESS")
    );
}

// === RGB Saturation Tests ===

#[test]
fn test_rgb_saturation_new() {
    let sat = RgbSaturation::new(0);
    assert_eq!(sat.as_percent(), 0);

    let sat = RgbSaturation::new(100);
    assert_eq!(sat.as_percent(), 100);

    let sat = RgbSaturation::new(200);
    assert_eq!(sat.as_percent(), 200);
}

#[test]
#[should_panic(expected = "Saturation must be 0-200")]
fn test_rgb_saturation_new_too_high() {
    let _ = RgbSaturation::new(201);
}

#[test]
fn test_rgb_saturation_default() {
    let sat = RgbSaturation::default();
    assert_eq!(sat.as_percent(), 100);
    assert_eq!(sat, RgbSaturation::NEUTRAL);
}

#[test]
fn test_rgb_saturation_from_u8() {
    let sat = RgbSaturation::from(50);
    assert_eq!(sat.as_percent(), 50);

    // Should clamp to 200
    let sat = RgbSaturation::from(255);
    assert_eq!(sat.as_percent(), 200);
}

#[test]
fn test_apply_rgb_settings_with_saturation() {
    let mut layout = Layout::new("Test").unwrap();

    // Test with saturation at neutral (100%)
    layout.rgb_saturation = RgbSaturation::new(100);
    layout.rgb_brightness = RgbBrightness::new(100);
    let color = RgbColor::new(200, 100, 50);
    let result = layout.apply_rgb_settings(color);
    assert_eq!(result, color); // Unchanged at 100% saturation and brightness

    // Test with reduced saturation (0% = grayscale)
    layout.rgb_saturation = RgbSaturation::new(0);
    let result = layout.apply_rgb_settings(color);
    // Should be grayscale (all channels equal)
    assert_eq!(result.r, result.g);
    assert_eq!(result.g, result.b);

    // Test with increased saturation (150%)
    layout.rgb_saturation = RgbSaturation::new(150);
    let result = layout.apply_rgb_settings(color);
    // Should be more saturated (more difference between channels)
    // The exact values depend on HSV conversion, just verify it's not the original
    // and channels are still different
    assert_ne!(result, color);
    assert_ne!(result.r, result.g);

    // Test order: saturation then brightness
    layout.rgb_saturation = RgbSaturation::new(50); // Half saturation
    layout.rgb_brightness = RgbBrightness::new(50); // Half brightness
    let result = layout.apply_rgb_settings(color);
    // Should be dimmed AND desaturated
    // With 50% saturation, color moves toward grayscale
    // Then 50% brightness dims everything
    // The total brightness should be reduced
    let original_brightness = u16::from(color.r) + u16::from(color.g) + u16::from(color.b);
    let result_brightness = u16::from(result.r) + u16::from(result.g) + u16::from(result.b);
    assert!(result_brightness < original_brightness);
}

#[test]
fn test_apply_rgb_settings_disabled() {
    let mut layout = Layout::new("Test").unwrap();
    layout.rgb_enabled = false;

    let color = RgbColor::new(200, 100, 50);
    let result = layout.apply_rgb_settings(color);

    // Should be black when RGB is disabled
    assert_eq!(result, RgbColor::new(0, 0, 0));
}

#[test]
fn test_layout_new_has_default_saturation() {
    let layout = Layout::new("Test").unwrap();
    assert_eq!(layout.rgb_saturation, RgbSaturation::NEUTRAL);
}

// === RGB Matrix Effect Tests ===

#[test]
fn test_rgb_matrix_effect_display_names() {
    assert_eq!(RgbMatrixEffect::Breathing.display_name(), "Breathing");
    assert_eq!(
        RgbMatrixEffect::RainbowMovingChevron.display_name(),
        "Rainbow Moving Chevron"
    );
    assert_eq!(RgbMatrixEffect::CycleAll.display_name(), "Cycle All");
}

#[test]
fn test_rgb_matrix_effect_from_name() {
    // Exact matches
    assert_eq!(
        RgbMatrixEffect::from_name("Breathing"),
        Some(RgbMatrixEffect::Breathing)
    );
    assert_eq!(
        RgbMatrixEffect::from_name("breathing"),
        Some(RgbMatrixEffect::Breathing)
    );

    // With spaces and underscores
    assert_eq!(
        RgbMatrixEffect::from_name("Rainbow Moving Chevron"),
        Some(RgbMatrixEffect::RainbowMovingChevron)
    );
    assert_eq!(
        RgbMatrixEffect::from_name("rainbow_moving_chevron"),
        Some(RgbMatrixEffect::RainbowMovingChevron)
    );

    // Short aliases
    assert_eq!(
        RgbMatrixEffect::from_name("breath"),
        Some(RgbMatrixEffect::Breathing)
    );
    assert_eq!(
        RgbMatrixEffect::from_name("chevron"),
        Some(RgbMatrixEffect::RainbowMovingChevron)
    );
    assert_eq!(
        RgbMatrixEffect::from_name("cycle"),
        Some(RgbMatrixEffect::CycleAll)
    );

    // Invalid name
    assert_eq!(RgbMatrixEffect::from_name("invalid_effect"), None);
}

#[test]
fn test_rgb_matrix_effect_default() {
    assert_eq!(RgbMatrixEffect::default(), RgbMatrixEffect::SolidColor);
}

// === Idle Effect Settings Tests ===

#[test]
fn test_idle_effect_settings_default() {
    let settings = IdleEffectSettings::default();
    assert!(settings.enabled);
    assert_eq!(settings.idle_timeout_ms, 60_000);
    assert_eq!(settings.idle_effect_duration_ms, 300_000);
    assert_eq!(settings.idle_effect_mode, RgbMatrixEffect::Breathing);
}

#[test]
fn test_idle_effect_settings_has_custom_settings() {
    let default_settings = IdleEffectSettings::default();
    assert!(!default_settings.has_custom_settings());

    // Test enabled change
    let custom = IdleEffectSettings {
        enabled: false,
        ..IdleEffectSettings::default()
    };
    assert!(custom.has_custom_settings());

    // Test timeout change
    let custom = IdleEffectSettings {
        idle_timeout_ms: 30_000,
        ..IdleEffectSettings::default()
    };
    assert!(custom.has_custom_settings());

    // Test duration change
    let custom = IdleEffectSettings {
        idle_effect_duration_ms: 600_000,
        ..IdleEffectSettings::default()
    };
    assert!(custom.has_custom_settings());

    // Test mode change
    let custom = IdleEffectSettings {
        idle_effect_mode: RgbMatrixEffect::RainbowBeacon,
        ..IdleEffectSettings::default()
    };
    assert!(custom.has_custom_settings());
}

#[test]
fn test_layout_new_has_default_idle_settings() {
    let layout = Layout::new("Test").unwrap();
    assert_eq!(layout.idle_effect_settings, IdleEffectSettings::default());
}

// === Combo Settings Tests ===

#[test]
fn test_combo_action_all() {
    let actions = ComboAction::all();
    assert_eq!(actions.len(), 3);
    assert!(actions.contains(&ComboAction::DisableEffects));
    assert!(actions.contains(&ComboAction::DisableLighting));
    assert!(actions.contains(&ComboAction::Bootloader));
}

#[test]
fn test_combo_action_display_name() {
    assert_eq!(
        ComboAction::DisableEffects.display_name(),
        "Disable Effects"
    );
    assert_eq!(
        ComboAction::DisableLighting.display_name(),
        "Disable Lighting"
    );
    assert_eq!(ComboAction::Bootloader.display_name(), "Bootloader");
}

#[test]
fn test_combo_action_from_name() {
    assert_eq!(
        ComboAction::from_name("disable effects"),
        Some(ComboAction::DisableEffects)
    );
    assert_eq!(
        ComboAction::from_name("DisableEffects"),
        Some(ComboAction::DisableEffects)
    );
    assert_eq!(
        ComboAction::from_name("lighting"),
        Some(ComboAction::DisableLighting)
    );
    assert_eq!(
        ComboAction::from_name("bootloader"),
        Some(ComboAction::Bootloader)
    );
    assert_eq!(ComboAction::from_name("invalid"), None);
}

#[test]
fn test_combo_definition_new() {
    let combo = ComboDefinition::new(
        Position::new(0, 0),
        Position::new(0, 1),
        ComboAction::DisableEffects,
    );
    assert_eq!(combo.key1, Position::new(0, 0));
    assert_eq!(combo.key2, Position::new(0, 1));
    assert_eq!(combo.action, ComboAction::DisableEffects);
    assert_eq!(combo.hold_duration_ms, 500);
}

#[test]
fn test_combo_definition_with_duration() {
    let combo = ComboDefinition::with_duration(
        Position::new(0, 0),
        Position::new(0, 1),
        ComboAction::Bootloader,
        1000,
    );
    assert_eq!(combo.hold_duration_ms, 1000);
}

#[test]
fn test_combo_definition_validate_same_keys() {
    let combo = ComboDefinition::new(
        Position::new(0, 0),
        Position::new(0, 0),
        ComboAction::DisableEffects,
    );
    assert!(combo.validate().is_err());
}

#[test]
fn test_combo_definition_validate_duration() {
    let combo = ComboDefinition::with_duration(
        Position::new(0, 0),
        Position::new(0, 1),
        ComboAction::DisableEffects,
        30, // Too short
    );
    assert!(combo.validate().is_err());

    let combo = ComboDefinition::with_duration(
        Position::new(0, 0),
        Position::new(0, 1),
        ComboAction::DisableEffects,
        3000, // Too long
    );
    assert!(combo.validate().is_err());

    let combo = ComboDefinition::with_duration(
        Position::new(0, 0),
        Position::new(0, 1),
        ComboAction::DisableEffects,
        500, // Valid
    );
    assert!(combo.validate().is_ok());
}

#[test]
fn test_combo_settings_default() {
    let settings = ComboSettings::default();
    assert!(!settings.enabled);
    assert!(settings.combos.is_empty());
    assert!(!settings.has_custom_settings());
}

#[test]
fn test_combo_settings_add_combo() {
    let mut settings = ComboSettings::new(true);

    let combo1 = ComboDefinition::new(
        Position::new(0, 0),
        Position::new(0, 1),
        ComboAction::DisableEffects,
    );
    assert!(settings.add_combo(combo1).is_ok());
    assert_eq!(settings.combos.len(), 1);

    let combo2 = ComboDefinition::new(
        Position::new(1, 0),
        Position::new(1, 1),
        ComboAction::DisableLighting,
    );
    assert!(settings.add_combo(combo2).is_ok());
    assert_eq!(settings.combos.len(), 2);

    let combo3 = ComboDefinition::new(
        Position::new(2, 0),
        Position::new(2, 1),
        ComboAction::Bootloader,
    );
    assert!(settings.add_combo(combo3).is_ok());
    assert_eq!(settings.combos.len(), 3);

    // Fourth combo should fail (max 3)
    let combo4 = ComboDefinition::new(
        Position::new(3, 0),
        Position::new(3, 1),
        ComboAction::DisableEffects,
    );
    assert!(settings.add_combo(combo4).is_err());
}

#[test]
fn test_combo_settings_duplicate_detection() {
    let mut settings = ComboSettings::new(true);

    let combo1 = ComboDefinition::new(
        Position::new(0, 0),
        Position::new(0, 1),
        ComboAction::DisableEffects,
    );
    assert!(settings.add_combo(combo1).is_ok());

    // Same key pair in same order
    let combo2 = ComboDefinition::new(
        Position::new(0, 0),
        Position::new(0, 1),
        ComboAction::DisableLighting,
    );
    assert!(settings.add_combo(combo2).is_err());

    // Same key pair in reverse order
    let combo3 = ComboDefinition::new(
        Position::new(0, 1),
        Position::new(0, 0),
        ComboAction::Bootloader,
    );
    assert!(settings.add_combo(combo3).is_err());
}

#[test]
fn test_combo_settings_has_custom_settings() {
    let settings = ComboSettings::default();
    assert!(!settings.has_custom_settings());

    let settings = ComboSettings::new(true);
    assert!(settings.has_custom_settings());

    let mut settings = ComboSettings::default();
    let combo = ComboDefinition::new(
        Position::new(0, 0),
        Position::new(0, 1),
        ComboAction::DisableEffects,
    );
    settings.add_combo(combo).unwrap();
    assert!(settings.has_custom_settings());
}
