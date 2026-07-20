//! Tests for template.
//!
//! Auto-extracted from template.rs.

use super::*;

    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("My Layout"), "my_layout");
        assert_eq!(sanitize_filename("Corne Base"), "corne_base");
        assert_eq!(sanitize_filename("test-123"), "test-123");
        assert_eq!(
            sanitize_filename("Special!@#$%Characters"),
            "special_____characters"
        );
    }
