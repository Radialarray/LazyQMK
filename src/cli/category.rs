//! Category management commands for layouts.
//!
//! Provides commands to list, add, and delete categories in a layout file.

use crate::cli::common::{CliError, CliResult};
use crate::models::{Category, RgbColor};
use crate::services::LayoutService;
use clap::{Args, Subcommand};
use regex::Regex;
use serde::Serialize;
use std::path::PathBuf;

/// Manage categories in a layout
#[derive(Debug, Clone, Args)]
pub struct CategoryArgs {
    /// Category subcommand
    #[command(subcommand)]
    pub command: CategoryCommand,
}

/// Category management subcommands
#[derive(Debug, Clone, Subcommand)]
pub enum CategoryCommand {
    /// List all categories in a layout
    List(ListCategoriesArgs),
    /// Add a new category to a layout
    Add(AddCategoryArgs),
    /// Remove a category from a layout
    Delete(DeleteCategoryArgs),
}

/// List all categories in a layout
#[derive(Debug, Clone, Args)]
pub struct ListCategoriesArgs {
    /// Path to layout markdown file
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,
}

/// Add a new category to a layout
#[derive(Debug, Clone, Args)]
pub struct AddCategoryArgs {
    /// Path to layout markdown file
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,

    /// Category ID (kebab-case)
    #[arg(long, value_name = "ID")]
    pub id: String,

    /// Category name
    #[arg(long, value_name = "NAME")]
    pub name: String,

    /// Color in hex format (#RRGGBB or #RGB)
    #[arg(long, value_name = "HEX")]
    pub color: String,
}

/// Delete a category from a layout
#[derive(Debug, Clone, Args)]
pub struct DeleteCategoryArgs {
    /// Path to layout markdown file
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,

    /// Category ID to delete
    #[arg(long, value_name = "ID")]
    pub id: String,

    /// Force deletion even if category is in use
    #[arg(long)]
    pub force: bool,
}

// JSON response types
#[derive(Debug, Serialize)]
struct CategoryItem {
    id: String,
    name: String,
    color: String,
}

#[derive(Debug, Serialize)]
struct ListCategoriesResponse {
    categories: Vec<CategoryItem>,
    count: usize,
}

impl CategoryArgs {
    /// Execute the category command
    pub fn execute(&self) -> CliResult<()> {
        match &self.command {
            CategoryCommand::List(args) => args.execute(),
            CategoryCommand::Add(args) => args.execute(),
            CategoryCommand::Delete(args) => args.execute(),
        }
    }
}

impl ListCategoriesArgs {
    /// Execute the list command
    pub fn execute(&self) -> CliResult<()> {
        // Load layout
        let layout = LayoutService::load(&self.layout)
            .map_err(|e| CliError::io(format!("Failed to load layout: {e}")))?;

        // Convert categories to response format
        let categories = layout
            .categories
            .iter()
            .map(|cat| CategoryItem {
                id: cat.id.clone(),
                name: cat.name.clone(),
                color: cat.color.to_hex(),
            })
            .collect();

        let response = ListCategoriesResponse {
            count: layout.categories.len(),
            categories,
        };

        if self.json {
            println!(
                "{}",
                serde_json::to_string(&response)
                    .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?
            );
        } else if response.count == 0 {
            println!("No categories defined.");
        } else {
            println!("Categories ({}):", response.count);
            println!();
            for cat in response.categories {
                println!("  {:<20} {:<30} {}", cat.id, cat.name, cat.color);
            }
        }

        Ok(())
    }
}

impl AddCategoryArgs {
    /// Execute the add command
    pub fn execute(&self) -> CliResult<()> {
        // Load layout
        let mut layout = LayoutService::load(&self.layout)
            .map_err(|e| CliError::io(format!("Failed to load layout: {e}")))?;

        // Validate hex color format
        let color = validate_and_parse_hex(&self.color).map_err(CliError::validation)?;

        // Create category with validation
        let category = Category::new(&self.id, &self.name, color)
            .map_err(|e| CliError::validation(format!("Invalid category: {e}")))?;

        // Check if category already exists
        if layout.categories.iter().any(|c| c.id == self.id) {
            return Err(CliError::validation(format!(
                "Category with ID '{}' already exists",
                self.id
            )));
        }

        // Add category
        layout
            .add_category(category)
            .map_err(|e| CliError::validation(format!("Failed to add category: {e}")))?;

        // Save layout
        LayoutService::save(&layout, &self.layout)
            .map_err(|e| CliError::io(format!("Failed to save layout: {e}")))?;

        println!("Category '{}' added successfully.", self.id);
        Ok(())
    }
}

impl DeleteCategoryArgs {
    /// Execute the delete command
    pub fn execute(&self) -> CliResult<()> {
        // Load layout
        let mut layout = LayoutService::load(&self.layout)
            .map_err(|e| CliError::io(format!("Failed to load layout: {e}")))?;

        // Check if category exists
        if !layout.categories.iter().any(|c| c.id == self.id) {
            return Err(CliError::validation(format!(
                "Category '{}' not found",
                self.id
            )));
        }

        // Check if category is in use (unless --force)
        if !self.force {
            if category_is_in_use(&layout, &self.id) {
                return Err(CliError::validation(format!(
                    "Category '{}' is in use by keys or layers. Use --force to delete anyway.",
                    self.id
                )));
            }
        } else {
            // Clear references from keys and layers
            clear_category_references(&mut layout, &self.id);
        }

        // Remove category
        layout.remove_category(&self.id);

        // Save layout
        LayoutService::save(&layout, &self.layout)
            .map_err(|e| CliError::io(format!("Failed to save layout: {e}")))?;

        println!("Category '{}' deleted successfully.", self.id);
        Ok(())
    }
}

/// Validates hex color format (#RRGGBB or #RGB) and returns RgbColor
fn validate_and_parse_hex(color: &str) -> Result<RgbColor, String> {
    // Match #RRGGBB or #RGB format
    let hex_regex = Regex::new(r"^#([0-9A-Fa-f]{6}|[0-9A-Fa-f]{3})$")
        .map_err(|_| "Failed to create hex regex".to_string())?;

    if !hex_regex.is_match(color) {
        return Err(format!(
            "Invalid hex color format: '{}'. Expected #RRGGBB or #RGB",
            color
        ));
    }

    // Expand short hex format (#RGB -> #RRGGBB)
    let expanded_color = if color.len() == 4 {
        // #RGB format - expand each digit
        let hex = &color[1..]; // remove #
        format!(
            "#{}{}{}{}{}{}",
            &hex[0..1],
            &hex[0..1],
            &hex[1..2],
            &hex[1..2],
            &hex[2..3],
            &hex[2..3]
        )
    } else {
        color.to_string()
    };

    RgbColor::from_hex(&expanded_color).map_err(|e| format!("Failed to parse color: {e}"))
}

/// Checks if a category is referenced by any key or layer
fn category_is_in_use(layout: &crate::models::Layout, category_id: &str) -> bool {
    // Check if any layer uses this category
    for layer in &layout.layers {
        if let Some(ref cat_id) = layer.category_id {
            if cat_id == category_id {
                return true;
            }
        }

        // Check if any key in this layer uses this category
        for key in &layer.keys {
            if let Some(ref cat_id) = key.category_id {
                if cat_id == category_id {
                    return true;
                }
            }
        }
    }

    false
}

/// Removes all references to a category from keys and layers
fn clear_category_references(layout: &mut crate::models::Layout, category_id: &str) {
    for layer in &mut layout.layers {
        // Clear layer category if it matches
        if let Some(ref cat_id) = layer.category_id {
            if cat_id == category_id {
                layer.category_id = None;
            }
        }

        // Clear key categories if they match
        for key in &mut layer.keys {
            if let Some(ref cat_id) = key.category_id {
                if cat_id == category_id {
                    key.category_id = None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_hex_color_valid_long() {
        let result = validate_and_parse_hex("#FF0000");
        assert!(result.is_ok());
        let color = result.unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_validate_hex_color_valid_short() {
        let result = validate_and_parse_hex("#F0F");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_hex_color_invalid_format() {
        assert!(validate_and_parse_hex("FF0000").is_err());
        assert!(validate_and_parse_hex("#FF00").is_err());
        assert!(validate_and_parse_hex("#GG0000").is_err());
    }

    #[test]
    fn test_category_is_in_use_by_key() {
        use crate::models::{KeyDefinition, Layer, Layout, Position};

        let mut layout = Layout::new("Test").unwrap();
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();

        let mut key = KeyDefinition::new(Position::new(0, 0), "KC_A");
        key = key.with_category("navigation");
        layer.add_key(key);

        layout.add_layer(layer).unwrap();

        assert!(category_is_in_use(&layout, "navigation"));
        assert!(!category_is_in_use(&layout, "symbols"));
    }

    #[test]
    fn test_category_is_in_use_by_layer() {
        use crate::models::{Layer, Layout};

        let mut layout = Layout::new("Test").unwrap();
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();
        layer.category_id = Some("navigation".to_string());

        layout.add_layer(layer).unwrap();

        assert!(category_is_in_use(&layout, "navigation"));
        assert!(!category_is_in_use(&layout, "symbols"));
    }
}
