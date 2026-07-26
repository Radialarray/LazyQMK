//! Shared application state for the web API.
//!
//! Extracted from src/web/mod.rs as part of LazyQMK-2rf6.2.

use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

use crate::config::Config;
use crate::keycode_db::KeycodeDb;
use crate::services::file_watcher::SelfWriteEpoch;
use crate::web::build_jobs::BuildJobManager;
use crate::web::events::LayoutEvent;
use crate::web::generate_jobs::GenerateJobManager;

#[cfg(test)]
use crate::web::build_jobs::MockFirmwareBuilder;

#[cfg(test)]
use crate::web::generate_jobs::MockGenerateWorker;

/// Buffer size for the layout-event broadcast channel.
///
/// Each subscriber gets its own copy; if a subscriber lags more than
/// this many events the broadcast receiver starts skipping them
/// (`Lagged`). 64 is more than enough for typical agent-edit rates.
/// more than enough for typical agent-edit rates.
pub const LAYOUT_EVENT_BUFFER: usize = 64;

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
    /// Broadcast sender for hot-reload layout events. Subscribed by the
    /// `/api/events` SSE endpoint.
    pub(crate) layout_events: broadcast::Sender<LayoutEvent>,
    /// Shared epoch for self-write suppression. Bumped before every
    /// `LayoutService::save_with_epoch` so the workspace watcher does
    /// not echo our own saves back to clients.
    pub(crate) self_write_epoch: SelfWriteEpoch,
    /// Owns the workspace file watcher's debouncer + worker thread.
    /// Dropped automatically when the last `AppState` clone goes
    /// away (or kept alive for the server's lifetime).
    pub(crate) workspace_watcher: Option<Arc<crate::web::watcher::WorkspaceWatcher>>,
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

        let (layout_events, _) = broadcast::channel(LAYOUT_EVENT_BUFFER);
        let self_write_epoch = crate::services::file_watcher::new_epoch();

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            keycode_db,
            workspace_root,
            build_manager,
            generate_manager,
            layout_events,
            self_write_epoch,
            workspace_watcher: None,
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

        let (layout_events, _) = broadcast::channel(LAYOUT_EVENT_BUFFER);
        let self_write_epoch = crate::services::file_watcher::new_epoch();

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            keycode_db,
            workspace_root,
            build_manager,
            generate_manager,
            layout_events,
            self_write_epoch,
            workspace_watcher: None,
        })
    }

    /// Returns the workspace root directory.
    #[must_use]
    pub fn workspace_root(&self) -> &PathBuf {
        &self.workspace_root
    }

    /// Returns a clone of the self-write epoch so other components
    /// (e.g. route handlers) can pass it to `LayoutService::save_with_epoch`.
    #[must_use]
    pub fn self_write_epoch(&self) -> SelfWriteEpoch {
        std::sync::Arc::clone(&self.self_write_epoch)
    }

    /// Returns a clone of the layout-event broadcast sender so route
    /// handlers (or tests) can inject events.
    #[must_use]
    pub fn layout_event_sender(&self) -> broadcast::Sender<LayoutEvent> {
        self.layout_events.clone()
    }

    /// Subscribes to layout-change events. Each call returns a
    /// fresh receiver; missing events (when the receiver lags
    /// beyond the channel buffer) are skipped.
    #[must_use]
    pub fn subscribe_layout_events(&self) -> broadcast::Receiver<LayoutEvent> {
        self.layout_events.subscribe()
    }

    /// Starts the workspace-level file watcher that feeds layout
    /// events into the broadcast channel. Idempotent — calling twice
    /// is harmless; the second call replaces the previous watcher.
    ///
    /// The watcher is owned by the AppState and lives for the
    /// lifetime of the server. When the AppState is dropped the
    /// watcher and its background thread are stopped.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying watcher cannot be started.
    /// Callers typically log and continue — the rest of the server
    /// works fine without hot-reload.
    pub fn start_workspace_watcher(&mut self) -> anyhow::Result<()> {
        let watcher = crate::web::watcher::spawn_workspace_watcher(
            &self.workspace_root,
            std::sync::Arc::clone(&self.self_write_epoch),
            self.layout_events.clone(),
        )?;
        self.workspace_watcher = Some(Arc::new(watcher));
        Ok(())
    }
}
