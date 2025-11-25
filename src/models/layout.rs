//! Layout and metadata data structures.

#[cfg(test)]
use crate::models::layer::Position;
use crate::models::layer::{KeyDefinition, Layer};
use crate::models::{Category, RgbColor};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// File metadata embedded in YAML frontmatter.
///
/// # Validation
///
/// - name must be non-empty, max 100 characters
/// - created must be <= modified
/// - tags must be lowercase, hyphen/alphanumeric only
/// - version must match supported versions (currently "1.0")
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutMetadata {
    /// Layout name (e.g., "My Corne Layout")
    pub name: String,
    /// Long description
    pub description: String,
    /// Creator name
    pub author: String,
    /// Creation timestamp (ISO 8601)
    pub created: DateTime<Utc>,
    /// Last modification timestamp (ISO 8601)
    pub modified: DateTime<Utc>,
    /// Searchable keywords
    pub tags: Vec<String>,
    /// Template flag (saves to templates/ directory)
    pub is_template: bool,
    /// Schema version (e.g., "1.0")
    pub version: String,
}

#[allow(dead_code)]
impl LayoutMetadata {
    /// Creates new metadata with default values.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        Self::validate_name(&name)?;

        let now = Utc::now();
        Ok(Self {
            name,
            description: String::new(),
            author: String::new(),
            created: now,
            modified: now,
            tags: Vec::new(),
            is_template: false,
            version: "1.0".to_string(),
        })
    }

    /// Validates metadata name.
    fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() {
            anyhow::bail!("Layout name cannot be empty");
        }

        if name.len() > 100 {
            anyhow::bail!(
                "Layout name '{}' exceeds maximum length of 100 characters (got {})",
                name,
                name.len()
            );
        }

        Ok(())
    }

    /// Updates the modification timestamp to now.
    pub fn touch(&mut self) {
        self.modified = Utc::now();
    }

    /// Sets the description.
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = description.into();
        self.touch();
    }

    /// Sets the author.
    pub fn set_author(&mut self, author: impl Into<String>) {
        self.author = author.into();
        self.touch();
    }

    /// Adds a tag with validation.
    pub fn add_tag(&mut self, tag: impl Into<String>) -> Result<()> {
        let tag = tag.into();
        Self::validate_tag(&tag)?;

        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.touch();
        }

        Ok(())
    }

    /// Validates tag format (lowercase, hyphens, alphanumeric).
    fn validate_tag(tag: &str) -> Result<()> {
        if tag.is_empty() {
            anyhow::bail!("Tag cannot be empty");
        }

        if !tag
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            anyhow::bail!(
                "Tag '{tag}' must be lowercase with hyphens and alphanumeric characters only"
            );
        }

        Ok(())
    }
}

impl Default for LayoutMetadata {
    fn default() -> Self {
        Self::new("Untitled Layout").unwrap()
    }
}

/// Complete keyboard mapping with metadata and multiple layers.
///
/// # Validation
///
/// - At least one layer required (layer 0)
/// - Layer numbers must be sequential without gaps
/// - All layers must have same number of keys (determined by keyboard layout)
/// - Category IDs must be unique within layout
///
/// # Color Resolution
///
/// The Layout provides a four-level color priority system:
/// 1. `KeyDefinition.color_override` (highest)
/// 2. `KeyDefinition.category_id` → Category.color
/// 3. `Layer.category_id` → Category.color
/// 4. `Layer.default_color` (lowest/fallback)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// File metadata
    pub metadata: LayoutMetadata,
    /// Ordered list of layers (0-N, typically 0-11)
    pub layers: Vec<Layer>,
    /// User-defined categories for organization
    pub categories: Vec<Category>,
}

#[allow(dead_code)]
impl Layout {
    /// Creates a new Layout with default metadata.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let metadata = LayoutMetadata::new(name)?;
        Ok(Self {
            metadata,
            layers: Vec::new(),
            categories: Vec::new(),
        })
    }

    /// Adds a layer to this layout.
    pub fn add_layer(&mut self, layer: Layer) -> Result<()> {
        // Validate sequential layer numbers
        if !self.layers.is_empty() {
            let expected_number = self.layers.len() as u8;
            if layer.number != expected_number {
                anyhow::bail!(
                    "Layer numbers must be sequential. Expected layer {}, got {}",
                    expected_number,
                    layer.number
                );
            }
        } else if layer.number != 0 {
            anyhow::bail!("First layer must have number 0, got {}", layer.number);
        }

        self.layers.push(layer);
        self.metadata.touch();
        Ok(())
    }

    /// Gets a reference to the layer at the given index.
    #[must_use] pub fn get_layer(&self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }

    /// Gets a mutable reference to the layer at the given index.
    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.metadata.touch();
        self.layers.get_mut(index)
    }

    /// Adds a category to this layout.
    pub fn add_category(&mut self, category: Category) -> Result<()> {
        // Check for duplicate ID
        if self.categories.iter().any(|c| c.id == category.id) {
            anyhow::bail!("Category with ID '{}' already exists", category.id);
        }

        self.categories.push(category);
        self.metadata.touch();
        Ok(())
    }

    /// Gets a category by ID.
    #[must_use] pub fn get_category(&self, id: &str) -> Option<&Category> {
        self.categories.iter().find(|c| c.id == id)
    }

    /// Gets a mutable reference to a category by ID.
    pub fn get_category_mut(&mut self, id: &str) -> Option<&mut Category> {
        self.metadata.touch();
        self.categories.iter_mut().find(|c| c.id == id)
    }

    /// Removes a category by ID.
    pub fn remove_category(&mut self, id: &str) -> Option<Category> {
        if let Some(index) = self.categories.iter().position(|c| c.id == id) {
            self.metadata.touch();
            Some(self.categories.remove(index))
        } else {
            None
        }
    }

    /// Resolves the color for a key using the four-level priority system.
    ///
    /// Priority (highest to lowest):
    /// 1. `KeyDefinition.color_override`
    /// 2. `KeyDefinition.category_id` → Category.color
    /// 3. `Layer.category_id` → Category.color
    /// 4. `Layer.default_color` (fallback)
    ///
    /// # Examples
    ///
    /// ```
    /// use keyboard_tui::models::{Layout, Layer, KeyDefinition, Category, Position, RgbColor};
    ///
    /// let mut layout = Layout::new("Test").unwrap();
    /// let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();
    ///
    /// let key = KeyDefinition::new(Position::new(0, 0), "KC_A")
    ///     .with_color(RgbColor::new(255, 0, 0));
    ///
    /// layer.add_key(key.clone());
    /// layout.add_layer(layer).unwrap();
    ///
    /// let color = layout.resolve_key_color(0, &key);
    /// assert_eq!(color, RgbColor::new(255, 0, 0)); // Individual override
    /// ```
    #[must_use] pub fn resolve_key_color(&self, layer_idx: usize, key: &KeyDefinition) -> RgbColor {
        // 1. Individual key color override (highest priority)
        if let Some(color) = key.color_override {
            return color;
        }

        // 2. Key category color
        if let Some(cat_id) = &key.category_id {
            if let Some(category) = self.get_category(cat_id) {
                return category.color;
            }
        }

        // 3. Layer category color
        if let Some(layer) = self.get_layer(layer_idx) {
            if let Some(cat_id) = &layer.category_id {
                if let Some(category) = self.get_category(cat_id) {
                    return category.color;
                }
            }

            // 4. Layer default color (fallback)
            return layer.default_color;
        }

        // Fallback to white if layer doesn't exist (shouldn't happen)
        RgbColor::default()
    }

    /// Validates the layout structure.
    ///
    /// Checks:
    /// - At least one layer exists
    /// - All layers have the same number of keys
    /// - No duplicate positions within each layer
    /// - All category references exist
    pub fn validate(&self) -> Result<()> {
        if self.layers.is_empty() {
            anyhow::bail!("Layout must have at least one layer");
        }

        // Check layer numbers are sequential
        for (idx, layer) in self.layers.iter().enumerate() {
            if layer.number != idx as u8 {
                anyhow::bail!(
                    "Layer numbers must be sequential. Layer at index {} has number {}",
                    idx,
                    layer.number
                );
            }
        }

        // Check all layers have same number of keys
        if let Some(first_layer) = self.layers.first() {
            let expected_key_count = first_layer.keys.len();
            for layer in &self.layers {
                if layer.keys.len() != expected_key_count {
                    anyhow::bail!(
                        "All layers must have the same number of keys. Layer {} has {}, expected {}",
                        layer.number,
                        layer.keys.len(),
                        expected_key_count
                    );
                }
            }
        }

        // Check for duplicate positions within each layer
        for layer in &self.layers {
            let mut positions = std::collections::HashSet::new();
            for key in &layer.keys {
                if !positions.insert(key.position) {
                    anyhow::bail!(
                        "Duplicate position ({}, {}) in layer {}",
                        key.position.row,
                        key.position.col,
                        layer.number
                    );
                }
            }
        }

        // Validate category references
        for layer in &self.layers {
            if let Some(cat_id) = &layer.category_id {
                if !self.categories.iter().any(|c| &c.id == cat_id) {
                    anyhow::bail!(
                        "Layer {} references non-existent category '{}'",
                        layer.number,
                        cat_id
                    );
                }
            }

            for key in &layer.keys {
                if let Some(cat_id) = &key.category_id {
                    if !self.categories.iter().any(|c| &c.id == cat_id) {
                        anyhow::bail!(
                            "Key at ({}, {}) in layer {} references non-existent category '{}'",
                            key.position.row,
                            key.position.col,
                            layer.number,
                            cat_id
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_metadata_new() {
        let metadata = LayoutMetadata::new("Test Layout").unwrap();
        assert_eq!(metadata.name, "Test Layout");
        assert!(metadata.description.is_empty());
        assert!(metadata.author.is_empty());
        assert!(metadata.tags.is_empty());
        assert!(!metadata.is_template);
        assert_eq!(metadata.version, "1.0");
    }

    #[test]
    fn test_layout_metadata_validate_name() {
        assert!(LayoutMetadata::new("Valid Name").is_ok());
        assert!(LayoutMetadata::new("").is_err());
        assert!(LayoutMetadata::new("a".repeat(101)).is_err());
    }

    #[test]
    fn test_layout_metadata_add_tag() {
        let mut metadata = LayoutMetadata::new("Test").unwrap();
        metadata.add_tag("programming").unwrap();
        metadata.add_tag("vim").unwrap();

        assert_eq!(metadata.tags, vec!["programming", "vim"]);

        // Duplicate tag should not be added
        metadata.add_tag("programming").unwrap();
        assert_eq!(metadata.tags, vec!["programming", "vim"]);
    }

    #[test]
    fn test_layout_new() {
        let layout = Layout::new("Test Layout").unwrap();
        assert_eq!(layout.metadata.name, "Test Layout");
        assert!(layout.layers.is_empty());
        assert!(layout.categories.is_empty());
    }

    #[test]
    fn test_layout_add_layer() {
        let mut layout = Layout::new("Test").unwrap();
        let layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();

        assert!(layout.add_layer(layer0).is_ok());
        assert!(layout.add_layer(layer1).is_ok());
        assert_eq!(layout.layers.len(), 2);
    }

    #[test]
    fn test_layout_add_layer_sequential_validation() {
        let mut layout = Layout::new("Test").unwrap();
        let layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        let layer2 = Layer::new(2, "Skip", RgbColor::new(0, 255, 0)).unwrap();

        assert!(layout.add_layer(layer0).is_ok());
        assert!(layout.add_layer(layer2).is_err()); // Should fail - not sequential
    }

    #[test]
    fn test_layout_add_category() {
        let mut layout = Layout::new("Test").unwrap();
        let category = Category::new("navigation", "Navigation", RgbColor::new(0, 255, 0)).unwrap();

        assert!(layout.add_category(category).is_ok());
        assert_eq!(layout.categories.len(), 1);
    }

    #[test]
    fn test_layout_add_category_duplicate() {
        let mut layout = Layout::new("Test").unwrap();
        let category1 =
            Category::new("navigation", "Navigation", RgbColor::new(0, 255, 0)).unwrap();
        let category2 = Category::new("navigation", "Nav Keys", RgbColor::new(255, 0, 0)).unwrap();

        assert!(layout.add_category(category1).is_ok());
        assert!(layout.add_category(category2).is_err()); // Duplicate ID
    }

    #[test]
    fn test_layout_resolve_key_color() {
        let mut layout = Layout::new("Test").unwrap();
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();

        // Test 1: Individual override (highest priority)
        let key_with_override =
            KeyDefinition::new(Position::new(0, 0), "KC_A").with_color(RgbColor::new(255, 0, 0));
        layer.add_key(key_with_override.clone());

        layout.add_layer(layer).unwrap();

        let color = layout.resolve_key_color(0, &key_with_override);
        assert_eq!(color, RgbColor::new(255, 0, 0));

        // Test 2: Key category (second priority)
        let category = Category::new("navigation", "Navigation", RgbColor::new(0, 255, 0)).unwrap();
        layout.add_category(category).unwrap();

        let key_with_category =
            KeyDefinition::new(Position::new(0, 1), "KC_B").with_category("navigation");
        layout
            .get_layer_mut(0)
            .unwrap()
            .add_key(key_with_category.clone());

        let color = layout.resolve_key_color(0, &key_with_category);
        assert_eq!(color, RgbColor::new(0, 255, 0));

        // Test 3: Layer default (fallback)
        let key_default = KeyDefinition::new(Position::new(0, 2), "KC_C");
        layout
            .get_layer_mut(0)
            .unwrap()
            .add_key(key_default.clone());

        let color = layout.resolve_key_color(0, &key_default);
        assert_eq!(color, RgbColor::new(255, 255, 255));
    }

    #[test]
    fn test_layout_validate() {
        let mut layout = Layout::new("Test").unwrap();

        // Empty layout should fail
        assert!(layout.validate().is_err());

        // Add a layer with keys
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_A"));
        layer.add_key(KeyDefinition::new(Position::new(0, 1), "KC_B"));
        layout.add_layer(layer).unwrap();

        // Should pass now
        assert!(layout.validate().is_ok());

        // Add another layer with different key count
        let mut layer2 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
        layer2.add_key(KeyDefinition::new(Position::new(0, 0), "KC_1"));
        layout.add_layer(layer2).unwrap();

        // Should fail - mismatched key counts
        assert!(layout.validate().is_err());
    }
}
