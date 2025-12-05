//! Layout picker for loading saved layouts.
//!
//! This module provides UI components for browsing and loading
//! saved layout files from ~/.config/KeyboardConfigurator/layouts/

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout as RatatuiLayout},
    style::{Modifier, Style},
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
    /// Layouts are stored in ~/.config/KeyboardConfigurator/layouts/
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
    /// - Unix/Linux/macOS: `~/.config/KeyboardConfigurator/layouts/`
    /// - Windows: `%APPDATA%\KeyboardConfigurator\layouts\`
    pub fn layouts_dir() -> Result<PathBuf> {
        Ok(Config::config_dir()?.join("layouts"))
    }

    /// Gets the selected layout path, or None if "Create New" is selected.
    #[must_use]
    #[allow(dead_code)]
    pub fn get_selected_layout(&self) -> Option<&LayoutInfo> {
        if self.create_new {
            None
        } else {
            self.layouts.get(self.selected)
        }
    }
}

impl Default for LayoutPickerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Renders the layout picker dialog.
pub fn render(f: &mut Frame, state: &LayoutPickerState, theme: &crate::tui::theme::Theme) {
    let size = f.size();

    // Create centered dialog
    let vertical_chunks = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // List
            Constraint::Length(3), // Instructions
        ])
        .split(size);

    // Render title
    let title = Paragraph::new("Select a Layout")
        .style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, vertical_chunks[0]);

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

        let modified = layout_info
            .metadata
            .modified
            .format("%Y-%m-%d %H:%M")
            .to_string();

        let text = format!("{} ({})", layout_info.metadata.name, modified);

        items.push(ListItem::new(text).style(style));
    }

    let list_title = if state.layouts.is_empty() {
        "No saved layouts found".to_string()
    } else {
        format!("Saved Layouts ({} total)", state.layouts.len())
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(list_title))
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, vertical_chunks[1]);

    // Render instructions
    let instructions = "↑↓: Navigate  |  Enter: Select  |  Esc: Cancel";
    let paragraph = Paragraph::new(instructions)
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, vertical_chunks[2]);
}

/// Action returned by layout picker when user makes a choice.
#[derive(Debug, Clone)]
pub enum PickerAction {
    /// User chose to create a new layout
    CreateNew,
    /// User selected an existing layout to load
    LoadLayout(PathBuf),
    /// User cancelled the picker
    Cancel,
}

/// Handles keyboard input for the layout picker.
///
/// Returns Some(action) if user made a choice, None otherwise.
pub fn handle_input(state: &mut LayoutPickerState, key: KeyEvent) -> Result<Option<PickerAction>> {
    match key.code {
         KeyCode::Up | KeyCode::Char('k') => {
             if state.create_new {
                 // Move from "Create New" to last layout
                 if !state.layouts.is_empty() {
                     state.create_new = false;
                     state.selected = state.layouts.len() - 1;
                 }
             } else if state.selected > 0 {
                 state.selected -= 1;
             } else {
                 // Wrap to "Create New"
                 state.create_new = true;
             }
             Ok(None)
         }
         KeyCode::Down | KeyCode::Char('j') => {
             if state.create_new {
                 // Move from "Create New" to first layout
                 if !state.layouts.is_empty() {
                     state.create_new = false;
                     state.selected = 0;
                 }
             } else if state.selected < state.layouts.len() - 1 {
                 state.selected += 1;
             } else {
                 // Wrap to "Create New"
                 state.create_new = true;
             }
             Ok(None)
         }
        KeyCode::Enter => {
            // User made a selection
            if state.create_new {
                Ok(Some(PickerAction::CreateNew))
            } else if let Some(layout_info) = state.layouts.get(state.selected) {
                Ok(Some(PickerAction::LoadLayout(layout_info.path.clone())))
            } else {
                // No layouts available, default to creating new
                Ok(Some(PickerAction::CreateNew))
            }
        }
        KeyCode::Esc => {
            // User cancelled
            Ok(Some(PickerAction::Cancel))
        }
        _ => Ok(None),
    }
}
