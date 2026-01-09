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

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::config::Config;
use crate::firmware::generator::FirmwareGenerator;
use crate::firmware::validator::FirmwareValidator;
use crate::keycode_db::KeycodeDb;
use crate::services::geometry::{self, GeometryContext};
use crate::services::LayoutService;

/// Maximum number of concurrent generate jobs.
const MAX_CONCURRENT_JOBS: usize = 1;

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
    fn new(layout_filename: String, keyboard: String, layout_variant: String) -> Self {
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

/// Generate command to be executed by worker thread.
pub(crate) struct GenerateCommand {
    job_id: String,
    layout_filename: String,
    layout_path: PathBuf,
    #[allow(dead_code)] // Stored for potential future logging/display
    workspace_root: PathBuf,
    qmk_path: PathBuf,
    log_path: PathBuf,
    output_dir: PathBuf,
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

/// Real generate worker that produces firmware files.
pub(crate) struct RealGenerateWorker;

impl GenerateWorker for RealGenerateWorker {
    fn generate(
        &self,
        cmd: &GenerateCommand,
        log_writer: &mut dyn Write,
        keycode_db: &KeycodeDb,
    ) -> Result<PathBuf, String> {
        let _ = writeln!(log_writer, "[INFO] Starting firmware generation...");
        let _ = writeln!(log_writer, "[INFO] Layout: {}", cmd.layout_filename);

        // Load the layout
        let _ = writeln!(log_writer, "[INFO] Loading layout...");
        let layout = LayoutService::load(&cmd.layout_path)
            .map_err(|e| format!("Failed to load layout: {e}"))?;

        // Get keyboard and layout variant
        let keyboard = layout
            .metadata
            .keyboard
            .as_ref()
            .ok_or("Layout has no keyboard defined")?;
        let layout_variant = layout
            .metadata
            .layout_variant
            .as_ref()
            .ok_or("Layout has no layout variant defined")?;

        let _ = writeln!(log_writer, "[INFO] Keyboard: {keyboard}");
        let _ = writeln!(log_writer, "[INFO] Layout variant: {layout_variant}");

        // Build config with QMK path
        let mut config = Config::load().unwrap_or_default();
        config.paths.qmk_firmware = Some(cmd.qmk_path.clone());
        config.build.output_dir.clone_from(&cmd.output_dir);

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
        let validator = FirmwareValidator::new(&layout, &geometry, &mapping, keycode_db);
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
        let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, keycode_db);

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

        // Create output directory
        fs::create_dir_all(&cmd.output_dir)
            .map_err(|e| format!("Failed to create output directory: {e}"))?;

        // Read layout source file
        let layout_source = fs::read_to_string(&cmd.layout_path)
            .map_err(|e| format!("Failed to read layout source: {e}"))?;

        // Read logs so far
        let logs_content = fs::read_to_string(&cmd.log_path).unwrap_or_default();

        // Create manifest
        let manifest = serde_json::json!({
            "version": "1.0",
            "generator": "lazyqmk",
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "layout": {
                "name": layout.metadata.name,
                "filename": cmd.layout_filename,
                "keyboard": keyboard,
                "layout_variant": layout_variant,
            },
            "files": [
                "keymap.c",
                "config.h",
                "layout.md",
                "generate.log",
                "manifest.json"
            ]
        });

        // Create zip file
        let keyboard_clean = keyboard.replace('/', "_");
        let zip_filename = format!("{}_firmware.zip", keyboard_clean);
        let zip_path = cmd.output_dir.join(&zip_filename);

        let _ = writeln!(log_writer, "[INFO] Creating zip archive: {}", zip_filename);

        create_firmware_zip(
            &zip_path,
            &keymap_c,
            &config_h,
            &layout_source,
            &logs_content,
            &manifest,
        )?;

        let _ = writeln!(
            log_writer,
            "[INFO] Firmware generation completed successfully"
        );
        let _ = writeln!(log_writer, "[INFO] Output: {}", zip_path.display());

        Ok(zip_path)
    }
}

/// Creates a firmware zip archive with safe filename handling.
fn create_firmware_zip(
    zip_path: &Path,
    keymap_c: &str,
    config_h: &str,
    layout_source: &str,
    logs: &str,
    manifest: &serde_json::Value,
) -> Result<(), String> {
    let file = File::create(zip_path).map_err(|e| format!("Failed to create zip file: {e}"))?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Add files with safe, fixed names (no user input in filenames)
    add_file_to_zip(&mut zip, "keymap.c", keymap_c.as_bytes(), options)?;
    add_file_to_zip(&mut zip, "config.h", config_h.as_bytes(), options)?;
    add_file_to_zip(&mut zip, "layout.md", layout_source.as_bytes(), options)?;
    add_file_to_zip(&mut zip, "generate.log", logs.as_bytes(), options)?;

    let manifest_str = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize manifest: {e}"))?;
    add_file_to_zip(&mut zip, "manifest.json", manifest_str.as_bytes(), options)?;

    zip.finish()
        .map_err(|e| format!("Failed to finalize zip: {e}"))?;

    Ok(())
}

/// Adds a file to a zip archive with zip-slip prevention.
fn add_file_to_zip(
    zip: &mut ZipWriter<File>,
    name: &str,
    content: &[u8],
    options: SimpleFileOptions,
) -> Result<(), String> {
    // Validate filename for zip-slip prevention
    if name.contains("..") || name.starts_with('/') || name.starts_with('\\') {
        return Err(format!("Invalid filename in zip: {name}"));
    }

    zip.start_file(name, options)
        .map_err(|e| format!("Failed to start file {name}: {e}"))?;
    zip.write_all(content)
        .map_err(|e| format!("Failed to write file {name}: {e}"))?;

    Ok(())
}

/// Mock generate worker for testing.
#[allow(dead_code)] // Used in tests and with_mock_builder
pub(crate) struct MockGenerateWorker {
    /// Simulated generation duration in milliseconds.
    pub duration_ms: u64,
    /// Whether the generation should succeed.
    pub should_succeed: bool,
    /// Error message if generation should fail.
    pub error_message: Option<String>,
}

impl Default for MockGenerateWorker {
    fn default() -> Self {
        Self {
            duration_ms: 100,
            should_succeed: true,
            error_message: None,
        }
    }
}

impl GenerateWorker for MockGenerateWorker {
    fn generate(
        &self,
        cmd: &GenerateCommand,
        log_writer: &mut dyn Write,
        _keycode_db: &KeycodeDb,
    ) -> Result<PathBuf, String> {
        let _ = writeln!(log_writer, "[INFO] Mock generation starting...");
        let _ = writeln!(log_writer, "[INFO] Layout: {}", cmd.layout_filename);

        // Simulate generation progress
        let steps = 5;
        let step_duration = self.duration_ms / steps;

        for i in 1..=steps {
            thread::sleep(std::time::Duration::from_millis(step_duration));
            let _ = writeln!(log_writer, "[INFO] Generation progress: {}%", i * 20);
        }

        if self.should_succeed {
            // Create a mock zip file
            fs::create_dir_all(&cmd.output_dir)
                .map_err(|e| format!("Failed to create output dir: {e}"))?;
            let zip_path = cmd.output_dir.join("mock_firmware.zip");

            // Create minimal mock zip
            let file =
                File::create(&zip_path).map_err(|e| format!("Failed to create mock zip: {e}"))?;
            let mut zip = ZipWriter::new(file);
            let options = SimpleFileOptions::default();
            zip.start_file("keymap.c", options)
                .map_err(|e| format!("Failed to add file: {e}"))?;
            zip.write_all(b"// Mock keymap\n")
                .map_err(|e| format!("Failed to write: {e}"))?;
            zip.finish()
                .map_err(|e| format!("Failed to finish zip: {e}"))?;

            let _ = writeln!(
                log_writer,
                "[INFO] Mock generation completed: {}",
                zip_path.display()
            );
            Ok(zip_path)
        } else {
            let err = self
                .error_message
                .clone()
                .unwrap_or_else(|| "Mock generation failed".to_string());
            let _ = writeln!(log_writer, "[ERROR] {}", err);
            Err(err)
        }
    }
}

/// Generate job manager that coordinates background generation.
pub struct GenerateJobManager {
    /// All jobs indexed by ID.
    jobs: RwLock<HashMap<String, GenerateJob>>,
    /// Set of cancelled job IDs.
    cancelled: RwLock<std::collections::HashSet<String>>,
    /// Number of currently running jobs.
    running_count: Mutex<usize>,
    /// Channel sender for generate commands.
    command_tx: Mutex<Option<mpsc::Sender<GenerateCommand>>>,
    /// Directory for storing job logs.
    logs_dir: PathBuf,
    /// Directory for storing generated output.
    output_dir: PathBuf,
    /// Workspace root directory.
    workspace_root: PathBuf,
    /// QMK firmware path from config.
    qmk_path: Option<PathBuf>,
    /// Generate worker (real or mock).
    worker: Arc<dyn GenerateWorker>,
    /// Keycode database.
    keycode_db: Arc<KeycodeDb>,
}

impl GenerateJobManager {
    /// Creates a new generate job manager.
    pub fn new(
        logs_dir: PathBuf,
        output_dir: PathBuf,
        workspace_root: PathBuf,
        qmk_path: Option<PathBuf>,
        keycode_db: Arc<KeycodeDb>,
    ) -> Arc<Self> {
        Self::with_worker(
            logs_dir,
            output_dir,
            workspace_root,
            qmk_path,
            keycode_db,
            Arc::new(RealGenerateWorker),
        )
    }

    /// Creates a new generate job manager with a custom worker (for testing).
    pub(crate) fn with_worker(
        logs_dir: PathBuf,
        output_dir: PathBuf,
        workspace_root: PathBuf,
        qmk_path: Option<PathBuf>,
        keycode_db: Arc<KeycodeDb>,
        worker: Arc<dyn GenerateWorker>,
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
            workspace_root,
            qmk_path,
            worker,
            keycode_db,
        });

        // Start worker thread
        manager.start_worker();

        manager
    }

    /// Starts the background worker thread.
    fn start_worker(self: &Arc<Self>) {
        let (tx, rx) = mpsc::channel::<GenerateCommand>();
        *self.command_tx.lock().unwrap() = Some(tx);

        let manager = Arc::clone(self);

        eprintln!("[INFO] Generate worker thread starting");

        thread::spawn(move || {
            eprintln!("[INFO] Generate worker thread started, waiting for jobs");
            for cmd in rx {
                eprintln!("[INFO] Worker received job {} for processing", cmd.job_id);
                manager.process_generate(cmd);
            }
            eprintln!("[INFO] Generate worker thread stopped (channel closed)");
        });
    }

    /// Processes a generate command.
    fn process_generate(self: &Arc<Self>, cmd: GenerateCommand) {
        eprintln!(
            "[INFO] Processing job {}: transition Pending → Running",
            cmd.job_id
        );

        // Check if cancelled before starting
        if self.is_cancelled(&cmd.job_id) {
            eprintln!("[INFO] Job {} was cancelled before processing", cmd.job_id);
            self.update_job_status(&cmd.job_id, GenerateJobStatus::Cancelled, None, None);
            return;
        }

        // Update job to running
        {
            let mut jobs = self.jobs.write().unwrap();
            if let Some(job) = jobs.get_mut(&cmd.job_id) {
                job.status = GenerateJobStatus::Running;
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
                let _ = writeln!(file, "[INFO] Generate started at {}", chrono::Utc::now());

                // Check if cancelled during setup
                if self.is_cancelled(&cmd.job_id) {
                    let _ = writeln!(file, "[INFO] Generation cancelled by user");
                    Err("Generation cancelled".to_string())
                } else {
                    // Run the generation
                    self.worker.generate(&cmd, &mut file, &self.keycode_db)
                }
            }
            Err(e) => Err(format!("Failed to open log file: {e}")),
        };

        // Decrement running count
        {
            let mut count = self.running_count.lock().unwrap();
            *count = count.saturating_sub(1);
        }

        // Check if cancelled after generation
        if self.is_cancelled(&cmd.job_id) {
            eprintln!(
                "[INFO] Job {} completed but was cancelled, marking as Cancelled",
                cmd.job_id
            );
            self.update_job_status(&cmd.job_id, GenerateJobStatus::Cancelled, None, None);
            return;
        }

        // Update job with result
        match result {
            Ok(zip_path) => {
                eprintln!(
                    "[INFO] Job {} completed successfully: transition Running → Completed",
                    cmd.job_id
                );
                self.update_job_status(
                    &cmd.job_id,
                    GenerateJobStatus::Completed,
                    None,
                    Some(zip_path.display().to_string()),
                );
            }
            Err(error) => {
                eprintln!(
                    "[WARN] Job {} failed: transition Running → Failed ({})",
                    cmd.job_id, error
                );
                self.update_job_status(&cmd.job_id, GenerateJobStatus::Failed, Some(error), None);
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
        status: GenerateJobStatus,
        error: Option<String>,
        zip_path: Option<String>,
    ) {
        let mut jobs = self.jobs.write().unwrap();
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = status;
            job.completed_at = Some(chrono::Utc::now().to_rfc3339());
            job.progress = if status == GenerateJobStatus::Completed {
                100
            } else {
                0
            };
            job.error = error;
            job.zip_path = zip_path;
        }
    }

    /// Starts a new generate job.
    ///
    /// Returns the created job or an error if generation cannot be started.
    pub fn start_generate(
        self: &Arc<Self>,
        layout_filename: String,
        keyboard: String,
        layout_variant: String,
    ) -> Result<GenerateJob, String> {
        // Check QMK path
        let qmk_path = self
            .qmk_path
            .clone()
            .ok_or_else(|| "QMK firmware path not configured".to_string())?;

        // Check concurrency limit
        {
            let count = self.running_count.lock().unwrap();
            if *count >= MAX_CONCURRENT_JOBS {
                return Err(
                    "Generation already in progress. Please wait for it to complete.".to_string(),
                );
            }
        }

        // Build layout path
        let layout_path = self.workspace_root.join(&layout_filename);
        if !layout_path.exists() {
            return Err(format!("Layout file not found: {layout_filename}"));
        }

        // Create job
        let job = GenerateJob::new(layout_filename.clone(), keyboard, layout_variant);
        let job_id = job.id.clone();

        // Store job
        {
            let mut jobs = self.jobs.write().unwrap();
            jobs.insert(job_id.clone(), job.clone());
        }

        // Create log file path
        let log_path = self.logs_dir.join(format!("{job_id}.log"));

        // Create job-specific output directory
        let job_output_dir = self.output_dir.join(&job_id);

        // Increment running count
        {
            let mut count = self.running_count.lock().unwrap();
            *count += 1;
        }

        // Send command to worker
        let cmd = GenerateCommand {
            job_id: job_id.clone(),
            layout_filename,
            layout_path,
            workspace_root: self.workspace_root.clone(),
            qmk_path,
            log_path: log_path.clone(),
            output_dir: job_output_dir,
        };

        // Check if worker is running and send command
        let send_result = {
            let tx = self.command_tx.lock().unwrap();
            match tx.as_ref() {
                None => {
                    eprintln!(
                        "[ERROR] Generate worker not running for job {}, cannot enqueue",
                        job_id
                    );
                    Err("Generate worker not running".to_string())
                }
                Some(sender) => sender
                    .send(cmd)
                    .map_err(|e| format!("Failed to queue generation: {e}")),
            }
        };

        // Handle send failure - rollback state and write error log
        if let Err(error_msg) = send_result {
            // Rollback running count
            {
                let mut count = self.running_count.lock().unwrap();
                *count = count.saturating_sub(1);
            }

            // Write error to log file
            self.write_error_log(&log_path, &error_msg);

            // Mark job as failed
            self.update_job_status(
                &job_id,
                GenerateJobStatus::Failed,
                Some(error_msg.clone()),
                None,
            );

            eprintln!("[WARN] Job {} failed to enqueue: {}", job_id, error_msg);
            return Err(error_msg);
        }

        eprintln!(
            "[INFO] Job {} queued successfully for layout {}",
            job_id, job.layout_filename
        );
        Ok(job)
    }

    /// Writes an error message to the job log file.
    fn write_error_log(&self, log_path: &std::path::Path, error: &str) {
        use std::io::Write;
        let timestamp = chrono::Utc::now().to_rfc3339();
        let log_entry = format!("[ERROR] {timestamp} {error}\n");
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .and_then(|mut file| file.write_all(log_entry.as_bytes()));
    }

    /// Gets the status of a job.
    pub fn get_job(&self, job_id: &str) -> Option<GenerateJob> {
        self.jobs.read().unwrap().get(job_id).cloned()
    }

    /// Gets the zip file path for a completed job.
    pub fn get_zip_path(&self, job_id: &str) -> Option<PathBuf> {
        let jobs = self.jobs.read().unwrap();
        jobs.get(job_id).and_then(|job| {
            if job.status == GenerateJobStatus::Completed {
                job.zip_path.as_ref().map(PathBuf::from)
            } else {
                None
            }
        })
    }

    /// Gets the logs for a job.
    pub fn get_logs(
        &self,
        job_id: &str,
        offset: usize,
        limit: usize,
    ) -> Option<GenerateJobLogsResponse> {
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

        Some(GenerateJobLogsResponse {
            job_id: job_id.to_string(),
            logs,
            has_more,
        })
    }

    /// Cancels a running job.
    pub fn cancel_job(&self, job_id: &str) -> CancelGenerateJobResponse {
        // Check job exists
        let job = {
            let jobs = self.jobs.read().unwrap();
            jobs.get(job_id).cloned()
        };

        match job {
            Some(job) => {
                if job.status == GenerateJobStatus::Running
                    || job.status == GenerateJobStatus::Pending
                {
                    // Mark as cancelled
                    self.cancelled.write().unwrap().insert(job_id.to_string());

                    // Update job status
                    self.update_job_status(job_id, GenerateJobStatus::Cancelled, None, None);

                    CancelGenerateJobResponse {
                        success: true,
                        message: "Generation cancelled".to_string(),
                    }
                } else {
                    CancelGenerateJobResponse {
                        success: false,
                        message: format!("Cannot cancel job with status: {}", job.status),
                    }
                }
            }
            None => CancelGenerateJobResponse {
                success: false,
                message: "Job not found".to_string(),
            },
        }
    }

    /// Gets the health status of the generate job system.
    ///
    /// Returns information about whether the worker is running and current capacity.
    pub fn health(&self) -> GenerateJobHealth {
        let worker_running = self.command_tx.lock().unwrap().is_some();
        let running_count = *self.running_count.lock().unwrap();

        GenerateJobHealth {
            worker_running,
            running_count,
            max_concurrent_jobs: MAX_CONCURRENT_JOBS,
        }
    }

    /// Lists all jobs.
    pub fn list_jobs(&self) -> Vec<GenerateJob> {
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

    fn create_test_manager(workspace_root: &Path) -> Arc<GenerateJobManager> {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_gen_test_{}", Uuid::new_v4()));
        let mock_worker = Arc::new(MockGenerateWorker {
            duration_ms: 50,
            should_succeed: true,
            error_message: None,
        });
        let keycode_db = Arc::new(KeycodeDb::load().unwrap());
        GenerateJobManager::with_worker(
            temp_dir.join("logs"),
            temp_dir.join("output"),
            workspace_root.to_path_buf(),
            Some(PathBuf::from("/tmp/qmk")),
            keycode_db,
            mock_worker,
        )
    }

    #[test]
    fn test_job_status_display() {
        assert_eq!(GenerateJobStatus::Pending.to_string(), "pending");
        assert_eq!(GenerateJobStatus::Running.to_string(), "running");
        assert_eq!(GenerateJobStatus::Completed.to_string(), "completed");
        assert_eq!(GenerateJobStatus::Failed.to_string(), "failed");
        assert_eq!(GenerateJobStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_generate_job_new() {
        let job = GenerateJob::new(
            "test.md".to_string(),
            "crkbd".to_string(),
            "LAYOUT_split_3x6_3".to_string(),
        );

        assert!(!job.id.is_empty());
        assert_eq!(job.status, GenerateJobStatus::Pending);
        assert_eq!(job.layout_filename, "test.md");
        assert_eq!(job.keyboard, "crkbd");
        assert_eq!(job.layout_variant, "LAYOUT_split_3x6_3");
        assert!(job.started_at.is_none());
        assert!(job.completed_at.is_none());
        assert!(job.error.is_none());
        assert!(job.zip_path.is_none());
        assert!(job.download_url.is_some());
        assert_eq!(job.progress, 0);
    }

    #[test]
    fn test_start_generate_no_qmk_path() {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_gen_test_{}", Uuid::new_v4()));
        let keycode_db = Arc::new(KeycodeDb::load().unwrap());
        let manager = GenerateJobManager::with_worker(
            temp_dir.join("logs"),
            temp_dir.join("output"),
            temp_dir.clone(),
            None, // No QMK path
            keycode_db,
            Arc::new(MockGenerateWorker::default()),
        );

        let result = manager.start_generate(
            "test.md".to_string(),
            "crkbd".to_string(),
            "LAYOUT_split_3x6_3".to_string(),
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("QMK firmware path not configured"));
    }

    #[test]
    fn test_start_generate_file_not_found() {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_gen_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();
        let manager = create_test_manager(&temp_dir);

        let result = manager.start_generate(
            "nonexistent.md".to_string(),
            "crkbd".to_string(),
            "LAYOUT_split_3x6_3".to_string(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_start_generate_success() {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_gen_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a dummy layout file
        let layout_path = temp_dir.join("test.md");
        fs::write(&layout_path, "---\nname: Test\n---\n").unwrap();

        let manager = create_test_manager(&temp_dir);

        let result = manager.start_generate(
            "test.md".to_string(),
            "crkbd".to_string(),
            "LAYOUT_split_3x6_3".to_string(),
        );

        assert!(result.is_ok());
        let job = result.unwrap();
        assert_eq!(job.layout_filename, "test.md");

        // Wait for mock generation to complete
        thread::sleep(Duration::from_millis(200));

        let updated_job = manager.get_job(&job.id);
        assert!(updated_job.is_some());
        let updated = updated_job.unwrap();
        assert_eq!(updated.status, GenerateJobStatus::Completed);
        assert!(updated.zip_path.is_some());
    }

    #[test]
    fn test_cancel_job() {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_gen_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a dummy layout file
        let layout_path = temp_dir.join("test.md");
        fs::write(&layout_path, "---\nname: Test\n---\n").unwrap();

        let keycode_db = Arc::new(KeycodeDb::load().unwrap());
        let mock_worker = Arc::new(MockGenerateWorker {
            duration_ms: 500, // Slow to allow cancellation
            should_succeed: true,
            error_message: None,
        });
        let manager = GenerateJobManager::with_worker(
            temp_dir.join("logs"),
            temp_dir.join("output"),
            temp_dir.clone(),
            Some(PathBuf::from("/tmp/qmk")),
            keycode_db,
            mock_worker,
        );

        let job = manager
            .start_generate(
                "test.md".to_string(),
                "crkbd".to_string(),
                "LAYOUT_split_3x6_3".to_string(),
            )
            .unwrap();

        // Cancel immediately
        let result = manager.cancel_job(&job.id);
        assert!(result.success);

        // Wait for worker to process
        thread::sleep(Duration::from_millis(100));

        let updated = manager.get_job(&job.id).unwrap();
        assert_eq!(updated.status, GenerateJobStatus::Cancelled);
    }

    #[test]
    fn test_list_jobs() {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_gen_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create layout files
        fs::write(temp_dir.join("a.md"), "---\nname: A\n---\n").unwrap();
        fs::write(temp_dir.join("b.md"), "---\nname: B\n---\n").unwrap();

        let manager = create_test_manager(&temp_dir);

        // Start two generations
        let _ = manager.start_generate(
            "a.md".to_string(),
            "crkbd".to_string(),
            "LAYOUT_split_3x6_3".to_string(),
        );
        thread::sleep(Duration::from_millis(10));
        let _ = manager.start_generate(
            "b.md".to_string(),
            "crkbd".to_string(),
            "LAYOUT_split_3x6_3".to_string(),
        );

        let jobs = manager.list_jobs();
        // First job may still be running, second will be pending or running
        assert!(!jobs.is_empty());
    }

    #[test]
    fn test_parse_log_line() {
        let (level, msg) = parse_log_line("[INFO] Generation started");
        assert_eq!(level, "INFO");
        assert_eq!(msg, "Generation started");

        let (level, msg) = parse_log_line("[ERROR] Something went wrong");
        assert_eq!(level, "ERROR");
        assert_eq!(msg, "Something went wrong");

        let (level, msg) = parse_log_line("Plain message without level");
        assert_eq!(level, "INFO");
        assert_eq!(msg, "Plain message without level");
    }

    #[test]
    fn test_mock_worker_failure() {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_gen_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a dummy layout file
        fs::write(temp_dir.join("test.md"), "---\nname: Test\n---\n").unwrap();

        let keycode_db = Arc::new(KeycodeDb::load().unwrap());
        let mock_worker = Arc::new(MockGenerateWorker {
            duration_ms: 50,
            should_succeed: false,
            error_message: Some("Generation error".to_string()),
        });
        let manager = GenerateJobManager::with_worker(
            temp_dir.join("logs"),
            temp_dir.join("output"),
            temp_dir.clone(),
            Some(PathBuf::from("/tmp/qmk")),
            keycode_db,
            mock_worker,
        );

        let job = manager
            .start_generate(
                "test.md".to_string(),
                "crkbd".to_string(),
                "LAYOUT_split_3x6_3".to_string(),
            )
            .unwrap();

        // Wait for generation to complete
        thread::sleep(Duration::from_millis(200));

        let updated = manager.get_job(&job.id).unwrap();
        assert_eq!(updated.status, GenerateJobStatus::Failed);
        assert!(updated.error.is_some());
        assert!(updated.error.unwrap().contains("Generation error"));
    }

    #[test]
    fn test_add_file_to_zip_validation() {
        // Test zip-slip prevention
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_zip_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();
        let zip_path = temp_dir.join("test.zip");

        let file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        // Valid filename should work
        let result = add_file_to_zip(&mut zip, "valid.txt", b"content", options);
        assert!(result.is_ok());

        // Path traversal should fail
        let result = add_file_to_zip(&mut zip, "../escape.txt", b"content", options);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid filename"));

        // Absolute path should fail
        let result = add_file_to_zip(&mut zip, "/etc/passwd", b"content", options);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid filename"));
    }

    #[test]
    fn test_worker_not_running_returns_error() {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_gen_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a dummy layout file
        let layout_path = temp_dir.join("test.md");
        fs::write(&layout_path, "---\nname: Test\n---\n").unwrap();

        let keycode_db = Arc::new(KeycodeDb::load().unwrap());
        let mock_worker = Arc::new(MockGenerateWorker::default());

        // Create manager but don't start the worker thread
        let manager = Arc::new(GenerateJobManager {
            jobs: RwLock::new(HashMap::new()),
            cancelled: RwLock::new(std::collections::HashSet::new()),
            running_count: Mutex::new(0),
            command_tx: Mutex::new(None), // No worker!
            logs_dir: temp_dir.join("logs"),
            output_dir: temp_dir.join("output"),
            workspace_root: temp_dir.clone(),
            qmk_path: Some(PathBuf::from("/tmp/qmk")),
            worker: mock_worker,
            keycode_db,
        });

        // Ensure directories exist
        let _ = fs::create_dir_all(&manager.logs_dir);
        let _ = fs::create_dir_all(&manager.output_dir);

        let result = manager.start_generate(
            "test.md".to_string(),
            "crkbd".to_string(),
            "LAYOUT_split_3x6_3".to_string(),
        );

        // Should return error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.contains("worker not running"),
            "Expected 'worker not running' in error, got: {}",
            error
        );

        // Running count should be unaffected (not incremented)
        let count = *manager.running_count.lock().unwrap();
        assert_eq!(count, 0, "Running count should remain 0 after failure");

        // Job should exist but be marked as failed
        let job_id = {
            let jobs = manager.jobs.read().unwrap();
            assert_eq!(jobs.len(), 1, "Job should be created even if enqueue fails");
            let job = jobs.values().next().unwrap();
            assert_eq!(
                job.status,
                GenerateJobStatus::Failed,
                "Job should be marked as Failed"
            );
            assert!(job.error.is_some(), "Job should have an error message");
            let id = job.id.clone();
            drop(jobs);
            id
        };

        // Log file should exist with error entry
        let log_path = manager.logs_dir.join(format!("{job_id}.log"));
        assert!(log_path.exists(), "Log file should be created");
        let log_content = fs::read_to_string(&log_path).unwrap();
        assert!(
            log_content.contains("[ERROR]"),
            "Log should contain error entry"
        );
        assert!(
            log_content.contains("worker not running"),
            "Log should explain the failure"
        );
    }

    #[test]
    fn test_send_failure_rolls_back_state() {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_gen_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a dummy layout file
        let layout_path = temp_dir.join("test.md");
        fs::write(&layout_path, "---\nname: Test\n---\n").unwrap();

        let keycode_db = Arc::new(KeycodeDb::load().unwrap());
        let mock_worker = Arc::new(MockGenerateWorker::default());

        // Create a channel and immediately drop the receiver to simulate send failure
        let (tx, rx) = mpsc::channel::<GenerateCommand>();
        drop(rx); // This will cause send to fail

        let manager = Arc::new(GenerateJobManager {
            jobs: RwLock::new(HashMap::new()),
            cancelled: RwLock::new(std::collections::HashSet::new()),
            running_count: Mutex::new(0),
            command_tx: Mutex::new(Some(tx)),
            logs_dir: temp_dir.join("logs"),
            output_dir: temp_dir.join("output"),
            workspace_root: temp_dir.clone(),
            qmk_path: Some(PathBuf::from("/tmp/qmk")),
            worker: mock_worker,
            keycode_db,
        });

        // Ensure directories exist
        let _ = fs::create_dir_all(&manager.logs_dir);
        let _ = fs::create_dir_all(&manager.output_dir);

        let result = manager.start_generate(
            "test.md".to_string(),
            "crkbd".to_string(),
            "LAYOUT_split_3x6_3".to_string(),
        );

        // Should return error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.contains("Failed to queue"),
            "Expected 'Failed to queue' in error, got: {}",
            error
        );

        // Running count should be rolled back to 0
        let count = *manager.running_count.lock().unwrap();
        assert_eq!(count, 0, "Running count should be rolled back to 0");

        // Job should exist and be marked as failed
        let job_id = {
            let jobs = manager.jobs.read().unwrap();
            assert_eq!(jobs.len(), 1, "Job should exist");
            let job = jobs.values().next().unwrap();
            assert_eq!(
                job.status,
                GenerateJobStatus::Failed,
                "Job should be marked as Failed"
            );
            assert!(job.error.is_some(), "Job should have error message");
            let id = job.id.clone();
            drop(jobs);
            id
        };

        // Log file should contain error
        let log_path = manager.logs_dir.join(format!("{job_id}.log"));
        assert!(log_path.exists(), "Log file should be created");
        let log_content = fs::read_to_string(&log_path).unwrap();
        assert!(
            log_content.contains("[ERROR]"),
            "Log should contain error entry"
        );
    }

    #[test]
    fn test_health_accessor() {
        let temp_dir = std::env::temp_dir().join(format!("lazyqmk_gen_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        let manager = create_test_manager(&temp_dir);

        let health = manager.health();
        assert!(
            health.worker_running,
            "Worker should be running after initialization"
        );
        assert_eq!(
            health.running_count, 0,
            "No jobs should be running initially"
        );
        assert_eq!(
            health.max_concurrent_jobs, MAX_CONCURRENT_JOBS,
            "Max concurrent jobs should match constant"
        );
    }
}
