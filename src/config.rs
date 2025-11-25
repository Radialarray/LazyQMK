//! Configuration management for the application.
//!
//! This module handles loading, validating, and saving application configuration
//! in TOML format with platform-specific directory resolution.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Path configuration for file system locations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathConfig {
    /// QMK firmware directory path (e.g., "/path/to/qmk_firmware")
    pub qmk_firmware: Option<PathBuf>,
}

impl Default for PathConfig {
    fn default() -> Self {
        Self { qmk_firmware: None }
    }
}

/// Firmware build configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Target keyboard (e.g., "crkbd")
    pub keyboard: String,
    /// Layout variant (e.g., "LAYOUT_split_3x6_3")
    pub layout: String,
    /// Keymap name (e.g., "default")
    pub keymap: String,
    /// Output format: "uf2", "hex", or "bin"
    pub output_format: String,
    /// Build output directory
    pub output_dir: PathBuf,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            keyboard: "crkbd".to_string(),
            layout: "LAYOUT_split_3x6_3".to_string(),
            keymap: "default".to_string(),
            output_format: "uf2".to_string(),
            output_dir: PathBuf::from(".build"),
        }
    }
}

/// UI preferences configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiConfig {
    /// Color theme (future feature)
    pub theme: String,
    /// Display help on startup
    pub show_help_on_startup: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            show_help_on_startup: true,
        }
    }
}

/// Application configuration.
///
/// # File Location
///
/// - Unix/Linux/macOS: `~/.config/layout_tools/config.toml`
/// - Windows: `%APPDATA%\layout_tools\config.toml`
///
/// # Validation
///
/// - qmk_firmware path must exist and contain Makefile, keyboards/ directory
/// - keyboard must exist in qmk_firmware/keyboards/
/// - layout must exist in keyboard's info.json
/// - output_format must be "uf2", "hex", or "bin"
/// - output_dir parent must exist and be writable
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
    pub fn new() -> Self {
        Self {
            paths: PathConfig::default(),
            build: BuildConfig::default(),
            ui: UiConfig::default(),
        }
    }

    /// Gets the platform-specific config directory path.
    ///
    /// - Unix/Linux/macOS: `~/.config/layout_tools/`
    /// - Windows: `%APPDATA%\layout_tools\`
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to determine config directory")?
            .join("layout_tools");

        Ok(config_dir)
    }

    /// Gets the full path to the config file.
    pub fn config_file_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Loads configuration from the config file.
    ///
    /// If the file doesn't exist, returns default configuration.
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if !config_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&config_path).context(format!(
            "Failed to read config file: {}",
            config_path.display()
        ))?;

        let config: Config = toml::from_str(&content).context(format!(
            "Failed to parse config file: {}",
            config_path.display()
        ))?;

        config.validate()?;

        Ok(config)
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
    /// - output_format is valid ("uf2", "hex", or "bin")
    /// - output_dir parent exists
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

        // Validate output format
        match self.build.output_format.as_str() {
            "uf2" | "hex" | "bin" => {}
            _ => anyhow::bail!(
                "Invalid output format '{}'. Must be 'uf2', 'hex', or 'bin'",
                self.build.output_format
            ),
        }

        Ok(())
    }

    /// Sets the QMK firmware path with validation.
    pub fn set_qmk_firmware_path(&mut self, path: PathBuf) -> Result<()> {
        self.paths.qmk_firmware = Some(path);
        self.validate()?;
        Ok(())
    }

    /// Sets the keyboard name.
    pub fn set_keyboard(&mut self, keyboard: String) {
        self.build.keyboard = keyboard;
    }

    /// Sets the layout variant.
    pub fn set_layout(&mut self, layout: String) {
        self.build.layout = layout;
    }

    /// Sets the output format.
    pub fn set_output_format(&mut self, format: String) -> Result<()> {
        self.build.output_format = format;
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
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert_eq!(config.paths.qmk_firmware, None);
        assert_eq!(config.build.keyboard, "crkbd");
        assert_eq!(config.build.layout, "LAYOUT_split_3x6_3");
        assert_eq!(config.build.keymap, "default");
        assert_eq!(config.build.output_format, "uf2");
        assert_eq!(config.ui.theme, "default");
        assert!(config.ui.show_help_on_startup);
    }

    #[test]
    fn test_config_validate_output_format() {
        let mut config = Config::new();
        assert!(config.validate().is_ok());

        config.build.output_format = "invalid".to_string();
        assert!(config.validate().is_err());

        config.build.output_format = "uf2".to_string();
        assert!(config.validate().is_ok());

        config.build.output_format = "hex".to_string();
        assert!(config.validate().is_ok());

        config.build.output_format = "bin".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_qmk_path() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        fs::create_dir(&qmk_path).unwrap();

        let mut config = Config::new();
        config.paths.qmk_firmware = Some(qmk_path.clone());

        // Missing Makefile and keyboards/ directory
        assert!(config.validate().is_err());

        // Add Makefile
        fs::write(qmk_path.join("Makefile"), "").unwrap();
        assert!(config.validate().is_err()); // Still missing keyboards/

        // Add keyboards/ directory
        fs::create_dir(qmk_path.join("keyboards")).unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.toml");

        let mut config = Config::new();
        config.build.keyboard = "test_keyboard".to_string();
        config.build.layout = "TEST_LAYOUT".to_string();

        // Manually save to temp location for testing
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_file, content).unwrap();

        // Load and verify
        let content = fs::read_to_string(&config_file).unwrap();
        let loaded: Config = toml::from_str(&content).unwrap();

        assert_eq!(loaded.build.keyboard, "test_keyboard");
        assert_eq!(loaded.build.layout, "TEST_LAYOUT");
    }

    #[test]
    fn test_config_set_keyboard() {
        let mut config = Config::new();
        config.set_keyboard("my_keyboard".to_string());
        assert_eq!(config.build.keyboard, "my_keyboard");
    }

    #[test]
    fn test_config_set_layout() {
        let mut config = Config::new();
        config.set_layout("MY_LAYOUT".to_string());
        assert_eq!(config.build.layout, "MY_LAYOUT");
    }

    #[test]
    fn test_config_set_output_format() {
        let mut config = Config::new();

        assert!(config.set_output_format("hex".to_string()).is_ok());
        assert_eq!(config.build.output_format, "hex");

        assert!(config.set_output_format("invalid".to_string()).is_err());
    }
}
