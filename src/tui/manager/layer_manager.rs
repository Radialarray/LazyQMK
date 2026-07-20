//! Layer manager for CRUD operations on layers.
//!
//! Provides a UI for creating, renaming, reordering, toggling colors, and deleting layers.
//! Accessible via Shift+L shortcut.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::Rect, Frame};

use crate::models::{Layer, Position, RgbColor};
use crate::tui::component::Component;
use crate::tui::Theme;

/// Events emitted by the `LayerManager` component
#[derive(Debug, Clone)]
pub enum LayerManagerEvent {
    /// User created a new layer
    LayerAdded {
        /// The new layer
        layer: Layer,
    },
    /// User deleted a layer
    LayerDeleted {
        /// Index of the deleted layer
        index: usize,
    },
    /// User renamed a layer
    LayerRenamed {
        /// Index of the renamed layer
        index: usize,
        /// New name
        name: String,
    },
    /// User reordered layers (swap)
    LayerReordered {
        /// Original index
        from: usize,
        /// New index
        to: usize,
    },
    /// User duplicated a layer
    LayerDuplicated {
        /// Source layer index
        source_index: usize,
        /// The new layer
        layer: Layer,
    },
    /// User copied keys from one layer to another
    LayerKeysCopied {
        /// Source layer index
        from: usize,
        /// Target layer index
        to: usize,
        /// Keys to copy (position, keycode, color, category)
        keys: Vec<(Position, String, Option<RgbColor>, Option<String>)>,
    },
    /// User swapped keys between two layers
    LayersSwapped {
        /// First layer index
        layer1: usize,
        /// Second layer index
        layer2: usize,
        /// Keys from layer1 (position, keycode, color, category)
        keys1: Vec<(Position, String, Option<RgbColor>, Option<String>)>,
        /// Keys from layer2 (position, keycode, color, category)
        keys2: Vec<(Position, String, Option<RgbColor>, Option<String>)>,
    },
    /// User toggled layer colors
    LayerColorsToggled {
        /// Index of the layer
        index: usize,
        /// New enabled state
        enabled: bool,
    },
    /// User wants to switch to a layer
    LayerSwitched {
        /// Index of the layer to switch to
        index: usize,
    },
    /// User cancelled without making changes
    Cancelled,
    /// Component closed naturally
    Closed,
}

/// Manager mode - determines what operation is being performed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerMode {
    /// Browsing layers (default mode)
    Browsing,
    /// Creating a new layer (entering name)
    CreatingName {
        /// User input for layer name
        input: String,
    },
    /// Renaming a layer
    Renaming {
        /// Index of layer being renamed
        layer_index: usize,
        /// User input for new name
        input: String,
    },
    /// Confirming deletion
    ConfirmingDelete {
        /// Index of layer to delete
        layer_index: usize,
    },
    /// Duplicating a layer (entering name for copy)
    Duplicating {
        /// Index of layer being duplicated
        source_index: usize,
        /// User input for new layer name
        input: String,
    },
    /// Copying all keys to another layer (selecting target)
    CopyingTo {
        /// Index of source layer
        source_index: usize,
        /// Currently selected target layer
        target_selected: usize,
    },
    /// Swapping two layers (selecting swap target)
    Swapping {
        /// Index of first layer (source)
        source_index: usize,
        /// Currently selected swap target
        target_selected: usize,
    },
}

/// State for the layer manager dialog
#[derive(Debug, Clone)]
pub struct LayerManagerState {
    /// Currently selected layer index
    pub selected: usize,
    /// Current operation mode
    pub mode: ManagerMode,
}

impl LayerManagerState {
    /// Create a new layer manager state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected: 0,
            mode: ManagerMode::Browsing,
        }
    }

    /// Reset to default state, optionally setting selection to current layer
    pub fn reset(&mut self, current_layer: usize) {
        self.selected = current_layer;
        self.mode = ManagerMode::Browsing;
    }

    /// Move selection up
    pub const fn select_previous(&mut self, layer_count: usize) {
        if layer_count > 0 {
            if self.selected > 0 {
                self.selected -= 1;
            } else {
                self.selected = layer_count - 1;
            }
        }
    }

    /// Move selection down
    pub const fn select_next(&mut self, layer_count: usize) {
        if layer_count > 0 {
            self.selected = (self.selected + 1) % layer_count;
        }
    }

    /// Start creating a new layer
    pub fn start_creating(&mut self) {
        self.mode = ManagerMode::CreatingName {
            input: String::new(),
        };
    }

    /// Start renaming the selected layer
    pub fn start_renaming(&mut self, layer: &Layer) {
        self.mode = ManagerMode::Renaming {
            layer_index: self.selected,
            input: layer.name.clone(),
        };
    }

    /// Start confirming deletion of the selected layer
    pub fn start_deleting(&mut self) {
        self.mode = ManagerMode::ConfirmingDelete {
            layer_index: self.selected,
        };
    }

    /// Start duplicating the selected layer
    pub fn start_duplicating(&mut self, layer: &Layer) {
        self.mode = ManagerMode::Duplicating {
            source_index: self.selected,
            input: format!("{} (copy)", layer.name),
        };
    }

    /// Start copying to another layer
    pub fn start_copy_to(&mut self, layer_count: usize) {
        // Start with next layer as default target (or first if at end)
        let target = if self.selected + 1 < layer_count {
            self.selected + 1
        } else if self.selected > 0 {
            0
        } else {
            return; // Only one layer, can't copy to itself
        };
        self.mode = ManagerMode::CopyingTo {
            source_index: self.selected,
            target_selected: target,
        };
    }

    /// Start swapping with another layer
    pub fn start_swapping(&mut self, layer_count: usize) {
        // Start with next layer as default target (or first if at end)
        let target = if self.selected + 1 < layer_count {
            self.selected + 1
        } else if self.selected > 0 {
            0
        } else {
            return; // Only one layer, can't swap with itself
        };
        self.mode = ManagerMode::Swapping {
            source_index: self.selected,
            target_selected: target,
        };
    }

    /// Navigate in copy-to or swap mode
    pub const fn select_target_previous(&mut self, layer_count: usize) {
        match &mut self.mode {
            ManagerMode::CopyingTo {
                source_index,
                target_selected,
            }
            | ManagerMode::Swapping {
                source_index,
                target_selected,
            } => {
                // Skip the source layer when navigating
                loop {
                    if *target_selected > 0 {
                        *target_selected -= 1;
                    } else {
                        *target_selected = layer_count - 1;
                    }
                    if *target_selected != *source_index {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    /// Navigate in copy-to or swap mode
    pub const fn select_target_next(&mut self, layer_count: usize) {
        match &mut self.mode {
            ManagerMode::CopyingTo {
                source_index,
                target_selected,
            }
            | ManagerMode::Swapping {
                source_index,
                target_selected,
            } => {
                // Skip the source layer when navigating
                loop {
                    *target_selected = (*target_selected + 1) % layer_count;
                    if *target_selected != *source_index {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    /// Cancel current operation and return to browsing
    pub fn cancel(&mut self) {
        self.mode = ManagerMode::Browsing;
    }

    /// Get the current input text (for name entry or renaming)
    #[must_use]
    pub fn get_input(&self) -> Option<&str> {
        match &self.mode {
            ManagerMode::CreatingName { input }
            | ManagerMode::Renaming { input, .. }
            | ManagerMode::Duplicating { input, .. } => Some(input),
            _ => None,
        }
    }

    /// Get mutable reference to current input text
    pub const fn get_input_mut(&mut self) -> Option<&mut String> {
        match &mut self.mode {
            ManagerMode::CreatingName { input }
            | ManagerMode::Renaming { input, .. }
            | ManagerMode::Duplicating { input, .. } => Some(input),
            _ => None,
        }
    }
}

impl Default for LayerManagerState {
    fn default() -> Self {
        Self::new()
    }
}

/// `LayerManager` component that implements the Component trait
#[derive(Debug, Clone)]
pub struct LayerManager {
    /// Internal state of the layer manager
    state: LayerManagerState,
    /// Layers to display and modify (cached copy)
    cached_layers: Vec<Layer>,
}

impl LayerManager {
    /// Create a new `LayerManager` with initial layers
    #[must_use]
    pub fn new(layers: Vec<Layer>, current_layer: usize) -> Self {
        let mut state = LayerManagerState::new();
        state.reset(current_layer);
        Self {
            state,
            cached_layers: layers,
        }
    }

    /// Update the layers list (needed after CRUD operations)
    pub fn set_layers(&mut self, layers: Vec<Layer>) {
        self.cached_layers = layers;
        // Clamp selection to valid range
        if self.state.selected >= self.cached_layers.len() && !self.cached_layers.is_empty() {
            self.state.selected = self.cached_layers.len() - 1;
        }
    }
}

impl Component for LayerManager {
    type Event = LayerManagerEvent;

    fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event> {
        match &self.state.mode {
            ManagerMode::Browsing => self.handle_browsing_input(key),
            ManagerMode::CreatingName { .. }
            | ManagerMode::Renaming { .. }
            | ManagerMode::Duplicating { .. } => self.handle_text_input(key),
            ManagerMode::ConfirmingDelete { layer_index } => {
                self.handle_delete_confirmation(key, *layer_index)
            }
            ManagerMode::CopyingTo {
                source_index,
                target_selected,
            } => self.handle_copy_to_input(key, *source_index, *target_selected),
            ManagerMode::Swapping {
                source_index,
                target_selected,
            } => self.handle_swapping_input(key, *source_index, *target_selected),
        }
    }

    fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        render_layer_manager(f, area, &self.state, &self.cached_layers, theme);
    }
}

impl LayerManager {
    /// Handle input in browsing mode
    fn handle_browsing_input(&mut self, key: KeyEvent) -> Option<LayerManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.reset(self.state.selected);
                Some(LayerManagerEvent::Closed)
            }
            KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                // Move layer up (reorder)
                let selected = self.state.selected;
                if selected > 0 && self.cached_layers.len() > 1 {
                    self.state.selected = selected - 1;
                    Some(LayerManagerEvent::LayerReordered {
                        from: selected,
                        to: selected - 1,
                    })
                } else {
                    None
                }
            }
            KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                // Move layer down (reorder)
                let selected = self.state.selected;
                if selected < self.cached_layers.len() - 1 {
                    self.state.selected = selected + 1;
                    Some(LayerManagerEvent::LayerReordered {
                        from: selected,
                        to: selected + 1,
                    })
                } else {
                    None
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.select_previous(self.cached_layers.len());
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.select_next(self.cached_layers.len());
                None
            }
            KeyCode::Enter => {
                // Switch to selected layer
                Some(LayerManagerEvent::LayerSwitched {
                    index: self.state.selected,
                })
            }
            KeyCode::Char('n') => {
                // Start creating new layer
                self.state.start_creating();
                None
            }
            KeyCode::Char('r') => {
                // Start renaming
                if let Some(layer) = self.cached_layers.get(self.state.selected) {
                    self.state.start_renaming(layer);
                }
                None
            }
            KeyCode::Char('v') => {
                // Toggle layer colors
                let selected_idx = self.state.selected;
                if let Some(layer) = self.cached_layers.get(selected_idx) {
                    let new_enabled = !layer.layer_colors_enabled;
                    Some(LayerManagerEvent::LayerColorsToggled {
                        index: selected_idx,
                        enabled: new_enabled,
                    })
                } else {
                    None
                }
            }
            KeyCode::Char('d') => {
                // Start delete confirmation
                if self.cached_layers.len() <= 1 {
                    None // Can't delete last layer
                } else {
                    self.state.start_deleting();
                    None
                }
            }
            KeyCode::Char('D') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                // Start duplicating
                if let Some(layer) = self.cached_layers.get(self.state.selected) {
                    self.state.start_duplicating(layer);
                }
                None
            }
            KeyCode::Char('c') => {
                // Start copy to another layer
                if self.cached_layers.len() <= 1 {
                    None // Need at least 2 layers
                } else {
                    self.state.start_copy_to(self.cached_layers.len());
                    None
                }
            }
            KeyCode::Char('s') => {
                // Start swap with another layer
                if self.cached_layers.len() <= 1 {
                    None // Need at least 2 layers
                } else {
                    self.state.start_swapping(self.cached_layers.len());
                    None
                }
            }
            _ => None,
        }
    }

    /// Handle text input (for creating, renaming, duplicating)
    fn handle_text_input(&mut self, key: KeyEvent) -> Option<LayerManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                Some(LayerManagerEvent::Cancelled)
            }
            KeyCode::Enter => {
                if let Some(input) = self.state.get_input() {
                    let input = input.to_string();

                    if input.trim().is_empty() {
                        return None; // Don't process empty input
                    }

                    match &self.state.mode {
                        ManagerMode::CreatingName { .. } => {
                            // Create new layer
                            let new_index = self.cached_layers.len();
                            let default_color = RgbColor::new(128, 128, 128); // Gray-500

                            if let Ok(mut new_layer) =
                                Layer::new(new_index as u8, &input, default_color)
                            {
                                // Copy key positions from first layer (with transparent keycodes)
                                if let Some(first_layer) = self.cached_layers.first() {
                                    for key in &first_layer.keys {
                                        use crate::models::layer::KeyDefinition;
                                        new_layer
                                            .add_key(KeyDefinition::new(key.position, "KC_TRNS"));
                                    }
                                }

                                self.state.cancel();
                                Some(LayerManagerEvent::LayerAdded { layer: new_layer })
                            } else {
                                None
                            }
                        }
                        ManagerMode::Renaming { layer_index, .. } => {
                            let layer_index = *layer_index;
                            self.state.cancel();
                            Some(LayerManagerEvent::LayerRenamed {
                                index: layer_index,
                                name: input,
                            })
                        }
                        ManagerMode::Duplicating { source_index, .. } => {
                            let source_index = *source_index;
                            let new_index = self.cached_layers.len();

                            if let Some(source) = self.cached_layers.get(source_index) {
                                if let Ok(mut new_layer) =
                                    Layer::new(new_index as u8, &input, source.default_color)
                                {
                                    // Copy all keys from source
                                    for key in &source.keys {
                                        use crate::models::layer::KeyDefinition;
                                        let mut new_key =
                                            KeyDefinition::new(key.position, &key.keycode);
                                        new_key.color_override = key.color_override;
                                        new_key.category_id = key.category_id.clone();
                                        new_layer.add_key(new_key);
                                    }
                                    // Copy layer settings
                                    new_layer.layer_colors_enabled = source.layer_colors_enabled;
                                    new_layer.category_id = source.category_id.clone();

                                    self.state.cancel();
                                    Some(LayerManagerEvent::LayerDuplicated {
                                        source_index,
                                        layer: new_layer,
                                    })
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            KeyCode::Char(c) => {
                if let Some(input) = self.state.get_input_mut() {
                    input.push(c);
                }
                None
            }
            KeyCode::Backspace => {
                if let Some(input) = self.state.get_input_mut() {
                    input.pop();
                }
                None
            }
            _ => None,
        }
    }

    /// Handle delete confirmation input
    fn handle_delete_confirmation(
        &mut self,
        key: KeyEvent,
        layer_index: usize,
    ) -> Option<LayerManagerEvent> {
        match key.code {
            KeyCode::Char('y' | 'Y') => {
                if self.cached_layers.len() > 1 {
                    self.state.cancel();
                    Some(LayerManagerEvent::LayerDeleted { index: layer_index })
                } else {
                    // Can't delete last layer
                    self.state.cancel();
                    None
                }
            }
            KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                self.state.cancel();
                Some(LayerManagerEvent::Cancelled)
            }
            _ => None,
        }
    }

    /// Handle copy-to mode input
    fn handle_copy_to_input(
        &mut self,
        key: KeyEvent,
        source_index: usize,
        target_selected: usize,
    ) -> Option<LayerManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                Some(LayerManagerEvent::Cancelled)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.select_target_previous(self.cached_layers.len());
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.select_target_next(self.cached_layers.len());
                None
            }
            KeyCode::Enter => {
                // Copy all keys from source to target
                if let Some(source) = self.cached_layers.get(source_index) {
                    let keys: Vec<_> = source
                        .keys
                        .iter()
                        .map(|k| {
                            (
                                k.position,
                                k.keycode.clone(),
                                k.color_override,
                                k.category_id.clone(),
                            )
                        })
                        .collect();

                    self.state.cancel();
                    Some(LayerManagerEvent::LayerKeysCopied {
                        from: source_index,
                        to: target_selected,
                        keys,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Handle swapping mode input
    fn handle_swapping_input(
        &mut self,
        key: KeyEvent,
        source_index: usize,
        target_selected: usize,
    ) -> Option<LayerManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                Some(LayerManagerEvent::Cancelled)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.select_target_previous(self.cached_layers.len());
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.select_target_next(self.cached_layers.len());
                None
            }
            KeyCode::Enter => {
                // Swap all keys between source and target
                if let (Some(source), Some(target)) = (
                    self.cached_layers.get(source_index),
                    self.cached_layers.get(target_selected),
                ) {
                    let keys1: Vec<_> = source
                        .keys
                        .iter()
                        .map(|k| {
                            (
                                k.position,
                                k.keycode.clone(),
                                k.color_override,
                                k.category_id.clone(),
                            )
                        })
                        .collect();
                    let keys2: Vec<_> = target
                        .keys
                        .iter()
                        .map(|k| {
                            (
                                k.position,
                                k.keycode.clone(),
                                k.color_override,
                                k.category_id.clone(),
                            )
                        })
                        .collect();

                    self.state.cancel();
                    Some(LayerManagerEvent::LayersSwapped {
                        layer1: source_index,
                        layer2: target_selected,
                        keys1,
                        keys2,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Render the layer manager dialog — lives in `layer_manager_render` to keep this file under 1000 lines.
pub use super::layer_manager_render::render_layer_manager;
