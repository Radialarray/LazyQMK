//! Contract tests for QMK info.json parsing.
//!
//! These tests validate that we can correctly parse real QMK info.json files
//! from the vial-qmk-keebart submodule. They ensure compatibility with actual
//! keyboard definitions used in the QMK ecosystem.

use keyboard_configurator::models::visual_layout_mapping::VisualLayoutMapping;
use keyboard_configurator::parser::keyboard_json::{
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
    let result = parse_keyboard_info_json(&qmk_path, "crkbd");

    assert!(
        result.is_ok(),
        "Failed to parse crkbd info.json: {:?}",
        result.err()
    );

    let info = result.unwrap();
    assert!(
        !info.layouts.is_empty(),
        "crkbd should have at least one layout"
    );

    let layouts = extract_layout_names(&info);
    println!("Available layouts for crkbd: {layouts:?}");

    // crkbd typically has LAYOUT or LAYOUT_split_3x6_3
    assert!(!layouts.is_empty(), "crkbd should have layout definitions");
}

#[test]
fn test_build_crkbd_geometry() {
    if !qmk_submodule_exists() {
        eprintln!("Skipping test: QMK submodule not initialized");
        return;
    }

    let qmk_path = get_qmk_path();
    let info =
        parse_keyboard_info_json(&qmk_path, "crkbd").expect("Failed to parse crkbd info.json");

    let layouts = extract_layout_names(&info);
    assert!(!layouts.is_empty(), "crkbd should have layouts");

    // Try to build geometry for the first available layout
    let layout_name = &layouts[0];
    let geometry_result = build_keyboard_geometry(&info, "crkbd", layout_name);

    assert!(
        geometry_result.is_ok(),
        "Failed to build geometry for crkbd/{}: {:?}",
        layout_name,
        geometry_result.err()
    );

    let geometry = geometry_result.unwrap();
    assert_eq!(geometry.keyboard_name, "crkbd");
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
        parse_keyboard_info_json(&qmk_path, "crkbd").expect("Failed to parse crkbd info.json");
    let layouts = extract_layout_names(&info);
    let layout_name = &layouts[0];

    let geometry =
        build_keyboard_geometry(&info, "crkbd", layout_name).expect("Failed to build geometry");

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
    let keyboards = vec![
        "crkbd",
        "ferris/sweep",
        "keebart/corne_choc_pro/standard",
    ];

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

#[test]
fn test_scan_keyboards_finds_crkbd() {
    if !qmk_submodule_exists() {
        eprintln!("Skipping test: QMK submodule not initialized");
        return;
    }

    let qmk_path = get_qmk_path();
    let keyboards = keyboard_configurator::parser::keyboard_json::scan_keyboards(&qmk_path)
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
    let keebart_keyboards: Vec<_> = keyboards.iter()
        .filter(|k| k.starts_with("keebart/corne_choc_pro/"))
        .collect();
    
    if !keebart_keyboards.is_empty() {
        println!("Found keebart/corne_choc_pro variants: {:?}", keebart_keyboards);
        // Should have standard and/or mini, not the parent directory
        assert!(
            keebart_keyboards.iter().any(|k| k.ends_with("/standard") || k.ends_with("/mini")),
            "Should find specific variants (standard/mini), not parent directory"
        );
    }
    
    // Check splitkb keyboards - should find revision paths
    let splitkb_keyboards: Vec<_> = keyboards.iter()
        .filter(|k| k.starts_with("splitkb/"))
        .collect();
    
    if !splitkb_keyboards.is_empty() {
        println!("Sample splitkb keyboards: {:?}", &splitkb_keyboards[..3.min(splitkb_keyboards.len())]);
        // Should include revision numbers like /rev1, /rev2, etc.
        assert!(
            splitkb_keyboards.iter().any(|k| k.contains("/rev")),
            "splitkb keyboards should include revision paths"
        );
    }
}
