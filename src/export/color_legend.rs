//! Color legend generator for layout exports.
//!
//! Generates a markdown section documenting the color system used in a keyboard layout,
//! including the color priority system, a reference table mapping numbers to colors,
//! and a full category listing.

use crate::models::{Layout, RgbColor};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Generates a markdown color legend section for a layout export.
///
/// Creates a comprehensive color documentation that includes:
/// 1. Color priority explanation (individual > category > layer > default)
/// 2. Color reference table mapping [1], [2], etc. to hex colors and sources
/// 3. Full category listing with colors and descriptions
///
/// # Examples
///
/// ```no_run
/// use lazyqmk::models::Layout;
/// use lazyqmk::export::color_legend::generate_color_legend;
///
/// let layout = Layout::new("My Layout").unwrap();
/// let legend = generate_color_legend(&layout);
/// println!("{}", legend);
/// ```
pub fn generate_color_legend(layout: &Layout) -> String {
    let mut output = String::new();

    // Header
    output.push_str("## Color System\n\n");

    // Color Priority Section
    output.push_str("### Color Priority\n\n");
    output.push_str("1. **Individual Key Color** (highest) - Manually assigned to specific key\n");
    output.push_str("2. **Key Category Color** - Color from assigned category\n");
    output.push_str("3. **Layer Category Color** - Category assigned to entire layer\n");
    output.push_str("4. **Layer Default Color** (lowest) - Fallback color for layer\n\n");

    // Collect all unique colors and their sources from the layout
    let color_sources = collect_color_sources(layout);

    if !color_sources.is_empty() {
        // Color Reference Table
        output.push_str("### Color Reference\n\n");

        for (index, (color, sources)) in color_sources.iter().enumerate() {
            let color_hex = color.to_hex();
            let color_num = index + 1;

            // Format: [N] #RRGGBB - source description
            let _ = write!(output, "[{}] {} - ", color_num, color_hex);

            if sources.len() == 1 {
                let source = &sources[0];
                output.push_str(source);
            } else {
                // Multiple sources of same color
                output.push_str(&sources.join(", "));
            }

            output.push('\n');
        }

        output.push('\n');
    }

    // Categories Section
    if !layout.categories.is_empty() {
        output.push_str("### Categories\n\n");

        for category in &layout.categories {
            let color_hex = category.color.to_hex();
            let _ = writeln!(
                output,
                "- **{}** ({}) - {}",
                category.id, color_hex, category.name
            );
        }

        output.push('\n');
    }

    output
}

/// Collects all unique colors and their sources from a layout.
///
/// Returns a BTreeMap of colors to their sources, ordered by color value (for consistency).
/// Each source describes where the color comes from (category ID or "layer default").
fn collect_color_sources(layout: &Layout) -> BTreeMap<RgbColor, Vec<String>> {
    let mut sources: BTreeMap<RgbColor, Vec<String>> = BTreeMap::new();

    // Collect colors from categories
    for category in &layout.categories {
        sources
            .entry(category.color)
            .or_default()
            .push(format!("{} (category)", category.id));
    }

    // Collect colors from layer defaults
    for layer in &layout.layers {
        let source = format!("layer {} default", layer.number);
        sources.entry(layer.default_color).or_default().push(source);
    }

    // Collect colors from individual key overrides
    for layer in &layout.layers {
        for key in &layer.keys {
            if let Some(color) = key.color_override {
                let source = format!("{} (individual)", key.keycode);
                sources.entry(color).or_default().push(source);
            }
        }
    }

    // Deduplicate and sort sources for each color
    for sources_list in sources.values_mut() {
        sources_list.sort();
        sources_list.dedup();
    }

    sources
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Category, KeyDefinition, Layer, Position};

    #[test]
    fn test_generate_color_legend_basic() {
        let layout = Layout::new("Test Layout").unwrap();
        let legend = generate_color_legend(&layout);

        assert!(legend.contains("## Color System"));
        assert!(legend.contains("### Color Priority"));
        assert!(legend.contains("Individual Key Color"));
        assert!(legend.contains("Key Category Color"));
        assert!(legend.contains("Layer Category Color"));
        assert!(legend.contains("Layer Default Color"));
    }

    #[test]
    fn test_generate_color_legend_with_categories() {
        let mut layout = Layout::new("Test Layout").unwrap();

        let nav_category =
            Category::new("navigation", "Navigation Keys", RgbColor::new(0, 255, 0)).unwrap();
        let delete_category =
            Category::new("delete", "Delete Keys", RgbColor::new(255, 0, 0)).unwrap();

        layout.add_category(nav_category).unwrap();
        layout.add_category(delete_category).unwrap();

        let legend = generate_color_legend(&layout);

        assert!(legend.contains("### Categories"));
        assert!(legend.contains("**navigation**"));
        assert!(legend.contains("Navigation Keys"));
        assert!(legend.contains("**delete**"));
        assert!(legend.contains("Delete Keys"));
        assert!(legend.contains("#00FF00"));
        assert!(legend.contains("#FF0000"));
    }

    #[test]
    fn test_generate_color_legend_with_colors() {
        let mut layout = Layout::new("Test Layout").unwrap();

        // Add a layer with default color
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();

        // Add a key with color override
        let key =
            KeyDefinition::new(Position::new(0, 0), "KC_A").with_color(RgbColor::new(255, 0, 0));
        layer.add_key(key);

        layout.add_layer(layer).unwrap();

        let legend = generate_color_legend(&layout);

        assert!(legend.contains("### Color Reference"));
        assert!(legend.contains("#FF0000")); // Individual key color
        assert!(legend.contains("#FFFFFF")); // Layer default color
    }

    #[test]
    fn test_color_sources_deduplication() {
        let mut layout = Layout::new("Test Layout").unwrap();

        // Create a category
        let nav_category =
            Category::new("navigation", "Navigation Keys", RgbColor::new(0, 255, 0)).unwrap();
        layout.add_category(nav_category).unwrap();

        // Create layers with same default color
        let layer0 = Layer::new(0, "Base", RgbColor::new(0, 255, 0)).unwrap();
        let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();

        layout.add_layer(layer0).unwrap();
        layout.add_layer(layer1).unwrap();

        let sources = collect_color_sources(&layout);

        // Green color should appear with multiple sources
        let green = RgbColor::new(0, 255, 0);
        assert!(sources.contains_key(&green));

        let green_sources = &sources[&green];
        assert!(green_sources.iter().any(|s| s.contains("navigation")));
        assert!(green_sources.iter().any(|s| s.contains("layer 0 default")));
        assert!(green_sources.iter().any(|s| s.contains("layer 1 default")));
    }

    #[test]
    fn test_color_sources_individual_override() {
        let mut layout = Layout::new("Test Layout").unwrap();

        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();

        let key1 =
            KeyDefinition::new(Position::new(0, 0), "KC_A").with_color(RgbColor::new(255, 0, 0));
        let key2 =
            KeyDefinition::new(Position::new(0, 1), "KC_B").with_color(RgbColor::new(255, 0, 0));

        layer.add_key(key1);
        layer.add_key(key2);

        layout.add_layer(layer).unwrap();

        let sources = collect_color_sources(&layout);
        let red = RgbColor::new(255, 0, 0);

        assert!(sources.contains_key(&red));
        let red_sources = &sources[&red];

        assert_eq!(red_sources.len(), 2);
        assert!(red_sources.iter().any(|s| s.contains("KC_A")));
        assert!(red_sources.iter().any(|s| s.contains("KC_B")));
    }

    #[test]
    fn test_color_legend_format() {
        let mut layout = Layout::new("Test Layout").unwrap();

        let category =
            Category::new("test-cat", "Test Category", RgbColor::new(100, 150, 200)).unwrap();
        layout.add_category(category).unwrap();

        let legend = generate_color_legend(&layout);

        // Verify format includes [N] #RRGGBB pattern
        assert!(legend.contains("[1] #"));
        // Verify hex color is present
        assert!(legend.contains("#6496C8")); // hex for RGB(100, 150, 200)
    }

    #[test]
    fn test_generate_color_legend_empty_layout() {
        let layout = Layout::new("Empty Layout").unwrap();
        let legend = generate_color_legend(&layout);

        // Should still have header and priority section
        assert!(legend.contains("## Color System"));
        assert!(legend.contains("### Color Priority"));
        // But no categories or references
        assert!(!legend.contains("### Color Reference"));
        assert!(!legend.contains("### Categories"));
    }
}
