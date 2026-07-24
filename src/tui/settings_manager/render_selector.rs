//! Selector widgets for the settings manager (enum selectors with option lists).

use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::models::{
    ComboAction, PaletteFxEffect, PaletteFxPalette, RgbMatrixEffect, RippleColorMode,
};

use super::SettingItem;
use super::Theme;

/// Generic enum selector renderer
fn render_enum_selector<S1: AsRef<str>, S2: AsRef<str>>(
    f: &mut Frame,
    area: Rect,
    title: &str,
    options: &[(S1, S2)],
    selected: usize,
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(5),    // Options list
            Constraint::Length(4), // Help
        ])
        .split(area);

    // Title
    let title_text = Paragraph::new(title).alignment(Alignment::Center).style(
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(title_text, chunks[0]);

    // Options list
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let style = if i == selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let marker = if i == selected { "▶ " } else { "  " };

            let content = Line::from(vec![
                Span::styled(marker, Style::default().fg(theme.primary)),
                Span::styled(name.as_ref(), style),
                Span::styled(" - ", Style::default().fg(theme.text_muted)),
                Span::styled(desc.as_ref(), Style::default().fg(theme.text_muted)),
            ]);

            ListItem::new(content)
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
            Span::raw(": Select  "),
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

/// Render tap-hold preset selector
pub(super) fn render_tap_hold_preset_selector(
    f: &mut Frame,
    area: Rect,
    selected: usize,
    theme: &Theme,
) {
    let options = TapHoldPreset::all();
    render_enum_selector(
        f,
        area,
        "Tap-Hold Preset",
        options
            .iter()
            .map(|o| (o.display_name(), o.description()))
            .collect::<Vec<_>>()
            .as_slice(),
        selected,
        theme,
    );
}

/// Render hold mode selector
pub(super) fn render_hold_mode_selector(f: &mut Frame, area: Rect, selected: usize, theme: &Theme) {
    let options = HoldDecisionMode::all();
    render_enum_selector(
        f,
        area,
        "Hold Mode",
        options
            .iter()
            .map(|o| (o.display_name(), o.description()))
            .collect::<Vec<_>>()
            .as_slice(),
        selected,
        theme,
    );
}

use crate::models::{HoldDecisionMode, TapHoldPreset};

/// Render output format selector
pub(super) fn render_output_format_selector(
    f: &mut Frame,
    area: Rect,
    selected_option: usize,
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
    let title_text = Paragraph::new("Output Format")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title_text, chunks[0]);

    // Options
    let options = ["uf2", "hex", "bin"];
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, label)| {
            let selected = idx == selected_option;
            let style = if selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let marker = if selected { "▶ " } else { "  " };
            let description = match *label {
                "uf2" => " (USB Flashing Format - RP2040, etc.)",
                "hex" => " (Intel HEX - AVR, etc.)",
                "bin" => " (Raw binary)",
                _ => "",
            };

            ListItem::new(Line::from(vec![
                Span::styled(marker, Style::default().fg(theme.primary)),
                Span::styled(*label, style),
                Span::styled(description, Style::default().fg(theme.text_muted)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Format"))
        .highlight_style(Style::default().bg(theme.surface));

    f.render_widget(list, chunks[1]);

    // Help text
    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(theme.primary)),
            Span::raw(": Select  "),
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

/// Render theme mode selector
pub(super) fn render_theme_mode_selector(
    f: &mut Frame,
    area: Rect,
    selected_option: usize,
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
    let title_text = Paragraph::new("Theme Mode")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title_text, chunks[0]);

    // Options
    let options = [
        ("Auto", "Follow OS dark/light mode setting"),
        ("Dark", "Always use dark theme"),
        ("Light", "Always use light theme"),
    ];
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, (label, description))| {
            let selected = idx == selected_option;
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
                Span::styled(" - ", Style::default().fg(theme.text_muted)),
                Span::styled(*description, Style::default().fg(theme.text_muted)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Theme"))
        .highlight_style(Style::default().bg(theme.surface));

    f.render_widget(list, chunks[1]);

    // Help text
    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(theme.primary)),
            Span::raw(": Select  "),
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

/// Render idle effect mode selector
pub(super) fn render_idle_effect_mode_selector(
    f: &mut Frame,
    area: Rect,
    selected: usize,
    theme: &Theme,
) {
    let options = RgbMatrixEffect::all();
    render_enum_selector(
        f,
        area,
        "Idle Effect Mode",
        options
            .iter()
            .map(|o| (o.display_name(), "RGB animation during idle period"))
            .collect::<Vec<_>>()
            .as_slice(),
        selected,
        theme,
    );
}

/// Render ripple color mode selector
pub(super) fn render_ripple_color_mode_selector(
    f: &mut Frame,
    area: Rect,
    selected: usize,
    theme: &Theme,
) {
    let options = RippleColorMode::all();
    render_enum_selector(
        f,
        area,
        "Ripple Color Mode",
        options
            .iter()
            .map(|o| (o.display_name(), o.description()))
            .collect::<Vec<_>>()
            .as_slice(),
        selected,
        theme,
    );
}

/// Render `PaletteFX` effect selector
pub(super) fn render_palette_fx_effect_selector(
    f: &mut Frame,
    area: Rect,
    selected: usize,
    theme: &Theme,
) {
    let options = PaletteFxEffect::all();
    render_enum_selector(
        f,
        area,
        "PaletteFX Effect",
        options
            .iter()
            .map(|o| (o.display_name(), "Palette-based RGB animation"))
            .collect::<Vec<_>>()
            .as_slice(),
        selected,
        theme,
    );
}

/// Render `PaletteFX` palette selector
pub(super) fn render_palette_fx_palette_selector(
    f: &mut Frame,
    area: Rect,
    selected: usize,
    theme: &Theme,
) {
    let options = PaletteFxPalette::all();
    render_enum_selector(
        f,
        area,
        "PaletteFX Palette",
        options
            .iter()
            .map(|o| (o.display_name(), "Color palette for effects"))
            .collect::<Vec<_>>()
            .as_slice(),
        selected,
        theme,
    );
}

/// Render key action palette selector with "Default" as first option
pub(super) fn render_key_action_palette_selector(
    f: &mut Frame,
    area: Rect,
    selected: usize,
    theme: &Theme,
) {
    let palettes = PaletteFxPalette::all();
    let mut options: Vec<(&str, &str)> = Vec::with_capacity(palettes.len() + 1);
    options.push(("Default (current)", "Use the active PaletteFX palette"));
    for p in palettes {
        options.push((p.display_name(), "Fixed palette for key-action bursts"));
    }
    render_enum_selector(f, area, "Key-Action Palette", &options, selected, theme);
}

/// Render key position selector instruction
pub(super) fn render_key_position_selector(
    f: &mut Frame,
    area: Rect,
    setting: SettingItem,
    instruction: &str,
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(3),    // Instructions
            Constraint::Length(4), // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled("Select Key Position: ", Style::default().fg(theme.primary)),
        Span::styled(setting.display_name(), Style::default().fg(theme.accent)),
    ])])
    .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            instruction,
            Style::default().fg(theme.text),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Use arrow keys to navigate the keyboard below",
            Style::default().fg(theme.text_muted),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Instructions")
            .style(Style::default().bg(theme.background)),
    );
    f.render_widget(instructions, chunks[1]);

    // Help
    let help = Paragraph::new(vec![Line::from(vec![
        Span::styled("Arrow Keys", Style::default().fg(theme.primary)),
        Span::raw(": Navigate  "),
        Span::styled("Enter", Style::default().fg(theme.primary)),
        Span::raw(": Select Key  "),
        Span::styled("Esc", Style::default().fg(theme.primary)),
        Span::raw(": Cancel"),
    ])])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Controls")
            .style(Style::default().bg(theme.background)),
    );
    f.render_widget(help, chunks[2]);
}

/// Render the combo action selector with the highlighted option matching `current`.
pub(super) fn render_combo_action_selector(
    f: &mut Frame,
    area: Rect,
    idx: usize,
    current: &ComboAction,
    theme: &Theme,
) {
    let actions = ComboAction::all();
    let selected = actions.iter().position(|a| a == current).unwrap_or(0);
    let options: Vec<(&str, &str)> = actions
        .iter()
        .map(|a| {
            let desc = match a {
                ComboAction::DisableEffects => "Disable RGB effects, fall back to static colors",
                ComboAction::DisableLighting => "Turn off all keyboard lighting",
                ComboAction::Bootloader => "Enter bootloader mode for firmware flashing",
            };
            (a.display_name(), desc)
        })
        .collect();

    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(4),
        ])
        .split(area);

    let title = Paragraph::new(format!("Combo {} Action", idx + 1))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let is_selected = i == selected;
            let style = if is_selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            let marker = if is_selected { "▶ " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::styled(marker, Style::default().fg(theme.primary)),
                Span::styled(*name, style),
                Span::styled(" - ", Style::default().fg(theme.text_muted)),
                Span::styled(*desc, Style::default().fg(theme.text_muted)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Action"))
        .highlight_style(Style::default().bg(theme.surface));
    f.render_widget(list, chunks[1]);

    let help = Paragraph::new(vec![Line::from(vec![
        Span::styled("↑/↓", Style::default().fg(theme.primary)),
        Span::raw(": Select  "),
        Span::styled("Enter", Style::default().fg(theme.primary)),
        Span::raw(": Apply  "),
        Span::styled("Esc", Style::default().fg(theme.primary)),
        Span::raw(": Cancel"),
    ])])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Controls")
            .style(Style::default().bg(theme.background)),
    );
    f.render_widget(help, chunks[2]);
}
