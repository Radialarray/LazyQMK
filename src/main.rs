//! Keyboard TUI - Terminal-based keyboard layout editor
//!
//! This application provides a visual editor for mechanical keyboard layouts,
//! allowing users to design layouts, assign keycodes, and generate QMK firmware.

// Module declarations
mod config;
mod firmware;
mod keycode_db;
mod models;
mod parser;
mod tui;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Keyboard TUI - Terminal-based keyboard layout editor
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

    println!("Keyboard TUI v{}", env!("CARGO_PKG_VERSION"));
    println!("Terminal-based keyboard layout editor");
    println!();

    if cli.init {
        println!("Configuration setup wizard will be implemented in Phase 3");
        return Ok(());
    }

    if let Some(path) = cli.layout_path {
        // Load the layout
        let layout = parser::parse_markdown_layout(&path)?;

        // For now, create minimal geometry (will be loaded from QMK in future)
        let geometry = models::KeyboardGeometry {
            keyboard_name: "test".to_string(),
            layout_name: "test".to_string(),
            matrix_rows: 4,
            matrix_cols: 2,
            keys: vec![],
        };

        // Create minimal mapping (will be built from geometry in future)
        let mapping = models::VisualLayoutMapping::new();

        // Load or create default config
        let config_result = config::Config::load();
        let config = config_result.unwrap_or_else(|_| config::Config::default());

        // Initialize TUI
        let mut terminal = tui::setup_terminal()?;
        let mut app_state = tui::AppState::new(layout, Some(path), geometry, mapping, config)?;

        // Run main TUI loop
        let result = tui::run_tui(&mut app_state, &mut terminal);

        // Restore terminal
        tui::restore_terminal(terminal)?;

        // Check for errors
        result?;
    } else {
        println!("Usage: keyboard_tui [FILE]");
        println!("  or:  keyboard_tui --init");
        println!();
        println!("Run with --help for more options");
    }

    Ok(())
}
