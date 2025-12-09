# Implementation Plan: Robust Keyboard Picker

## Overview

This plan details the implementation of a robust keyboard selection and validation system that handles QMK firmware's complex directory structures and prevents user errors.

## Goals

1. **Prevent invalid keyboard names** from reaching the parser
2. **Provide helpful error messages** with suggestions for typos
3. **Handle all QMK keyboard structure patterns** robustly
4. **Improve user experience** during keyboard selection
5. **Maintain backward compatibility** with existing layouts

## Non-Goals

- Modifying QMK firmware structure
- Supporting non-QMK keyboards
- Implementing keyboard auto-detection via USB

## Architecture

### New Components

```
src/parser/keyboard_json.rs
  ├── validate_keyboard_name()      [NEW] - Validates keyboard against QMK CLI
  ├── suggest_similar_keyboards()   [NEW] - Fuzzy matching for typos
  ├── discover_keyboard_config()    [NEW] - Systematic config file discovery
  └── parse_keyboard_info_json()    [ENHANCED] - Better error messages

src/services/keyboard_validator.rs [NEW] - Keyboard validation service
  ├── KeyboardValidator
  ├── ValidationResult
  └── KeyboardConfig (discovered files)
```

### Data Structures

```rust
/// Result of keyboard validation
pub struct ValidationResult {
    pub is_valid: bool,
    pub keyboard_name: String,
    pub suggestions: Vec<String>,
    pub error_message: Option<String>,
}

/// Discovered keyboard configuration files
pub struct KeyboardConfig {
    pub keyboard_name: String,
    pub keyboard_dir: PathBuf,
    pub info_json: Option<PathBuf>,
    pub keyboard_json: Option<PathBuf>,
    pub parent_info_json: Option<PathBuf>,
    pub has_layouts: bool,
}

/// Fuzzy match result with confidence score
pub struct SimilarKeyboard {
    pub name: String,
    pub similarity: f32,  // 0.0 - 1.0
}
```

## Implementation Steps

### Phase 1: Validation Layer (Priority: High)

#### Step 1.1: Create KeyboardValidator Service
**File**: `src/services/keyboard_validator.rs`

```rust
pub struct KeyboardValidator {
    qmk_path: PathBuf,
    valid_keyboards: Vec<String>,
}

impl KeyboardValidator {
    /// Create validator and cache valid keyboards list
    pub fn new(qmk_path: &Path) -> Result<Self>;
    
    /// Validate keyboard name against QMK CLI list
    pub fn validate(&self, keyboard: &str) -> ValidationResult;
    
    /// Find similar keyboard names (fuzzy matching)
    pub fn find_similar(&self, keyboard: &str, max_results: usize) -> Vec<SimilarKeyboard>;
    
    /// Refresh cached keyboard list from QMK CLI
    pub fn refresh_cache(&mut self) -> Result<()>;
}
```

**Implementation Details**:
- Cache `qmk list-keyboards` output on initialization
- Use case-insensitive comparison
- Implement Levenshtein distance for fuzzy matching
- Handle common OCR errors: `1` ↔ `l`, `0` ↔ `O`, `I` ↔ `1`

#### Step 1.2: Add Validation to Parser
**File**: `src/parser/keyboard_json.rs`

Add validation function:
```rust
/// Validates keyboard name against QMK CLI list
pub fn validate_keyboard_name(qmk_path: &Path, keyboard: &str) -> Result<ValidationResult> {
    let validator = KeyboardValidator::new(qmk_path)?;
    Ok(validator.validate(keyboard))
}
```

Enhance `parse_keyboard_info_json()`:
```rust
pub fn parse_keyboard_info_json(qmk_path: &Path, keyboard: &str) -> Result<QmkInfoJson> {
    // Validate keyboard name first
    let validation = validate_keyboard_name(qmk_path, keyboard)?;
    
    if !validation.is_valid {
        let mut error_msg = format!("Keyboard '{}' not found in QMK firmware.", keyboard);
        
        if !validation.suggestions.is_empty() {
            error_msg.push_str("\n\nDid you mean:");
            for suggestion in validation.suggestions.iter().take(3) {
                error_msg.push_str(&format!("\n  - {}", suggestion));
            }
        }
        
        anyhow::bail!(error_msg);
    }
    
    // Existing discovery logic...
}
```

### Phase 2: Config Discovery (Priority: High)

#### Step 2.1: Implement discover_keyboard_config()
**File**: `src/parser/keyboard_json.rs`

```rust
/// Discovers all configuration files for a keyboard
pub fn discover_keyboard_config(qmk_path: &Path, keyboard: &str) -> Result<KeyboardConfig> {
    let keyboards_dir = qmk_path.join("keyboards");
    let keyboard_dir = keyboards_dir.join(keyboard);
    
    let mut config = KeyboardConfig {
        keyboard_name: keyboard.to_string(),
        keyboard_dir: keyboard_dir.clone(),
        info_json: None,
        keyboard_json: None,
        parent_info_json: None,
        has_layouts: false,
    };
    
    // Check variant directory
    let info_json_path = keyboard_dir.join("info.json");
    let keyboard_json_path = keyboard_dir.join("keyboard.json");
    
    if info_json_path.exists() {
        config.info_json = Some(info_json_path);
    }
    
    if keyboard_json_path.exists() {
        config.keyboard_json = Some(keyboard_json_path);
    }
    
    // Check parent directories for info.json
    if let Some((parent, _)) = keyboard.rsplit_once('/') {
        let parent_info_json = keyboards_dir.join(parent).join("info.json");
        if parent_info_json.exists() {
            config.parent_info_json = Some(parent_info_json);
        }
    }
    
    // Determine if keyboard has layouts
    config.has_layouts = check_for_layouts(&config)?;
    
    Ok(config)
}

fn check_for_layouts(config: &KeyboardConfig) -> Result<bool> {
    // Check info.json for layouts
    if let Some(ref path) = config.info_json {
        let info = parse_info_json(path)?;
        if !info.layouts.is_empty() {
            return Ok(true);
        }
    }
    
    // Check keyboard.json for layouts
    if let Some(ref path) = config.keyboard_json {
        let variant = parse_variant_json(path)?;
        if !variant.layouts.is_empty() {
            return Ok(true);
        }
    }
    
    // Check parent info.json
    if let Some(ref path) = config.parent_info_json {
        let info = parse_info_json(path)?;
        if !info.layouts.is_empty() {
            return Ok(true);
        }
    }
    
    Ok(false)
}
```

#### Step 2.2: Refactor parse_keyboard_info_json()
**File**: `src/parser/keyboard_json.rs`

Simplify using discovery:
```rust
pub fn parse_keyboard_info_json(qmk_path: &Path, keyboard: &str) -> Result<QmkInfoJson> {
    // Validate keyboard name
    let validation = validate_keyboard_name(qmk_path, keyboard)?;
    if !validation.is_valid {
        return Err(create_validation_error(&validation));
    }
    
    // Discover configuration files
    let config = discover_keyboard_config(qmk_path, keyboard)?;
    
    // Load and merge configuration
    let info = load_merged_config(&config)?;
    
    // Validate layouts exist
    if info.layouts.is_empty() {
        anyhow::bail!(
            "Keyboard '{}' has no layouts defined.\n\
            Found config files: {}\n\
            This keyboard may not be fully configured in QMK firmware.",
            keyboard,
            format_found_files(&config)
        );
    }
    
    Ok(info)
}

fn load_merged_config(config: &KeyboardConfig) -> Result<QmkInfoJson> {
    // Try loading from variant directory first
    let mut info = if let Some(ref path) = config.info_json {
        parse_info_json(path)?
    } else if let Some(ref parent_path) = config.parent_info_json {
        parse_info_json(parent_path)?
    } else if let Some(ref kb_path) = config.keyboard_json {
        // Create QmkInfoJson from keyboard.json
        let variant = parse_variant_json(kb_path)?;
        QmkInfoJson {
            keyboard_name: variant.keyboard_name,
            manufacturer: None,
            maintainer: None,
            url: None,
            layouts: variant.layouts,
            matrix_pins: None,
            encoder: variant.encoder,
        }
    } else {
        anyhow::bail!(
            "No configuration files found for keyboard '{}' at {}",
            config.keyboard_name,
            config.keyboard_dir.display()
        );
    };
    
    // Merge layouts from keyboard.json if present
    if info.layouts.is_empty() {
        if let Some(ref kb_path) = config.keyboard_json {
            let variant = parse_variant_json(kb_path)?;
            if !variant.layouts.is_empty() {
                info.layouts = variant.layouts;
            }
            // Also merge encoder if parent doesn't have it
            if info.encoder.is_none() {
                info.encoder = variant.encoder;
            }
        }
    }
    
    Ok(info)
}

fn format_found_files(config: &KeyboardConfig) -> String {
    let mut files = Vec::new();
    if config.info_json.is_some() {
        files.push("info.json");
    }
    if config.keyboard_json.is_some() {
        files.push("keyboard.json");
    }
    if config.parent_info_json.is_some() {
        files.push("parent/info.json");
    }
    if files.is_empty() {
        "none".to_string()
    } else {
        files.join(", ")
    }
}
```

### Phase 3: Fuzzy Matching (Priority: Medium)

#### Step 3.1: Implement Levenshtein Distance
**File**: `src/services/keyboard_validator.rs`

```rust
/// Computes Levenshtein distance between two strings
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();
    
    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }
    
    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];
    
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }
    
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i-1] == b_chars[j-1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i-1][j] + 1,      // deletion
                    matrix[i][j-1] + 1       // insertion
                ),
                matrix[i-1][j-1] + cost      // substitution
            );
        }
    }
    
    matrix[a_len][b_len]
}

/// Computes similarity score (0.0 - 1.0) with OCR error handling
fn similarity_score(input: &str, candidate: &str) -> f32 {
    // Normalize strings
    let input_norm = normalize_for_ocr(input);
    let candidate_norm = normalize_for_ocr(candidate);
    
    let distance = levenshtein_distance(&input_norm, &candidate_norm);
    let max_len = std::cmp::max(input_norm.len(), candidate_norm.len());
    
    if max_len == 0 {
        return 1.0;
    }
    
    1.0 - (distance as f32 / max_len as f32)
}

/// Normalizes string for OCR error handling
fn normalize_for_ocr(s: &str) -> String {
    s.to_lowercase()
        .replace('l', "1")  // Common OCR error
        .replace('o', "0")  // Common OCR error
        .replace('i', "1")  // Common OCR error
}
```

#### Step 3.2: Implement find_similar()
```rust
impl KeyboardValidator {
    pub fn find_similar(&self, keyboard: &str, max_results: usize) -> Vec<SimilarKeyboard> {
        let mut similarities: Vec<(String, f32)> = self.valid_keyboards
            .iter()
            .map(|kb| {
                let score = similarity_score(keyboard, kb);
                (kb.clone(), score)
            })
            .collect();
        
        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Filter: only show if similarity > 0.6
        similarities.into_iter()
            .filter(|(_, score)| *score > 0.6)
            .take(max_results)
            .map(|(name, similarity)| SimilarKeyboard { name, similarity })
            .collect()
    }
}
```

### Phase 4: UI Integration (Priority: High)

#### Step 4.1: Update Onboarding Wizard
**File**: `src/tui/onboarding_wizard.rs`

Enhance error handling:
```rust
WizardStep::KeyboardSelection => {
    // ... existing keyboard selection logic ...
    
    // Parse keyboard info.json to get layouts
    let qmk_path = PathBuf::from(self.inputs.get("qmk_path").unwrap());
    match parse_keyboard_info_json(&qmk_path, &keyboard) {
        Ok(info) => {
            self.available_layouts = extract_layout_names(&info);
            self.layout_selected_index = 0;
            self.current_step = WizardStep::LayoutSelection;
        }
        Err(e) => {
            // Enhanced error message with suggestions
            self.error_message = Some(format_keyboard_error(&e));
        }
    }
}

fn format_keyboard_error(error: &anyhow::Error) -> String {
    let error_str = error.to_string();
    
    // Check if error contains suggestions
    if error_str.contains("Did you mean:") {
        error_str  // Already formatted with suggestions
    } else {
        format!("Failed to load keyboard: {}", error_str)
    }
}
```

#### Step 4.2: Add Keyboard Validation Preview
**File**: `src/tui/onboarding_wizard.rs`

Show validation status during keyboard selection:
```rust
fn render_keyboard_selection(
    f: &mut Frame,
    state: &OnboardingWizardState,
    area: Rect,
    theme: &crate::tui::theme::Theme,
) {
    // ... existing rendering ...
    
    // Add status line showing validation for selected keyboard
    if let Some(keyboard) = get_currently_selected_keyboard(state) {
        let status_text = match validate_keyboard_quick(&keyboard) {
            Ok(true) => format!("✓ {} (valid)", keyboard),
            Ok(false) => format!("✗ {} (not found)", keyboard),
            Err(_) => keyboard.clone(),
        };
        
        let status = Paragraph::new(status_text)
            .style(Style::default().fg(theme.text_muted));
        f.render_widget(status, status_area);
    }
}
```

### Phase 5: Testing (Priority: High)

#### Step 5.1: Unit Tests
**File**: `src/parser/keyboard_json.rs` (tests module)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_keyboard_name_valid() {
        let qmk_path = get_test_qmk_path();
        let result = validate_keyboard_name(&qmk_path, "crkbd").unwrap();
        assert!(result.is_valid);
        assert!(result.suggestions.is_empty());
    }
    
    #[test]
    fn test_validate_keyboard_name_typo() {
        let qmk_path = get_test_qmk_path();
        let result = validate_keyboard_name(&qmk_path, "lupkeyboards/super16v3").unwrap();
        assert!(!result.is_valid);
        assert!(!result.suggestions.is_empty());
        assert!(result.suggestions[0].contains("1upkeyboards"));
    }
    
    #[test]
    fn test_discover_keyboard_config_variants() {
        let qmk_path = get_test_qmk_path();
        
        // Test keyboard with parent info.json + variant keyboard.json
        let config = discover_keyboard_config(&qmk_path, "1upkeyboards/pi50/grid").unwrap();
        assert!(config.keyboard_json.is_some());
        assert!(config.parent_info_json.is_some());
        assert!(config.has_layouts);
    }
    
    #[test]
    fn test_discover_keyboard_config_keyboard_json_only() {
        let qmk_path = get_test_qmk_path();
        
        // Test keyboard with only keyboard.json
        let config = discover_keyboard_config(&qmk_path, "1upkeyboards/1upsuper16v3").unwrap();
        assert!(config.keyboard_json.is_some());
        assert!(config.info_json.is_none());
        assert!(config.has_layouts);
    }
}
```

#### Step 5.2: Integration Tests
**File**: `tests/keyboard_validation_tests.rs`

```rust
#[test]
fn test_parse_all_structure_patterns() {
    let qmk_path = get_qmk_firmware_path();
    
    let test_cases = vec![
        // Pattern 1: info.json only
        "crkbd",
        // Pattern 2: Parent + variant
        "1upkeyboards/pi50/grid",
        // Pattern 3: keyboard.json only
        "1upkeyboards/1upsuper16v3",
        // Pattern 4: Deep nesting
        "splitkb/aurora/corne/rev1",
    ];
    
    for keyboard in test_cases {
        let result = parse_keyboard_info_json(&qmk_path, keyboard);
        assert!(result.is_ok(), "Failed to parse {}: {:?}", keyboard, result.err());
        
        let info = result.unwrap();
        assert!(!info.layouts.is_empty(), "No layouts for {}", keyboard);
    }
}

#[test]
fn test_fuzzy_matching_ocr_errors() {
    let qmk_path = get_qmk_firmware_path();
    let validator = KeyboardValidator::new(&qmk_path).unwrap();
    
    // Test OCR error: 1 → l
    let similar = validator.find_similar("lupkeyboards/lupsuper16v3", 5);
    assert!(!similar.is_empty());
    assert_eq!(similar[0].name, "1upkeyboards/1upsuper16v3");
    assert!(similar[0].similarity > 0.8);
}
```

### Phase 6: Documentation (Priority: Medium)

#### Step 6.1: Update Developer Docs
**File**: `docs/ARCHITECTURE.md`

Add section on keyboard validation:
```markdown
## Keyboard Validation

The system validates keyboard names against QMK CLI output before attempting
to parse configuration files. This prevents errors from typos or invalid names.

### Validation Flow

1. User selects keyboard → Name validated against cached QMK list
2. If invalid → Show suggestions using fuzzy matching
3. If valid → Discover configuration files
4. Load and merge configurations → Parse layouts

### Configuration Discovery

QMK keyboards can have various file structures:
- `info.json` only (simple keyboards)
- `keyboard.json` only (some variants)
- Parent `info.json` + variant `keyboard.json`
- Deep nesting (5+ directory levels)

The discovery system handles all patterns systematically.
```

#### Step 6.2: Update User Documentation
**File**: `QUICKSTART.md`

Add troubleshooting section:
```markdown
## Troubleshooting

### "Keyboard not found" Error

If you see an error like:
```
Keyboard 'lupkeyboards/lupsuper16v3' not found.

Did you mean:
  - 1upkeyboards/1upsuper16v3
  - 1upkeyboards/super16v2
```

This means the keyboard name has a typo. Select one of the suggested corrections.

**Common Issues:**
- OCR errors: `1` misread as `l`, `0` as `O`
- Copy-paste errors: Extra spaces, wrong casing
- Old keyboard names: Some keyboards were renamed in QMK firmware

**Solution:** Use the keyboard picker instead of typing manually.
```

## Implementation Checklist

### Phase 1: Validation (Week 1)
- [ ] Create `KeyboardValidator` service
- [ ] Implement `validate_keyboard_name()`
- [ ] Add validation to `parse_keyboard_info_json()`
- [ ] Write unit tests for validation
- [ ] Test with real QMK firmware

### Phase 2: Discovery (Week 1)
- [ ] Implement `discover_keyboard_config()`
- [ ] Implement `load_merged_config()`
- [ ] Refactor `parse_keyboard_info_json()` to use discovery
- [ ] Handle keyboards with only `keyboard.json`
- [ ] Write unit tests for discovery
- [ ] Test all keyboard structure patterns

### Phase 3: Fuzzy Matching (Week 2)
- [ ] Implement Levenshtein distance algorithm
- [ ] Implement `similarity_score()` with OCR normalization
- [ ] Implement `find_similar()` in validator
- [ ] Test fuzzy matching with common typos
- [ ] Test OCR error handling (`1` ↔ `l`, etc.)

### Phase 4: UI Integration (Week 2)
- [ ] Update onboarding wizard error handling
- [ ] Add validation preview to keyboard selector
- [ ] Test end-to-end keyboard selection flow
- [ ] Test error messages with real users

### Phase 5: Testing (Week 2)
- [ ] Write unit tests for all new functions
- [ ] Write integration tests with QMK firmware
- [ ] Test all keyboard structure patterns
- [ ] Test error recovery flows
- [ ] Test fuzzy matching accuracy

### Phase 6: Documentation (Week 2)
- [ ] Update `ARCHITECTURE.md`
- [ ] Update `QUICKSTART.md`
- [ ] Add inline code documentation
- [ ] Write examples for common use cases

## Success Criteria

1. ✅ Validation catches typos before parser errors
2. ✅ Fuzzy matching suggests correct keyboards for common typos
3. ✅ All QMK keyboard structure patterns work correctly
4. ✅ Error messages are clear and actionable
5. ✅ No regressions in existing layout loading
6. ✅ All tests pass with real QMK firmware
7. ✅ User can recover from errors without restarting

## Rollback Plan

If critical issues arise:
1. Revert validation layer (keep existing parser)
2. Add feature flag to disable validation
3. Keep existing error messages as fallback
4. Roll back UI changes independently

## Future Enhancements

1. **Keyboard Metadata Cache**: Pre-compute all keyboard configs
2. **Visual Preview**: Show keyboard layout before selection
3. **Favorites/Recents**: Quick access to commonly used keyboards
4. **Bulk Validation**: Check all saved layouts for valid keyboards
5. **Auto-Update**: Detect QMK firmware updates and refresh cache

## Dependencies

- QMK CLI (`qmk list-keyboards` command)
- QMK firmware submodule at `qmk_firmware/`
- Existing parser functions (refactored, not replaced)

## Timeline

- **Week 1**: Phases 1-2 (Validation + Discovery)
- **Week 2**: Phases 3-6 (Fuzzy Matching + UI + Testing + Docs)
- **Total**: 2 weeks for full implementation

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Validation too strict | Low | Medium | Add bypass for advanced users |
| Fuzzy matching inaccurate | Medium | Low | Tune threshold, add manual override |
| Performance with 3699 keyboards | Low | Medium | Cache results, lazy load |
| Breaking existing layouts | Low | High | Comprehensive testing, gradual rollout |

## Conclusion

This plan provides a systematic approach to fixing the keyboard picker fragility
by adding proper validation, discovery, and error recovery mechanisms. The
implementation is phased to allow incremental testing and rollback if needed.
