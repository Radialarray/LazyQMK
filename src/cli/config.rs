//! Configuration management CLI commands.

use crate::cli::common::{CliError, CliResult};
use crate::config::{Config, ThemeMode};
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

/// Configuration management commands
#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommand,
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    /// Display current configuration
    Show(ConfigShowArgs),
    /// Set configuration values
    Set(ConfigSetArgs),
}

/// Display current configuration
#[derive(Args, Debug)]
pub struct ConfigShowArgs {
    /// Output as JSON
    #[arg(long)]
    json: bool,
}

/// Set configuration values
#[derive(Args, Debug)]
pub struct ConfigSetArgs {
    /// QMK firmware directory path
    #[arg(long, value_name = "DIR")]
    qmk_path: Option<PathBuf>,

    /// Firmware build output directory
    #[arg(long, value_name = "DIR")]
    output_dir: Option<PathBuf>,

    /// Theme mode (auto, light, or dark)
    #[arg(long, value_name = "MODE")]
    theme: Option<String>,
}

/// JSON-serializable configuration for output
#[derive(Serialize, Debug)]
struct ConfigOutput {
    paths: PathsOutput,
    build: BuildOutput,
    ui: UiOutput,
}

#[derive(Serialize, Debug)]
struct PathsOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    qmk_firmware: Option<String>,
}

#[derive(Serialize, Debug)]
struct BuildOutput {
    output_dir: String,
}

#[derive(Serialize, Debug)]
struct UiOutput {
    theme: String,
}

impl ConfigArgs {
    /// Execute config subcommand
    pub fn execute(&self) -> CliResult<()> {
        match &self.command {
            ConfigCommand::Show(args) => args.execute(),
            ConfigCommand::Set(args) => args.execute(),
        }
    }
}

impl ConfigShowArgs {
    /// Execute show command
    pub fn execute(&self) -> CliResult<()> {
        let config = Config::load()
            .map_err(|e| CliError::validation(format!("Failed to load configuration: {}", e)))?;

        if self.json {
            output_json(&config)?;
        } else {
            output_human_readable(&config);
        }

        Ok(())
    }
}

impl ConfigSetArgs {
    /// Execute set command
    pub fn execute(&self) -> CliResult<()> {
        // At least one argument must be provided
        if self.qmk_path.is_none() && self.output_dir.is_none() && self.theme.is_none() {
            return Err(CliError::validation(
                "At least one configuration option must be specified: --qmk-path, --output-dir, or --theme"
            ));
        }

        // Load current configuration
        let mut config = Config::load().unwrap_or_else(|_| Config::default());

        // Validate and apply qmk_path if provided
        if let Some(path) = &self.qmk_path {
            if !path.exists() {
                return Err(CliError::validation(format!(
                    "QMK firmware directory does not exist: {}",
                    path.display()
                )));
            }

            if !path.join("Makefile").exists() {
                return Err(CliError::validation(format!(
                    "QMK firmware directory is invalid: Makefile not found at {}",
                    path.join("Makefile").display()
                )));
            }

            if !path.join("keyboards").exists() {
                return Err(CliError::validation(format!(
                    "QMK firmware directory is invalid: keyboards/ directory not found at {}",
                    path.join("keyboards").display()
                )));
            }

            config.paths.qmk_firmware = Some(path.clone());
        }

        // Apply output_dir if provided (create if doesn't exist)
        if let Some(path) = &self.output_dir {
            // Try to create the directory
            std::fs::create_dir_all(path).map_err(|e| {
                CliError::io(format!(
                    "Failed to create output directory {}: {}",
                    path.display(),
                    e
                ))
            })?;

            config.build.output_dir.clone_from(path);
        }

        // Validate and apply theme if provided
        if let Some(theme_str) = &self.theme {
            let theme = match theme_str.to_lowercase().as_str() {
                "auto" => ThemeMode::Auto,
                "light" => ThemeMode::Light,
                "dark" => ThemeMode::Dark,
                _ => {
                    return Err(CliError::validation(
                        "Invalid theme mode. Must be 'auto', 'light', or 'dark'".to_string(),
                    ))
                }
            };
            config.ui.theme_mode = theme;
        }

        // Save configuration
        config
            .save()
            .map_err(|e| CliError::io(format!("Failed to save configuration: {}", e)))?;

        println!("Configuration updated successfully.");

        Ok(())
    }
}

/// Output configuration in JSON format
fn output_json(config: &Config) -> CliResult<()> {
    let output = ConfigOutput {
        paths: PathsOutput {
            qmk_firmware: config
                .paths
                .qmk_firmware
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
        },
        build: BuildOutput {
            output_dir: config.build.output_dir.to_string_lossy().to_string(),
        },
        ui: UiOutput {
            theme: format!("{:?}", config.ui.theme_mode).to_lowercase(),
        },
    };

    let json = serde_json::to_string_pretty(&output)
        .map_err(|e| CliError::io(format!("Failed to serialize configuration to JSON: {}", e)))?;

    println!("{}", json);
    Ok(())
}

/// Output configuration in human-readable format
fn output_human_readable(config: &Config) {
    println!("LazyQMK Configuration");
    println!("====================");
    println!();

    println!("Paths:");
    if let Some(qmk_path) = &config.paths.qmk_firmware {
        println!("  QMK Firmware: {}", qmk_path.display());
    } else {
        println!("  QMK Firmware: (not configured)");
    }
    println!();

    println!("Build:");
    println!("  Output Directory: {}", config.build.output_dir.display());
    println!();

    println!("UI:");
    println!(
        "  Theme Mode: {}",
        format!("{:?}", config.ui.theme_mode).to_lowercase()
    );
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_mode_parsing() {
        assert_eq!(
            match "auto" {
                "auto" => ThemeMode::Auto,
                "light" => ThemeMode::Light,
                "dark" => ThemeMode::Dark,
                _ => ThemeMode::Auto,
            },
            ThemeMode::Auto
        );
        assert_eq!(
            match "light" {
                "auto" => ThemeMode::Auto,
                "light" => ThemeMode::Light,
                "dark" => ThemeMode::Dark,
                _ => ThemeMode::Auto,
            },
            ThemeMode::Light
        );
        assert_eq!(
            match "dark" {
                "auto" => ThemeMode::Auto,
                "light" => ThemeMode::Light,
                "dark" => ThemeMode::Dark,
                _ => ThemeMode::Auto,
            },
            ThemeMode::Dark
        );
    }
}
