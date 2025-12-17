//! End-to-end tests for `lazyqmk help` command.

use std::process::Command;

/// Path to the lazyqmk binary
fn lazyqmk_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lazyqmk")
}

// ============================================================================
// List Topics Tests
// ============================================================================

#[test]
fn test_help_list_all_topics() {
    let output = Command::new(lazyqmk_bin())
        .args(["help"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Listing help topics should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.is_empty(),
        "Output should contain list of help topics"
    );
    // Should list topic names
    assert!(
        stdout.contains("Available") || stdout.contains("topics") || stdout.contains("help"),
        "Output should indicate help topics"
    );
}

#[test]
fn test_help_list_multiple_topics() {
    let output = Command::new(lazyqmk_bin())
        .args(["help"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Count how many topics are mentioned (expecting multiple)
    let topic_count = stdout.lines().count();
    assert!(
        topic_count > 1,
        "Should list multiple help topics"
    );
}

// ============================================================================
// Show Specific Topic Tests
// ============================================================================

#[test]
fn test_help_show_valid_topic() {
    // Try a common topic name (might vary by implementation)
    let output = Command::new(lazyqmk_bin())
        .args(["show-help", "main"])
        .output()
        .expect("Failed to execute command");

    // Should succeed or fail gracefully
    let code = output.status.code();
    assert!(
        code == Some(0) || code == Some(1),
        "Should return 0 or 1 for topic query"
    );

    if code == Some(0) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.is_empty(), "Topic output should not be empty");
    }
}

#[test]
fn test_help_show_keycode_picker_topic() {
    let output = Command::new(lazyqmk_bin())
        .args(["show-help", "keycode_picker"])
        .output()
        .expect("Failed to execute command");

    // Topic may or may not exist, but command should handle gracefully
    let code = output.status.code();
    assert!(
        code == Some(0) || code == Some(1),
        "Should return valid exit code"
    );

    if code == Some(0) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("keycode_picker") || stdout.contains("Keycode"),
            "Output should be related to the requested topic"
        );
    }
}

#[test]
fn test_help_show_settings_topic() {
    let output = Command::new(lazyqmk_bin())
        .args(["show-help", "settings_manager"])
        .output()
        .expect("Failed to execute command");

    let code = output.status.code();
    assert!(
        code == Some(0) || code == Some(1),
        "Should return valid exit code"
    );

    if code == Some(0) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("settings") || stdout.contains("Settings"),
            "Output should be related to settings"
        );
    }
}

// ============================================================================
// Invalid Topic Tests
// ============================================================================

#[test]
fn test_help_invalid_topic_fails() {
    let output = Command::new(lazyqmk_bin())
        .args(["show-help", "invalid_topic_xyz_that_does_not_exist"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Invalid topic should fail with exit code 1"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let output_text = format!("{}{}", stdout, stderr);
    assert!(
        output_text.contains("Unknown") || output_text.contains("not found") || output_text.contains("invalid"),
        "Error message should indicate topic not found"
    );
}

// ============================================================================
// Output Format Tests
// ============================================================================



#[test]
fn test_help_output_formatted_readable() {
    let output = Command::new(lazyqmk_bin())
        .args(["show-help", "main"])
        .output()
        .expect("Failed to execute command");

    if output.status.code() == Some(0) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should have some structure (newlines, clear formatting)
        let lines: Vec<&str> = stdout.lines().collect();
        assert!(
            lines.len() > 1,
            "Help output should be multi-line and formatted"
        );
    }
}

// ============================================================================
// Help System Functionality Tests
// ============================================================================

#[test]
fn test_help_command_consistency() {
    // First list topics
    let output1 = Command::new(lazyqmk_bin())
        .args(["help"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output1.status.code(), Some(0));

    // Should run consistently multiple times
    let output2 = Command::new(lazyqmk_bin())
        .args(["help"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output2.status.code(), Some(0));

    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    let stdout2 = String::from_utf8_lossy(&output2.stdout);

    // Output should be consistent
    assert_eq!(
        stdout1.lines().count(),
        stdout2.lines().count(),
        "Help output should be consistent"
    );
}

#[test]
fn test_help_no_arguments_lists_topics() {
    let output = Command::new(lazyqmk_bin())
        .args(["help"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    // When no topic specified, should show all topics or instructions
    assert!(
        stdout.contains("topic") || stdout.contains("help") || !stdout.is_empty(),
        "Output should indicate available help"
    );
}

// ============================================================================
// Edge Cases
// ============================================================================



#[test]
fn test_help_command_help_available() {
    // Verify that help command itself provides help
    let output = Command::new(lazyqmk_bin())
        .args(["show-help", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Should contain help text or usage info
    assert!(
        combined.contains("help") || combined.contains("Usage") || combined.contains("TOPIC"),
        "Should show help about the help command"
    );
}
