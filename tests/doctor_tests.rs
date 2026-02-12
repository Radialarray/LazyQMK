//! Integration tests for the doctor module.
//!
//! These tests verify that the DependencyChecker can properly detect
//! external tools and validate the QMK firmware directory.

use lazyqmk::doctor::{DependencyChecker, ToolStatus};
use std::path::Path;

#[test]
fn test_check_all_returns_expected_dependencies() {
    let checker = DependencyChecker::new();
    let statuses = checker.check_all(None);

    // Should always check these 4 dependencies
    assert_eq!(statuses.len(), 4);

    // Verify dependency names
    let names: Vec<&str> = statuses.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["QMK CLI", "ARM GCC", "AVR GCC", "QMK Firmware"]);

    // Each status should have a non-empty message
    for status in &statuses {
        assert!(
            !status.message.is_empty(),
            "Status for {} has empty message",
            status.name
        );
    }
}

#[test]
fn test_qmk_firmware_path_not_configured() {
    let checker = DependencyChecker::new();
    let status = checker.check_qmk_firmware(None);

    assert_eq!(status.name, "QMK Firmware");
    assert_eq!(status.status, ToolStatus::Missing);
    assert!(status.message.contains("not configured"));
}

#[test]
fn test_qmk_firmware_invalid_path() {
    let checker = DependencyChecker::new();
    let invalid_path = Path::new("/this/path/does/not/exist/qmk_firmware");
    let status = checker.check_qmk_firmware(Some(invalid_path));

    assert_eq!(status.name, "QMK Firmware");
    assert_eq!(status.status, ToolStatus::Missing);
    assert!(status.message.contains("does not exist"));
}

#[test]
fn test_qmk_firmware_file_not_directory() {
    let checker = DependencyChecker::new();
    // Use Cargo.toml as a path that exists but is not a directory
    let file_path = Path::new("Cargo.toml");

    if file_path.exists() {
        let status = checker.check_qmk_firmware(Some(file_path));
        assert_eq!(status.name, "QMK Firmware");
        assert_eq!(status.status, ToolStatus::Missing);
        assert!(status.message.contains("not a directory"));
    }
}

#[test]
#[ignore] // Requires actual QMK submodule
fn test_qmk_firmware_valid_directory() {
    let checker = DependencyChecker::new();
    let qmk_path = Path::new("qmk_firmware");

    if qmk_path.exists() && qmk_path.is_dir() {
        let status = checker.check_qmk_firmware(Some(qmk_path));

        // Should be available if keyboards/ and quantum/ exist
        if qmk_path.join("keyboards").exists() && qmk_path.join("quantum").exists() {
            assert_eq!(status.status, ToolStatus::Available);
            assert!(status.message.contains("Valid"));
        } else {
            assert_eq!(status.status, ToolStatus::Missing);
            assert!(status.message.contains("missing"));
        }
    }
}

#[test]
fn test_custom_timeout() {
    let checker = DependencyChecker::with_timeout(10);
    let statuses = checker.check_all(None);

    // Should still return all 4 dependencies
    assert_eq!(statuses.len(), 4);
}

#[test]
fn test_tool_status_equality() {
    assert_eq!(ToolStatus::Available, ToolStatus::Available);
    assert_eq!(ToolStatus::Missing, ToolStatus::Missing);
    assert_eq!(ToolStatus::Unknown, ToolStatus::Unknown);
    assert_ne!(ToolStatus::Available, ToolStatus::Missing);
}

#[test]
fn test_dependency_status_fields() {
    let checker = DependencyChecker::new();
    let status = checker.check_qmk_cli();

    // Should have a name
    assert_eq!(status.name, "QMK CLI");

    // Should have a status (Available, Missing, or Unknown)
    assert!(matches!(
        status.status,
        ToolStatus::Available | ToolStatus::Missing | ToolStatus::Unknown
    ));

    // If available, should have a version
    if status.status == ToolStatus::Available {
        assert!(
            status.version.is_some(),
            "Available tool should have version"
        );
    }

    // Should always have a message
    assert!(!status.message.is_empty());
}

#[test]
fn test_individual_tool_checks() {
    let checker = DependencyChecker::new();

    // Each check should return a properly formed status
    let qmk_status = checker.check_qmk_cli();
    assert_eq!(qmk_status.name, "QMK CLI");
    assert!(!qmk_status.message.is_empty());

    let arm_status = checker.check_arm_gcc();
    assert_eq!(arm_status.name, "ARM GCC");
    assert!(!arm_status.message.is_empty());

    let avr_status = checker.check_avr_gcc();
    assert_eq!(avr_status.name, "AVR GCC");
    assert!(!avr_status.message.is_empty());

    let firmware_status = checker.check_qmk_firmware(None);
    assert_eq!(firmware_status.name, "QMK Firmware");
    assert!(!firmware_status.message.is_empty());
}
