//! Build job manager — coordinates background firmware builds.
//!
//! Contains [`BuildJobManager`], its companion types ([`BuildCommand`],
//! [`DeployResult`]), and the full `impl` block with all public and
//! private methods.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::firmware::generator::FirmwareGenerator;
use crate::firmware::validator::FirmwareValidator;
use crate::keycode_db::KeycodeDb;
use crate::services::geometry::{self, GeometryContext};
use crate::services::LayoutService;

use super::CancelJobResponse;
use super::JobLogsResponse;
use super::MAX_CONCURRENT_BUILDS;
use super::{is_valid_artifact_id, parse_log_line};
use super::{BuildArtifact, BuildJob, FirmwareBuilder, JobStatus, LogEntry};

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

/// Result of deploying keymap files to the QMK tree.
enum DeployResult {
    /// We created the keymap directory and all files — safe to remove the entire directory.
    CreatedDirectory(PathBuf),
    /// Directory already existed — only remove the files we wrote.
    ExistingDirectory(PathBuf),
    /// Deployment was skipped (e.g., layout file missing in tests).
    Skipped,
}

/// Build command to be executed by worker thread.
struct BuildCommand {
    job_id: String,
    layout_filename: String,
    keyboard: String,
    keymap: String,
    qmk_path: PathBuf,
    log_path: PathBuf,
    /// Job-specific output directory for artifacts.
    output_dir: PathBuf,
    /// Path to the layout markdown file for keymap deployment.
    layout_path: PathBuf,
    /// Keycode database for firmware generation.
    keycode_db: Arc<KeycodeDb>,
}

// ---------------------------------------------------------------------------
// BuildJobManager
// ---------------------------------------------------------------------------

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
    pub(crate) logs_dir: PathBuf,
    /// Directory for storing build artifacts.
    pub(crate) output_dir: PathBuf,
    /// QMK firmware path from config.
    qmk_path: RwLock<Option<PathBuf>>,
    /// Firmware builder (real or mock).
    builder: Arc<dyn FirmwareBuilder>,
    /// Maximum age of artifacts in hours (default: 168 = 7 days).
    max_artifacts_age_hours: u64,
    /// Maximum total number of artifacts to keep (default: 50).
    max_total_artifacts: usize,
    /// Keycode database for firmware generation during keymap deployment.
    keycode_db: Arc<KeycodeDb>,
}

impl BuildJobManager {
    /// Locks the jobs map for writing. Recovers from a poisoned mutex
    /// (which only happens when a worker thread panicked mid-build) so
    /// other builds can continue.
    fn jobs_write(&self) -> std::sync::RwLockWriteGuard<'_, HashMap<String, BuildJob>> {
        self.jobs
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Locks the jobs map for reading. Recovers from a poisoned mutex.
    fn jobs_read(&self) -> std::sync::RwLockReadGuard<'_, HashMap<String, BuildJob>> {
        self.jobs
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Locks the cancelled set for reading.
    fn cancelled_read(&self) -> std::sync::RwLockReadGuard<'_, std::collections::HashSet<String>> {
        self.cancelled
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Locks the cancelled set for writing.
    fn cancelled_write(
        &self,
    ) -> std::sync::RwLockWriteGuard<'_, std::collections::HashSet<String>> {
        self.cancelled
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Locks the running count mutex.
    fn running_count_lock(&self) -> std::sync::MutexGuard<'_, usize> {
        self.running_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Locks the command channel sender slot.
    fn command_tx_lock(&self) -> std::sync::MutexGuard<'_, Option<mpsc::Sender<BuildCommand>>> {
        self.command_tx
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Locks the `qmk_path` setter.
    fn qmk_path_write(&self) -> std::sync::RwLockWriteGuard<'_, Option<PathBuf>> {
        self.qmk_path
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Creates a new build job manager.
    pub fn new(
        logs_dir: PathBuf,
        output_dir: PathBuf,
        qmk_path: Option<PathBuf>,
        keycode_db: Arc<KeycodeDb>,
    ) -> Arc<Self> {
        Self::with_builder(
            logs_dir,
            output_dir,
            qmk_path,
            Arc::new(super::RealFirmwareBuilder),
            keycode_db,
        )
    }

    /// Creates a new build job manager with a custom builder (for testing).
    pub fn with_builder(
        logs_dir: PathBuf,
        output_dir: PathBuf,
        qmk_path: Option<PathBuf>,
        builder: Arc<dyn FirmwareBuilder>,
        keycode_db: Arc<KeycodeDb>,
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
            keycode_db,
        });

        // Start worker thread
        manager.start_worker();

        manager
    }

    /// Starts the background worker thread.
    fn start_worker(self: &Arc<Self>) {
        let (tx, rx) = mpsc::channel::<BuildCommand>();
        *self.command_tx_lock() = Some(tx);

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
            let mut jobs = self.jobs_write();
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

        let (result, deploy_result) = match log_file {
            Ok(mut file) => {
                let _ = writeln!(file, "[INFO] Build started at {}", chrono::Utc::now());

                // Check if cancelled during setup
                if self.is_cancelled(&cmd.job_id) {
                    let _ = writeln!(file, "[INFO] Build cancelled by user");
                    (Err("Build cancelled".to_string()), DeployResult::Skipped)
                } else {
                    // Deploy keymap files before building
                    let _ = writeln!(file, "[INFO] Deploying keymap to QMK tree...");
                    match Self::deploy_keymap(&cmd, &mut file) {
                        Ok(deploy_res) => {
                            let _ = writeln!(file, "[INFO] Keymap deployed successfully");

                            // Create cancellation check closure
                            let job_id = cmd.job_id.clone();
                            let manager = Arc::clone(self);
                            let is_cancelled = move || manager.is_cancelled(&job_id);

                            // Run the build with cancellation callback
                            let build_result = self.builder.build(
                                &cmd.qmk_path,
                                &cmd.keyboard,
                                &cmd.keymap,
                                &cmd.output_dir,
                                &cmd.job_id,
                                &mut file,
                                &is_cancelled,
                            );

                            (build_result, deploy_res)
                        }
                        Err(e) => {
                            let _ = writeln!(file, "[ERROR] Keymap deployment failed: {e}");
                            (
                                Err(format!("Keymap deployment failed: {e}")),
                                DeployResult::Skipped,
                            )
                        }
                    }
                }
            }
            Err(e) => (
                Err(format!("Failed to open log file: {e}")),
                DeployResult::Skipped,
            ),
        };

        // Clean up deployed keymap (before decrementing running_count to prevent race)
        match &deploy_result {
            DeployResult::CreatedDirectory(dir) => {
                if dir.exists() {
                    if let Err(e) = fs::remove_dir_all(dir) {
                        tracing::warn!(
                            dir = %dir.display(),
                            error = %e,
                            "failed to clean up created keymap directory"
                        );
                    }
                }
            }
            DeployResult::ExistingDirectory(dir) => {
                // Only remove files we created
                let keymap_c_path = dir.join("keymap.c");
                if keymap_c_path.exists() {
                    if let Err(e) = fs::remove_file(&keymap_c_path) {
                        tracing::warn!(error = %e, "failed to clean up keymap.c");
                    }
                }
                let config_h_path = dir.join("config.h");
                if config_h_path.exists() {
                    if let Err(e) = fs::remove_file(&config_h_path) {
                        tracing::warn!(error = %e, "failed to clean up config.h");
                    }
                }
            }
            DeployResult::Skipped => {}
        }

        // Decrement running count
        {
            let mut count = self.running_count_lock();
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

    /// Deploys keymap files (keymap.c, config.h) into the QMK firmware tree.
    ///
    /// This generates the firmware source files from the layout and writes them
    /// to `qmk_firmware/keyboards/<keyboard>/keymaps/<keymap>/` so that
    /// `qmk compile` can find them.
    ///
    /// Returns a `DeployResult` indicating what was created, so that cleanup
    /// can be done safely without removing pre-existing user files.
    fn deploy_keymap(
        cmd: &BuildCommand,
        log_writer: &mut dyn Write,
    ) -> Result<DeployResult, String> {
        // Skip deployment if the layout file doesn't exist (e.g. in tests with mock builders).
        // In production, the HTTP handler validates file existence before starting a build.
        if !cmd.layout_path.exists() {
            let _ = writeln!(
                log_writer,
                "[WARN] Layout file not found, skipping keymap deployment: {}",
                cmd.layout_path.display()
            );
            return Ok(DeployResult::Skipped);
        }

        // Load the layout
        let _ = writeln!(log_writer, "[INFO] Loading layout: {}", cmd.layout_filename);
        let layout = LayoutService::load(&cmd.layout_path)
            .map_err(|e| format!("Failed to load layout: {e}"))?;

        // Get layout variant
        let layout_variant = layout
            .metadata
            .layout_variant
            .as_ref()
            .ok_or("Layout has no layout variant defined")?;

        let _ = writeln!(log_writer, "[INFO] Layout variant: {layout_variant}");

        // Build config with QMK path
        let mut config = Config::load().unwrap_or_default();
        config.paths.qmk_firmware = Some(cmd.qmk_path.clone());

        // Build geometry
        let _ = writeln!(log_writer, "[INFO] Building keyboard geometry...");
        let geo_context = GeometryContext {
            config: &config,
            metadata: &layout.metadata,
        };

        let geo_result = geometry::build_geometry_for_layout(geo_context, layout_variant)
            .map_err(|e| format!("Failed to build geometry: {e}"))?;
        let geometry = geo_result.geometry;
        let mapping = geo_result.mapping;

        // Validate layout
        let _ = writeln!(log_writer, "[INFO] Validating layout...");
        let validator = FirmwareValidator::new(&layout, &geometry, &mapping, &cmd.keycode_db);
        let report = validator
            .validate()
            .map_err(|e| format!("Validation failed: {e}"))?;

        if !report.is_valid() {
            let _ = writeln!(log_writer, "[ERROR] Layout validation failed:");
            let _ = writeln!(log_writer, "[ERROR] {}", report.format_message());
            return Err(format!(
                "Layout validation failed: {}",
                report.format_message()
            ));
        }
        let _ = writeln!(log_writer, "[INFO] Layout validation passed");

        // Generate firmware files
        let _ = writeln!(log_writer, "[INFO] Generating firmware files...");
        let generator =
            FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &cmd.keycode_db);

        let keymap_c = generator
            .generate_keymap_c()
            .map_err(|e| format!("Failed to generate keymap.c: {e}"))?;
        let config_h = generator
            .generate_merged_config_h()
            .map_err(|e| format!("Failed to generate config.h: {e}"))?;

        let _ = writeln!(
            log_writer,
            "[INFO] Generated keymap.c ({} bytes)",
            keymap_c.len()
        );
        let _ = writeln!(
            log_writer,
            "[INFO] Generated config.h ({} bytes)",
            config_h.len()
        );

        // Compute keymap directory
        let keymap_dir = cmd
            .qmk_path
            .join("keyboards")
            .join(&cmd.keyboard)
            .join("keymaps")
            .join(&cmd.keymap);

        // Check if directory already exists
        let dir_existed = keymap_dir.exists();

        // Create directory if needed
        fs::create_dir_all(&keymap_dir)
            .map_err(|e| format!("Failed to create keymap directory: {e}"))?;

        // Write files to QMK tree
        let keymap_c_path = keymap_dir.join("keymap.c");
        fs::write(&keymap_c_path, &keymap_c)
            .map_err(|e| format!("Failed to write keymap.c: {e}"))?;

        let config_h_path = keymap_dir.join("config.h");
        fs::write(&config_h_path, &config_h)
            .map_err(|e| format!("Failed to write config.h: {e}"))?;

        let _ = writeln!(
            log_writer,
            "[INFO] Keymap deployed to {}",
            keymap_dir.display()
        );

        // Return appropriate result based on whether directory existed
        if dir_existed {
            Ok(DeployResult::ExistingDirectory(keymap_dir))
        } else {
            Ok(DeployResult::CreatedDirectory(keymap_dir))
        }
    }

    /// Checks if a job has been cancelled.
    fn is_cancelled(&self, job_id: &str) -> bool {
        self.cancelled_read().contains(job_id)
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
        let mut jobs = self.jobs_write();
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
    pub(crate) fn cleanup_old_artifacts(&self) {
        use std::time::SystemTime;

        // Get list of completed/failed/cancelled jobs with their completion times
        let jobs = self.jobs_read();
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
    pub(crate) fn remove_job_artifacts(&self, job_id: &str) {
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
        layout_path: PathBuf,
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
            let count = self.running_count_lock();
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
            let mut jobs = self.jobs_write();
            jobs.insert(job_id.clone(), job.clone());
        }

        // Create log file path
        let log_path = self.logs_dir.join(format!("{job_id}.log"));

        // Create job-specific output directory for artifacts
        let output_dir = self.output_dir.join(&job_id);

        // Increment running count
        {
            let mut count = self.running_count_lock();
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
            layout_path,
            keycode_db: Arc::clone(&self.keycode_db),
        };

        {
            let tx = self.command_tx_lock();
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
        self.jobs_read().get(job_id).cloned()
    }

    /// Gets the logs for a job.
    pub fn get_logs(&self, job_id: &str, offset: usize, limit: usize) -> Option<JobLogsResponse> {
        // Check job exists
        if !self.jobs_read().contains_key(job_id) {
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
            let jobs = self.jobs_read();
            jobs.get(job_id).cloned()
        };

        match job {
            Some(job) => {
                if job.status == JobStatus::Running || job.status == JobStatus::Pending {
                    // Mark as cancelled
                    self.cancelled_write().insert(job_id.to_string());

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
        let mut list: Vec<_> = self.jobs_read().values().cloned().collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list
    }

    /// Updates the QMK firmware path.
    pub fn set_qmk_path(&self, path: Option<PathBuf>) {
        *self.qmk_path_write() = path;
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
