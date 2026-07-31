//! Web-side events broadcast over the Server-Sent Events endpoint.
//!
//! The web backend watches the workspace directory for layout file
//! changes and publishes them as `LayoutEvent` messages. The Svelte
//! frontend subscribes via `EventSource` (see
//! `web/src/lib/api/events.ts`) and triggers refetches when relevant
//! events arrive.

use serde::Serialize;

/// SSE event payload for layout file changes.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LayoutEvent {
    /// A layout file was created or modified on disk.
    Changed {
        /// Bare file name (e.g. `"my_layout.json"`) — never an absolute
        /// path, so we don't leak filesystem layout to clients.
        filename: String,
        /// File mtime as Unix seconds (best-effort).
        mtime: u64,
    },
    /// A layout file was removed.
    Removed {
        /// Bare file name of the removed file.
        filename: String,
    },
    /// A revision was created/deleted/renamed/restored for a layout.
    RevisionChanged {
        /// Bare layout name (e.g. `"my_layout"`).
        layout_name: String,
        /// Revision id affected.
        revision: u32,
        /// What happened.
        action: String,
    },
}

impl LayoutEvent {
    /// Returns the file name this event refers to, if any.
    #[must_use]
    pub fn filename(&self) -> &str {
        match self {
            Self::Changed { filename, .. } | Self::Removed { filename } => filename,
            Self::RevisionChanged { layout_name, .. } => layout_name,
        }
    }
}
