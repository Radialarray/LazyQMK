use lazyqmk::models::layout::{Layout, TapDanceAction};

#[test]
fn test_tap_dance_two_way_creation() {
    let td = TapDanceAction::new("esc_caps".to_string(), "KC_ESC".to_string())
        .with_double_tap("KC_CAPS".to_string());

    assert_eq!(td.name, "esc_caps");
    assert_eq!(td.single_tap, "KC_ESC");
    assert_eq!(td.double_tap, Some("KC_CAPS".to_string()));
    assert_eq!(td.hold, None);
    assert!(td.is_two_way());
    assert!(!td.is_three_way());
    assert!(!td.has_hold());
}

#[test]
fn test_tap_dance_three_way_creation() {
    let td = TapDanceAction::new("shift_caps".to_string(), "KC_LSFT".to_string())
        .with_double_tap("KC_CAPS".to_string())
        .with_hold("KC_RSFT".to_string());

    assert_eq!(td.name, "shift_caps");
    assert_eq!(td.single_tap, "KC_LSFT");
    assert_eq!(td.double_tap, Some("KC_CAPS".to_string()));
    assert_eq!(td.hold, Some("KC_RSFT".to_string()));
    assert!(!td.is_two_way());
    assert!(td.is_three_way());
    assert!(td.has_hold());
}

#[test]
fn test_tap_dance_validation_valid_names() {
    let valid = TapDanceAction::new("esc_caps".to_string(), "KC_ESC".to_string());
    assert!(valid.validate().is_ok());

    let valid_underscore = TapDanceAction::new("my_tap_dance_123".to_string(), "KC_A".to_string());
    assert!(valid_underscore.validate().is_ok());
}

#[test]
fn test_tap_dance_validation_invalid_names() {
    let empty_name = TapDanceAction::new("".to_string(), "KC_A".to_string());
    assert!(empty_name.validate().is_err());

    let space = TapDanceAction::new("my tap".to_string(), "KC_A".to_string());
    assert!(space.validate().is_err());

    let hyphen = TapDanceAction::new("my-tap".to_string(), "KC_A".to_string());
    assert!(hyphen.validate().is_err());

    let special = TapDanceAction::new("my@tap".to_string(), "KC_A".to_string());
    assert!(special.validate().is_err());
}

#[test]
fn test_tap_dance_validation_empty_keycodes() {
    let empty_single = TapDanceAction::new("test".to_string(), "".to_string());
    assert!(empty_single.validate().is_err());

    // Note: with_double_tap and with_hold wrap values in Some(), even empty strings
    // The validation should catch these as errors
    let mut empty_double = TapDanceAction::new("test".to_string(), "KC_A".to_string());
    empty_double.double_tap = Some("".to_string());
    assert!(empty_double.validate().is_err());

    let mut empty_hold = TapDanceAction::new("test".to_string(), "KC_A".to_string());
    empty_hold.hold = Some("".to_string());
    assert!(empty_hold.validate().is_err());
}

#[test]
fn test_layout_add_tap_dance() {
    let mut layout = Layout::new("test").unwrap();

    let td = TapDanceAction::new("esc_caps".to_string(), "KC_ESC".to_string())
        .with_double_tap("KC_CAPS".to_string());

    assert!(layout.add_tap_dance(td.clone()).is_ok());
    assert_eq!(layout.tap_dances.len(), 1);

    // Adding duplicate should fail
    assert!(layout.add_tap_dance(td).is_err());
}

#[test]
fn test_layout_get_tap_dance() {
    let mut layout = Layout::new("test").unwrap();

    let td = TapDanceAction::new("esc_caps".to_string(), "KC_ESC".to_string())
        .with_double_tap("KC_CAPS".to_string());

    layout.add_tap_dance(td).unwrap();

    let retrieved = layout.get_tap_dance("esc_caps");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "esc_caps");

    let not_found = layout.get_tap_dance("nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn test_layout_remove_tap_dance() {
    let mut layout = Layout::new("test").unwrap();

    let td = TapDanceAction::new("esc_caps".to_string(), "KC_ESC".to_string())
        .with_double_tap("KC_CAPS".to_string());

    layout.add_tap_dance(td).unwrap();
    assert_eq!(layout.tap_dances.len(), 1);

    let removed = layout.remove_tap_dance("esc_caps");
    assert!(removed.is_some());
    assert_eq!(layout.tap_dances.len(), 0);

    let not_found = layout.remove_tap_dance("nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn test_tap_dance_round_trip_serialization() {
    // Create a layout programmatically with tap dances
    let mut layout = Layout::new("Test Layout").unwrap();
    
    let td1 = TapDanceAction::new("esc_caps".to_string(), "KC_ESC".to_string())
        .with_double_tap("KC_CAPS".to_string());
    let td2 = TapDanceAction::new("shift_caps".to_string(), "KC_LSFT".to_string())
        .with_double_tap("KC_CAPS".to_string())
        .with_hold("KC_RSFT".to_string());
    
    layout.add_tap_dance(td1).unwrap();
    layout.add_tap_dance(td2).unwrap();

    // Verify tap dances were added
    assert_eq!(layout.tap_dances.len(), 2);

    let esc_caps = layout.get_tap_dance("esc_caps").unwrap();
    assert_eq!(esc_caps.single_tap, "KC_ESC");
    assert_eq!(esc_caps.double_tap, Some("KC_CAPS".to_string()));
    assert_eq!(esc_caps.hold, None);

    let shift_caps = layout.get_tap_dance("shift_caps").unwrap();
    assert_eq!(shift_caps.single_tap, "KC_LSFT");
    assert_eq!(shift_caps.double_tap, Some("KC_CAPS".to_string()));
    assert_eq!(shift_caps.hold, Some("KC_RSFT".to_string()));
}

#[test]
fn test_tap_dance_empty_vec_serde_behavior() {
    // Test that empty tap_dances vec uses skip_serializing_if
    let layout = Layout::new("Test Layout").unwrap();
    assert_eq!(layout.tap_dances.len(), 0);
    
    // Serialize to YAML to verify skip_serializing_if works
    let yaml = serde_yml::to_string(&layout.metadata).unwrap();
    assert!(!yaml.contains("tap_dances"), "Empty tap_dances should not appear in YAML");
}

#[test]
fn test_validate_tap_dances_with_td_keycodes() {
    use lazyqmk::models::{Layer, KeyDefinition, Position, RgbColor};
    
    let mut layout = Layout::new("Test Layout").unwrap();
    
    // Add tap dance definitions
    let td1 = TapDanceAction::new("esc_caps".to_string(), "KC_ESC".to_string())
        .with_double_tap("KC_CAPS".to_string());
    let td2 = TapDanceAction::new("shift_caps".to_string(), "KC_LSFT".to_string())
        .with_double_tap("KC_CAPS".to_string());
    
    layout.add_tap_dance(td1).unwrap();
    layout.add_tap_dance(td2).unwrap();
    
    // Add a layer with TD() keycodes
    let mut layer = Layer::new(0, "Base".to_string(), RgbColor::new(212, 212, 212)).unwrap();
    layer.add_key(KeyDefinition::new(Position { row: 0, col: 0 }, "KC_Q"));
    layer.add_key(KeyDefinition::new(Position { row: 0, col: 1 }, "TD(esc_caps)"));
    layer.add_key(KeyDefinition::new(Position { row: 0, col: 2 }, "TD(shift_caps)"));
    
    layout.add_layer(layer).unwrap();
    
    // Validation should pass
    assert!(layout.validate_tap_dances().is_ok());
}

#[test]
fn test_validate_tap_dances_with_missing_definition() {
    use lazyqmk::models::{Layer, KeyDefinition, Position, RgbColor};
    
    let mut layout = Layout::new("Test Layout").unwrap();
    
    // Add only one tap dance definition
    let td = TapDanceAction::new("esc_caps".to_string(), "KC_ESC".to_string())
        .with_double_tap("KC_CAPS".to_string());
    layout.add_tap_dance(td).unwrap();
    
    // Add a layer that references a non-existent tap dance
    let mut layer = Layer::new(0, "Base".to_string(), RgbColor::new(212, 212, 212)).unwrap();
    layer.add_key(KeyDefinition::new(Position { row: 0, col: 0 }, "KC_Q"));
    layer.add_key(KeyDefinition::new(Position { row: 0, col: 1 }, "TD(nonexistent)"));  // This doesn't exist!
    layer.add_key(KeyDefinition::new(Position { row: 0, col: 2 }, "KC_W"));
    
    layout.add_layer(layer).unwrap();
    
    // Validation should fail
    let result = layout.validate_tap_dances();
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(err_msg.contains("nonexistent"));
}

#[test]
fn test_get_orphaned_tap_dances() {
    use lazyqmk::models::{Layer, KeyDefinition, Position, RgbColor};
    
    let mut layout = Layout::new("Test Layout").unwrap();
    
    // Add two tap dance definitions
    let td1 = TapDanceAction::new("esc_caps".to_string(), "KC_ESC".to_string())
        .with_double_tap("KC_CAPS".to_string());
    let td2 = TapDanceAction::new("unused_td".to_string(), "KC_A".to_string())
        .with_double_tap("KC_B".to_string());
    
    layout.add_tap_dance(td1).unwrap();
    layout.add_tap_dance(td2).unwrap();
    
    // Add a layer that only uses esc_caps
    let mut layer = Layer::new(0, "Base".to_string(), RgbColor::new(212, 212, 212)).unwrap();
    layer.add_key(KeyDefinition::new(Position { row: 0, col: 0 }, "KC_Q"));
    layer.add_key(KeyDefinition::new(Position { row: 0, col: 1 }, "TD(esc_caps)"));
    layer.add_key(KeyDefinition::new(Position { row: 0, col: 2 }, "KC_W"));
    
    layout.add_layer(layer).unwrap();
    
    let orphaned = layout.get_orphaned_tap_dances();
    assert_eq!(orphaned.len(), 1);
    assert_eq!(orphaned[0], "unused_td");
}
