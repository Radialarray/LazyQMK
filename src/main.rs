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
        // Re-detect OS theme on each loop iteration to respond to system theme changes
        let theme = tui::Theme::detect();

        terminal.draw(|f| {
            tui::onboarding_wizard::render(f, &wizard_state, &theme);
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

                        // Get the layout name from wizard inputs
                        let layout_name = wizard_state.inputs.get("layout_name")
                            .cloned()
                            .unwrap_or_else(|| {
                                // Fallback to keyboard-based name if not set
                                format!("{}_layout", config.build.keyboard.replace('/', "_"))
                            });

                        // Restore terminal before continuing
                        tui::restore_terminal(terminal)?;

                        println!("Configuration saved successfully!");
                        println!();
                        println!("Generating default layout and launching editor...");
                        println!();

                        // Launch the editor with default layout using the user-specified name
                        launch_editor_with_default_layout(&config, &layout_name)?;
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
fn launch_editor_with_default_layout(config: &config::Config, layout_file_name: &str) -> Result<()> {
    // Get QMK path from config
    let qmk_path = config
        .paths
        .qmk_firmware
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("QMK firmware path not set in configuration"))?;

    // Get the base keyboard name (without any variant subdirectory)
    // This mirrors the logic in AppState::rebuild_geometry()
    let base_keyboard = tui::AppState::extract_base_keyboard(&config.build.keyboard);

    // Parse keyboard info.json using the base keyboard path
    let keyboard_info =
        parser::keyboard_json::parse_keyboard_info_json(qmk_path, &base_keyboard)?;

    // Get the key count for the selected layout to determine the correct variant
    let layout_def = keyboard_info.layouts.get(&config.build.layout)
        .ok_or_else(|| anyhow::anyhow!("Layout '{}' not found in keyboard info.json", config.build.layout))?;
    let key_count = layout_def.layout.len();

    // Determine the correct keyboard variant based on key count
    // This is critical for split keyboards where different variants have different key counts
    let variant_path = config.build.determine_keyboard_variant(qmk_path, &base_keyboard, key_count)
        .unwrap_or_else(|_| base_keyboard.clone());

    // Try to get RGB matrix mapping from the variant's keyboard.json
    let matrix_to_led = parser::keyboard_json::parse_variant_keyboard_json(qmk_path, &variant_path)
        .and_then(|variant| variant.rgb_matrix)
        .map(|rgb_config| parser::keyboard_json::build_matrix_to_led_map(&rgb_config));

    // Build geometry from the selected layout with RGB matrix mapping if available
    let geometry = parser::keyboard_json::build_keyboard_geometry_with_rgb(
        &keyboard_info,
        &base_keyboard,
        &config.build.layout,
        matrix_to_led.as_ref(),
    )?;

    // Build visual mapping
    let mapping = models::VisualLayoutMapping::build(&geometry);

    // Create a default layout with the user-specified name
    let mut layout = models::Layout::new(layout_file_name)?;

    // Store the layout variant in metadata for future reference
    layout.metadata.layout_variant = Some(config.build.layout.clone());

    // Add a default base layer with KC_TRNS for all positions
    let base_layer = create_default_layer(0, "Base", &mapping)?;
    layout.add_layer(base_layer)?;

    // Create save path using the user-specified layout name
    let layouts_dir = config::Config::config_dir()?.join("layouts");
    std::fs::create_dir_all(&layouts_dir)?;

    // Sanitize the filename: remove/replace problematic characters
    let sanitized_name = layout_file_name
        .replace('/', "_")
        .replace('\\', "_")
        .replace(':', "_")
        .replace(' ', "_");
    
    let layout_path = layouts_dir.join(format!("{}.md", sanitized_name));

    // Save the layout immediately so it can be found on restart
    parser::save_markdown_layout(&layout, &layout_path)?;

    println!("Layout saved to: {}", layout_path.display());
    println!();

    // Update config with the determined variant path before creating AppState
    let mut updated_config = config.clone();
    updated_config.build.keyboard = variant_path;

    // Initialize TUI with the generated layout
    let mut terminal = tui::setup_terminal()?;
    let mut app_state = tui::AppState::new(
        layout,
        Some(layout_path),
        geometry,
        mapping,
        updated_config,
    )?;

    // Layout is clean since we just saved it
    app_state.dirty = false;

    // Run main TUI loop
    let result = tui::run_tui(&mut app_state, &mut terminal);

    // Restore terminal
    tui::restore_terminal(terminal)?;

    // Check for errors
    result?;

    Ok(())
}

/// Creates a default layer with KC_TRNS for all key positions
fn create_default_layer(
    number: u8,
    name: &str,
    mapping: &models::VisualLayoutMapping,
) -> Result<models::Layer> {
    use models::layer::KeyDefinition;
    use models::ColorPalette;

    // Use the color palette's default layer color (Gray-500)
    let palette = ColorPalette::load().unwrap_or_default();
    let default_color = palette.default_layer_color();

    let mut layer = models::Layer::new(
        number,
        name.to_string(),
        default_color,
    )?;

    // Add KC_TRNS for each visual position in the mapping
    // This ensures keys use visual positions (not matrix positions) for proper rendering
    for pos in mapping.get_all_visual_positions() {
        let key = KeyDefinition::new(pos, "KC_TRNS".to_string());
        layer.add_key(key);
    }

    Ok(layer)
}

/// Runs the layout picker to choose between creating new or loading existing layouts
fn run_layout_picker(config: &config::Config) -> Result<()> {
    use crossterm::event::{self, Event};
    use std::time::Duration;

    // Initialize terminal
    let mut terminal = tui::setup_terminal()?;
    let mut picker_state = tui::layout_picker::LayoutPickerState::new();

    // Scan for saved layouts
    picker_state.scan_layouts()?;

    // Run picker loop
    loop {
        // Re-detect OS theme on each loop iteration to respond to system theme changes
        let theme = tui::Theme::detect();

        terminal.draw(|f| {
            tui::layout_picker::render(f, &picker_state, &theme);
        })?;

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if let Some(action) = tui::layout_picker::handle_input(&mut picker_state, key)? {
                    // Restore terminal before proceeding
                    tui::restore_terminal(terminal)?;

                    match action {
                        tui::layout_picker::PickerAction::CreateNew => {
                            // Run wizard to let user select keyboard, layout, and name
                            run_new_layout_wizard(config)?;
                            return Ok(());
                        }
                        tui::layout_picker::PickerAction::LoadLayout(path) => {
                            println!("Loading layout: {}", path.display());
                            println!();

                            // Load the selected layout
                            let layout = parser::parse_markdown_layout(&path)?;

                            // Use saved layout variant if available, otherwise fall back to config
                            let layout_variant = layout.metadata.layout_variant.as_ref()
                                .unwrap_or(&config.build.layout);

                            // Parse keyboard info.json to rebuild geometry
                            let qmk_path = config.paths.qmk_firmware.as_ref().ok_or_else(|| {
                                anyhow::anyhow!("QMK firmware path not set in configuration")
                            })?;

                            // Extract base keyboard name (without variant subdirectory)
                            let base_keyboard = tui::AppState::extract_base_keyboard(&config.build.keyboard);

                            let keyboard_info = parser::keyboard_json::parse_keyboard_info_json(
                                qmk_path,
                                &base_keyboard,
                            )?;

                            // Get key count from layout to determine correct variant
                            let layout_def = keyboard_info.layouts.get(layout_variant)
                                .ok_or_else(|| anyhow::anyhow!("Layout '{}' not found in keyboard info.json", layout_variant))?;
                            let key_count = layout_def.layout.len();

                            // Determine keyboard variant based on key count
                            let variant_path = config.build.determine_keyboard_variant(qmk_path, &base_keyboard, key_count)
                                .unwrap_or_else(|_| base_keyboard.clone());

                            // Try to get RGB matrix mapping from variant keyboard.json
                            let matrix_to_led = parser::keyboard_json::parse_variant_keyboard_json(qmk_path, &variant_path)
                                .and_then(|variant| variant.rgb_matrix)
                                .map(|rgb_config| parser::keyboard_json::build_matrix_to_led_map(&rgb_config));

                            let geometry = parser::keyboard_json::build_keyboard_geometry_with_rgb(
                                &keyboard_info,
                                &base_keyboard,
                                layout_variant,
                                matrix_to_led.as_ref(),
                            )?;

                            let mapping = models::VisualLayoutMapping::build(&geometry);

                            // Re-initialize terminal for editor
                            let mut terminal = tui::setup_terminal()?;
                            let mut app_state = tui::AppState::new(
                                layout,
                                Some(path),
                                geometry,
                                mapping,
                                config.clone(),
                            )?;

                            // Adjust layers to match geometry (ensures keys match visual positions)
                            app_state.adjust_layers_to_geometry()?;

                            // Run main TUI loop
                            let result = tui::run_tui(&mut app_state, &mut terminal);

                            // Restore terminal
                            tui::restore_terminal(terminal)?;

                            // Check for errors
                            result?;
                            return Ok(());
                        }
                        tui::layout_picker::PickerAction::Cancel => {
                            println!("Layout selection cancelled.");
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}

/// Runs a wizard for creating a new layout (skips QMK path since config exists)
fn run_new_layout_wizard(config: &config::Config) -> Result<()> {
    use crossterm::event::{self, Event};
    use std::time::Duration;

    // Initialize terminal
    let mut terminal = tui::setup_terminal()?;
    let mut wizard_state = tui::onboarding_wizard::OnboardingWizardState::new_for_new_layout(config)?;

    // Run wizard loop
    loop {
        // Re-detect OS theme on each loop iteration to respond to system theme changes
        let theme = tui::Theme::detect();

        terminal.draw(|f| {
            tui::onboarding_wizard::render(f, &wizard_state, &theme);
        })?;

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let should_exit = tui::onboarding_wizard::handle_input(&mut wizard_state, key)?;

                if should_exit {
                    if wizard_state.is_complete {
                        // Get values from wizard
                        let keyboard = wizard_state.inputs.get("keyboard")
                            .cloned()
                            .unwrap_or_else(|| config.build.keyboard.clone());
                        let layout_variant = wizard_state.inputs.get("layout")
                            .cloned()
                            .unwrap_or_else(|| config.build.layout.clone());
                        let layout_name = wizard_state.inputs.get("layout_name")
                            .cloned()
                            .unwrap_or_else(|| format!("{}_layout", keyboard.replace('/', "_")));

                        // Update config with new keyboard/layout if changed
                        let mut updated_config = config.clone();
                        updated_config.build.keyboard = keyboard;
                        updated_config.build.layout = layout_variant;
                        
                        // Save updated config
                        if let Err(e) = updated_config.save() {
                            eprintln!("Warning: Failed to save config: {}", e);
                        }

                        // Restore terminal before continuing
                        tui::restore_terminal(terminal)?;

                        println!("Creating new layout...");
                        println!();

                        // Launch the editor with the new layout
                        launch_editor_with_default_layout(&updated_config, &layout_name)?;
                        return Ok(());
                    } else {
                        // User cancelled - return to layout picker
                        tui::restore_terminal(terminal)?;
                        println!("Layout creation cancelled.");
                        return Ok(());
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
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
        let (geometry, mapping) = if let Some(qmk_path) = config.paths.qmk_firmware.as_ref() {
            // Use saved layout variant if available, otherwise fall back to config
            let layout_variant = layout.metadata.layout_variant.as_ref()
                .unwrap_or(&config.build.layout);

            // Extract base keyboard name (without variant subdirectory)
            let base_keyboard = tui::AppState::extract_base_keyboard(&config.build.keyboard);

            match parser::keyboard_json::parse_keyboard_info_json(qmk_path, &base_keyboard) {
                Ok(keyboard_info) => {
                    // Get key count from layout to determine correct variant
                    let variant_path = if let Some(layout_def) = keyboard_info.layouts.get(layout_variant) {
                        let key_count = layout_def.layout.len();
                        config.build.determine_keyboard_variant(qmk_path, &base_keyboard, key_count)
                            .unwrap_or_else(|_| base_keyboard.clone())
                    } else {
                        base_keyboard.clone()
                    };

                    // Try to get RGB matrix mapping from variant keyboard.json
                    let matrix_to_led = parser::keyboard_json::parse_variant_keyboard_json(qmk_path, &variant_path)
                        .and_then(|variant| variant.rgb_matrix)
                        .map(|rgb_config| parser::keyboard_json::build_matrix_to_led_map(&rgb_config));

                    match parser::keyboard_json::build_keyboard_geometry_with_rgb(
                        &keyboard_info,
                        &base_keyboard,
                        layout_variant,
                        matrix_to_led.as_ref(),
                    ) {
                        Ok(geometry) => {
                            let mapping = models::VisualLayoutMapping::build(&geometry);
                            (geometry, mapping)
                        }
                        Err(_) => {
                            // Fall back to minimal geometry
                            (models::KeyboardGeometry {
                                keyboard_name: "test".to_string(),
                                layout_name: "test".to_string(),
                                matrix_rows: 4,
                                matrix_cols: 2,
                                keys: vec![],
                            }, models::VisualLayoutMapping::new())
                        }
                    }
                }
                Err(_) => {
                    // Fall back to minimal geometry
                    (models::KeyboardGeometry {
                        keyboard_name: "test".to_string(),
                        layout_name: "test".to_string(),
                        matrix_rows: 4,
                        matrix_cols: 2,
                        keys: vec![],
                    }, models::VisualLayoutMapping::new())
                }
            }
        } else {
            // No QMK path configured, use minimal geometry
            (models::KeyboardGeometry {
                keyboard_name: "test".to_string(),
                layout_name: "test".to_string(),
                matrix_rows: 4,
                matrix_cols: 2,
                keys: vec![],
            }, models::VisualLayoutMapping::new())
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
        // No file argument provided - check if config exists
        match config::Config::load() {
            Ok(config) => {
                // Config exists - show layout picker
                println!("No layout file specified.");
                println!();
                run_layout_picker(&config)?;
            }
            Err(_) => {
                // No config exists or config invalid - suggest running wizard
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
