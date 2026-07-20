//! Editor widgets for the settings manager (numeric, string, boolean, path editors).

use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::SettingItem;
use super::Theme;

/// Render numeric editor for integer values
pub(super) fn render_numeric_editor(
    f: &mut Frame,
    area: Rect,
    setting: SettingItem,
    value: &str,
    min: u16,
    max: u16,
    default: u16,
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Length(3), // Input field
            Constraint::Min(2),    // Description + range
            Constraint::Length(4), // Help
        ])
        .split(area);

    // Title
    let title_text = Paragraph::new(setting.display_name())
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title_text, chunks[0]);

    // Input field with cursor
    let display_value = format!("{value}▌");
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title("Value");
    let input_text = Paragraph::new(display_value)
        .style(Style::default().fg(theme.text))
        .block(input_block);
    f.render_widget(input_text, chunks[1]);

    // Description and range
    let desc = vec![
        Line::from(setting.description()),
        Line::from(""),
        Line::from(Span::styled(
            format!("Range: {min} to {max}  •  Default: {default}"),
            Style::default().fg(theme.text_muted),
        )),
    ];

    let desc_text = Paragraph::new(desc)
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(Style::default().fg(theme.text_muted));
    f.render_widget(desc_text, chunks[2]);

    // Help text
    let help = vec![
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(theme.primary)),
            Span::raw(": ±10  "),
            Span::styled("←/→", Style::default().fg(theme.primary)),
            Span::raw(": ±1"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(theme.primary)),
            Span::raw(": Apply  "),
            Span::styled("Home", Style::default().fg(theme.primary)),
            Span::raw(": Default  "),
            Span::styled("Esc", Style::default().fg(theme.primary)),
            Span::raw(": Cancel  "),
            Span::styled("Backspace", Style::default().fg(theme.primary)),
            Span::raw(": Delete"),
        ]),
    ];

    let help_widget = Paragraph::new(help)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_muted));

    f.render_widget(help_widget, chunks[3]);
}

/// Render signed numeric editor
pub(super) fn render_signed_numeric_editor(
    f: &mut Frame,
    area: Rect,
    setting: SettingItem,
    value: &str,
    min: i16,
    max: i16,
    default: i16,
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(2),
            Constraint::Length(4),
        ])
        .split(area);

    let title_text = Paragraph::new(setting.display_name())
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title_text, chunks[0]);

    let display_value = format!("{value}▌");
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title("Value");
    let input_text = Paragraph::new(display_value)
        .style(Style::default().fg(theme.text))
        .block(input_block);
    f.render_widget(input_text, chunks[1]);

    let desc = vec![
        Line::from(setting.description()),
        Line::from(""),
        Line::from(Span::styled(
            format!("Range: {min} to {max}  •  Default: {default}"),
            Style::default().fg(theme.text_muted),
        )),
    ];

    let desc_text = Paragraph::new(desc)
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(Style::default().fg(theme.text_muted));
    f.render_widget(desc_text, chunks[2]);

    let help = vec![
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(theme.primary)),
            Span::raw(": ±10  "),
            Span::styled("←/→", Style::default().fg(theme.primary)),
            Span::raw(": ±1  "),
            Span::styled("-", Style::default().fg(theme.primary)),
            Span::raw(": Negative"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(theme.primary)),
            Span::raw(": Apply  "),
            Span::styled("Home", Style::default().fg(theme.primary)),
            Span::raw(": Default  "),
            Span::styled("Esc", Style::default().fg(theme.primary)),
            Span::raw(": Cancel  "),
            Span::styled("Backspace", Style::default().fg(theme.primary)),
            Span::raw(": Delete"),
        ]),
    ];

    let help_widget = Paragraph::new(help)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_muted));

    f.render_widget(help_widget, chunks[3]);
}

/// Render string editor (for keymap name, etc.)
pub(super) fn render_string_editor(
    f: &mut Frame,
    area: Rect,
    setting: SettingItem,
    value: &str,
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Length(3), // Input field
            Constraint::Min(2),    // Description
            Constraint::Length(4), // Help
        ])
        .split(area);

    // Title
    let title_text = Paragraph::new(setting.display_name())
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title_text, chunks[0]);

    // Input field with cursor
    let display_value = format!("{value}▌");
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title("Value");
    let input_text = Paragraph::new(display_value)
        .style(Style::default().fg(theme.text))
        .block(input_block);
    f.render_widget(input_text, chunks[1]);

    // Description
    let desc_text = Paragraph::new(setting.description())
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(Style::default().fg(theme.text_muted));
    f.render_widget(desc_text, chunks[2]);

    // Help text
    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(theme.primary)),
            Span::raw(": Apply  "),
            Span::styled("Esc", Style::default().fg(theme.primary)),
            Span::raw(": Cancel  "),
            Span::styled("Backspace", Style::default().fg(theme.primary)),
            Span::raw(": Delete"),
        ]),
    ];

    let help_widget = Paragraph::new(help)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_muted));

    f.render_widget(help_widget, chunks[3]);
}

/// Render path editor (for QMK path, output directory, etc.)
pub(super) fn render_path_editor(
    f: &mut Frame,
    area: Rect,
    setting: SettingItem,
    value: &str,
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Length(3), // Input field
            Constraint::Min(2),    // Description + current path
            Constraint::Length(4), // Help
        ])
        .split(area);

    // Title
    let title_text = Paragraph::new(setting.display_name())
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title_text, chunks[0]);

    // Input field with cursor
    let display_value = format!("{value}▌");
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title("Path");
    let input_text = Paragraph::new(display_value)
        .style(Style::default().fg(theme.text))
        .block(input_block);
    f.render_widget(input_text, chunks[1]);

    // Description and path validation hint
    let path_exists = std::path::Path::new(value).exists();
    let status_line = if value.is_empty() {
        Line::from(Span::styled(
            "Enter a path",
            Style::default().fg(theme.text_muted),
        ))
    } else if path_exists {
        Line::from(Span::styled(
            "✓ Path exists",
            Style::default().fg(theme.primary),
        ))
    } else {
        Line::from(Span::styled(
            "⚠ Path does not exist (will be created if needed)",
            Style::default().fg(theme.warning),
        ))
    };

    let desc = vec![
        Line::from(setting.description()),
        Line::from(""),
        status_line,
    ];

    let desc_text = Paragraph::new(desc)
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(Style::default().fg(theme.text_muted));
    f.render_widget(desc_text, chunks[2]);

    // Help text
    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(theme.primary)),
            Span::raw(": Apply  "),
            Span::styled("Esc", Style::default().fg(theme.primary)),
            Span::raw(": Cancel  "),
            Span::styled("Backspace", Style::default().fg(theme.primary)),
            Span::raw(": Delete"),
        ]),
    ];

    let help_widget = Paragraph::new(help)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_muted));

    f.render_widget(help_widget, chunks[3]);
}

/// Render boolean toggle
pub(super) fn render_boolean_toggle(
    f: &mut Frame,
    area: Rect,
    setting: SettingItem,
    value: bool,
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(5),    // Options
            Constraint::Length(4), // Help
        ])
        .split(area);

    // Title
    let title_text = Paragraph::new(setting.display_name())
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title_text, chunks[0]);

    // Options (On/Off)
    let items: Vec<ListItem> = [("On", true), ("Off", false)]
        .iter()
        .map(|(label, is_on)| {
            let selected = *is_on == value;
            let style = if selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let marker = if selected { "▶ " } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(marker, Style::default().fg(theme.primary)),
                Span::styled(*label, style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Options"))
        .highlight_style(Style::default().bg(theme.surface));

    f.render_widget(list, chunks[1]);

    // Help text
    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(theme.primary)),
            Span::raw(": Toggle  "),
            Span::styled("Enter", Style::default().fg(theme.primary)),
            Span::raw(": Apply  "),
            Span::styled("Esc", Style::default().fg(theme.primary)),
            Span::raw(": Cancel"),
        ]),
    ];

    let help_widget = Paragraph::new(help)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_muted));

    f.render_widget(help_widget, chunks[2]);
}
