//! Layout versioning commands.
//!
//! These commands operate on the per-layout snapshot folder:
//! `<layouts>/<name>/current.json`, `<layouts>/<name>/manifest.json`, and
//! `<layouts>/<name>/versions/<n>.json`.

use crate::cli::common::{CliError, CliResult};
use crate::config::Config;
use crate::services::layout_versions::LayoutVersionService;
use crate::services::LayoutService;
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

/// Manage layout revisions (snapshots).
#[derive(Debug, Clone, Args)]
pub struct VersionsArgs {
    /// Subcommand to run.
    #[command(subcommand)]
    pub command: VersionsCommand,
}

/// Available revision subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum VersionsCommand {
    /// List all revisions for a layout (newest first).
    List(ListRevisionsArgs),
    /// Show a single revision's full layout.
    Show(ShowRevisionArgs),
    /// Save the current layout as a new revision.
    Save(SaveRevisionArgs),
    /// Restore current.json from a revision (auto-snapshots current first).
    Restore(RestoreRevisionArgs),
    /// Delete a revision (refuses the active one).
    Delete(DeleteRevisionArgs),
    /// Rename a revision's label/note.
    Rename(RenameRevisionArgs),
    /// Show a diff between two revisions.
    Diff(DiffRevisionsArgs),
}

/// `versions list`
#[derive(Debug, Clone, Args)]
pub struct ListRevisionsArgs {
    /// Layout name (e.g., `corne_choc_pro`).
    pub name: String,
    /// Emit JSON instead of a human-readable table.
    #[arg(long)]
    pub json: bool,
}

/// `versions show`
#[derive(Debug, Clone, Args)]
pub struct ShowRevisionArgs {
    /// Layout name.
    pub name: String,
    /// Revision id.
    pub revision: u32,
    /// Emit JSON instead of a human-readable summary.
    #[arg(long)]
    pub json: bool,
}

/// `versions save`
#[derive(Debug, Clone, Args)]
pub struct SaveRevisionArgs {
    /// Path to layout file (used to load current state).
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,
    /// Optional short label (e.g., "pre-rgb-overhaul").
    #[arg(long, value_name = "TEXT")]
    pub label: Option<String>,
    /// Optional longer note.
    #[arg(long, value_name = "TEXT")]
    pub note: Option<String>,
    /// Emit JSON instead of a human-readable summary.
    #[arg(long)]
    pub json: bool,
}

/// `versions restore`
#[derive(Debug, Clone, Args)]
pub struct RestoreRevisionArgs {
    /// Path to layout file (used to determine the layout folder).
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,
    /// Revision id to restore.
    pub revision: u32,
    /// Skip confirmation prompt (CI/scripting).
    #[arg(long)]
    pub yes: bool,
}

/// `versions delete`
#[derive(Debug, Clone, Args)]
pub struct DeleteRevisionArgs {
    /// Path to layout file.
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,
    /// Revision id to delete.
    pub revision: u32,
    /// Skip confirmation prompt.
    #[arg(long)]
    pub yes: bool,
}

/// `versions rename`
#[derive(Debug, Clone, Args)]
pub struct RenameRevisionArgs {
    /// Path to layout file.
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,
    /// Revision id.
    pub revision: u32,
    /// New label.
    #[arg(long, value_name = "TEXT")]
    pub label: Option<String>,
    /// New note.
    #[arg(long, value_name = "TEXT")]
    pub note: Option<String>,
}

/// `versions diff`
#[derive(Debug, Clone, Args)]
pub struct DiffRevisionsArgs {
    /// Path to layout file.
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,
    /// From revision id.
    pub from: u32,
    /// To revision id.
    pub to: u32,
    /// Emit JSON instead of a human-readable diff.
    #[arg(long)]
    pub json: bool,
}

impl VersionsArgs {
    /// Execute the chosen subcommand.
    ///
    /// # Errors
    ///
    /// Returns [`CliError`] on validation, IO, or service failures.
    pub fn execute(&self) -> CliResult<()> {
        match &self.command {
            VersionsCommand::List(a) => list_revisions(a),
            VersionsCommand::Show(a) => show_revision(a),
            VersionsCommand::Save(a) => save_revision(a),
            VersionsCommand::Restore(a) => restore_revision(a),
            VersionsCommand::Delete(a) => delete_revision(a),
            VersionsCommand::Rename(a) => rename_revision(a),
            VersionsCommand::Diff(a) => diff_revisions(a),
        }
    }
}

fn service_for_layout(layout_path: &std::path::Path) -> CliResult<(String, LayoutVersionService)> {
    let name = layout_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| CliError::validation("Cannot derive layout name from path"))?
        .to_string();
    let layouts_dir = layout_path
        .parent()
        .map(std::path::Path::to_path_buf)
        .or_else(|| Config::config_dir().ok().map(|c| c.join("layouts")))
        .ok_or_else(|| CliError::validation("Cannot determine layouts directory"))?;
    Ok((name, LayoutVersionService::new(layouts_dir)))
}

fn print_revision_row(r: &crate::models::RevisionSummary) {
    let label = r.label.as_deref().unwrap_or("-");
    let note = r.note.as_deref().unwrap_or("");
    println!(
        "  #{:<4} {:<25} {:<20} {}",
        r.revision,
        r.created.format("%Y-%m-%dT%H:%M:%SZ"),
        label,
        note
    );
}

fn list_revisions(args: &ListRevisionsArgs) -> CliResult<()> {
    let layouts_dir = Config::config_dir()
        .map_err(|e| CliError::io(format!("Failed to resolve config dir: {e}")))?
        .join("layouts");
    let svc = LayoutVersionService::new(layouts_dir);
    let list = svc
        .list(&args.name)
        .map_err(|e| CliError::io(e.to_string()))?;
    if args.json {
        let json = serde_json::to_string_pretty(&list)
            .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?;
        println!("{json}");
        return Ok(());
    }
    if list.is_empty() {
        println!("No revisions for layout '{}'.", args.name);
        return Ok(());
    }
    println!("Revisions for '{}':", args.name);
    print_revision_header();
    for r in &list {
        print_revision_row(r);
    }
    Ok(())
}

fn print_revision_header() {
    println!("  {:<5} {:<25} {:<20} Note", "#", "Created", "Label");
}

fn show_revision(args: &ShowRevisionArgs) -> CliResult<()> {
    let layouts_dir = Config::config_dir()
        .map_err(|e| CliError::io(format!("Failed to resolve config dir: {e}")))?
        .join("layouts");
    let svc = LayoutVersionService::new(layouts_dir);
    let snap = svc
        .get(&args.name, args.revision)
        .map_err(|e| CliError::io(e.to_string()))?;
    if args.json {
        let json = serde_json::to_string_pretty(&snap)
            .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?;
        println!("{json}");
        return Ok(());
    }
    println!("Revision #{} of '{}'", snap.revision, args.name);
    println!(
        "  Created: {}",
        snap.created.format("%Y-%m-%dT%H:%M:%SZ")
    );
    println!(
        "  Label:   {}",
        snap.label.as_deref().unwrap_or("-")
    );
    println!(
        "  Note:    {}",
        snap.note.as_deref().unwrap_or("-")
    );
    println!("  Author:  {}", snap.author);
    println!("  Layers:  {}", snap.layout.layers.len());
    println!("  Name:    {}", snap.layout.metadata.name);
    Ok(())
}

fn save_revision(args: &SaveRevisionArgs) -> CliResult<()> {
    let layout = LayoutService::load(&args.layout)
        .map_err(|e| CliError::io(format!("Failed to load layout: {e}")))?;
    let (name, svc) = service_for_layout(&args.layout)?;
    let summary = svc
        .create_snapshot(&name, &layout, args.label.as_deref(), args.note.as_deref(), None)
        .map_err(|e| CliError::io(e.to_string()))?;
    if args.json {
        let json = serde_json::to_string_pretty(&summary)
            .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?;
        println!("{json}");
    } else {
        println!(
            "Saved revision #{} for '{}' (file: {})",
            summary.revision, name, summary.filename
        );
        if let Some(label) = &summary.label {
            println!("  Label: {label}");
        }
    }
    Ok(())
}

fn restore_revision(args: &RestoreRevisionArgs) -> CliResult<()> {
    if !args.yes {
        eprint!(
            "Restore revision {} for layout '{}'? Current layout will be auto-snapshotted first. [y/N] ",
            args.revision,
            args.layout.display()
        );
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| CliError::io(format!("Failed to read stdin: {e}")))?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }
    let (name, svc) = service_for_layout(&args.layout)?;
    svc.restore(&name, args.revision, None)
        .map_err(|e| CliError::io(e.to_string()))?;
    println!(
        "Restored revision #{} for layout '{}'.",
        args.revision, name
    );
    Ok(())
}

fn delete_revision(args: &DeleteRevisionArgs) -> CliResult<()> {
    if !args.yes {
        eprint!(
            "Delete revision {} for layout '{}'? [y/N] ",
            args.revision,
            args.layout.display()
        );
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| CliError::io(format!("Failed to read stdin: {e}")))?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }
    let (name, svc) = service_for_layout(&args.layout)?;
    svc.delete(&name, args.revision)
        .map_err(|e| CliError::io(e.to_string()))?;
    println!(
        "Deleted revision #{} for layout '{}'.",
        args.revision, name
    );
    Ok(())
}

fn rename_revision(args: &RenameRevisionArgs) -> CliResult<()> {
    let (name, svc) = service_for_layout(&args.layout)?;
    let summary = svc
        .rename(
            &name,
            args.revision,
            args.label.as_deref(),
            args.note.as_deref(),
        )
        .map_err(|e| CliError::io(e.to_string()))?;
    println!(
        "Updated revision #{} for '{}' (file: {}).",
        summary.revision, name, summary.filename
    );
    Ok(())
}

#[derive(Serialize)]
#[allow(clippy::struct_excessive_bools)] // Flat DTO shape mirrors the JSON consumers expect.
struct DiffOutput {
    from_revision: u32,
    to_revision: u32,
    layers_added: u32,
    layers_removed: u32,
    keys_changed: u32,
    rgb_changed: bool,
    combos_changed: bool,
    tap_dances_changed: bool,
    metadata_changed: bool,
    layer_changes: Vec<serde_json::Value>,
    setting_changes: Vec<serde_json::Value>,
}

fn diff_revisions(args: &DiffRevisionsArgs) -> CliResult<()> {
    let (name, svc) = service_for_layout(&args.layout)?;
    let diff = svc
        .diff(&name, args.from, args.to)
        .map_err(|e| CliError::io(e.to_string()))?;
    if args.json {
        let out = DiffOutput {
            from_revision: diff.from_revision,
            to_revision: diff.to_revision,
            layers_added: diff.summary.layers_added,
            layers_removed: diff.summary.layers_removed,
            keys_changed: diff.summary.keys_changed,
            rgb_changed: diff.summary.rgb_changed,
            combos_changed: diff.summary.combos_changed,
            tap_dances_changed: diff.summary.tap_dances_changed,
            metadata_changed: diff.summary.metadata_changed,
            layer_changes: serde_json::to_value(&diff.layer_changes)
                .map(|v| {
                    v.as_array()
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .collect()
                })
                .unwrap_or_default(),
            setting_changes: serde_json::to_value(&diff.setting_changes)
                .map(|v| {
                    v.as_array()
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .collect()
                })
                .unwrap_or_default(),
        };
        let json = serde_json::to_string_pretty(&out)
            .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?;
        println!("{json}");
        return Ok(());
    }

    println!(
        "Diff: revision {} → {} (layout '{}')",
        diff.from_revision, diff.to_revision, name
    );
    println!("  Layers added:     {}", diff.summary.layers_added);
    println!("  Layers removed:   {}", diff.summary.layers_removed);
    println!("  Keys changed:     {}", diff.summary.keys_changed);
    println!(
        "  RGB changed:      {}",
        if diff.summary.rgb_changed { "yes" } else { "no" }
    );
    println!(
        "  Combos changed:   {}",
        if diff.summary.combos_changed { "yes" } else { "no" }
    );
    println!(
        "  Tap dances changed: {}",
        if diff.summary.tap_dances_changed {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "  Metadata changed: {}",
        if diff.summary.metadata_changed {
            "yes"
        } else {
            "no"
        }
    );

    for change in &diff.layer_changes {
        match change {
            crate::models::LayerDiff::Added { index, layer } => {
                println!("+ Layer {} '{}' (added)", index, layer.name);
            }
            crate::models::LayerDiff::Removed { index, name } => {
                println!("- Layer {} '{}' (removed)", index, name);
            }
            crate::models::LayerDiff::KeysChanged {
                index,
                name,
                changes,
            } => {
                println!("~ Layer {} '{}' ({} keys)", index, name, changes.len());
                for kc in changes {
                    println!(
                        "    ({},{}): {} → {}",
                        kc.row, kc.col, kc.from, kc.to
                    );
                }
            }
            crate::models::LayerDiff::Renamed { index, from, to } => {
                println!("~ Layer {} renamed: '{}' → '{}'", index, from, to);
            }
        }
    }

    if !diff.setting_changes.is_empty() {
        println!("Settings:");
        for s in &diff.setting_changes {
            println!("  {}: {} → {}", s.path, s.from, s.to);
        }
    }
    Ok(())
}
