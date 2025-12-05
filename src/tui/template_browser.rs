//! Template browser for loading and managing layout templates.
//!
//! This module provides UI components for browsing, searching, and loading
//! reusable layout templates stored in ~/.`config/KeyboardConfigurator/templates`/

// Allow intentional type casts for layout rendering
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]

use anyhow::{Context, Result};
use ratatui::{
    layout::{Constraint, Direction, Layout as RatatuiLayout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::models::{Layout, LayoutMetadata};
use crate::parser::layout as layout_parser;

/// Template metadata with file path for loading.
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    /// Full path to the template file
    pub path: PathBuf,
    /// Template metadata
    pub metadata: LayoutMetadata,
}

/// State for the template browser dialog.
#[derive(Debug, Clone)]
pub struct TemplateBrowserState {
    /// List of available templates
    pub templates: Vec<TemplateInfo>,
    /// Search filter text
    pub search: String,
    /// Currently selected template index (in filtered list)
    pub selected: usize,
    /// Whether search is active
    pub search_active: bool,
}

impl TemplateBrowserState {
    /// Creates a new template browser state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            templates: Vec::new(),
            search: String::new(),
            selected: 0,
            search_active: false,
        }
    }

    /// Scans the templates directory and loads template metadata.
    ///
    /// Templates are stored in ~/.`config/KeyboardConfigurator/templates`/
    pub fn scan_templates(&mut self) -> Result<()> {
        self.templates.clear();

        let templates_dir = Self::templates_dir()?;

        // Create directory if it doesn't exist
        if !templates_dir.exists() {
            fs::create_dir_all(&templates_dir).context(format!(
                "Failed to create templates directory: {}",
                templates_dir.display()
            ))?;
            return Ok(()); // Empty directory, no templates
        }

        // Scan directory for .md files
        let entries = fs::read_dir(&templates_dir).context(format!(
            "Failed to read templates directory: {}",
            templates_dir.display()
        ))?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            // Only process .md files
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            // Try to parse the template file
            match layout_parser::parse_markdown_layout(&path) {
                Ok(layout) => {
                    // Only include files marked as templates
                    if layout.metadata.is_template {
                        self.templates.push(TemplateInfo {
                            path: path.clone(),
                            metadata: layout.metadata,
                        });
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse template {}: {}",
                        path.display(),
                        e
                    );
                    // Continue scanning other files
                }
            }
        }

        // Sort templates by name
        self.templates
            .sort_by(|a, b| a.metadata.name.cmp(&b.metadata.name));

        // Reset selection
        self.selected = 0;

        Ok(())
    }

    /// Gets the platform-specific templates directory path.
    ///
    /// - Unix/Linux/macOS: `~/.config/KeyboardConfigurator/templates/`
    /// - Windows: `%APPDATA%\KeyboardConfigurator\templates\`
    pub fn templates_dir() -> Result<PathBuf> {
        Ok(Config::config_dir()?.join("templates"))
    }

    /// Filters templates by search text.
    ///
    /// Searches in:
    /// - Template name
    /// - Template description
    /// - Template tags
    fn filtered_templates(&self) -> Vec<&TemplateInfo> {
        if self.search.is_empty() {
            return self.templates.iter().collect();
        }

        let search_lower = self.search.to_lowercase();

        self.templates
            .iter()
            .filter(|t| {
                // Search in name
                t.metadata.name.to_lowercase().contains(&search_lower)
                    // Search in description
                    || t.metadata.description.to_lowercase().contains(&search_lower)
                    // Search in tags
                    || t.metadata.tags.iter().any(|tag| tag.contains(&search_lower))
            })
            .collect()
    }

    /// Gets the currently selected template (if any).
    #[must_use]
    pub fn get_selected_template(&self) -> Option<&TemplateInfo> {
        let filtered = self.filtered_templates();
        filtered.get(self.selected).copied()
    }

    /// Loads the selected template as a Layout.
    pub fn load_selected_template(&self) -> Result<Layout> {
        let template = self
            .get_selected_template()
            .context("No template selected")?;

        let layout = layout_parser::parse_markdown_layout(&template.path).context(format!(
            "Failed to load template: {}",
            template.path.display()
        ))?;

        Ok(layout)
    }

    /// Moves selection up in the filtered list.
    pub const fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Moves selection down in the filtered list.
    pub fn select_next(&mut self) {
        let filtered_count = self.filtered_templates().len();
        if filtered_count > 0 && self.selected < filtered_count - 1 {
            self.selected += 1;
        }
    }

    /// Adds a character to the search text.
    pub fn search_push(&mut self, ch: char) {
        self.search.push(ch);
        // Reset selection when search changes
        self.selected = 0;
    }

    /// Removes last character from search text.
    pub fn search_pop(&mut self) {
        self.search.pop();
        // Reset selection when search changes
        self.selected = 0;
    }

    /// Clears the search text.
    pub fn search_clear(&mut self) {
        self.search.clear();
        self.selected = 0;
    }

    /// Toggles search mode.
    pub fn toggle_search(&mut self) {
        self.search_active = !self.search_active;
        if !self.search_active {
            // Clear search when exiting search mode
            self.search_clear();
        }
    }
}

impl Default for TemplateBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

/// Renders the template browser popup.
#[allow(clippy::too_many_lines)]
pub fn render(
    f: &mut Frame,
    state: &TemplateBrowserState,
    area: Rect,
    theme: &crate::tui::theme::Theme,
) {
    // Center the popup (60% width, 80% height)
    let popup_width = (f32::from(area.width) * 0.6) as u16;
    let popup_height = (f32::from(area.height) * 0.8) as u16;

    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the background area first
    f.render_widget(Clear, popup_area);

    // Render opaque background
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, popup_area);

    // Split into title, search, list, and details sections
    let chunks = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Search bar
            Constraint::Min(10),   // Template list
            Constraint::Length(8), // Template details
            Constraint::Length(2), // Help line
        ])
        .split(popup_area);

    // Render title
    let title_style = if state.search_active {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD)
    };

    let title = Paragraph::new("Template Browser")
        .block(Block::default().borders(Borders::ALL).style(title_style));
    f.render_widget(title, chunks[0]);

    // Render search bar
    let search_text = if state.search_active {
        format!("Search: {}█", state.search)
    } else {
        format!("Search: {} (Press / to search)", state.search)
    };

    let search_style = if state.search_active {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.text_muted)
    };

    let search = Paragraph::new(search_text)
        .block(Block::default().borders(Borders::ALL).style(search_style));
    f.render_widget(search, chunks[1]);

    // Get filtered templates
    let filtered = state.filtered_templates();

    // Render template list
    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, template)| {
            let tags_str = if template.metadata.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", template.metadata.tags.join(", "))
            };

            let content = format!("{}{}", template.metadata.name, tags_str);

            let style = if i == state.selected {
                Style::default()
                    .fg(theme.background)
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let list_title = format!("Templates ({})", filtered.len());
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(list_title));
    f.render_widget(list, chunks[2]);

    // Render selected template details
    let details_content = if let Some(template) = state.get_selected_template() {
        vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(theme.primary)),
                Span::raw(&template.metadata.name),
            ]),
            Line::from(vec![
                Span::styled("Author: ", Style::default().fg(theme.primary)),
                Span::raw(&template.metadata.author),
            ]),
            Line::from(vec![
                Span::styled("Description: ", Style::default().fg(theme.primary)),
                Span::raw(&template.metadata.description),
            ]),
            Line::from(vec![
                Span::styled("Tags: ", Style::default().fg(theme.primary)),
                Span::raw(template.metadata.tags.join(", ")),
            ]),
            Line::from(vec![
                Span::styled("Created: ", Style::default().fg(theme.primary)),
                Span::raw(template.metadata.created.format("%Y-%m-%d").to_string()),
            ]),
        ]
    } else if filtered.is_empty() {
        vec![Line::from(Span::styled(
            "No templates found. Create one with Shift+T",
            Style::default().fg(theme.warning),
        ))]
    } else {
        vec![Line::from(Span::raw("No template selected"))]
    };

    let details = Paragraph::new(details_content)
        .block(Block::default().borders(Borders::ALL).title("Details"));
    f.render_widget(details, chunks[3]);

    // Render help line
    let help_text = if state.search_active {
        "Type to search | Esc: exit search | Enter: load template | q: cancel"
    } else {
        "↑/↓: navigate | /: search | Enter: load template | Esc/q: cancel"
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(theme.text_muted))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[4]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_browser_state_new() {
        let state = TemplateBrowserState::new();
        assert!(state.templates.is_empty());
        assert!(state.search.is_empty());
        assert_eq!(state.selected, 0);
        assert!(!state.search_active);
    }

    #[test]
    fn test_template_browser_search() {
        let mut state = TemplateBrowserState::new();

        state.search_push('t');
        state.search_push('e');
        state.search_push('s');
        state.search_push('t');
        assert_eq!(state.search, "test");

        state.search_pop();
        assert_eq!(state.search, "tes");

        state.search_clear();
        assert!(state.search.is_empty());
    }

    #[test]
    fn test_template_browser_navigation() {
        let mut state = TemplateBrowserState::new();

        // Add some dummy templates
        for i in 0..5 {
            state.templates.push(TemplateInfo {
                path: PathBuf::from(format!("test{i}.md")),
                metadata: LayoutMetadata::new(format!("Template {i}")).unwrap(),
            });
        }

        assert_eq!(state.selected, 0);

        state.select_next();
        assert_eq!(state.selected, 1);

        state.select_next();
        assert_eq!(state.selected, 2);

        state.select_previous();
        assert_eq!(state.selected, 1);

        state.select_previous();
        assert_eq!(state.selected, 0);

        // Can't go below 0
        state.select_previous();
        assert_eq!(state.selected, 0);
    }
}
