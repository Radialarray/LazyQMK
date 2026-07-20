//! Rendering code for the onboarding wizard.
//! Extracted from `onboarding_wizard` to keep that file under 1000 lines.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use super::help_registry::HelpRegistry;
use super::onboarding_wizard::{
    KeyboardSelectionFocus, OnboardingWizardState, WelcomeChoice, WizardStep,
};
use crate::tui::Theme;

/// Renders the onboarding wizard
pub fn render(f: &mut Frame, state: &OnboardingWizardState, theme: &Theme) {
    let size = f.area();

    // Clear the background area first
    f.render_widget(Clear, size);

    // Render opaque background for entire wizard
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, size);

    // Create centered layout
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Instructions
            Constraint::Length(2), // Error message
        ])
        .split(size);

    // Render title
    let title = Paragraph::new(state.current_step.title())
        .style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(theme.background)),
        );
    f.render_widget(title, vertical_chunks[0]);

    // Render content based on current step
    match state.current_step {
        WizardStep::Welcome => render_welcome(f, state, vertical_chunks[1], theme),
        WizardStep::QmkPath => render_qmk_path_input(f, state, vertical_chunks[1], theme),
        WizardStep::KeyboardSelection => {
            render_keyboard_selection(f, state, vertical_chunks[1], theme);
        }
        WizardStep::LayoutSelection => render_layout_selection(f, state, vertical_chunks[1], theme),
        WizardStep::LayoutName => render_layout_name_input(f, state, vertical_chunks[1], theme),
        WizardStep::OutputPath => render_output_path_input(f, state, vertical_chunks[1], theme),
        WizardStep::Confirmation => render_confirmation(f, state, vertical_chunks[1], theme),
    }

    // Render instructions at the bottom
    render_instructions(f, state, vertical_chunks[2], theme);

    // Render error message if present
    if let Some(ref error) = state.error_message {
        render_error(f, error, vertical_chunks[3], theme);
    }
}

/// Render welcome screen with options
fn render_welcome(f: &mut Frame, state: &OnboardingWizardState, area: Rect, theme: &Theme) {
    let app_name = HelpRegistry::default().app_name().to_string();
    let welcome_text = vec![
        Line::from(vec![Span::styled(
            format!("Welcome to {app_name}!"),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("Get started by choosing how you'd like to set up your layout:"),
        Line::from(""),
    ];

    // Create items for each welcome option
    let options = state.get_welcome_options();
    let items: Vec<ListItem> = options
        .iter()
        .map(|opt| {
            let label = match opt {
                WelcomeChoice::LoadExisting => "📂  Load an existing layout",
                WelcomeChoice::FromScratch => "✨  Create a new layout from scratch",
                WelcomeChoice::FromTemplate => "📋  Create a layout from a template",
            };
            ListItem::new(Line::from(Span::styled(
                label,
                Style::default().fg(theme.text),
            )))
        })
        .collect();

    // Combine welcome text and list
    let lines: Vec<Line> = welcome_text;
    let list_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(lines.len() as u16), // Welcome text
            Constraint::Min(5),                     // Option list
        ])
        .split(area);

    // Render welcome text
    let text_widget = Paragraph::new(lines)
        .style(Style::default().fg(theme.text))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(text_widget, list_area[0]);

    // Render options list
    let list = List::new(items)
        .block(
            Block::default()
                .title(" Choose your path ")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .fg(theme.background)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(state.welcome_selected_index));
    f.render_stateful_widget(list, list_area[1], &mut list_state);
}

/// Render QMK path input screen
fn render_qmk_path_input(f: &mut Frame, state: &OnboardingWizardState, area: Rect, theme: &Theme) {
    let app_name = HelpRegistry::default().app_name().to_string();
    let text = vec![
        Line::from(vec![Span::styled(
            format!("{app_name} needs to know where your QMK firmware is located."),
            Style::default().fg(theme.text),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("QMK Path: ", Style::default().fg(theme.primary)),
            Span::styled(&state.input_buffer, Style::default().fg(theme.text)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tip: ", Style::default().fg(theme.text_muted)),
            Span::raw(
                "You can usually find this at ~/qmk_firmware or /opt/homebrew/share/qmk_firmware",
            ),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" QMK Path ")
                .style(Style::default().fg(theme.primary)),
        )
        .style(Style::default().fg(theme.text));

    f.render_widget(paragraph, area);
}

/// Render keyboard selection screen
fn render_keyboard_selection(
    f: &mut Frame,
    state: &OnboardingWizardState,
    area: Rect,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter input area
            Constraint::Min(5),    // Keyboard list
        ])
        .split(area);

    // Filter input
    let filter_focused = state.keyboard_selection_focus == KeyboardSelectionFocus::FilterInput;
    let filter_style = if filter_focused {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    let filter_display = if filter_focused {
        format!("🔍 {}█", state.keyboard_filter)
    } else {
        format!("🔍 {}", state.keyboard_filter)
    };

    let filter = Paragraph::new(filter_display).style(filter_style).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Search Keyboards")
            .style(Style::default().fg(theme.primary)),
    );
    f.render_widget(filter, chunks[0]);

    // Keyboard list
    let filtered = state.get_filtered_keyboards();
    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == state.keyboard_selected_index {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            ListItem::new(Line::from(Span::styled(name, style)))
        })
        .collect();

    let list_focused = state.keyboard_selection_focus == KeyboardSelectionFocus::List;
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " Available Keyboards ({} filtered) ",
                    filtered.len()
                ))
                .style(Style::default().fg(theme.primary)),
        )
        .highlight_style(Style::default().fg(theme.background).bg(if list_focused {
            theme.primary
        } else {
            theme.surface
        }));

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(state.keyboard_selected_index));
    f.render_stateful_widget(list, chunks[1], &mut list_state);
}

/// Render layout selection screen
fn render_layout_selection(
    f: &mut Frame,
    state: &OnboardingWizardState,
    area: Rect,
    theme: &Theme,
) {
    let items: Vec<ListItem> = state
        .available_layouts
        .iter()
        .enumerate()
        .map(|(i, layout)| {
            let style = if i == state.layout_selected_index {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            ListItem::new(Line::from(Span::styled(layout, style)))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Available Layouts ")
                .style(Style::default().fg(theme.primary)),
        )
        .highlight_style(
            Style::default()
                .fg(theme.background)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(state.layout_selected_index));
    f.render_stateful_widget(list, area, &mut list_state);
}

/// Render layout name input screen
fn render_layout_name_input(
    f: &mut Frame,
    state: &OnboardingWizardState,
    area: Rect,
    theme: &Theme,
) {
    let text = vec![
        Line::from(vec![
            Span::styled("Layout Name: ", Style::default().fg(theme.primary)),
            Span::styled(&state.input_buffer, Style::default().fg(theme.text)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Creates: ", Style::default().fg(theme.text_muted)),
            Span::raw("layouts/"),
            Span::styled(&state.input_buffer, Style::default().fg(theme.accent)),
            Span::raw(".md"),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Layout Name ")
                .style(Style::default().fg(theme.primary)),
        )
        .style(Style::default().fg(theme.text));

    f.render_widget(paragraph, area);
}

/// Render output path input screen
fn render_output_path_input(
    f: &mut Frame,
    state: &OnboardingWizardState,
    area: Rect,
    theme: &Theme,
) {
    let text = vec![
        Line::from(vec![
            Span::styled("Output Path: ", Style::default().fg(theme.primary)),
            Span::styled(&state.input_buffer, Style::default().fg(theme.text)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tip: ", Style::default().fg(theme.text_muted)),
            Span::raw("Build artifacts will be placed in this folder"),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Build Output ")
                .style(Style::default().fg(theme.primary)),
        )
        .style(Style::default().fg(theme.text));

    f.render_widget(paragraph, area);
}

/// Render confirmation screen
fn render_confirmation(f: &mut Frame, state: &OnboardingWizardState, area: Rect, theme: &Theme) {
    let items: Vec<ListItem> = state
        .inputs
        .iter()
        .map(|(key, value)| {
            let content = Line::from(vec![
                Span::styled(format!("{key}: "), Style::default().fg(theme.primary)),
                Span::styled(value, Style::default().fg(theme.text)),
            ]);
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Review Your Settings ")
                .style(Style::default().fg(theme.primary)),
        )
        .highlight_style(Style::default().fg(theme.accent));

    f.render_widget(list, area);
}

/// Render instructions at the bottom
fn render_instructions(f: &mut Frame, state: &OnboardingWizardState, area: Rect, theme: &Theme) {
    let step_info = format!(
        "Step {} of {}",
        state.current_step.step_number(),
        WizardStep::total_steps()
    );

    let instructions = match state.current_step {
        WizardStep::Welcome => "↑↓: Choose path  |  Enter: Continue  |  Esc: Exit",
        WizardStep::QmkPath | WizardStep::LayoutName | WizardStep::OutputPath => {
            "Enter: Continue  |  Backspace: Delete  |  Esc: Back"
        }
        WizardStep::KeyboardSelection => {
            "Tab/Shift+Tab: Move focus  |  Type: Filter  |  ↑↓: Navigate  |  Enter: Select"
        }
        WizardStep::LayoutSelection => "↑↓: Navigate  |  Enter: Select  |  Esc: Back",
        WizardStep::Confirmation => "Enter: Save & Exit  |  Esc: Back",
    };

    let text = vec![Line::from(step_info), Line::from(instructions)];

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(theme.primary).bg(theme.background)),
        );
    f.render_widget(paragraph, area);
}

/// Render error message
fn render_error(f: &mut Frame, error: &str, area: Rect, theme: &Theme) {
    let text = vec![Line::from(vec![
        Span::styled("⚠ ", Style::default().fg(theme.error)),
        Span::styled(error, Style::default().fg(theme.error)),
    ])];

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(theme.error))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(theme.error).bg(theme.background)),
        );
    f.render_widget(paragraph, area);
}
