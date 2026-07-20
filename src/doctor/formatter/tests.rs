//! Tests for formatter.
//!
//! Auto-extracted from formatter.rs.

use super::*;

    use super::*;

    fn sample_statuses() -> Vec<DependencyStatus> {
        vec![
            DependencyStatus::available("QMK CLI", "1.1.5"),
            DependencyStatus::missing("ARM GCC", "Not found in PATH"),
            DependencyStatus::available("AVR GCC", "5.4.0"),
            DependencyStatus::unknown("QMK Firmware", "Could not determine status"),
        ]
    }

    #[test]
    fn test_platform_detect() {
        let platform = Platform::detect();
        // Just verify it doesn't panic and returns a valid platform
        assert!(matches!(
            platform,
            Platform::MacOs | Platform::Linux | Platform::Windows | Platform::Unknown
        ));
    }

    #[test]
    fn test_platform_names() {
        assert_eq!(Platform::MacOs.name(), "macOS");
        assert_eq!(Platform::Linux.name(), "Linux");
        assert_eq!(Platform::Windows.name(), "Windows");
        assert_eq!(Platform::Unknown.name(), "Unknown");
    }

    #[test]
    fn test_formatter_new() {
        let formatter = DoctorFormatter::new();
        assert_eq!(formatter.format, OutputFormat::Terminal);
    }

    #[test]
    fn test_formatter_with_format() {
        let formatter = DoctorFormatter::with_format(OutputFormat::Json);
        assert_eq!(formatter.format, OutputFormat::Json);
    }

    #[test]
    fn test_formatter_with_platform() {
        let formatter = DoctorFormatter::with_platform(Platform::Linux);
        assert_eq!(formatter.platform, Platform::Linux);
    }

    #[test]
    fn test_format_terminal_basic() {
        let formatter = DoctorFormatter::new();
        let statuses = sample_statuses();
        let output = formatter.format_terminal(&statuses);

        // Verify header
        assert!(output.contains("QMK Development Environment Status"));

        // Verify all dependencies are listed
        assert!(output.contains("QMK CLI"));
        assert!(output.contains("ARM GCC"));
        assert!(output.contains("AVR GCC"));
        assert!(output.contains("QMK Firmware"));

        // Verify symbols
        assert!(output.contains("✓")); // Available
        assert!(output.contains("✗")); // Missing
        assert!(output.contains("⚠")); // Unknown

        // Verify versions
        assert!(output.contains("1.1.5"));
        assert!(output.contains("5.4.0"));

        // Verify summary
        assert!(output.contains("Summary:"));
        assert!(output.contains("passed"));
    }

    #[test]
    fn test_format_terminal_all_passed() {
        let formatter = DoctorFormatter::new();
        let statuses = vec![
            DependencyStatus::available("QMK CLI", "1.1.5"),
            DependencyStatus::available("ARM GCC", "10.3.1"),
            DependencyStatus::available("AVR GCC", "5.4.0"),
            DependencyStatus::available("QMK Firmware", "Valid directory"),
        ];
        let output = formatter.format_terminal(&statuses);

        assert!(output.contains("All dependencies are ready"));
        assert!(output.contains("4 passed"));
    }

    #[test]
    fn test_format_terminal_with_failures() {
        let formatter = DoctorFormatter::new();
        let statuses = vec![
            DependencyStatus::missing("QMK CLI", "Not found"),
            DependencyStatus::missing("ARM GCC", "Not found"),
        ];
        let output = formatter.format_terminal(&statuses);

        assert!(output.contains("Missing required dependencies"));
        assert!(output.contains("2 failed"));
        assert!(output.contains("Next Steps:"));
    }

    #[test]
    fn test_format_json_basic() {
        let formatter = DoctorFormatter::with_format(OutputFormat::Json);
        let statuses = sample_statuses();
        let output = formatter.format_json(&statuses);

        // Parse JSON to verify structure
        let json: serde_json::Value =
            serde_json::from_str(&output).expect("Output should be valid JSON");

        assert_eq!(json["passed"], 2);
        assert_eq!(json["failed"], 1);
        assert_eq!(json["unknown"], 1);
        assert_eq!(json["status"], "missing_dependencies");
        assert!(json["dependencies"].is_array());
        assert_eq!(json["dependencies"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_format_json_all_ready() {
        let formatter = DoctorFormatter::with_format(OutputFormat::Json);
        let statuses = vec![
            DependencyStatus::available("QMK CLI", "1.1.5"),
            DependencyStatus::available("ARM GCC", "10.3.1"),
        ];
        let output = formatter.format_json(&statuses);

        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["status"], "ready");
        assert_eq!(json["passed"], 2);
        assert_eq!(json["failed"], 0);
    }

    #[test]
    fn test_format_results_terminal() {
        let formatter = DoctorFormatter::new();
        let statuses = sample_statuses();
        let output = formatter.format_results(&statuses);

        assert!(output.contains("QMK Development Environment Status"));
    }

    #[test]
    fn test_format_results_json() {
        let formatter = DoctorFormatter::with_format(OutputFormat::Json);
        let statuses = sample_statuses();
        let output = formatter.format_results(&statuses);

        // Should be valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&output).is_ok());
    }

    #[test]
    fn test_installation_instructions_qmk_cli() {
        let mac_formatter = DoctorFormatter::with_platform(Platform::MacOs);
        let linux_formatter = DoctorFormatter::with_platform(Platform::Linux);
        let windows_formatter = DoctorFormatter::with_platform(Platform::Windows);

        assert!(mac_formatter
            .format_qmk_install_instructions()
            .contains("pip3"));
        assert!(linux_formatter
            .format_qmk_install_instructions()
            .contains("pip3"));
        assert!(windows_formatter
            .format_qmk_install_instructions()
            .contains("pip"));
    }

    #[test]
    fn test_installation_instructions_arm_gcc() {
        let mac_formatter = DoctorFormatter::with_platform(Platform::MacOs);
        let linux_formatter = DoctorFormatter::with_platform(Platform::Linux);
        let windows_formatter = DoctorFormatter::with_platform(Platform::Windows);

        assert!(mac_formatter.format_arm_gcc_install().contains("brew"));
        assert!(linux_formatter.format_arm_gcc_install().contains("apt-get"));
        assert!(windows_formatter
            .format_arm_gcc_install()
            .contains("winget"));
    }

    #[test]
    fn test_installation_instructions_avr_gcc() {
        let mac_formatter = DoctorFormatter::with_platform(Platform::MacOs);
        let linux_formatter = DoctorFormatter::with_platform(Platform::Linux);

        assert!(mac_formatter.format_avr_gcc_install().contains("brew"));
        assert!(linux_formatter.format_avr_gcc_install().contains("apt-get"));
    }

    #[test]
    fn test_installation_instructions_firmware() {
        let formatter = DoctorFormatter::new();
        let instructions = formatter.format_firmware_setup();

        assert!(instructions.contains("git clone"));
        assert!(instructions.contains("qmk_firmware"));
    }

    #[test]
    fn test_get_installation_instructions() {
        let formatter = DoctorFormatter::new();

        assert!(formatter.get_installation_instructions("QMK CLI").is_some());
        assert!(formatter.get_installation_instructions("ARM GCC").is_some());
        assert!(formatter.get_installation_instructions("AVR GCC").is_some());
        assert!(formatter
            .get_installation_instructions("QMK Firmware")
            .is_some());
        assert!(formatter
            .get_installation_instructions("Unknown Tool")
            .is_none());
    }

    #[test]
    fn test_json_dependency_serialization() {
        let dep = JsonDependency {
            name: "QMK CLI".to_string(),
            status: "available".to_string(),
            version: Some("1.1.5".to_string()),
            message: "Found version 1.1.5".to_string(),
            installation_hint: None,
        };

        let json = serde_json::to_string(&dep).expect("Should serialize");
        assert!(json.contains("QMK CLI"));
        assert!(json.contains("1.1.5"));
    }

    #[test]
    fn test_json_output_serialization() {
        let output = JsonOutput {
            status: "ready".to_string(),
            passed: 4,
            failed: 0,
            unknown: 0,
            dependencies: vec![],
            platform: "macOS".to_string(),
        };

        let json = serde_json::to_string(&output).expect("Should serialize");
        assert!(json.contains("ready"));
        assert!(json.contains("macOS"));
    }

    #[test]
    fn test_terminal_output_includes_installation_for_missing() {
        let formatter = DoctorFormatter::with_platform(Platform::MacOs);
        let statuses = vec![DependencyStatus::missing("QMK CLI", "Not found in PATH")];
        let output = formatter.format_terminal(&statuses);

        // Should include installation instructions
        assert!(output.contains("Install:"));
        assert!(output.contains("pip"));
    }

    #[test]
    fn test_json_output_includes_installation_hints() {
        let formatter = DoctorFormatter::with_format(OutputFormat::Json);
        let statuses = vec![DependencyStatus::missing("ARM GCC", "Not found in PATH")];
        let output = formatter.format_json(&statuses);

        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        let deps = json["dependencies"].as_array().unwrap();
        let installation_hint = deps[0]["installation_hint"].as_str();

        assert!(installation_hint.is_some());
        assert!(!installation_hint.unwrap().is_empty());
    }
