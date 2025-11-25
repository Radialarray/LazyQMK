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

/// Runs the onboarding wizard, saves config, and launches the editor
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
                        
                        // Restore terminal before continuing
                        tui::restore_terminal(terminal)?;
                        
                        println!("Configuration saved successfully!");
                        println!();
                        println!("Generating default layout and launching editor...");
                        println!();
                        
                        // Launch the editor with default layout
                        launch_editor_with_default_layout(&config)?;
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

/// Creates a default layout from QMK keyboard info and launches the editor
fn launch_editor_with_default_layout(config: &config::Config) -> Result<()> {
    // Get QMK path from config
    let qmk_path = config.paths.qmk_firmware.as_ref()
        .ok_or_else(|| anyhow::anyhow!("QMK firmware path not set in configuration"))?;
    
    // Parse keyboard info.json
    let keyboard_info = parser::keyboard_json::parse_keyboard_info_json(
        qmk_path,
        &config.build.keyboard
    )?;
    
    // Build geometry from the selected layout
    let geometry = parser::keyboard_json::build_keyboard_geometry(
        &keyboard_info,
        &config.build.keyboard,
        &config.build.layout
    )?;
    
    // Build visual mapping
    let mapping = models::VisualLayoutMapping::build(&geometry);
    
    // Create a default layout with empty keys
    let layout_name = format!("{} Layout", config.build.keyboard);
    let mut layout = models::Layout::new(&layout_name)?;
    
    // Add a default base layer with KC_TRNS for all positions
    let base_layer = create_default_layer(0, "Base", &geometry)?;
    layout.add_layer(base_layer)?;
    
    // Suggest a save path in the config directory
    let layouts_dir = config::Config::config_dir()?.join("layouts");
    std::fs::create_dir_all(&layouts_dir)?;
    
    let suggested_filename = format!("{}_{}.md", 
        config.build.keyboard.replace('/', "_"),
        chrono::Local::now().format("%Y%m%d")
    );
    let suggested_path = layouts_dir.join(suggested_filename);
    
    println!("Opening editor with default layout...");
    println!("Suggested save location: {}", suggested_path.display());
    println!();
    
    // Initialize TUI with the generated layout
    let mut terminal = tui::setup_terminal()?;
    let mut app_state = tui::AppState::new(
        layout,
        Some(suggested_path),
        geometry,
        mapping,
        config.clone()
    )?;
    
    // Mark as dirty since it's a new unsaved layout
    app_state.dirty = true;
    
    // Run main TUI loop
    let result = tui::run_tui(&mut app_state, &mut terminal);
    
    // Restore terminal
    tui::restore_terminal(terminal)?;
    
    // Check for errors
    result?;
    
    Ok(())
}

/// Creates a default layer with KC_TRNS for all key positions
fn create_default_layer(number: u8, name: &str, geometry: &models::KeyboardGeometry) -> Result<models::Layer> {
    use models::layer::{KeyDefinition, Position};
    
    let mut layer = models::Layer::new(
        number,
        name.to_string(),
        models::RgbColor::new(128, 128, 128) // Default gray color
    )?;
    
    // Add KC_TRNS for each key position in the geometry
    for key_geo in &geometry.keys {
        let (matrix_row, matrix_col) = key_geo.matrix_position;
        let position = Position::new(matrix_row, matrix_col);
        let key = KeyDefinition::new(position, "KC_TRNS".to_string());
        layer.add_key(key);
    }
    
    Ok(layer)
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
            Ok(config) => {
                // Config exists - launch editor with default layout
                println!("No layout file specified.");
                println!("Generating default layout from configured keyboard...");
                println!();
                launch_editor_with_default_layout(&config)?;
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
