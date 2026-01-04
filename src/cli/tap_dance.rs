//! Tap dance management commands for CLI.

use crate::cli::common::{CliError, CliResult};
use crate::models::{Layout, TapDanceAction};
use crate::services::LayoutService;
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

/// Manage tap dance definitions
#[derive(Debug, Clone, Args)]
pub struct TapDanceArgs {
    /// Tap dance subcommand to execute
    #[command(subcommand)]
    pub command: TapDanceCommand,
}

/// Tap dance subcommands
#[derive(Debug, Clone, Subcommand)]
pub enum TapDanceCommand {
    /// List all tap dance definitions
    List(ListArgs),
    /// Add a new tap dance definition
    Add(AddArgs),
    /// Delete a tap dance definition
    Delete(DeleteArgs),
    /// Validate tap dance references
    Validate(ValidateArgs),
}

/// List tap dance definitions
#[derive(Debug, Clone, Args)]
pub struct ListArgs {
    /// Path to layout markdown file
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,
}

/// Add a new tap dance definition
#[derive(Debug, Clone, Args)]
pub struct AddArgs {
    /// Path to layout markdown file
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,

    /// Unique name for the tap dance (must be valid C identifier)
    #[arg(short, long)]
    pub name: String,

    /// Keycode for single tap
    #[arg(short, long)]
    pub single: String,

    /// Optional keycode for double tap
    #[arg(short, long)]
    pub double: Option<String>,

    /// Optional keycode for hold
    #[arg(long)]
    pub hold: Option<String>,
}

/// Delete a tap dance definition
#[derive(Debug, Clone, Args)]
pub struct DeleteArgs {
    /// Path to layout markdown file
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,

    /// Name of the tap dance to delete
    #[arg(short, long)]
    pub name: String,

    /// Force deletion even if referenced in layers (replaces with KC_TRNS)
    #[arg(short, long)]
    pub force: bool,
}

/// Validate tap dance references
#[derive(Debug, Clone, Args)]
pub struct ValidateArgs {
    /// Path to layout markdown file
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct TapDanceListResponse {
    tap_dances: Vec<TapDanceInfo>,
    count: usize,
}

#[derive(Debug, Serialize)]
struct TapDanceInfo {
    name: String,
    single_tap: String,
    double_tap: Option<String>,
    hold: Option<String>,
    #[serde(rename = "type")]
    td_type: String,
}

#[derive(Debug, Serialize)]
struct TapDanceValidateResponse {
    valid: bool,
    orphaned: Vec<String>,
    unused: Vec<String>,
}

impl TapDanceArgs {
    /// Execute the tap-dance subcommand
    pub fn execute(&self) -> CliResult<()> {
        match &self.command {
            TapDanceCommand::List(args) => execute_list(args),
            TapDanceCommand::Add(args) => execute_add(args),
            TapDanceCommand::Delete(args) => execute_delete(args),
            TapDanceCommand::Validate(args) => execute_validate(args),
        }
    }
}

/// Execute the list subcommand
fn execute_list(args: &ListArgs) -> CliResult<()> {
    // Load layout
    let layout = LayoutService::load(&args.layout)
        .map_err(|e| CliError::io(format!("Failed to load layout: {e}")))?;

    let tap_dances: Vec<TapDanceInfo> = layout
        .tap_dances
        .iter()
        .map(|td| {
            let td_type = if td.is_three_way() {
                "three_way"
            } else if td.is_two_way() {
                "two_way"
            } else {
                "single"
            };

            TapDanceInfo {
                name: td.name.clone(),
                single_tap: td.single_tap.clone(),
                double_tap: td.double_tap.clone(),
                hold: td.hold.clone(),
                td_type: td_type.to_string(),
            }
        })
        .collect();

    if args.json {
        let response = TapDanceListResponse {
            count: tap_dances.len(),
            tap_dances,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&response)
                .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?
        );
    } else {
        // Text output: one per line with name and actions
        for td in &tap_dances {
            print!("{}: single={}", td.name, td.single_tap);
            if let Some(ref double) = td.double_tap {
                print!(", double={double}");
            }
            if let Some(ref hold) = td.hold {
                print!(", hold={hold}");
            }
            println!();
        }
    }

    Ok(())
}

/// Execute the add subcommand
fn execute_add(args: &AddArgs) -> CliResult<()> {
    // Load layout
    let mut layout = LayoutService::load(&args.layout)
        .map_err(|e| CliError::io(format!("Failed to load layout: {e}")))?;

    // Check if name already exists
    if layout.get_tap_dance(&args.name).is_some() {
        return Err(CliError::validation(format!(
            "Tap dance with name '{}' already exists",
            args.name
        )));
    }

    // Create tap dance action
    let mut tap_dance = TapDanceAction::new(&args.name, &args.single);

    if let Some(ref double) = args.double {
        tap_dance = tap_dance.with_double_tap(double);
    }

    if let Some(ref hold) = args.hold {
        tap_dance = tap_dance.with_hold(hold);
    }

    // Validate tap dance
    tap_dance
        .validate()
        .map_err(|e| CliError::validation(format!("Invalid tap dance: {e}")))?;

    // Add to layout
    layout
        .add_tap_dance(tap_dance)
        .map_err(|e| CliError::validation(format!("Failed to add tap dance: {e}")))?;

    // Save layout
    LayoutService::save(&layout, &args.layout)
        .map_err(|e| CliError::io(format!("Failed to save layout: {e}")))?;

    println!(
        "Successfully added tap dance '{}' to {}",
        args.name,
        args.layout.display()
    );

    Ok(())
}

/// Execute the delete subcommand
fn execute_delete(args: &DeleteArgs) -> CliResult<()> {
    // Load layout
    let mut layout = LayoutService::load(&args.layout)
        .map_err(|e| CliError::io(format!("Failed to load layout: {e}")))?;

    // Check if tap dance exists
    if layout.get_tap_dance(&args.name).is_none() {
        return Err(CliError::validation(format!(
            "Tap dance '{}' not found",
            args.name
        )));
    }

    // Check if tap dance is referenced in layers
    let references = find_tap_dance_references(&layout, &args.name);

    if !references.is_empty() && !args.force {
        return Err(CliError::validation(format!(
            "Tap dance '{}' is referenced in {} location(s). Use --force to delete and replace references with KC_TRNS",
            args.name,
            references.len()
        )));
    }

    // If --force, replace all references with KC_TRNS
    if args.force {
        remove_tap_dance_references(&mut layout, &args.name);
    }

    // Remove the tap dance definition
    layout
        .remove_tap_dance(&args.name)
        .expect("Tap dance should exist");

    // Save layout
    LayoutService::save(&layout, &args.layout)
        .map_err(|e| CliError::io(format!("Failed to save layout: {e}")))?;

    if args.force && !references.is_empty() {
        println!(
            "Successfully deleted tap dance '{}' and replaced {} reference(s) with KC_TRNS",
            args.name,
            references.len()
        );
    } else {
        println!("Successfully deleted tap dance '{}'", args.name);
    }

    Ok(())
}

/// Execute the validate subcommand
fn execute_validate(args: &ValidateArgs) -> CliResult<()> {
    // Load layout
    let layout = LayoutService::load(&args.layout)
        .map_err(|e| CliError::io(format!("Failed to load layout: {e}")))?;

    // Find orphaned references (used in layers but no definition)
    let orphaned = find_orphaned_references(&layout);

    // Find unused definitions (defined but never referenced)
    let unused = layout.get_orphaned_tap_dances();

    let valid = orphaned.is_empty();

    if args.json {
        let response = TapDanceValidateResponse {
            valid,
            orphaned,
            unused,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&response)
                .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?
        );

        return if !valid {
            Err(CliError::validation("Tap dance validation failed"))
        } else {
            Ok(())
        };
    }

    // Text output
    if !orphaned.is_empty() {
        println!("Errors:");
        for name in &orphaned {
            println!("  ✗ Orphaned reference: TD({name}) used but not defined");
        }
    }

    if !unused.is_empty() {
        println!("{}Warnings:", if orphaned.is_empty() { "" } else { "\n" });
        for name in &unused {
            println!("  ⚠ Unused definition: '{name}' defined but never used");
        }
    }

    if valid && unused.is_empty() {
        println!("✓ All tap dance references are valid");
    }

    if !valid {
        Err(CliError::validation("Tap dance validation failed"))
    } else {
        Ok(())
    }
}

/// Finds all references to a specific tap dance in the layout.
/// Returns a list of (layer_idx, key_idx) tuples.
fn find_tap_dance_references(layout: &Layout, name: &str) -> Vec<(usize, usize)> {
    let td_pattern = format!("TD({name})");
    let mut references = Vec::new();

    for (layer_idx, layer) in layout.layers.iter().enumerate() {
        for (key_idx, key) in layer.keys.iter().enumerate() {
            if key.keycode.starts_with(&td_pattern) {
                references.push((layer_idx, key_idx));
            }
        }
    }

    references
}

/// Removes all references to a specific tap dance by replacing them with KC_TRNS.
fn remove_tap_dance_references(layout: &mut Layout, name: &str) {
    let td_pattern = format!("TD({name})");

    for layer in &mut layout.layers {
        for key in &mut layer.keys {
            if key.keycode.starts_with(&td_pattern) {
                key.keycode = "KC_TRNS".to_string();
            }
        }
    }
}

/// Finds orphaned tap dance references (used but not defined).
fn find_orphaned_references(layout: &Layout) -> Vec<String> {
    use regex::Regex;
    let td_pattern = Regex::new(r"TD\(([^)]+)\)").unwrap();
    let mut referenced_names = std::collections::HashSet::new();

    // Collect all TD() references
    for layer in &layout.layers {
        for key in &layer.keys {
            if let Some(captures) = td_pattern.captures(&key.keycode) {
                referenced_names.insert(captures[1].to_string());
            }
        }
    }

    // Find which ones don't have definitions
    referenced_names
        .into_iter()
        .filter(|name| layout.get_tap_dance(name).is_none())
        .collect()
}
