//! Top-level rendering functions for the TUI.
//!
//! Orchestrates the title bar, main content, status bar, popups, and error overlay.

mod main_content;
mod title_bar;

use ratatui::{
    layout::{Constraint, Direction, Layout as RatatuiLayout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::app_state::TemplateSaveField;
use crate::tui::app_state::{ActiveComponent, AppState};
use crate::tui::component::{Component, ContextualComponent};
use crate::tui::editor::key_editor;
use crate::tui::onboarding_wizard;
use crate::tui::popup_type::{popup_border_style, popup_title, PopupType};
use crate::tui::settings_manager;
use crate::tui::status_bar::StatusBar;
use crate::tui::theme::Theme;
use main_content::render_main_content;
use title_bar::render_title_bar;

/// Render the UI from current state
pub fn render(f: &mut Frame, state: &AppState) {
    // Fill entire screen with theme background color first
    // This ensures consistent background regardless of terminal settings
    let full_bg = Block::default().style(Style::default().bg(state.theme.background));
    f.render_widget(full_bg, f.area());

    let chunks = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Title bar (border + 2 content lines)
            Constraint::Min(10),   // Main content
            Constraint::Length(6), // Status bar (increased for description + clipboard + build + help)
        ])
        .split(f.area());

    // Title bar with dirty indicator
    render_title_bar(f, chunks[0], state);

    // Main content area
    render_main_content(f, chunks[1], state);

    // Status bar
    StatusBar::render(f, chunks[2], state, &state.theme);

    // Render popup if active
    if let Some(popup_type) = &state.active_popup {
        render_popup(f, popup_type, state);
    }

    // Render error overlay on top of everything if error is present
    if let Some(ref error) = state.error_message {
        render_error_overlay(f, error, &state.theme);
    }
}

/// Render active popup
fn render_popup(f: &mut Frame, popup_type: &PopupType, state: &AppState) {
    match popup_type {
        PopupType::KeycodePicker => {
            // Use ContextualComponent trait pattern
            if let Some(ActiveComponent::KeycodePicker(ref picker)) = state.active_component {
                picker.render(f, f.area(), &state.theme, &state.keycode_db);
            }
        }
        PopupType::ColorPicker => {
            // Use Component trait pattern
            if let Some(ActiveComponent::ColorPicker(ref picker)) = state.active_component {
                picker.render(f, f.area(), &state.theme);
            }
        }
        PopupType::CategoryPicker => {
            // Use ContextualComponent trait pattern
            if let Some(ActiveComponent::CategoryPicker(ref picker)) = state.active_component {
                picker.render(f, f.area(), &state.theme, &state.layout.categories);
            }
        }
        PopupType::CategoryManager => {
            // Use Component trait pattern
            if let Some(ActiveComponent::CategoryManager(ref manager)) = state.active_component {
                manager.render(f, f.area(), &state.theme);
            }
        }
        PopupType::LayerManager => {
            // Use Component trait pattern
            if let Some(ActiveComponent::LayerManager(ref manager)) = state.active_component {
                manager.render(f, f.area(), &state.theme);
            }
        }
        PopupType::LayerPicker => {
            // Use ContextualComponent trait pattern
            if let Some(ActiveComponent::LayerPicker(ref picker)) = state.active_component {
                picker.render(f, f.area(), &state.theme, &state.layout.layers);
            }
        }
        PopupType::TemplateBrowser => {
            if let Some(ActiveComponent::TemplateBrowser(ref browser)) = state.active_component {
                browser.render(f, f.area(), &state.theme);
            }
        }
        PopupType::TemplateSaveDialog => {
            render_template_save_dialog(f, state);
        }
        PopupType::ExportFilenameDialog => {
            render_export_filename_dialog(f, state);
        }
        PopupType::UnsavedChangesPrompt => {
            render_unsaved_prompt(f, &state.theme);
        }
        PopupType::BuildLog => {
            // Use ContextualComponent trait pattern
            if let Some(ActiveComponent::BuildLog(ref log)) = state.active_component {
                if let Some(ref build_state) = state.build_state {
                    log.render(f, f.area(), &state.theme, build_state);
                }
            }
        }
        PopupType::HelpOverlay => {
            if let Some(ActiveComponent::HelpOverlay(ref help)) = state.active_component {
                help.render(f, f.area(), &state.theme);
            }
        }
        PopupType::LayoutPicker => {
            if let Some(ActiveComponent::LayoutVariantPicker(ref picker)) = state.active_component {
                picker.render(f, f.area(), &state.theme);
            }
        }
        PopupType::MetadataEditor => {
            if let Some(ActiveComponent::MetadataEditor(ref editor)) = state.active_component {
                editor.render(f, f.area(), &state.theme);
            }
        }
        PopupType::SetupWizard => {
            onboarding_wizard::render(f, &state.wizard_state, &state.theme);
        }
        PopupType::SettingsManager => {
            if let Some(ActiveComponent::SettingsManager(ref manager)) = state.active_component {
                let context = settings_manager::SettingsManagerContext {
                    rgb_enabled: state.layout.rgb_enabled,
                    rgb_brightness: state.layout.rgb_brightness,
                    rgb_timeout_ms: state.layout.rgb_timeout_ms,
                    uncolored_key_behavior: state.layout.uncolored_key_behavior,
                    idle_effect_settings: state.layout.idle_effect_settings.clone(),
                    overlay_ripple_settings: state.layout.rgb_overlay_ripple.clone(),
                    tap_hold_settings: state.layout.tap_hold_settings.clone(),
                    config: state.config.clone(),
                    layout: state.layout.clone(),
                };
                manager.render_with_context(f, f.area(), &state.theme, &context);
            }
        }
        PopupType::TapKeycodePicker => {
            // Use component-based rendering (same as KeycodePicker)
            if let Some(ActiveComponent::KeycodePicker(ref picker)) = state.active_component {
                picker.render(f, f.area(), &state.theme, &state.keycode_db);
            }
        }
        PopupType::ModifierPicker => {
            if let Some(ActiveComponent::ModifierPicker(ref picker)) = state.active_component {
                picker.render(f, f.area(), &state.theme);
            }
        }
        PopupType::KeyEditor => {
            key_editor::render_key_editor(f, state);
        }
        PopupType::TapDanceEditor => {
            if let Some(ActiveComponent::TapDanceEditor(ref editor)) = state.active_component {
                editor.render(f, f.area(), &state.theme);
            }
        }
        PopupType::TapDanceForm => {
            if let Some(ActiveComponent::TapDanceForm(ref form)) = state.active_component {
                form.render(f, f.area(), &state.theme);
            }
        }
    }
}

/// Helper to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Render unsaved changes prompt
fn render_unsaved_prompt(f: &mut Frame, theme: &Theme) {
    let area = centered_rect(60, 30, f.area());

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    let text = vec![
        Line::from(""),
        Line::from("You have unsaved changes."),
        Line::from(""),
        Line::from("  [Ctrl+S] Save and quit"),
        Line::from("  [Ctrl+Q] Quit without saving"),
        Line::from("  [Esc] Cancel"),
    ];

    let prompt = Paragraph::new(text).block(
        Block::default()
            .title(popup_title(
                &PopupType::UnsavedChangesPrompt,
                "Unsaved changes",
            ))
            .borders(Borders::ALL)
            .border_style(popup_border_style(&PopupType::UnsavedChangesPrompt, theme))
            .style(Style::default().fg(theme.warning)),
    );

    f.render_widget(prompt, area);
}

/// Render error overlay on top of all other UI elements
fn render_error_overlay(f: &mut Frame, error: &str, theme: &Theme) {
    let area = centered_rect(64, 24, f.area());

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background with error color
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    // Split into title and message
    let chunks = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(4),    // Error message
            Constraint::Length(1), // Help text
        ])
        .split(area);

    // Title with error styling
    let title = Paragraph::new("Something needs attention")
        .style(
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(theme.error).bg(theme.background)),
        );
    f.render_widget(title, chunks[0]);

    // Error message with word wrap
    let error_text = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "Error: ",
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(error.to_string()),
    ])
    .style(Style::default().fg(theme.text))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Details ")
            .style(Style::default().bg(theme.background)),
    )
    .wrap(Wrap { trim: true });
    f.render_widget(error_text, chunks[1]);

    // Help text
    let help = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "Enter/Esc",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Dismiss"),
    ])])
    .style(Style::default().fg(theme.text_muted).bg(theme.background));
    f.render_widget(help, chunks[2]);
}

/// Render template save dialog
fn render_template_save_dialog(f: &mut Frame, state: &AppState) {
    let area = centered_rect(70, 60, f.area());

    let dialog_state = &state.template_save_dialog_state;
    let theme = &state.theme;

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    // Split into fields
    let chunks = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Name field
            Constraint::Length(3), // Description field
            Constraint::Length(3), // Author field
            Constraint::Length(3), // Tags field
            Constraint::Min(2),    // Help text
            Constraint::Length(2), // Action buttons
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Save as Template")
        .style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .title(popup_title(&PopupType::TemplateSaveDialog, "Template save"))
                .borders(Borders::ALL)
                .border_style(popup_border_style(&PopupType::TemplateSaveDialog, theme)),
        );
    f.render_widget(title, chunks[0]);

    // Name field
    let name_style = if matches!(dialog_state.active_field, TemplateSaveField::Name) {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.text)
    };
    let name_text = if matches!(dialog_state.active_field, TemplateSaveField::Name) {
        format!("Name: {}█", dialog_state.name)
    } else {
        format!("Name: {}", dialog_state.name)
    };
    let name = Paragraph::new(name_text)
        .style(name_style)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(name, chunks[1]);

    // Description field
    let desc_style = if matches!(dialog_state.active_field, TemplateSaveField::Description) {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.text)
    };
    let desc_text = if matches!(dialog_state.active_field, TemplateSaveField::Description) {
        format!("Description: {}█", dialog_state.description)
    } else {
        format!("Description: {}", dialog_state.description)
    };
    let description = Paragraph::new(desc_text)
        .style(desc_style)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(description, chunks[2]);

    // Author field
    let author_style = if matches!(dialog_state.active_field, TemplateSaveField::Author) {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.text)
    };
    let author_text = if matches!(dialog_state.active_field, TemplateSaveField::Author) {
        format!("Author: {}█", dialog_state.author)
    } else {
        format!("Author: {}", dialog_state.author)
    };
    let author = Paragraph::new(author_text)
        .style(author_style)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(author, chunks[3]);

    // Tags field
    let tags_style = if matches!(dialog_state.active_field, TemplateSaveField::Tags) {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.text)
    };
    let tags_text = if matches!(dialog_state.active_field, TemplateSaveField::Tags) {
        format!("Tags (comma-separated): {}█", dialog_state.tags_input)
    } else {
        format!("Tags (comma-separated): {}", dialog_state.tags_input)
    };
    let tags = Paragraph::new(tags_text)
        .style(tags_style)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(tags, chunks[4]);

    // Help text
    let help_text = vec![
        Line::from(""),
        Line::from("Tab/Shift+Tab: navigate fields"),
        Line::from("Type: enter text | Backspace: delete"),
    ];
    let help = Paragraph::new(help_text).style(Style::default().fg(theme.text_muted));
    f.render_widget(help, chunks[5]);

    // Action buttons
    let actions = Paragraph::new("Enter: save template | Esc: cancel")
        .style(Style::default().fg(theme.success))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(actions, chunks[6]);
}

/// Render export filename dialog
fn render_export_filename_dialog(f: &mut Frame, state: &AppState) {
    let area = centered_rect(60, 30, f.area());

    let dialog_state = &state.export_filename_dialog_state;
    let theme = &state.theme;

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    // Split into sections
    let chunks = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Filename input
            Constraint::Min(2),    // Help text
            Constraint::Length(2), // Action buttons
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Export Layout to Markdown")
        .style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .title(popup_title(&PopupType::ExportFilenameDialog, "Export"))
                .borders(Borders::ALL)
                .border_style(popup_border_style(&PopupType::ExportFilenameDialog, theme)),
        );
    f.render_widget(title, chunks[0]);

    // Filename field with cursor
    let filename_text = format!("Filename: {}█", dialog_state.filename);
    let filename = Paragraph::new(filename_text)
        .style(Style::default().fg(theme.accent))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(filename, chunks[1]);

    // Help text
    let help_text = vec![
        Line::from(""),
        Line::from("Type: enter filename | Backspace: delete"),
    ];
    let help = Paragraph::new(help_text).style(Style::default().fg(theme.text_muted));
    f.render_widget(help, chunks[2]);

    // Action buttons
    let actions = Paragraph::new("Enter: export | Esc: cancel")
        .style(Style::default().fg(theme.success))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(actions, chunks[3]);
}
