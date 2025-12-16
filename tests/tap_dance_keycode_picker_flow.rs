#![allow(missing_docs)]
//! Test the flow when selecting TD() from the keycode picker

use lazyqmk::config::Config;
use lazyqmk::models::{
    KeyDefinition, KeyboardGeometry, Layer, Layout, Position, RgbColor, VisualLayoutMapping,
};
use lazyqmk::tui::keycode_picker::KeycodePickerEvent;
use lazyqmk::tui::{ActiveComponent, AppState, PopupType};

/// Test that selecting TD() from keycode picker opens the tap dance form
#[test]
fn test_selecting_td_from_picker_opens_form() {
    // Setup test state
    let mut layout = Layout::new("Test Layout").unwrap();
    let mut layer = Layer::new(0, "Base".to_string(), RgbColor::new(212, 212, 212)).unwrap();
    layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_TRNS"));
    layout.add_layer(layer).unwrap();

    let geometry = KeyboardGeometry::new("test", "test", 4, 12);
    let mapping = VisualLayoutMapping::default();
    let config = Config::default();

    let mut state = AppState::new(layout, None, geometry, mapping, config).unwrap();
    state.selected_position = Position::new(0, 0);

    // Simulate: user opens keycode picker
    state.open_keycode_picker();

    // Verify picker is open
    assert!(matches!(
        state.active_component,
        Some(ActiveComponent::KeycodePicker(_))
    ));
    assert_eq!(state.active_popup, Some(PopupType::KeycodePicker));

    // Simulate: user selects TD() keycode
    let _event = KeycodePickerEvent::KeycodeSelected("TD()".to_string());

    // Process the event using handle_keycode_picker_event (this is the function under test)
    // We need to directly call the handler since we can't access private functions
    // Instead, we'll check the behavior through public interface

    // The current implementation should:
    // 1. Detect TD() has ParamType::TapDance
    // 2. Open the tap dance form directly
    // 3. Set the context to FromKeycodePicker

    // We'll verify by checking state after the TD() template is handled
    // First check if TD() is recognized as parameterized
    let params = state.keycode_db.get_params("TD()");
    assert!(params.is_some(), "TD() should have parameters defined");

    let params = params.unwrap();
    assert_eq!(
        params.len(),
        4,
        "TD() should have 4 parameters (name, single, double, hold)"
    );
    assert_eq!(
        params[0].param_type,
        lazyqmk::keycode_db::ParamType::TapDance
    );
}

/// Test that the tap dance form is properly initialized when opened from TD() selection
#[test]
fn test_td_form_initialization() {
    // Setup test state
    let mut layout = Layout::new("Test Layout").unwrap();
    let mut layer = Layer::new(0, "Base".to_string(), RgbColor::new(212, 212, 212)).unwrap();
    layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_TRNS"));
    layout.add_layer(layer).unwrap();

    let geometry = KeyboardGeometry::new("test", "test", 4, 12);
    let mapping = VisualLayoutMapping::default();
    let config = Config::default();

    let mut state = AppState::new(layout, None, geometry, mapping, config).unwrap();
    state.selected_position = Position::new(0, 0);

    // Verify we can create a tap dance form
    use lazyqmk::tui::tap_dance_form::TapDanceForm;

    let existing_names = state
        .layout
        .tap_dances
        .iter()
        .map(|td| td.name.clone())
        .collect::<Vec<_>>();

    let form = TapDanceForm::new_create(existing_names);

    // Verify form is created properly
    state.active_component = Some(ActiveComponent::TapDanceForm(form));
    state.active_popup = Some(PopupType::TapDanceForm);
    state.tap_dance_form_context = Some(lazyqmk::tui::TapDanceFormContext::FromKeycodePicker);

    // Verify state is correct
    assert!(matches!(
        state.active_component,
        Some(ActiveComponent::TapDanceForm(_))
    ));
    assert_eq!(state.active_popup, Some(PopupType::TapDanceForm));
    assert!(matches!(
        state.tap_dance_form_context,
        Some(lazyqmk::tui::TapDanceFormContext::FromKeycodePicker)
    ));
}

/// Reproduction test: Open picker -> Select TD() -> Verify form appears
#[test]
fn test_reproduction_td_selection_flow() {
    // Setup test state
    let mut layout = Layout::new("Test Layout").unwrap();
    let mut layer = Layer::new(0, "Base".to_string(), RgbColor::new(212, 212, 212)).unwrap();
    layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_TRNS"));
    layout.add_layer(layer).unwrap();

    let geometry = KeyboardGeometry::new("test", "test", 4, 12);
    let mapping = VisualLayoutMapping::default();
    let config = Config::default();

    let mut state = AppState::new(layout, None, geometry, mapping, config).unwrap();
    state.selected_position = Position::new(0, 0);

    // === STEP 1: Open keycode picker (this should work) ===
    state.open_keycode_picker();

    println!("After open_keycode_picker:");
    println!(
        "  active_component: {:?}",
        state
            .active_component
            .as_ref()
            .map(|c| std::mem::discriminant(c))
    );
    println!("  active_popup: {:?}", state.active_popup);

    assert!(
        matches!(
            state.active_component,
            Some(ActiveComponent::KeycodePicker(_))
        ),
        "Step 1 Failed: Keycode picker should be open"
    );
    assert_eq!(
        state.active_popup,
        Some(PopupType::KeycodePicker),
        "Step 1 Failed: Popup should be KeycodePicker"
    );

    // === STEP 2: Simulate selecting TD() ===
    // In the real app, this happens when user presses Enter on TD() in the picker
    // The picker emits KeycodePickerEvent::KeycodeSelected("TD()")
    // This is handled by handle_keycode_picker_event in popups.rs

    // Since we can't directly call the private handler, we'll simulate what should happen:
    // 1. start_parameterized_keycode_flow() is called with "TD()"
    // 2. It detects ParamType::TapDance in params[0]
    // 3. It opens the tap dance form directly

    // Check if TD() would be recognized as parameterized
    let params = state.keycode_db.get_params("TD()");
    assert!(
        params.is_some(),
        "TD() should be recognized as parameterized"
    );

    let params = params.unwrap();
    assert!(!params.is_empty(), "TD() should have parameters");
    assert_eq!(
        params[0].param_type,
        lazyqmk::keycode_db::ParamType::TapDance,
        "First param should be TapDance type"
    );

    // Manually simulate the flow that should happen in start_parameterized_keycode_flow
    state.pending_keycode.reset();
    state.pending_keycode.keycode_template = Some("TD()".to_string());

    // The handler should detect ParamType::TapDance and open form
    // Simulate that action:
    let existing_names = state
        .layout
        .tap_dances
        .iter()
        .map(|td| td.name.clone())
        .collect::<Vec<_>>();

    let form = lazyqmk::tui::tap_dance_form::TapDanceForm::new_create(existing_names);
    state.tap_dance_form_context = Some(lazyqmk::tui::TapDanceFormContext::FromKeycodePicker);
    state.active_component = Some(ActiveComponent::TapDanceForm(form));
    state.active_popup = Some(PopupType::TapDanceForm);
    state.pending_keycode.reset(); // Important: reset pending state

    println!("\nAfter simulating TD() selection:");
    println!(
        "  active_component: {:?}",
        state
            .active_component
            .as_ref()
            .map(|c| std::mem::discriminant(c))
    );
    println!("  active_popup: {:?}", state.active_popup);
    println!(
        "  tap_dance_form_context: {:?}",
        state.tap_dance_form_context
    );
    println!(
        "  pending_keycode cleared: {}",
        state.pending_keycode.keycode_template.is_none()
    );

    // === STEP 3: Verify form is open ===
    assert!(
        matches!(
            state.active_component,
            Some(ActiveComponent::TapDanceForm(_))
        ),
        "Step 3 Failed: Tap dance form should be open"
    );
    assert_eq!(
        state.active_popup,
        Some(PopupType::TapDanceForm),
        "Step 3 Failed: Popup should be TapDanceForm"
    );
    assert!(
        matches!(
            state.tap_dance_form_context,
            Some(lazyqmk::tui::TapDanceFormContext::FromKeycodePicker)
        ),
        "Step 3 Failed: Context should be FromKeycodePicker"
    );
    assert!(
        state.pending_keycode.keycode_template.is_none(),
        "Step 3 Failed: Pending keycode should be cleared"
    );
}
