//! Integration tests for layer navigation (Tab/Shift+Tab cycling).
//!
//! Tests the layer cycling behavior:
//! - Tab cycles forward through layers (0 -> 1 -> 2 -> 0)
//! - Shift+Tab cycles backward through layers (0 -> 2 -> 1 -> 0)

use chrono::Utc;
use lazyqmk::config::{BuildConfig, Config, PathConfig, UiConfig};
use lazyqmk::models::{
    KeyDefinition, KeyGeometry, KeyboardGeometry, Layer, Layout, LayoutMetadata, Position,
    RgbColor, VisualLayoutMapping,
};
use lazyqmk::tui::AppState;
use std::collections::HashMap;

/// Creates a test layout with 3 layers for cycling tests
fn create_test_layout_with_layers() -> Layout {
    let metadata = LayoutMetadata {
        name: "Layer Navigation Test".to_string(),
        description: "Test layout for layer cycling".to_string(),
        author: "Test Suite".to_string(),
        created: Utc::now(),
        modified: Utc::now(),
        tags: vec!["test".to_string()],
        is_template: false,
        version: "1.0.0".to_string(),
        layout_variant: Some("LAYOUT_test".to_string()),
        keyboard: Some("test_kb".to_string()),
        keymap_name: Some("test_keymap".to_string()),
        output_format: Some("uf2".to_string()),
    };

    // Create a simple 2x3 layout (6 keys)
    let mut keys = Vec::new();
    for row in 0..2 {
        for col in 0..3 {
            keys.push(KeyDefinition {
                position: Position { row, col },
                keycode: "KC_TRNS".to_string(),
                label: None,
                color_override: None,
                category_id: None,
                combo_participant: false,
                description: None,
            });
        }
    }

    // Create 3 layers
    let layer0 = Layer {
        number: 0,
        name: "Base".to_string(),
        id: uuid::Uuid::new_v4().to_string(),
        default_color: RgbColor::new(255, 255, 255),
        category_id: None,
        keys: keys.clone(),
        layer_colors_enabled: true,
    };

    let layer1 = Layer {
        number: 1,
        name: "Lower".to_string(),
        id: uuid::Uuid::new_v4().to_string(),
        default_color: RgbColor::new(100, 100, 255),
        category_id: None,
        keys: keys.clone(),
        layer_colors_enabled: true,
    };

    let layer2 = Layer {
        number: 2,
        name: "Raise".to_string(),
        id: uuid::Uuid::new_v4().to_string(),
        default_color: RgbColor::new(255, 100, 100),
        category_id: None,
        keys: keys.clone(),
        layer_colors_enabled: true,
    };

    Layout {
        metadata,
        layers: vec![layer0, layer1, layer2],
        categories: vec![],
        uncolored_key_behavior: lazyqmk::models::UncoloredKeyBehavior::default(),
        idle_effect_settings: lazyqmk::models::IdleEffectSettings::default(),
        rgb_overlay_ripple: lazyqmk::models::RgbOverlayRippleSettings::default(),
        tap_hold_settings: lazyqmk::models::TapHoldSettings::default(),
        rgb_enabled: true,
        rgb_brightness: lazyqmk::models::RgbBrightness::default(),
        rgb_saturation: lazyqmk::models::RgbSaturation::default(),
        rgb_matrix_default_speed: 127,
        rgb_timeout_ms: 0,
        tap_dances: vec![],
        combo_settings: lazyqmk::models::ComboSettings::default(),
    }
}

/// Creates a test keyboard geometry
fn create_test_geometry() -> KeyboardGeometry {
    let mut keys = Vec::new();

    for row in 0..2 {
        for col in 0..3 {
            let layout_idx = row * 3 + col;
            let key_geo = KeyGeometry {
                matrix_position: (row, col),
                led_index: layout_idx,
                layout_index: layout_idx,
                visual_x: f32::from(col) * 2.0,
                visual_y: f32::from(row) * 2.0,
                width: 1.0,
                height: 1.0,
                rotation: 0.0,
            };
            keys.push(key_geo);
        }
    }

    KeyboardGeometry {
        keyboard_name: "test_kb".to_string(),
        layout_name: "LAYOUT_test".to_string(),
        matrix_rows: 2,
        matrix_cols: 3,
        keys,
        encoder_count: 0,
    }
}

/// Creates a test visual layout mapping
fn create_test_mapping() -> VisualLayoutMapping {
    let mut led_to_matrix = Vec::new();
    let mut matrix_to_led = HashMap::new();
    let mut layout_to_matrix = Vec::new();
    let mut matrix_to_layout = HashMap::new();
    let mut matrix_to_visual = HashMap::new();
    let mut visual_to_matrix = HashMap::new();

    for row in 0..2 {
        for col in 0..3 {
            let pos = Position { row, col };
            let matrix_pos = (row, col);
            let led_idx = row * 3 + col;
            let layout_idx = row * 3 + col;

            led_to_matrix.push(matrix_pos);
            layout_to_matrix.push(matrix_pos);

            matrix_to_led.insert(matrix_pos, led_idx);
            matrix_to_layout.insert(matrix_pos, layout_idx);
            matrix_to_visual.insert(matrix_pos, pos);
            visual_to_matrix.insert(pos, matrix_pos);
        }
    }

    VisualLayoutMapping {
        led_to_matrix,
        matrix_to_led,
        layout_to_matrix,
        matrix_to_layout,
        matrix_to_visual,
        visual_to_matrix,
        max_col: 2, // 3 columns means max_col = 2
    }
}

/// Creates a test config
fn create_test_config() -> Config {
    Config {
        paths: PathConfig { qmk_firmware: None },
        build: BuildConfig {
            output_dir: std::env::temp_dir(),
        },
        ui: UiConfig::default(),
    }
}

/// Creates a test AppState with 3 layers
fn create_test_app_state() -> AppState {
    let layout = create_test_layout_with_layers();
    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config();

    AppState::new(layout, None, geometry, mapping, config).expect("Failed to create test app state")
}

#[test]
fn test_next_layer_cycles_forward() {
    use lazyqmk::tui::handlers::action_handlers::navigation;

    let mut state = create_test_app_state();
    assert_eq!(state.layout.layers.len(), 3, "Should have 3 layers");

    // Start at layer 0
    assert_eq!(state.current_layer, 0);

    // Tab to layer 1
    navigation::handle_next_layer(&mut state).expect("Should handle next layer");
    assert_eq!(state.current_layer, 1, "Should move to layer 1");

    // Tab to layer 2
    navigation::handle_next_layer(&mut state).expect("Should handle next layer");
    assert_eq!(state.current_layer, 2, "Should move to layer 2");

    // Tab should cycle back to layer 0
    navigation::handle_next_layer(&mut state).expect("Should handle next layer");
    assert_eq!(
        state.current_layer, 0,
        "Should cycle back to layer 0 from last layer"
    );
}

#[test]
fn test_previous_layer_cycles_backward() {
    use lazyqmk::tui::handlers::action_handlers::navigation;

    let mut state = create_test_app_state();
    assert_eq!(state.layout.layers.len(), 3, "Should have 3 layers");

    // Start at layer 0
    assert_eq!(state.current_layer, 0);

    // Shift+Tab should cycle back to layer 2 (last layer)
    navigation::handle_previous_layer(&mut state).expect("Should handle previous layer");
    assert_eq!(
        state.current_layer, 2,
        "Should cycle to last layer (2) from layer 0"
    );

    // Shift+Tab to layer 1
    navigation::handle_previous_layer(&mut state).expect("Should handle previous layer");
    assert_eq!(state.current_layer, 1, "Should move to layer 1");

    // Shift+Tab to layer 0
    navigation::handle_previous_layer(&mut state).expect("Should handle previous layer");
    assert_eq!(state.current_layer, 0, "Should move to layer 0");

    // Shift+Tab should cycle back to layer 2 again
    navigation::handle_previous_layer(&mut state).expect("Should handle previous layer");
    assert_eq!(
        state.current_layer, 2,
        "Should cycle to last layer (2) from layer 0 again"
    );
}

#[test]
fn test_layer_cycling_bidirectional() {
    use lazyqmk::tui::handlers::action_handlers::navigation;

    let mut state = create_test_app_state();

    // Start at layer 0, go forward then backward
    assert_eq!(state.current_layer, 0);

    navigation::handle_next_layer(&mut state).expect("Should handle next layer");
    assert_eq!(state.current_layer, 1);

    navigation::handle_previous_layer(&mut state).expect("Should handle previous layer");
    assert_eq!(state.current_layer, 0, "Should return to layer 0");

    // Now go backward, should cycle to last layer
    navigation::handle_previous_layer(&mut state).expect("Should handle previous layer");
    assert_eq!(state.current_layer, 2);

    // Go forward, should cycle to first layer
    navigation::handle_next_layer(&mut state).expect("Should handle next layer");
    assert_eq!(state.current_layer, 0);
}

#[test]
fn test_single_layer_no_cycling() {
    use lazyqmk::tui::handlers::action_handlers::navigation;

    let mut state = create_test_app_state();

    // Remove all but one layer
    state.layout.layers.truncate(1);
    state.current_layer = 0;

    assert_eq!(state.layout.layers.len(), 1, "Should have 1 layer");

    // Tab should stay on layer 0
    navigation::handle_next_layer(&mut state).expect("Should handle next layer");
    assert_eq!(
        state.current_layer, 0,
        "Should stay on layer 0 with single layer"
    );

    // Shift+Tab should stay on layer 0
    navigation::handle_previous_layer(&mut state).expect("Should handle previous layer");
    assert_eq!(
        state.current_layer, 0,
        "Should stay on layer 0 with single layer"
    );
}

#[test]
fn test_layer_cycling_with_two_layers() {
    use lazyqmk::tui::handlers::action_handlers::navigation;

    let mut state = create_test_app_state();

    // Keep only 2 layers
    state.layout.layers.truncate(2);
    state.current_layer = 0;

    assert_eq!(state.layout.layers.len(), 2, "Should have 2 layers");

    // Tab: 0 -> 1
    navigation::handle_next_layer(&mut state).expect("Should handle next layer");
    assert_eq!(state.current_layer, 1);

    // Tab: 1 -> 0 (cycle)
    navigation::handle_next_layer(&mut state).expect("Should handle next layer");
    assert_eq!(state.current_layer, 0);

    // Shift+Tab: 0 -> 1 (cycle backward)
    navigation::handle_previous_layer(&mut state).expect("Should handle previous layer");
    assert_eq!(state.current_layer, 1);

    // Shift+Tab: 1 -> 0
    navigation::handle_previous_layer(&mut state).expect("Should handle previous layer");
    assert_eq!(state.current_layer, 0);
}
