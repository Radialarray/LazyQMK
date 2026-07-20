//! Tests for branding constants.

use super::*;

#[test]
fn test_branding_consistency() {
    // Ensure binary name is lowercase
    assert_eq!(APP_BINARY_NAME, APP_BINARY_NAME.to_lowercase());

    // Ensure no spaces in binary name
    assert!(!APP_BINARY_NAME.contains(' '));

    // Ensure no spaces in data dir
    assert!(!APP_DATA_DIR.contains(' '));
}
