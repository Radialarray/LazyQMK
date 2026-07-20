//! Tests for template_browser.
//!
//! Auto-extracted from template_browser.rs.

use super::*;

use super::*;

#[test]
fn test_template_browser_state_new() {
    let state = TemplateBrowserState::new();
    assert!(state.templates.is_empty());
    assert!(state.search.is_empty());
    assert_eq!(state.selected, 0);
    assert!(!state.search_active);
}

#[test]
fn test_template_browser_search() {
    let mut state = TemplateBrowserState::new();

    state.search_push('t');
    state.search_push('e');
    state.search_push('s');
    state.search_push('t');
    assert_eq!(state.search, "test");

    state.search_pop();
    assert_eq!(state.search, "tes");

    state.search_clear();
    assert!(state.search.is_empty());
}

#[test]
fn test_template_browser_navigation() {
    let mut state = TemplateBrowserState::new();

    // Add some dummy templates
    for i in 0..5 {
        state.templates.push(TemplateInfo {
            path: PathBuf::from(format!("test{i}.md")),
            metadata: LayoutMetadata::new(format!("Template {i}")).unwrap(),
        });
    }

    assert_eq!(state.selected, 0);

    state.select_next();
    assert_eq!(state.selected, 1);

    state.select_next();
    assert_eq!(state.selected, 2);

    state.select_previous();
    assert_eq!(state.selected, 1);

    state.select_previous();
    assert_eq!(state.selected, 0);

    // Can't go below 0
    state.select_previous();
    assert_eq!(state.selected, 0);
}
