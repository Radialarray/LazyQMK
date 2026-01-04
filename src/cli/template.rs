//! Template management commands for layout files.

use crate::cli::common::{CliError, CliResult};
use crate::config::Config;
use crate::parser::layout::parse_markdown_layout;
use crate::parser::template_gen::save_markdown_layout;
use crate::services::LayoutService;
use chrono::Utc;
use clap::{Args, Subcommand};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

/// Manage layout templates
#[derive(Debug, Clone, Args)]
pub struct TemplateArgs {
    /// Template subcommand
    #[command(subcommand)]
    pub command: TemplateCommand,
}

/// Template subcommands
#[derive(Debug, Clone, Subcommand)]
pub enum TemplateCommand {
    /// List available templates
    List(ListArgs),
    /// Save current layout as a template
    Save(SaveArgs),
    /// Apply a template to create a new layout
    Apply(ApplyArgs),
}

/// List available templates
#[derive(Debug, Clone, Args)]
pub struct ListArgs {
    /// Output results as JSON
    #[arg(long)]
    pub json: bool,
}

/// Save layout as a template
#[derive(Debug, Clone, Args)]
pub struct SaveArgs {
    /// Path to layout file
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,

    /// Template name
    #[arg(short, long, value_name = "NAME")]
    pub name: String,

    /// Comma-separated tags (e.g., "corne,42-key")
    #[arg(short, long, value_name = "TAGS")]
    pub tags: Option<String>,
}

/// Apply a template to create a new layout
#[derive(Debug, Clone, Args)]
pub struct ApplyArgs {
    /// Template name
    #[arg(short, long, value_name = "NAME")]
    pub name: String,

    /// Output file path
    #[arg(short, long, value_name = "FILE")]
    pub out: PathBuf,
}

/// Template metadata for JSON output
#[derive(Debug, Clone, Serialize)]
pub struct TemplateInfo {
    /// Template name
    pub name: String,
    /// Template filename
    pub file: String,
    /// Template tags
    pub tags: Vec<String>,
    /// Template author
    pub author: String,
    /// Creation timestamp (RFC 3339)
    pub created: String,
}

/// Template list response
#[derive(Debug, Clone, Serialize)]
pub struct TemplateListResponse {
    /// List of templates
    pub templates: Vec<TemplateInfo>,
    /// Total number of templates
    pub count: usize,
}

impl TemplateArgs {
    /// Execute the template command
    pub fn execute(&self) -> CliResult<()> {
        match &self.command {
            TemplateCommand::List(args) => args.execute(),
            TemplateCommand::Save(args) => args.execute(),
            TemplateCommand::Apply(args) => args.execute(),
        }
    }
}

impl ListArgs {
    /// Execute the list command
    pub fn execute(&self) -> CliResult<()> {
        let template_dir = get_template_dir()?;

        // Create template directory if it doesn't exist
        if !template_dir.exists() {
            fs::create_dir_all(&template_dir)
                .map_err(|e| CliError::io(format!("Failed to create template directory: {e}")))?;
        }

        // Scan for .md files
        let entries = fs::read_dir(&template_dir)
            .map_err(|e| CliError::io(format!("Failed to read template directory: {e}")))?;

        let mut templates = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                // Try to parse the layout to extract metadata
                match parse_markdown_layout(&path) {
                    Ok(layout) => {
                        let file_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown.md")
                            .to_string();

                        templates.push(TemplateInfo {
                            name: layout.metadata.name.clone(),
                            file: file_name,
                            tags: layout.metadata.tags.clone(),
                            author: layout.metadata.author.clone(),
                            created: layout.metadata.created.to_rfc3339(),
                        });
                    }
                    Err(_) => {
                        // Skip files that can't be parsed
                    }
                }
            }
        }

        // Sort by name
        templates.sort_by(|a, b| a.name.cmp(&b.name));

        let count = templates.len();
        let response = TemplateListResponse { templates, count };

        // Output
        if self.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&response)
                    .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?
            );
        } else {
            // Human-readable output
            if count == 0 {
                println!("No templates found.");
                println!("Template directory: {}", template_dir.display());
            } else {
                println!("Available templates ({}):\n", count);
                for template in &response.templates {
                    println!("  {} ({})", template.name, template.file);
                    if !template.tags.is_empty() {
                        println!("    Tags: {}", template.tags.join(", "));
                    }
                    if !template.author.is_empty() {
                        println!("    Author: {}", template.author);
                    }
                    println!();
                }
                println!("Template directory: {}", template_dir.display());
            }
        }

        Ok(())
    }
}

impl SaveArgs {
    /// Execute the save command
    pub fn execute(&self) -> CliResult<()> {
        // Validate layout file exists and is readable
        if !self.layout.exists() {
            return Err(CliError::validation(format!(
                "Layout file not found: {}",
                self.layout.display()
            )));
        }

        // Load the layout
        let mut layout = LayoutService::load(&self.layout)
            .map_err(|e| CliError::io(format!("Failed to load layout: {e}")))?;

        // Parse tags if provided
        let tags = if let Some(tag_str) = &self.tags {
            tag_str
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<String>>()
        } else {
            Vec::new()
        };

        // Update metadata
        layout.metadata.name.clone_from(&self.name);
        layout.metadata.tags = tags;
        layout.metadata.is_template = true;
        layout.metadata.modified = Utc::now();

        // Get template directory
        let template_dir = get_template_dir()?;

        // Create template directory if it doesn't exist
        if !template_dir.exists() {
            fs::create_dir_all(&template_dir)
                .map_err(|e| CliError::io(format!("Failed to create template directory: {e}")))?;
        }

        // Generate safe file name from template name
        let file_name = sanitize_filename(&self.name);
        let template_path = template_dir.join(format!("{file_name}.md"));

        // Check if template already exists
        if template_path.exists() {
            return Err(CliError::validation(format!(
                "Template '{}' already exists: {}",
                self.name,
                template_path.display()
            )));
        }

        // Save the template
        save_markdown_layout(&layout, &template_path)
            .map_err(|e| CliError::io(format!("Failed to save template: {e}")))?;

        println!("✓ Template saved: {}", self.name);
        println!("  File: {}", template_path.display());

        Ok(())
    }
}

impl ApplyArgs {
    /// Execute the apply command
    pub fn execute(&self) -> CliResult<()> {
        // Check if output file already exists
        if self.out.exists() {
            return Err(CliError::validation(format!(
                "Output file already exists: {}",
                self.out.display()
            )));
        }

        // Get template directory
        let template_dir = get_template_dir()?;

        // Find template by name
        let entries = fs::read_dir(&template_dir)
            .map_err(|e| CliError::io(format!("Failed to read template directory: {e}")))?;

        let mut template_path: Option<PathBuf> = None;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                // Try to parse and check name
                if let Ok(layout) = parse_markdown_layout(&path) {
                    if layout.metadata.name == self.name {
                        template_path = Some(path);
                        break;
                    }
                }
            }
        }

        let template_path = template_path
            .ok_or_else(|| CliError::validation(format!("Template '{}' not found", self.name)))?;

        // Load the template
        let mut layout = LayoutService::load(&template_path)
            .map_err(|e| CliError::io(format!("Failed to load template: {e}")))?;

        // Update metadata for new layout
        layout.metadata.is_template = false;
        layout.metadata.created = Utc::now();
        layout.metadata.modified = Utc::now();

        // Save to output file
        save_markdown_layout(&layout, &self.out)
            .map_err(|e| CliError::io(format!("Failed to save layout: {e}")))?;

        println!("✓ Template applied: {}", self.name);
        println!("  Output: {}", self.out.display());

        Ok(())
    }
}

/// Get the platform-specific template directory
fn get_template_dir() -> CliResult<PathBuf> {
    let config_dir = Config::config_dir()
        .map_err(|e| CliError::io(format!("Failed to get config directory: {e}")))?;
    Ok(config_dir.join("templates"))
}

/// Sanitize a string to be a valid filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() {
                '_'
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("My Layout"), "my_layout");
        assert_eq!(sanitize_filename("Corne Base"), "corne_base");
        assert_eq!(sanitize_filename("test-123"), "test-123");
        assert_eq!(
            sanitize_filename("Special!@#$%Characters"),
            "special_____characters"
        );
    }
}
