//! Tests for color_legend.
//!
//! Auto-extracted from color_legend.rs.

use super::*;

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
