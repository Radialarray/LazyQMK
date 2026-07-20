//! Tests for checker.
//!
//! Auto-extracted from checker.rs.

use super::*;

use super::*;

#[test]
fn test_parse_version_simple() {
    assert_eq!(
        DependencyChecker::parse_version_simple("1.1.5"),
        Some("1.1.5".to_string())
    );
    assert_eq!(
        DependencyChecker::parse_version_simple("qmk 1.1.5"),
        Some("1.1.5".to_string())
    );
    assert_eq!(
        DependencyChecker::parse_version_simple("  2.0.13  "),
        Some("2.0.13".to_string())
    );
    assert_eq!(
        DependencyChecker::parse_version_simple("no version here"),
        None
    );
}

#[test]
fn test_parse_gcc_version() {
    assert_eq!(
        DependencyChecker::parse_gcc_version("avr-gcc (GCC) 5.4.0"),
        Some("5.4.0".to_string())
    );
    assert_eq!(
        DependencyChecker::parse_gcc_version(
            "arm-none-eabi-gcc (GNU Arm Embedded Toolchain 10.3-2021.10) 10.3.1 20210824 (release)"
        ),
        Some("10.3.1".to_string())
    );
    assert_eq!(
        DependencyChecker::parse_gcc_version("gcc version 11.2.0"),
        Some("11.2.0".to_string())
    );
}

#[test]
fn test_dependency_status_constructors() {
    let available = DependencyStatus::available("Test Tool", "1.0.0");
    assert_eq!(available.status, ToolStatus::Available);
    assert_eq!(available.version, Some("1.0.0".to_string()));

    let missing = DependencyStatus::missing("Test Tool", "Not found");
    assert_eq!(missing.status, ToolStatus::Missing);
    assert_eq!(missing.version, None);

    let unknown = DependencyStatus::unknown("Test Tool", "Unknown error");
    assert_eq!(unknown.status, ToolStatus::Unknown);
    assert_eq!(unknown.version, None);
}

#[test]
fn test_check_qmk_firmware_missing_path() {
    let checker = DependencyChecker::new();
    let status = checker.check_qmk_firmware(None);
    assert_eq!(status.status, ToolStatus::Missing);
    assert!(status.message.contains("not configured"));
}

#[test]
fn test_check_qmk_firmware_nonexistent() {
    let checker = DependencyChecker::new();
    let fake_path = Path::new("/nonexistent/path/to/qmk");
    let status = checker.check_qmk_firmware(Some(fake_path));
    assert_eq!(status.status, ToolStatus::Missing);
    assert!(status.message.contains("does not exist"));
}

#[test]
#[ignore] // Requires actual QMK submodule
fn test_check_qmk_firmware_valid() {
    let checker = DependencyChecker::new();
    let qmk_path = Path::new("qmk_firmware");

    if qmk_path.exists() {
        let status = checker.check_qmk_firmware(Some(qmk_path));
        assert_eq!(status.status, ToolStatus::Available);
        assert!(status.message.contains("Valid"));
    }
}

#[test]
fn test_check_all_returns_all_statuses() {
    let checker = DependencyChecker::new();
    let statuses = checker.check_all(None);

    // Should check 4 dependencies
    assert_eq!(statuses.len(), 4);

    // Verify names
    assert_eq!(statuses[0].name, "QMK CLI");
    assert_eq!(statuses[1].name, "ARM GCC");
    assert_eq!(statuses[2].name, "AVR GCC");
    assert_eq!(statuses[3].name, "QMK Firmware");
}
