//! Tests for template_gen.
//!
//! Auto-extracted from template_gen.rs.

use super::*;

    use super::*;
    use crate::models::{
        Category, ColorPalette, KeyDefinition, Layer, LayoutMetadata, Position, RgbColor,
    };
    use crate::parser::layout::parse_markdown_layout_str;
    use chrono::Utc;

    fn create_test_layout() -> Layout {
        let metadata = LayoutMetadata {
            name: "Test Layout".to_string(),
            description: "A test layout".to_string(),
            author: "test".to_string(),
            created: Utc::now(),
            modified: Utc::now(),
            tags: vec!["test".to_string()],
            is_template: false,
            version: "1.0".to_string(),
            layout_variant: None,
            keyboard: None,
            keymap_name: None,
            output_format: None,
        };

        let mut layer = Layer {
            id: "test-layer-0".to_string(),
            number: 0,
            name: "Base".to_string(),
            default_color: ColorPalette::load()
                .unwrap_or_default()
                .default_layer_color(),
            category_id: None,
            keys: vec![],
            layer_colors_enabled: true,
        };

        // Add some keys
        layer.keys.push(KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_A".to_string(),
            label: None,
            color_override: None,
            category_id: None,
            combo_participant: false,
            description: None,
        });

        layer.keys.push(KeyDefinition {
            position: Position { row: 0, col: 1 },
            keycode: "KC_B".to_string(),
            label: None,
            color_override: Some(RgbColor::new(255, 0, 0)),
            category_id: None,
            combo_participant: false,
            description: None,
        });

        let category = Category {
            id: "navigation".to_string(),
            name: "Navigation".to_string(),
            color: RgbColor::new(0, 0, 255),
        };

        Layout {
            metadata,
            layers: vec![layer],
            categories: vec![category],
            rgb_enabled: true,
            rgb_brightness: crate::models::RgbBrightness::default(),
            rgb_saturation: crate::models::RgbSaturation::default(),
            rgb_matrix_default_speed: 127,
            rgb_timeout_ms: 0,
            uncolored_key_behavior: crate::models::UncoloredKeyBehavior::default(),
            idle_effect_settings: crate::models::IdleEffectSettings::default(),
            rgb_overlay_ripple: crate::models::RgbOverlayRippleSettings::default(),
            palette_fx: crate::models::PaletteFxSettings::default(),
            tap_hold_settings: crate::models::TapHoldSettings::default(),
            combo_settings: crate::models::ComboSettings::default(),
            tap_dances: vec![],
        }
    }

    #[test]
    fn test_generate_frontmatter() {
        let layout = create_test_layout();
        let frontmatter = generate_frontmatter(&layout).unwrap();

        println!("Generated frontmatter:\n{frontmatter}");

        assert!(frontmatter.starts_with("---\n"));
        assert!(frontmatter.ends_with("---\n"));
        // YAML may use single quotes or no quotes depending on content
        assert!(frontmatter.contains("name:") && frontmatter.contains("Test Layout"));
        assert!(frontmatter.contains("version:") && frontmatter.contains("1.0"));
    }

    #[test]
    fn test_serialize_keycode_syntax() {
        // Basic keycode
        let key = KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_A".to_string(),
            label: None,
            color_override: None,
            category_id: None,
            combo_participant: false,
            description: None,
        };
        assert_eq!(serialize_keycode_syntax(&key), "KC_A");

        // With color override
        let key_with_color = KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_A".to_string(),
            label: None,
            color_override: Some(RgbColor::new(255, 0, 0)),
            category_id: None,
            combo_participant: false,
            description: None,
        };
        assert_eq!(serialize_keycode_syntax(&key_with_color), "KC_A{#FF0000}");

        // With category
        let key_with_category = KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_LEFT".to_string(),
            label: None,
            color_override: None,
            category_id: Some("navigation".to_string()),
            combo_participant: false,
            description: None,
        };
        assert_eq!(
            serialize_keycode_syntax(&key_with_category),
            "KC_LEFT@navigation"
        );

        // With both
        let key_with_both = KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_A".to_string(),
            label: None,
            color_override: Some(RgbColor::new(0, 255, 0)),
            category_id: Some("symbols".to_string()),
            combo_participant: false,
            description: None,
        };
        assert_eq!(
            serialize_keycode_syntax(&key_with_both),
            "KC_A{#00FF00}@symbols"
        );
    }

    #[test]
    fn test_generate_categories() {
        let layout = create_test_layout();
        let categories_section = generate_categories(&layout);

        assert!(categories_section.contains("## Categories"));
        assert!(categories_section.contains("- navigation: Navigation (#0000FF)"));
    }

    #[test]
    fn test_round_trip() {
        let layout = create_test_layout();

        // Generate markdown
        let markdown = generate_markdown(&layout).unwrap();

        println!("Generated markdown:\n{markdown}");

        // Parse it back
        let parsed_layout = parse_markdown_layout_str(&markdown).unwrap();

        // Verify key data is preserved
        assert_eq!(parsed_layout.metadata.name, layout.metadata.name);
        assert_eq!(parsed_layout.layers.len(), layout.layers.len());
        println!("Original categories: {}", layout.categories.len());
        println!("Parsed categories: {}", parsed_layout.categories.len());
        assert_eq!(parsed_layout.categories.len(), layout.categories.len());
        assert_eq!(
            parsed_layout.layers[0].keys.len(),
            layout.layers[0].keys.len()
        );
    }

    #[test]
    fn test_settings_round_trip() {
        use crate::models::UncoloredKeyBehavior;

        let mut layout = create_test_layout();

        // Test with Off setting (0%)
        layout.uncolored_key_behavior = UncoloredKeyBehavior::from(0);
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with Off:\n{markdown}");
        assert!(markdown.contains("**Uncolored Key Brightness**: 0%"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.uncolored_key_behavior.as_percent(), 0);

        // Test with 50% brightness
        layout.uncolored_key_behavior = UncoloredKeyBehavior::from(50);
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with 50% brightness:\n{markdown}");
        assert!(markdown.contains("**Uncolored Key Brightness**: 50%"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.uncolored_key_behavior.as_percent(), 50);

        // Test RGB brightness
        layout.rgb_brightness = crate::models::RgbBrightness::from(50);
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with 50% brightness:\n{markdown}");
        assert!(markdown.contains("**RGB Brightness**: 50%"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.rgb_brightness.as_percent(), 50);

        // Test RGB saturation
        layout.rgb_saturation = crate::models::RgbSaturation::from(75);
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with 75% saturation:\n{markdown}");
        assert!(markdown.contains("**RGB Saturation**: 75%"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.rgb_saturation.as_percent(), 75);

        // Test RGB Matrix Speed
        layout.rgb_matrix_default_speed = 200;
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with RGB speed 200:\n{markdown}");
        assert!(markdown.contains("**RGB Matrix Speed**: 200"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.rgb_matrix_default_speed, 200);
    }

    mod test_helpers {
        use crate::models::{HoldDecisionMode, TapHoldPreset, TapHoldSettings};

        pub fn assert_home_row_mods_markdown(markdown: &str) {
            assert!(markdown.contains("## Settings"));
            assert!(markdown.contains("**Tap-Hold Preset**: Home Row Mods"));
            assert!(markdown.contains("**Tapping Term**: 175ms"));
            assert!(markdown.contains("**Retro Tapping**: On"));
            assert!(markdown.contains("**Flow Tap Term**: 150ms"));
            assert!(markdown.contains("**Chordal Hold**: On"));
        }

        pub fn assert_home_row_mods_settings(settings: &TapHoldSettings) {
            assert_eq!(settings.preset, TapHoldPreset::HomeRowMods);
            assert_eq!(settings.tapping_term, 175);
            assert!(settings.retro_tapping);
            assert_eq!(settings.flow_tap_term, Some(150));
            assert!(settings.chordal_hold);
        }

        pub fn assert_custom_settings_markdown(markdown: &str) {
            assert!(markdown.contains("**Tap-Hold Preset**: Custom"));
            assert!(markdown.contains("**Tapping Term**: 180ms"));
            assert!(markdown.contains("**Quick Tap Term**: 100ms"));
            assert!(markdown.contains("**Hold Mode**: Hold On Other Key Press"));
            assert!(markdown.contains("**Retro Tapping**: On"));
            assert!(markdown.contains("**Tapping Toggle**: 3 taps"));
            assert!(markdown.contains("**Flow Tap Term**: 120ms"));
            assert!(markdown.contains("**Chordal Hold**: On"));
        }

        pub fn assert_custom_settings(settings: &TapHoldSettings) {
            assert_eq!(settings.preset, TapHoldPreset::Custom);
            assert_eq!(settings.tapping_term, 180);
            assert_eq!(settings.quick_tap_term, Some(100));
            assert_eq!(settings.hold_mode, HoldDecisionMode::HoldOnOtherKeyPress);
            assert!(settings.retro_tapping);
            assert_eq!(settings.tapping_toggle, 3);
            assert_eq!(settings.flow_tap_term, Some(120));
            assert!(settings.chordal_hold);
        }
    }

    #[test]
    fn test_tap_hold_home_row_mods_preset_round_trip() {
        use crate::models::{TapHoldPreset, TapHoldSettings};
        use test_helpers::*;

        let mut layout = create_test_layout();
        layout.tap_hold_settings = TapHoldSettings::from_preset(TapHoldPreset::HomeRowMods);

        let markdown = generate_markdown(&layout).unwrap();
        assert_home_row_mods_markdown(&markdown);

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_home_row_mods_settings(&parsed.tap_hold_settings);
    }

    #[test]
    fn test_tap_hold_custom_settings_round_trip() {
        use crate::models::{HoldDecisionMode, TapHoldPreset, TapHoldSettings};
        use test_helpers::*;

        let mut layout = create_test_layout();
        layout.tap_hold_settings = TapHoldSettings {
            tapping_term: 180,
            quick_tap_term: Some(100),
            hold_mode: HoldDecisionMode::HoldOnOtherKeyPress,
            retro_tapping: true,
            tapping_toggle: 3,
            flow_tap_term: Some(120),
            chordal_hold: true,
            preset: TapHoldPreset::Custom,
        };

        let markdown = generate_markdown(&layout).unwrap();
        assert_custom_settings_markdown(&markdown);

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_custom_settings(&parsed.tap_hold_settings);
    }

    #[test]
    fn test_tap_hold_default_settings_not_written() {
        use crate::models::TapHoldSettings;

        let mut layout = create_test_layout();
        layout.tap_hold_settings = TapHoldSettings::default();

        let markdown = generate_markdown(&layout).unwrap();
        assert!(!markdown.contains("Tap-Hold"));
        assert!(!markdown.contains("Tapping Term"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.tap_hold_settings, TapHoldSettings::default());
    }

    #[test]
    fn test_tap_hold_responsive_preset_round_trip() {
        use crate::models::{HoldDecisionMode, TapHoldPreset, TapHoldSettings};

        let mut layout = create_test_layout();

        // Test with Responsive preset
        layout.tap_hold_settings = TapHoldSettings::from_preset(TapHoldPreset::Responsive);
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with Responsive:\n{markdown}");
        assert!(markdown.contains("**Tap-Hold Preset**: Responsive"));
        assert!(markdown.contains("**Tapping Term**: 150ms"));
        assert!(markdown.contains("**Quick Tap Term**: 100ms"));
        assert!(markdown.contains("**Hold Mode**: Hold On Other Key Press"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.tap_hold_settings.preset, TapHoldPreset::Responsive);
        assert_eq!(parsed.tap_hold_settings.tapping_term, 150);
        assert_eq!(parsed.tap_hold_settings.quick_tap_term, Some(100));
        assert_eq!(
            parsed.tap_hold_settings.hold_mode,
            HoldDecisionMode::HoldOnOtherKeyPress
        );
    }

    #[test]
    fn test_tap_hold_deliberate_preset_round_trip() {
        use crate::models::{TapHoldPreset, TapHoldSettings};

        let mut layout = create_test_layout();

        // Test with Deliberate preset
        layout.tap_hold_settings = TapHoldSettings::from_preset(TapHoldPreset::Deliberate);
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with Deliberate:\n{markdown}");
        assert!(markdown.contains("**Tap-Hold Preset**: Deliberate"));
        assert!(markdown.contains("**Tapping Term**: 250ms"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.tap_hold_settings.preset, TapHoldPreset::Deliberate);
        assert_eq!(parsed.tap_hold_settings.tapping_term, 250);
    }

    #[test]
    fn test_key_descriptions_round_trip() {
        let mut layout = create_test_layout();

        // Add descriptions to some keys
        layout.layers[0].keys[0].description = Some("Primary thumb key".to_string());
        layout.layers[0].keys[1].description = Some("Secondary action key".to_string());

        // Generate markdown
        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown with descriptions:\n{markdown}");

        // Verify the descriptions section is present
        assert!(markdown.contains("## Key Descriptions"));
        assert!(markdown.contains("- 0:0:0: Primary thumb key"));
        assert!(markdown.contains("- 0:0:1: Secondary action key"));

        // Parse it back
        let parsed = parse_markdown_layout_str(&markdown).unwrap();

        // Verify descriptions are preserved
        assert_eq!(
            parsed.layers[0].keys[0].description,
            Some("Primary thumb key".to_string())
        );
        assert_eq!(
            parsed.layers[0].keys[1].description,
            Some("Secondary action key".to_string())
        );
    }

    #[test]
    fn test_key_descriptions_no_section_when_empty() {
        let layout = create_test_layout();

        // No descriptions - should not have descriptions section
        let markdown = generate_markdown(&layout).unwrap();
        assert!(!markdown.contains("## Key Descriptions"));
    }

    // === Idle Effect Settings Tests ===

    #[test]
    fn test_idle_effect_default_not_written() {
        let layout = create_test_layout();
        let markdown = generate_markdown(&layout).unwrap();

        // Default settings should not be written
        assert!(!markdown.contains("Idle Effect"));
        assert!(!markdown.contains("Idle Timeout"));
        assert!(!markdown.contains("Idle Effect Duration"));
        assert!(!markdown.contains("Idle Effect Mode"));
    }

    #[test]
    fn test_idle_effect_disabled() {
        let mut layout = create_test_layout();
        layout.idle_effect_settings = crate::models::IdleEffectSettings {
            enabled: false,
            ..crate::models::IdleEffectSettings::default()
        };

        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Effect**: Off"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert!(!parsed.idle_effect_settings.enabled);
    }

    #[test]
    fn test_idle_effect_timeout_formats() {
        let mut layout = create_test_layout();

        // Test minutes format
        layout.idle_effect_settings.idle_timeout_ms = 120_000; // 2 minutes
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Timeout**: 2 min"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.idle_effect_settings.idle_timeout_ms, 120_000);

        // Test seconds format
        layout.idle_effect_settings.idle_timeout_ms = 45_000; // 45 seconds
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Timeout**: 45 sec"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.idle_effect_settings.idle_timeout_ms, 45_000);

        // Test milliseconds format
        layout.idle_effect_settings.idle_timeout_ms = 12_345; // Odd milliseconds
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Timeout**: 12345ms"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.idle_effect_settings.idle_timeout_ms, 12_345);

        // Test zero (disabled)
        layout.idle_effect_settings.idle_timeout_ms = 0;
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Timeout**: 0"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.idle_effect_settings.idle_timeout_ms, 0);
    }

    #[test]
    fn test_idle_effect_duration_formats() {
        let mut layout = create_test_layout();

        // Test minutes format
        layout.idle_effect_settings.idle_effect_duration_ms = 600_000; // 10 minutes
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Effect Duration**: 10 min"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.idle_effect_settings.idle_effect_duration_ms, 600_000);

        // Test seconds format
        layout.idle_effect_settings.idle_effect_duration_ms = 90_000; // 90 seconds
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Effect Duration**: 90 sec"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.idle_effect_settings.idle_effect_duration_ms, 90_000);

        // Test zero (immediate off)
        layout.idle_effect_settings.idle_effect_duration_ms = 0;
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Idle Effect Duration**: 0"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.idle_effect_settings.idle_effect_duration_ms, 0);
    }

    #[test]
    fn test_idle_effect_mode_round_trip() {
        use crate::models::RgbMatrixEffect;

        let mut layout = create_test_layout();

        // Test various effect modes (skip Breathing which is the default)
        let effects = [
            RgbMatrixEffect::RainbowMovingChevron,
            RgbMatrixEffect::CycleAll,
            RgbMatrixEffect::JellybeanRaindrops,
        ];

        for effect in effects {
            layout.idle_effect_settings.idle_effect_mode = effect;
            let markdown = generate_markdown(&layout).unwrap();
            assert!(
                markdown.contains("**Idle Effect Mode**:"),
                "Missing Idle Effect Mode for {}",
                effect.display_name()
            );
            assert!(
                markdown.contains(effect.display_name()),
                "Missing effect name {}",
                effect.display_name()
            );

            let parsed = parse_markdown_layout_str(&markdown).unwrap();
            assert_eq!(parsed.idle_effect_settings.idle_effect_mode, effect);
        }
    }

    #[test]
    fn test_idle_effect_complete_settings() {
        use crate::models::RgbMatrixEffect;

        let mut layout = create_test_layout();
        layout.idle_effect_settings = crate::models::IdleEffectSettings {
            enabled: true,
            idle_timeout_ms: 30_000,          // 30 seconds
            idle_effect_duration_ms: 180_000, // 3 minutes
            idle_effect_mode: RgbMatrixEffect::RainbowBeacon,
        };

        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown:\n{markdown}");

        assert!(markdown.contains("**Idle Timeout**: 30 sec"));
        assert!(markdown.contains("**Idle Effect Duration**: 3 min"));
        assert!(markdown.contains("**Idle Effect Mode**: Rainbow Beacon"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert!(parsed.idle_effect_settings.enabled);
        assert_eq!(parsed.idle_effect_settings.idle_timeout_ms, 30_000);
        assert_eq!(parsed.idle_effect_settings.idle_effect_duration_ms, 180_000);
        assert_eq!(
            parsed.idle_effect_settings.idle_effect_mode,
            RgbMatrixEffect::RainbowBeacon
        );
    }

    // === RGB Overlay Ripple Settings Tests ===

    #[test]
    fn test_ripple_overlay_default_not_written() {
        let layout = create_test_layout();
        let markdown = generate_markdown(&layout).unwrap();

        // Default settings should not be written
        assert!(!markdown.contains("Ripple Overlay"));
        assert!(!markdown.contains("Max Ripples"));
        assert!(!markdown.contains("Ripple Duration"));
    }

    #[test]
    fn test_ripple_overlay_enabled() {
        let mut layout = create_test_layout();
        layout.rgb_overlay_ripple = crate::models::RgbOverlayRippleSettings {
            enabled: true,
            ..crate::models::RgbOverlayRippleSettings::default()
        };

        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown:\n{markdown}");
        assert!(markdown.contains("**Ripple Overlay**: On"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert!(parsed.rgb_overlay_ripple.enabled);
    }

    #[test]
    fn test_ripple_overlay_basic_settings_round_trip() {
        let mut layout = create_test_layout();
        layout.rgb_overlay_ripple.enabled = true;
        layout.rgb_overlay_ripple.max_ripples = 6;
        layout.rgb_overlay_ripple.duration_ms = 750;
        layout.rgb_overlay_ripple.speed = 128; // non-default (default is 200)
        layout.rgb_overlay_ripple.band_width = 5;
        layout.rgb_overlay_ripple.amplitude_pct = 75;

        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown:\n{markdown}");

        assert!(markdown.contains("**Ripple Overlay**: On"));
        assert!(markdown.contains("**Max Ripples**: 6"));
        assert!(markdown.contains("**Ripple Duration**: 750ms"));
        assert!(markdown.contains("**Ripple Speed**: 128"));
        assert!(markdown.contains("**Ripple Band Width**: 5"));
        assert!(markdown.contains("**Ripple Amplitude**: 75%"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert!(parsed.rgb_overlay_ripple.enabled);
        assert_eq!(parsed.rgb_overlay_ripple.max_ripples, 6);
        assert_eq!(parsed.rgb_overlay_ripple.duration_ms, 750);
        assert_eq!(parsed.rgb_overlay_ripple.speed, 128);
        assert_eq!(parsed.rgb_overlay_ripple.band_width, 5);
        assert_eq!(parsed.rgb_overlay_ripple.amplitude_pct, 75);
    }

    #[test]
    fn test_ripple_overlay_color_modes_round_trip() {
        use crate::models::{RgbColor, RippleColorMode};

        let mut layout = create_test_layout();
        layout.rgb_overlay_ripple.enabled = true;

        // Test Fixed mode with custom color
        layout.rgb_overlay_ripple.color_mode = RippleColorMode::Fixed;
        layout.rgb_overlay_ripple.fixed_color = RgbColor::new(255, 0, 255); // Magenta
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Ripple Fixed Color**: #FF00FF"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(parsed.rgb_overlay_ripple.color_mode, RippleColorMode::Fixed);
        assert_eq!(
            parsed.rgb_overlay_ripple.fixed_color,
            RgbColor::new(255, 0, 255)
        );

        // Test KeyBased mode
        layout.rgb_overlay_ripple.color_mode = RippleColorMode::KeyBased;
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Ripple Color Mode**: Key Color"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(
            parsed.rgb_overlay_ripple.color_mode,
            RippleColorMode::KeyBased
        );

        // Test HueShift mode with custom shift
        layout.rgb_overlay_ripple.color_mode = RippleColorMode::HueShift;
        layout.rgb_overlay_ripple.hue_shift_deg = 120;
        let markdown = generate_markdown(&layout).unwrap();
        assert!(markdown.contains("**Ripple Color Mode**: Hue Shift"));
        assert!(markdown.contains("**Ripple Hue Shift**: 120°"));
        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert_eq!(
            parsed.rgb_overlay_ripple.color_mode,
            RippleColorMode::HueShift
        );
        assert_eq!(parsed.rgb_overlay_ripple.hue_shift_deg, 120);
    }

    #[test]
    fn test_ripple_overlay_trigger_settings_round_trip() {
        let mut layout = create_test_layout();
        layout.rgb_overlay_ripple.enabled = true;
        layout.rgb_overlay_ripple.trigger_on_press = false;
        layout.rgb_overlay_ripple.trigger_on_release = true;

        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown:\n{markdown}");

        assert!(markdown.contains("**Ripple Trigger on Press**: Off"));
        assert!(markdown.contains("**Ripple Trigger on Release**: On"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert!(!parsed.rgb_overlay_ripple.trigger_on_press);
        assert!(parsed.rgb_overlay_ripple.trigger_on_release);
    }

    #[test]
    fn test_ripple_overlay_ignore_settings_round_trip() {
        let mut layout = create_test_layout();
        layout.rgb_overlay_ripple.enabled = true;
        layout.rgb_overlay_ripple.ignore_transparent = false;
        layout.rgb_overlay_ripple.ignore_modifiers = true;
        layout.rgb_overlay_ripple.ignore_layer_switch = true;

        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown:\n{markdown}");

        assert!(markdown.contains("**Ripple Ignore Transparent**: Off"));
        assert!(markdown.contains("**Ripple Ignore Modifiers**: On"));
        assert!(markdown.contains("**Ripple Ignore Layer Switch**: On"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        assert!(!parsed.rgb_overlay_ripple.ignore_transparent);
        assert!(parsed.rgb_overlay_ripple.ignore_modifiers);
        assert!(parsed.rgb_overlay_ripple.ignore_layer_switch);
    }

    #[test]
    fn test_ripple_overlay_complete_settings() {
        use crate::models::{RgbColor, RippleColorMode};

        let mut layout = create_test_layout();
        layout.rgb_overlay_ripple = crate::models::RgbOverlayRippleSettings {
            enabled: true,
            max_ripples: 8,
            duration_ms: 1000,
            speed: 255,
            band_width: 4,
            amplitude_pct: 100,
            wave_count: 3,
            wave_delay_ms: 150,
            color_mode: RippleColorMode::HueShift,
            fixed_color: RgbColor::new(255, 255, 0), // Yellow
            hue_shift_deg: -90,
            trigger_on_press: true,
            trigger_on_release: true,
            ignore_transparent: false,
            ignore_modifiers: true,
            ignore_layer_switch: true,
            key_action_palette: None,
        };

        let markdown = generate_markdown(&layout).unwrap();
        println!("Generated markdown:\n{markdown}");

        assert!(markdown.contains("**Ripple Overlay**: On"));
        assert!(markdown.contains("**Max Ripples**: 8"));
        assert!(markdown.contains("**Ripple Duration**: 1000ms"));
        assert!(markdown.contains("**Ripple Speed**: 255"));
        assert!(markdown.contains("**Ripple Band Width**: 4"));
        assert!(markdown.contains("**Ripple Amplitude**: 100%"));
        assert!(markdown.contains("**Ripple Color Mode**: Hue Shift"));
        assert!(markdown.contains("**Ripple Fixed Color**: #FFFF00"));
        assert!(markdown.contains("**Ripple Hue Shift**: -90°"));
        assert!(markdown.contains("**Ripple Trigger on Release**: On"));
        assert!(markdown.contains("**Ripple Ignore Transparent**: Off"));
        assert!(markdown.contains("**Ripple Ignore Modifiers**: On"));
        assert!(markdown.contains("**Ripple Ignore Layer Switch**: On"));
        assert!(markdown.contains("**Ripple Wave Count**: 3"));
        assert!(markdown.contains("**Ripple Wave Delay**: 150ms"));

        let parsed = parse_markdown_layout_str(&markdown).unwrap();
        let rip = &parsed.rgb_overlay_ripple;
        assert!(rip.enabled);
        assert_eq!(rip.max_ripples, 8);
        assert_eq!(rip.duration_ms, 1000);
        assert_eq!(rip.speed, 255);
        assert_eq!(rip.band_width, 4);
        assert_eq!(rip.amplitude_pct, 100);
        assert_eq!(rip.wave_count, 3);
        assert_eq!(rip.wave_delay_ms, 150);
        assert_eq!(rip.color_mode, RippleColorMode::HueShift);
        assert_eq!(rip.fixed_color, RgbColor::new(255, 255, 0));
        assert_eq!(rip.hue_shift_deg, -90);
        assert!(rip.trigger_on_press);
        assert!(rip.trigger_on_release);
        assert!(!rip.ignore_transparent);
        assert!(rip.ignore_modifiers);
        assert!(rip.ignore_layer_switch);
    }
