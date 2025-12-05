use anyhow::Result;
use crate::{config, models, parser, tui};

/// Creates a default layout from QMK keyboard info and launches the editor
pub fn launch_editor_with_default_layout(
    config: &config::Config,
    keyboard: &str,
    layout_variant: &str,
    layout_file_name: &str,
) -> Result<()> {
    // Get QMK path from config
    let qmk_path = config
        .paths
        .qmk_firmware
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("QMK firmware path not set in configuration"))?;

    // Get the base keyboard name (without any variant subdirectory)
    // This mirrors the logic in AppState::rebuild_geometry()
    let base_keyboard = tui::AppState::extract_base_keyboard(keyboard);

    // Parse keyboard info.json using the base keyboard path
    let keyboard_info = parser::keyboard_json::parse_keyboard_info_json(qmk_path, &base_keyboard)?;

    // Get the key count for the selected layout to determine the correct variant
    let layout_def = keyboard_info.layouts.get(layout_variant).ok_or_else(|| {
        anyhow::anyhow!(
            "Layout '{}' not found in keyboard info.json",
            layout_variant
        )
    })?;
    let key_count = layout_def.layout.len();

    // Determine the correct keyboard variant based on key count
    // This is critical for split keyboards where different variants have different key counts
    let variant_path = config
        .build
        .determine_keyboard_variant(qmk_path, &base_keyboard, key_count)
        .unwrap_or_else(|_| base_keyboard.clone());

    // Try to get RGB matrix mapping from the variant's keyboard.json
    let matrix_to_led = parser::keyboard_json::parse_variant_keyboard_json(qmk_path, &variant_path)
        .and_then(|variant| variant.rgb_matrix)
        .map(|rgb_config| parser::keyboard_json::build_matrix_to_led_map(&rgb_config));

    // Build geometry from the selected layout with RGB matrix mapping if available
    let geometry = parser::keyboard_json::build_keyboard_geometry_with_rgb(
        &keyboard_info,
        &base_keyboard,
        layout_variant,
        matrix_to_led.as_ref(),
    )?;

    // Build visual mapping
    let mapping = models::VisualLayoutMapping::build(&geometry);

    // Create a default layout with the user-specified name
    let mut layout = models::Layout::new(layout_file_name)?;

    // Sanitize the layout name for use as keymap directory name
    // This must be done BEFORE setting metadata to avoid conflicts with QMK's built-in keymaps
    let sanitized_name = layout_file_name
        .replace('/', "_")
        .replace('\\', "_")
        .replace(':', "_")
        .replace(' ', "_")
        .to_lowercase();

    // Store keyboard and layout info in metadata
    layout.metadata.keyboard = Some(variant_path);
    layout.metadata.layout_variant = Some(layout_variant.to_string());
    // Use sanitized layout name as keymap name to avoid conflicts with default keymaps
    layout.metadata.keymap_name = Some(sanitized_name.clone());
    layout.metadata.output_format = Some("uf2".to_string());

    // Add a default base layer with KC_TRNS for all positions
    let base_layer = create_default_layer(0, "Base", &mapping)?;
    layout.add_layer(base_layer)?;

    // Create save path using the user-specified layout name
    let layouts_dir = config::Config::config_dir()?.join("layouts");
    std::fs::create_dir_all(&layouts_dir)?;

    let layout_path = layouts_dir.join(format!("{}.md", sanitized_name));

    // Save the layout immediately so it can be found on restart
    parser::save_markdown_layout(&layout, &layout_path)?;

    println!("Layout saved to: {}", layout_path.display());
    println!();

    // Initialize TUI with the generated layout
    let mut terminal = tui::setup_terminal()?;
    let mut app_state =
        tui::AppState::new(layout, Some(layout_path), geometry, mapping, config.clone())?;

    // Layout is clean since we just saved it
    app_state.dirty = false;

    // Run main TUI loop
    let result = tui::run_tui(&mut app_state, &mut terminal);

    // Restore terminal
    tui::restore_terminal(terminal)?;

    // Check for errors
    result?;

    Ok(())
}

/// Creates a default layer with KC_TRNS for all key positions
pub fn create_default_layer(
    number: u8,
    name: &str,
    mapping: &models::VisualLayoutMapping,
) -> Result<models::Layer> {
    use models::layer::KeyDefinition;
    use models::ColorPalette;

    // Use the color palette's default layer color (Gray-500)
    let palette = ColorPalette::load().unwrap_or_default();
    let default_color = palette.default_layer_color();

    let mut layer = models::Layer::new(number, name.to_string(), default_color)?;

    // Add KC_TRNS for each visual position in the mapping
    // This ensures keys use visual positions (not matrix positions) for proper rendering
    for pos in mapping.get_all_visual_positions() {
        let key = KeyDefinition::new(pos, "KC_TRNS".to_string());
        layer.add_key(key);
    }

    Ok(layer)
}
