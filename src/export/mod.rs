//! Export functionality for keyboard layouts.
//!
//! This module provides tools to export keyboard layout configurations in various formats,
//! currently focused on generating markdown documentation with visual representations
//! and configuration summaries.

use crate::keycode_db::KeycodeDb;
use crate::models::{KeyboardGeometry, Layout};
use anyhow::Result;
use std::fmt::Write as _;

pub mod color_legend;
pub mod keyboard_renderer;
pub mod layer_navigation;
pub mod settings_summary;
pub mod tap_dance_docs;

pub use color_legend::generate_color_legend;
pub use keyboard_renderer::render_layer_diagram;
pub use layer_navigation::generate_layer_navigation;
pub use settings_summary::generate_settings_summary;
pub use tap_dance_docs::generate_tap_dance_docs;

/// Export a complete keyboard layout to markdown format.
///
/// Generates a comprehensive markdown document including:
/// - Header with metadata
/// - Quick reference
/// - Keyboard diagram (base layer)
/// - Layer-by-layer diagrams
/// - Color legend
/// - Layer navigation map
/// - Tap dance documentation
/// - Settings summary
pub fn export_to_markdown(
    layout: &Layout,
    geometry: &KeyboardGeometry,
    keycode_db: &KeycodeDb,
) -> Result<String> {
    let mut output = String::new();

    // 1. Header Section
    generate_header(&mut output, layout);

    // 2. Quick Reference
    generate_quick_reference(&mut output, layout);

    // 3. Keyboard Overview (Base Layer)
    output.push_str("## Keyboard Layout\n\n");
    let base_diagram = render_layer_diagram(layout, 0, geometry)?;
    output.push_str("```\n");
    output.push_str(&base_diagram);
    output.push_str("```\n\n");

    // 4. Layer-by-Layer Diagrams
    for (idx, layer) in layout.layers.iter().enumerate() {
        if idx == 0 {
            continue; // Base layer already shown
        }

        let layer_diagram = render_layer_diagram(layout, idx, geometry)?;

        let _ = writeln!(output, "## Layer {idx}: {}\n", layer.name);
        output.push_str("```\n");
        output.push_str(&layer_diagram);
        output.push_str("```\n\n");

        // Add layer metadata
        let _ = writeln!(
            output,
            "**Default Color:** #{:02X}{:02X}{:02X}\n",
            layer.default_color.r, layer.default_color.g, layer.default_color.b
        );
    }

    // 5. Color Legend
    output.push_str(&generate_color_legend(layout));

    // 6. Layer Navigation Map
    output.push_str(&generate_layer_navigation(layout));

    // 7. Tap Dance Reference
    if !layout.tap_dances.is_empty() {
        output.push_str(&generate_tap_dance_docs(layout, keycode_db));
    }

    // 8. Settings Summary
    output.push_str(&generate_settings_summary(layout));

    Ok(output)
}

/// Generate the document header section
fn generate_header(output: &mut String, layout: &Layout) {
    let metadata = &layout.metadata;

    let _ = writeln!(output, "# {}\n", metadata.name);

    if let Some(keyboard) = &metadata.keyboard {
        let _ = writeln!(output, "**Keyboard:** {keyboard}");
    }

    if let Some(variant) = &metadata.layout_variant {
        let _ = writeln!(output, "**Variant:** {variant}");
    }

    if !metadata.author.is_empty() {
        let _ = writeln!(output, "**Author:** {}", metadata.author);
    }

    let _ = writeln!(
        output,
        "**Created:** {}",
        metadata.created.format("%Y-%m-%d")
    );
    let _ = writeln!(
        output,
        "**Modified:** {}",
        metadata.modified.format("%Y-%m-%d")
    );

    output.push('\n');

    if !metadata.description.is_empty() {
        output.push_str(&metadata.description);
        output.push_str("\n\n");
    }
}

/// Generate the quick reference section
fn generate_quick_reference(output: &mut String, layout: &Layout) {
    output.push_str("## Quick Reference\n\n");

    let _ = writeln!(output, "- **Layers:** {}", layout.layers.len());
    let _ = writeln!(output, "- **Categories:** {}", layout.categories.len());
    let _ = writeln!(output, "- **Tap Dances:** {}", layout.tap_dances.len());
    output.push_str("- **Color System:** Individual > Category > Layer > Default\n\n");
}
