//! Generate job manager — coordinates background firmware generation.
//!
//! Contains [`GenerateJobManager`] and the full `impl` block with all public
//! and private methods.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;

use crate::keycode_db::KeycodeDb;
use tracing::{info, warn};

use super::parse_log_line;
use super::workers::RealGenerateWorker;
use super::{
    CancelGenerateJobResponse, GenerateCommand, GenerateJob, GenerateJobHealth,
    GenerateJobLogsResponse, GenerateJobStatus, GenerateWorker, LogEntry, MAX_CONCURRENT_JOBS,
};

/// Generate job manager that coordinates background generation.
pub struct GenerateJobManager {
    /// All jobs indexed by ID.
    pub(crate) jobs: RwLock<HashMap<String, GenerateJob>>,
    /// Set of cancelled job IDs.
    pub(crate) cancelled: RwLock<std::collections::HashSet<String>>,
    /// Number of currently running jobs.
    pub(crate) running_count: Mutex<usize>,
    /// Channel sender for generate commands.
    pub(crate) command_tx: Mutex<Option<mpsc::Sender<GenerateCommand>>>,
    /// Directory for storing job logs.
    pub(crate) logs_dir: PathBuf,
    /// Directory for storing generated output.
    pub(crate) output_dir: PathBuf,
    /// Workspace root directory.
    pub(crate) workspace_root: PathBuf,
    /// QMK firmware path from config.
    pub(crate) qmk_path: RwLock<Option<PathBuf>>,
    /// Generate worker (real or mock).
    pub(crate) worker: Arc<dyn GenerateWorker>,
    /// Keycode database.
    pub(crate) keycode_db: Arc<KeycodeDb>,
}

impl GenerateJobManager {
    /// Locks the jobs map for writing. Recovers from a poisoned mutex
    /// (which only happens when a worker thread panicked mid-generation)
    /// so other generations can continue.
    fn jobs_write(&self) -> std::sync::RwLockWriteGuard<'_, HashMap<String, GenerateJob>> {
        self.jobs
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Locks the jobs map for reading. Recovers from a poisoned mutex.
    fn jobs_read(&self) -> std::sync::RwLockReadGuard<'_, HashMap<String, GenerateJob>> {
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
    fn command_tx_lock(&self) -> std::sync::MutexGuard<'_, Option<mpsc::Sender<GenerateCommand>>> {
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
            qmk_path: RwLock::new(qmk_path),
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
        *self.command_tx_lock() = Some(tx);

        let manager = Arc::clone(self);

        info!("Generate worker thread starting");

        thread::spawn(move || {
            info!("Generate worker thread started, waiting for jobs");
            for cmd in rx {
                info!(job_id = %cmd.job_id, "Worker received job for processing");
                manager.process_generate(cmd);
            }
            info!("Generate worker thread stopped (channel closed)");
        });
    }

    /// Processes a generate command.
    fn process_generate(self: &Arc<Self>, cmd: GenerateCommand) {
        info!(
            job_id = %cmd.job_id,
            "Processing job: transition Pending → Running"
        );

        // Check if cancelled before starting
        if self.is_cancelled(&cmd.job_id) {
            info!(job_id = %cmd.job_id, "Job was cancelled before processing");
            self.update_job_status(&cmd.job_id, GenerateJobStatus::Cancelled, None, None);
            return;
        }

        // Update job to running
        {
            let mut jobs = self.jobs_write();
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
            let mut count = self.running_count_lock();
            *count = count.saturating_sub(1);
        }

        // Check if cancelled after generation
        if self.is_cancelled(&cmd.job_id) {
            info!(
                job_id = %cmd.job_id,
                "Job completed but was cancelled, marking as Cancelled"
            );
            self.update_job_status(&cmd.job_id, GenerateJobStatus::Cancelled, None, None);
            return;
        }

        // Update job with result
        match result {
            Ok(zip_path) => {
                info!(
                    job_id = %cmd.job_id,
                    "Job completed successfully: transition Running → Completed"
                );
                self.update_job_status(
                    &cmd.job_id,
                    GenerateJobStatus::Completed,
                    None,
                    Some(zip_path.display().to_string()),
                );
            }
            Err(error) => {
                warn!(
                    job_id = %cmd.job_id,
                    error = %error,
                    "Job failed: transition Running → Failed"
                );
                self.update_job_status(&cmd.job_id, GenerateJobStatus::Failed, Some(error), None);
            }
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
        status: GenerateJobStatus,
        error: Option<String>,
        zip_path: Option<String>,
    ) {
        let mut jobs = self.jobs_write();
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
            .read()
            .unwrap()
            .clone()
            .ok_or_else(|| "QMK firmware path not configured".to_string())?;

        // Check concurrency limit
        {
            let count = self.running_count_lock();
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
            let mut jobs = self.jobs_write();
            jobs.insert(job_id.clone(), job.clone());
        }

        // Create log file path
        let log_path = self.logs_dir.join(format!("{job_id}.log"));

        // Create job-specific output directory
        let job_output_dir = self.output_dir.join(&job_id);

        // Increment running count
        {
            let mut count = self.running_count_lock();
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
            let tx = self.command_tx_lock();
            match tx.as_ref() {
                None => {
                    warn!(
                        job_id = %job_id,
                        "Generate worker not running, cannot enqueue"
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
                let mut count = self.running_count_lock();
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

            warn!(job_id = %job_id, error = %error_msg, "Job failed to enqueue");
            return Err(error_msg);
        }

        info!(
            job_id = %job_id,
            layout = %job.layout_filename,
            "Job queued successfully"
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
        self.jobs_read().get(job_id).cloned()
    }

    /// Gets the zip file path for a completed job.
    pub fn get_zip_path(&self, job_id: &str) -> Option<PathBuf> {
        let jobs = self.jobs_read();
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
            let jobs = self.jobs_read();
            jobs.get(job_id).cloned()
        };

        match job {
            Some(job) => {
                if job.status == GenerateJobStatus::Running
                    || job.status == GenerateJobStatus::Pending
                {
                    // Mark as cancelled
                    self.cancelled_write().insert(job_id.to_string());

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
        let worker_running = self.command_tx_lock().is_some();
        let running_count = *self.running_count_lock();

        GenerateJobHealth {
            worker_running,
            running_count,
            max_concurrent_jobs: MAX_CONCURRENT_JOBS,
        }
    }

    /// Lists all jobs.
    pub fn list_jobs(&self) -> Vec<GenerateJob> {
        let mut list: Vec<_> = self.jobs_read().values().cloned().collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list
    }

    /// Updates the QMK firmware path.
    pub fn set_qmk_path(&self, path: Option<PathBuf>) {
        *self.qmk_path_write() = path;
    }
}
