#!/bin/bash
# Quick test to verify encoder_map generation

cd "$(dirname "$0")"

# Build the project
cargo build --release --quiet

# Create a minimal test
cat > /tmp/test_keymap_gen.rs << 'RUST'
use keyboard_tui::firmware::FirmwareGenerator;
use keyboard_tui::config::Config;
use keyboard_tui::models::{Layout, KeyboardGeometry, VisualLayoutMapping};

fn main() {
    let layout = Layout::new("Test").unwrap();
    let geometry = KeyboardGeometry::new("test", "LAYOUT", 4, 6);
    let mapping = VisualLayoutMapping::build(&geometry);
    let config = Config::new();
    
    let gen = FirmwareGenerator::new(&layout, &geometry, &mapping, &config);
    match gen.generate_keymap_c() {
        Ok(code) => {
            if code.contains("#ifdef ENCODER_MAP_ENABLE") {
                println!("✓ Encoder map generation working!");
                println!("\nGenerated code sample:");
                for line in code.lines().rev().take(15).rev() {
                    println!("{}", line);
                }
            } else {
                println!("✗ Encoder map NOT found in generated code!");
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
RUST

echo "Test created. The encoder_map should be in the generated keymap.c"
echo "You need to trigger firmware generation (Ctrl+G) in the TUI to regenerate the files."
