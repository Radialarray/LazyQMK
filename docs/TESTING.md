# Testing Guide

This document provides a comprehensive guide to running, writing, and understanding tests in the LazyQMK project.

## Running Tests

### Fast Unit & Integration Tests

Run the test suite without QMK-dependent tests (completes in <30 seconds):

```bash
cargo test --tests
```

This runs all unit tests (in `src/`) and integration tests (in `tests/`) that don't require the QMK firmware submodule.

### All Tests Including QMK-dependent

Run the full test suite including contract tests that depend on the QMK submodule:

```bash
cargo test -- --ignored
```

**Note:** Requires the QMK submodule to be initialized:

```bash
git submodule update --init --recursive qmk_firmware
```

### Updating Golden Files

When you update firmware generation logic, you need to update expected output files:

```bash
UPDATE_GOLDEN=1 cargo test --tests
```

This regenerates all golden files with the new output. Always review the changes before committing.

### Running Specific Tests

Run tests matching a pattern:

```bash
# Run all CLI tests
cargo test --test cli_*

# Run only generate command tests
cargo test --test cli_generate_tests

# Run a single test
cargo test test_generate_basic_succeeds

# Run tests matching a pattern with output
cargo test --test firmware_gen_tests -- --nocapture
```

### Running Clippy (Linter)

Run the Rust linter with zero-warning policy:

```bash
cargo clippy --all-features -- -D warnings
```

This must pass before committing. Never use `allow` or `ignore` flags; fix the underlying issue instead.

### Quick Build Check

For faster iteration during development:

```bash
cargo check
```

This catches compilation errors without generating binaries.

## Test Structure

The project organizes tests in a clear hierarchy:

### `tests/fixtures/mod.rs`

Shared test fixtures and builders for creating test data:

- **Layout fixtures**: `test_layout_basic()`, `test_layout_with_tap_dances()`, `test_layout_with_idle_effect()`, etc.
- **Geometry fixtures**: `test_geometry_basic()`, `test_mapping_basic()`
- **Config fixtures**: `temp_config_with_qmk()`
- **File helpers**: `create_temp_layout_file()`, `write_layout_file()`

Benefits:
- Consistent test data across all tests
- Deterministic timestamps and UUIDs for predictable output
- Temp directory management
- Reduces duplication and maintenance burden

### `tests/golden/`

Expected output files for golden tests:

```
golden/
  config_basic.h                    # Expected config.h for basic layout
  config_idle_effect.h              # Expected config.h with idle effect
  keymap_basic.c                    # Expected keymap.c for basic layout
  keymap_idle_effect_on.c           # Expected keymap.c with idle effect enabled
  keymap_tap_dances.c               # Expected keymap.c with tap dances
```

Golden tests compare generated code against these expected files, with automatic normalization for timestamps, UUIDs, and paths. See [Golden Testing](#golden-testing) for details.

### `tests/cli_*.rs`

End-to-end CLI command tests. Test the actual command-line interface and exit codes:

- `cli_generate_tests.rs` - Tests for `lazyqmk generate` command
- `cli_validate_tests.rs` - Tests for `lazyqmk validate` command
- `cli_inspect_tests.rs` - Tests for `lazyqmk inspect` command
- `cli_keycode_tests.rs` - Tests for `lazyqmk keycode` command
- `cli_tap_dance_tests.rs` - Tests for tap dance CLI features
- `cli_layer_refs_tests.rs` - Tests for layer reference resolution

These tests:
- Execute the actual compiled binary
- Verify exit codes and output format
- Test error cases and edge conditions
- Validate JSON output structure

### `tests/*_tests.rs`

Unit and integration tests for specific modules:

- `firmware_gen_tests.rs` - Firmware generation pipeline
- `layer_refs_tests.rs` - Layer reference resolution
- `tap_dance_tests.rs` - Tap dance handling
- `qmk_info_json_tests.rs` - QMK metadata parsing
- `layer_navigation_tests.rs` - Layer navigation logic

These tests:
- Import and use internal APIs directly
- Verify business logic in isolation
- Test error conditions thoroughly
- Use fixtures for consistent test data

## Test Statistics

LazyQMK has comprehensive test coverage:

- **Total tests**: ~970 tests
- **CLI E2E tests**: 156 tests across 12 command categories
- **Integration tests**: ~800 tests
- **Unit tests**: ~10 tests
- **Ignored tests (manual)**: 4 tests (pre-release validation only)
- **Golden files**: 5 files for firmware generation regression testing

**Test execution time:**
- Fast suite (`cargo test --tests`): <2 seconds
- Full suite with ignored tests: ~10 seconds

## Test Categories

### Unit Tests

Located in `src/` files, marked with `#[test]`:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_function_behavior() {
        let result = function_under_test();
        assert_eq!(result, expected);
    }
}
```

**Characteristics:**
- Test individual functions or methods
- Fast execution (<10ms typically)
- Can run in parallel
- Import internal private items

### Integration Tests

Located in `tests/` directory, test multiple modules together:

```rust
use lazyqmk::firmware::FirmwareGenerator;
use lazyqmk::models::Layout;

#[test]
fn test_generation_pipeline() {
    let layout = test_layout_basic(2, 3);
    let result = FirmwareGenerator::generate(&layout);
    assert!(result.is_ok());
}
```

**Characteristics:**
- Test public APIs and workflows
- Can use fixtures for setup
- Slightly slower than unit tests
- Run sequentially by default

### E2E CLI Tests

Located in `tests/cli_*.rs`, test the compiled binary:

```rust
use std::process::Command;

#[test]
fn test_cli_command() {
    let output = Command::new(lazyqmk_bin())
        .args(["generate", "--layout", "test.md"])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
}
```

**Characteristics:**
- Test complete CLI workflows
- Verify exit codes and signal handling
- Test input/output redirection
- Validate user-facing behavior

### Golden Tests

Tests that compare generated output against expected files:

```rust
#[test]
fn test_generate_matches_golden() {
    let layout = test_layout_basic(2, 3);
    let generated = generate_firmware(&layout);
    assert_golden(&generated, "tests/golden/keymap_basic.c");
}
```

**Characteristics:**
- Detect unintended changes to generated code
- Support updating with `UPDATE_GOLDEN=1`
- Normalize non-deterministic elements
- Essential for firmware generation testing

### Mock QMK Fixture Tests

**Most QMK-dependent tests now use lightweight fixtures instead of the full submodule!**

QMK metadata commands (`list-keyboards`, `list-layouts`, `geometry`) use a mock QMK structure in `tests/fixtures/mock_qmk/` for fast CI-friendly testing:

```
tests/fixtures/mock_qmk/
  keyboards/
    crkbd/
      info.json           # Real crkbd keyboard metadata
    keebart/
      corne_choc_pro/
        info.json         # Real corne_choc_pro keyboard metadata
    planck/
      info.json           # Real planck keyboard metadata
```

**How it works:**

Commands check for `LAZYQMK_QMK_FIXTURE` environment variable:

```rust
#[test]
fn test_list_keyboards_with_fixture() {
    let fixture_path = PathBuf::from("tests/fixtures/mock_qmk");
    
    let output = Command::new(lazyqmk_bin())
        .args(["list-keyboards", "--qmk-path", "dummy"])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
}
```

**Benefits:**
- ✅ No QMK submodule needed (saves 500MB+ download)
- ✅ Fast tests (<1 second vs 5-10 seconds)
- ✅ Run in CI without special setup
- ✅ Deterministic output
- ✅ 18 QMK metadata tests run automatically

**When to use the real QMK submodule:**
Only for the 4 manual pre-release tests (see below).

### Manual Pre-Release Tests (Ignored)

Only **4 tests** remain ignored for manual pre-release validation. These require full QMK compilation or are deprecated:

```bash
# Find ignored tests
cargo test -- --list --ignored
```

**The 4 ignored tests:**

1. **`test_generation_vial_json_structure`** (deprecated)
   - File: `tests/firmware_gen_tests.rs`
   - Reason: Vial support removed, can be deleted
   - Status: Safe to remove

2. **`test_scan_keyboards_finds_crkbd`** (QMK CLI integration)
   - File: `tests/qmk_info_json_tests.rs`
   - Reason: Requires QMK CLI + submodule for end-to-end validation
   - Status: Manual pre-release validation only

3. **`test_tap_dance_add_use_generate`** (full pipeline)
   - File: `tests/cli_tap_dance_tests.rs`
   - Reason: Requires full QMK compilation (slow, 5-10 minutes)
   - Status: Manual pre-release validation only

4. **`test_check_deprecated_options_clean`** (deprecated)
   - File: `src/firmware/validator.rs`
   - Reason: Vial-specific checks no longer needed after migration to standard QMK
   - Status: Deprecated, can be removed

**Run ignored tests:**

```bash
# All ignored tests
cargo test -- --ignored

# Specific test
cargo test test_scan_keyboards_finds_crkbd -- --ignored
```

**Requirements for ignored tests:**
```bash
# Initialize QMK submodule
git submodule update --init --recursive qmk_firmware

# Optional: Install QMK CLI for full pipeline test
pip3 install qmk
qmk setup
```

**Characteristics:**
- Require QMK submodule (500MB+)
- Slow execution (5-10 minutes for compilation tests)
- Only needed for pre-release validation
- Documented in AGENTS.md pre-release checklist

## Writing New Tests

### Golden Test Workflow

When testing firmware generation:

1. **Create test layout** using fixtures:
   ```rust
   #[test]
   fn test_generate_new_feature() {
       let layout = test_layout_basic(2, 3);
       // Modify layout to test new feature
       layout.some_new_setting = true;
       
       let generated = generate_firmware(&layout);
       assert_golden(&generated, "tests/golden/keymap_new_feature.c");
   }
   ```

2. **Run test to generate golden file**:
   ```bash
   UPDATE_GOLDEN=1 cargo test test_generate_new_feature
   ```

3. **Review the generated file**:
   ```bash
   cat tests/golden/keymap_new_feature.c
   ```

4. **Verify it's correct**, then commit:
   ```bash
   git add tests/golden/keymap_new_feature.c
   git commit -m "test: add golden file for new feature"
   ```

5. **Run test normally** to verify comparison works:
   ```bash
   cargo test test_generate_new_feature
   ```

### CLI E2E Test Patterns

Testing command-line interface:

```rust
#[test]
fn test_validate_json_output() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);

    let output = Command::new(lazyqmk_bin())
        .args(["validate", "--layout", layout_path.to_str().unwrap(), "--json"])
        .output()
        .expect("Failed to execute command");

    // Test exit code
    assert_eq!(
        output.status.code(),
        Some(0),
        "Should exit successfully. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Parse and validate JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should parse JSON output");

    assert_eq!(result["valid"], true);
    assert!(result["errors"].is_array());
    assert_eq!(result["errors"].as_array().unwrap().len(), 0);
}
```

**Key patterns:**
- Use `Command::new()` to spawn the binary
- Test exit codes: `output.status.code() == Some(0)`
- Capture stdout/stderr: `output.stdout`, `output.stderr`
- Parse JSON: `serde_json::from_str(&stdout)`
- Use temp directories for file arguments

### Fixture Usage Examples

**Basic layout:**
```rust
let layout = test_layout_basic(2, 3);  // 2 rows, 3 cols
assert_eq!(layout.layers.len(), 2);     // Base + Function layers
```

**Layout with tap dances:**
```rust
let layout = test_layout_with_tap_dances();
assert_eq!(layout.tap_dances.len(), 2);
assert!(layout.layers[0].keys[0].keycode.contains("TD("));
```

**Layout with idle effect:**
```rust
let layout = test_layout_with_idle_effect(true);
assert!(layout.idle_effect_settings.enabled);
assert_eq!(layout.idle_effect_settings.idle_timeout_ms, 30_000);
```

**Layout with categories:**
```rust
let layout = test_layout_with_categories();
assert_eq!(layout.categories.len(), 2);
assert_eq!(layout.layers[0].keys[0].category_id, Some("navigation".to_string()));
```

**Temp config with QMK:**
```rust
let (config, temp_dir) = temp_config_with_qmk(None);
// config.paths.qmk_firmware is set to a temp QMK structure
// temp_dir keeps directory alive; it's deleted when temp_dir is dropped
```

### Environment-Based Test Isolation

LazyQMK uses environment variables to isolate tests and enable fixture-based testing:

#### `LAZYQMK_CONFIG_DIR` - Config Isolation

Prevents tests from modifying your real configuration:

```rust
#[test]
fn test_config_set_safe() {
    let temp_dir = TempDir::new().unwrap();
    
    let output = Command::new(lazyqmk_bin())
        .env("LAZYQMK_CONFIG_DIR", temp_dir.path())
        .args(["config", "set", "qmk_firmware", "/path/to/qmk"])
        .output()
        .expect("Failed to execute");
    
    assert_eq!(output.status.code(), Some(0));
    // Config written to temp_dir, not your real config
}
```

**Implementation** (in `src/config.rs`):
```rust
pub fn config_dir() -> Result<PathBuf> {
    // Check for test override first
    if let Ok(test_dir) = std::env::var("LAZYQMK_CONFIG_DIR") {
        return Ok(PathBuf::from(test_dir));
    }
    
    // Normal behavior: use platform-specific config directory
    let config_dir = dirs::config_dir()?.join(APP_DATA_DIR);
    Ok(config_dir)
}
```

**Benefits:**
- ✅ 13 config tests run safely in CI
- ✅ No risk of corrupting real user config
- ✅ Parallel test execution is safe
- ✅ Temp dirs auto-cleanup after tests

#### `LAZYQMK_QMK_FIXTURE` - Mock QMK Structure

Enables QMK metadata tests without the 500MB+ submodule:

```rust
#[test]
fn test_list_keyboards_fast() {
    let fixture_path = PathBuf::from("tests/fixtures/mock_qmk");
    
    let output = Command::new(lazyqmk_bin())
        .args(["list-keyboards", "--qmk-path", "ignored"])
        .env("LAZYQMK_QMK_FIXTURE", &fixture_path)
        .output()
        .expect("Failed to execute");
    
    assert_eq!(output.status.code(), Some(0));
}
```

**Implementation** (in `src/cli/qmk.rs`):
```rust
pub fn list_keyboards(qmk_path: &Path) -> Result<Vec<String>> {
    // Check for test fixture override
    let qmk_path = if let Ok(fixture_path) = std::env::var("LAZYQMK_QMK_FIXTURE") {
        PathBuf::from(fixture_path)
    } else {
        qmk_path.to_path_buf()
    };
    
    // Scan keyboards using either fixture or real QMK path
    scan_qmk_keyboards(&qmk_path)
}
```

**Fixture structure:**
```
tests/fixtures/mock_qmk/
  keyboards/
    crkbd/info.json           # Minimal real keyboard metadata
    keebart/corne_choc_pro/info.json
    planck/info.json
```

**Benefits:**
- ✅ 18 QMK metadata tests run in CI (<1 second)
- ✅ No submodule initialization needed
- ✅ Deterministic test results
- ✅ Easy to add new test keyboards (just add info.json)

### Temp Directory Handling

Always keep temp directories alive until test completion:

```rust
#[test]
fn test_with_temp_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test.txt");
    
    // File operations...
    
    // temp_dir automatically cleaned up when dropped at end of scope
}
```

Use `_temp_dir` prefix if not directly accessed:

```rust
#[test]
fn test_file_generation() {
    let layout = test_layout_basic(2, 3);
    let (_layout_path, _temp_dir) = create_temp_layout_file(&layout);
    // Compiler won't warn about unused variables
}
```

### JSON Output Validation

Always validate JSON structure, not just format:

```rust
// Good: validates structure
let result: serde_json::Value = serde_json::from_str(&output)?;
assert!(result["valid"].is_boolean(), "Should have boolean 'valid' field");
assert!(result["errors"].is_array(), "Should have errors array");
assert!(result["checks"].is_object(), "Should have checks object");

// Not enough: just checks format
assert!(output.contains("{") && output.contains("}"));
```

### Exit Code Testing

Always test exit codes explicitly:

```rust
#[test]
fn test_error_handling() {
    let output = Command::new(lazyqmk_bin())
        .args(["generate", "--layout", "nonexistent.md"])
        .output()
        .expect("Failed to execute command");

    // Test failure case
    assert_ne!(
        output.status.code(),
        Some(0),
        "Should fail for missing layout. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
```

## Test Fixtures

### Available Fixtures

All fixtures are in `tests/fixtures/mod.rs`:

#### Layout Fixtures

| Fixture | Purpose | Details |
|---------|---------|---------|
| `test_layout_basic(rows, cols)` | Simple 2-layer layout | Base layer + Function layer with simple keycodes |
| `test_layout_with_tap_dances()` | Tests tap dance handling | Includes 2 tap dance definitions |
| `test_layout_with_idle_effect(enabled)` | Tests idle effect | Configurable idle settings |
| `test_layout_with_categories()` | Tests key categorization | Includes color-coded key categories |
| `test_layout_with_layer_refs()` | Tests layer references | LT, MO, TG keycodes with UUIDs |
| `test_layout_with_invalid_keycode()` | Tests validation | Contains intentionally invalid keycode |

#### Geometry Fixtures

| Fixture | Purpose | Details |
|---------|---------|---------|
| `test_geometry_basic(rows, cols)` | Basic keyboard geometry | Matches layout dimensions |
| `test_mapping_basic(rows, cols)` | Visual layout mapping | Bidirectional matrix ↔ visual transforms |

#### Config Fixtures

| Fixture | Purpose | Details |
|---------|---------|---------|
| `temp_config_with_qmk(path)` | Temporary config with QMK | Creates minimal QMK structure; None creates auto |

#### File Helpers

| Helper | Purpose | Details |
|--------|---------|---------|
| `write_layout_file(layout, path)` | Serialize layout to file | Returns `std::io::Result` |
| `create_temp_layout_file(layout)` | Create layout in temp dir | Returns `(PathBuf, TempDir)` |

### Creating New Fixtures

Add to `tests/fixtures/mod.rs`:

```rust
/// Fixture for testing a specific feature.
pub fn test_layout_for_my_feature() -> Layout {
    let mut layout = test_layout_basic(2, 3);
    
    // Customize for your test
    layout.some_setting = SomeValue::new();
    layout.layers[0].keys[0].keycode = "CUSTOM_KEYCODE".to_string();
    
    layout
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_fixture_my_feature() {
        let layout = test_layout_for_my_feature();
        assert_eq!(layout.layers[0].keys[0].keycode, "CUSTOM_KEYCODE");
    }
}
```

Guidelines:
- Use deterministic values (fixed UUIDs, timestamps)
- Document the fixture with doc comments
- Add test for the fixture itself
- Keep fixtures focused on one aspect
- Reuse existing fixtures when possible

## Golden Testing

### What Are Golden Tests?

Golden tests compare generated output against expected ("golden") files. They're essential for testing firmware generation because:

1. **Detect unintended changes** - Catches modifications to generated code
2. **Document expected output** - Golden files show what the code should generate
3. **Support easy updates** - `UPDATE_GOLDEN=1` regenerates files when intentionally changing behavior
4. **Enable normalization** - Non-deterministic elements (timestamps, UUIDs) are normalized for reliable comparison

### When to Use Golden Tests

Use golden tests for:
- Generated code (keymap.c, config.h)
- Formatted output that's hard to verify line-by-line
- Content that changes rarely but affects many aspects
- Regression detection for complex generation logic

Don't use golden tests for:
- Simple return values (use direct assertions)
- Frequently changing output
- Platform-specific behavior (use normalization instead)

### How to Update Them

When you intentionally change firmware generation:

1. **Run tests with update flag**:
   ```bash
   UPDATE_GOLDEN=1 cargo test --tests
   ```

2. **Review changes**:
   ```bash
   git diff tests/golden/
   ```

3. **Verify changes are correct** by examining the diff

4. **Commit the changes**:
   ```bash
   git add tests/golden/
   git commit -m "test: update golden files for feature X"
   ```

5. **Run tests normally** to verify comparison works:
   ```bash
   cargo test --tests
   ```

### Deterministic Mode for Firmware Generation

The `--deterministic` flag ensures firmware generation output is identical across runs:

```rust
#[test]
fn test_generate_deterministic_output() {
    let layout = test_layout_basic(2, 3);
    let (layout_path, _temp_dir) = create_temp_layout_file(&layout);
    let (config, config_temp) = temp_config_with_qmk(None);
    let out_dir = config_temp.path().join("output");

    // Generate twice
    run_generate(&layout_path, &config, &out_dir, true);  // --deterministic
    let output1 = fs::read_to_string(out_dir.join("keymap.c")).unwrap();

    run_generate(&layout_path, &config, &out_dir, true);  // --deterministic
    let output2 = fs::read_to_string(out_dir.join("keymap.c")).unwrap();

    // Should be identical
    assert_eq!(output1, output2);
}
```

Deterministic mode eliminates:
- Timestamps in comments
- Random UUIDs
- Current date/time references

### Normalizing Output (Timestamps, UUIDs)

The `golden_helper.rs` module automatically normalizes output:

```rust
use golden_helper::normalize_output;

let raw_output = generate_firmware(&layout);
let normalized = normalize_output(&raw_output);

// Raw: "// Generated at: 2025-01-16T10:30:45Z"
// Normalized: "// Generated at: <TIMESTAMP>"

// Raw: "layer_id: 12345678-1234-1234-1234-123456789abc"
// Normalized: "layer_id: <UUID>"
```

Normalization handles:
- **Timestamps**: `// Generated at:` → `<TIMESTAMP>`
- **UUIDs**: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` → `<UUID>`
- **Paths**: `/absolute/path` → `<PATH>`, `C:\Windows\Path` → `<PATH>`
- **Line endings**: Normalizes to `\n`
- **Trailing whitespace**: Removed

## Pre-Release Manual Testing

### The 4 Ignored Tests

LazyQMK has only **4 ignored tests** that require manual execution before releases. Most tests (including QMK metadata commands) now use fixtures and run automatically in CI.

**Find all ignored tests:**
```bash
cargo test -- --list --ignored
```

**The 4 tests:**

1. **`test_generation_vial_json_structure`** *(deprecated - can be removed)*
   - **Location**: `tests/firmware_gen_tests.rs`
   - **Purpose**: Validates vial.json structure
   - **Status**: ⚠️ Deprecated after migration to standard QMK
   - **Action**: Safe to remove this test
   
2. **`test_scan_keyboards_finds_crkbd`** *(pre-release validation)*
   - **Location**: `tests/qmk_info_json_tests.rs`
   - **Purpose**: Validates QMK CLI integration with real submodule
   - **Status**: ✅ Manual pre-release test
   - **Runtime**: ~5 seconds
   
3. **`test_tap_dance_add_use_generate`** *(pre-release validation)*
   - **Location**: `tests/cli_tap_dance_tests.rs`
   - **Purpose**: Full pipeline test with QMK firmware compilation
   - **Status**: ✅ Manual pre-release test
   - **Runtime**: ~5-10 minutes (QMK compilation)
   
4. **`test_check_deprecated_options_clean`** *(deprecated - can be removed)*
   - **Location**: `src/firmware/validator.rs`
   - **Purpose**: Checks for deprecated Vial options
   - **Status**: ⚠️ Deprecated after migration to standard QMK
   - **Action**: Safe to remove this test

### Running Pre-Release Tests

**Before any release**, run these 2 critical tests:

```bash
# 1. Initialize QMK submodule (one-time setup)
git submodule update --init --recursive qmk_firmware

# 2. Run the 2 critical pre-release tests
cargo test test_scan_keyboards_finds_crkbd -- --ignored
cargo test test_tap_dance_add_use_generate -- --ignored

# 3. Optional: Install QMK CLI for full pipeline test
pip3 install qmk
qmk setup
```

**Expected results:**
- Both tests should pass ✅
- Total runtime: ~5-15 minutes (depending on QMK compilation)

**If tests fail:**
- Check QMK submodule is initialized: `ls qmk_firmware/keyboards`
- Verify QMK CLI is installed: `qmk --version`
- Check QMK submodule commit matches expected version

### Why Only 4 Tests Are Ignored

**Previous state (before Spec 025/026):**
- 23 tests were ignored
- Required QMK submodule for most QMK commands
- Slow test suite (~10+ minutes)

**Current state (after Spec 025/026):**
- ✅ **Only 4 tests ignored** (2 deprecated, 2 pre-release validation)
- ✅ **18 QMK metadata tests** use fixtures (run in CI automatically)
- ✅ **13 config tests** use environment isolation (run in CI automatically)
- ✅ **Fast test suite** (<2 seconds for ~970 tests)

**Changes made:**
1. **Mock QMK fixtures** (`tests/fixtures/mock_qmk/`) enable QMK command testing without submodule
2. **Environment-based config isolation** (`LAZYQMK_CONFIG_DIR`) enables safe config tests
3. **Fixture override** (`LAZYQMK_QMK_FIXTURE`) allows commands to use mock data in tests

**Result:**
- 91% reduction in ignored tests (23 → 4)
- QMK metadata commands fully tested in CI
- Only full-pipeline compilation tests remain manual

### Pre-Release Checklist

See `AGENTS.md` section "Pre-Release Testing Requirements" for complete checklist:

1. All CI tests pass: `cargo test --tests && cargo test --lib`
2. All pre-release tests pass: `cargo test -- --ignored`
3. Clippy clean: `cargo clippy --all-features -- -D warnings`
4. Tested on target platform (macOS/Linux/Windows)
5. QMK submodule at expected commit
6. Version number updated in `Cargo.toml`
7. CHANGELOG reviewed (auto-generated from commits)

**Only create release if all checklist items are complete.**

## CI Integration

### GitHub Actions Workflow

The release workflow (`.github/workflows/release.yml`) runs tests on:
- Linux (x86_64)
- macOS (x86_64 and ARM64)
- Windows (x86_64)

Test stages:
1. **Fast tests** (2 minutes): Unit + integration tests
2. **Clippy** (1 minute): Linter with `-D warnings`
3. **Build release** (5 minutes): Compile binaries
4. **Tests pass** ✓

### Fast Test Suite Target (<2 minutes)

The fast test suite is optimized for quick feedback:

```bash
# This target is <2 minutes
cargo test --tests
```

This excludes:
- Contract tests (`#[ignore]`)
- Tests that spawn threads
- Tests that do heavy I/O

Achieves fast execution through:
- Parallel test execution
- Minimal I/O operations
- Reusable fixtures
- In-memory operations

### Clippy Requirements (Zero Warnings)

Before committing, run clippy:

```bash
cargo clippy --all-features -- -D warnings
```

**Rules:**
- All warnings must be fixed
- Never use `#[allow]` to suppress warnings
- Fix the underlying issue instead
- CI will fail if warnings are present

Example of fixing a warning:

```rust
// ❌ Don't do this
#[allow(unused_mut)]
let mut x = 5;

// ✓ Do this
let x = 5;  // Remove unnecessary mut
```

### How CI Handles QMK Tests

The release workflow:

1. **Doesn't run contract tests** - They require manual setup
2. **Fast tests are sufficient** - All logic is unit tested
3. **Can add optional workflow** - For manual QMK testing

To run contract tests in CI, create a separate workflow:

```yaml
name: Contract Tests

on: [workflow_dispatch]  # Manual trigger only

jobs:
  contract-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          submodules: 'recursive'
      - run: cargo test -- --ignored
```

## Troubleshooting

### Common Test Failures and Solutions

| Problem | Cause | Solution |
|---------|-------|----------|
| `thread panicked at 'Golden file not found'` | Golden file doesn't exist | Run with `UPDATE_GOLDEN=1` first time |
| `Golden file mismatch` | Generated output changed | Review diff, run `UPDATE_GOLDEN=1`, verify changes |
| `Failed to execute command` | Binary not found | Run `cargo build` first, or use `cargo test` |
| `Thread '<unnamed>' panicked` | Test data corruption | Reset temp directory state or create new fixtures |
| `QMK path not found` | Submodule not initialized | Run `git submodule update --init --recursive` |

### Golden Test Mismatches

If a golden test fails:

1. **Check if change is intentional**:
   ```bash
   git diff tests/golden/
   ```

2. **If intentional, update golden files**:
   ```bash
   UPDATE_GOLDEN=1 cargo test test_name
   git add tests/golden/
   git commit -m "test: update golden file for ..."
   ```

3. **If unintentional, revert your code**:
   ```bash
   git checkout -- src/
   cargo test
   ```

4. **If deterministic mode is failing**:
   ```bash
   # Check if timestamps are being normalized
   cargo test test_deterministic -- --nocapture
   ```

### Temp Directory Issues

**Problem:** Permission denied when writing to temp dir

```
Error: Permission denied (os error 13)
```

**Solution:** Don't modify temp dir after tests start. Create child directories:

```rust
let temp_dir = TempDir::new()?;
let work_dir = temp_dir.path().join("work");
fs::create_dir(&work_dir)?;  // Create subdirectory
// Use work_dir for operations
```

**Problem:** Temp dir not cleaned up

**Solution:** Keep TempDir reference alive:

```rust
// ❌ Wrong: temp_dir is dropped immediately
let temp_path = TempDir::new()?.path().to_path_buf();

// ✓ Correct: keep TempDir alive
let (layout_path, _temp_dir) = create_temp_layout_file(&layout);
// _temp_dir is dropped at end of function, cleaning up
```

### QMK Submodule Problems

**Problem:** QMK submodule shows "modified" or "missing"

```
m qmk_firmware (modified content)
```

**Solution:** Don't commit submodule changes; reset it:

```bash
git submodule update --init --recursive qmk_firmware
git checkout HEAD -- qmk_firmware
```

**Problem:** "QMK path not found" in contract tests

```
thread 'test_list_keyboards' panicked at 'QMK path not found'
```

**Solution:** Initialize submodule:

```bash
git submodule update --init --recursive qmk_firmware
cargo test -- --ignored
```

### Permission Errors

**Problem:** "Permission denied" on Unix

```
Error: Permission denied (os error 13)
```

**Causes:**
- Writing to read-only temp directory
- Not closing file handles before deletion
- Running tests in parallel (race condition)

**Solutions:**
1. Ensure parent directory is writable: `ls -ld /tmp`
2. Use unique temp directories per test
3. Don't share temp dirs between tests
4. Keep TempDir handles alive

## Testing Best Practices

### Use Temp Directories for File Operations

Always use `tempfile::TempDir` for file operations:

```rust
#[test]
fn test_file_operation() {
    // Good: files are automatically cleaned up
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test.txt");
    
    fs::write(&file_path, "content").expect("Failed to write");
    assert!(file_path.exists());
    
    // temp_dir automatically deleted when dropped
}
```

Benefits:
- Automatic cleanup
- No leftover files
- Parallel test safety
- Works across platforms

### Test Exit Codes Explicitly

Always verify exit codes:

```rust
#[test]
fn test_success_case() {
    let output = Command::new(binary)
        .args(args)
        .output()
        .expect("Failed to execute");
    
    // Test both code and message
    assert_eq!(
        output.status.code(),
        Some(0),
        "Should exit successfully. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_failure_case() {
    let output = Command::new(binary)
        .args(invalid_args)
        .output()
        .expect("Failed to execute");
    
    assert_ne!(
        output.status.code(),
        Some(0),
        "Should exit with error"
    );
}
```

### Validate JSON Structure, Not Just Format

Test JSON structure, not string format:

```rust
#[test]
fn test_json_output() {
    let output = Command::new(binary)
        .args(["--json"])
        .output()
        .expect("Failed to execute");
    
    let result: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
            .expect("Should parse JSON");
    
    // Validate structure
    assert!(result["status"].is_string(), "status should be string");
    assert!(result["errors"].is_array(), "errors should be array");
    assert!(result["warnings"].is_object(), "warnings should be object");
    
    // Validate specific values if needed
    assert_eq!(result["status"], "success");
}
```

### Use Fixtures for Consistency

Reuse fixtures to keep tests consistent:

```rust
// Good: uses shared fixtures
#[test]
fn test_with_fixture() {
    let layout = test_layout_basic(2, 3);
    assert_eq!(layout.layers.len(), 2);
}

// Avoid: creates custom data each time
#[test]
fn test_custom_data() {
    let mut layout = Layout::default();
    layout.name = "Test".to_string();
    // ... lots of setup ...
}
```

Benefits:
- Consistent test data
- Easier maintenance
- Deterministic output
- Faster to write

### Keep Tests Independent and Isolated

Tests must be runnable in any order:

```rust
// Good: completely independent
#[test]
fn test_operation_a() {
    let layout = test_layout_basic(2, 3);
    assert_eq!(validate(&layout), true);
}

#[test]
fn test_operation_b() {
    let layout = test_layout_basic(2, 3);
    assert_eq!(transform(&layout), true);
}

// Avoid: tests depending on execution order
static mut SHARED_LAYOUT: Option<Layout> = None;

#[test]
fn test_setup() {
    unsafe { SHARED_LAYOUT = Some(test_layout_basic(2, 3)); }
}

#[test]
fn test_uses_setup() {
    let layout = unsafe { SHARED_LAYOUT.as_ref().unwrap() };
    // ERROR: setup test might not have run!
}
```

### Clear Test Names Describing Scenarios

Test names should describe what they test:

```rust
// Good: describes the scenario and expectation
#[test]
fn test_validate_succeeds_with_valid_keycodes() { }
#[test]
fn test_validate_fails_with_invalid_keycodes() { }
#[test]
fn test_generate_includes_tap_dance_definitions() { }

// Avoid: unclear what's being tested
#[test]
fn test_validate() { }
#[test]
fn test_1() { }
#[test]
fn test_it_works() { }
```

This makes test output clear and helps when tests fail:

```
test test_validate_succeeds_with_valid_keycodes ... ok
test test_validate_fails_with_invalid_keycodes ... FAILED
```

---

## Summary

- **Run fast tests**: `cargo test --tests` (<30 seconds)
- **Update golden files**: `UPDATE_GOLDEN=1 cargo test --tests`
- **Run all tests**: `cargo test -- --ignored` (requires QMK submodule)
- **Check quality**: `cargo clippy --all-features -- -D warnings`
- **Use fixtures**: Leverage `tests/fixtures/` for consistent test data
- **Golden tests**: Detect regressions in firmware generation
- **Contract tests**: Test real QMK integration when needed
- **Clear names**: Make test purposes obvious
- **Independent tests**: Ensure tests don't depend on execution order
