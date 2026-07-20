//! Tests for help_registry.
//!
//! Auto-extracted from help_registry.rs.

use super::*;

use super::*;

#[test]
fn test_load_help_registry() {
    let registry = HelpRegistry::load().expect("Failed to load help registry");
    assert_eq!(registry.app_name(), "LazyQMK");
}

#[test]
fn test_get_main_context() {
    let registry = HelpRegistry::load().unwrap();
    let context = registry.get_context(contexts::MAIN);
    assert!(context.is_some());
    assert_eq!(context.unwrap().name, "Main View");
}

#[test]
fn test_get_bindings_sorted_by_priority() {
    let registry = HelpRegistry::load().unwrap();
    let bindings = registry.get_bindings(contexts::MAIN);
    assert!(!bindings.is_empty());

    // Verify bindings are sorted by priority
    for window in bindings.windows(2) {
        assert!(window[0].priority <= window[1].priority);
    }
}

#[test]
fn test_status_bar_hints() {
    let registry = HelpRegistry::load().unwrap();
    let hints = registry.get_status_bar_hints(contexts::MAIN);

    // All returned bindings should have hints
    for binding in &hints {
        assert!(binding.hint.is_some());
    }
}

#[test]
fn test_format_status_bar_hints() {
    let registry = HelpRegistry::load().unwrap();
    let hints = registry.format_status_bar_hints(contexts::MAIN, 5);
    assert!(!hints.is_empty());
    assert!(hints.len() <= 5);
}

#[test]
fn test_tap_dance_editor_context_has_hints() {
    let registry = HelpRegistry::load().unwrap();
    let hints = registry.get_status_bar_hints(contexts::TAP_DANCE_EDITOR);

    assert!(!hints.is_empty());
    assert_eq!(hints[0].hint.as_deref(), Some("Navigate"));
}
