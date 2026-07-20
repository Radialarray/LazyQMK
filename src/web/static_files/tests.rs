//! Tests for static_files.
//!
//! Auto-extracted from static_files.rs.

use super::*;

    use super::*;

    #[test]
    fn test_cache_control_for_path() {
        // Immutable assets get long cache
        assert_eq!(
            cache_control_for_path("_app/immutable/chunks/0.abc123.js"),
            "public, max-age=31536000, immutable"
        );

        // HTML gets no-cache
        assert_eq!(
            cache_control_for_path("index.html"),
            "no-cache, must-revalidate"
        );
        assert_eq!(
            cache_control_for_path("about.html"),
            "no-cache, must-revalidate"
        );

        // Other files get short cache
        assert_eq!(
            cache_control_for_path("favicon.png"),
            "public, max-age=3600"
        );
        assert_eq!(
            cache_control_for_path("manifest.json"),
            "public, max-age=3600"
        );
    }

    #[test]
    fn test_has_embedded_assets() {
        // This test checks the function works - result depends on whether
        // web/build exists at compile time
        let _ = has_embedded_assets();
    }
