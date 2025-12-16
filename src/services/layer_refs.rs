//! Layer reference tracking and analysis.
//!
//! This module provides functionality to detect and track layer-switching keycodes
//! across the keyboard layout, enabling features like:
//! - Displaying inbound references when editing a layer
//! - Warning when non-transparent keys might conflict with hold-to-layer keys

use crate::models::{Layer, Position};
use std::collections::HashMap;

/// Type of layer reference (how a key activates another layer)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerRefKind {
    /// Momentary layer switch while held - MO(n)
    Momentary,
    /// Layer tap - tap for key, hold for layer - LT(n, key)
    TapHold,
    /// Toggle layer on/off - TG(n)
    Toggle,
    /// One-shot layer (next key only) - OSL(n)
    OneShot,
    /// Switch to layer - TO(n)
    SwitchTo,
    /// Tap toggle - TT(n)
    TapToggle,
    /// Set default layer - DF(n)
    DefaultSet,
    /// Layer with modifier - LM(n, mod)
    LayerMod,
    /// Other/unknown layer keycode
    Other,
}

impl LayerRefKind {
    /// Returns true if this is a "hold-like" layer reference that requires
    /// the target key position to remain transparent to avoid conflicts.
    ///
    /// Hold-like keycodes activate the layer while the key is held down,
    /// meaning the key at that position on the target layer will be accessed.
    #[must_use]
    pub const fn is_hold_like(self) -> bool {
        matches!(
            self,
            Self::Momentary | Self::TapHold | Self::TapToggle | Self::LayerMod
        )
    }

    /// Get a human-readable name for this layer reference kind
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Momentary => "Momentary (MO)",
            Self::TapHold => "Tap-Hold (LT)",
            Self::Toggle => "Toggle (TG)",
            Self::OneShot => "One-Shot (OSL)",
            Self::SwitchTo => "Switch (TO)",
            Self::TapToggle => "Tap-Toggle (TT)",
            Self::DefaultSet => "Default Set (DF)",
            Self::LayerMod => "Layer-Mod (LM)",
            Self::Other => "Other",
        }
    }
}

/// Target for a layer reference (numeric index or UUID)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerRefTarget {
    /// Numeric layer index (0-based)
    Index(usize),
    /// UUID-based target (will be resolved to index if possible)
    Uuid(String),
}

/// A reference from one layer to another via a layer-switching keycode
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerRef {
    /// Source layer index (where the key is)
    pub from_layer: usize,
    /// Target layer index (which layer it activates)
    pub to_layer: usize,
    /// Position of the key in the source layer
    pub position: Position,
    /// Type of layer reference
    pub kind: LayerRefKind,
    /// The full keycode string (e.g., "MO(1)", "LT(2, KC_SPC)")
    pub keycode: String,
}

/// Parse a keycode to extract layer reference information
///
/// Returns `Some((target, kind))` if the keycode is a layer-switching keycode
/// with a numeric or UUID layer parameter. Returns `None` for non-layer keycodes
/// or malformed patterns.
///
/// # Examples
/// ```
/// use lazyqmk::services::layer_refs::{parse_layer_keycode, LayerRefKind, LayerRefTarget};
///
/// assert_eq!(
///     parse_layer_keycode("MO(1)"),
///     Some((LayerRefTarget::Index(1), LayerRefKind::Momentary))
/// );
/// assert_eq!(
///     parse_layer_keycode("LT(@abc, KC_SPC)"),
///     Some((LayerRefTarget::Uuid("@abc".to_string()), LayerRefKind::TapHold))
/// );
/// assert_eq!(parse_layer_keycode("KC_A"), None);
/// ```
#[must_use]
pub fn parse_layer_keycode(keycode: &str) -> Option<(LayerRefTarget, LayerRefKind)> {
    // Handle each known layer keycode prefix
    let (prefix, layer_str) = if let Some(inner) = keycode.strip_prefix("MO(") {
        ("MO", inner.strip_suffix(')')?)
    } else if let Some(inner) = keycode.strip_prefix("TG(") {
        ("TG", inner.strip_suffix(')')?)
    } else if let Some(inner) = keycode.strip_prefix("TO(") {
        ("TO", inner.strip_suffix(')')?)
    } else if let Some(inner) = keycode.strip_prefix("TT(") {
        ("TT", inner.strip_suffix(')')?)
    } else if let Some(inner) = keycode.strip_prefix("OSL(") {
        ("OSL", inner.strip_suffix(')')?)
    } else if let Some(inner) = keycode.strip_prefix("DF(") {
        ("DF", inner.strip_suffix(')')?)
    } else if let Some(inner) = keycode.strip_prefix("LT(") {
        // LT(layer, keycode) - extract just the layer part
        let inner = inner.strip_suffix(')')?;
        let (layer_part, _) = inner.split_once(',')?;
        ("LT", layer_part.trim())
    } else if let Some(inner) = keycode.strip_prefix("LM(") {
        // LM(layer, mod) - extract just the layer part
        let inner = inner.strip_suffix(')')?;
        let (layer_part, _) = inner.split_once(',')?;
        ("LM", layer_part.trim())
    } else {
        return None;
    };

    // Determine target: UUID (prefixed with '@') or numeric
    let target = if let Some(uuid) = layer_str.strip_prefix('@') {
        LayerRefTarget::Uuid(format!("@{uuid}"))
    } else if let Ok(layer_num) = layer_str.parse::<usize>() {
        LayerRefTarget::Index(layer_num)
    } else {
        return None;
    };

    // Map prefix to kind
    let kind = match prefix {
        "MO" => LayerRefKind::Momentary,
        "LT" => LayerRefKind::TapHold,
        "TG" => LayerRefKind::Toggle,
        "OSL" => LayerRefKind::OneShot,
        "TO" => LayerRefKind::SwitchTo,
        "TT" => LayerRefKind::TapToggle,
        "DF" => LayerRefKind::DefaultSet,
        "LM" => LayerRefKind::LayerMod,
        _ => LayerRefKind::Other,
    };

    Some((target, kind))
}

/// Build a reverse index of all layer references in the layout
///
/// Returns a map from target layer index to a list of all keys that reference it.
/// This is used to show "inbound references" when editing a layer.
///
/// # Examples
/// ```
/// use lazyqmk::models::{Layer, KeyDefinition, Position, RgbColor};
/// use lazyqmk::services::layer_refs::build_layer_ref_index;
///
/// let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
/// layer0.add_key(KeyDefinition::new(Position::new(0, 0), "MO(1)"));
///
/// let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
///
/// let layers = vec![layer0, layer1];
/// let index = build_layer_ref_index(&layers);
///
/// // Layer 1 has one inbound reference from layer 0
/// assert_eq!(index.get(&1).unwrap().len(), 1);
/// ```
#[must_use]
pub fn build_layer_ref_index(layers: &[Layer]) -> HashMap<usize, Vec<LayerRef>> {
    let mut index: HashMap<usize, Vec<LayerRef>> = HashMap::new();

    // Build map from layer UUID (string) to index for resolution
    let mut id_to_index: HashMap<String, usize> = HashMap::new();
    for (idx, layer) in layers.iter().enumerate() {
        id_to_index.insert(layer.id.clone(), idx);
    }

    for (from_layer_idx, layer) in layers.iter().enumerate() {
        for key in &layer.keys {
            // Skip transparent/no-op keys
            if key.is_transparent() || key.is_no_op() {
                continue;
            }

            // Try to parse as layer keycode
            if let Some((target, kind)) = parse_layer_keycode(&key.keycode) {
                match target {
                    LayerRefTarget::Index(to_layer) => {
                        // Only track references to existing layers
                        if to_layer < layers.len() {
                            let layer_ref = LayerRef {
                                from_layer: from_layer_idx,
                                to_layer,
                                position: key.position,
                                kind,
                                keycode: key.keycode.clone(),
                            };

                            index.entry(to_layer).or_default().push(layer_ref);
                        }
                    }
                    LayerRefTarget::Uuid(uuid) => {
                        // Strip leading '@' if present and resolve against layer IDs
                        let trimmed = uuid.strip_prefix('@').unwrap_or(uuid.as_str());
                        if let Some(&to_layer) = id_to_index.get(trimmed) {
                            let layer_ref = LayerRef {
                                from_layer: from_layer_idx,
                                to_layer,
                                position: key.position,
                                kind,
                                keycode: key.keycode.clone(),
                            };
                            index.entry(to_layer).or_default().push(layer_ref);
                        }
                    }
                }
            }
        }
    }

    index
}

/// Check if a keycode is transparent (allows fallthrough to lower layers)
#[must_use]
pub fn is_transparent(keycode: &str) -> bool {
    keycode == "KC_TRNS" || keycode == "KC_TRANSPARENT"
}

/// Check if assigning a keycode at a position with inbound hold-like references
/// would create a potential conflict (non-transparent key where transparency expected)
///
/// Returns `Some(warning_message)` if there's a potential conflict, `None` otherwise
#[must_use]
pub fn check_transparency_conflict(
    target_layer: usize,
    position: Position,
    new_keycode: &str,
    layer_refs: &HashMap<usize, Vec<LayerRef>>,
) -> Option<String> {
    // If the new keycode is transparent, no conflict
    if is_transparent(new_keycode) {
        return None;
    }

    // Get all references to this layer
    let refs = layer_refs.get(&target_layer)?;

    // Check if any hold-like reference targets this position
    let conflicting_refs: Vec<&LayerRef> = refs
        .iter()
        .filter(|r| r.position == position && r.kind.is_hold_like())
        .collect();

    if conflicting_refs.is_empty() {
        return None;
    }

    // Build warning message
    let ref_descriptions: Vec<String> = conflicting_refs
        .iter()
        .map(|r| format!("Layer {} {}", r.from_layer, r.kind.display_name()))
        .collect();

    Some(format!(
        "Warning: This position has hold-like references from {}. Consider using KC_TRNS to avoid conflicts.",
        ref_descriptions.join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{KeyDefinition, Layer, RgbColor};

    #[test]
    fn test_parse_layer_keycode_simple() {
        assert_eq!(
            parse_layer_keycode("MO(1)"),
            Some((LayerRefTarget::Index(1), LayerRefKind::Momentary))
        );
        assert_eq!(
            parse_layer_keycode("TG(2)"),
            Some((LayerRefTarget::Index(2), LayerRefKind::Toggle))
        );
        assert_eq!(
            parse_layer_keycode("TO(0)"),
            Some((LayerRefTarget::Index(0), LayerRefKind::SwitchTo))
        );
        assert_eq!(
            parse_layer_keycode("TT(3)"),
            Some((LayerRefTarget::Index(3), LayerRefKind::TapToggle))
        );
        assert_eq!(
            parse_layer_keycode("OSL(1)"),
            Some((LayerRefTarget::Index(1), LayerRefKind::OneShot))
        );
        assert_eq!(
            parse_layer_keycode("DF(0)"),
            Some((LayerRefTarget::Index(0), LayerRefKind::DefaultSet))
        );
    }

    #[test]
    fn test_parse_layer_keycode_compound() {
        assert_eq!(
            parse_layer_keycode("LT(2, KC_SPC)"),
            Some((LayerRefTarget::Index(2), LayerRefKind::TapHold))
        );
        assert_eq!(
            parse_layer_keycode("LT(1, KC_A)"),
            Some((LayerRefTarget::Index(1), LayerRefKind::TapHold))
        );
        assert_eq!(
            parse_layer_keycode("LM(3, MOD_LSFT)"),
            Some((LayerRefTarget::Index(3), LayerRefKind::LayerMod))
        );

        assert_eq!(
            parse_layer_keycode("LT(1, KC_A)"),
            Some((LayerRefTarget::Index(1), LayerRefKind::TapHold))
        );
        assert_eq!(
            parse_layer_keycode("LM(3, MOD_LSFT)"),
            Some((LayerRefTarget::Index(3), LayerRefKind::LayerMod))
        );
    }

    #[test]
    fn test_parse_layer_keycode_invalid() {
        // Non-layer keycodes
        assert_eq!(parse_layer_keycode("KC_A"), None);
        assert_eq!(parse_layer_keycode("KC_TRNS"), None);
        assert_eq!(parse_layer_keycode("LCTL_T(KC_A)"), None);

        // UUID references should parse to LayerRefTarget::Uuid
        assert_eq!(
            parse_layer_keycode("MO(@layer-id)"),
            Some((
                LayerRefTarget::Uuid("@layer-id".to_string()),
                LayerRefKind::Momentary
            ))
        );
        assert_eq!(
            parse_layer_keycode("LT(@abc-123, KC_SPC)"),
            Some((
                LayerRefTarget::Uuid("@abc-123".to_string()),
                LayerRefKind::TapHold
            ))
        );

        // Malformed
        assert_eq!(parse_layer_keycode("MO("), None);
        assert_eq!(parse_layer_keycode("MO(abc)"), None);
        assert_eq!(parse_layer_keycode("LT(1)"), None); // Missing second param
    }

    #[test]
    fn test_layer_ref_kind_is_hold_like() {
        assert!(LayerRefKind::Momentary.is_hold_like());
        assert!(LayerRefKind::TapHold.is_hold_like());
        assert!(LayerRefKind::TapToggle.is_hold_like());
        assert!(LayerRefKind::LayerMod.is_hold_like());

        assert!(!LayerRefKind::Toggle.is_hold_like());
        assert!(!LayerRefKind::OneShot.is_hold_like());
        assert!(!LayerRefKind::SwitchTo.is_hold_like());
        assert!(!LayerRefKind::DefaultSet.is_hold_like());
    }

    #[test]
    fn test_build_layer_ref_index_basic() {
        let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        layer0.add_key(KeyDefinition::new(Position::new(0, 0), "MO(1)"));
        layer0.add_key(KeyDefinition::new(Position::new(0, 1), "LT(2, KC_SPC)"));

        let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
        let layer2 = Layer::new(2, "Raise", RgbColor::new(0, 0, 255)).unwrap();

        let layers = vec![layer0, layer1, layer2];
        let index = build_layer_ref_index(&layers);

        // Layer 1 has one reference
        assert_eq!(index.get(&1).unwrap().len(), 1);
        assert_eq!(index.get(&1).unwrap()[0].kind, LayerRefKind::Momentary);
        assert_eq!(index.get(&1).unwrap()[0].position, Position::new(0, 0));

        // Layer 2 has one reference
        assert_eq!(index.get(&2).unwrap().len(), 1);
        assert_eq!(index.get(&2).unwrap()[0].kind, LayerRefKind::TapHold);
        assert_eq!(index.get(&2).unwrap()[0].position, Position::new(0, 1));

        // Layer 0 has no inbound references
        assert!(index.get(&0).is_none());
    }

    #[test]
    fn test_build_layer_ref_index_multiple_refs() {
        let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        layer0.add_key(KeyDefinition::new(Position::new(0, 0), "MO(1)"));
        layer0.add_key(KeyDefinition::new(Position::new(0, 1), "TG(1)"));

        let mut layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
        layer1.add_key(KeyDefinition::new(Position::new(1, 0), "MO(1)")); // Self-reference

        let layers = vec![layer0, layer1];
        let index = build_layer_ref_index(&layers);

        // Layer 1 has three references (2 from layer 0, 1 self-reference)
        assert_eq!(index.get(&1).unwrap().len(), 3);
    }

    #[test]
    fn test_build_layer_ref_index_ignores_invalid() {
        let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        layer0.add_key(KeyDefinition::new(Position::new(0, 0), "KC_A")); // Not a layer keycode
        layer0.add_key(KeyDefinition::new(Position::new(0, 1), "KC_TRNS")); // Transparent
        layer0.add_key(KeyDefinition::new(Position::new(0, 2), "MO(@uuid)")); // UUID ref
        layer0.add_key(KeyDefinition::new(Position::new(0, 3), "MO(99)")); // Out of bounds

        let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();

        let layers = vec![layer0, layer1];
        let index = build_layer_ref_index(&layers);

        // No valid references
        assert!(index.is_empty());
    }

    #[test]
    fn test_is_transparent() {
        assert!(is_transparent("KC_TRNS"));
        assert!(is_transparent("KC_TRANSPARENT"));
        assert!(!is_transparent("KC_A"));
        assert!(!is_transparent("MO(1)"));
    }

    #[test]
    fn test_check_transparency_conflict_no_conflict() {
        let mut index = HashMap::new();
        index.insert(
            1,
            vec![LayerRef {
                from_layer: 0,
                to_layer: 1,
                position: Position::new(0, 0),
                kind: LayerRefKind::Momentary,
                keycode: "MO(1)".to_string(),
            }],
        );

        // No conflict if assigning transparent
        assert!(check_transparency_conflict(1, Position::new(0, 0), "KC_TRNS", &index).is_none());

        // No conflict if position doesn't have references
        assert!(check_transparency_conflict(1, Position::new(1, 1), "KC_A", &index).is_none());

        // No conflict if layer has no inbound references
        assert!(check_transparency_conflict(2, Position::new(0, 0), "KC_A", &index).is_none());
    }

    #[test]
    fn test_check_transparency_conflict_with_conflict() {
        let mut index = HashMap::new();
        index.insert(
            1,
            vec![LayerRef {
                from_layer: 0,
                to_layer: 1,
                position: Position::new(0, 0),
                kind: LayerRefKind::Momentary,
                keycode: "MO(1)".to_string(),
            }],
        );

        // Conflict: non-transparent key at position with hold-like reference
        let warning = check_transparency_conflict(1, Position::new(0, 0), "KC_A", &index);
        assert!(warning.is_some());
        let msg = warning.unwrap();
        assert!(msg.contains("Layer 0"));
        assert!(msg.contains("Momentary"));
    }

    #[test]
    fn test_check_transparency_conflict_only_hold_like() {
        let mut index = HashMap::new();
        index.insert(
            1,
            vec![
                // Hold-like - should trigger warning
                LayerRef {
                    from_layer: 0,
                    to_layer: 1,
                    position: Position::new(0, 0),
                    kind: LayerRefKind::TapHold,
                    keycode: "LT(1, KC_SPC)".to_string(),
                },
                // Not hold-like - should NOT trigger warning
                LayerRef {
                    from_layer: 0,
                    to_layer: 1,
                    position: Position::new(0, 1),
                    kind: LayerRefKind::Toggle,
                    keycode: "TG(1)".to_string(),
                },
            ],
        );

        // Position with TapHold should warn
        let warning = check_transparency_conflict(1, Position::new(0, 0), "KC_A", &index);
        assert!(warning.is_some());

        // Position with Toggle should NOT warn
        let warning = check_transparency_conflict(1, Position::new(0, 1), "KC_A", &index);
        assert!(warning.is_none());
    }

    #[test]
    fn test_check_transparency_conflict_multiple_refs() {
        let mut index = HashMap::new();
        index.insert(
            1,
            vec![
                LayerRef {
                    from_layer: 0,
                    to_layer: 1,
                    position: Position::new(0, 0),
                    kind: LayerRefKind::Momentary,
                    keycode: "MO(1)".to_string(),
                },
                LayerRef {
                    from_layer: 2,
                    to_layer: 1,
                    position: Position::new(0, 0),
                    kind: LayerRefKind::TapHold,
                    keycode: "LT(1, KC_A)".to_string(),
                },
            ],
        );

        let warning = check_transparency_conflict(1, Position::new(0, 0), "KC_B", &index);
        assert!(warning.is_some());
        let msg = warning.unwrap();
        // Should mention both layers
        assert!(msg.contains("Layer 0"));
        assert!(msg.contains("Layer 2"));
    }
}
