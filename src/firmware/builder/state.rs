//! `BuildState` + `BuildStatus` + `LogLevel` types and the build
//! lifecycle driver.

use anyhow::Result;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use super::build::run_build;


/// Build status tracking.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // All variants exercised in tests; bin doesn't link
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

impl LogLevel {
    /// Returns the terminal color for this log level.
    #[must_use]
    #[allow(dead_code)] // Display helper for tests; bin target doesn't link
    pub const fn color(self) -> ratatui::style::Color {
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
    pub(super) fn handle_message(&mut self, message: BuildMessage) {
        match message {
            BuildMessage::Progress { status, message } => {
                self.status = status.clone();
                self.last_message.clone_from(&message);
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
                    self.last_message.clone_from(&err);
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