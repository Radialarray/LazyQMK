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
    /// Source position for cut operation (layer_index, position)
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
    pub fn new() -> Self {
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
            format!("Copied: {} (with color/category)", keycode)
        } else {
            format!("Copied: {}", keycode)
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

        format!("Cut: {} - press p to paste, Esc to cancel", keycode)
    }

    /// Copy multiple keys to the clipboard.
    ///
    /// Keys are stored with positions relative to the anchor (first key).
    pub fn copy_multi(&mut self, keys: Vec<(Position, ClipboardContent)>, anchor: Position) -> String {
        // Clear single-key and cut state
        self.content = None;
        self.cut_source = None;
        self.multi_cut_sources.clear();

        let count = keys.len();
        self.multi_content = Some(MultiKeyContent { keys, anchor });

        format!("Copied {} keys", count)
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

        format!("Cut {} keys - press p to paste, Esc to cancel", count)
    }

    /// Check if there is content to paste (single or multi).
    #[must_use]
    pub fn has_content(&self) -> bool {
        self.content.is_some() || self.multi_content.is_some()
    }

    /// Check if this is a single-key clipboard.
    #[must_use]
    pub fn is_single(&self) -> bool {
        self.content.is_some()
    }

    /// Check if this is a multi-key clipboard.
    #[must_use]
    pub fn is_multi(&self) -> bool {
        self.multi_content.is_some()
    }

    /// Check if this is a cut operation (vs copy).
    #[must_use]
    pub fn is_cut(&self) -> bool {
        self.cut_source.is_some() || !self.multi_cut_sources.is_empty()
    }

    /// Get the clipboard content for pasting (single key).
    #[must_use]
    pub fn get_content(&self) -> Option<&ClipboardContent> {
        self.content.as_ref()
    }

    /// Get the multi-key clipboard content for pasting.
    #[must_use]
    pub fn get_multi_content(&self) -> Option<&MultiKeyContent> {
        self.multi_content.as_ref()
    }

    /// Get the cut source position (if this was a single-key cut operation).
    #[must_use]
    pub fn get_cut_source(&self) -> Option<(usize, Position)> {
        self.cut_source
    }

    /// Get the multi-key cut sources.
    #[must_use]
    pub fn get_multi_cut_sources(&self) -> &[(usize, Position)] {
        &self.multi_cut_sources
    }

    /// Check if a given position is a cut source (for visual feedback).
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
            let cut_indicator = if self.is_cut() { " (cut)" } else { "" };
            Some(format!("{}{}", content.keycode, cut_indicator))
        } else if let Some(multi) = &self.multi_content {
            let cut_indicator = if self.is_cut() { " (cut)" } else { "" };
            Some(format!("{} keys{}", multi.keys.len(), cut_indicator))
        } else {
            None
        }
    }

    /// Save undo state before making changes.
    pub fn save_undo(&mut self, layer_index: usize, keys: Vec<(Position, ClipboardContent)>, description: String) {
        self.undo_state = Some(UndoState {
            layer_index,
            original_keys: keys,
            description,
        });
    }

    /// Get the undo state (if any).
    #[must_use]
    pub fn get_undo(&self) -> Option<&UndoState> {
        self.undo_state.as_ref()
    }

    /// Take the undo state (consuming it).
    pub fn take_undo(&mut self) -> Option<UndoState> {
        self.undo_state.take()
    }

    /// Check if undo is available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.undo_state.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_new() {
        let clipboard = KeyClipboard::new();
        assert!(!clipboard.has_content());
        assert!(!clipboard.is_cut());
        assert!(clipboard.get_preview().is_none());
    }

    #[test]
    fn test_clipboard_copy() {
        let mut clipboard = KeyClipboard::new();
        let msg = clipboard.copy("KC_A", None, None);

        assert!(clipboard.has_content());
        assert!(clipboard.is_single());
        assert!(!clipboard.is_multi());
        assert!(!clipboard.is_cut());
        assert!(msg.contains("Copied: KC_A"));

        let content = clipboard.get_content().unwrap();
        assert_eq!(content.keycode, "KC_A");
        assert!(content.color_override.is_none());
        assert!(content.category_id.is_none());
    }

    #[test]
    fn test_clipboard_copy_with_extras() {
        let mut clipboard = KeyClipboard::new();
        let color = RgbColor::new(255, 0, 0);
        let msg = clipboard.copy("KC_B", Some(color), Some("navigation"));

        assert!(msg.contains("with color/category"));

        let content = clipboard.get_content().unwrap();
        assert_eq!(content.keycode, "KC_B");
        assert_eq!(content.color_override, Some(color));
        assert_eq!(content.category_id, Some("navigation".to_string()));
    }

    #[test]
    fn test_clipboard_cut() {
        let mut clipboard = KeyClipboard::new();
        let pos = Position::new(1, 2);
        let msg = clipboard.cut("KC_C", None, None, 0, pos);

        assert!(clipboard.has_content());
        assert!(clipboard.is_cut());
        assert!(msg.contains("Cut: KC_C"));
        assert!(clipboard.is_cut_source(0, pos));
        assert!(!clipboard.is_cut_source(1, pos)); // Different layer
        assert!(!clipboard.is_cut_source(0, Position::new(0, 0))); // Different position
    }

    #[test]
    fn test_clipboard_clear_cut_source() {
        let mut clipboard = KeyClipboard::new();
        let pos = Position::new(1, 2);
        clipboard.cut("KC_D", None, None, 0, pos);

        assert!(clipboard.is_cut());
        clipboard.clear_cut_source();

        assert!(!clipboard.is_cut());
        assert!(clipboard.has_content()); // Content still there
    }

    #[test]
    fn test_clipboard_cancel_cut() {
        let mut clipboard = KeyClipboard::new();
        let pos = Position::new(1, 2);
        clipboard.cut("KC_E", None, None, 0, pos);

        clipboard.cancel_cut();

        assert!(!clipboard.is_cut());
        assert!(clipboard.has_content()); // Content preserved
    }

    #[test]
    fn test_clipboard_clear() {
        let mut clipboard = KeyClipboard::new();
        clipboard.copy("KC_F", None, None);

        clipboard.clear();

        assert!(!clipboard.has_content());
        assert!(!clipboard.is_cut());
    }

    #[test]
    fn test_copy_clears_cut_state() {
        let mut clipboard = KeyClipboard::new();
        let pos = Position::new(1, 2);
        clipboard.cut("KC_G", None, None, 0, pos);

        assert!(clipboard.is_cut());

        clipboard.copy("KC_H", None, None);

        assert!(!clipboard.is_cut()); // Cut state cleared
        assert!(clipboard.has_content());
        assert_eq!(clipboard.get_content().unwrap().keycode, "KC_H");
    }

    #[test]
    fn test_clipboard_preview() {
        let mut clipboard = KeyClipboard::new();

        // No content
        assert!(clipboard.get_preview().is_none());

        // After copy
        clipboard.copy("LT(1,SPC)", None, None);
        assert_eq!(clipboard.get_preview(), Some("LT(1,SPC)".to_string()));

        // After cut
        clipboard.cut("KC_A", None, None, 0, Position::new(0, 0));
        assert_eq!(clipboard.get_preview(), Some("KC_A (cut)".to_string()));
    }

    #[test]
    fn test_clipboard_multi_copy() {
        let mut clipboard = KeyClipboard::new();
        let keys = vec![
            (Position::new(0, 0), ClipboardContent {
                keycode: "KC_A".to_string(),
                color_override: None,
                category_id: None,
            }),
            (Position::new(0, 1), ClipboardContent {
                keycode: "KC_B".to_string(),
                color_override: None,
                category_id: None,
            }),
        ];

        let msg = clipboard.copy_multi(keys, Position::new(0, 0));
        assert!(msg.contains("2 keys"));
        assert!(clipboard.is_multi());
        assert!(!clipboard.is_single());
        assert_eq!(clipboard.get_preview(), Some("2 keys".to_string()));
    }

    #[test]
    fn test_clipboard_multi_cut() {
        let mut clipboard = KeyClipboard::new();
        let keys = vec![
            (Position::new(0, 0), ClipboardContent {
                keycode: "KC_A".to_string(),
                color_override: None,
                category_id: None,
            }),
            (Position::new(0, 1), ClipboardContent {
                keycode: "KC_B".to_string(),
                color_override: None,
                category_id: None,
            }),
        ];
        let positions = vec![Position::new(0, 0), Position::new(0, 1)];

        let msg = clipboard.cut_multi(keys, Position::new(0, 0), 0, positions);
        assert!(msg.contains("2 keys"));
        assert!(clipboard.is_cut());
        assert!(clipboard.is_cut_source(0, Position::new(0, 0)));
        assert!(clipboard.is_cut_source(0, Position::new(0, 1)));
        assert!(!clipboard.is_cut_source(0, Position::new(1, 0)));
    }

    #[test]
    fn test_undo_state() {
        let mut clipboard = KeyClipboard::new();

        assert!(!clipboard.can_undo());

        let keys = vec![(Position::new(0, 0), ClipboardContent {
            keycode: "KC_A".to_string(),
            color_override: None,
            category_id: None,
        })];
        clipboard.save_undo(0, keys, "Pasted KC_B".to_string());

        assert!(clipboard.can_undo());

        let undo = clipboard.take_undo().unwrap();
        assert_eq!(undo.layer_index, 0);
        assert_eq!(undo.original_keys.len(), 1);
        assert_eq!(undo.description, "Pasted KC_B");

        assert!(!clipboard.can_undo());
    }
}
