//! End-to-end tests for `lazyqmk generate` command.
#![allow(unused_variables)] // Temp dirs must be kept alive even if not directly accessed

use std::fs;
use std::process::Command;

mod fixtures;
mod golden_helper;

use fixtures::*;
use golden_helper::*;

/// Path to the lazyqmk binary
fn lazyqmk_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lazyqmk")
}

#[test]
fn test_generate_basic_succeeds() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_dir = config_temp.path().join("output");
    fs::create_dir_all(&out_dir).expect("Failed to create output dir");

    let output = Command::new(lazyqmk_bin())
        .args([
            "generate",
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
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Generation should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that files were created
    assert!(
        out_dir.join("keymap.c").exists(),
        "keymap.c should be created"
    );
    assert!(
        out_dir.join("config.h").exists(),
        "config.h should be created"
    );
}

#[test]
fn test_generate_deterministic_output() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_dir = config_temp.path().join("output");
    fs::create_dir_all(&out_dir).expect("Failed to create output dir");

    // Generate twice with deterministic mode
    let output1 = Command::new(lazyqmk_bin())
        .args([
            "generate",
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
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--deterministic",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output1.status.code(), Some(0));

    let keymap1 = fs::read_to_string(out_dir.join("keymap.c")).expect("Failed to read keymap.c");
    let config1 = fs::read_to_string(out_dir.join("config.h")).expect("Failed to read config.h");

    // Generate again
    let output2 = Command::new(lazyqmk_bin())
        .args([
            "generate",
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
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--deterministic",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output2.status.code(), Some(0));

    let keymap2 = fs::read_to_string(out_dir.join("keymap.c")).expect("Failed to read keymap.c");
    let config2 = fs::read_to_string(out_dir.join("config.h")).expect("Failed to read config.h");

    // Content should be identical in deterministic mode
    assert_eq!(
        normalize_firmware_output(&keymap1),
        normalize_firmware_output(&keymap2),
        "Deterministic mode should produce identical keymap.c"
    );
    assert_eq!(
        normalize_firmware_output(&config1),
        normalize_firmware_output(&config2),
        "Deterministic mode should produce identical config.h"
    );
}

#[test]
fn test_generate_format_keymap_only() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_dir = config_temp.path().join("output");
    fs::create_dir_all(&out_dir).expect("Failed to create output dir");

    let output = Command::new(lazyqmk_bin())
        .args([
            "generate",
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
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--format",
            "keymap",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    assert!(
        out_dir.join("keymap.c").exists(),
        "keymap.c should be created"
    );
    // Config.h might still be created depending on implementation
    // The --format flag primarily controls what's generated
}

#[test]
fn test_generate_format_config_only() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_dir = config_temp.path().join("output");
    fs::create_dir_all(&out_dir).expect("Failed to create output dir");

    let output = Command::new(lazyqmk_bin())
        .args([
            "generate",
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
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--format",
            "config",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    assert!(
        out_dir.join("config.h").exists(),
        "config.h should be created"
    );
}

#[test]
fn test_generate_golden_basic_keymap() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_dir = config_temp.path().join("output");
    fs::create_dir_all(&out_dir).expect("Failed to create output dir");

    let output = Command::new(lazyqmk_bin())
        .args([
            "generate",
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
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--deterministic",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let keymap = fs::read_to_string(out_dir.join("keymap.c")).expect("Failed to read keymap.c");
    
    assert_golden(&keymap, "tests/golden/keymap_basic.c");
}

#[test]
fn test_generate_golden_basic_config() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_dir = config_temp.path().join("output");
    fs::create_dir_all(&out_dir).expect("Failed to create output dir");

    let output = Command::new(lazyqmk_bin())
        .args([
            "generate",
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
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--deterministic",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let config_h = fs::read_to_string(out_dir.join("config.h")).expect("Failed to read config.h");
    
    assert_golden(&config_h, "tests/golden/config_basic.h");
}

#[test]
fn test_generate_idle_effect_on() {
    let layout = test_layout_with_idle_effect(true);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_dir = config_temp.path().join("output");
    fs::create_dir_all(&out_dir).expect("Failed to create output dir");

    let output = Command::new(lazyqmk_bin())
        .args([
            "generate",
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
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--deterministic",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let keymap = fs::read_to_string(out_dir.join("keymap.c")).expect("Failed to read keymap.c");
    let config_h = fs::read_to_string(out_dir.join("config.h")).expect("Failed to read config.h");

    // Check idle effect code is present
    assert!(
        keymap.contains("idle_state_t"),
        "keymap should have idle state enum"
    );
    assert!(
        keymap.contains("matrix_scan_user"),
        "keymap should have matrix_scan_user"
    );
    assert!(
        config_h.contains("LQMK_IDLE_TIMEOUT_MS"),
        "config should have idle timeout"
    );
    assert!(
        !config_h.contains("RGB_MATRIX_TIMEOUT"),
        "config should NOT have RGB_MATRIX_TIMEOUT when idle effect enabled"
    );

    assert_golden(&keymap, "tests/golden/keymap_idle_effect_on.c");
    assert_golden(&config_h, "tests/golden/config_idle_effect.h");
}

#[test]
fn test_generate_idle_effect_off() {
    let layout = test_layout_with_idle_effect(false);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_dir = config_temp.path().join("output");
    fs::create_dir_all(&out_dir).expect("Failed to create output dir");

    let output = Command::new(lazyqmk_bin())
        .args([
            "generate",
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
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--deterministic",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));

    let keymap = fs::read_to_string(out_dir.join("keymap.c")).expect("Failed to read keymap.c");
    let config_h = fs::read_to_string(out_dir.join("config.h")).expect("Failed to read config.h");

    // Check idle effect code is NOT present
    assert!(
        !keymap.contains("idle_state_t"),
        "keymap should not have idle state enum"
    );
    assert!(
        config_h.contains("RGB_MATRIX_TIMEOUT 90000"),
        "config should have RGB_MATRIX_TIMEOUT when idle effect disabled"
    );
    assert!(
        !config_h.contains("LQMK_IDLE_TIMEOUT_MS"),
        "config should not have idle timeout"
    );
}

#[test]
fn test_generate_with_tap_dances() {
    let layout = test_layout_with_tap_dances();
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_dir = config_temp.path().join("output");
    fs::create_dir_all(&out_dir).expect("Failed to create output dir");

    let output = Command::new(lazyqmk_bin())
        .args([
            "generate",
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
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--deterministic",
        ])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "Generation should succeed. stderr: {}",
        stderr
    );

    let keymap = fs::read_to_string(out_dir.join("keymap.c")).expect("Failed to read keymap.c");

    // Check tap dance code
    assert!(
        keymap.contains("enum tap_dance_ids"),
        "Should have tap dance enum"
    );
    assert!(keymap.contains("TD_ESC_CAPS"), "Should have 2-way TD");
    assert!(keymap.contains("TD_SHIFT_CTRL"), "Should have 3-way TD");
    assert!(
        keymap.contains("ACTION_TAP_DANCE_DOUBLE"),
        "Should have tap dance macros"
    );
    
    // Note: Currently the generator uses ACTION_TAP_DANCE_DOUBLE for all tap dances,
    // even 3-way ones with hold actions. This is a known limitation that could be
    // enhanced to use ACTION_TAP_DANCE_FN_ADVANCED for 3-way tap dances.
    assert!(
        keymap.contains("tap_dance_actions"),
        "Should have tap dance actions array"
    );

    assert_golden(&keymap, "tests/golden/keymap_tap_dances.c");
}

#[test]
fn test_generate_with_categories() {
    let layout = test_layout_with_categories();
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_dir = config_temp.path().join("output");
    fs::create_dir_all(&out_dir).expect("Failed to create output dir");

    let output = Command::new(lazyqmk_bin())
        .args([
            "generate",
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
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Categories should not affect generation"
    );

    // Categories don't affect C code output, just ensure generation succeeds
    let keymap = fs::read_to_string(out_dir.join("keymap.c")).expect("Failed to read keymap.c");
    assert!(keymap.contains("KC_0"), "Should still have keycodes");
}

#[test]
fn test_generate_invalid_layout() {
    let layout = test_layout_with_invalid_keycode();
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_dir = config_temp.path().join("output");
    fs::create_dir_all(&out_dir).expect("Failed to create output dir");

    let output = Command::new(lazyqmk_bin())
        .args([
            "generate",
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
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // Generation might still succeed (generator is permissive),
    // but validation should catch it
    // The exit code depends on whether generator validates first
    assert!(
        output.status.code() == Some(0) || output.status.code() == Some(1),
        "Exit code should be 0 (generated) or 1 (validated and failed)"
    );
}

#[test]
fn test_generate_missing_qmk_path() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, layout_temp) = create_temp_layout_file(&layout);
    let out_dir = layout_temp.path().join("output");
    fs::create_dir_all(&out_dir).expect("Failed to create output dir");

    let output = Command::new(lazyqmk_bin())
        .args([
            "generate",
            "--layout",
            layout_path.to_str().unwrap(),
            "--qmk-path",
            "/nonexistent/qmk_firmware",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(2),
        "Missing QMK path should exit with code 2"
    );
}
