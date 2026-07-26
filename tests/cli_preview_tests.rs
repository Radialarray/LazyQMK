//! End-to-end tests for `lazyqmk preview` command.
#![allow(unused_variables)] // Temp dirs must be kept alive even if not directly accessed

use std::process::Command;

mod fixtures;
use fixtures::{create_temp_layout_file, temp_config_with_qmk, test_layout_basic};

/// Path to the lazyqmk binary
fn lazyqmk_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lazyqmk")
}

fn run_preview(args: &[&str]) -> std::process::Output {
    Command::new(lazyqmk_bin())
        .args(args)
        .output()
        .expect("Failed to execute preview command")
}

fn assert_success(output: &std::process::Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "preview should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_preview_base_layer_prints_diagram() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _layout_temp) = create_temp_layout_file(&layout);
    let (config, _config_temp) = temp_config_with_qmk(None);

    let output = run_preview(&[
        "preview",
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
    ]);

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Layer 0: Base"), "stdout: {stdout}");
    assert!(stdout.contains('┌'), "expected box-drawing chars: {stdout}");
    assert!(stdout.contains('│'), "expected box-drawing chars: {stdout}");
}

#[test]
fn test_preview_highlight_marker_appears_in_top_right_slot() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _layout_temp) = create_temp_layout_file(&layout);
    let (config, _config_temp) = temp_config_with_qmk(None);

    let output = run_preview(&[
        "preview",
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
        "--highlight",
        "(0,1)=B",
    ]);

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    // The marker must appear in the top border of the target key box (so it
    // sits as the last `─` before `┐` of that box). Find a top border line
    // and inspect the char immediately before the marker.
    let target_top = stdout
        .lines()
        .find(|line| line.starts_with('┌') && line.contains('B'))
        .unwrap_or_else(|| panic!("expected marker 'B' in a top border. stdout:\n{stdout}"));

    let chars: Vec<char> = target_top.chars().collect();
    let marker_pos = chars.iter().position(|c| *c == 'B').unwrap();
    assert_eq!(chars[marker_pos - 1], '─', "char before marker must be `─`");
    assert_eq!(chars[marker_pos + 1], '┐', "char after marker must be `┐`");
    assert!(target_top.ends_with('┐'));
}

#[test]
fn test_preview_legend_appears_below_diagram() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _layout_temp) = create_temp_layout_file(&layout);
    let (config, _config_temp) = temp_config_with_qmk(None);

    let output = run_preview(&[
        "preview",
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
        "--highlight",
        "(0,1)=B",
        "--legend",
        "B: bootloader combo",
    ]);

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("B: bootloader combo"),
        "legend missing. stdout:\n{stdout}"
    );
}

#[test]
fn test_preview_json_output_shape() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _layout_temp) = create_temp_layout_file(&layout);
    let (config, _config_temp) = temp_config_with_qmk(None);

    let output = run_preview(&[
        "preview",
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
        "--highlight",
        "(0,1)=B",
        "--highlight",
        "(0,2)=E",
        "--legend",
        "combo: B boot, E eff",
        "--json",
    ]);

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output must be valid JSON");

    assert_eq!(parsed["layer"], 0);
    assert_eq!(parsed["layer_name"], "Base");
    assert_eq!(parsed["legend"], "combo: B boot, E eff");
    let diagram = parsed["diagram"].as_str().expect("diagram is a string");
    assert!(diagram.contains("Layer 0: Base"));
    assert!(diagram.contains('B'));
    assert!(diagram.contains('E'));

    let highlights = parsed["highlights"].as_array().expect("highlights array");
    assert_eq!(highlights.len(), 2);
    assert_eq!(highlights[0]["position"], serde_json::json!([0, 1]));
    assert_eq!(highlights[0]["marker"], "B");
    assert_eq!(highlights[1]["position"], serde_json::json!([0, 2]));
    assert_eq!(highlights[1]["marker"], "E");
}

#[test]
fn test_preview_missing_position_is_ignored() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _layout_temp) = create_temp_layout_file(&layout);
    let (config, _config_temp) = temp_config_with_qmk(None);

    let output = run_preview(&[
        "preview",
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
        "--highlight",
        "(99,99)=X",
    ]);

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains('X'),
        "marker for missing position should not be rendered. stdout:\n{stdout}"
    );
}

#[test]
fn test_preview_layer_index_out_of_bounds_fails() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _layout_temp) = create_temp_layout_file(&layout);
    let (config, _config_temp) = temp_config_with_qmk(None);

    let output = run_preview(&[
        "preview",
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
        "--layer",
        "42",
    ]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "should exit with validation error. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_preview_rejects_malformed_highlight() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _layout_temp) = create_temp_layout_file(&layout);
    let (config, _config_temp) = temp_config_with_qmk(None);

    let output = run_preview(&[
        "preview",
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
        "--highlight",
        "0,2=B", // missing parens
    ]);

    assert_ne!(
        output.status.code(),
        Some(0),
        "should fail for malformed highlight. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_preview_explicit_layer_index_picks_correct_layer() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _layout_temp) = create_temp_layout_file(&layout);
    let (config, _config_temp) = temp_config_with_qmk(None);

    let output = run_preview(&[
        "preview",
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
        "--layer",
        "1",
    ]);

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Layer 1: Function"),
        "stdout: {stdout}"
    );
}
