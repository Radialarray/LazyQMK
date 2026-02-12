//! Output formatting for dependency check results.
//!
//! This module provides user-friendly formatting of dependency check results with:
//! - Clear visual indicators (✓/✗/⚠)
//! - Color coding (green=success, red=failure, yellow=warning)
//! - Version information when available
//! - Platform-specific installation instructions
//! - Links to documentation
//! - Summary section with actionable next steps
//!
//! # Example
//!
//! ```rust
//! use lazyqmk::doctor::{DependencyChecker, DoctorFormatter};
//! use std::path::Path;
//!
//! let checker = DependencyChecker::new();
//! let qmk_path = Some(Path::new("/path/to/qmk_firmware"));
//! let statuses = checker.check_all(qmk_path);
//!
//! let formatter = DoctorFormatter::new();
//! let output = formatter.format_results(&statuses);
//! println!("{}", output);
//! ```

use crate::doctor::{DependencyStatus, ToolStatus};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

/// Output format for doctor results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable terminal output with colors
    Terminal,
    /// Machine-readable JSON output
    Json,
}

/// Platform-specific information for installation instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// macOS
    MacOs,
    /// Linux
    Linux,
    /// Windows
    Windows,
    /// Unknown platform
    Unknown,
}

impl Platform {
    /// Detects the current platform from OS configuration.
    #[must_use]
    pub fn detect() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Unknown
        }
    }

    /// Returns the platform name as a string.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::MacOs => "macOS",
            Self::Linux => "Linux",
            Self::Windows => "Windows",
            Self::Unknown => "Unknown",
        }
    }
}

/// JSON output structure for doctor results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOutput {
    /// Overall health status
    pub status: String,
    /// Number of successful checks
    pub passed: usize,
    /// Number of failed checks
    pub failed: usize,
    /// Number of unknown checks
    pub unknown: usize,
    /// Individual dependency results
    pub dependencies: Vec<JsonDependency>,
    /// Platform information
    pub platform: String,
}

/// JSON representation of a single dependency check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonDependency {
    /// Dependency name
    pub name: String,
    /// Status (available, missing, unknown)
    pub status: String,
    /// Version string if detected
    pub version: Option<String>,
    /// Status message
    pub message: String,
    /// Installation instructions if missing
    pub installation_hint: Option<String>,
}

/// Formatter for dependency check results.
pub struct DoctorFormatter {
    /// Output format
    format: OutputFormat,
    /// Target platform (for installation instructions)
    platform: Platform,
}

impl DoctorFormatter {
    /// Creates a new formatter with terminal output and auto-detected platform.
    #[must_use]
    pub fn new() -> Self {
        Self {
            format: OutputFormat::Terminal,
            platform: Platform::detect(),
        }
    }

    /// Creates a new formatter with specified output format.
    #[must_use]
    pub fn with_format(format: OutputFormat) -> Self {
        Self {
            format,
            platform: Platform::detect(),
        }
    }

    /// Creates a new formatter with specified platform (for testing).
    #[must_use]
    pub fn with_platform(platform: Platform) -> Self {
        Self {
            format: OutputFormat::Terminal,
            platform,
        }
    }

    /// Formats dependency check results into a human-readable or JSON string.
    ///
    /// # Arguments
    ///
    /// * `statuses` - Slice of dependency check results
    ///
    /// # Returns
    ///
    /// Formatted output string ready for display or parsing.
    pub fn format_results(&self, statuses: &[DependencyStatus]) -> String {
        match self.format {
            OutputFormat::Terminal => self.format_terminal(statuses),
            OutputFormat::Json => self.format_json(statuses),
        }
    }

    /// Formats results as human-readable terminal output with colors.
    fn format_terminal(&self, statuses: &[DependencyStatus]) -> String {
        let mut output = String::new();

        // Header
        output.push_str("QMK Development Environment Status\n");
        output.push_str("═══════════════════════════════════\n\n");

        // Count statuses
        let passed = statuses
            .iter()
            .filter(|s| s.status == ToolStatus::Available)
            .count();
        let failed = statuses
            .iter()
            .filter(|s| s.status == ToolStatus::Missing)
            .count();
        let unknown = statuses
            .iter()
            .filter(|s| s.status == ToolStatus::Unknown)
            .count();

        // Individual checks
        for status in statuses {
            let (symbol, status_text) = match status.status {
                ToolStatus::Available => ("✓", "OK"),
                ToolStatus::Missing => ("✗", "MISSING"),
                ToolStatus::Unknown => ("⚠", "UNKNOWN"),
            };

            // Format: ✓ QMK CLI ............... OK (v1.1.5)
            let name_width: usize = 20;
            let dots = ".".repeat(name_width.saturating_sub(status.name.len()));
            write!(output, "{} {}{} {}", symbol, status.name, dots, status_text)
                .expect("Writing to String should not fail");

            if let Some(version) = &status.version {
                write!(output, " (v{})", version).expect("Writing to String should not fail");
            }
            output.push('\n');

            // Add installation instructions for missing dependencies
            if status.status == ToolStatus::Missing {
                if let Some(instructions) = self.get_installation_instructions(&status.name) {
                    output.push_str("    Install: ");
                    output.push_str(&instructions);
                    output.push('\n');
                }
            }

            // Add warning/error details
            if status.status != ToolStatus::Available {
                let indented = status
                    .message
                    .lines()
                    .map(|line| format!("    {}", line))
                    .collect::<Vec<_>>()
                    .join("\n");
                output.push_str(&indented);
                output.push_str("\n\n");
            } else {
                output.push('\n');
            }
        }

        // Summary
        output.push_str("───────────────────────────────────\n");
        write!(output, "Summary: {} passed", passed).expect("Writing to String should not fail");
        if failed > 0 {
            write!(output, ", {} failed", failed).expect("Writing to String should not fail");
        }
        if unknown > 0 {
            write!(output, ", {} unknown", unknown).expect("Writing to String should not fail");
        }
        output.push('\n');

        // Overall status
        if failed == 0 && unknown == 0 {
            output.push_str("\n✓ All dependencies are ready!\n");
            output.push_str("  You can now compile QMK firmware.\n");
        } else if failed > 0 {
            output.push_str("\n✗ Missing required dependencies\n");
            output.push_str("  Install missing tools and run 'doctor' again.\n");
        } else {
            output.push_str("\n⚠ Some checks could not be completed\n");
            output.push_str("  Review warnings above and verify your setup.\n");
        }

        // Next steps
        if failed > 0 || unknown > 0 {
            output.push_str("\nNext Steps:\n");
            if statuses
                .iter()
                .any(|s| s.name == "QMK CLI" && s.status == ToolStatus::Missing)
            {
                output.push_str("  1. Install QMK CLI (see instructions above)\n");
            }
            if statuses.iter().any(|s| {
                (s.name == "ARM GCC" || s.name == "AVR GCC") && s.status == ToolStatus::Missing
            }) {
                output.push_str("  2. Install GCC toolchains for your keyboard architecture\n");
            }
            if statuses
                .iter()
                .any(|s| s.name == "QMK Firmware" && s.status == ToolStatus::Missing)
            {
                output.push_str("  3. Clone QMK firmware repository (see instructions above)\n");
            }
            output.push_str("\nFor detailed setup instructions, visit:\n");
            output.push_str("  https://docs.qmk.fm/newbs_getting_started\n");
        }

        output
    }

    /// Formats results as JSON for machine-readable output.
    fn format_json(&self, statuses: &[DependencyStatus]) -> String {
        let passed = statuses
            .iter()
            .filter(|s| s.status == ToolStatus::Available)
            .count();
        let failed = statuses
            .iter()
            .filter(|s| s.status == ToolStatus::Missing)
            .count();
        let unknown = statuses
            .iter()
            .filter(|s| s.status == ToolStatus::Unknown)
            .count();

        let overall_status = if failed == 0 && unknown == 0 {
            "ready"
        } else if failed > 0 {
            "missing_dependencies"
        } else {
            "warnings"
        };

        let dependencies: Vec<JsonDependency> = statuses
            .iter()
            .map(|s| JsonDependency {
                name: s.name.clone(),
                status: match s.status {
                    ToolStatus::Available => "available".to_string(),
                    ToolStatus::Missing => "missing".to_string(),
                    ToolStatus::Unknown => "unknown".to_string(),
                },
                version: s.version.clone(),
                message: s.message.clone(),
                installation_hint: if s.status == ToolStatus::Missing {
                    self.get_installation_instructions(&s.name)
                } else {
                    None
                },
            })
            .collect();

        let json_output = JsonOutput {
            status: overall_status.to_string(),
            passed,
            failed,
            unknown,
            dependencies,
            platform: self.platform.name().to_string(),
        };

        serde_json::to_string_pretty(&json_output).unwrap_or_else(|_| {
            r#"{"status":"error","message":"Failed to serialize JSON output"}"#.to_string()
        })
    }

    /// Gets platform-specific installation instructions for a dependency.
    fn get_installation_instructions(&self, name: &str) -> Option<String> {
        match name {
            "QMK CLI" => Some(self.format_qmk_install_instructions()),
            "ARM GCC" => Some(self.format_arm_gcc_install()),
            "AVR GCC" => Some(self.format_avr_gcc_install()),
            "QMK Firmware" => Some(self.format_firmware_setup()),
            _ => None,
        }
    }

    /// Formats QMK CLI installation instructions for the current platform.
    fn format_qmk_install_instructions(&self) -> String {
        match self.platform {
            Platform::MacOs | Platform::Linux => "pip3 install qmk".to_string(),
            Platform::Windows => {
                "pip install qmk  (or use QMK Toolbox: https://qmk.fm/toolbox/)".to_string()
            }
            Platform::Unknown => "pip3 install qmk  (may vary by platform)".to_string(),
        }
    }

    /// Formats ARM GCC installation instructions for the current platform.
    fn format_arm_gcc_install(&self) -> String {
        match self.platform {
            Platform::MacOs => "brew install --cask gcc-arm-embedded".to_string(),
            Platform::Linux => {
                "sudo apt-get install gcc-arm-none-eabi  (Debian/Ubuntu)\n         or: sudo pacman -S arm-none-eabi-gcc  (Arch)"
                    .to_string()
            }
            Platform::Windows => {
                "winget install Arm.GnuArmEmbeddedToolchain  (or download from arm.com)".to_string()
            }
            Platform::Unknown => {
                "Install ARM GCC toolchain from: https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm".to_string()
            }
        }
    }

    /// Formats AVR GCC installation instructions for the current platform.
    fn format_avr_gcc_install(&self) -> String {
        match self.platform {
            Platform::MacOs => "brew install avr-gcc".to_string(),
            Platform::Linux => {
                "sudo apt-get install gcc-avr avr-libc  (Debian/Ubuntu)\n         or: sudo pacman -S avr-gcc avr-libc  (Arch)"
                    .to_string()
            }
            Platform::Windows => {
                "Install via MSYS2: pacman -S mingw-w64-x86_64-avr-gcc  (or use QMK MSYS)".to_string()
            }
            Platform::Unknown => "Install AVR GCC toolchain for your platform".to_string(),
        }
    }

    /// Formats QMK firmware repository setup instructions.
    fn format_firmware_setup(&self) -> String {
        "git clone https://github.com/qmk/qmk_firmware.git\n         Then set qmk_firmware_path in LazyQMK config".to_string()
    }
}

impl Default for DoctorFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
}
