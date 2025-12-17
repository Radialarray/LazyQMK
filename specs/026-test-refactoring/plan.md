# Spec 026: Test Refactoring & CI Integration

**Status:** Planning  
**Created:** 2025-01-XX  
**Priority:** High

## Problem Statement

Currently, 23 integration tests are marked with `#[ignore]` and cannot run in CI, reducing test coverage and confidence in releases. These tests have external dependencies (QMK submodule, user config) that prevent them from running automatically.

**Current State:**
- 234 tests run in CI ✅
- 23 tests ignored (not run in CI) 🔒
- 3 unit tests failing in `validator.rs` ❌

**Goals:**
1. Enable all meaningful tests to run in CI
2. Remove obsolete/redundant tests
3. Fix failing unit tests
4. Add pre-release validation checklist
5. Document test categories and when to run them

---

## Test Categories

### Category 1: Config Modification Tests (5 tests)
**File:** `tests/cli_config_tests.rs`  
**Current Issue:** Modify user's actual config file  
**Solution:** Environment-based config isolation

**Tests:**
- `test_config_set_qmk_path`
- `test_config_set_output_dir`
- `test_config_set_theme_mode`
- `test_config_set_invalid_key`
- `test_config_set_invalid_theme_value`

**Refactoring Plan:**
1. Add `LAZYQMK_CONFIG_DIR` environment variable support to `src/config.rs`
2. Modify `Config::config_dir()` to check env var first
3. Update tests to use `tempfile::TempDir` for isolated config
4. Remove `#[ignore]` attributes
5. Tests can now run safely in CI

**Implementation:**
```rust
// In src/config.rs
pub fn config_dir() -> Result<PathBuf> {
    // Check for test override first
    if let Ok(test_dir) = std::env::var("LAZYQMK_CONFIG_DIR") {
        return Ok(PathBuf::from(test_dir));
    }
    
    // Normal behavior
    let config_dir = dirs::config_dir()
        .context("Failed to determine config directory")?
        .join(crate::branding::APP_DATA_DIR);
    
    Ok(config_dir)
}

// In tests/cli_config_tests.rs
#[test]
fn test_config_set_qmk_path() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    
    let output = Command::new(lazyqmk_bin())
        .env("LAZYQMK_CONFIG_DIR", temp_dir.path())
        .args(["config", "set", "qmk_firmware", "/path/to/qmk"])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    // Verify config file in temp_dir
}
```

**Impact:** ✅ 5 tests can now run in CI

---

### Category 2: QMK Submodule Tests (15 tests)
**File:** `tests/cli_qmk_tests.rs`  
**Current Issue:** Require 500MB+ QMK submodule  
**Solution:** Split into unit tests with fixtures + manual integration tests

**Tests:**
- `list-keyboards` command: 5 tests
- `list-layouts` command: 5 tests  
- `geometry` command: 5 tests

**Refactoring Plan:**

#### Option A: Fixture-Based Tests (Recommended)
1. Create `tests/fixtures/qmk_metadata/` directory
2. Add minimal test fixtures:
   - `keyboards.json` - Sample keyboard list
   - `crkbd_info.json` - Sample keyboard metadata
   - `layout_variants.json` - Sample layout data
3. Modify commands to accept `--test-mode` flag that uses fixtures
4. Remove `#[ignore]` from tests that use fixtures
5. Keep 3-5 tests as manual integration tests for real QMK

**Fixture Structure:**
```
tests/fixtures/qmk_metadata/
  keyboards.json          # Output of qmk list-keyboards
  crkbd/
    info.json            # Real crkbd/rev1 info.json
  corne_choc_pro/
    info.json            # Real keebart/corne_choc_pro info.json
```

**Implementation:**
```rust
// In src/cli/qmk.rs
pub fn list_keyboards(qmk_path: &Path, test_fixture: Option<&Path>) -> Result<Vec<String>> {
    if let Some(fixture_path) = test_fixture {
        // Load from fixture for testing
        let data = fs::read_to_string(fixture_path.join("keyboards.json"))?;
        return Ok(serde_json::from_str(&data)?);
    }
    
    // Normal QMK scanning
    scan_qmk_keyboards(qmk_path)
}

// In tests/cli_qmk_tests.rs
#[test] // No longer ignored!
fn test_list_keyboards_json_output() {
    let fixture_dir = PathBuf::from("tests/fixtures/qmk_metadata");
    
    let output = Command::new(lazyqmk_bin())
        .args(["list-keyboards", "--json"])
        .env("LAZYQMK_TEST_FIXTURES", fixture_dir)
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    // Validate JSON structure
}

#[test]
#[ignore = "requires QMK submodule - run before release"]
fn test_list_keyboards_real_qmk() {
    // This test runs against real qmk_firmware/ for validation
    let output = Command::new(lazyqmk_bin())
        .args(["list-keyboards", "--qmk-path", "qmk_firmware"])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("crkbd"));
}
```

**Impact:** 
- ✅ 12 tests can run in CI (using fixtures)
- 🔒 3 tests remain ignored (real QMK validation - manual pre-release)

#### Option B: Mock QMK Structure (Alternative)
1. Create minimal mock QMK structure in `tests/fixtures/mock_qmk/`
2. Only include 2-3 keyboard directories with real `info.json` files
3. Tests use this mock structure instead of full submodule
4. Much smaller (~10KB vs 500MB)

**Impact:** ✅ All 15 tests can run in CI

---

### Category 3: Full Pipeline Tests (3 tests)
**Files:** 
- `tests/firmware_gen_tests.rs::test_firmware_gen_with_all_features`
- `tests/qmk_info_json_tests.rs::test_scan_keyboards_finds_crkbd`
- `tests/cli_tap_dance_tests.rs::test_generate_with_tap_dances_full_pipeline`

**Current Issue:** Require full QMK setup + compilation  
**Solution:** Keep as manual pre-release tests

**Refactoring Plan:**
1. Keep these tests ignored (they validate end-to-end with real QMK)
2. Document in pre-release checklist
3. Create `make test-release` command to run them
4. Add these to release documentation

**Rationale:**
- These tests provide critical validation that fixtures can't replace
- They ensure real QMK compilation works
- Running in CI would add 10+ minutes and require 500MB+ submodule
- Better to run manually before releases

**Impact:** 🔒 3 tests remain ignored (intentionally - pre-release only)

---

### Category 4: Failing Unit Tests (3 tests)
**File:** `src/firmware/validator.rs`  
**Current Issue:** Tests are failing  
**Solution:** Fix or remove

**Tests:**
- `test_check_deprecated_options_vial_enable`
- `test_check_deprecated_options_vial_keyboard_uid`
- `test_check_deprecated_options_both`

**Investigation Needed:**
1. Are these tests still relevant? (Vial support status?)
2. Is the validation logic broken?
3. Should deprecated options still be checked?

**Refactoring Plan:**
1. Investigate current Vial support in LazyQMK
2. If Vial is not supported: Remove tests and related code
3. If Vial is supported but broken: Fix the validation logic
4. If tests are obsolete: Remove them

**Impact:** ❌ → ✅ 3 tests will either pass or be removed

---

## Implementation Phases

### Phase 1: Config Test Isolation (Quickest Win)
**Effort:** 2-3 hours  
**Impact:** +5 tests in CI

1. Add `LAZYQMK_CONFIG_DIR` environment variable support
2. Update all 5 config tests to use temp directories
3. Remove `#[ignore]` attributes
4. Verify tests pass in CI

**Files to modify:**
- `src/config.rs` (add env var check)
- `tests/cli_config_tests.rs` (update all 5 tests)

### Phase 2: QMK Fixture-Based Tests
**Effort:** 4-6 hours  
**Impact:** +12 tests in CI

1. Create `tests/fixtures/qmk_metadata/` structure
2. Extract minimal real data from QMK:
   - `qmk list-keyboards > keyboards.json`
   - Copy 2-3 keyboard `info.json` files
3. Add fixture loading support to CLI commands
4. Update 12 tests to use fixtures
5. Keep 3 tests as manual integration tests

**Files to modify:**
- `tests/fixtures/qmk_metadata/` (new)
- `src/cli/qmk.rs` (add fixture support)
- `tests/cli_qmk_tests.rs` (update tests)

### Phase 3: Fix Failing Unit Tests
**Effort:** 1-2 hours  
**Impact:** +3 tests passing

1. Investigate Vial validation logic
2. Fix tests or remove if obsolete
3. Ensure `cargo test --lib` passes completely

**Files to modify:**
- `src/firmware/validator.rs`

### Phase 4: Documentation & Pre-Release Checklist
**Effort:** 1 hour  
**Impact:** Better release process

1. Update `AGENTS.md` with pre-release checklist
2. Create `docs/RELEASE_CHECKLIST.md`
3. Add test categories to `docs/TESTING.md`
4. Document which tests require manual runs

---

## Expected Results

### Test Coverage After Refactoring

| Category | Before | After | Change |
|----------|--------|-------|--------|
| CI Tests (auto) | 234 | 251 (+17) | ✅ |
| Manual Tests (pre-release) | 0 | 6 | 📋 |
| Ignored Tests (not needed) | 23 | 6 | ⬇️ |
| Failing Tests | 3 | 0 | ✅ |
| **Total Tests** | **257** | **257** | **-** |

### CI Improvements
- **Before:** 91% test coverage in CI (234/257)
- **After:** 98% test coverage in CI (251/257)
- **Manual tests:** 2% (6 tests - run before release)

---

## Pre-Release Checklist

Add to `AGENTS.md`:

```markdown
### Pre-Release Testing Requirements

Before creating a new release, the following manual tests MUST be run:

#### 1. Initialize QMK Submodule
```bash
git submodule update --init --recursive qmk_firmware
```

#### 2. Run QMK Integration Tests
```bash
# Test QMK metadata commands with real submodule
cargo test --test cli_qmk_tests test_list_keyboards_real_qmk -- --ignored
cargo test --test cli_qmk_tests test_list_layouts_real_qmk -- --ignored  
cargo test --test cli_qmk_tests test_geometry_real_qmk -- --ignored

# Test full firmware generation pipeline
cargo test --test firmware_gen_tests test_firmware_gen_with_all_features -- --ignored

# Test QMK CLI integration
cargo test --test qmk_info_json_tests test_scan_keyboards_finds_crkbd -- --ignored

# Test tap dance full pipeline
cargo test --test cli_tap_dance_tests test_generate_with_tap_dances_full_pipeline -- --ignored
```

#### 3. Verify Results
- All tests must pass ✅
- No clippy warnings: `cargo clippy --all-features -- -D warnings`
- Test on target platform (macOS/Linux/Windows)

#### 4. Create Release
Only proceed with release if all pre-release tests pass.
```

---

## Migration Strategy

### Step 1: Run Tests Locally
```bash
# Before starting refactor - establish baseline
cargo test --tests
cargo test --lib

# Run ignored tests to see current state
cargo test -- --ignored
```

### Step 2: Implement Phase 1 (Config)
```bash
# Implement config isolation
# Run tests to verify
cargo test --test cli_config_tests
```

### Step 3: Implement Phase 2 (QMK Fixtures)
```bash
# Create fixtures and update tests
# Run tests to verify
cargo test --test cli_qmk_tests
```

### Step 4: Implement Phase 3 (Fix Validator)
```bash
# Fix or remove failing tests
cargo test --lib
```

### Step 5: Update Documentation
```bash
# Update AGENTS.md with pre-release checklist
# Update docs/TESTING.md with test categories
```

### Step 6: CI Validation
```bash
# Push to GitHub and verify CI passes
# All 251 tests should run and pass
```

---

## Success Criteria

- [ ] Config tests run in CI without `#[ignore]` (5 tests)
- [ ] QMK tests run in CI using fixtures (12 tests)
- [ ] 3 failing unit tests are fixed or removed
- [ ] 6 manual tests documented for pre-release
- [ ] CI runs 251 tests automatically (98% coverage)
- [ ] Pre-release checklist added to `AGENTS.md`
- [ ] `docs/TESTING.md` updated with test categories
- [ ] All tests pass: `cargo test --tests && cargo test --lib`
- [ ] Zero clippy warnings: `cargo clippy --all-features -- -D warnings`

---

## Alternative Approaches Considered

### Alternative 1: Run Everything in CI with QMK Submodule
**Pros:** Complete test coverage in CI  
**Cons:** 
- Adds 500MB+ to CI storage/bandwidth
- Increases CI time by 10+ minutes
- Submodule sync issues in CI
- Not worth it for 3-6 tests

**Decision:** ❌ Rejected - too expensive for marginal benefit

### Alternative 2: Remove All Ignored Tests
**Pros:** Clean test suite  
**Cons:**
- Lose valuable integration tests
- No validation that real QMK works
- Regressions could slip through

**Decision:** ❌ Rejected - tests provide critical validation

### Alternative 3: Mock Everything
**Pros:** All tests run in CI  
**Cons:**
- Mocks might not reflect reality
- Maintenance burden to keep mocks updated
- False confidence in test coverage

**Decision:** ⚠️ Partial - Use fixtures for most, keep some real tests

---

## Risks & Mitigations

### Risk 1: Fixtures Diverge from Reality
**Mitigation:** 
- Keep manual integration tests for real QMK validation
- Update fixtures when QMK changes significantly
- Run pre-release tests before every release

### Risk 2: Config Isolation Breaks Production
**Mitigation:**
- Thorough testing of env var logic
- Ensure env var is only checked in test scenarios
- Document behavior clearly

### Risk 3: Removing Wrong Tests
**Mitigation:**
- Review each test carefully before removal
- Keep git history for easy revert
- Document why tests were removed

---

## Questions to Answer

1. **Vial Support:** Is LazyQMK still planning to support Vial? (Affects validator tests)
2. **QMK Fixtures:** Which keyboards should we include in fixtures? (Suggest: crkbd, corne_choc_pro, planck)
3. **CI Budget:** Is 10KB of fixture data acceptable in repo?
4. **Release Process:** Should pre-release tests be mandatory or optional?

---

## Appendix: Current Test Breakdown

### By Status
- ✅ Passing: 234 tests
- 🔒 Ignored: 23 tests
- ❌ Failing: 3 tests
- **Total:** 260 tests

### By File (Ignored Tests Only)
- `tests/cli_config_tests.rs`: 5 ignored
- `tests/cli_qmk_tests.rs`: 15 ignored
- `tests/cli_tap_dance_tests.rs`: 1 ignored
- `tests/firmware_gen_tests.rs`: 1 ignored
- `tests/qmk_info_json_tests.rs`: 1 ignored

### By Type
- Unit tests: ~100
- Integration tests: ~157
- E2E CLI tests: ~97
