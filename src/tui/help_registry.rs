//! Help registry for centralized keybinding definitions.
//!
//! This module loads help definitions from an embedded TOML file and provides
//! methods to query bindings by context for the status bar, help overlay, and
//! picker dialogs.

use serde::Deserialize;
use std::collections::HashMap;

/// Embedded help definitions TOML file
const HELP_TOML: &str = include_str!("../data/help.toml");

/// A single keybinding definition
#[derive(Debug, Clone, Deserialize)]
pub struct Binding {
    /// Primary key(s) for this action
    pub keys: Vec<String>,
    /// Alternative keys (optional)
    #[serde(default)]
    pub alt_keys: Vec<String>,
    /// Full description of the action
    pub action: String,
    /// Short hint for status bar (optional, uses first key + action if not set)
    pub hint: Option<String>,
    /// Priority for display order (lower = more important, shown first)
    #[serde(default = "default_priority")]
    pub priority: u32,
}

const fn default_priority() -> u32 {
    50
}

/// A context with its bindings
#[derive(Debug, Clone, Deserialize)]
pub struct Context {
    /// Human-readable name
    pub name: String,
    /// Description of when this context is active
    pub description: String,
    /// Keybindings for this context
    pub bindings: Vec<Binding>,
}

/// Metadata about the help file
#[derive(Debug, Clone, Deserialize)]
pub struct HelpMeta {
    /// Version of the help file format
    pub version: String,
    /// Name of the application
    pub app_name: String,
}

/// Root structure of the help TOML file
#[derive(Debug, Clone, Deserialize)]
struct HelpFile {
    meta: HelpMeta,
    contexts: HashMap<String, Context>,
}

/// Help registry providing access to keybinding definitions
#[derive(Debug, Clone)]
pub struct HelpRegistry {
    meta: HelpMeta,
    contexts: HashMap<String, Context>,
}

impl HelpRegistry {
    /// Load the help registry from the embedded TOML file
    pub fn load() -> Result<Self, toml::de::Error> {
        let help_file: HelpFile = toml::from_str(HELP_TOML)?;
        Ok(Self {
            meta: help_file.meta,
            contexts: help_file.contexts,
        })
    }

    /// Get the app name from metadata
    #[must_use]
    pub fn app_name(&self) -> &str {
        &self.meta.app_name
    }

    /// Get a context by name
    #[must_use]
    pub fn get_context(&self, name: &str) -> Option<&Context> {
        self.contexts.get(name)
    }

    /// Get all bindings for a context, sorted by priority
    #[must_use]
    pub fn get_bindings(&self, context_name: &str) -> Vec<&Binding> {
        if let Some(context) = self.contexts.get(context_name) {
            let mut bindings: Vec<&Binding> = context.bindings.iter().collect();
            bindings.sort_by_key(|b| b.priority);
            bindings
        } else {
            Vec::new()
        }
    }

    /// Get the most important bindings for status bar display (those with hints)
    /// Returns bindings sorted by priority that have a hint defined
    #[must_use]
    pub fn get_status_bar_hints(&self, context_name: &str) -> Vec<&Binding> {
        if let Some(context) = self.contexts.get(context_name) {
            let mut bindings: Vec<&Binding> = context
                .bindings
                .iter()
                .filter(|b| b.hint.is_some())
                .collect();
            bindings.sort_by_key(|b| b.priority);
            bindings
        } else {
            Vec::new()
        }
    }

    /// Get all context names
    #[must_use]
    pub fn context_names(&self) -> Vec<&String> {
        self.contexts.keys().collect()
    }

    /// Format a binding for display in help overlay
    /// Returns (`keys_string`, `action_string`)
    #[must_use]
    pub fn format_binding_for_help(binding: &Binding) -> (String, String) {
        let keys = if binding.alt_keys.is_empty() {
            binding.keys.join("/")
        } else {
            format!(
                "{} ({})",
                binding.keys.join("/"),
                binding.alt_keys.join("/")
            )
        };
        (keys, binding.action.clone())
    }

    /// Format bindings for status bar display
    /// Returns a compact string like "Enter Keycode  c Color  ? Help"
    #[must_use]
    pub fn format_status_bar_hints(
        &self,
        context_name: &str,
        max_hints: usize,
    ) -> Vec<(String, String)> {
        self.get_status_bar_hints(context_name)
            .into_iter()
            .take(max_hints)
            .map(|b| {
                let key = b.keys.first().map_or("", std::string::String::as_str);
                let hint = b.hint.as_deref().unwrap_or(&b.action);
                (key.to_string(), hint.to_string())
            })
            .collect()
    }

    /// Get help file version from metadata.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.meta.version
    }

    /// Get information about all available contexts.
    /// Returns a list of (`context_key`, `context_name`, description) tuples.
    #[must_use]
    pub fn get_context_info(&self) -> Vec<(&str, &str, &str)> {
        self.context_names()
            .into_iter()
            .filter_map(|key| {
                self.contexts
                    .get(key)
                    .map(|ctx| (key.as_str(), ctx.name.as_str(), ctx.description.as_str()))
            })
            .collect()
    }
}

impl Default for HelpRegistry {
    fn default() -> Self {
        Self::load().expect("Failed to load embedded help.toml")
    }
}

/// Known context names as constants for type safety
#[allow(clippy::missing_docs_in_private_items)]
pub mod contexts {
    /// Main keyboard editing view
    pub const MAIN: &str = "main";
    /// Keycode picker popup
    pub const KEYCODE_PICKER: &str = "keycode_picker";
    /// Color picker in palette mode
    pub const COLOR_PICKER_PALETTE: &str = "color_picker_palette";
    /// Color picker in RGB slider mode
    pub const COLOR_PICKER_RGB: &str = "color_picker_rgb";
    /// Layer manager popup
    pub const LAYER_MANAGER: &str = "layer_manager";
    /// Category manager popup
    pub const CATEGORY_MANAGER: &str = "category_manager";
    /// Category picker popup
    pub const CATEGORY_PICKER: &str = "category_picker";
    /// Settings manager popup
    pub const SETTINGS_MANAGER: &str = "settings_manager";
    /// Metadata editor popup
    pub const METADATA_EDITOR: &str = "metadata_editor";
    /// Modifier picker popup
    pub const MODIFIER_PICKER: &str = "modifier_picker";
    /// Layer picker popup
    pub const LAYER_PICKER: &str = "layer_picker";
    /// Layout picker popup
    pub const LAYOUT_PICKER: &str = "layout_picker";
    /// Keyboard picker popup
    pub const KEYBOARD_PICKER: &str = "keyboard_picker";
    /// Build log popup
    pub const BUILD_LOG: &str = "build_log";
    /// Help overlay
    pub const HELP: &str = "help";
    /// Selection mode
    pub const SELECTION: &str = "selection";
    /// Template browser
    pub const TEMPLATE_BROWSER: &str = "template_browser";
    /// Template save dialog
    pub const TEMPLATE_SAVE: &str = "template_save";
    /// Setup wizard
    pub const SETUP_WIZARD: &str = "setup_wizard";
    /// Unsaved changes prompt
    pub const UNSAVED_PROMPT: &str = "unsaved_prompt";
    /// Clipboard operations (informational)
    pub const CLIPBOARD: &str = "clipboard";
    /// Parameterized keycodes (informational)
    pub const PARAMETERIZED_KEYCODES: &str = "parameterized_keycodes";
    /// Tap-hold settings (informational)
    pub const TAP_HOLD_INFO: &str = "tap_hold_info";
    /// Color priority (informational)
    pub const COLOR_PRIORITY: &str = "color_priority";
    /// Tips (informational)
    pub const TIPS: &str = "tips";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_help_registry() {
        let registry = HelpRegistry::load().expect("Failed to load help registry");
        assert_eq!(registry.app_name(), "Keyboard Layout Editor");
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
}
