//! Tests for keyboard.
//!
//! Auto-extracted from keyboard.rs.

use super::*;

use super::*;

#[test]
fn test_format_simple_keycode() {
    assert_eq!(crate::keycode_db::format::strip_kc_prefix("KC_A"), "A");
    assert_eq!(crate::keycode_db::format::strip_kc_prefix("KC_SPC"), "SPC");
    assert_eq!(
        crate::keycode_db::format::strip_kc_prefix("KC_ENTER"),
        "ENTER"
    );
    assert_eq!(crate::keycode_db::format::strip_kc_prefix("MO(1)"), "MO(1)");
}

#[test]
fn test_format_modifier() {
    assert_eq!(crate::keycode_db::format::format_modifier("MOD_LCTL"), "C");
    assert_eq!(crate::keycode_db::format::format_modifier("MOD_LSFT"), "S");
    assert_eq!(crate::keycode_db::format::format_modifier("MOD_LALT"), "A");
    assert_eq!(crate::keycode_db::format::format_modifier("MOD_LGUI"), "G");
    assert_eq!(
        crate::keycode_db::format::format_modifier("MOD_LCTL | MOD_LSFT"),
        "CS"
    );
    assert_eq!(
        crate::keycode_db::format::format_modifier("MOD_LCTL | MOD_LSFT | MOD_LALT"),
        "CSA"
    );
    assert_eq!(
        crate::keycode_db::format::format_modifier("MOD_LCTL | MOD_LSFT | MOD_LALT | MOD_LGUI"),
        "CSAG"
    );
}

#[test]
fn test_truncate() {
    assert_eq!(KeyboardWidget::truncate("ABC", 5), "ABC");
    assert_eq!(KeyboardWidget::truncate("ABCDEF", 5), "ABCDE");
    assert_eq!(KeyboardWidget::truncate("", 5), "");
}

#[test]
fn test_color_indicator_legend() {
    assert_eq!(
        KeyboardWidget::color_indicator_legend(),
        "i key override  c key category  L layer category  d layer default  - colors off"
    );
}

#[test]
fn test_combo_action_at_returns_action_for_participating_keys() {
    use crate::models::{ComboAction, ComboDefinition};

    let combos = vec![
        ComboDefinition::new(
            Position::new(0, 0),
            Position::new(0, 1),
            ComboAction::Bootloader,
        ),
        ComboDefinition::new(
            Position::new(1, 2),
            Position::new(1, 3),
            ComboAction::DisableEffects,
        ),
        ComboDefinition::new(
            Position::new(2, 0),
            Position::new(3, 0),
            ComboAction::DisableLighting,
        ),
    ];

    // First key of each combo resolves to its action.
    assert_eq!(
        combo_action_at(&combos, Position::new(0, 0)),
        Some(ComboAction::Bootloader)
    );
    // Second key resolves too (combo participation is symmetric).
    assert_eq!(
        combo_action_at(&combos, Position::new(0, 1)),
        Some(ComboAction::Bootloader)
    );
    assert_eq!(
        combo_action_at(&combos, Position::new(1, 3)),
        Some(ComboAction::DisableEffects)
    );
    assert_eq!(
        combo_action_at(&combos, Position::new(3, 0)),
        Some(ComboAction::DisableLighting)
    );
    // Unrelated position returns None.
    assert_eq!(combo_action_at(&combos, Position::new(9, 9)), None);
}

#[test]
fn test_combo_action_at_ignores_placeholders() {
    use crate::models::{ComboAction, ComboDefinition};

    let mut placeholder = ComboDefinition::new(
        Position::new(4, 4),
        Position::new(4, 5),
        ComboAction::Bootloader,
    );
    placeholder.placeholder = true;
    let combos = vec![placeholder];

    // Placeholders must never surface a combo overlay on the canvas.
    assert_eq!(combo_action_at(&combos, Position::new(4, 4)), None);
    assert_eq!(combo_action_at(&combos, Position::new(4, 5)), None);
}

#[test]
fn test_combo_action_at_handles_empty_list() {
    assert_eq!(combo_action_at(&[], Position::new(0, 0)), None);
}
