//! Background build job system for firmware generation.
//!
//! This module provides a thread-safe job queue for running firmware builds
//! in the background. Jobs can be started, monitored, cancelled, and their
//! logs retrieved.
//!
//! ## Design
//!
//! - Jobs are identified by UUIDs
//! - Concurrency limit of 1 (single build at a time)
//! - Logs are persisted to disk for durability
//! - Uses mpsc channels for thread communication
//! - Firmware artifacts (.uf2/.bin/.hex) are copied to job-specific directories
//!
//! ## Artifact Management
//!
//! After a successful build, firmware artifacts are discovered in QMK's `.build`
//! directory and copied to `.lazyqmk/build_output/<job_id>/`. Each artifact gets
//! a stable ID based on its extension (e.g., "uf2", "bin", "hex") for easy reference.
//!
//! ## Artifact Cleanup Policy
//!
//! To prevent disk bloat, old artifacts are automatically cleaned up when new builds
//! are started:
//! - Artifacts older than 7 days (168 hours) are removed
//! - If more than 50 completed builds exist, oldest are removed first
//! - Active (pending/running) jobs are never cleaned up
//! - Both artifact files and log files are removed during cleanup
//!
//! ## Cancellation Support
//!
//! Running builds can be cancelled via the `cancel_job()` method. When a build is
//! cancelled:
//! - The underlying `qmk compile` process is killed immediately
//! - The job status is updated to `Cancelled`
//! - Build logs reflect the cancellation event
//! - Partial artifacts are preserved (not automatically cleaned)
//!
//! ## Mock Support
//!
//! For testing, a mock builder can be injected that simulates builds without
//! invoking real QMK CLI commands.

pub mod builders;
pub mod log_parse;
pub mod manager;

#[cfg(test)]
mod tests;

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(test)]
use crate::keycode_db::KeycodeDb;
#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use std::thread;

pub(crate) use builders::is_valid_artifact_id;
pub use builders::{MockFirmwareBuilder, RealFirmwareBuilder};
pub(crate) use log_parse::parse_log_line;
pub use manager::BuildJobManager;

/// Maximum number of concurrent builds.
const MAX_CONCURRENT_BUILDS: usize = 1;

/// Supported firmware artifact extensions.
const ARTIFACT_EXTENSIONS: &[&str] = &["uf2", "bin", "hex"];

/// A firmware artifact produced by a build job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifact {
    /// Stable artifact identifier (based on extension, e.g., "uf2", "bin", "hex").
    pub id: String,
    /// Original filename of the artifact.
    pub filename: String,
    /// File extension/type (e.g., "uf2", "bin", "hex").
    pub artifact_type: String,
    /// Size of the artifact in bytes.
    pub size: u64,
    /// SHA256 hash of the artifact content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// Download URL for this artifact.
    pub download_url: String,
}

/// Build job status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// Job is queued, waiting to start.
    Pending,
    /// Job is currently running.
    Running,
    /// Job completed successfully.
    Completed,
    /// Job failed.
    Failed,
    /// Job was cancelled by user.
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Build job information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildJob {
    /// Unique job identifier.
    pub id: String,
    /// Current job status.
    pub status: JobStatus,
    /// Layout filename being built.
    pub layout_filename: String,
    /// Keyboard name.
    pub keyboard: String,
    /// Keymap name.
    pub keymap: String,
    /// Time when job was created.
    pub created_at: String,
    /// Time when job started running (if started).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// Time when job completed (if finished).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// Error message if job failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Path to generated firmware file (if successful).
    /// Deprecated: Use `artifacts` field instead for new integrations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware_path: Option<String>,
    /// Progress percentage (0-100).
    pub progress: u8,
    /// List of firmware artifacts produced by this build.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<BuildArtifact>,
}

impl BuildJob {
    /// Creates a new pending build job.
    fn new(layout_filename: String, keyboard: String, keymap: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            status: JobStatus::Pending,
            layout_filename,
            keyboard,
            keymap,
            created_at: chrono::Utc::now().to_rfc3339(),
            started_at: None,
            completed_at: None,
            error: None,
            firmware_path: None,
            progress: 0,
            artifacts: Vec::new(),
        }
    }
}

/// Log entry for a build job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp of the log entry.
    pub timestamp: String,
    /// Log level (info, error, warn).
    pub level: String,
    /// Log message.
    pub message: String,
}

/// Request to start a new build job.
#[derive(Debug, Deserialize)]
pub struct StartBuildRequest {
    /// Layout filename to build.
    pub layout_filename: String,
}

/// Response for starting a build job.
#[derive(Debug, Serialize)]
pub struct StartBuildResponse {
    /// The created job.
    pub job: BuildJob,
}

/// Response for job status.
#[derive(Debug, Serialize)]
pub struct JobStatusResponse {
    /// The job information.
    pub job: BuildJob,
}

/// Response for job logs.
#[derive(Debug, Serialize)]
pub struct JobLogsResponse {
    /// Job ID.
    pub job_id: String,
    /// Log entries.
    pub logs: Vec<LogEntry>,
    /// Whether there are more logs to fetch.
    pub has_more: bool,
}

/// Response for cancelling a job.
#[derive(Debug, Serialize)]
pub struct CancelJobResponse {
    /// Whether cancellation was successful.
    pub success: bool,
    /// Message describing result.
    pub message: String,
}

/// Result of a successful firmware build.
#[derive(Debug, Clone)]
pub struct BuildResult {
    /// Primary firmware path (first discovered artifact).
    pub firmware_path: PathBuf,
    /// All discovered artifacts with their metadata.
    pub artifacts: Vec<BuildArtifact>,
}

/// Trait for firmware builders, allowing mock injection for tests.
pub trait FirmwareBuilder: Send + Sync {
    /// Runs the firmware build.
    ///
    /// # Arguments
    /// * `qmk_path` - Path to QMK firmware directory
    /// * `keyboard` - Keyboard identifier
    /// * `keymap` - Keymap name
    /// * `output_dir` - Directory to copy artifacts into
    /// * `job_id` - Job identifier (for generating download URLs)
    /// * `log_writer` - Writer for build log output
    /// * `is_cancelled` - Function to check if build has been cancelled
    ///
    /// Returns `Ok(BuildResult)` on success or `Err(error_message)` on failure.
    fn build(
        &self,
        qmk_path: &PathBuf,
        keyboard: &str,
        keymap: &str,
        output_dir: &Path,
        job_id: &str,
        log_writer: &mut dyn Write,
        is_cancelled: &dyn Fn() -> bool,
    ) -> Result<BuildResult, String>;
}
