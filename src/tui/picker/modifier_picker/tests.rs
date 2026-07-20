//! Tests for modifier_picker.
//!
//! Auto-extracted from modifier_picker.rs.

use super::*;

use super::*;

#[test]
fn test_modifier_picker_initial_state() {
    let state = ModifierPickerState::new();
    assert_eq!(state.selected_mods, 0);
    assert_eq!(state.focus, 0);
    assert!(!state.has_selection());
}

#[test]
fn test_toggle_individual_modifier() {
    let mut state = ModifierPickerState::new();

    // Toggle LCtrl on
    state.toggle_mod(QmkModifier::LCtrl as u8);
    assert!(state.is_selected(QmkModifier::LCtrl as u8));
    assert!(state.has_selection());

    // Toggle LShift on
    state.toggle_mod(QmkModifier::LShift as u8);
    assert!(state.is_selected(QmkModifier::LCtrl as u8));
    assert!(state.is_selected(QmkModifier::LShift as u8));

    // Toggle LCtrl off
    state.toggle_mod(QmkModifier::LCtrl as u8);
    assert!(!state.is_selected(QmkModifier::LCtrl as u8));
    assert!(state.is_selected(QmkModifier::LShift as u8));
}

#[test]
fn test_to_mod_string_single() {
    let mut state = ModifierPickerState::new();
    state.toggle_mod(QmkModifier::LCtrl as u8);

    assert_eq!(state.to_mod_string(), "MOD_LCTL");
}

#[test]
fn test_to_mod_string_multiple() {
    let mut state = ModifierPickerState::new();
    state.toggle_mod(QmkModifier::LCtrl as u8);
    state.toggle_mod(QmkModifier::LShift as u8);

    assert_eq!(state.to_mod_string(), "MOD_LCTL | MOD_LSFT");
}

#[test]
fn test_to_mod_string_meh() {
    let mut state = ModifierPickerState::new();
    state.selected_mods = ModifierPreset::Meh.bits();

    assert_eq!(state.to_mod_string(), "MOD_LCTL | MOD_LSFT | MOD_LALT");
}

#[test]
fn test_to_mod_string_hyper() {
    let mut state = ModifierPickerState::new();
    state.selected_mods = ModifierPreset::Hyper.bits();

    assert_eq!(
        state.to_mod_string(),
        "MOD_LCTL | MOD_LSFT | MOD_LALT | MOD_LGUI"
    );
}

#[test]
fn test_to_mod_string_empty() {
    let state = ModifierPickerState::new();
    assert_eq!(state.to_mod_string(), "");
}

#[test]
fn test_toggle_focused_modifier() {
    let mut state = ModifierPickerState::new();

    // Focus is at 0 (LCtrl)
    state.toggle_focused();
    assert!(state.is_selected(QmkModifier::LCtrl as u8));

    // Move focus to 1 (LShift) and toggle
    state.focus = 1;
    state.toggle_focused();
    assert!(state.is_selected(QmkModifier::LShift as u8));

    // Move focus to 8 (Meh preset) and toggle
    state.focus = 8;
    state.toggle_focused();
    assert_eq!(state.selected_mods, ModifierPreset::Meh.bits());
}

#[test]
fn test_focus_navigation() {
    let mut state = ModifierPickerState::new();

    // Start at 0 (LCtrl)
    assert_eq!(state.focus, 0);

    // Move right to 4 (RCtrl)
    state.focus_right();
    assert_eq!(state.focus, 4);

    // Move left back to 0 (LCtrl)
    state.focus_left();
    assert_eq!(state.focus, 0);

    // Move down to 1 (LShift)
    state.focus_down();
    assert_eq!(state.focus, 1);

    // Move up back to 0 (LCtrl)
    state.focus_up();
    assert_eq!(state.focus, 0);

    // Move up wraps to 8 (Meh)
    state.focus_up();
    assert_eq!(state.focus, 8);
}
