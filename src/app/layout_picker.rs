use anyhow::Result;
use crossterm::event::{self, Event};
use std::time::Duration;

use crate::{config, models, parser, tui};

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

                            // Get keyboard and layout variant from layout metadata
                            let keyboard = layout.metadata.keyboard.as_ref().ok_or_else(|| {
                                anyhow::anyhow!("Keyboard not specified in layout metadata")
                            })?;
                            let layout_variant =
                                layout.metadata.layout_variant.as_ref().ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "Layout variant not specified in layout metadata"
                                    )
                                })?;

                            // Parse keyboard info.json to rebuild geometry
                            let qmk_path = config.paths.qmk_firmware.as_ref().ok_or_else(|| {
                                anyhow::anyhow!("QMK firmware path not set in configuration")
                            })?;

                            // Extract base keyboard name (without variant subdirectory)
                            let base_keyboard = tui::AppState::extract_base_keyboard(keyboard);

                            let keyboard_info = parser::keyboard_json::parse_keyboard_info_json(
                                qmk_path,
                                &base_keyboard,
                            )?;

                            // Get key count from layout to determine correct variant
                            let layout_def =
                                keyboard_info.layouts.get(layout_variant).ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "Layout '{}' not found in keyboard info.json",
                                        layout_variant
                                    )
                                })?;
                            let key_count = layout_def.layout.len();

                            // Determine keyboard variant based on key count
                            let variant_path = config
                                .build
                                .determine_keyboard_variant(qmk_path, &base_keyboard, key_count)
                                .unwrap_or_else(|_| base_keyboard.clone());

                            // Try to get RGB matrix mapping from variant keyboard.json
                            let matrix_to_led = parser::keyboard_json::parse_variant_keyboard_json(
                                qmk_path,
                                &variant_path,
                            )
                            .and_then(|variant| variant.rgb_matrix)
                            .map(|rgb_config| {
                                parser::keyboard_json::build_matrix_to_led_map(&rgb_config)
                            });

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
