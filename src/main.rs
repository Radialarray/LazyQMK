//! Keyboard Configurator - Terminal-based keyboard layout editor
//!
//! This application provides a visual editor for mechanical keyboard layouts,
//! allowing users to design layouts, assign keycodes, and generate QMK firmware.

// Module declarations
mod app;
mod config;
mod constants;
mod firmware;
mod keycode_db;
mod models;
mod parser;
mod services;
mod shortcuts;
mod tui;

use anyhow::Result;
use clap::Parser;
use constants::{APP_BINARY_NAME, APP_NAME};
use std::path::PathBuf;

/// Keyboard Configurator - Terminal-based keyboard layout editor
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to layout markdown file
    #[arg(value_name = "FILE")]
    layout_path: Option<PathBuf>,

    /// Initialize configuration (run setup wizard)
    #[arg(short, long)]
    init: bool,

    /// Specify QMK firmware path
    #[arg(long, value_name = "PATH")]
    qmk_path: Option<PathBuf>,
}



fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("{} v{}", APP_NAME, env!("CARGO_PKG_VERSION"));
    println!("Terminal-based keyboard layout editor");
    println!();

    if cli.init {
        // Run onboarding wizard
        app::run_onboarding_wizard_terminal()?;
        return Ok(());
    }

    if let Some(path) = cli.layout_path {
        // Validate the file path before attempting to load
        if !path.exists() {
            eprintln!("Error: Layout file not found: {}", path.display());
            eprintln!();
            eprintln!("Please provide a valid path to a Markdown layout file.");
            eprintln!();
            eprintln!("Examples:");
            eprintln!("  {} my_layout.md", APP_BINARY_NAME);
            eprintln!("  {} path/to/layout.md", APP_BINARY_NAME);
            eprintln!();
            eprintln!("To set up the application for the first time, run:");
            eprintln!("  {} --init", APP_BINARY_NAME);
            eprintln!();
            eprintln!("For more options, run:");
            eprintln!("  {} --help", APP_BINARY_NAME);
            std::process::exit(1);
        }

        // Check if the file has a reasonable extension
        if let Some(ext) = path.extension() {
            if ext != "md" && ext != "markdown" {
                eprintln!(
                    "Warning: Expected a Markdown file (.md), but got: {}",
                    path.display()
                );
                eprintln!();
            }
        }

        // Load the layout
        let layout = parser::parse_markdown_layout(&path)?;

        // Load or create default config
        let config_result = config::Config::load();
        let config = config_result.unwrap_or_else(|_| config::Config::default());

        // Try to build proper geometry from QMK if config is available
        let (geometry, mapping) = if config.paths.qmk_firmware.is_some() {
            // Get layout variant from metadata
            let layout_variant = layout.metadata.layout_variant.as_ref()
                .ok_or_else(|| anyhow::anyhow!("Layout variant not specified in layout metadata - layout may be from an older version"))?;
            
            // Try to build geometry using the service
            let geo_context = services::geometry::GeometryContext {
                config: &config,
                metadata: &layout.metadata,
            };
            
            match services::geometry::build_geometry_for_layout(geo_context, layout_variant) {
                Ok(geo_result) => (geo_result.geometry, geo_result.mapping),
                Err(_) => {
                    // Fall back to minimal geometry on error
                    let geo_result = services::geometry::build_minimal_geometry();
                    (geo_result.geometry, geo_result.mapping)
                }
            }
        } else {
            // No QMK path configured, use minimal geometry
            let geo_result = services::geometry::build_minimal_geometry();
            (geo_result.geometry, geo_result.mapping)
        };

        // Initialize TUI
        let mut terminal = tui::setup_terminal()?;
        let mut app_state = tui::AppState::new(layout, Some(path), geometry, mapping, config)?;

        // Adjust layers to match geometry (ensures keys match visual positions)
        app_state.adjust_layers_to_geometry()?;

        // Run main TUI loop
        let result = tui::run_tui(&mut app_state, &mut terminal);

        // Restore terminal
        tui::restore_terminal(terminal)?;

        // Check for errors
        result?;
    } else {
        // No file argument provided - check if config exists and is properly configured
        if !config::Config::exists() {
            // No config file exists - automatically run the onboarding wizard
            println!(
                "Welcome! It looks like this is your first time running {}.",
                APP_NAME
            );
            println!();
            println!("Starting the setup wizard...");
            println!();
            app::run_onboarding_wizard_terminal()?;
        } else {
            // Config file exists - try to load it
            match config::Config::load() {
                Ok(config) if config.is_configured() => {
                    // Config exists and is properly configured - show layout picker
                    println!("No layout file specified.");
                    println!();
                    app::run_layout_picker_terminal(&config)?;
                }
                Ok(_) => {
                    // Config exists but is not properly configured (missing QMK path)
                    println!("Configuration is incomplete. Starting the setup wizard...");
                    println!();
                    app::run_onboarding_wizard_terminal()?;
                }
                Err(e) => {
                    // Config file exists but failed to load (corrupted, etc.)
                    eprintln!("Warning: Failed to load config: {e}");
                    eprintln!();
                    println!("Starting the setup wizard to create a new configuration...");
                    println!();
                    app::run_onboarding_wizard_terminal()?;
                }
            }
        }
    }

    Ok(())
}
