//! Tests for category.
//!
//! Auto-extracted from category.rs.

use super::*;

use super::*;

#[test]
fn test_new_valid() {
    let color = RgbColor::new(0, 255, 0);
    let category = Category::new("navigation", "Navigation Keys", color).unwrap();

    assert_eq!(category.id, "navigation");
    assert_eq!(category.name, "Navigation Keys");
    assert_eq!(category.color, color);
}

#[test]
fn test_validate_id_valid() {
    assert!(Category::validate_id("navigation").is_ok());
    assert!(Category::validate_id("media-keys").is_ok());
    assert!(Category::validate_id("layer-1").is_ok());
    assert!(Category::validate_id("f-keys").is_ok());
}

#[test]
fn test_validate_id_invalid() {
    assert!(Category::validate_id("").is_err());
    assert!(Category::validate_id("Navigation").is_err()); // uppercase
    assert!(Category::validate_id("media keys").is_err()); // space
    assert!(Category::validate_id("media_keys").is_err()); // underscore
    assert!(Category::validate_id("-navigation").is_err()); // starts with hyphen
    assert!(Category::validate_id("navigation-").is_err()); // ends with hyphen
}

#[test]
fn test_validate_name_valid() {
    assert!(Category::validate_name("Navigation").is_ok());
    assert!(Category::validate_name("Media Keys").is_ok());
    assert!(Category::validate_name("A").is_ok());
    assert!(Category::validate_name("This is a valid name with 50 chars exactly!!").is_ok());
}

#[test]
fn test_validate_name_invalid() {
    assert!(Category::validate_name("").is_err());
    assert!(Category::validate_name(&"a".repeat(51)).is_err());
}

#[test]
fn test_set_color() {
    let mut category = Category::new("test", "Test", RgbColor::new(255, 0, 0)).unwrap();
    let new_color = RgbColor::new(0, 255, 0);
    category.set_color(new_color);
    assert_eq!(category.color, new_color);
}

#[test]
fn test_set_name() {
    let mut category = Category::new("test", "Test", RgbColor::new(255, 0, 0)).unwrap();
    category.set_name("New Name").unwrap();
    assert_eq!(category.name, "New Name");

    assert!(category.set_name("").is_err());
    assert!(category.set_name("a".repeat(51)).is_err());
}
