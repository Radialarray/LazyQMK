//! Display help topics from help.toml

use crate::cli::common::{CliError, CliResult};
use clap::Args;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Display help topics and keybindings from help.toml
#[derive(Args, Debug)]
pub struct HelpArgs {
    /// Help topic name to display (e.g., "main", "keycode_picker", "settings_manager")
    #[arg(value_name = "TOPIC")]
    topic: Option<String>,
}

#[derive(Deserialize, Debug)]
struct HelpData {
    #[serde(default)]
    contexts: BTreeMap<String, Context>,
}

#[derive(Deserialize, Debug)]
struct Context {
    name: String,
    description: String,
    #[serde(default)]
    bindings: Vec<Binding>,
}

#[derive(Deserialize, Debug, Clone)]
struct Binding {
    keys: Vec<String>,
    #[serde(default)]
    alt_keys: Vec<String>,
    action: String,
    #[serde(default)]
    #[allow(dead_code)] // May be used in future for context-aware hints
    hint: String,
    #[serde(default)]
    priority: u32,
}

impl HelpArgs {
    /// Execute the help command: display topics or specific topic details
    pub fn execute(&self) -> CliResult<()> {
        let help_toml = include_str!("../data/help.toml");
        let help_data: HelpData = toml::from_str(help_toml)
            .map_err(|e| CliError::io(format!("Failed to parse help.toml: {}", e)))?;

        if let Some(topic) = &self.topic {
            // Display specific topic
            display_topic(topic, &help_data)
        } else {
            // List all topics
            list_all_topics(&help_data);
            Ok(())
        }
    }
}

fn display_topic(topic: &str, help_data: &HelpData) -> CliResult<()> {
    // Try to find the context with matching name (using underscore normalization)
    let normalized_topic = topic.replace('-', "_");

    let context = help_data.contexts.get(&normalized_topic).ok_or_else(|| {
        CliError::validation(format!(
            "Unknown help topic: '{}'\n\nRun 'lazyqmk help' to see available topics.",
            topic
        ))
    })?;

    // Display context header
    println!("{}", context.name);
    println!("{}", "=".repeat(context.name.len()));
    println!();
    println!("{}", context.description);
    println!();

    if context.bindings.is_empty() {
        println!("(No keybindings defined for this context)");
        return Ok(());
    }

    // Sort bindings by priority
    let mut sorted_bindings = context.bindings.clone();
    sorted_bindings.sort_by_key(|b| b.priority);

    // Display bindings
    println!("Keybindings:");
    println!();

    for binding in sorted_bindings {
        let keys_str = if binding.alt_keys.is_empty() {
            binding.keys.join(", ")
        } else {
            format!(
                "{} (or {})",
                binding.keys.join(", "),
                binding.alt_keys.join(", ")
            )
        };

        println!("  {}  →  {}", keys_str, binding.action);
    }

    Ok(())
}

fn list_all_topics(help_data: &HelpData) {
    println!("Available Help Topics");
    println!("====================");
    println!();

    // Group contexts by category (main vs other)
    let mut main_contexts = Vec::new();
    let mut ui_contexts = Vec::new();
    let mut info_contexts = Vec::new();

    for (key, context) in &help_data.contexts {
        if key == "main" {
            main_contexts.push((key, context));
        } else if context.description.contains("informational") {
            info_contexts.push((key, context));
        } else {
            ui_contexts.push((key, context));
        }
    }

    // Display main context first
    if !main_contexts.is_empty() {
        println!("Core Navigation:");
        for (key, context) in main_contexts {
            println!("  {}  -  {}", key, context.description);
        }
        println!();
    }

    // Display UI contexts
    if !ui_contexts.is_empty() {
        println!("UI Dialogs & Editors:");
        for (key, context) in ui_contexts {
            println!("  {}  -  {}", key, context.description);
        }
        println!();
    }

    // Display informational contexts
    if !info_contexts.is_empty() {
        println!("Reference Information:");
        for (key, context) in info_contexts {
            println!("  {}  -  {}", key, context.description);
        }
        println!();
    }

    println!("Usage: lazyqmk help <topic>");
    println!("       lazyqmk help main");
    println!("       lazyqmk help keycode_picker");
}
