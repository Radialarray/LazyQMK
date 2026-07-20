//! Tests for firmware::builder::build.
//!
//! Auto-extracted from src/firmware/builder/build.rs.

use super::super::state::{BuildMessage, BuildState, BuildStatus, LogLevel};
use super::*;

#[test]
fn test_build_status_display() {
assert_eq!(BuildStatus::Idle.to_string(), "Idle");
assert_eq!(BuildStatus::Compiling.to_string(), "Compiling...");
assert_eq!(BuildStatus::Success.to_string(), "✓ Success");
assert_eq!(BuildStatus::Failed.to_string(), "✗ Failed");
}

#[test]
fn test_build_state_new() {
let state = BuildState::new();
assert_eq!(state.status, BuildStatus::Idle);
assert!(!state.is_building());
assert!(state.receiver.is_none());
assert!(state.log_lines.is_empty());
}

#[test]
fn test_build_state_is_building() {
let mut state = BuildState::new();
assert!(!state.is_building());

state.status = BuildStatus::Compiling;
assert!(state.is_building());

state.status = BuildStatus::Success;
assert!(!state.is_building());
}

#[test]
fn test_build_message_progress() {
let mut state = BuildState::new();
let message = BuildMessage::Progress {
    status: BuildStatus::Compiling,
    message: "Test".to_string(),
};

state.handle_message(message);
assert_eq!(state.status, BuildStatus::Compiling);
assert_eq!(state.last_message, "Test");
assert_eq!(state.log_lines.len(), 1);
}

#[test]
fn test_build_message_complete_success() {
let mut state = BuildState::new();
let message = BuildMessage::Complete {
    success: true,
    firmware_path: Some(PathBuf::from("/test/firmware.uf2")),
    error: None,
};

state.handle_message(message);
assert_eq!(state.status, BuildStatus::Success);
assert!(state.last_message.contains("firmware.uf2"));
}

#[test]
fn test_build_message_complete_failure() {
let mut state = BuildState::new();
let message = BuildMessage::Complete {
    success: false,
    firmware_path: None,
    error: Some("Build failed".to_string()),
};

state.handle_message(message);
assert_eq!(state.status, BuildStatus::Failed);
assert_eq!(state.last_message, "Build failed");
}

#[test]
fn test_log_level_color() {
assert_eq!(LogLevel::Info.color(), ratatui::style::Color::Gray);
assert_eq!(LogLevel::Ok.color(), ratatui::style::Color::Green);
assert_eq!(LogLevel::Error.color(), ratatui::style::Color::Red);
}

#[test]
fn test_enhance_qmk_error_command_not_found() {
let error = "qmk: command not found";
let enhanced = enhance_qmk_error(error);
assert!(enhanced.contains("lazyqmk doctor"));
assert!(enhanced.contains(error));
}

#[test]
fn test_enhance_qmk_error_not_found() {
let error = "qmk: not found";
let enhanced = enhance_qmk_error(error);
assert!(enhanced.contains("lazyqmk doctor"));
}

#[test]
fn test_enhance_qmk_error_no_such_file() {
let error = "No such file or directory";
let enhanced = enhance_qmk_error(error);
assert!(enhanced.contains("lazyqmk doctor"));
}

#[test]
fn test_enhance_qmk_error_windows_not_recognized() {
let error = "'qmk' is not recognized as an internal or external command";
let enhanced = enhance_qmk_error(error);
assert!(enhanced.contains("lazyqmk doctor"));
}

#[test]
fn test_enhance_qmk_error_other_error() {
let error = "Some other qmk compilation error";
let enhanced = enhance_qmk_error(error);
assert_eq!(enhanced, error);
assert!(!enhanced.contains("lazyqmk doctor"));
}

#[test]
fn test_enhance_qmk_error_preserves_original_message() {
let error = "qmk compile failed: not found in PATH";
let enhanced = enhance_qmk_error(error);
assert!(enhanced.starts_with(error));

}
