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
//!
//! ## Mock Support
//!
//! For testing, a mock builder can be injected that simulates builds without
//! invoking real QMK CLI commands.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Maximum number of concurrent builds.
const MAX_CONCURRENT_BUILDS: usize = 1;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware_path: Option<String>,
    /// Progress percentage (0-100).
    pub progress: u8,
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

/// Build command to be executed by worker thread.
struct BuildCommand {
    job_id: String,
    #[allow(dead_code)] // Stored for potential future logging/display
    layout_filename: String,
    keyboard: String,
    keymap: String,
    qmk_path: PathBuf,
    log_path: PathBuf,
}

/// Trait for firmware builders, allowing mock injection for tests.
pub trait FirmwareBuilder: Send + Sync {
    /// Runs the firmware build.
    ///
    /// Returns `Ok(firmware_path)` on success or `Err(error_message)` on failure.
    fn build(
        &self,
        qmk_path: &PathBuf,
        keyboard: &str,
        keymap: &str,
        log_writer: &mut dyn Write,
    ) -> Result<PathBuf, String>;
}

/// Real firmware builder using QMK CLI.
pub struct RealFirmwareBuilder;

impl FirmwareBuilder for RealFirmwareBuilder {
    fn build(
        &self,
        qmk_path: &PathBuf,
        keyboard: &str,
        keymap: &str,
        log_writer: &mut dyn Write,
    ) -> Result<PathBuf, String> {
        use std::process::{Command, Stdio};

        let _ = writeln!(log_writer, "[INFO] Starting QMK compile...");
        let _ = writeln!(
            log_writer,
            "[INFO] Running: qmk compile -kb {} -km {}",
            keyboard, keymap
        );

        let mut cmd = Command::new("qmk");
        cmd.arg("compile")
            .arg("-kb")
            .arg(keyboard)
            .arg("-km")
            .arg(keymap)
            .current_dir(qmk_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute qmk: {e}"))?;

        // Write stdout to log
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let level = if line.contains("error") || line.contains("Error") {
                "ERROR"
            } else {
                "INFO"
            };
            let _ = writeln!(log_writer, "[{level}] {line}");
        }

        // Write stderr to log
        for line in String::from_utf8_lossy(&output.stderr).lines() {
            if !line.trim().is_empty() {
                let _ = writeln!(log_writer, "[ERROR] {line}");
            }
        }

        if !output.status.success() {
            return Err("QMK compile failed. Check build log for details.".to_string());
        }

        // Find firmware file
        let keyboard_clean = keyboard.replace('/', "_");
        let extensions = ["uf2", "hex", "bin"];

        for ext in &extensions {
            let firmware_name = format!("{keyboard_clean}_{keymap}.{ext}");
            let firmware_path = qmk_path.join(".build").join(&firmware_name);
            if firmware_path.exists() {
                let _ = writeln!(
                    log_writer,
                    "[INFO] Firmware generated: {}",
                    firmware_path.display()
                );
                return Ok(firmware_path);
            }
        }

        Err(format!(
            "Could not find firmware file for {keyboard} {keymap}"
        ))
    }
}

/// Mock firmware builder for testing.
pub struct MockFirmwareBuilder {
    /// Simulated build duration in milliseconds.
    pub build_duration_ms: u64,
    /// Whether the build should succeed.
    pub should_succeed: bool,
    /// Error message if build should fail.
    pub error_message: Option<String>,
}

impl Default for MockFirmwareBuilder {
    fn default() -> Self {
        Self {
            build_duration_ms: 100,
            should_succeed: true,
            error_message: None,
        }
    }
}

impl FirmwareBuilder for MockFirmwareBuilder {
    fn build(
        &self,
        _qmk_path: &PathBuf,
        keyboard: &str,
        keymap: &str,
        log_writer: &mut dyn Write,
    ) -> Result<PathBuf, String> {
        let _ = writeln!(log_writer, "[INFO] Mock build starting...");
        let _ = writeln!(
            log_writer,
            "[INFO] Building {} keymap for {}",
            keymap, keyboard
        );

        // Simulate build progress
        let steps = 5;
        let step_duration = self.build_duration_ms / steps;

        for i in 1..=steps {
            thread::sleep(Duration::from_millis(step_duration));
            let _ = writeln!(log_writer, "[INFO] Build progress: {}%", i * 20);
        }

        if self.should_succeed {
            let keyboard_clean = keyboard.replace('/', "_");
            let firmware_path = PathBuf::from(format!("/tmp/{keyboard_clean}_{keymap}.uf2"));
            let _ = writeln!(
                log_writer,
                "[INFO] Mock firmware generated: {}",
                firmware_path.display()
            );
            Ok(firmware_path)
        } else {
            let err = self
                .error_message
                .clone()
                .unwrap_or_else(|| "Mock build failed".to_string());
            let _ = writeln!(log_writer, "[ERROR] {}", err);
            Err(err)
        }
    }
}

/// Build job manager that coordinates background builds.
pub struct BuildJobManager {
    /// All jobs indexed by ID.
    jobs: RwLock<HashMap<String, BuildJob>>,
    /// Set of cancelled job IDs.
    cancelled: RwLock<std::collections::HashSet<String>>,
    /// Number of currently running jobs.
    running_count: Mutex<usize>,
    /// Channel sender for build commands.
    command_tx: Mutex<Option<mpsc::Sender<BuildCommand>>>,
    /// Directory for storing job logs.
    logs_dir: PathBuf,
    /// QMK firmware path from config.
    qmk_path: Option<PathBuf>,
    /// Firmware builder (real or mock).
    builder: Arc<dyn FirmwareBuilder>,
}

impl BuildJobManager {
    /// Creates a new build job manager.
    pub fn new(logs_dir: PathBuf, qmk_path: Option<PathBuf>) -> Arc<Self> {
        Self::with_builder(logs_dir, qmk_path, Arc::new(RealFirmwareBuilder))
    }

    /// Creates a new build job manager with a custom builder (for testing).
    pub fn with_builder(
        logs_dir: PathBuf,
        qmk_path: Option<PathBuf>,
        builder: Arc<dyn FirmwareBuilder>,
    ) -> Arc<Self> {
        // Ensure logs directory exists
        let _ = fs::create_dir_all(&logs_dir);

        let manager = Arc::new(Self {
            jobs: RwLock::new(HashMap::new()),
            cancelled: RwLock::new(std::collections::HashSet::new()),
            running_count: Mutex::new(0),
            command_tx: Mutex::new(None),
            logs_dir,
            qmk_path,
            builder,
        });

        // Start worker thread
        manager.start_worker();

        manager
    }

    /// Starts the background worker thread.
    fn start_worker(self: &Arc<Self>) {
        let (tx, rx) = mpsc::channel::<BuildCommand>();
        *self.command_tx.lock().unwrap() = Some(tx);

        let manager = Arc::clone(self);

        thread::spawn(move || {
            for cmd in rx {
                manager.process_build(cmd);
            }
        });
    }

    /// Processes a build command.
    fn process_build(self: &Arc<Self>, cmd: BuildCommand) {
        // Check if cancelled before starting
        if self.is_cancelled(&cmd.job_id) {
            self.update_job_status(&cmd.job_id, JobStatus::Cancelled, None, None);
            return;
        }

        // Update job to running
        {
            let mut jobs = self.jobs.write().unwrap();
            if let Some(job) = jobs.get_mut(&cmd.job_id) {
                job.status = JobStatus::Running;
                job.started_at = Some(chrono::Utc::now().to_rfc3339());
                job.progress = 10;
            }
        }

        // Open log file
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&cmd.log_path);

        let result = match log_file {
            Ok(mut file) => {
                let _ = writeln!(file, "[INFO] Build started at {}", chrono::Utc::now());

                // Check if cancelled during setup
                if self.is_cancelled(&cmd.job_id) {
                    let _ = writeln!(file, "[INFO] Build cancelled by user");
                    Err("Build cancelled".to_string())
                } else {
                    // Run the build
                    self.builder
                        .build(&cmd.qmk_path, &cmd.keyboard, &cmd.keymap, &mut file)
                }
            }
            Err(e) => Err(format!("Failed to open log file: {e}")),
        };

        // Decrement running count
        {
            let mut count = self.running_count.lock().unwrap();
            *count = count.saturating_sub(1);
        }

        // Check if cancelled after build
        if self.is_cancelled(&cmd.job_id) {
            self.update_job_status(&cmd.job_id, JobStatus::Cancelled, None, None);
            return;
        }

        // Update job with result
        match result {
            Ok(firmware_path) => {
                self.update_job_status(
                    &cmd.job_id,
                    JobStatus::Completed,
                    None,
                    Some(firmware_path.display().to_string()),
                );
            }
            Err(error) => {
                self.update_job_status(&cmd.job_id, JobStatus::Failed, Some(error), None);
            }
        }
    }

    /// Checks if a job has been cancelled.
    fn is_cancelled(&self, job_id: &str) -> bool {
        self.cancelled.read().unwrap().contains(job_id)
    }

    /// Updates a job's status.
    fn update_job_status(
        &self,
        job_id: &str,
        status: JobStatus,
        error: Option<String>,
        firmware_path: Option<String>,
    ) {
        let mut jobs = self.jobs.write().unwrap();
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = status;
            job.completed_at = Some(chrono::Utc::now().to_rfc3339());
            job.progress = if status == JobStatus::Completed {
                100
            } else {
                0
            };
            job.error = error;
            job.firmware_path = firmware_path;
        }
    }

    /// Starts a new build job.
    ///
    /// Returns the created job or an error if the build cannot be started.
    pub fn start_build(
        self: &Arc<Self>,
        layout_filename: String,
        keyboard: String,
        keymap: String,
    ) -> Result<BuildJob, String> {
        // Check QMK path
        let qmk_path = self
            .qmk_path
            .clone()
            .ok_or_else(|| "QMK firmware path not configured".to_string())?;

        // Check concurrency limit
        {
            let count = self.running_count.lock().unwrap();
            if *count >= MAX_CONCURRENT_BUILDS {
                return Err(
                    "Build already in progress. Please wait for it to complete.".to_string()
                );
            }
        }

        // Create job
        let job = BuildJob::new(layout_filename.clone(), keyboard.clone(), keymap.clone());
        let job_id = job.id.clone();

        // Store job
        {
            let mut jobs = self.jobs.write().unwrap();
            jobs.insert(job_id.clone(), job.clone());
        }

        // Create log file path
        let log_path = self.logs_dir.join(format!("{job_id}.log"));

        // Increment running count
        {
            let mut count = self.running_count.lock().unwrap();
            *count += 1;
        }

        // Send command to worker
        let cmd = BuildCommand {
            job_id,
            layout_filename,
            keyboard,
            keymap,
            qmk_path,
            log_path,
        };

        {
            let tx = self.command_tx.lock().unwrap();
            if let Some(sender) = tx.as_ref() {
                sender
                    .send(cmd)
                    .map_err(|e| format!("Failed to queue build: {e}"))?;
            }
        }

        Ok(job)
    }

    /// Gets the status of a job.
    pub fn get_job(&self, job_id: &str) -> Option<BuildJob> {
        self.jobs.read().unwrap().get(job_id).cloned()
    }

    /// Gets the logs for a job.
    pub fn get_logs(&self, job_id: &str, offset: usize, limit: usize) -> Option<JobLogsResponse> {
        // Check job exists
        if !self.jobs.read().unwrap().contains_key(job_id) {
            return None;
        }

        let log_path = self.logs_dir.join(format!("{job_id}.log"));

        let logs = if log_path.exists() {
            let file = File::open(&log_path).ok()?;
            let reader = BufReader::new(file);
            let mut entries: Vec<LogEntry> = Vec::new();

            for (idx, line) in reader.lines().enumerate() {
                if idx < offset {
                    continue;
                }
                if entries.len() >= limit {
                    break;
                }

                if let Ok(line_content) = line {
                    let (level, message) = parse_log_line(&line_content);
                    entries.push(LogEntry {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        level,
                        message,
                    });
                }
            }

            entries
        } else {
            Vec::new()
        };

        let has_more = {
            let log_path = self.logs_dir.join(format!("{job_id}.log"));
            if log_path.exists() {
                if let Ok(file) = File::open(&log_path) {
                    let reader = BufReader::new(file);
                    reader.lines().count() > offset + limit
                } else {
                    false
                }
            } else {
                false
            }
        };

        Some(JobLogsResponse {
            job_id: job_id.to_string(),
            logs,
            has_more,
        })
    }

    /// Cancels a running job.
    pub fn cancel_job(&self, job_id: &str) -> CancelJobResponse {
        // Check job exists
        let job = {
            let jobs = self.jobs.read().unwrap();
            jobs.get(job_id).cloned()
        };

        match job {
            Some(job) => {
                if job.status == JobStatus::Running || job.status == JobStatus::Pending {
                    // Mark as cancelled
                    self.cancelled.write().unwrap().insert(job_id.to_string());

                    // Update job status
                    self.update_job_status(job_id, JobStatus::Cancelled, None, None);

                    CancelJobResponse {
                        success: true,
                        message: "Build cancelled".to_string(),
                    }
                } else {
                    CancelJobResponse {
                        success: false,
                        message: format!("Cannot cancel job with status: {}", job.status),
                    }
                }
            }
            None => CancelJobResponse {
                success: false,
                message: "Job not found".to_string(),
            },
        }
    }

    /// Lists all jobs.
    pub fn list_jobs(&self) -> Vec<BuildJob> {
        let mut list: Vec<_> = self.jobs.read().unwrap().values().cloned().collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list
    }
}

/// Parses a log line into (level, message).
fn parse_log_line(line: &str) -> (String, String) {
    // Format: [LEVEL] message
    if let Some(rest) = line.strip_prefix('[') {
        if let Some(end_bracket) = rest.find(']') {
            let level = rest[..end_bracket].to_string();
            let message = rest[end_bracket + 1..].trim().to_string();
            return (level, message);
        }
    }
    ("INFO".to_string(), line.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_test_manager() -> Arc<BuildJobManager> {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_test_{}", Uuid::new_v4()));
        let mock_builder = Arc::new(MockFirmwareBuilder {
            build_duration_ms: 50,
            should_succeed: true,
            error_message: None,
        });
        BuildJobManager::with_builder(
            temp_dir.join("logs"),
            Some(PathBuf::from("/tmp/qmk")),
            mock_builder,
        )
    }

    #[test]
    fn test_job_status_display() {
        assert_eq!(JobStatus::Pending.to_string(), "pending");
        assert_eq!(JobStatus::Running.to_string(), "running");
        assert_eq!(JobStatus::Completed.to_string(), "completed");
        assert_eq!(JobStatus::Failed.to_string(), "failed");
        assert_eq!(JobStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_build_job_new() {
        let job = BuildJob::new(
            "test.md".to_string(),
            "crkbd".to_string(),
            "default".to_string(),
        );

        assert!(!job.id.is_empty());
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.layout_filename, "test.md");
        assert_eq!(job.keyboard, "crkbd");
        assert_eq!(job.keymap, "default");
        assert!(job.started_at.is_none());
        assert!(job.completed_at.is_none());
        assert!(job.error.is_none());
        assert!(job.firmware_path.is_none());
        assert_eq!(job.progress, 0);
    }

    #[test]
    fn test_start_build_success() {
        let manager = create_test_manager();

        let result = manager.start_build(
            "test.md".to_string(),
            "crkbd".to_string(),
            "default".to_string(),
        );

        assert!(result.is_ok());
        let job = result.unwrap();
        assert_eq!(job.layout_filename, "test.md");

        // Wait for build to complete
        thread::sleep(Duration::from_millis(200));

        let updated_job = manager.get_job(&job.id);
        assert!(updated_job.is_some());
        let updated = updated_job.unwrap();
        assert_eq!(updated.status, JobStatus::Completed);
        assert!(updated.firmware_path.is_some());
    }

    #[test]
    fn test_start_build_no_qmk_path() {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_test_{}", Uuid::new_v4()));
        let manager = BuildJobManager::with_builder(
            temp_dir,
            None, // No QMK path
            Arc::new(MockFirmwareBuilder::default()),
        );

        let result = manager.start_build(
            "test.md".to_string(),
            "crkbd".to_string(),
            "default".to_string(),
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("QMK firmware path not configured"));
    }

    #[test]
    fn test_cancel_job() {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_test_{}", Uuid::new_v4()));
        let mock_builder = Arc::new(MockFirmwareBuilder {
            build_duration_ms: 500, // Slow build to allow cancellation
            should_succeed: true,
            error_message: None,
        });
        let manager = BuildJobManager::with_builder(
            temp_dir.join("logs"),
            Some(PathBuf::from("/tmp/qmk")),
            mock_builder,
        );

        let job = manager
            .start_build(
                "test.md".to_string(),
                "crkbd".to_string(),
                "default".to_string(),
            )
            .unwrap();

        // Cancel immediately
        let result = manager.cancel_job(&job.id);
        assert!(result.success);

        // Wait for worker to process
        thread::sleep(Duration::from_millis(100));

        let updated = manager.get_job(&job.id).unwrap();
        assert_eq!(updated.status, JobStatus::Cancelled);
    }

    #[test]
    fn test_list_jobs() {
        let manager = create_test_manager();

        // Start two builds
        let _ = manager.start_build(
            "a.md".to_string(),
            "crkbd".to_string(),
            "default".to_string(),
        );
        thread::sleep(Duration::from_millis(10));
        let _ = manager.start_build("b.md".to_string(), "crkbd".to_string(), "test".to_string());

        let jobs = manager.list_jobs();
        // First job may still be running, second will be pending
        assert!(!jobs.is_empty());
    }

    #[test]
    fn test_parse_log_line() {
        let (level, msg) = parse_log_line("[INFO] Build started");
        assert_eq!(level, "INFO");
        assert_eq!(msg, "Build started");

        let (level, msg) = parse_log_line("[ERROR] Something went wrong");
        assert_eq!(level, "ERROR");
        assert_eq!(msg, "Something went wrong");

        let (level, msg) = parse_log_line("Plain message without level");
        assert_eq!(level, "INFO");
        assert_eq!(msg, "Plain message without level");
    }

    #[test]
    fn test_mock_builder_failure() {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_test_{}", Uuid::new_v4()));
        let mock_builder = Arc::new(MockFirmwareBuilder {
            build_duration_ms: 50,
            should_succeed: false,
            error_message: Some("Compilation error".to_string()),
        });
        let manager = BuildJobManager::with_builder(
            temp_dir.join("logs"),
            Some(PathBuf::from("/tmp/qmk")),
            mock_builder,
        );

        let job = manager
            .start_build(
                "test.md".to_string(),
                "crkbd".to_string(),
                "default".to_string(),
            )
            .unwrap();

        // Wait for build to complete
        thread::sleep(Duration::from_millis(200));

        let updated = manager.get_job(&job.id).unwrap();
        assert_eq!(updated.status, JobStatus::Failed);
        assert!(updated.error.is_some());
        assert!(updated.error.unwrap().contains("Compilation error"));
    }
}
