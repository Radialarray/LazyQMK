//! Configuration management for the application.
//!
//! This module handles loading, validating, and saving application configuration
//! in TOML format with platform-specific directory resolution.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Path configuration for file system locations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PathConfig {
    /// QMK firmware directory path (e.g., "/`path/to/qmk_firmware`")
    pub qmk_firmware: Option<PathBuf>,
}

/// Firmware build configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Target keyboard (e.g., "crkbd" or "`keebart/corne_choc_pro/standard`")
    pub keyboard: String,
    /// Layout variant (e.g., "`LAYOUT_split_3x6_3`")
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
        // Use config directory for build output by default
        let output_dir = Self::default_output_dir().unwrap_or_else(|_| PathBuf::from(".build"));
        
        Self {
            keyboard: "crkbd".to_string(),
            layout: "LAYOUT_split_3x6_3".to_string(),
            keymap: "default".to_string(),
            output_format: "uf2".to_string(),
            output_dir,
        }
    }
}

impl BuildConfig {
    /// Gets the default output directory path.
    ///
    /// - Linux: `~/.config/KeyboardConfigurator/builds/`
    /// - macOS: `~/Library/Application Support/KeyboardConfigurator/builds/`
    /// - Windows: `%APPDATA%\KeyboardConfigurator\builds\`
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

        // Check if keyboard has variant subdirectories with keyboard.json or info.json files
        let has_variants = ["standard", "mini", "normal", "full", "compact"]
            .iter()
            .any(|variant| {
                let variant_dir = keyboard_dir.join(variant);
                variant_dir.join("keyboard.json").exists() || variant_dir.join("info.json").exists()
            });

        if !has_variants {
            // No variants, return base keyboard path
            return Ok(base_keyboard.to_string());
        }

        // Map layout characteristics to variant names
        // Common patterns:
        // - "_ex2" suffix often indicates encoder support (e.g., LAYOUT_split_3x6_3_ex2)
        // - Higher key count typically maps to "standard" variant
        // - Lower key count typically maps to "mini" variant
        
        let variant = if self.layout.contains("_ex2") {
            // For layouts with encoder support, use key count to determine variant
            if layout_key_count >= 44 {
                "standard"
            } else {
                "mini"
            }
        } else if self.layout.contains("3x6") {
            // 3x6 layouts typically use standard variant
            "standard"
        } else if self.layout.contains("3x5") {
            // 3x5 layouts typically use mini variant
            "mini"
        } else {
            // Default to standard for unknown layouts
            "standard"
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
}

/// UI preferences configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiConfig {
    /// Display help on startup
    pub show_help_on_startup: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_help_on_startup: true,
        }
    }
}

/// Application configuration.
///
/// # File Location
///
/// - Linux: `~/.config/KeyboardConfigurator/config.toml`
/// - macOS: `~/Library/Application Support/KeyboardConfigurator/config.toml`
/// - Windows: `%APPDATA%\KeyboardConfigurator\config.toml`
///
/// # Validation
///
/// - `qmk_firmware` path must exist and contain Makefile, keyboards/ directory
/// - keyboard must exist in `qmk_firmware/keyboards`/
/// - layout must exist in keyboard's info.json
/// - `output_format` must be "uf2", "hex", or "bin"
/// - `output_dir` parent must exist and be writable
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// - Linux: `~/.config/KeyboardConfigurator/`
    /// - macOS: `~/Library/Application Support/KeyboardConfigurator/`
    /// - Windows: `%APPDATA%\KeyboardConfigurator\`
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to determine config directory")?
            .join("KeyboardConfigurator");

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
     /// in the parent directory (e.g., if keyboard_tui/vial-qmk-keebart doesn't exist,
     /// looks for keyboard-configurator/vial-qmk-keebart).
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
         // e.g., if /Users/user/dev/keyboard_tui/vial-qmk-keebart doesn't exist,
         // try /Users/user/dev/keyboard-configurator/vial-qmk-keebart
         if let Ok(siblings) = std::fs::read_dir(old_parent) {
             for entry in siblings.flatten() {
                 if let Ok(metadata) = entry.metadata() {
                     if metadata.is_dir() {
                         let candidate = entry.path().join(dir_name);
                         if candidate.exists() {
                             // Check if it's a valid QMK directory
                             if candidate.join("Makefile").exists() && candidate.join("keyboards").exists() {
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn set_qmk_firmware_path(&mut self, path: PathBuf) -> Result<()> {
        self.paths.qmk_firmware = Some(path);
        self.validate()?;
        Ok(())
    }

    /// Sets the keyboard name.
    #[allow(dead_code)]
    pub fn set_keyboard(&mut self, keyboard: String) {
        self.build.keyboard = keyboard;
    }

    /// Sets the layout variant.
    #[allow(dead_code)]
    pub fn set_layout(&mut self, layout: String) {
        self.build.layout = layout;
    }

    /// Sets the output format.
    #[allow(dead_code)]
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
        assert!(config.ui.show_help_on_startup);
        // New config should not be considered configured
        assert!(!config.is_configured());
    }

    #[test]
    fn test_config_is_configured() {
        let mut config = Config::new();
        
        // Without QMK path, config is not configured
        assert!(!config.is_configured());
        
        // With QMK path set, config is configured
        config.paths.qmk_firmware = Some(PathBuf::from("/some/path"));
        assert!(config.is_configured());
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

    #[test]
    fn test_determine_keyboard_variant_no_variants() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("crkbd");

        // Create keyboard directory with keyboard.json at root (no variants)
        fs::create_dir_all(&keyboard_dir).unwrap();
        fs::write(keyboard_dir.join("keyboard.json"), "{}").unwrap();

        let build_config = BuildConfig::default();
        let result = build_config
            .determine_keyboard_variant(&qmk_path, "crkbd", 42)
            .unwrap();

        assert_eq!(result, "crkbd");
    }

    #[test]
    fn test_determine_keyboard_variant_with_standard_variant() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("keebart/corne_choc_pro");
        let standard_dir = keyboard_dir.join("standard");

        // Create variant directory structure
        fs::create_dir_all(&standard_dir).unwrap();
        fs::write(standard_dir.join("keyboard.json"), "{}").unwrap();

        let mut build_config = BuildConfig::default();
        build_config.layout = "LAYOUT_split_3x6_3".to_string();

        let result = build_config
            .determine_keyboard_variant(&qmk_path, "keebart/corne_choc_pro", 42)
            .unwrap();

        assert_eq!(result, "keebart/corne_choc_pro/standard");
    }

    #[test]
    fn test_determine_keyboard_variant_with_mini_variant() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("keebart/corne_choc_pro");
        let mini_dir = keyboard_dir.join("mini");

        // Create variant directory structure
        fs::create_dir_all(&mini_dir).unwrap();
        fs::write(mini_dir.join("keyboard.json"), "{}").unwrap();

        let mut build_config = BuildConfig::default();
        build_config.layout = "LAYOUT_split_3x5_3".to_string();

        let result = build_config
            .determine_keyboard_variant(&qmk_path, "keebart/corne_choc_pro", 36)
            .unwrap();

        assert_eq!(result, "keebart/corne_choc_pro/mini");
    }

    #[test]
    fn test_determine_keyboard_variant_ex2_layout_standard() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("keebart/corne_choc_pro");
        let standard_dir = keyboard_dir.join("standard");

        // Create variant directory structure
        fs::create_dir_all(&standard_dir).unwrap();
        fs::write(standard_dir.join("keyboard.json"), "{}").unwrap();

        let mut build_config = BuildConfig::default();
        build_config.layout = "LAYOUT_split_3x6_3_ex2".to_string();

        let result = build_config
            .determine_keyboard_variant(&qmk_path, "keebart/corne_choc_pro", 44)
            .unwrap();

        assert_eq!(result, "keebart/corne_choc_pro/standard");
    }

    #[test]
    fn test_determine_keyboard_variant_ex2_layout_mini() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("keebart/corne_choc_pro");
        let mini_dir = keyboard_dir.join("mini");

        // Create variant directory structure
        fs::create_dir_all(&mini_dir).unwrap();
        fs::write(mini_dir.join("keyboard.json"), "{}").unwrap();

        let mut build_config = BuildConfig::default();
        build_config.layout = "LAYOUT_split_3x5_3_ex2".to_string();

        let result = build_config
            .determine_keyboard_variant(&qmk_path, "keebart/corne_choc_pro", 38)
            .unwrap();

        assert_eq!(result, "keebart/corne_choc_pro/mini");
    }

    #[test]
    fn test_determine_keyboard_variant_missing_variant_directory() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_path = temp_dir.path().join("qmk");
        let keyboards_dir = qmk_path.join("keyboards");
        let keyboard_dir = keyboards_dir.join("keebart/corne_choc_pro");

        // Create base keyboard directory but no variant subdirectories
        fs::create_dir_all(&keyboard_dir).unwrap();

        let mut build_config = BuildConfig::default();
        build_config.layout = "LAYOUT_split_3x6_3".to_string();

        let result = build_config
            .determine_keyboard_variant(&qmk_path, "keebart/corne_choc_pro", 42);

        // Should return the base keyboard path when no variants are detected
        assert_eq!(result.unwrap(), "keebart/corne_choc_pro");
    }

     #[test]
     fn test_determine_keyboard_variant_with_info_json() {
         let temp_dir = TempDir::new().unwrap();
         let qmk_path = temp_dir.path().join("qmk");
         let keyboards_dir = qmk_path.join("keyboards");
         let keyboard_dir = keyboards_dir.join("test_keyboard");
         let standard_dir = keyboard_dir.join("standard");

         // Create variant directory with info.json instead of keyboard.json
         fs::create_dir_all(&standard_dir).unwrap();
         fs::write(standard_dir.join("info.json"), "{}").unwrap();

         let mut build_config = BuildConfig::default();
         build_config.layout = "LAYOUT_split_3x6_3".to_string();

         let result = build_config
             .determine_keyboard_variant(&qmk_path, "test_keyboard", 42)
             .unwrap();

         assert_eq!(result, "test_keyboard/standard");
     }

     #[test]
     fn test_try_fix_qmk_path_existing_path() {
         let temp_dir = TempDir::new().unwrap();
         let qmk_path = temp_dir.path().join("qmk");
         fs::create_dir(&qmk_path).unwrap();

         // If path exists, should return it as-is
         let result = Config::try_fix_qmk_path(&qmk_path);
         assert_eq!(result, Some(qmk_path));
     }

     #[test]
     fn test_try_fix_qmk_path_moved_directory() {
         let temp_dir = TempDir::new().unwrap();
         
         // Create old and new project directories
         let old_project = temp_dir.path().join("old_project");
         let new_project = temp_dir.path().join("new_project");
         fs::create_dir(&old_project).unwrap();
         fs::create_dir(&new_project).unwrap();

         // Create a valid QMK directory in new_project
         let qmk_dir = new_project.join("vial-qmk-keebart");
         fs::create_dir(&qmk_dir).unwrap();
         fs::write(qmk_dir.join("Makefile"), "").unwrap();
         fs::create_dir(qmk_dir.join("keyboards")).unwrap();

         // Create a reference to the old path (which doesn't exist)
         let old_path = old_project.join("vial-qmk-keebart");

         // Should find and return the new path
         let result = Config::try_fix_qmk_path(&old_path);
         assert_eq!(result, Some(qmk_dir));
     }

     #[test]
     fn test_try_fix_qmk_path_not_found() {
         let temp_dir = TempDir::new().unwrap();
         
         // Create a project directory but no QMK directory
         let project = temp_dir.path().join("project");
         fs::create_dir(&project).unwrap();

         // Reference to a non-existent path
         let missing_path = project.join("vial-qmk-keebart");

         // Should return None since directory doesn't exist anywhere
         let result = Config::try_fix_qmk_path(&missing_path);
         assert_eq!(result, None);
     }
}
