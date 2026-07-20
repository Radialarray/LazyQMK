//! Tests for clipboard.
//!
//! Auto-extracted from clipboard.rs.

use super::*;

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
        assert!(msg.contains("Cut queued: KC_C"));
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
        assert_eq!(
            clipboard.get_preview(),
            Some("KC_A (queued cut)".to_string())
        );
    }

    #[test]
    fn test_clipboard_multi_copy() {
        let mut clipboard = KeyClipboard::new();
        let keys = vec![
            (
                Position::new(0, 0),
                ClipboardContent {
                    keycode: "KC_A".to_string(),
                    color_override: None,
                    category_id: None,
                },
            ),
            (
                Position::new(0, 1),
                ClipboardContent {
                    keycode: "KC_B".to_string(),
                    color_override: None,
                    category_id: None,
                },
            ),
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
            (
                Position::new(0, 0),
                ClipboardContent {
                    keycode: "KC_A".to_string(),
                    color_override: None,
                    category_id: None,
                },
            ),
            (
                Position::new(0, 1),
                ClipboardContent {
                    keycode: "KC_B".to_string(),
                    color_override: None,
                    category_id: None,
                },
            ),
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

        let keys = vec![(
            Position::new(0, 0),
            ClipboardContent {
                keycode: "KC_A".to_string(),
                color_override: None,
                category_id: None,
            },
        )];
        clipboard.save_undo(0, keys, "Pasted KC_B".to_string());

        assert!(clipboard.can_undo());

        let undo = clipboard.take_undo().unwrap();
        assert_eq!(undo.layer_index, 0);
        assert_eq!(undo.original_keys.len(), 1);
        assert_eq!(undo.description, "Pasted KC_B");

        assert!(!clipboard.can_undo());
    }
