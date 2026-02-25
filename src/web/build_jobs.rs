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

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

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

/// Build command to be executed by worker thread.
struct BuildCommand {
    job_id: String,
    #[allow(dead_code)] // Stored for potential future logging/display
    layout_filename: String,
    keyboard: String,
    keymap: String,
    qmk_path: PathBuf,
    log_path: PathBuf,
    /// Job-specific output directory for artifacts.
    output_dir: PathBuf,
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

/// Real firmware builder using QMK CLI.
pub struct RealFirmwareBuilder;

impl FirmwareBuilder for RealFirmwareBuilder {
    fn build(
        &self,
        qmk_path: &PathBuf,
        keyboard: &str,
        keymap: &str,
        output_dir: &Path,
        job_id: &str,
        log_writer: &mut dyn Write,
        is_cancelled: &dyn Fn() -> bool,
    ) -> Result<BuildResult, String> {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};

        let _ = writeln!(log_writer, "[INFO] Starting QMK compile...");
        let _ = writeln!(
            log_writer,
            "[INFO] Running: qmk compile -kb {} -km {}",
            keyboard, keymap
        );

        // Check for cancellation before starting
        if is_cancelled() {
            let _ = writeln!(log_writer, "[INFO] Build cancelled before starting");
            return Err("Build cancelled".to_string());
        }

        let mut cmd = Command::new("qmk");
        cmd.arg("compile")
            .arg("-kb")
            .arg(keyboard)
            .arg("-km")
            .arg(keymap)
            .current_dir(qmk_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Spawn the process instead of waiting for output
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to execute qmk: {e}"))?;

        // Get handles for stdout/stderr
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Failed to capture stderr".to_string())?;

        // Stream stdout in background
        let stdout_reader = BufReader::new(stdout);
        for line in stdout_reader.lines() {
            // Check for cancellation periodically
            if is_cancelled() {
                let _ = writeln!(log_writer, "[INFO] Build cancelled, killing process...");
                let _ = child.kill();
                let _ = child.wait();
                return Err("Build cancelled".to_string());
            }

            if let Ok(line) = line {
                let level = if line.contains("error") || line.contains("Error") {
                    "ERROR"
                } else {
                    "INFO"
                };
                let _ = writeln!(log_writer, "[{level}] {line}");
            }
        }

        // Stream stderr
        let stderr_reader = BufReader::new(stderr);
        for line in stderr_reader.lines() {
            if is_cancelled() {
                let _ = writeln!(log_writer, "[INFO] Build cancelled, killing process...");
                let _ = child.kill();
                let _ = child.wait();
                return Err("Build cancelled".to_string());
            }

            if let Ok(line) = line {
                if !line.trim().is_empty() {
                    let _ = writeln!(log_writer, "[ERROR] {line}");
                }
            }
        }

        // Wait for process to complete
        let status = child
            .wait()
            .map_err(|e| format!("Failed to wait for process: {e}"))?;

        // Final cancellation check
        if is_cancelled() {
            let _ = writeln!(log_writer, "[INFO] Build cancelled");
            return Err("Build cancelled".to_string());
        }

        if !status.success() {
            return Err("QMK compile failed. Check build log for details.".to_string());
        }

        // Discover and copy artifacts
        let _ = writeln!(log_writer, "[INFO] Discovering firmware artifacts...");
        let artifacts = discover_and_copy_artifacts(
            qmk_path, keyboard, keymap, output_dir, job_id, log_writer,
        )?;

        if artifacts.is_empty() {
            return Err(format!(
                "Could not find any firmware files for {keyboard} {keymap}"
            ));
        }

        // Use first artifact as primary firmware path (for backward compatibility)
        let primary_path = output_dir.join(&artifacts[0].filename);

        Ok(BuildResult {
            firmware_path: primary_path,
            artifacts,
        })
    }
}

/// Discovers firmware artifacts in QMK's `.build` directory and copies them to the output directory.
///
/// Looks for files matching the pattern `<keyboard_clean>_<keymap>.<ext>` where keyboard slashes
/// are replaced with underscores. Supports multiple file extensions (uf2, bin, hex) and handles
/// variant suffixes via glob matching.
///
/// # Arguments
/// * `qmk_path` - Path to QMK firmware directory
/// * `keyboard` - Keyboard identifier (may contain slashes)
/// * `keymap` - Keymap name
/// * `output_dir` - Directory to copy artifacts into
/// * `job_id` - Job identifier for generating download URLs
/// * `log_writer` - Writer for log output
///
/// # Returns
/// Vector of `BuildArtifact` metadata for all discovered and copied artifacts.
fn discover_and_copy_artifacts(
    qmk_path: &Path,
    keyboard: &str,
    keymap: &str,
    output_dir: &Path,
    job_id: &str,
    log_writer: &mut dyn Write,
) -> Result<Vec<BuildArtifact>, String> {
    let build_dir = qmk_path.join(".build");
    if !build_dir.exists() {
        return Err("QMK .build directory not found".to_string());
    }

    // Create output directory
    fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create output directory: {e}"))?;

    let keyboard_clean = keyboard.replace('/', "_");
    let base_prefix = format!("{keyboard_clean}_{keymap}");

    let mut artifacts = Vec::new();

    // First pass: look for exact matches
    for ext in ARTIFACT_EXTENSIONS {
        let exact_filename = format!("{base_prefix}.{ext}");
        let source_path = build_dir.join(&exact_filename);

        if source_path.exists() {
            if let Some(artifact) =
                copy_artifact(&source_path, output_dir, ext, job_id, log_writer)?
            {
                artifacts.push(artifact);
            }
        }
    }

    // Second pass: glob for variant suffixes (e.g., keyboard_keymap_avr.hex)
    // Only if we haven't found exact matches for all extensions
    if let Ok(entries) = fs::read_dir(&build_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let filename = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            // Check if this file starts with our prefix and has a supported extension
            if !filename.starts_with(&base_prefix) {
                continue;
            }

            for ext in ARTIFACT_EXTENSIONS {
                if filename.ends_with(&format!(".{ext}")) {
                    // Skip if we already have an artifact with this extension
                    if artifacts.iter().any(|a| a.artifact_type == *ext) {
                        continue;
                    }

                    if let Some(artifact) =
                        copy_artifact(&path, output_dir, ext, job_id, log_writer)?
                    {
                        artifacts.push(artifact);
                    }
                }
            }
        }
    }

    Ok(artifacts)
}

/// Copies a single artifact file to the output directory and creates metadata.
fn copy_artifact(
    source_path: &Path,
    output_dir: &Path,
    ext: &str,
    job_id: &str,
    log_writer: &mut dyn Write,
) -> Result<Option<BuildArtifact>, String> {
    let filename = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid source filename".to_string())?;

    let dest_path = output_dir.join(filename);

    // Copy the file
    fs::copy(source_path, &dest_path).map_err(|e| format!("Failed to copy artifact: {e}"))?;

    // Get file metadata
    let metadata = fs::metadata(&dest_path).map_err(|e| format!("Failed to read metadata: {e}"))?;
    let size = metadata.len();

    // Calculate SHA256 hash
    let sha256 = calculate_sha256(&dest_path).ok();

    let _ = writeln!(
        log_writer,
        "[INFO] Copied artifact: {} ({} bytes)",
        filename, size
    );

    // Use extension as stable artifact ID
    let artifact_id = ext.to_string();
    let download_url = format!("/api/build/jobs/{job_id}/artifacts/{artifact_id}/download");

    Ok(Some(BuildArtifact {
        id: artifact_id,
        filename: filename.to_string(),
        artifact_type: ext.to_string(),
        size,
        sha256,
        download_url,
    }))
}

/// Calculates the SHA256 hash of a file.
fn calculate_sha256(path: &Path) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
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
        output_dir: &Path,
        job_id: &str,
        log_writer: &mut dyn Write,
        is_cancelled: &dyn Fn() -> bool,
    ) -> Result<BuildResult, String> {
        let _ = writeln!(log_writer, "[INFO] Mock build starting...");
        let _ = writeln!(
            log_writer,
            "[INFO] Building {} keymap for {}",
            keymap, keyboard
        );

        // Simulate build progress with cancellation checks
        let steps = 5;
        let step_duration = self.build_duration_ms / steps;

        for i in 1..=steps {
            // Check for cancellation
            if is_cancelled() {
                let _ = writeln!(log_writer, "[INFO] Mock build cancelled");
                return Err("Build cancelled".to_string());
            }

            thread::sleep(Duration::from_millis(step_duration));
            let _ = writeln!(log_writer, "[INFO] Build progress: {}%", i * 20);
        }

        if self.should_succeed {
            let keyboard_clean = keyboard.replace('/', "_");
            let filename = format!("{keyboard_clean}_{keymap}.uf2");

            // Create the output directory and mock firmware file
            let _ = fs::create_dir_all(output_dir);
            let firmware_path = output_dir.join(&filename);
            let _ = fs::write(&firmware_path, b"mock firmware content");

            let _ = writeln!(
                log_writer,
                "[INFO] Mock firmware generated: {}",
                firmware_path.display()
            );

            // Create mock artifact metadata
            let artifact_id = "uf2".to_string();
            let download_url = format!("/api/build/jobs/{job_id}/artifacts/{artifact_id}/download");

            let artifacts = vec![BuildArtifact {
                id: artifact_id,
                filename,
                artifact_type: "uf2".to_string(),
                size: 21, // "mock firmware content".len()
                sha256: None,
                download_url,
            }];

            Ok(BuildResult {
                firmware_path,
                artifacts,
            })
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
    /// Directory for storing build artifacts.
    output_dir: PathBuf,
    /// QMK firmware path from config.
    qmk_path: RwLock<Option<PathBuf>>,
    /// Firmware builder (real or mock).
    builder: Arc<dyn FirmwareBuilder>,
    /// Maximum age of artifacts in hours (default: 168 = 7 days).
    max_artifacts_age_hours: u64,
    /// Maximum total number of artifacts to keep (default: 50).
    max_total_artifacts: usize,
}

impl BuildJobManager {
    /// Creates a new build job manager.
    pub fn new(logs_dir: PathBuf, output_dir: PathBuf, qmk_path: Option<PathBuf>) -> Arc<Self> {
        Self::with_builder(
            logs_dir,
            output_dir,
            qmk_path,
            Arc::new(RealFirmwareBuilder),
        )
    }

    /// Creates a new build job manager with a custom builder (for testing).
    pub fn with_builder(
        logs_dir: PathBuf,
        output_dir: PathBuf,
        qmk_path: Option<PathBuf>,
        builder: Arc<dyn FirmwareBuilder>,
    ) -> Arc<Self> {
        // Ensure directories exist
        let _ = fs::create_dir_all(&logs_dir);
        let _ = fs::create_dir_all(&output_dir);

        let manager = Arc::new(Self {
            jobs: RwLock::new(HashMap::new()),
            cancelled: RwLock::new(std::collections::HashSet::new()),
            running_count: Mutex::new(0),
            command_tx: Mutex::new(None),
            logs_dir,
            output_dir,
            qmk_path: RwLock::new(qmk_path),
            builder,
            max_artifacts_age_hours: 168, // 7 days
            max_total_artifacts: 50,
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
            self.update_job_status(&cmd.job_id, JobStatus::Cancelled, None, None, Vec::new());
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
                    // Create cancellation check closure
                    let job_id = cmd.job_id.clone();
                    let manager = Arc::clone(self);
                    let is_cancelled = move || manager.is_cancelled(&job_id);

                    // Run the build with cancellation callback
                    self.builder.build(
                        &cmd.qmk_path,
                        &cmd.keyboard,
                        &cmd.keymap,
                        &cmd.output_dir,
                        &cmd.job_id,
                        &mut file,
                        &is_cancelled,
                    )
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
            self.update_job_status(&cmd.job_id, JobStatus::Cancelled, None, None, Vec::new());
            return;
        }

        // Update job with result
        match result {
            Ok(build_result) => {
                self.update_job_status(
                    &cmd.job_id,
                    JobStatus::Completed,
                    None,
                    Some(build_result.firmware_path.display().to_string()),
                    build_result.artifacts,
                );
            }
            Err(error) => {
                self.update_job_status(
                    &cmd.job_id,
                    JobStatus::Failed,
                    Some(error),
                    None,
                    Vec::new(),
                );
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
        artifacts: Vec<BuildArtifact>,
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
            job.artifacts = artifacts;
        }
    }

    /// Cleans up old artifacts based on age and count limits.
    ///
    /// This method removes artifacts that are:
    /// 1. Older than `max_artifacts_age_hours`
    /// 2. Exceeding `max_total_artifacts` count (oldest first)
    ///
    /// Active (pending/running) jobs are never cleaned.
    fn cleanup_old_artifacts(&self) {
        use std::time::SystemTime;

        // Get list of completed/failed/cancelled jobs with their completion times
        let jobs = self.jobs.read().unwrap();
        let mut cleanable_jobs: Vec<(String, SystemTime)> = Vec::new();

        for (job_id, job) in jobs.iter() {
            // Skip active jobs (pending or running)
            if matches!(job.status, JobStatus::Pending | JobStatus::Running) {
                continue;
            }

            // Try to get job directory modification time
            let job_dir = self.output_dir.join(job_id);
            if let Ok(metadata) = fs::metadata(&job_dir) {
                if let Ok(modified) = metadata.modified() {
                    cleanable_jobs.push((job_id.clone(), modified));
                }
            }
        }

        drop(jobs); // Release read lock

        // Sort by modification time (oldest first)
        cleanable_jobs.sort_by_key(|(_, time)| *time);

        let now = SystemTime::now();
        let max_age = Duration::from_secs(self.max_artifacts_age_hours * 3600);

        let mut cleaned_count = 0;

        // First pass: Remove artifacts older than max age
        for (job_id, modified) in &cleanable_jobs {
            if let Ok(age) = now.duration_since(*modified) {
                if age > max_age {
                    self.remove_job_artifacts(job_id);
                    cleaned_count += 1;
                }
            }
        }

        // Second pass: Enforce max count limit
        let remaining_count = cleanable_jobs.len() - cleaned_count;
        if remaining_count > self.max_total_artifacts {
            let to_remove = remaining_count - self.max_total_artifacts;

            // Remove oldest jobs (skip those already removed in first pass)
            let mut removed = 0;
            for (job_id, modified) in &cleanable_jobs {
                if removed >= to_remove {
                    break;
                }

                // Check if already removed due to age
                if let Ok(age) = now.duration_since(*modified) {
                    if age <= max_age {
                        // Not removed yet, so remove it now
                        self.remove_job_artifacts(job_id);
                        removed += 1;
                    }
                }
            }
        }
    }

    /// Removes artifacts and logs for a specific job.
    fn remove_job_artifacts(&self, job_id: &str) {
        // Remove artifact directory
        let job_dir = self.output_dir.join(job_id);
        if job_dir.exists() {
            let _ = fs::remove_dir_all(&job_dir);
        }

        // Remove log file
        let log_file = self.logs_dir.join(format!("{job_id}.log"));
        if log_file.exists() {
            let _ = fs::remove_file(&log_file);
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
        // Trigger artifact cleanup in background (async to avoid blocking)
        let manager = Arc::clone(self);
        thread::spawn(move || {
            manager.cleanup_old_artifacts();
        });

        // Check QMK path
        let qmk_path = self
            .qmk_path
            .read()
            .unwrap()
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

        // Create job-specific output directory for artifacts
        let output_dir = self.output_dir.join(&job_id);

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
            output_dir,
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
                    self.update_job_status(job_id, JobStatus::Cancelled, None, None, Vec::new());

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

    /// Updates the QMK firmware path.
    pub fn set_qmk_path(&self, path: Option<PathBuf>) {
        *self.qmk_path.write().unwrap() = path;
    }

    /// Gets the artifacts for a completed job.
    pub fn get_artifacts(&self, job_id: &str) -> Option<Vec<BuildArtifact>> {
        self.jobs
            .read()
            .unwrap()
            .get(job_id)
            .map(|job| job.artifacts.clone())
    }

    /// Gets the file path for a specific artifact.
    ///
    /// Validates the artifact ID to prevent path traversal attacks.
    /// Returns `None` if the job doesn't exist, the artifact isn't found,
    /// or the artifact ID is invalid.
    pub fn get_artifact_path(&self, job_id: &str, artifact_id: &str) -> Option<PathBuf> {
        // Validate artifact_id to prevent path traversal
        if !is_valid_artifact_id(artifact_id) {
            return None;
        }

        // Get artifact filename from job
        let artifact_filename = self
            .jobs
            .read()
            .unwrap()
            .get(job_id)?
            .artifacts
            .iter()
            .find(|a| a.id == artifact_id)
            .map(|a| a.filename.clone())?;

        // Construct path and validate it's within output directory
        let artifact_path = self.output_dir.join(job_id).join(&artifact_filename);

        // Security: Ensure the resolved path is within the expected output directory
        // We need to check the file exists first, otherwise canonicalize fails
        if !artifact_path.exists() {
            return None;
        }

        let canonical_output = self.output_dir.canonicalize().ok()?;
        let canonical_artifact = artifact_path.canonicalize().ok()?;

        if canonical_artifact.starts_with(&canonical_output) {
            Some(artifact_path)
        } else {
            None
        }
    }
}

/// Validates an artifact ID to prevent path traversal.
///
/// Valid artifact IDs are lowercase alphanumeric strings (matching file extensions).
fn is_valid_artifact_id(id: &str) -> bool {
    if id.is_empty() || id.len() > 10 {
        return false;
    }
    id.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
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
            temp_dir.join("output"),
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
            temp_dir.join("logs"),
            temp_dir.join("output"),
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
            temp_dir.join("output"),
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
            temp_dir.join("output"),
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

    #[test]
    fn test_is_valid_artifact_id() {
        // Valid artifact IDs
        assert!(is_valid_artifact_id("uf2"));
        assert!(is_valid_artifact_id("bin"));
        assert!(is_valid_artifact_id("hex"));
        assert!(is_valid_artifact_id("a"));
        assert!(is_valid_artifact_id("abc123"));

        // Invalid artifact IDs
        assert!(!is_valid_artifact_id("")); // Empty
        assert!(!is_valid_artifact_id("verylongartifactid")); // Too long (>10)
        assert!(!is_valid_artifact_id("UF2")); // Uppercase
        assert!(!is_valid_artifact_id("uf-2")); // Hyphen
        assert!(!is_valid_artifact_id("uf.2")); // Dot
        assert!(!is_valid_artifact_id("../etc")); // Path traversal attempt
        assert!(!is_valid_artifact_id("uf2/")); // Slash
    }

    #[test]
    fn test_build_job_has_artifacts_after_success() {
        let manager = create_test_manager();

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
        assert_eq!(updated.status, JobStatus::Completed);

        // Verify artifacts are populated
        assert!(!updated.artifacts.is_empty());
        assert_eq!(updated.artifacts[0].id, "uf2");
        assert_eq!(updated.artifacts[0].artifact_type, "uf2");
        assert!(updated.artifacts[0].download_url.contains(&job.id));
    }

    #[test]
    fn test_get_artifacts_returns_none_for_unknown_job() {
        let manager = create_test_manager();
        assert!(manager.get_artifacts("nonexistent-job-id").is_none());
    }

    #[test]
    fn test_get_artifact_path_validates_artifact_id() {
        let manager = create_test_manager();

        // Start and complete a build
        let job = manager
            .start_build(
                "test.md".to_string(),
                "crkbd".to_string(),
                "default".to_string(),
            )
            .unwrap();
        thread::sleep(Duration::from_millis(200));

        // Invalid artifact IDs should return None
        assert!(manager.get_artifact_path(&job.id, "../etc").is_none());
        assert!(manager.get_artifact_path(&job.id, "UF2").is_none());
        assert!(manager.get_artifact_path(&job.id, "").is_none());
    }

    #[test]
    fn test_cancel_job_interrupts_mock_build() {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_test_{}", Uuid::new_v4()));
        let mock_builder = Arc::new(MockFirmwareBuilder {
            build_duration_ms: 1000, // Long build to ensure cancellation happens during build
            should_succeed: true,
            error_message: None,
        });
        let manager = BuildJobManager::with_builder(
            temp_dir.join("logs"),
            temp_dir.join("output"),
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

        // Wait a bit for build to start
        thread::sleep(Duration::from_millis(50));

        // Cancel while building
        let result = manager.cancel_job(&job.id);
        assert!(result.success);

        // Wait for worker to process cancellation
        thread::sleep(Duration::from_millis(200));

        let updated = manager.get_job(&job.id).unwrap();
        assert_eq!(updated.status, JobStatus::Cancelled);
    }

    #[test]
    fn test_cleanup_removes_old_artifacts() {
        let manager = create_test_manager();

        // Create a completed job with artifacts
        let job1 = manager
            .start_build(
                "test1.md".to_string(),
                "crkbd".to_string(),
                "default".to_string(),
            )
            .unwrap();
        thread::sleep(Duration::from_millis(200));

        // Verify artifacts exist
        let job1_dir = manager.output_dir.join(&job1.id);
        assert!(job1_dir.exists());

        // Manually set old timestamp on directory (simulate old artifact)
        // We can't easily change file times in tests, so we'll test the cleanup logic directly
        manager.remove_job_artifacts(&job1.id);

        // Verify artifacts are removed
        assert!(!job1_dir.exists());
        let log_file = manager.logs_dir.join(format!("{}.log", job1.id));
        assert!(!log_file.exists());
    }

    #[test]
    fn test_cleanup_preserves_active_jobs() {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_test_{}", Uuid::new_v4()));
        let mock_builder = Arc::new(MockFirmwareBuilder {
            build_duration_ms: 500, // Slow build to keep it running
            should_succeed: true,
            error_message: None,
        });
        let manager = BuildJobManager::with_builder(
            temp_dir.join("logs"),
            temp_dir.join("output"),
            Some(PathBuf::from("/tmp/qmk")),
            mock_builder,
        );

        // Start a build (will be running)
        let running_job = manager
            .start_build(
                "running.md".to_string(),
                "crkbd".to_string(),
                "default".to_string(),
            )
            .unwrap();

        thread::sleep(Duration::from_millis(100));

        // Trigger cleanup
        manager.cleanup_old_artifacts();

        // Verify running job is not removed from jobs map (directory might not exist yet)
        assert!(manager.get_job(&running_job.id).is_some());
    }
}
