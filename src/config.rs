//! Configuration management for the application.
//!
//! This module handles loading, validating, and saving application configuration
//! in TOML format with platform-specific directory resolution.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Theme display mode preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeMode {
    /// Automatically detect OS theme (dark/light)
    #[default]
    Auto,
    /// Always use dark theme
    Dark,
    /// Always use light theme
    Light,
}

/// Path configuration for file system locations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PathConfig {
    /// QMK firmware directory path (e.g., "/`path/to/qmk_firmware`")
    pub qmk_firmware: Option<PathBuf>,
}

/// Firmware build configuration.
///
/// Note: keyboard, `layout_variant`, `keymap_name`, and `output_format` have been moved
/// to per-layout .md file metadata. Only global build settings remain here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Build output directory (where all firmware files go)
    pub output_dir: PathBuf,
}

impl Default for BuildConfig {
    fn default() -> Self {
        // Use config directory for build output by default
        let output_dir = Self::default_output_dir().unwrap_or_else(|_| PathBuf::from(".build"));

        Self { output_dir }
    }
}

impl BuildConfig {
    /// Gets the default output directory path.
    ///
    /// - Linux: `~/.config/LazyQMK/builds/`
    /// - macOS: `~/Library/Application Support/LazyQMK/builds/`
    /// - Windows: `%APPDATA%\LazyQMK\builds\`
    fn default_output_dir() -> Result<PathBuf> {
        Ok(Config::config_dir()?.join("builds"))
    }

    /// Determines the keyboard variant subdirectory based on layout and key count.
    ///
    /// Some keyboards have variant subdirectories (e.g., "standard", "mini") that contain
    /// variant-specific configuration like RGB matrix LED layouts. This function detects
    /// the appropriate variant based on the layout name and validates it exists.
    ///
    /// # Arguments
    ///
    /// * `qmk_path` - Path to QMK firmware directory
    /// * `base_keyboard` - Base keyboard path without variant (e.g., "`keebart/corne_choc_pro`")
    /// * `layout_key_count` - Number of keys in the selected layout
    ///
    /// # Returns
    ///
    /// Returns the full keyboard path with variant if applicable (e.g., "`keebart/corne_choc_pro/standard`"),
    /// or the base keyboard path if no variant is needed.
    pub fn determine_keyboard_variant(
        &self,
        qmk_path: &std::path::Path,
        base_keyboard: &str,
        layout_key_count: usize,
    ) -> Result<String> {
        let keyboard_dir = qmk_path.join("keyboards").join(base_keyboard);

        // Discover all variant subdirectories dynamically by scanning the filesystem
        let discovered_variants = Self::discover_keyboard_variants(&keyboard_dir)?;

        if discovered_variants.is_empty() {
            // No variants, return base keyboard path
            return Ok(base_keyboard.to_string());
        }

        // Map layout characteristics to variant names
        // Common patterns:
        // - "_ex2" suffix often indicates encoder support (e.g., LAYOUT_split_3x6_3_ex2)
        // - Higher key count typically maps to "standard" variant
        // - Lower key count typically maps to "mini" variant

        // Since layout field is removed, use key count heuristics
        // Common patterns:
        // - 44+ keys: typically "standard" variant
        // - Less than 44 keys: typically "mini" variant
        let preferred_variant = if layout_key_count >= 44 {
            "standard"
        } else {
            "mini"
        };

        // Try to find the preferred variant in discovered variants
        let variant = if discovered_variants.contains(&preferred_variant.to_string()) {
            preferred_variant
        } else {
            // Fallback: use the first discovered variant
            // This handles cases where keyboards use non-standard variant names
            &discovered_variants[0]
        };

        let variant_path = format!("{base_keyboard}/{variant}");
        let variant_dir = qmk_path.join("keyboards").join(&variant_path);

        // Validate the variant directory exists
        if !variant_dir.exists() {
            anyhow::bail!(
                "Keyboard variant directory not found: {}. Available variants should be in {}",
                variant_dir.display(),
                keyboard_dir.display()
            );
        }

        Ok(variant_path)
    }

    /// Discovers all keyboard variant subdirectories by scanning the filesystem.
    ///
    /// A directory is considered a variant if it contains `keyboard.json` or `info.json`.
    ///
    /// # Arguments
    ///
    /// * `keyboard_dir` - Path to the keyboard directory to scan
    ///
    /// # Returns
    ///
    /// Returns a vector of variant names (just the directory names, not full paths).
    /// Returns empty vector if no variants found or if the directory doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns error if filesystem operations fail (other than non-existent directory).
    fn discover_keyboard_variants(keyboard_dir: &std::path::Path) -> Result<Vec<String>> {
        let mut variants = Vec::new();

        // If directory doesn't exist, return empty list (not an error)
        if !keyboard_dir.exists() {
            return Ok(variants);
        }

        // Read all subdirectories
        let entries = fs::read_dir(keyboard_dir).context(format!(
            "Failed to read keyboard directory: {}",
            keyboard_dir.display()
        ))?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            // Check if it's a directory
            if !path.is_dir() {
                continue;
            }

            // Check if it contains keyboard.json or info.json
            let has_keyboard_json = path.join("keyboard.json").exists();
            let has_info_json = path.join("info.json").exists();

            if has_keyboard_json || has_info_json {
                // Extract just the directory name
                if let Some(dir_name) = path.file_name() {
                    if let Some(name_str) = dir_name.to_str() {
                        variants.push(name_str.to_string());
                    }
                }
            }
        }

        Ok(variants)
    }
}

/// UI preferences configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiConfig {
    /// Display help on startup
    pub show_help_on_startup: bool,
    /// Theme mode preference (Auto, Dark, Light)
    #[serde(default)]
    pub theme_mode: ThemeMode,
    /// Unified keyboard scale factor (1.0 = default, <1.0 smaller, >1.0 larger)
    #[serde(default = "default_keyboard_scale")]
    pub keyboard_scale: f32,
    /// Last selected language in the keycode picker (for convenience)
    #[serde(default)]
    pub last_language: Option<String>,
}

/// Default keyboard scale (1.0 = 100%)
fn default_keyboard_scale() -> f32 {
    1.0
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_help_on_startup: true,
            theme_mode: ThemeMode::default(),
            keyboard_scale: default_keyboard_scale(),
            last_language: None,
        }
    }
}

/// Application configuration.
///
/// # File Location
///
/// - Linux: `~/.config/LazyQMK/config.toml`
/// - macOS: `~/Library/Application Support/LazyQMK/config.toml`
/// - Windows: `%APPDATA%\LazyQMK\config.toml`
///
/// # Validation
///
/// - `qmk_firmware` path must exist and contain Makefile, keyboards/ directory
/// - keyboard must exist in `qmk_firmware/keyboards`/
/// - layout must exist in keyboard's info.json
/// - `output_format` must be "uf2", "hex", or "bin"
/// - `output_dir` parent must exist and be writable
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// File system paths
    pub paths: PathConfig,
    /// Firmware build settings
    pub build: BuildConfig,
    /// UI preferences
    pub ui: UiConfig,
}

impl Config {
    /// Creates a new Config with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            paths: PathConfig::default(),
            build: BuildConfig::default(),
            ui: UiConfig::default(),
        }
    }

    /// Checks if the config file exists on disk.
    ///
    /// Returns true if config.toml exists, false otherwise.
    #[must_use]
    pub fn exists() -> bool {
        Self::config_file_path()
            .map(|path| path.exists())
            .unwrap_or(false)
    }

    /// Checks if the configuration has been properly set up.
    ///
    /// A config is considered "configured" if the QMK firmware path is set.
    /// This is used to detect first-run scenarios where the wizard should be shown.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        self.paths.qmk_firmware.is_some()
    }

    /// Gets the platform-specific config directory path.
    ///
    /// - Linux: `~/.config/LazyQMK/`
    /// - macOS: `~/Library/Application Support/LazyQMK/`
    /// - Windows: `%APPDATA%\LazyQMK\`
    ///
    /// For testing, this can be overridden with the `LAZYQMK_CONFIG_DIR` environment variable.
    pub fn config_dir() -> Result<PathBuf> {
        // Check for test override first
        if let Ok(test_dir) = std::env::var("LAZYQMK_CONFIG_DIR") {
            return Ok(PathBuf::from(test_dir));
        }

        // Normal behavior: use platform-specific config directory
        let config_dir = dirs::config_dir()
            .context("Failed to determine config directory")?
            .join(crate::branding::APP_DATA_DIR);

        Ok(config_dir)
    }

    /// Gets the full path to the config file.
    pub fn config_file_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Loads configuration from the config file.
    ///
    /// If the file doesn't exist, returns default configuration.
    /// If the QMK path is invalid but the directory was moved, attempts to auto-fix it.
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if !config_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&config_path).context(format!(
            "Failed to read config file: {}",
            config_path.display()
        ))?;

        let mut config: Self = toml::from_str(&content).context(format!(
            "Failed to parse config file: {}",
            config_path.display()
        ))?;

        // Try to validate; if QMK path is invalid, attempt to auto-fix it
        if let Err(validation_err) = config.validate() {
            if let Some(qmk_path) = &config.paths.qmk_firmware {
                // Check if this looks like it was a renamed/moved directory
                if let Some(fixed_path) = Self::try_fix_qmk_path(qmk_path) {
                    config.paths.qmk_firmware = Some(fixed_path);
                    // Try validating again with the fixed path
                    config.validate()?;
                    // Successfully fixed - save the corrected config
                    config.save()?;
                    return Ok(config);
                }
            }
            // Couldn't auto-fix, return the original validation error
            return Err(validation_err);
        }

        Ok(config)
    }

    /// Attempts to fix a stale QMK firmware path.
    ///
    /// If the path doesn't exist, looks for a directory with similar naming
    /// in the parent directory (e.g., if `old_project/qmk_firmware` doesn't exist,
    /// looks for `LazyQMK/qmk_firmware`).
    fn try_fix_qmk_path(old_path: &std::path::Path) -> Option<PathBuf> {
        // If the path exists, no fix needed
        if old_path.exists() {
            return Some(old_path.to_path_buf());
        }

        // Get the directory name (e.g., "vial-qmk-keebart")
        let dir_name = old_path.file_name()?;

        // Get the parent of the parent (e.g., /Users/user/dev)
        let old_parent = old_path.parent()?.parent()?;

        // Look for the directory in siblings of the parent
        // e.g., if /Users/user/dev/old_project/qmk_firmware doesn't exist,
        // try /Users/user/dev/LazyQMK/qmk_firmware
        if let Ok(siblings) = std::fs::read_dir(old_parent) {
            for entry in siblings.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        let candidate = entry.path().join(dir_name);
                        if candidate.exists() {
                            // Check if it's a valid QMK directory
                            if candidate.join("Makefile").exists()
                                && candidate.join("keyboards").exists()
                            {
                                return Some(candidate);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Saves configuration to the config file using atomic write.
    ///
    /// Uses temp file + rename pattern for atomic writes.
    pub fn save(&self) -> Result<()> {
        self.validate()?;

        // Ensure config directory exists
        let config_dir = Self::config_dir()?;
        fs::create_dir_all(&config_dir).context(format!(
            "Failed to create config directory: {}",
            config_dir.display()
        ))?;

        // Serialize to TOML
        let content = toml::to_string_pretty(self).context("Failed to serialize configuration")?;

        let config_path = Self::config_file_path()?;
        let temp_path = config_path.with_extension("toml.tmp");

        // Write to temp file
        fs::write(&temp_path, content).context(format!(
            "Failed to write temp config file: {}",
            temp_path.display()
        ))?;

        // Atomic rename
        fs::rename(&temp_path, &config_path).context(format!(
            "Failed to rename temp config file to: {}",
            config_path.display()
        ))?;

        Ok(())
    }

    /// Validates configuration values.
    ///
    /// Checks:
    /// - QMK firmware path exists (if set) and contains required files
    /// - `output_format` is valid ("uf2", "hex", or "bin")
    /// - `theme` is valid ("dark" or "light")
    /// - `output_dir` parent exists
    pub fn validate(&self) -> Result<()> {
        // Validate QMK firmware path if set
        if let Some(qmk_path) = &self.paths.qmk_firmware {
            if !qmk_path.exists() {
                anyhow::bail!("QMK firmware path does not exist: {}", qmk_path.display());
            }

            let makefile_path = qmk_path.join("Makefile");
            if !makefile_path.exists() {
                anyhow::bail!(
                    "QMK firmware path is invalid: Makefile not found at {}",
                    makefile_path.display()
                );
            }

            let keyboards_dir = qmk_path.join("keyboards");
            if !keyboards_dir.exists() || !keyboards_dir.is_dir() {
                anyhow::bail!(
                    "QMK firmware path is invalid: keyboards/ directory not found at {}",
                    keyboards_dir.display()
                );
            }
        }

        // Keyboard-specific settings (keyboard, layout, keymap, output_format)
        // are now stored in layout metadata, not in config.toml

        Ok(())
    }

    /// Sets the QMK firmware path with validation.
    pub fn set_qmk_firmware_path(&mut self, path: PathBuf) -> Result<()> {
        self.paths.qmk_firmware = Some(path);
        self.validate()?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;

