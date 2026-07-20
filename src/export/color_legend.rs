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
/// 2. Color reference table mapping \[1\], \[2\], etc. to hex colors and sources
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
/// Returns a `BTreeMap` of colors to their sources, ordered by color value (for consistency).
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
mod tests;
