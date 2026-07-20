//! Tests for build_jobs.
//!
//! Auto-extracted from build_jobs.rs.

use super::*;

    use super::*;
    use std::time::Duration;

    fn test_keycode_db() -> Arc<KeycodeDb> {
        Arc::new(KeycodeDb::load().expect("Failed to load keycode database"))
    }

    /// Dummy layout path used by tests with mock builders (never actually read).
    fn dummy_layout_path() -> PathBuf {
        PathBuf::from("nonexistent_test_layout.md")
    }

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
            test_keycode_db(),
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
            dummy_layout_path(),
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
            test_keycode_db(),
        );

        let result = manager.start_build(
            "test.md".to_string(),
            "crkbd".to_string(),
            "default".to_string(),
            dummy_layout_path(),
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
            test_keycode_db(),
        );

        let job = manager
            .start_build(
                "test.md".to_string(),
                "crkbd".to_string(),
                "default".to_string(),
                dummy_layout_path(),
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
            dummy_layout_path(),
        );
        thread::sleep(Duration::from_millis(10));
        let _ = manager.start_build(
            "b.md".to_string(),
            "crkbd".to_string(),
            "test".to_string(),
            dummy_layout_path(),
        );

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
            test_keycode_db(),
        );

        let job = manager
            .start_build(
                "test.md".to_string(),
                "crkbd".to_string(),
                "default".to_string(),
                dummy_layout_path(),
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
                dummy_layout_path(),
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
                dummy_layout_path(),
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
            test_keycode_db(),
        );

        let job = manager
            .start_build(
                "test.md".to_string(),
                "crkbd".to_string(),
                "default".to_string(),
                dummy_layout_path(),
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
                dummy_layout_path(),
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
            test_keycode_db(),
        );

        // Start a build (will be running)
        let running_job = manager
            .start_build(
                "running.md".to_string(),
                "crkbd".to_string(),
                "default".to_string(),
                dummy_layout_path(),
            )
            .unwrap();

        thread::sleep(Duration::from_millis(100));

        // Trigger cleanup
        manager.cleanup_old_artifacts();

        // Verify running job is not removed from jobs map (directory might not exist yet)
        assert!(manager.get_job(&running_job.id).is_some());
    }
