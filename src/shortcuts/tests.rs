//! Tests for shortcuts.
//!
//! Auto-extracted from shortcuts.rs.

use super::*;

use super::*;

#[test]
fn test_basic_lookup() {
    let registry = ShortcutRegistry::new();

    // Test navigation
    let event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
    assert_eq!(registry.lookup("main", event), Some(Action::NavigateUp));

    // Test save
    let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert_eq!(registry.lookup("main", event), Some(Action::Save));
}

#[test]
fn test_v040_shortcuts() {
    let registry = ShortcutRegistry::new();

    // Test flipped color shortcuts
    let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
    assert_eq!(
        registry.lookup("main", event),
        Some(Action::SetIndividualKeyColor)
    );

    let event = KeyEvent::new(KeyCode::Char('C'), KeyModifiers::SHIFT);
    assert_eq!(registry.lookup("main", event), Some(Action::SetLayerColor));

    // Test new category manager shortcut
    let event = KeyEvent::new(KeyCode::Char('K'), KeyModifiers::SHIFT);
    assert_eq!(
        registry.lookup("main", event),
        Some(Action::OpenCategoryManager)
    );

    // Test new layer manager shortcut
    let event = KeyEvent::new(KeyCode::Char('L'), KeyModifiers::SHIFT);
    assert_eq!(
        registry.lookup("main", event),
        Some(Action::OpenLayerManager)
    );

    // Test new build log shortcut
    let event = KeyEvent::new(KeyCode::Char('B'), KeyModifiers::SHIFT);
    assert_eq!(registry.lookup("main", event), Some(Action::ViewBuildLog));

    // Test safer layout variant shortcut
    let event = KeyEvent::new(KeyCode::Char('Y'), KeyModifiers::SHIFT);
    assert_eq!(
        registry.lookup("main", event),
        Some(Action::SwitchLayoutVariant)
    );

    // Test new metadata editor shortcut
    let event = KeyEvent::new(KeyCode::Char('E'), KeyModifiers::SHIFT);
    assert_eq!(registry.lookup("main", event), Some(Action::EditMetadata));

    // Test quick category assignment
    let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
    assert_eq!(
        registry.lookup("main", event),
        Some(Action::AssignCategoryToKey)
    );

    let event = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL);
    assert_eq!(
        registry.lookup("main", event),
        Some(Action::AssignCategoryToLayer)
    );
}

#[test]
fn test_destructive_shortcuts_require_deliberate_keys() {
    let registry = ShortcutRegistry::new();

    assert_eq!(
        registry.lookup(
            "main",
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)
        ),
        None,
        "plain x should not clear or cut keys"
    );
    assert_eq!(
        registry.lookup(
            "main",
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE)
        ),
        None,
        "plain d should not cut keys"
    );
    assert_eq!(
        registry.lookup(
            "main",
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)
        ),
        Some(Action::ClearKey),
        "backspace should clear key"
    );
    assert_eq!(
        registry.lookup(
            "main",
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL)
        ),
        Some(Action::CutKey),
        "cut should remain on Ctrl+X"
    );
}

#[test]
fn test_vim_navigation() {
    let registry = ShortcutRegistry::new();

    // Vim keys should work for navigation
    assert_eq!(
        registry.lookup(
            "main",
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE)
        ),
        Some(Action::NavigateLeft)
    );
    assert_eq!(
        registry.lookup(
            "main",
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)
        ),
        Some(Action::NavigateDown)
    );
    assert_eq!(
        registry.lookup(
            "main",
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)
        ),
        Some(Action::NavigateUp)
    );
    assert_eq!(
        registry.lookup(
            "main",
            KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE)
        ),
        Some(Action::NavigateRight)
    );
}

#[test]
fn test_layer_navigation_shortcuts() {
    let registry = ShortcutRegistry::new();

    // Tab should go to next layer
    let tab_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(
        registry.lookup("main", tab_event),
        Some(Action::NextLayer),
        "Tab should be mapped to NextLayer"
    );

    // BackTab (Shift+Tab) should go to previous layer
    let backtab_event = KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE);
    assert_eq!(
        registry.lookup("main", backtab_event),
        Some(Action::PreviousLayer),
        "BackTab should be mapped to PreviousLayer"
    );

    // Some terminals send Tab+SHIFT instead of BackTab
    let shift_tab_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT);
    assert_eq!(
        registry.lookup("main", shift_tab_event),
        Some(Action::PreviousLayer),
        "Tab+SHIFT should also be mapped to PreviousLayer"
    );
}
