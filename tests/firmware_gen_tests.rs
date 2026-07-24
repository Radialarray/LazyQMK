//! Integration tests for firmware generation pipeline.
//!
//! Tests the complete flow:
//! 1. Validation of layouts before generation
//! 2. Generation of keymap.c and config.h files
//! 3. File writing with atomic operations
//! 4. Coordinate system transformations (visual -> matrix -> LED)

mod fixtures;

#[path = "firmware_gen_tests/combo_32.rs"]
mod combo_32;
#[path = "firmware_gen_tests/generation.rs"]
mod generation;
#[path = "firmware_gen_tests/helpers.rs"]
mod helpers;
#[path = "firmware_gen_tests/idle_effects.rs"]
mod idle_effects;
#[path = "firmware_gen_tests/parameterized_keycodes.rs"]
mod parameterized_keycodes;
#[path = "firmware_gen_tests/tap_dance.rs"]
mod tap_dance;
#[path = "firmware_gen_tests/validation.rs"]
mod validation;
