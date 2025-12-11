//! Contract tests for QMK info.json parsing.
//!
//! These tests validate that we can correctly parse real QMK info.json files
//! from the vial-qmk-keebart submodule. They ensure compatibility with actual
//! keyboard definitions used in the QMK ecosystem.

use lazyqmk::models::visual_layout_mapping::VisualLayoutMapping;
use lazyqmk::parser::keyboard_json::{
    build_keyboard_geometry, extract_layout_names, parse_keyboard_info_json,
};
use std::path::PathBuf;

/// Gets the QMK firmware path from the submodule.
fn get_qmk_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir).join("qmk_firmware")
}

/// Checks if the QMK submodule is initialized.
fn qmk_submodule_exists() -> bool {
    let qmk_path = get_qmk_path();
    qmk_path.join("keyboards").exists()
}

#[test]
fn test_parse_crkbd_info_json() {
    if !qmk_submodule_exists() {
        eprintln!("Skipping test: QMK submodule not initialized");
        return;
    }

    let qmk_path = get_qmk_path();
    let result = parse_keyboard_info_json(&qmk_path, "crkbd/rev1");

    assert!(
        result.is_ok(),
        "Failed to parse crkbd/rev1 info.json: {:?}",
        result.err()
    );

    let info = result.unwrap();
    assert!(
        !info.layouts.is_empty(),
        "crkbd/rev1 should have at least one layout"
    );

    let layouts = extract_layout_names(&info);
    println!("Available layouts for crkbd/rev1: {layouts:?}");

    // crkbd typically has LAYOUT or LAYOUT_split_3x6_3
    assert!(!layouts.is_empty(), "crkbd/rev1 should have layout definitions");
}

#[test]
fn test_build_crkbd_geometry() {
    if !qmk_submodule_exists() {
        eprintln!("Skipping test: QMK submodule not initialized");
        return;
    }

    let qmk_path = get_qmk_path();
    let info =
        parse_keyboard_info_json(&qmk_path, "crkbd/rev1").expect("Failed to parse crkbd/rev1 info.json");

    let layouts = extract_layout_names(&info);
    assert!(!layouts.is_empty(), "crkbd/rev1 should have layouts");

    // Try to build geometry for the first available layout
    let layout_name = &layouts[0];
    let geometry_result = build_keyboard_geometry(&info, "crkbd/rev1", layout_name);

    assert!(
        geometry_result.is_ok(),
        "Failed to build geometry for crkbd/rev1/{}: {:?}",
        layout_name,
        geometry_result.err()
    );

    let geometry = geometry_result.unwrap();
    assert_eq!(geometry.keyboard_name, "crkbd/rev1");
    assert_eq!(&geometry.layout_name, layout_name);

    // crkbd is a split keyboard with 42 keys (3x6 + 3 thumb keys per half)
    // The exact count depends on the layout variant
    assert!(
        geometry.keys.len() >= 36,
        "crkbd should have at least 36 keys, found {}",
        geometry.keys.len()
    );

    println!(
        "crkbd/{} has {} keys with matrix {}x{}",
        layout_name,
        geometry.keys.len(),
        geometry.matrix_rows,
        geometry.matrix_cols
    );
}

#[test]
fn test_build_crkbd_visual_mapping() {
    if !qmk_submodule_exists() {
        eprintln!("Skipping test: QMK submodule not initialized");
        return;
    }

    let qmk_path = get_qmk_path();
    let info =
        parse_keyboard_info_json(&qmk_path, "crkbd/rev1").expect("Failed to parse crkbd/rev1 info.json");
    let layouts = extract_layout_names(&info);
    let layout_name = &layouts[0];

    let geometry =
        build_keyboard_geometry(&info, "crkbd/rev1", layout_name).expect("Failed to build geometry");

    let mapping = VisualLayoutMapping::build(&geometry);

    assert_eq!(
        mapping.key_count(),
        geometry.keys.len(),
        "Mapping should contain all keys"
    );

    // Verify bidirectional mappings work for first key
    if let Some((matrix_row, matrix_col)) = mapping.led_to_matrix_pos(0) {
        // Matrix -> Visual
        if let Some(visual_pos) = mapping.matrix_to_visual_pos(matrix_row, matrix_col) {
            // Visual -> Matrix (should round-trip)
            let matrix_pos_back = mapping.visual_to_matrix_pos(visual_pos.row, visual_pos.col);
            assert_eq!(
                matrix_pos_back,
                Some((matrix_row, matrix_col)),
                "Visual -> Matrix mapping should round-trip"
            );

            // Visual -> LED
            let led_back = mapping.visual_to_led_index(visual_pos.row, visual_pos.col);
            assert_eq!(led_back, Some(0), "Visual -> LED mapping should round-trip");
        } else {
            panic!("Matrix -> Visual mapping failed for ({matrix_row}, {matrix_col})");
        }
    } else {
        panic!("LED -> Matrix mapping failed for LED 0");
    }

    println!(
        "Successfully built and validated visual mapping for crkbd/{} with {} keys",
        layout_name,
        mapping.key_count()
    );
}

#[test]
fn test_parse_multiple_keyboards() {
    if !qmk_submodule_exists() {
        eprintln!("Skipping test: QMK submodule not initialized");
        return;
    }

    let qmk_path = get_qmk_path();

    // Test a few common keyboards
    let keyboards = vec!["crkbd", "ferris/sweep", "keebart/corne_choc_pro/standard"];

    for keyboard in keyboards {
        let result = parse_keyboard_info_json(&qmk_path, keyboard);

        if result.is_err() {
            eprintln!(
                "Warning: Could not parse keyboard '{}': {:?}",
                keyboard,
                result.err()
            );
            continue;
        }

        let info = result.unwrap();
        let layouts = extract_layout_names(&info);

        println!("Keyboard '{keyboard}' has layouts: {layouts:?}");
        assert!(
            !layouts.is_empty(),
            "Keyboard '{keyboard}' should have at least one layout"
        );

        // Try building geometry for first layout
        if let Some(layout_name) = layouts.first() {
            let geometry_result = build_keyboard_geometry(&info, keyboard, layout_name);
            assert!(
                geometry_result.is_ok(),
                "Failed to build geometry for {}/{}: {:?}",
                keyboard,
                layout_name,
                geometry_result.err()
            );

            let geometry = geometry_result.unwrap();
            println!(
                "  Layout '{}': {} keys, matrix {}x{}",
                layout_name,
                geometry.keys.len(),
                geometry.matrix_rows,
                geometry.matrix_cols
            );
        }
    }
}

/// Test for keyboards with split configuration (parent info.json + variant keyboard.json).
///
/// This tests the 1upkeyboards/pi50/grid keyboard which has:
/// - Parent info.json: encoder config, RGB matrix animations, NO layouts
/// - Variant keyboard.json: layouts and RGB matrix LED positions
#[test]
fn test_parse_split_config_keyboard() {
    if !qmk_submodule_exists() {
        eprintln!("Skipping test: QMK submodule not initialized");
        return;
    }

    let qmk_path = get_qmk_path();
    
    // Test 1upkeyboards/pi50/grid which has split configuration
    let result = parse_keyboard_info_json(&qmk_path, "1upkeyboards/pi50/grid");

    assert!(
        result.is_ok(),
        "Failed to parse 1upkeyboards/pi50/grid: {:?}",
        result.err()
    );

    let info = result.unwrap();
    
    // Layouts should come from variant's keyboard.json
    assert!(
        !info.layouts.is_empty(),
        "1upkeyboards/pi50/grid should have layouts from keyboard.json"
    );
    
    let layouts = extract_layout_names(&info);
    println!("1upkeyboards/pi50/grid layouts: {:?}", layouts);
    
    // Should have LAYOUT_ortho_5x12
    assert!(
        layouts.contains(&"LAYOUT_ortho_5x12".to_string()),
        "Should have LAYOUT_ortho_5x12 from keyboard.json"
    );
    
    // Encoder should come from parent's info.json
    assert!(
        info.encoder.is_some(),
        "1upkeyboards/pi50/grid should have encoder config from parent info.json"
    );
    
    let encoder = info.encoder.as_ref().unwrap();
    let rotary = encoder.rotary.as_ref().unwrap();
    assert_eq!(rotary.len(), 1, "Should have 1 encoder");
    
    // Build geometry to verify it works end-to-end
    let geometry_result = build_keyboard_geometry(&info, "1upkeyboards/pi50/grid", "LAYOUT_ortho_5x12");
    assert!(
        geometry_result.is_ok(),
        "Failed to build geometry for 1upkeyboards/pi50/grid: {:?}",
        geometry_result.err()
    );
    
    let geometry = geometry_result.unwrap();
    // pi50 grid has 61 keys (5x12 + encoder)
    assert!(
        geometry.keys.len() >= 60,
        "pi50/grid should have at least 60 keys, found {}",
        geometry.keys.len()
    );
    
    println!(
        "1upkeyboards/pi50/grid: {} keys, {}x{} matrix",
        geometry.keys.len(),
        geometry.matrix_rows,
        geometry.matrix_cols
    );
}

/// Test for keebart/corne_choc_pro keyboard variants.
///
/// This keyboard has a standard structure with info.json in the parent directory
/// and variant subdirectories (standard, mini) that have their own keyboard.json
/// files with RGB matrix configurations.
#[test]
fn test_parse_keebart_corne_choc_pro() {
    if !qmk_submodule_exists() {
        eprintln!("Skipping test: QMK submodule not initialized");
        return;
    }

    let qmk_path = get_qmk_path();
    
    // Test keebart/corne_choc_pro/standard
    let result = parse_keyboard_info_json(&qmk_path, "keebart/corne_choc_pro/standard");

    assert!(
        result.is_ok(),
        "Failed to parse keebart/corne_choc_pro/standard: {:?}",
        result.err()
    );

    let info = result.unwrap();
    
    // Should have layouts
    assert!(
        !info.layouts.is_empty(),
        "keebart/corne_choc_pro/standard should have layouts"
    );
    
    let layouts = extract_layout_names(&info);
    println!("keebart/corne_choc_pro/standard layouts: {:?}", layouts);
    
    // Corne typically has LAYOUT_split_3x6_3 or similar
    assert!(
        layouts.iter().any(|l| l.contains("split") || l == "LAYOUT"),
        "Should have a split layout or LAYOUT"
    );
    
    // Build geometry to verify it works end-to-end
    let layout_name = &layouts[0];
    let geometry_result = build_keyboard_geometry(&info, "keebart/corne_choc_pro", layout_name);
    assert!(
        geometry_result.is_ok(),
        "Failed to build geometry for keebart/corne_choc_pro/standard/{}: {:?}",
        layout_name,
        geometry_result.err()
    );
    
    let geometry = geometry_result.unwrap();
    // Corne has 42 keys (3x6 + 3 thumb keys per half)
    assert!(
        geometry.keys.len() >= 36,
        "keebart/corne_choc_pro should have at least 36 keys, found {}",
        geometry.keys.len()
    );
    
    println!(
        "keebart/corne_choc_pro/standard/{}: {} keys, {}x{} matrix",
        layout_name,
        geometry.keys.len(),
        geometry.matrix_rows,
        geometry.matrix_cols
    );
    
    // Also test the mini variant if it exists
    let mini_result = parse_keyboard_info_json(&qmk_path, "keebart/corne_choc_pro/mini");
    if let Ok(mini_info) = mini_result {
        let mini_layouts = extract_layout_names(&mini_info);
        println!("keebart/corne_choc_pro/mini layouts: {:?}", mini_layouts);
        
        if let Some(mini_layout_name) = mini_layouts.first() {
            let mini_geometry = build_keyboard_geometry(&mini_info, "keebart/corne_choc_pro", mini_layout_name);
            if let Ok(geom) = mini_geometry {
                println!(
                    "keebart/corne_choc_pro/mini/{}: {} keys, {}x{} matrix",
                    mini_layout_name,
                    geom.keys.len(),
                    geom.matrix_rows,
                    geom.matrix_cols
                );
            }
        }
    }
}

#[test]
fn test_scan_keyboards_finds_crkbd() {
    if !qmk_submodule_exists() {
        eprintln!("Skipping test: QMK submodule not initialized");
        return;
    }

    let qmk_path = get_qmk_path();
    let keyboards = lazyqmk::parser::keyboard_json::scan_keyboards(&qmk_path)
        .expect("Failed to scan keyboards");

    assert!(
        !keyboards.is_empty(),
        "Should find at least one keyboard in QMK firmware"
    );

    println!("Found {} keyboards in QMK firmware", keyboards.len());
    println!(
        "First 10 keyboards: {:?}",
        &keyboards[..10.min(keyboards.len())]
    );

    // crkbd should be in the list (with specific revision paths)
    let has_crkbd = keyboards.iter().any(|k| k.contains("crkbd"));
    assert!(has_crkbd, "crkbd should be found in keyboard scan");

    // Should find specific compilable paths, not parent directories
    // e.g., "keebart/corne_choc_pro/standard" not "keebart/corne_choc_pro"
    let keebart_keyboards: Vec<_> = keyboards
        .iter()
        .filter(|k| k.starts_with("keebart/corne_choc_pro/"))
        .collect();

    if !keebart_keyboards.is_empty() {
        println!(
            "Found keebart/corne_choc_pro variants: {:?}",
            keebart_keyboards
        );
        // Should have standard and/or mini, not the parent directory
        assert!(
            keebart_keyboards
                .iter()
                .any(|k| k.ends_with("/standard") || k.ends_with("/mini")),
            "Should find specific variants (standard/mini), not parent directory"
        );
    }

    // Check splitkb keyboards - should find revision paths
    let splitkb_keyboards: Vec<_> = keyboards
        .iter()
        .filter(|k| k.starts_with("splitkb/"))
        .collect();

    if !splitkb_keyboards.is_empty() {
        println!(
            "Sample splitkb keyboards: {:?}",
            &splitkb_keyboards[..3.min(splitkb_keyboards.len())]
        );
        // Should include revision numbers like /rev1, /rev2, etc.
        assert!(
            splitkb_keyboards.iter().any(|k| k.contains("/rev")),
            "splitkb keyboards should include revision paths"
        );
    }
}

/// Test for keyboards with JSON5 format (comments in keyboard.json).
///
/// This tests the splitkb/aurora/lily58/rev1 keyboard which has:
/// - Parent info.json: manufacturer, USB vendor ID
/// - Variant keyboard.json: layouts, RGB matrix with C++ style comments
///
/// QMK uses JSON5 format which allows comments, trailing commas, etc.
/// Example: {"flags": 2, "x": 51, "y": 13},  // L RGB1
#[test]
fn test_parse_json5_keyboard() {
    if !qmk_submodule_exists() {
        eprintln!("Skipping test: QMK submodule not initialized");
        return;
    }

    let qmk_path = get_qmk_path();
    
    // Test splitkb/aurora/lily58/rev1 which has JSON5 comments
    let result = parse_keyboard_info_json(&qmk_path, "splitkb/aurora/lily58/rev1");

    assert!(
        result.is_ok(),
        "Failed to parse splitkb/aurora/lily58/rev1 (JSON5 with comments): {:?}",
        result.err()
    );

    let info = result.unwrap();
    
    // Should have layouts from keyboard.json
    assert!(
        !info.layouts.is_empty(),
        "splitkb/aurora/lily58/rev1 should have layouts from keyboard.json"
    );
    
    let layouts = extract_layout_names(&info);
    println!("splitkb/aurora/lily58/rev1 layouts: {:?}", layouts);
    
    // Should have LAYOUT
    assert!(
        layouts.contains(&"LAYOUT".to_string()),
        "Should have LAYOUT from keyboard.json with JSON5 comments"
    );
    
    // Build geometry to verify parsing worked correctly
    let geometry_result = build_keyboard_geometry(&info, "splitkb/aurora/lily58/rev1", "LAYOUT");
    assert!(
        geometry_result.is_ok(),
        "Failed to build geometry for splitkb/aurora/lily58/rev1: {:?}",
        geometry_result.err()
    );
    
    let geometry = geometry_result.unwrap();
    // lily58 has 58 keys
    assert_eq!(
        geometry.keys.len(), 58,
        "lily58 should have 58 keys, found {}",
        geometry.keys.len()
    );
    
    println!(
        "splitkb/aurora/lily58/rev1: {} keys, {}x{} matrix (parsed from JSON5 with comments)",
        geometry.keys.len(),
        geometry.matrix_rows,
        geometry.matrix_cols
    );
}
