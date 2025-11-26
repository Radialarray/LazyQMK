# Config Merger Fix - Quick Reference

**Feature:** 004-config-merger-fix  
**Branch:** feature/config-merger-fix  
**Status:** Planning Complete

## Problem
TUI fails to compile QMK firmware for keyboards with variant structures due to:
1. Missing `RGB_MATRIX_LED_COUNT` definition
2. Deprecated VIAL options in config files
3. Incorrect variant path handling

## Solution
Implement ConfigMerger module that:
- Auto-detects keyboard variants
- Extracts RGB LED count from variant keyboard.json
- Filters deprecated VIAL options
- Merges keyboard config with TUI-generated config

## Key Files

### Documents
- `spec.md` - Complete feature specification
- `implementation-plan.md` - Step-by-step implementation checklist
- `error-analysis.md` - Detailed error analysis and root causes

### Code to Create
- `src/firmware/config_merger.rs` - New module (core logic)

### Code to Modify
- `src/firmware/generator.rs` - Use ConfigMerger
- `src/firmware/mod.rs` - Export ConfigMerger
- TUI call sites - Handle 4 generated files

## Quick Start

### Read First
1. `error-analysis.md` - Understand the problem
2. `spec.md` - Understand the solution
3. `implementation-plan.md` - Follow the steps

### Implementation Order
1. Phase 1: Create ConfigMerger module (4-6 hours)
2. Phase 2: Update FirmwareGenerator (2-3 hours)
3. Phase 3: Module Integration (1-2 hours)
4. Phase 4: Testing (3-4 hours)
5. Phase 5: Documentation (1-2 hours)

**Total Time:** ~11-17 hours

## Testing Checklist

- [ ] Unit tests for ConfigMerger pass
- [ ] config.h has `RGB_MATRIX_LED_COUNT 46`
- [ ] config.h lacks `VIAL_KEYBOARD_UID`
- [ ] rules.mk lacks `VIAL_ENABLE`
- [ ] Compilation succeeds via TUI
- [ ] Firmware file generated
- [ ] All existing keyboards still work

## Success Criteria

✅ Compilation succeeds without errors  
✅ No deprecated option warnings  
✅ RGB matrix works correctly  
✅ All tests pass  
✅ No regressions  

## Branch Status

**Current Branch:** feature/config-merger-fix  
**Specs Location:** `specs/004-config-merger-fix/`  
**Ready to Implement:** Yes

## Next Steps

1. Review all three spec documents
2. Start with Phase 1: Create ConfigMerger module
3. Follow implementation-plan.md checklist
4. Test thoroughly before merging
