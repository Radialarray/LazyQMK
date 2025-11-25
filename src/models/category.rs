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
    /// use keyboard_tui::models::{Category, RgbColor};
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
mod tests {
    use super::*;

    #[test]
    fn test_new_valid() {
        let color = RgbColor::new(0, 255, 0);
        let category = Category::new("navigation", "Navigation Keys", color).unwrap();

        assert_eq!(category.id, "navigation");
        assert_eq!(category.name, "Navigation Keys");
        assert_eq!(category.color, color);
    }

    #[test]
    fn test_validate_id_valid() {
        assert!(Category::validate_id("navigation").is_ok());
        assert!(Category::validate_id("media-keys").is_ok());
        assert!(Category::validate_id("layer-1").is_ok());
        assert!(Category::validate_id("f-keys").is_ok());
    }

    #[test]
    fn test_validate_id_invalid() {
        assert!(Category::validate_id("").is_err());
        assert!(Category::validate_id("Navigation").is_err()); // uppercase
        assert!(Category::validate_id("media keys").is_err()); // space
        assert!(Category::validate_id("media_keys").is_err()); // underscore
        assert!(Category::validate_id("-navigation").is_err()); // starts with hyphen
        assert!(Category::validate_id("navigation-").is_err()); // ends with hyphen
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(Category::validate_name("Navigation").is_ok());
        assert!(Category::validate_name("Media Keys").is_ok());
        assert!(Category::validate_name("A").is_ok());
        assert!(Category::validate_name("This is a valid name with 50 chars exactly!!").is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        assert!(Category::validate_name("").is_err());
        assert!(Category::validate_name(&"a".repeat(51)).is_err());
    }

    #[test]
    fn test_set_color() {
        let mut category = Category::new("test", "Test", RgbColor::new(255, 0, 0)).unwrap();
        let new_color = RgbColor::new(0, 255, 0);
        category.set_color(new_color);
        assert_eq!(category.color, new_color);
    }

    #[test]
    fn test_set_name() {
        let mut category = Category::new("test", "Test", RgbColor::new(255, 0, 0)).unwrap();
        category.set_name("New Name").unwrap();
        assert_eq!(category.name, "New Name");

        assert!(category.set_name("").is_err());
        assert!(category.set_name("a".repeat(51)).is_err());
    }
}
