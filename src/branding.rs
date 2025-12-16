//! Branding and application identity configuration.
//!
//! This module centralizes all branding-related strings (names, paths, URLs) to make
//! future rebranding easier. Change values here to rebrand the entire application.

/// The human-readable display name of the application.
///
/// Used in:
/// - Window titles
/// - Help text
/// - About dialogs
/// - Documentation
pub const APP_DISPLAY_NAME: &str = "LazyQMK";

/// The binary/executable name (lowercase, no spaces).
///
/// Used in:
/// - Cargo.toml package name
/// - Binary executable name
/// - Command examples in documentation
/// - Generated code comments
pub const APP_BINARY_NAME: &str = "lazyqmk";

/// The directory name for application data (config, templates, builds).
///
/// Used in platform-specific paths:
/// - Linux: `~/.config/{APP_DATA_DIR}/`
/// - macOS: `~/Library/Application Support/{APP_DATA_DIR}/`
/// - Windows: `%APPDATA%\{APP_DATA_DIR}\`
pub const APP_DATA_DIR: &str = "LazyQMK";

/// Short description for package metadata and help text.
pub const APP_DESCRIPTION: &str = "Interactive terminal workspace for QMK firmware";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branding_consistency() {
        // Ensure binary name is lowercase
        assert_eq!(APP_BINARY_NAME, APP_BINARY_NAME.to_lowercase());

        // Ensure no spaces in binary name
        assert!(!APP_BINARY_NAME.contains(' '));

        // Ensure no spaces in data dir
        assert!(!APP_DATA_DIR.contains(' '));
    }
}
