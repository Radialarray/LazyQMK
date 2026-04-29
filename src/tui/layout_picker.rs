//! Layout picker for loading saved layouts.
//!
//! This module provides UI components for browsing and loading
//! saved layout files from ~/.config/LazyQMK/layouts/

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout as RatatuiLayout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::models::LayoutMetadata;
use crate::parser::layout as layout_parser;

/// Layout file information with path and metadata.
#[derive(Debug, Clone)]
pub struct LayoutInfo {
    /// Full path to the layout file
    pub path: PathBuf,
    /// Layout metadata
    pub metadata: LayoutMetadata,
}

/// State for the layout picker dialog.
#[derive(Debug, Clone)]
pub struct LayoutPickerState {
    /// List of available layouts
    pub layouts: Vec<LayoutInfo>,
    /// Currently selected layout index
    pub selected: usize,
    /// Whether user wants to create new layout
    pub create_new: bool,
}

fn format_timestamp(timestamp: DateTime<chrono::Utc>) -> String {
    timestamp
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

fn metadata_summary(layout_info: &LayoutInfo) -> Vec<Line<'static>> {
    let metadata = &layout_info.metadata;
    let modified = format_timestamp(metadata.modified);
    let created = format_timestamp(metadata.created);
    let keyboard = metadata.keyboard.as_deref().unwrap_or("Not set");
    let variant = metadata.layout_variant.as_deref().unwrap_or("Default");
    let keymap = metadata.keymap_name.as_deref().unwrap_or("Not set");
    let output = metadata.output_format.as_deref().unwrap_or("Not set");

    let description = if metadata.description.trim().is_empty() {
        "No description yet.".to_string()
    } else {
        metadata.description.trim().to_string()
    };

    let tags = if metadata.tags.is_empty() {
        "none".to_string()
    } else {
        metadata.tags.join(", ")
    };

    vec![
        Line::from(vec![Span::raw("Name: "), Span::raw(metadata.name.clone())]),
        Line::from(vec![
            Span::raw("Keyboard: "),
            Span::raw(keyboard.to_string()),
        ]),
        Line::from(vec![
            Span::raw("Layout variant: "),
            Span::raw(variant.to_string()),
        ]),
        Line::from(vec![
            Span::raw("Keymap name: "),
            Span::raw(keymap.to_string()),
        ]),
        Line::from(vec![
            Span::raw("Output format: "),
            Span::raw(output.to_string()),
        ]),
        Line::from(vec![Span::raw("Updated: "), Span::raw(modified)]),
        Line::from(vec![Span::raw("Created: "), Span::raw(created)]),
        Line::from(vec![Span::raw("Tags: "), Span::raw(tags)]),
        Line::from(""),
        Line::from(description),
    ]
}

impl LayoutPickerState {
    /// Creates a new layout picker state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            layouts: Vec::new(),
            selected: 0,
            create_new: false,
        }
    }

    /// Scans the layouts directory and loads layout metadata.
    ///
    /// Layouts are stored in ~/.config/LazyQMK/layouts/
    pub fn scan_layouts(&mut self) -> Result<()> {
        self.layouts.clear();

        let layouts_dir = Self::layouts_dir()?;

        // Create directory if it doesn't exist
        if !layouts_dir.exists() {
            fs::create_dir_all(&layouts_dir).context(format!(
                "Failed to create layouts directory: {}",
                layouts_dir.display()
            ))?;
            return Ok(()); // Empty directory, no layouts
        }

        // Scan directory for .md files
        let entries = fs::read_dir(&layouts_dir).context(format!(
            "Failed to read layouts directory: {}",
            layouts_dir.display()
        ))?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            // Only process .md files
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            // Try to parse the layout file
            match layout_parser::parse_markdown_layout(&path) {
                Ok(layout) => {
                    // Don't include template files
                    if !layout.metadata.is_template {
                        self.layouts.push(LayoutInfo {
                            path: path.clone(),
                            metadata: layout.metadata,
                        });
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse layout {}: {}", path.display(), e);
                    // Continue scanning other files
                }
            }
        }

        // Sort layouts by modified date (most recent first)
        self.layouts
            .sort_by(|a, b| b.metadata.modified.cmp(&a.metadata.modified));

        // Reset selection
        self.selected = 0;

        Ok(())
    }

    /// Gets the platform-specific layouts directory path.
    ///
    /// - Unix/Linux/macOS: `~/.config/LazyQMK/layouts/`
    /// - Windows: `%APPDATA%\LazyQMK\layouts\`
    pub fn layouts_dir() -> Result<PathBuf> {
        Ok(Config::config_dir()?.join("layouts"))
    }
}

impl Default for LayoutPickerState {
    fn default() -> Self {
        Self::new()
    }
}

/// LayoutPicker component that implements the Component trait
#[derive(Debug, Clone)]
pub struct LayoutPicker {
    /// Internal state of the layout picker
    state: LayoutPickerState,
}

impl LayoutPicker {
    /// Create a new LayoutPicker
    #[must_use]
    pub fn new() -> Self {
        let mut state = LayoutPickerState::new();
        // Attempt to scan layouts on creation (ignore errors)
        let _ = state.scan_layouts();
        Self { state }
    }
}

impl Default for LayoutPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::tui::component::Component for LayoutPicker {
    type Event = LayoutPickerEvent;

    fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> Option<Self::Event> {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.state.create_new {
                    // Move from "Create New" to last layout
                    if !self.state.layouts.is_empty() {
                        self.state.create_new = false;
                        self.state.selected = self.state.layouts.len() - 1;
                    }
                } else if self.state.selected > 0 {
                    self.state.selected -= 1;
                } else {
                    // Wrap to "Create New"
                    self.state.create_new = true;
                }
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.state.create_new {
                    // Move from "Create New" to first layout
                    if !self.state.layouts.is_empty() {
                        self.state.create_new = false;
                        self.state.selected = 0;
                    }
                } else if self.state.selected < self.state.layouts.len() - 1 {
                    self.state.selected += 1;
                } else {
                    // Wrap to "Create New"
                    self.state.create_new = true;
                }
                None
            }
            KeyCode::Enter => {
                // User made a selection
                if self.state.create_new {
                    Some(LayoutPickerEvent::CreateNew)
                } else if let Some(layout_info) = self.state.layouts.get(self.state.selected) {
                    Some(LayoutPickerEvent::LayoutSelected(layout_info.path.clone()))
                } else {
                    // No layouts available, default to creating new
                    Some(LayoutPickerEvent::CreateNew)
                }
            }
            KeyCode::Esc => Some(LayoutPickerEvent::Cancelled),
            _ => None,
        }
    }

    fn render(&self, f: &mut Frame, area: Rect, theme: &crate::tui::theme::Theme) {
        render_layout_picker_component(f, self, area, theme);
    }
}

/// Renders the layout picker dialog (for Component)
fn render_layout_picker_component(
    f: &mut Frame,
    picker: &LayoutPicker,
    _area: Rect,
    theme: &crate::tui::theme::Theme,
) {
    let size = f.area();
    let state = &picker.state;

    // Create centered dialog
    let vertical_chunks = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // List + details
            Constraint::Length(3), // Instructions
        ])
        .split(size);

    // Render title
    let title = Paragraph::new("Open Saved Layout")
        .style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, vertical_chunks[0]);

    let content_chunks = RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(vertical_chunks[1]);

    // Build list items
    let mut items: Vec<ListItem> = Vec::new();

    // Add "Create New" option first
    let create_new_style = if state.create_new {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.success)
    };
    items.push(ListItem::new("+ Create New Layout").style(create_new_style));

    // Add saved layouts
    for (i, layout_info) in state.layouts.iter().enumerate() {
        let is_selected = !state.create_new && i == state.selected;

        let style = if is_selected {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let modified = format_timestamp(layout_info.metadata.modified);
        let keyboard = layout_info
            .metadata
            .keyboard
            .as_deref()
            .unwrap_or("keyboard not set");

        let text = format!(
            "{}  ·  {}  ·  {}",
            layout_info.metadata.name, keyboard, modified
        );

        items.push(ListItem::new(text).style(style));
    }

    let list_title = if state.layouts.is_empty() {
        "No saved layouts yet".to_string()
    } else {
        format!("Saved layouts ({} total)", state.layouts.len())
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(list_title))
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, content_chunks[0]);

    let details = if state.create_new {
        vec![
            Line::from("Start fresh layout."),
            Line::from(""),
            Line::from("Use this when you want:"),
            Line::from("• blank layout with current keyboard setup"),
            Line::from("• new experiment without touching saved work"),
            Line::from("• different keymap or RGB plan"),
        ]
    } else if let Some(layout_info) = state.layouts.get(state.selected) {
        metadata_summary(layout_info)
    } else {
        vec![
            Line::from("No saved layouts yet."),
            Line::from("Create new layout to get started."),
        ]
    };

    let details_title = if state.create_new {
        "What happens next"
    } else {
        "Selected layout details"
    };

    let details_widget = Paragraph::new(details)
        .style(Style::default().fg(theme.text))
        .block(Block::default().borders(Borders::ALL).title(details_title));
    f.render_widget(details_widget, content_chunks[1]);

    // Render instructions
    let instructions = "↑↓: Review options  |  Enter: Open selected layout  |  Esc: Cancel";
    let paragraph = Paragraph::new(instructions)
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, vertical_chunks[2]);
}

/// Events emitted by the LayoutPicker component
#[derive(Debug, Clone)]
pub enum LayoutPickerEvent {
    /// User chose to create a new layout
    CreateNew,
    /// User selected an existing layout to load
    LayoutSelected(PathBuf),
    /// User cancelled the picker
    Cancelled,
}
