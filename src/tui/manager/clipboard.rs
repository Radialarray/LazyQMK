//! Clipboard functionality for key copy/cut/paste operations.
//!
//! Provides clipboard state management for copying, cutting, and pasting
//! keys within and across layers. Supports both single-key and multi-key
//! selection operations, plus undo functionality.

use crate::models::{Position, RgbColor};

/// Content stored in the clipboard (key data without position).
#[derive(Debug, Clone)]
pub struct ClipboardContent {
    /// The QMK keycode string
    pub keycode: String,
    /// Individual key color override (if any)
    pub color_override: Option<RgbColor>,
    /// Category assignment (if any)
    pub category_id: Option<String>,
}

/// Content for multi-key clipboard operations (with relative positions).
#[derive(Debug, Clone)]
pub struct MultiKeyContent {
    /// Keys with their relative positions from the anchor point
    pub keys: Vec<(Position, ClipboardContent)>,
    /// The anchor position (first selected key, used as reference)
    pub anchor: Position,
}

/// State saved for undo operation
#[derive(Debug, Clone)]
pub struct UndoState {
    /// Layer index where the change was made
    pub layer_index: usize,
    /// Keys that were modified (position + original content)
    pub original_keys: Vec<(Position, ClipboardContent)>,
    /// Description of the operation (for status message)
    pub description: String,
}

/// Clipboard state for key operations.
#[derive(Debug, Clone, Default)]
pub struct KeyClipboard {
    /// Stored key data for paste operation (single key)
    content: Option<ClipboardContent>,
    /// Stored multi-key data for paste operation
    multi_content: Option<MultiKeyContent>,
    /// Source position for cut operation (`layer_index`, position)
    /// Used for visual feedback (dimming the cut source)
    cut_source: Option<(usize, Position)>,
    /// Multiple cut sources for multi-key cut
    multi_cut_sources: Vec<(usize, Position)>,
    /// Undo stack (most recent operation)
    undo_state: Option<UndoState>,
}

impl KeyClipboard {
    /// Create a new empty clipboard.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            content: None,
            multi_content: None,
            cut_source: None,
            multi_cut_sources: Vec::new(),
            undo_state: None,
        }
    }

    /// Copy a key's data to the clipboard.
    ///
    /// Returns a description of what was copied for status message.
    pub fn copy(
        &mut self,
        keycode: &str,
        color_override: Option<RgbColor>,
        category_id: Option<&str>,
    ) -> String {
        // Clear any previous cut state and multi-key state
        self.cut_source = None;
        self.multi_cut_sources.clear();
        self.multi_content = None;

        let has_extras = color_override.is_some() || category_id.is_some();

        self.content = Some(ClipboardContent {
            keycode: keycode.to_string(),
            color_override,
            category_id: category_id.map(String::from),
        });

        if has_extras {
            format!("Copied: {keycode} (with color/category)")
        } else {
            format!("Copied: {keycode}")
        }
    }

    /// Cut a key's data to the clipboard.
    ///
    /// Returns a description of what was cut for status message.
    /// The source key is NOT cleared yet - that happens on paste.
    pub fn cut(
        &mut self,
        keycode: &str,
        color_override: Option<RgbColor>,
        category_id: Option<&str>,
        layer_index: usize,
        position: Position,
    ) -> String {
        // Clear multi-key state
        self.multi_content = None;
        self.multi_cut_sources.clear();

        self.content = Some(ClipboardContent {
            keycode: keycode.to_string(),
            color_override,
            category_id: category_id.map(String::from),
        });

        self.cut_source = Some((layer_index, position));

        format!("Cut queued: {keycode} - source clears after paste, Esc cancels")
    }

    /// Copy multiple keys to the clipboard.
    ///
    /// Keys are stored with positions relative to the anchor (first key).
    pub fn copy_multi(
        &mut self,
        keys: Vec<(Position, ClipboardContent)>,
        anchor: Position,
    ) -> String {
        // Clear single-key and cut state
        self.content = None;
        self.cut_source = None;
        self.multi_cut_sources.clear();

        let count = keys.len();
        self.multi_content = Some(MultiKeyContent { keys, anchor });

        format!("Copied {count} keys")
    }

    /// Cut multiple keys to the clipboard.
    ///
    /// Keys are stored with positions relative to the anchor.
    pub fn cut_multi(
        &mut self,
        keys: Vec<(Position, ClipboardContent)>,
        anchor: Position,
        layer_index: usize,
        positions: Vec<Position>,
    ) -> String {
        // Clear single-key state
        self.content = None;
        self.cut_source = None;

        let count = keys.len();
        self.multi_content = Some(MultiKeyContent { keys, anchor });
        self.multi_cut_sources = positions.into_iter().map(|p| (layer_index, p)).collect();

        format!("Cut queued: {count} keys - sources clear after paste, Esc cancels")
    }

    /// Check if there is content to paste (single or multi).
    #[must_use]
    pub const fn has_content(&self) -> bool {
        self.content.is_some() || self.multi_content.is_some()
    }

    /// Check if this is a single-key clipboard.
    #[must_use]
    pub const fn is_single(&self) -> bool {
        self.content.is_some()
    }

    /// Check if this is a multi-key clipboard.
    #[must_use]
    pub const fn is_multi(&self) -> bool {
        self.multi_content.is_some()
    }

    /// Check if this is a cut operation (vs copy).
    #[must_use]
    pub const fn is_cut(&self) -> bool {
        self.cut_source.is_some() || !self.multi_cut_sources.is_empty()
    }

    /// Get the clipboard content for pasting (single key).
    #[must_use]
    pub const fn get_content(&self) -> Option<&ClipboardContent> {
        self.content.as_ref()
    }

    /// Get the multi-key clipboard content for pasting.
    #[must_use]
    pub const fn get_multi_content(&self) -> Option<&MultiKeyContent> {
        self.multi_content.as_ref()
    }

    /// Get the cut source position (if this was a single-key cut operation).
    #[must_use]
    pub const fn get_cut_source(&self) -> Option<(usize, Position)> {
        self.cut_source
    }

    /// Get the multi-key cut sources.
    #[must_use]
    pub fn get_multi_cut_sources(&self) -> &[(usize, Position)] {
        &self.multi_cut_sources
    }

    /// Returns `true` if `(layer_index, position)` is a pending cut source.
    ///
    /// `position` is a **visual**-grid [`Position`] (matching `KeyDefinition.position`).
    #[must_use]
    pub fn is_cut_source(&self, layer_index: usize, position: Position) -> bool {
        if self.cut_source == Some((layer_index, position)) {
            return true;
        }
        self.multi_cut_sources.contains(&(layer_index, position))
    }

    /// Clear the cut source after paste.
    /// The clipboard content is kept so the same data can be pasted again.
    pub fn clear_cut_source(&mut self) {
        self.cut_source = None;
        self.multi_cut_sources.clear();
    }

    /// Cancel cut operation (Esc pressed).
    /// Clears cut source but keeps clipboard content for potential paste.
    pub fn cancel_cut(&mut self) {
        self.cut_source = None;
        self.multi_cut_sources.clear();
    }

    /// Clear the entire clipboard.
    pub fn clear(&mut self) {
        self.content = None;
        self.multi_content = None;
        self.cut_source = None;
        self.multi_cut_sources.clear();
    }

    /// Get a preview string for status bar display.
    #[must_use]
    pub fn get_preview(&self) -> Option<String> {
        if let Some(content) = &self.content {
            let cut_indicator = if self.is_cut() { " (queued cut)" } else { "" };
            Some(format!("{}{}", content.keycode, cut_indicator))
        } else if let Some(multi) = &self.multi_content {
            let cut_indicator = if self.is_cut() { " (queued cut)" } else { "" };
            Some(format!("{} keys{}", multi.keys.len(), cut_indicator))
        } else {
            None
        }
    }

    /// Save undo state before making changes.
    pub fn save_undo(
        &mut self,
        layer_index: usize,
        keys: Vec<(Position, ClipboardContent)>,
        description: String,
    ) {
        self.undo_state = Some(UndoState {
            layer_index,
            original_keys: keys,
            description,
        });
    }

    /// Get the undo state (if any).
    #[must_use]
    pub const fn get_undo(&self) -> Option<&UndoState> {
        self.undo_state.as_ref()
    }

    /// Take the undo state (consuming it).
    pub const fn take_undo(&mut self) -> Option<UndoState> {
        self.undo_state.take()
    }

    /// Check if undo is available.
    #[must_use]
    pub const fn can_undo(&self) -> bool {
        self.undo_state.is_some()
    }
}

#[cfg(test)]
mod tests;
