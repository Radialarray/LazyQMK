//! Action handlers organized by category

/// Category assignment handlers for keys and layers
pub mod category;

/// Color management and customization handlers
pub mod color;

/// File operations handlers (save, load, export)
pub mod file_ops;

/// Firmware generation and build handlers
pub mod firmware;

/// Key manipulation handlers (clear, copy, cut, paste, undo)
pub mod key_ops;

/// Layout variant switching handlers
pub mod layout;

/// Keyboard and layer navigation handlers
pub mod navigation;

/// Popup and overlay management handlers
pub mod popups;

/// Key selection mode handlers
pub mod selection;
