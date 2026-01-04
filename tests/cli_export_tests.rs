//! End-to-end tests for `lazyqmk export` command.
#![allow(unused_variables)] // Temp dirs must be kept alive even if not directly accessed

use std::fs;
use std::process::Command;

mod fixtures;
use fixtures::*;

/// Path to the lazyqmk binary
fn lazyqmk_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lazyqmk")
}

#[test]
fn test_export_basic_succeeds() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_path = config_temp.path().join("export.md");

    let output = Command::new(lazyqmk_bin())
        .args([
            "export",
            "--layout",
            layout_path.to_str().unwrap(),
            "--qmk-path",
            config
                .paths
                .qmk_firmware
                .as_ref()
                .unwrap()
                .to_str()
                .unwrap(),
            "--output",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Export should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output file exists
    assert!(
        out_path.exists(),
        "Export file should exist at: {}",
        out_path.display()
    );

    // Verify basic content
    let content = fs::read_to_string(&out_path).expect("Failed to read export file");
    assert!(content.contains("# Test Layout"));
    assert!(content.contains("## Quick Reference"));
    assert!(content.contains("## Keyboard Layout"));
    assert!(content.contains("## Layer 0: Base"));
}

#[test]
fn test_export_validates_diagram_structure() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_path = config_temp.path().join("export.md");

    let output = Command::new(lazyqmk_bin())
        .args([
            "export",
            "--layout",
            layout_path.to_str().unwrap(),
            "--qmk-path",
            config
                .paths
                .qmk_firmware
                .as_ref()
                .unwrap()
                .to_str()
                .unwrap(),
            "--output",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Export should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&out_path).expect("Failed to read export file");

    // Check for Unicode box-drawing characters
    assert!(content.contains('┌'), "Should contain top-left corner");
    assert!(content.contains('┐'), "Should contain top-right corner");
    assert!(content.contains('└'), "Should contain bottom-left corner");
    assert!(content.contains('┘'), "Should contain bottom-right corner");
    assert!(content.contains('─'), "Should contain horizontal line");
    assert!(content.contains('│'), "Should contain vertical line");

    // Check for code blocks (diagrams should be in code blocks)
    assert!(content.contains("```"), "Should contain code blocks for diagrams");

    // Check that keycodes are present
    assert!(content.contains("KC_"), "Should contain keycodes");
}

#[test]
fn test_export_multiple_layers() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_path = config_temp.path().join("export.md");

    let output = Command::new(lazyqmk_bin())
        .args([
            "export",
            "--layout",
            layout_path.to_str().unwrap(),
            "--qmk-path",
            config
                .paths
                .qmk_firmware
                .as_ref()
                .unwrap()
                .to_str()
                .unwrap(),
            "--output",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Export should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&out_path).expect("Failed to read export file");

    // Should have all layers
    assert!(content.contains("## Layer 0: Base"), "Should have base layer");
    assert!(
        content.contains("## Layer 1: Function"),
        "Should have layer 1"
    );

    // Each layer should have its diagram
    let layer_0_count = content.matches("Layer 0: Base").count();
    assert!(layer_0_count >= 1, "Should have layer 0 header");

    // Check for default color in non-base layers
    assert!(
        content.contains("**Default Color:**"),
        "Should have default color for layers"
    );
}
