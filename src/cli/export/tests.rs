//! Tests for export.
//!
//! Auto-extracted from export.rs.

use super::*;

    use super::*;

    #[test]
    fn test_get_output_path_default() {
        let args = ExportArgs {
            layout: PathBuf::from("test.md"),
            qmk_path: PathBuf::from("/qmk"),
            output: None,
            layout_name: None,
        };

        let layout = Layout::new("My Test Layout").unwrap();
        let path = args.get_output_path(&layout);

        let path_str = path.to_string_lossy();
        assert!(path_str.contains("my_test_layout_export_"));
        assert!(path_str.ends_with(".md"));
    }

    #[test]
    fn test_get_output_path_custom() {
        let custom_path = PathBuf::from("/tmp/my_export.md");
        let args = ExportArgs {
            layout: PathBuf::from("test.md"),
            qmk_path: PathBuf::from("/qmk"),
            output: Some(custom_path.clone()),
            layout_name: None,
        };

        let layout = Layout::new("Test").unwrap();
        let path = args.get_output_path(&layout);

        assert_eq!(path, custom_path);
    }
