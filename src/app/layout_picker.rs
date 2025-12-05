use anyhow::Result;
use crossterm::event::{self, Event};
use std::time::Duration;

use crate::{config, parser, services::geometry, tui};

use super::onboarding;

/// Runs the layout picker to choose between creating new or loading existing layouts
pub fn run_layout_picker_terminal(config: &config::Config) -> Result<()> {
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
                            onboarding::run_new_layout_wizard_terminal(config)?;
                            return Ok(());
                        }
                        tui::layout_picker::PickerAction::LoadLayout(path) => {
                            println!("Loading layout: {}", path.display());
                            println!();

                            // Load the selected layout
                            let layout = parser::parse_markdown_layout(&path)?;

                            // Get layout variant from layout metadata
                            let layout_variant =
                                layout.metadata.layout_variant.as_ref().ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "Layout variant not specified in layout metadata"
                                    )
                                })?;

                            // Build geometry using the centralized geometry service
                            let geo_context = geometry::GeometryContext {
                                config,
                                metadata: &layout.metadata,
                            };

                            let geo_result = geometry::build_geometry_for_layout(geo_context, layout_variant)?;
                            let geometry = geo_result.geometry;
                            let mapping = geo_result.mapping;

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
