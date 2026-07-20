//! Category system for organizing keys by logical function.

use crate::models::RgbColor;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// User-defined category for organizing keys.
///
/// Categories provide a way to group keys by logical function (e.g., "navigation",
/// "symbols", "media") and assign a color to all keys in that category.
///
/// # Validation
///
/// - ID must be unique within a Layout
/// - ID format: kebab-case (lowercase, hyphens only, no spaces)
/// - Name must be non-empty, max 50 characters
/// - Color must be valid RGB (enforced by `RgbColor` type)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    /// Unique identifier in kebab-case (e.g., "navigation", "symbols")
    pub id: String,
    /// Display name (e.g., "Navigation", "Symbols")
    pub name: String,
    /// RGB color for visual identification
    pub color: RgbColor,
}

impl Category {
    /// Creates a new Category with validation.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazyqmk::models::{Category, RgbColor};
    ///
    /// let category = Category::new(
    ///     "navigation",
    ///     "Navigation Keys",
    ///     RgbColor::from_hex("#00FF00").unwrap()
    /// ).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - ID is empty or not in kebab-case format
    /// - Name is empty or exceeds 50 characters
    pub fn new(id: impl Into<String>, name: impl Into<String>, color: RgbColor) -> Result<Self> {
        let id = id.into();
        let name = name.into();

        Self::validate_id(&id)?;
        Self::validate_name(&name)?;

        Ok(Self { id, name, color })
    }

    /// Validates category ID format (kebab-case).
    fn validate_id(id: &str) -> Result<()> {
        if id.is_empty() {
            anyhow::bail!("Category ID cannot be empty");
        }

        if !id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            anyhow::bail!(
                "Category ID '{id}' must be kebab-case (lowercase, hyphens, and digits only)"
            );
        }

        if id.starts_with('-') || id.ends_with('-') {
            anyhow::bail!("Category ID '{id}' cannot start or end with a hyphen");
        }

        Ok(())
    }

    /// Validates category name.
    fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() {
            anyhow::bail!("Category name cannot be empty");
        }

        if name.len() > 50 {
            anyhow::bail!(
                "Category name '{}' exceeds maximum length of 50 characters (got {})",
                name,
                name.len()
            );
        }

        Ok(())
    }

    /// Updates the category color.
    pub const fn set_color(&mut self, color: RgbColor) {
        self.color = color;
    }

    /// Updates the category name with validation.
    pub fn set_name(&mut self, name: impl Into<String>) -> Result<()> {
        let name = name.into();
        Self::validate_name(&name)?;
        self.name = name;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
