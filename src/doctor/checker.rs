//! Dependency checking for QMK development environment.
//!
//! This module provides functionality to detect and validate external tools
//! required for QMK firmware compilation.
//!
//! # Example
//!
//! ```rust
//! use lazyqmk::doctor::{DependencyChecker, ToolStatus};
//! use std::path::Path;
//!
//! // Create a checker
//! let checker = DependencyChecker::new();
//!
//! // Check all dependencies
//! let qmk_path = Some(Path::new("/path/to/qmk_firmware"));
//! let statuses = checker.check_all(qmk_path);
//!
//! // Print results
//! for status in &statuses {
//!     match status.status {
//!         ToolStatus::Available => {
//!             println!("✓ {}: {}", status.name, status.message);
//!         }
//!         ToolStatus::Missing => {
//!             println!("✗ {}: {}", status.name, status.message);
//!         }
//!         ToolStatus::Unknown => {
//!             println!("? {}: {}", status.name, status.message);
//!         }
//!     }
//! }
//!
//! // Check individual tools
//! let qmk_cli = checker.check_qmk_cli();
//! if qmk_cli.status == ToolStatus::Available {
//!     println!("QMK CLI version: {}", qmk_cli.version.unwrap());
//! }
//! ```
//!
//! # Cross-Platform Support
//!
//! The checker works on Linux, macOS, and Windows by using `std::process::Command`
//! to execute external tools. It handles platform-specific differences such as:
//! - Command not found errors (ENOENT on Unix, OS error 2)
//! - PATH environment variable differences
//! - Version string format variations
//!
//! # Error Handling
//!
//! The checker is designed to be resilient:
//! - Missing tools return `ToolStatus::Missing` with helpful installation instructions
//! - Command execution errors return `ToolStatus::Unknown` with error details
//! - Parsing failures return `ToolStatus::Unknown` rather than panicking
//! - All methods are non-panicking and return structured results

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

/// Status of a single dependency check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolStatus {
    /// Dependency is present and working
    Available,
    /// Dependency is not found or not working
    Missing,
    /// Could not determine status (timeout, error)
    Unknown,
}

/// Result of checking a single dependency.
#[derive(Debug, Clone)]
pub struct DependencyStatus {
    /// Name of the dependency (e.g., "QMK CLI", "ARM GCC")
    pub name: String,
    /// Status of the dependency
    pub status: ToolStatus,
    /// Version string if detected (e.g., "1.1.5")
    pub version: Option<String>,
    /// Human-readable message about the status
    pub message: String,
}

impl DependencyStatus {
    /// Creates a new dependency status.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        status: ToolStatus,
        version: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            status,
            version,
            message: message.into(),
        }
    }

    /// Creates a status for an available dependency.
    #[must_use]
    pub fn available(name: impl Into<String>, version: impl Into<String>) -> Self {
        let version_str = version.into();
        Self::new(
            name,
            ToolStatus::Available,
            Some(version_str.clone()),
            format!("Found version {version_str}"),
        )
    }

    /// Creates a status for a missing dependency.
    #[must_use]
    pub fn missing(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(name, ToolStatus::Missing, None, message)
    }

    /// Creates a status for an unknown dependency state.
    #[must_use]
    pub fn unknown(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(name, ToolStatus::Unknown, None, message)
    }
}

/// Checker for QMK development environment dependencies.
pub struct DependencyChecker {
    /// Timeout for running external commands (in seconds)
    /// Currently unused but reserved for future timeout implementation
    #[allow(dead_code)]
    command_timeout: Duration,
}

impl DependencyChecker {
    /// Creates a new dependency checker with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            command_timeout: Duration::from_secs(5),
        }
    }

    /// Creates a new dependency checker with custom timeout.
    #[must_use]
    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            command_timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Checks all dependencies and returns their status.
    ///
    /// # Arguments
    ///
    /// * `qmk_firmware_path` - Optional path to QMK firmware directory
    ///
    /// # Returns
    ///
    /// Vector of dependency statuses for all checked tools.
    pub fn check_all(&self, qmk_firmware_path: Option<&Path>) -> Vec<DependencyStatus> {
        vec![
            self.check_qmk_cli(),
            self.check_arm_gcc(),
            self.check_avr_gcc(),
            self.check_qmk_firmware(qmk_firmware_path),
        ]
    }

    /// Checks if QMK CLI is installed and working.
    ///
    /// Runs `qmk --version` and parses the output.
    ///
    /// # Returns
    ///
    /// Status indicating whether QMK CLI is available and its version.
    pub fn check_qmk_cli(&self) -> DependencyStatus {
        match self.run_version_command("qmk", &["--version"]) {
            Ok(output) => {
                // QMK CLI outputs: "1.1.5" or similar
                if let Some(version) = Self::parse_version_simple(&output) {
                    DependencyStatus::available("QMK CLI", version)
                } else {
                    DependencyStatus::unknown(
                        "QMK CLI",
                        format!("Found but could not parse version: {}", output.trim()),
                    )
                }
            }
            Err(e) => {
                if Self::is_command_not_found(&e) {
                    DependencyStatus::missing(
                        "QMK CLI",
                        "Not found in PATH. Install with: pip3 install qmk",
                    )
                } else {
                    DependencyStatus::unknown("QMK CLI", format!("Error checking: {e}"))
                }
            }
        }
    }

    /// Checks if ARM GCC toolchain is installed.
    ///
    /// Runs `arm-none-eabi-gcc --version` and parses the output.
    ///
    /// # Returns
    ///
    /// Status indicating whether ARM GCC is available and its version.
    pub fn check_arm_gcc(&self) -> DependencyStatus {
        match self.run_version_command("arm-none-eabi-gcc", &["--version"]) {
            Ok(output) => {
                // GCC outputs: "arm-none-eabi-gcc (GNU Arm Embedded Toolchain 10.3-2021.10) 10.3.1 20210824 (release)"
                if let Some(version) = Self::parse_gcc_version(&output) {
                    DependencyStatus::available("ARM GCC", version)
                } else {
                    DependencyStatus::unknown(
                        "ARM GCC",
                        format!(
                            "Found but could not parse version: {}",
                            output.lines().next().unwrap_or("")
                        ),
                    )
                }
            }
            Err(e) => {
                if Self::is_command_not_found(&e) {
                    DependencyStatus::missing(
                        "ARM GCC",
                        "Not found in PATH. Required for ARM-based keyboards (STM32, RP2040)",
                    )
                } else {
                    DependencyStatus::unknown("ARM GCC", format!("Error checking: {e}"))
                }
            }
        }
    }

    /// Checks if AVR GCC toolchain is installed.
    ///
    /// Runs `avr-gcc --version` and parses the output.
    ///
    /// # Returns
    ///
    /// Status indicating whether AVR GCC is available and its version.
    pub fn check_avr_gcc(&self) -> DependencyStatus {
        match self.run_version_command("avr-gcc", &["--version"]) {
            Ok(output) => {
                // GCC outputs: "avr-gcc (GCC) 5.4.0"
                if let Some(version) = Self::parse_gcc_version(&output) {
                    DependencyStatus::available("AVR GCC", version)
                } else {
                    DependencyStatus::unknown(
                        "AVR GCC",
                        format!(
                            "Found but could not parse version: {}",
                            output.lines().next().unwrap_or("")
                        ),
                    )
                }
            }
            Err(e) => {
                if Self::is_command_not_found(&e) {
                    DependencyStatus::missing(
                        "AVR GCC",
                        "Not found in PATH. Required for AVR-based keyboards (ATmega32U4, ATmega32A)",
                    )
                } else {
                    DependencyStatus::unknown("AVR GCC", format!("Error checking: {e}"))
                }
            }
        }
    }

    /// Checks if QMK firmware directory exists and is valid.
    ///
    /// Validates that the directory contains the expected QMK firmware structure
    /// (keyboards/ and quantum/ subdirectories).
    ///
    /// # Arguments
    ///
    /// * `path` - Optional path to QMK firmware directory
    ///
    /// # Returns
    ///
    /// Status indicating whether the QMK firmware directory is valid.
    pub fn check_qmk_firmware(&self, path: Option<&Path>) -> DependencyStatus {
        let Some(qmk_path) = path else {
            return DependencyStatus::missing(
                "QMK Firmware",
                "Path not configured. Run onboarding or set in config.toml",
            );
        };

        // Check if directory exists
        if !qmk_path.exists() {
            return DependencyStatus::missing(
                "QMK Firmware",
                format!("Directory does not exist: {}", qmk_path.display()),
            );
        }

        if !qmk_path.is_dir() {
            return DependencyStatus::missing(
                "QMK Firmware",
                format!("Path is not a directory: {}", qmk_path.display()),
            );
        }

        // Check for expected subdirectories
        let keyboards_dir = qmk_path.join("keyboards");
        let quantum_dir = qmk_path.join("quantum");

        let missing_dirs: Vec<&str> = [
            ("keyboards", keyboards_dir.exists()),
            ("quantum", quantum_dir.exists()),
        ]
        .iter()
        .filter_map(|(name, exists)| if *exists { None } else { Some(*name) })
        .collect();

        if missing_dirs.is_empty() {
            DependencyStatus::available(
                "QMK Firmware",
                format!("Valid QMK directory at {}", qmk_path.display()),
            )
        } else {
            DependencyStatus::missing(
                "QMK Firmware",
                format!(
                    "Invalid QMK directory (missing: {}): {}",
                    missing_dirs.join(", "),
                    qmk_path.display()
                ),
            )
        }
    }

    /// Runs a command to get version information.
    ///
    /// # Arguments
    ///
    /// * `command` - Command name to execute
    /// * `args` - Command arguments
    ///
    /// # Returns
    ///
    /// Combined stdout and stderr output from the command.
    fn run_version_command(&self, command: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(command)
            .args(args)
            .output()
            .context(format!("Failed to execute '{command}'"))?;

        // Some tools output version to stderr (like gcc)
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Prefer stdout, but fall back to stderr if stdout is empty
        let result = if stdout.trim().is_empty() {
            stderr.to_string()
        } else {
            stdout.to_string()
        };

        Ok(result)
    }

    /// Parses a simple version string (just the version number).
    ///
    /// Examples:
    /// - "1.1.5" -> Some("1.1.5")
    /// - "qmk 1.1.5" -> Some("1.1.5")
    fn parse_version_simple(output: &str) -> Option<String> {
        // Try to find a version pattern like X.Y.Z
        output
            .split_whitespace()
            .find(|word| {
                // Check if this looks like a version number
                let parts: Vec<&str> = word.split('.').collect();
                parts.len() >= 2
                    && parts
                        .iter()
                        .all(|part| part.chars().all(|c| c.is_ascii_digit()))
            })
            .map(String::from)
    }

    /// Parses GCC version output.
    ///
    /// Examples:
    /// - "avr-gcc (GCC) 5.4.0" -> Some("5.4.0")
    /// - "arm-none-eabi-gcc (...) 10.3.1 20210824 (release)" -> Some("10.3.1")
    fn parse_gcc_version(output: &str) -> Option<String> {
        // First line typically contains version info
        let first_line = output.lines().next()?;

        // Look for version pattern after parentheses or GCC
        for word in first_line.split_whitespace() {
            // Skip non-version words
            if word.starts_with('(') || word.ends_with(')') {
                continue;
            }

            // Check if this looks like a version number (X.Y or X.Y.Z)
            let clean_word = word.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
            let parts: Vec<&str> = clean_word.split('.').collect();

            if parts.len() >= 2
                && parts
                    .iter()
                    .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
            {
                return Some(clean_word.to_string());
            }
        }

        None
    }

    /// Checks if an error indicates a command was not found.
    ///
    /// This handles platform-specific error messages for missing commands.
    fn is_command_not_found(error: &anyhow::Error) -> bool {
        let error_msg = error.to_string().to_lowercase();
        error_msg.contains("not found")
            || error_msg.contains("no such file")
            || error_msg.contains("cannot find")
            || error_msg.contains("os error 2") // ENOENT on Unix
    }
}

impl Default for DependencyChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
            DependencyChecker::parse_gcc_version("arm-none-eabi-gcc (GNU Arm Embedded Toolchain 10.3-2021.10) 10.3.1 20210824 (release)"),
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
}
