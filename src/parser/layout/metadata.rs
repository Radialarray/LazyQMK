//! Metadata phase: YAML frontmatter parsing and validation.

use crate::models::LayoutMetadata;
use anyhow::{Context, Result};

/// Parses YAML frontmatter from the beginning of the file.
///
/// Returns the parsed metadata and the line index where content starts.
pub(super) fn parse_frontmatter(lines: &[&str]) -> Result<(LayoutMetadata, usize)> {
    // Find frontmatter boundaries
    let mut start_idx = None;
    let mut end_idx = None;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if start_idx.is_none() {
                start_idx = Some(idx);
            } else if end_idx.is_none() {
                end_idx = Some(idx);
                break;
            }
        }
    }

    let start =
        start_idx.ok_or_else(|| anyhow::anyhow!("Missing frontmatter start marker (---)"))?;
    let end = end_idx.ok_or_else(|| anyhow::anyhow!("Missing frontmatter end marker (---)"))?;

    // Extract YAML content (between the --- markers)
    let yaml_content = lines[start + 1..end].join("\n");

    // Parse YAML
    let metadata: LayoutMetadata =
        serde_yml::from_str(&yaml_content).context("Failed to parse YAML frontmatter")?;

    // Validate metadata
    validate_metadata(&metadata)?;

    Ok((metadata, end + 1))
}

/// Validates metadata after parsing.
pub(super) fn validate_metadata(metadata: &LayoutMetadata) -> Result<()> {
    if metadata.name.is_empty() {
        anyhow::bail!("Layout name cannot be empty");
    }

    if metadata.name.len() > 100 {
        anyhow::bail!(
            "Layout name exceeds maximum length of 100 characters (got {})",
            metadata.name.len()
        );
    }

    if metadata.modified < metadata.created {
        anyhow::bail!("Modified timestamp cannot be before created timestamp");
    }

    if metadata.version != "1.0" {
        anyhow::bail!(
            "Unsupported schema version '{}'. Only version '1.0' is supported.",
            metadata.version
        );
    }

    // Validate tags
    for tag in &metadata.tags {
        if !super::tag_regex().is_match(tag) {
            anyhow::bail!(
                "Invalid tag '{tag}'. Tags must be lowercase with hyphens and alphanumeric characters only"
            );
        }
    }

    Ok(())
}
