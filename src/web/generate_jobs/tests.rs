//! Tests for generate_jobs.
//!
//! Auto-extracted from generate_jobs.rs.

use super::*;

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
        qmk_path: RwLock::new(Some(PathBuf::from("/tmp/qmk"))),
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
        qmk_path: RwLock::new(Some(PathBuf::from("/tmp/qmk"))),
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
