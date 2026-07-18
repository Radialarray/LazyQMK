// Editor module: key editing UI components.

pub mod key_editor;

// Re-export AppState so the moved key_editor module keeps its `use super::AppState;`.
pub use crate::tui::AppState;
