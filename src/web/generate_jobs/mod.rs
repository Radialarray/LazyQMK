//! Background generate job system for firmware generation and zip packaging.
//!
//! This module provides a thread-safe job queue for generating firmware files
//! (keymap.c, config.h) and packaging them into downloadable zip archives.
//!
//! ## Design
//!
//! - Jobs are identified by UUIDs
//! - Concurrency limit of 1 (single generation at a time)
//! - Logs are persisted to disk for durability
//! - Uses mpsc channels for thread communication
//! - Generated zip contains: layout source, keymap.c, config.h, manifest.json, logs
//!
//! ## Security
//!
//! - Validates all filenames to prevent path traversal (zip-slip)
//! - Uses safe path construction for all file operations

pub mod log_parse;
pub mod manager;
pub mod workers;

#[cfg(test)]
mod tests;

// ---------------------------------------------------------------------------
// Imports — kept here so that tests.rs (child module) sees them via `use super::*`
// ---------------------------------------------------------------------------
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::keycode_db::KeycodeDb;

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::collections::HashSet;
#[cfg(test)]
use std::fs::{self, File};
#[cfg(test)]
use std::io::{BufRead, BufReader};
#[cfg(test)]
use std::path::Path;
#[cfg(test)]
use std::sync::{mpsc, Arc, Mutex, RwLock};
#[cfg(test)]
use std::thread;
#[cfg(test)]
use zip::write::SimpleFileOptions;
#[cfg(test)]
use zip::ZipWriter;

// ---------------------------------------------------------------------------
// Re-exports from sub-modules
// ---------------------------------------------------------------------------
pub(crate) use log_parse::parse_log_line;
pub use manager::GenerateJobManager;

#[cfg(test)]
pub(crate) use workers::add_file_to_zip;
#[cfg(test)]
pub(crate) use workers::MockGenerateWorker;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of concurrent generate jobs.
pub(crate) const MAX_CONCURRENT_JOBS: usize = 1;

// ---------------------------------------------------------------------------
// GenerateJobStatus
// ---------------------------------------------------------------------------

/// Generate job status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerateJobStatus {
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

impl std::fmt::Display for GenerateJobStatus {
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

// ---------------------------------------------------------------------------
// GenerateJob
// ---------------------------------------------------------------------------

/// Generate job information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateJob {
    /// Unique job identifier.
    pub id: String,
    /// Current job status.
    pub status: GenerateJobStatus,
    /// Layout filename being generated.
    pub layout_filename: String,
    /// Keyboard name.
    pub keyboard: String,
    /// Layout variant name.
    pub layout_variant: String,
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
    /// Path to generated zip file (if successful).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip_path: Option<String>,
    /// Download URL for the zip file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    /// Progress percentage (0-100).
    pub progress: u8,
}

impl GenerateJob {
    /// Creates a new pending generate job.
    pub(crate) fn new(layout_filename: String, keyboard: String, layout_variant: String) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            id: id.clone(),
            status: GenerateJobStatus::Pending,
            layout_filename,
            keyboard,
            layout_variant,
            created_at: chrono::Utc::now().to_rfc3339(),
            started_at: None,
            completed_at: None,
            error: None,
            zip_path: None,
            download_url: Some(format!("/api/generate/jobs/{id}/download")),
            progress: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// LogEntry
// ---------------------------------------------------------------------------

/// Log entry for a generate job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp of the log entry.
    pub timestamp: String,
    /// Log level (info, error, warn).
    pub level: String,
    /// Log message.
    pub message: String,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Response for starting a generate job.
#[derive(Debug, Serialize)]
pub struct StartGenerateResponse {
    /// Status of the request.
    pub status: String,
    /// Message describing the current state.
    pub message: String,
    /// The created job.
    pub job: GenerateJob,
}

/// Response for job status.
#[derive(Debug, Serialize)]
pub struct GenerateJobStatusResponse {
    /// The job information.
    pub job: GenerateJob,
}

/// Response for job logs.
#[derive(Debug, Serialize)]
pub struct GenerateJobLogsResponse {
    /// Job ID.
    pub job_id: String,
    /// Log entries.
    pub logs: Vec<LogEntry>,
    /// Whether there are more logs to fetch.
    pub has_more: bool,
}

/// Response for cancelling a job.
#[derive(Debug, Serialize)]
pub struct CancelGenerateJobResponse {
    /// Whether cancellation was successful.
    pub success: bool,
    /// Message describing result.
    pub message: String,
}

/// Health information for the generate job system.
#[derive(Debug, Clone, Serialize)]
pub struct GenerateJobHealth {
    /// Whether the background worker thread is running.
    pub worker_running: bool,
    /// Number of currently running jobs.
    pub running_count: usize,
    /// Maximum number of concurrent jobs allowed.
    pub max_concurrent_jobs: usize,
}

// ---------------------------------------------------------------------------
// Internal command & worker trait
// ---------------------------------------------------------------------------

/// Generate command to be executed by worker thread.
pub(crate) struct GenerateCommand {
    pub(crate) job_id: String,
    pub(crate) layout_filename: String,
    pub(crate) layout_path: PathBuf,
    #[allow(dead_code)] // Used by bin; lib target doesn't link bin's test usage
    pub(crate) workspace_root: PathBuf,
    pub(crate) qmk_path: PathBuf,
    pub(crate) log_path: PathBuf,
    pub(crate) output_dir: PathBuf,
}

/// Trait for generate workers, allowing mock injection for tests.
pub(crate) trait GenerateWorker: Send + Sync {
    /// Runs the generate operation.
    ///
    /// Returns `Ok(zip_path)` on success or `Err(error_message)` on failure.
    fn generate(
        &self,
        cmd: &GenerateCommand,
        log_writer: &mut dyn Write,
        keycode_db: &KeycodeDb,
    ) -> Result<PathBuf, String>;
}
