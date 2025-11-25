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

/// Runs the onboarding wizard and saves the configuration
fn run_onboarding_wizard() -> Result<()> {
    use crossterm::event::{self, Event};
    use std::time::Duration;

    // Initialize terminal
    let mut terminal = tui::setup_terminal()?;
    let mut wizard_state = tui::onboarding_wizard::OnboardingWizardState::new();

    // Run wizard loop
    loop {
        terminal.draw(|f| {
            tui::onboarding_wizard::render(f, &wizard_state);
        })?;

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let should_exit = tui::onboarding_wizard::handle_input(&mut wizard_state, key)?;

                if should_exit {
                    if wizard_state.is_complete {
                        // Build and save configuration
                        let config = wizard_state.build_config()?;
                        config.save()?;
                        
                        // Restore terminal before showing success message
                        tui::restore_terminal(terminal)?;
                        
                        println!("Configuration saved successfully!");
                        println!();
                        println!("Configuration file: {:?}", config::Config::config_file_path()?);
                        println!();
                        println!("You can now:");
                        println!("  • Create a new layout: keyboard_tui my_layout.md");
                        println!("  • Load an existing layout: keyboard_tui path/to/layout.md");
                        println!();
                        return Ok(());
                    } else {
                        // User cancelled
                        tui::restore_terminal(terminal)?;
                        println!("Setup cancelled.");
                        return Ok(());
                    }
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("Keyboard TUI v{}", env!("CARGO_PKG_VERSION"));
    println!("Terminal-based keyboard layout editor");
    println!();

    if cli.init {
        // Run onboarding wizard
        run_onboarding_wizard()?;
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
            eprintln!("  keyboard_tui my_layout.md");
            eprintln!("  keyboard_tui path/to/layout.md");
            eprintln!();
            eprintln!("To set up the application for the first time, run:");
            eprintln!("  keyboard_tui --init");
            eprintln!();
            eprintln!("For more options, run:");
            eprintln!("  keyboard_tui --help");
            std::process::exit(1);
        }

        // Check if the file has a reasonable extension
        if let Some(ext) = path.extension() {
            if ext != "md" && ext != "markdown" {
                eprintln!("Warning: Expected a Markdown file (.md), but got: {}", path.display());
                eprintln!();
            }
        }

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
        // No file argument provided - check if config exists
        match config::Config::load() {
            Ok(_config) => {
                // Config exists, show helpful usage message
                println!("No layout file specified.");
                println!();
                println!("Usage: keyboard_tui [FILE]");
                println!();
                println!("Examples:");
                println!("  keyboard_tui my_layout.md         # Load a layout file");
                println!("  keyboard_tui test_layout.md       # Try the example file");
                println!();
                println!("To reconfigure the application, run:");
                println!("  keyboard_tui --init");
                println!();
                println!("For more options, run:");
                println!("  keyboard_tui --help");
            }
            Err(_) => {
                // No config exists - suggest running wizard
                println!("Welcome! It looks like this is your first time running Keyboard TUI.");
                println!();
                println!("To get started, you need to configure the application.");
                println!("Run the setup wizard:");
                println!();
                println!("  keyboard_tui --init");
                println!();
                println!("Or, if you already have a layout file:");
                println!();
                println!("  keyboard_tui path/to/layout.md");
                println!();
                println!("For more information, run:");
                println!();
                println!("  keyboard_tui --help");
            }
        }
    }

    Ok(())
}
