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

/// The GitHub repository owner/organization name.
pub const GITHUB_OWNER: &str = "Radialarray";

/// The GitHub repository name.
pub const GITHUB_REPO: &str = "LazyQMK";

/// The full GitHub repository URL.
pub const GITHUB_URL: &str = "https://github.com/Radialarray/LazyQMK";

/// Short description for package metadata and help text.
pub const APP_DESCRIPTION: &str = "Interactive terminal workspace for QMK firmware";

/// Long description for documentation and README.
pub const APP_DESCRIPTION_LONG: &str = 
    "A modern terminal-based keyboard layout editor for QMK firmware. \
     Design keymaps, manage layers, organize with colors and categories, \
     and compile firmware—all without leaving your terminal.";

// ============================================================================
// Derived constants (computed from the above values)
// ============================================================================

/// GitHub releases URL.
pub fn github_releases_url() -> String {
    format!("{}/releases", GITHUB_URL)
}

/// GitHub issues URL.
pub fn github_issues_url() -> String {
    format!("{}/issues", GITHUB_URL)
}

/// Installation command (cargo install from git).
pub fn install_command() -> String {
    format!("cargo install --git {}.git", GITHUB_URL)
}

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
        
        // Ensure GitHub URL is well-formed
        assert!(GITHUB_URL.starts_with("https://github.com/"));
        assert!(GITHUB_URL.contains(GITHUB_OWNER));
        assert!(GITHUB_URL.contains(GITHUB_REPO));
    }

    #[test]
    fn test_derived_urls() {
        assert_eq!(
            github_releases_url(),
            format!("{}/releases", GITHUB_URL)
        );
        assert_eq!(
            github_issues_url(),
            format!("{}/issues", GITHUB_URL)
        );
    }

    #[test]
    fn test_install_command() {
        let cmd = install_command();
        assert!(cmd.contains("cargo install"));
        assert!(cmd.contains(GITHUB_URL));
        assert!(cmd.ends_with(".git"));
    }
}
