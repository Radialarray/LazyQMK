use anyhow::Result;
use crossterm::event::{self, Event};
use std::time::Duration;

use crate::tui::component::Component;
use crate::tui::layout_picker::{LayoutPicker, LayoutPickerEvent};
use crate::{config, services, tui};

use super::onboarding;

/// Runs the layout picker to choose between creating new or loading existing layouts
pub fn run_layout_picker_terminal(config: &config::Config) -> Result<()> {
    // Initialize terminal
    let mut terminal = tui::setup_terminal()?;

    // Create component-based layout picker
    let mut picker = LayoutPicker::new();

    // Run picker loop
    loop {
        // Re-detect OS theme on each loop iteration to respond to system theme changes
        let theme = tui::Theme::detect();

        terminal.draw(|f| {
            picker.render(f, f.area(), &theme);
        })?;

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if let Some(event) = picker.handle_input(key) {
                    // Restore terminal before proceeding
                    tui::restore_terminal(terminal)?;

                    match event {
                        LayoutPickerEvent::CreateNew => {
                            // Run wizard to let user select keyboard, layout, and name
                            onboarding::run_new_layout_wizard_terminal(config)?;
                            return Ok(());
                        }
                        LayoutPickerEvent::LayoutSelected(path) => {
                            println!("Loading layout: {}", path.display());
                            println!();

                            // Load the selected layout
                            let layout = services::LayoutService::load(&path)?;

                            // Get layout variant from layout metadata
                            let layout_variant =
                                layout.metadata.layout_variant.as_ref().ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "Layout variant not specified in layout metadata"
                                    )
                                })?;

                            // Build geometry using the centralized geometry service
                            let geo_context = services::geometry::GeometryContext {
                                config,
                                metadata: &layout.metadata,
                            };

                            let geo_result = services::geometry::build_geometry_for_layout(
                                geo_context,
                                layout_variant,
                            )?;
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

                            // Start watching the on-disk layout for external
                            // changes so the hot-reload flow can pick up edits
                            // from a background agent or another tool.
                            app_state.start_file_watcher();

                            // Run main TUI loop
                            let result = tui::run_tui(&mut app_state, &mut terminal);

                            // Restore terminal
                            tui::restore_terminal(terminal)?;

                            // Check for errors
                            result?;
                            return Ok(());
                        }
                        LayoutPickerEvent::Cancelled => {
                            println!("Layout selection cancelled.");
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}
