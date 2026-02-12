//! LazyQMK - Terminal-based keyboard layout editor
//!
//! This application provides a visual editor for mechanical keyboard layouts,
//! allowing users to design layouts, assign keycodes, and generate QMK firmware.

// Allow intentional type casts for terminal coordinates and QMK data structures
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_wrap)]

// Module declarations
mod app;
mod cli;
mod constants;
mod doctor;
mod export;
mod firmware;
mod keycode_db;
mod models;
mod parser;
mod services;
mod shortcuts;
mod tui;

// Import from library to avoid module conflicts and duplication
use lazyqmk::{branding, config};

use anyhow::Result;
use clap::{Parser, Subcommand};
use constants::{APP_BINARY_NAME, APP_DESCRIPTION, APP_NAME};
use std::path::PathBuf;

/// LazyQMK - Keyboard layout editor for QMK firmware
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "\
LazyQMK - Keyboard layout editor for QMK firmware

INTERFACES:
  LazyQMK provides two interfaces in one binary:
  
  1. Terminal UI (TUI) - Default when no command specified
     - Launch: lazyqmk [layout.md]
     - Keyboard-driven interface in your terminal
  
  2. Web Editor - Browser-based interface
     - Launch: lazyqmk web [--port 3001] [--host 127.0.0.1]
     - Modern GUI with mouse support
     - Access at http://localhost:3001

QUICK START:
  # Launch TUI with layout picker
  lazyqmk
  
  # Open specific layout in TUI
  lazyqmk my-layout.md
  
  # Start web server (default: http://localhost:3001)
  lazyqmk web
  
  # Start web server on custom port
  lazyqmk web --port 8080
  
  # Run setup wizard
  lazyqmk --init

FEATURES:
  - Visual keyboard layout editor
  - Layer management and navigation  
  - Color coding system with categories
  - QMK firmware generation and compilation
  - Tap dance configuration
  - Template system for layout sharing
  - Support for 500+ QMK keyboards
  
  Both interfaces provide identical functionality - choose based on your preference!

DOCUMENTATION:
  - TUI features: docs/FEATURES.md
  - Web features: docs/WEB_FEATURES.md
  - Architecture: docs/ARCHITECTURE.md
  - GitHub: https://github.com/Radialarray/LazyQMK
"
)]
struct Cli {
    /// Subcommand to execute (if none provided, launches TUI)
    #[command(subcommand)]
    command: Option<Command>,

    /// Path to layout markdown file (TUI mode only)
    #[arg(value_name = "FILE")]
    layout_path: Option<PathBuf>,

    /// Initialize configuration (run setup wizard)
    #[arg(short, long)]
    init: bool,

    /// Specify QMK firmware path
    #[arg(long, value_name = "PATH")]
    qmk_path: Option<PathBuf>,
}

/// Web server arguments
#[cfg(feature = "web")]
#[derive(clap::Args, Debug)]
struct WebArgs {
    /// Port to listen on
    #[arg(short, long, default_value = "3001")]
    port: u16,

    /// Host to bind to (default: localhost only for security)
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Workspace directory containing layout files.
    /// Defaults to platform-specific layouts directory.
    #[arg(short, long)]
    workspace: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Validate a layout file for errors and warnings
    Validate(cli::ValidateArgs),
    /// Generate QMK firmware files (keymap.c, config.h)
    Generate(cli::GenerateArgs),
    /// Export keyboard layout to markdown documentation
    Export(cli::ExportArgs),
    /// Display help topics and keybindings
    #[command(name = "show-help")]
    ShowHelp(cli::HelpArgs),
    /// Inspect specific sections of a layout file
    Inspect(cli::InspectArgs),
    /// Resolve layer UUID references in keycodes
    Keycode(cli::KeycodeArgs),
    /// List available keycodes from the embedded keycode database
    Keycodes(cli::KeycodesArgs),
    /// Manage tap dance definitions
    #[command(name = "tap-dance")]
    TapDance(cli::TapDanceArgs),
    /// Show layer references and transparency warnings
    #[command(name = "layer-refs")]
    LayerRefs(cli::LayerRefsArgs),
    /// List all compilable keyboards in QMK firmware directory
    #[command(name = "list-keyboards")]
    ListKeyboards(cli::ListKeyboardsArgs),
    /// List layout variants for a specific keyboard
    #[command(name = "list-layouts")]
    ListLayouts(cli::ListLayoutsArgs),
    /// Display matrix, LED, and visual coordinate mappings
    Geometry(cli::GeometryArgs),
    /// Manage application configuration
    Config(cli::ConfigArgs),
    /// Manage categories in a layout
    Category(cli::CategoryArgs),
    /// Manage layout templates
    Template(cli::TemplateArgs),
    /// Check development environment dependencies
    Doctor(cli::DoctorArgs),
    /// Start web server for browser-based editor
    #[cfg(feature = "web")]
    Web(WebArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle CLI subcommands first (headless mode)
    if let Some(command) = cli.command {
        use cli::ExitCode;

        let exit_code = match command {
            Command::Validate(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::Generate(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::Export(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::ShowHelp(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::Inspect(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::Keycode(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::Keycodes(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::TapDance(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::LayerRefs(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::ListKeyboards(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::ListLayouts(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::Geometry(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::Config(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::Category(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::Template(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            Command::Doctor(args) => match args.execute() {
                Ok(()) => ExitCode::Success,
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    e.exit_code
                }
            },
            #[cfg(feature = "web")]
            Command::Web(args) => {
                // Web command uses async runtime, handle it differently
                return run_web_server(args);
            }
        };

        std::process::exit(exit_code as i32);
    }

    // TUI mode - print branding
    println!("{} v{}", APP_NAME, env!("CARGO_PKG_VERSION"));
    println!("{}", APP_DESCRIPTION);
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
        let layout = services::LayoutService::load(&path)?;

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

/// Run the web server (available when built with 'web' feature)
#[cfg(feature = "web")]
fn run_web_server(args: WebArgs) -> Result<()> {
    use anyhow::Context;
    use std::net::SocketAddr;

    // Initialize tracing for web server logs
    {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

        let filter = if args.verbose { "debug" } else { "info" };
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| filter.into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    // Load or create configuration
    let cfg = config::Config::load().unwrap_or_default();

    // Determine workspace root
    let workspace_root = match args.workspace {
        Some(path) => path,
        None => {
            let layouts_dir = config::Config::config_dir()?.join("layouts");
            if !layouts_dir.exists() {
                std::fs::create_dir_all(&layouts_dir).context(format!(
                    "Failed to create layouts directory: {}",
                    layouts_dir.display()
                ))?;
            }
            layouts_dir
        }
    };

    println!("{} v{} - Web Server", APP_NAME, env!("CARGO_PKG_VERSION"));
    println!("Workspace: {}", workspace_root.display());
    println!("Starting server on http://{}:{}", args.host, args.port);
    println!();
    println!("Press Ctrl+C to stop the server");
    println!();

    // Build socket address
    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .context("Failed to parse socket address")?;

    // Use tokio runtime to run the async web server
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async move {
        // Import web module from library
        lazyqmk::web::run_server(cfg, workspace_root, addr).await
    })
}
