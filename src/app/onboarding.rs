//! Onboarding and new layout wizards for initial setup and layout creation

use anyhow::Result;
use crossterm::event::{self, Event};
use std::time::Duration;

use crate::{config, tui};

use super::{launch, layout_picker};

/// Runs the onboarding wizard, saves config, and launches the editor
pub fn run_onboarding_wizard_terminal() -> Result<()> {
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
                    // Check if user chose to load existing layout
                    if wizard_state.welcome_choice
                        == Some(tui::onboarding_wizard::WelcomeChoice::LoadExisting)
                    {
                        // Restore terminal and launch layout picker
                        tui::restore_terminal(terminal)?;

                        // Load config if it exists, otherwise use default
                        let config =
                            config::Config::load().unwrap_or_else(|_| config::Config::new());

                        // Launch the layout picker
                        return layout_picker::run_layout_picker_terminal(&config);
                    } else if wizard_state.is_complete {
                        // Build and save configuration
                        let config = wizard_state.build_config()?;
                        config.save()?;

                        // Get keyboard and layout from wizard inputs
                        let keyboard = wizard_state
                            .inputs
                            .get("keyboard")
                            .cloned()
                            .ok_or_else(|| anyhow::anyhow!("Keyboard not selected"))?;
                        let layout_variant = wizard_state
                            .inputs
                            .get("layout")
                            .cloned()
                            .ok_or_else(|| anyhow::anyhow!("Layout not selected"))?;

                        // Get the layout name from wizard inputs
                        let layout_name = wizard_state
                            .inputs
                            .get("layout_name")
                            .cloned()
                            .unwrap_or_else(|| {
                                // Fallback to keyboard-based name if not set
                                format!("{}_layout", keyboard.replace('/', "_"))
                            });

                        // Restore terminal before continuing
                        tui::restore_terminal(terminal)?;

                        println!("Configuration saved successfully!");
                        println!();
                        println!("Generating default layout and launching editor...");
                        println!();

                        // Launch the editor with default layout using keyboard/layout from wizard
                        launch::launch_editor_with_default_layout(
                            &config,
                            &keyboard,
                            &layout_variant,
                            &layout_name,
                        )?;
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

/// Runs a wizard for creating a new layout (skips QMK path since config exists)
pub fn run_new_layout_wizard_terminal(config: &config::Config) -> Result<()> {
    // Initialize terminal
    let mut terminal = tui::setup_terminal()?;
    let mut wizard_state =
        tui::onboarding_wizard::OnboardingWizardState::new_for_new_layout(config)?;

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
                        let keyboard = wizard_state
                            .inputs
                            .get("keyboard")
                            .cloned()
                            .ok_or_else(|| anyhow::anyhow!("Keyboard not selected"))?;
                        let layout_variant = wizard_state
                            .inputs
                            .get("layout")
                            .cloned()
                            .ok_or_else(|| anyhow::anyhow!("Layout not selected"))?;
                        let layout_name = wizard_state
                            .inputs
                            .get("layout_name")
                            .cloned()
                            .unwrap_or_else(|| format!("{}_layout", keyboard.replace('/', "_")));

                        // Restore terminal before continuing
                        tui::restore_terminal(terminal)?;

                        println!("Creating new layout...");
                        println!();

                        // Launch the editor with the new layout
                        launch::launch_editor_with_default_layout(
                            &config,
                            &keyboard,
                            &layout_variant,
                            &layout_name,
                        )?;
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
