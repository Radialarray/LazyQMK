//! Tests for config.
//!
//! Auto-extracted from config.rs.

use super::*;

    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert_eq!(config.paths.qmk_firmware, None);
        assert!(config.ui.show_help_on_startup);
        assert_eq!(config.ui.theme_mode, ThemeMode::Auto);
        assert!((config.ui.keyboard_scale - 1.0).abs() < f32::EPSILON);
        // New config should not be considered configured
        assert!(!config.is_configured());
        // Note: keyboard, layout, keymap, and output_format are now per-layout in metadata
    }

    #[test]
    fn test_config_is_configured() {
        let mut config = Config::new();

        // Without QMK path, config is not configured
        assert!(!config.is_configured());

        // With QMK path set, config is configured
        config.paths.qmk_firmware = Some(PathBuf::from("/some/path"));
        assert!(config.is_configured());
    }

    #[test]
    fn test_config_validate() {
        let config = Config::new();
        assert!(config.validate().is_ok());
        // Note: output_format validation is now per-layout in metadata
    }

    #[test]
    fn test_config_validate_qmk_path() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        fs::create_dir(&qmk_path).unwrap();

        let mut config = Config::new();
        config.paths.qmk_firmware = Some(qmk_path.clone());

        // Missing Makefile and keyboards/ directory
        assert!(config.validate().is_err());

        // Add Makefile
        fs::write(qmk_path.join("Makefile"), "").unwrap();
        assert!(config.validate().is_err()); // Still missing keyboards/

        // Add keyboards/ directory
        fs::create_dir(qmk_path.join("keyboards")).unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.toml");

        let config = Config::new();

        // Manually save to temp location for testing
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_file, content).unwrap();

        // Load and verify
        let content = fs::read_to_string(&config_file).unwrap();
        let loaded: Config = toml::from_str(&content).unwrap();

        // Verify basic fields are preserved
        assert_eq!(
            loaded.ui.show_help_on_startup,
            config.ui.show_help_on_startup
        );
        // Note: keyboard and layout are now per-layout in metadata
    }

    // Note: set_keyboard, set_layout, and set_output_format methods removed
    // These settings are now per-layout in metadata, not global config

    #[test]
    fn test_determine_keyboard_variant_no_variants() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("crkbd");

        // Create keyboard directory with keyboard.json at root (no variants)
        fs::create_dir_all(&keyboard_dir).unwrap();
        fs::write(keyboard_dir.join("keyboard.json"), "{}").unwrap();

        let build_config = BuildConfig::default();
        let result = build_config
            .determine_keyboard_variant(&qmk_path, "crkbd", 42)
            .unwrap();

        assert_eq!(result, "crkbd");
    }

    #[test]
    fn test_determine_keyboard_variant_with_standard_variant() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("keebart/corne_choc_pro");
        let standard_dir = keyboard_dir.join("standard");

        // Create variant directory structure
        fs::create_dir_all(&standard_dir).unwrap();
        fs::write(standard_dir.join("keyboard.json"), "{}").unwrap();

        let build_config = BuildConfig::default();

        // Use 44 keys which maps to "standard" variant
        let result = build_config
            .determine_keyboard_variant(&qmk_path, "keebart/corne_choc_pro", 44)
            .unwrap();

        assert_eq!(result, "keebart/corne_choc_pro/standard");
    }

    #[test]
    fn test_determine_keyboard_variant_with_mini_variant() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("keebart/corne_choc_pro");
        let mini_dir = keyboard_dir.join("mini");

        // Create variant directory structure
        fs::create_dir_all(&mini_dir).unwrap();
        fs::write(mini_dir.join("keyboard.json"), "{}").unwrap();

        let build_config = BuildConfig::default();

        let result = build_config
            .determine_keyboard_variant(&qmk_path, "keebart/corne_choc_pro", 36)
            .unwrap();

        assert_eq!(result, "keebart/corne_choc_pro/mini");
    }

    #[test]
    fn test_determine_keyboard_variant_ex2_layout_standard() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("keebart/corne_choc_pro");
        let standard_dir = keyboard_dir.join("standard");

        // Create variant directory structure
        fs::create_dir_all(&standard_dir).unwrap();
        fs::write(standard_dir.join("keyboard.json"), "{}").unwrap();

        let build_config = BuildConfig::default();

        let result = build_config
            .determine_keyboard_variant(&qmk_path, "keebart/corne_choc_pro", 44)
            .unwrap();

        assert_eq!(result, "keebart/corne_choc_pro/standard");
    }

    #[test]
    fn test_determine_keyboard_variant_ex2_layout_mini() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("keebart/corne_choc_pro");
        let mini_dir = keyboard_dir.join("mini");

        // Create variant directory structure
        fs::create_dir_all(&mini_dir).unwrap();
        fs::write(mini_dir.join("keyboard.json"), "{}").unwrap();

        let build_config = BuildConfig::default();

        let result = build_config
            .determine_keyboard_variant(&qmk_path, "keebart/corne_choc_pro", 38)
            .unwrap();

        assert_eq!(result, "keebart/corne_choc_pro/mini");
    }

    #[test]
    fn test_determine_keyboard_variant_missing_variant_directory() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("keebart/corne_choc_pro");

        // Create base keyboard directory but no variant subdirectories
        fs::create_dir_all(&keyboard_dir).unwrap();

        let build_config = BuildConfig::default();

        let result =
            build_config.determine_keyboard_variant(&qmk_path, "keebart/corne_choc_pro", 42);

        // Should return the base keyboard path when no variants are detected
        assert_eq!(result.unwrap(), "keebart/corne_choc_pro");
    }

    #[test]
    fn test_determine_keyboard_variant_with_info_json() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("test_keyboard");
        let standard_dir = keyboard_dir.join("standard");

        // Create variant directory with info.json instead of keyboard.json
        fs::create_dir_all(&standard_dir).unwrap();
        fs::write(standard_dir.join("info.json"), "{}").unwrap();

        let build_config = BuildConfig::default();

        // Use 44 keys which maps to "standard" variant
        let result = build_config
            .determine_keyboard_variant(&qmk_path, "test_keyboard", 44)
            .unwrap();

        assert_eq!(result, "test_keyboard/standard");
    }

    #[test]
    fn test_try_fix_qmk_path_existing_path() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        fs::create_dir(&qmk_path).unwrap();

        // If path exists, should return it as-is
        let result = Config::try_fix_qmk_path(&qmk_path);
        assert_eq!(result, Some(qmk_path));
    }

    #[test]
    fn test_try_fix_qmk_path_moved_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create old and new project directories
        let old_project = temp_dir.path().join("old_project");
        let new_project = temp_dir.path().join("new_project");
        fs::create_dir(&old_project).unwrap();
        fs::create_dir(&new_project).unwrap();

        // Create a valid QMK directory in new_project
        let qmk_dir = new_project.join("vial-qmk-keebart");
        fs::create_dir(&qmk_dir).unwrap();
        fs::write(qmk_dir.join("Makefile"), "").unwrap();
        fs::create_dir(qmk_dir.join("keyboards")).unwrap();

        // Create a reference to the old path (which doesn't exist)
        let old_path = old_project.join("vial-qmk-keebart");

        // Should find and return the new path
        let result = Config::try_fix_qmk_path(&old_path);
        assert_eq!(result, Some(qmk_dir));
    }

    #[test]
    fn test_try_fix_qmk_path_not_found() {
        let temp_dir = TempDir::new().unwrap();

        // Create a project directory but no QMK directory
        let project = temp_dir.path().join("project");
        fs::create_dir(&project).unwrap();

        // Reference to a non-existent path
        let missing_path = project.join("vial-qmk-keebart");

        // Should return None since directory doesn't exist anywhere
        let result = Config::try_fix_qmk_path(&missing_path);
        assert_eq!(result, None);
    }

    #[test]
    fn test_discover_keyboard_variants_with_rev_variants() {
        let temp_dir = TempDir::new().unwrap();
        let keyboard_dir = temp_dir.path().join("keyboard");
        fs::create_dir(&keyboard_dir).unwrap();

        // Create rev1 and rev2 variant directories
        let rev1_dir = keyboard_dir.join("rev1");
        let rev2_dir = keyboard_dir.join("rev2");
        fs::create_dir_all(&rev1_dir).unwrap();
        fs::create_dir_all(&rev2_dir).unwrap();

        // Add info.json to each variant
        fs::write(rev1_dir.join("info.json"), "{}").unwrap();
        fs::write(rev2_dir.join("keyboard.json"), "{}").unwrap();

        let variants = BuildConfig::discover_keyboard_variants(&keyboard_dir).unwrap();
        assert_eq!(variants.len(), 2);
        assert!(variants.contains(&"rev1".to_string()));
        assert!(variants.contains(&"rev2".to_string()));
    }

    #[test]
    fn test_discover_keyboard_variants_with_rgb_wireless() {
        let temp_dir = TempDir::new().unwrap();
        let keyboard_dir = temp_dir.path().join("keyboard");
        fs::create_dir(&keyboard_dir).unwrap();

        // Create rgb and wireless variant directories
        let rgb_dir = keyboard_dir.join("rgb");
        let wireless_dir = keyboard_dir.join("wireless");
        fs::create_dir_all(&rgb_dir).unwrap();
        fs::create_dir_all(&wireless_dir).unwrap();

        // Add keyboard.json to each variant
        fs::write(rgb_dir.join("keyboard.json"), "{}").unwrap();
        fs::write(wireless_dir.join("keyboard.json"), "{}").unwrap();

        let variants = BuildConfig::discover_keyboard_variants(&keyboard_dir).unwrap();
        assert_eq!(variants.len(), 2);
        assert!(variants.contains(&"rgb".to_string()));
        assert!(variants.contains(&"wireless".to_string()));
    }

    #[test]
    fn test_discover_keyboard_variants_ignores_non_variant_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let keyboard_dir = temp_dir.path().join("keyboard");
        fs::create_dir(&keyboard_dir).unwrap();

        // Create a variant directory with info.json
        let standard_dir = keyboard_dir.join("standard");
        fs::create_dir_all(&standard_dir).unwrap();
        fs::write(standard_dir.join("info.json"), "{}").unwrap();

        // Create directories without info.json or keyboard.json (should be ignored)
        let keymaps_dir = keyboard_dir.join("keymaps");
        let docs_dir = keyboard_dir.join("docs");
        fs::create_dir_all(&keymaps_dir).unwrap();
        fs::create_dir_all(&docs_dir).unwrap();

        let variants = BuildConfig::discover_keyboard_variants(&keyboard_dir).unwrap();
        assert_eq!(variants.len(), 1);
        assert!(variants.contains(&"standard".to_string()));
    }

    #[test]
    fn test_discover_keyboard_variants_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("does_not_exist");

        let variants = BuildConfig::discover_keyboard_variants(&nonexistent).unwrap();
        assert!(variants.is_empty());
    }

    #[test]
    fn test_determine_keyboard_variant_with_rev1() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("test_keyboard");
        let rev1_dir = keyboard_dir.join("rev1");

        // Create variant directory with keyboard.json
        fs::create_dir_all(&rev1_dir).unwrap();
        fs::write(rev1_dir.join("keyboard.json"), "{}").unwrap();

        let build_config = BuildConfig::default();

        // With 44 keys, it prefers "standard" but should fallback to "rev1" since standard doesn't exist
        let result = build_config
            .determine_keyboard_variant(&qmk_path, "test_keyboard", 44)
            .unwrap();

        assert_eq!(result, "test_keyboard/rev1");
    }

    #[test]
    fn test_determine_keyboard_variant_multiple_variants_prefers_standard() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("test_keyboard");

        // Create multiple variant directories
        let rev1_dir = keyboard_dir.join("rev1");
        let standard_dir = keyboard_dir.join("standard");
        fs::create_dir_all(&rev1_dir).unwrap();
        fs::create_dir_all(&standard_dir).unwrap();
        fs::write(rev1_dir.join("keyboard.json"), "{}").unwrap();
        fs::write(standard_dir.join("keyboard.json"), "{}").unwrap();

        let build_config = BuildConfig::default();

        // With 44 keys, should prefer "standard" when it exists
        let result = build_config
            .determine_keyboard_variant(&qmk_path, "test_keyboard", 44)
            .unwrap();

        assert_eq!(result, "test_keyboard/standard");
    }
