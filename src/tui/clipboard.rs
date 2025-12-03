//! Clipboard functionality for key copy/cut/paste operations.
//!
//! Provides clipboard state management for copying, cutting, and pasting
//! keys within and across layers.

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

/// Clipboard state for key operations.
#[derive(Debug, Clone, Default)]
pub struct KeyClipboard {
    /// Stored key data for paste operation
    content: Option<ClipboardContent>,
    /// Source position for cut operation (layer_index, position)
    /// Used for visual feedback (dimming the cut source)
    cut_source: Option<(usize, Position)>,
}

impl KeyClipboard {
    /// Create a new empty clipboard.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            content: None,
            cut_source: None,
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
        // Clear any previous cut state
        self.cut_source = None;

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
        self.content = Some(ClipboardContent {
            keycode: keycode.to_string(),
            color_override,
            category_id: category_id.map(String::from),
        });

        self.cut_source = Some((layer_index, position));

        format!("Cut: {} - press p to paste, Esc to cancel", keycode)
    }

    /// Check if there is content to paste.
    #[must_use]
    pub fn has_content(&self) -> bool {
        self.content.is_some()
    }

    /// Check if this is a cut operation (vs copy).
    #[must_use]
    pub fn is_cut(&self) -> bool {
        self.cut_source.is_some()
    }

    /// Get the clipboard content for pasting.
    #[must_use]
    pub fn get_content(&self) -> Option<&ClipboardContent> {
        self.content.as_ref()
    }

    /// Get the cut source position (if this was a cut operation).
    #[must_use]
    pub fn get_cut_source(&self) -> Option<(usize, Position)> {
        self.cut_source
    }

    /// Check if a given position is the cut source (for visual feedback).
    #[must_use]
    pub fn is_cut_source(&self, layer_index: usize, position: Position) -> bool {
        self.cut_source == Some((layer_index, position))
    }

    /// Clear the cut source after paste.
    /// The clipboard content is kept so the same data can be pasted again.
    pub fn clear_cut_source(&mut self) {
        self.cut_source = None;
    }

    /// Cancel cut operation (Esc pressed).
    /// Clears cut source but keeps clipboard content for potential paste.
    pub fn cancel_cut(&mut self) {
        self.cut_source = None;
    }

    /// Clear the entire clipboard.
    pub fn clear(&mut self) {
        self.content = None;
        self.cut_source = None;
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
    }

    #[test]
    fn test_clipboard_copy() {
        let mut clipboard = KeyClipboard::new();
        let msg = clipboard.copy("KC_A", None, None);

        assert!(clipboard.has_content());
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
}
