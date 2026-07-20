//! Tests for settings_summary.
//!
//! Auto-extracted from settings_summary.rs.

use super::*;

    use super::*;
    use crate::models::{
        Layout, RgbBrightness, RgbMatrixEffect, RgbSaturation, UncoloredKeyBehavior,
    };

    #[test]
    fn test_format_timeout_ms_disabled() {
        assert_eq!(format_timeout_ms(0), "Disabled");
    }

    #[test]
    fn test_format_timeout_ms_seconds() {
        assert_eq!(format_timeout_ms(5000), "5 sec");
        assert_eq!(format_timeout_ms(1000), "1 sec");
        assert_eq!(format_timeout_ms(30000), "30 sec");
    }

    #[test]
    fn test_format_timeout_ms_minutes() {
        assert_eq!(format_timeout_ms(60_000), "1 min");
        assert_eq!(format_timeout_ms(300_000), "5 min");
        assert_eq!(format_timeout_ms(120_000), "2 min");
    }

    #[test]
    fn test_format_timeout_ms_fractional_seconds() {
        assert_eq!(format_timeout_ms(1500), "1.500 sec");
        assert_eq!(format_timeout_ms(2750), "2.750 sec");
    }

    #[test]
    fn test_generate_settings_summary_minimal() {
        let layout = Layout::new("Test Layout").unwrap();

        let summary = generate_settings_summary(&layout);

        // Check for main sections
        assert!(summary.contains("## Configuration"));
        assert!(summary.contains("### RGB Settings"));
        assert!(summary.contains("### Idle Effect"));
        assert!(summary.contains("### Firmware"));
    }

    #[test]
    fn test_generate_settings_summary_rgb_defaults() {
        let layout = Layout::new("Test Layout").unwrap();
        let summary = generate_settings_summary(&layout);

        // Check RGB defaults (saturation=100%, brightness=100%, timeout=0)
        assert!(summary.contains("- **Saturation:** 100%"));
        assert!(summary.contains("- **Brightness (uncolored keys):** 100%"));
        assert!(summary.contains("- **Timeout:** Disabled"));
    }

    #[test]
    fn test_generate_settings_summary_idle_effect_defaults() {
        let layout = Layout::new("Test Layout").unwrap();
        let summary = generate_settings_summary(&layout);

        // Check idle effect defaults
        assert!(summary.contains("- **Enabled:** Yes"));
        assert!(summary.contains("- **Timeout:** 1 min (60000 ms)"));
        assert!(summary.contains("- **Effect:** Breathing"));
        assert!(summary.contains("- **Duration:** 5 min (300000 ms)"));
    }

    #[test]
    fn test_generate_settings_summary_custom_rgb() {
        let mut layout = Layout::new("Test Layout").unwrap();
        layout.rgb_saturation = RgbSaturation::new(150);
        layout.rgb_brightness = RgbBrightness::new(75);
        layout.uncolored_key_behavior = UncoloredKeyBehavior::new(40);
        layout.rgb_timeout_ms = 120_000;

        let summary = generate_settings_summary(&layout);

        assert!(summary.contains("- **Saturation:** 150%"));
        assert!(summary.contains("- **Brightness (uncolored keys):** 40%"));
        assert!(summary.contains("- **Timeout:** 2 min"));
    }

    #[test]
    fn test_generate_settings_summary_custom_idle_effect() {
        let mut layout = Layout::new("Test Layout").unwrap();
        layout.idle_effect_settings.enabled = false;
        layout.idle_effect_settings.idle_timeout_ms = 30_000;
        layout.idle_effect_settings.idle_effect_duration_ms = 600_000;
        layout.idle_effect_settings.idle_effect_mode = RgbMatrixEffect::RainbowMovingChevron;

        let summary = generate_settings_summary(&layout);

        assert!(summary.contains("- **Enabled:** No"));
        assert!(summary.contains("- **Timeout:** 30 sec (30000 ms)"));
        assert!(summary.contains("- **Effect:** Rainbow Moving Chevron"));
        assert!(summary.contains("- **Duration:** 10 min (600000 ms)"));
    }

    #[test]
    fn test_generate_settings_summary_firmware_settings() {
        let mut layout = Layout::new("Test Layout").unwrap();
        layout.metadata.keyboard = Some("splitkb/halcyon/corne".to_string());
        layout.metadata.keymap_name = Some("my_keymap".to_string());
        layout.metadata.output_format = Some("uf2".to_string());

        let summary = generate_settings_summary(&layout);

        assert!(summary.contains("- **Keyboard:** splitkb/halcyon/corne"));
        assert!(summary.contains("- **Keymap Name:** my_keymap"));
        assert!(summary.contains("- **Output Format:** uf2"));
    }

    #[test]
    fn test_generate_settings_summary_firmware_not_specified() {
        let layout = Layout::new("Test Layout").unwrap();
        let summary = generate_settings_summary(&layout);

        assert!(summary.contains("- **Keyboard:** Not specified"));
        assert!(summary.contains("- **Keymap Name:** Not specified"));
        assert!(summary.contains("- **Output Format:** Not specified"));
    }

    #[test]
    fn test_generate_settings_summary_all_custom_values() {
        let mut layout = Layout::new("Test Layout").unwrap();

        // Custom RGB settings
        layout.rgb_saturation = RgbSaturation::new(200);
        layout.rgb_brightness = RgbBrightness::new(50);
        layout.uncolored_key_behavior = UncoloredKeyBehavior::new(25);
        layout.rgb_timeout_ms = 180_000;

        // Custom idle effect
        layout.idle_effect_settings.enabled = false;
        layout.idle_effect_settings.idle_timeout_ms = 45_000;
        layout.idle_effect_settings.idle_effect_duration_ms = 120_000;
        layout.idle_effect_settings.idle_effect_mode = RgbMatrixEffect::CycleAll;

        // Firmware info
        layout.metadata.keyboard = Some("keebart/corne_choc_pro/standard".to_string());
        layout.metadata.keymap_name = Some("corne_choc_pro".to_string());
        layout.metadata.output_format = Some("hex".to_string());

        let summary = generate_settings_summary(&layout);

        // Verify all custom values are present
        assert!(summary.contains("- **Saturation:** 200%"));
        assert!(summary.contains("- **Brightness (uncolored keys):** 25%"));
        assert!(summary.contains("- **Timeout:** 3 min"));
        assert!(summary.contains("- **Enabled:** No"));
        assert!(summary.contains("- **Timeout:** 45 sec (45000 ms)"));
        assert!(summary.contains("- **Effect:** Cycle All"));
        assert!(summary.contains("- **Duration:** 2 min (120000 ms)"));
        assert!(summary.contains("- **Keyboard:** keebart/corne_choc_pro/standard"));
        assert!(summary.contains("- **Keymap Name:** corne_choc_pro"));
        assert!(summary.contains("- **Output Format:** hex"));
    }

    #[test]
    fn test_generate_settings_summary_formatting() {
        let layout = Layout::new("Test Layout").unwrap();
        let summary = generate_settings_summary(&layout);

        // Check markdown formatting
        assert!(summary.starts_with("## Configuration\n\n"));
        assert!(summary.contains("\n### RGB Settings\n"));
        assert!(summary.contains("\n### Idle Effect\n"));
        assert!(summary.contains("\n### Firmware\n"));

        // Verify no trailing spaces and proper structure
        for line in summary.lines() {
            if !line.is_empty() {
                assert!(
                    !line.ends_with(' '),
                    "Line should not end with space: {}",
                    line
                );
            }
        }
    }
