//! Background firmware compilation with progress tracking.
//!
//! This module handles spawning background threads to compile QMK firmware
//! and reporting progress via message channels.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/// Build status tracking.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BuildStatus {
    /// Build not started
    Idle,
    /// Validating layout before generation
    Validating,
    /// Generating firmware files
    Generating,
    /// Compiling QMK firmware
    Compiling,
    /// Build completed successfully
    Success,
    /// Build failed with error
    Failed,
}

impl std::fmt::Display for BuildStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Validating => write!(f, "Validating..."),
            Self::Generating => write!(f, "Generating..."),
            Self::Compiling => write!(f, "Compiling..."),
            Self::Success => write!(f, "✓ Success"),
            Self::Failed => write!(f, "✗ Failed"),
        }
    }
}

/// Build message types sent from background thread to main thread.
#[derive(Debug, Clone)]
pub enum BuildMessage {
    /// Build progress update
    Progress {
        /// Current build status
        status: BuildStatus,
        /// Progress message
        message: String,
    },
    /// Build log output
    Log {
        /// Log level (Info, Ok, Error)
        level: LogLevel,
        /// Log message content
        message: String,
    },
    /// Build completed (success or failure)
    Complete {
        /// Whether the build succeeded
        success: bool,
        /// Path to generated firmware file
        firmware_path: Option<PathBuf>,
        /// Error message if build failed
        error: Option<String>,
    },
}

/// Log level for build output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Informational message
    Info,
    /// Success message
    Ok,
    /// Error message
    Error,
}

#[allow(dead_code)]
impl LogLevel {
    /// Returns the terminal color for this log level.
    #[must_use]
    pub const fn color(&self) -> ratatui::style::Color {
        match self {
            Self::Info => ratatui::style::Color::Gray,
            Self::Ok => ratatui::style::Color::Green,
            Self::Error => ratatui::style::Color::Red,
        }
    }
}

/// Build state for tracking background compilation.
pub struct BuildState {
    /// Current build status
    pub status: BuildStatus,
    /// Message channel receiver
    pub receiver: Option<Receiver<BuildMessage>>,
    /// Accumulated build log
    pub log_lines: Vec<(LogLevel, String)>,
    /// Last status message
    pub last_message: String,
}

impl BuildState {
    /// Creates a new idle build state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            status: BuildStatus::Idle,
            receiver: None,
            log_lines: Vec::new(),
            last_message: String::new(),
        }
    }

    /// Checks if a build is currently running.
    #[must_use]
    pub const fn is_building(&self) -> bool {
        matches!(
            self.status,
            BuildStatus::Validating | BuildStatus::Generating | BuildStatus::Compiling
        )
    }

    /// Polls the message channel for new messages.
    ///
    /// Returns true if a message was received.
    pub fn poll(&mut self) -> bool {
        if let Some(receiver) = &self.receiver {
            match receiver.try_recv() {
                Ok(message) => {
                    self.handle_message(message);
                    true
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => false,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // Thread finished
                    self.receiver = None;
                    false
                }
            }
        } else {
            false
        }
    }

    /// Handles a build message.
    fn handle_message(&mut self, message: BuildMessage) {
        match message {
            BuildMessage::Progress { status, message } => {
                self.status = status.clone();
                self.last_message = message.clone();
                self.log_lines
                    .push((LogLevel::Info, format!("[{status}] {message}")));
            }
            BuildMessage::Log { level, message } => {
                self.log_lines.push((level, message));
            }
            BuildMessage::Complete {
                success,
                firmware_path,
                error,
            } => {
                self.status = if success {
                    BuildStatus::Success
                } else {
                    BuildStatus::Failed
                };

                if let Some(path) = firmware_path {
                    self.last_message = format!("Firmware written to {}", path.display());
                    self.log_lines
                        .push((LogLevel::Ok, self.last_message.clone()));
                }

                if let Some(err) = error {
                    self.last_message = err.clone();
                    self.log_lines.push((LogLevel::Error, err));
                }

                self.receiver = None;
            }
        }
    }

    /// Starts a build in the background.
    ///
    /// Returns a receiver for build messages.
    pub fn start_build(
        &mut self,
        qmk_path: PathBuf,
        keyboard: String,
        keymap: String,
    ) -> Result<()> {
        if self.is_building() {
            anyhow::bail!("Build already in progress");
        }

        let (sender, receiver) = channel();
        self.receiver = Some(receiver);
        self.status = BuildStatus::Compiling;
        self.log_lines.clear();
        self.last_message = "Starting build...".to_string();

        // Spawn background thread
        thread::spawn(move || {
            if let Err(e) = run_build(sender.clone(), qmk_path, keyboard, keymap) {
                let _ = sender.send(BuildMessage::Complete {
                    success: false,
                    firmware_path: None,
                    error: Some(format!("Build failed: {e}")),
                });
            }
        });

        Ok(())
    }
}

impl Default for BuildState {
    fn default() -> Self {
        Self::new()
    }
}

/// Runs the QMK build process in a background thread.
///
/// The keyboard parameter may include variant subdirectories (e.g., "`keebart/corne_choc_pro/standard`").
/// QMK's build system will use the variant-specific keyboard.json for configuration.
fn run_build(
    sender: Sender<BuildMessage>,
    qmk_path: PathBuf,
    keyboard: String,
    keymap: String,
) -> Result<()> {
    // Send progress: Compiling
    sender
        .send(BuildMessage::Progress {
            status: BuildStatus::Compiling,
            message: format!("Compiling {keymap} keymap for {keyboard}..."),
        })
        .context("Failed to send progress message")?;

    sender
        .send(BuildMessage::Log {
            level: LogLevel::Info,
            message: format!("Running: make {keyboard}:{keymap}"),
        })
        .ok();

    // Build make command with full keyboard path (including variant if present)
    let make_target = format!("{keyboard}:{keymap}");

    let mut cmd = Command::new("make");
    cmd.arg(&make_target)
        .current_dir(&qmk_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Execute command
    let output = cmd.output().context("Failed to execute make command")?;

    // Parse output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Send stdout logs
    for line in stdout.lines() {
        let level = if line.contains("error") || line.contains("Error") {
            LogLevel::Error
        } else if line.contains("warning") || line.contains("Warning") {
            LogLevel::Info
        } else {
            LogLevel::Info
        };

        sender
            .send(BuildMessage::Log {
                level,
                message: line.to_string(),
            })
            .ok();
    }

    // Send stderr logs (usually errors)
    for line in stderr.lines() {
        if !line.trim().is_empty() {
            sender
                .send(BuildMessage::Log {
                    level: LogLevel::Error,
                    message: line.to_string(),
                })
                .ok();
        }
    }

    // Check success
    if output.status.success() {
        // Find firmware file
        let firmware_path = find_firmware_file(&qmk_path, &keyboard, &keymap)?;

        sender
            .send(BuildMessage::Complete {
                success: true,
                firmware_path: Some(firmware_path),
                error: None,
            })
            .ok();
    } else {
        sender
            .send(BuildMessage::Complete {
                success: false,
                firmware_path: None,
                error: Some("Make command failed. Check build log for details.".to_string()),
            })
            .ok();
    }

    Ok(())
}

/// Finds the compiled firmware file.
///
/// QMK typically outputs to .build/{keyboard}_{keymap}.{ext}
fn find_firmware_file(qmk_path: &PathBuf, keyboard: &str, keymap: &str) -> Result<PathBuf> {
    // Clean keyboard path (replace / with _)
    let keyboard_clean = keyboard.replace('/', "_");

    // Try common firmware extensions in order
    let extensions = ["uf2", "hex", "bin"];

    for ext in &extensions {
        let firmware_name = format!("{keyboard_clean}_{keymap}.{ext}");
        let firmware_path = qmk_path.join(".build").join(&firmware_name);

        if firmware_path.exists() {
            return Ok(firmware_path);
        }
    }

    anyhow::bail!("Could not find firmware file for {keyboard} {keymap}. Check .build/ directory.")
}

#[cfg(test)]
mod tests {
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
}
