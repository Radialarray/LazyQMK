//! Shared application state for the web API.
//!
//! Extracted from src/web/mod.rs as part of LazyQMK-2rf6.2.

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::config::Config;
use crate::keycode_db::KeycodeDb;
use crate::web::build_jobs::BuildJobManager;
use crate::web::generate_jobs::GenerateJobManager;

#[cfg(test)]
use crate::web::build_jobs::MockFirmwareBuilder;

#[cfg(test)]
use crate::web::generate_jobs::MockGenerateWorker;

/// Shared application state for the web API.
#[derive(Clone)]
pub struct AppState {
    /// Application configuration
    pub(crate) config: Arc<RwLock<Config>>,
    /// Keycode database (immutable after load)
    pub(crate) keycode_db: Arc<KeycodeDb>,
    /// Working directory for layout files (defaults to current dir)
    pub(crate) workspace_root: PathBuf,
    /// Build job manager for background firmware builds
    pub(crate) build_manager: Arc<BuildJobManager>,
    /// Generate job manager for firmware generation and zip packaging
    pub(crate) generate_manager: Arc<GenerateJobManager>,
}

impl AppState {
    /// Creates a new application state.
    pub fn new(config: Config, workspace_root: PathBuf) -> anyhow::Result<Self> {
        let keycode_db = Arc::new(KeycodeDb::load()?);

        // Set up build job manager
        let logs_dir = workspace_root.join(".lazyqmk").join("build_logs");
        let output_dir = workspace_root.join(".lazyqmk").join("build_output");
        let qmk_path = config.paths.qmk_firmware.clone();
        let build_manager = BuildJobManager::new(
            logs_dir,
            output_dir,
            qmk_path.clone(),
            Arc::clone(&keycode_db),
        );

        // Set up generate job manager
        let gen_logs_dir = workspace_root.join(".lazyqmk").join("generate_logs");
        let gen_output_dir = workspace_root.join(".lazyqmk").join("generate_output");
        let generate_manager = GenerateJobManager::new(
            gen_logs_dir,
            gen_output_dir,
            workspace_root.clone(),
            qmk_path,
            Arc::clone(&keycode_db),
        );

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            keycode_db,
            workspace_root,
            build_manager,
            generate_manager,
        })
    }

    /// Creates a new application state with a mock builder (for testing).
    #[cfg(test)]
    pub fn with_mock_builder(config: Config, workspace_root: PathBuf) -> anyhow::Result<Self> {
        let keycode_db = Arc::new(KeycodeDb::load()?);

        // Set up build job manager with mock builder
        let logs_dir = workspace_root.join(".lazyqmk").join("build_logs");
        let output_dir = workspace_root.join(".lazyqmk").join("build_output");
        let qmk_path = config.paths.qmk_firmware.clone();
        let mock_builder = Arc::new(MockFirmwareBuilder::default());
        let build_manager = BuildJobManager::with_builder(
            logs_dir,
            output_dir,
            qmk_path.clone(),
            mock_builder,
            Arc::clone(&keycode_db),
        );

        // Set up generate job manager with mock worker
        let gen_logs_dir = workspace_root.join(".lazyqmk").join("generate_logs");
        let gen_output_dir = workspace_root.join(".lazyqmk").join("generate_output");
        let mock_worker = Arc::new(MockGenerateWorker::default());
        let generate_manager = GenerateJobManager::with_worker(
            gen_logs_dir,
            gen_output_dir,
            workspace_root.clone(),
            qmk_path,
            Arc::clone(&keycode_db),
            mock_worker,
        );

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            keycode_db,
            workspace_root,
            build_manager,
            generate_manager,
        })
    }

    /// Returns the workspace root directory.
    #[must_use]
    pub fn workspace_root(&self) -> &PathBuf {
        &self.workspace_root
    }
}