# Spec 022: Documentation Restructure & Cleanup

**Status**: Planning  
**Created**: 2025-12-11  
**Owner**: AI Assistant

## Overview

Comprehensive documentation restructure to improve README clarity, add MIT license, archive completed specs, and consolidate branding documentation.

## Goals

1. **README Simplification**: Remove marketing fluff, focus on essential sections
2. **License**: Add MIT license information to README
3. **Motivation Section**: Add personal "why" section explaining the project's origin
4. **Branding Organization**: Move branding.md to specs subfolder
5. **Specs Archival**: Archive all completed spec folders (019, 020, 021)
6. **Architecture Consolidation**: Remove duplicate content from README (already in ARCHITECTURE.md)
7. **Documentation Consistency**: Ensure all docs reference correct locations

## Current State

### README Issues
- Too much marketing language ("Built with ❤️", badges, etc.)
- Duplicate content already in ARCHITECTURE.md (tech stack, design patterns)
- Missing motivation/origin story
- No MIT license badge/reference (LICENSE file exists but not referenced)
- Status section with detailed version info (not essential for users)
- Future roadmap items not committed to

### Branding Documentation
- `docs/BRANDING.md` exists with configuration guide
- Should be moved to `specs/branding/` as it's more developer-focused

### Completed Specs
All currently in `specs/`:
- `019-legacy-code-cleanup/` ✅ Complete
- `020-robust-keyboard-picker/` ✅ Complete  
- `021-dependency-updates/` ✅ Complete

## Proposed Structure

### README Sections (Final)

1. **Header with ASCII Art** (keep as-is)
2. **Intro** (1-2 paragraphs - concise description)
3. **Motivation** (new - why this project exists)
4. **Features** (existing - keep ✨ section)
5. **Installation** (existing - keep 📦 section)
6. **Quick Start** (existing - keep 🚀 section)
7. **Keyboard Shortcuts** (existing - brief reference)
8. **File Format** (existing - markdown examples)
9. **Color Organization** (existing - priority system)
10. **Supported Keyboards** (existing - list of compatible keyboards)
11. **License** (new - MIT)

### Removed from README
- ❌ Badges (License, Rust Version, Latest Release)
- ❌ "Architecture" section (→ ARCHITECTURE.md)
- ❌ "Tech Stack" subsection (→ ARCHITECTURE.md)
- ❌ "Design Patterns" subsection (→ ARCHITECTURE.md)
- ❌ "Project Status" section
- ❌ "Recent Updates" section
- ❌ "Future Enhancements" roadmap
- ❌ Footer with emoji hearts

### New Motivation Section

```markdown
## 💡 Motivation

I created LazyQMK because I wanted to edit my keyboard firmware for my **Keebio Corne Choc Pro** directly without diving into code every time I needed to tweak a keymap. At the same time, I wanted to support complex coloring of layers and individual keys for better visual organization.

This led me to add custom code to my QMK fork and implement visual layer-aware coloring in a terminal UI editor. Why a TUI? Because I love having small, focused utilities in the terminal—like `lazygit` and `neovim`. LazyQMK follows that philosophy: stay in the terminal, work efficiently, and keep it simple.
```

### Directory Restructure

**Before:**
```
specs/
├── 019-legacy-code-cleanup/
├── 020-robust-keyboard-picker/
├── 021-dependency-updates/
├── archived/
│   ├── 001-tui-complete-features/
│   ├── 002-fix-startup-warnings/
│   └── [others]
docs/
└── BRANDING.md
```

**After:**
```
specs/
├── archived/
│   ├── 001-tui-complete-features/
│   ├── 002-fix-startup-warnings/
│   ├── [others]
│   ├── 019-legacy-code-cleanup/
│   ├── 020-robust-keyboard-picker/
│   └── 021-dependency-updates/
└── branding/
    └── BRANDING.md
docs/
├── ARCHITECTURE.md
└── FEATURES.md
```

## Implementation Plan

### Phase 1: Archive Completed Specs
- [ ] Move `specs/019-legacy-code-cleanup/` → `specs/archived/019-legacy-code-cleanup/`
- [ ] Move `specs/020-robust-keyboard-picker/` → `specs/archived/020-robust-keyboard-picker/`
- [ ] Move `specs/021-dependency-updates/` → `specs/archived/021-dependency-updates/`

### Phase 2: Reorganize Branding
- [ ] Create `specs/branding/` directory
- [ ] Move `docs/BRANDING.md` → `specs/branding/BRANDING.md`

### Phase 3: Update README
- [ ] Remove badges section
- [ ] Remove navigation links line
- [ ] Shorten intro paragraph (reduce marketing language)
- [ ] Add **Motivation** section after intro
- [ ] Keep Features section as-is (already good)
- [ ] Keep Installation section as-is
- [ ] Keep Quick Start section as-is
- [ ] Keep Keyboard Shortcuts reference
- [ ] Keep File Format examples
- [ ] Keep Color Organization explanation
- [ ] Remove "Architecture" section (keep as reference to docs/)
- [ ] Remove "Tech Stack" subsection
- [ ] Remove "Design Patterns" subsection
- [ ] Remove "Documentation" section entirely
- [ ] Remove "Contributing" section
- [ ] Keep "Roadmap" section but simplify (completed items only)
- [ ] Remove "Project Status" section
- [ ] Remove "Recent Updates" section
- [ ] Add **License** section with MIT reference
- [ ] Remove footer with emoji hearts

### Phase 4: Update Documentation Cross-References
- [ ] Update QUICKSTART.md if it references moved files
- [ ] Update ARCHITECTURE.md if it references moved files
- [ ] Update AGENTS.md if needed
- [ ] Check for broken links in all docs

### Phase 5: Verification
- [ ] Run tests: `cargo test`
- [ ] Check for clippy warnings: `cargo clippy`
- [ ] Verify all links in README work
- [ ] Verify all internal doc references work

## Motivation Section - Draft

```markdown
## 💡 Motivation

I created LazyQMK because I wanted to edit my keyboard firmware for my **Keebio Corne Choc Pro** directly without diving into code every time I needed to tweak a keymap. At the same time, I wanted to support complex coloring of layers and individual keys for better visual organization.

This led me to add custom code to my QMK fork and implement visual layer-aware coloring in a terminal UI editor. Why a TUI? Because I love having small, focused utilities in the terminal—like `lazygit` and `neovim`. LazyQMK follows that philosophy: stay in the terminal, work efficiently, and keep it simple.
```

## License Section - Draft

```markdown
## 📄 License

This project is licensed under the **MIT License** - see [LICENSE](LICENSE) for details.

```

## Documentation Section - Updated

```markdown
## 📚 Documentation

### User Guides
- **[Quick Start Guide](QUICKSTART.md)** - Getting started, workflows, shortcuts

### Technical Documentation
- **[Architecture Guide](docs/ARCHITECTURE.md)** - Deep dive into technical design

### Specifications
- **[Archived Specs](specs/archived/)** - Historical development specifications

```

## Files to Modify

1. **README.md** - Major simplification and additions
2. **specs/019-legacy-code-cleanup/** - Move to archived/
3. **specs/020-robust-keyboard-picker/** - Move to archived/
4. **specs/021-dependency-updates/** - Move to archived/
5. **docs/BRANDING.md** - Move to specs/branding/
6. **QUICKSTART.md** - Update if needed (verify links)
7. **docs/ARCHITECTURE.md** - Update if needed (verify links)
8. **AGENTS.md** - Update if needed (verify links)

## Success Criteria

- [ ] README is concise, focused, and free of marketing fluff
- [ ] Motivation section clearly explains project origin
- [ ] MIT license is prominently referenced
- [ ] All completed specs are archived
- [ ] Branding docs are in appropriate location
- [ ] No duplicate content between README and ARCHITECTURE.md
- [ ] All documentation links work correctly
- [ ] Tests pass (`cargo test`)
- [ ] No new clippy warnings

## Risks & Mitigation

**Risk**: Breaking existing external links to documentation  
**Mitigation**: Use git mv to preserve history; GitHub redirects will work for moved files

**Risk**: README becomes too sparse  
**Mitigation**: Keep essential feature highlights and examples; full details in linked docs

**Risk**: Loss of useful "Recent Updates" information  
**Mitigation**: Information preserved in git history and CHANGELOG (if we add one)

## Notes

- Keep the ASCII art logo - it's iconic
- Keep the feature table with emojis - it's clear and scannable
- Keep code examples - they're essential for users
- Focus README on "what" and "how to get started", not "why we built it this way"
- Move all architectural decisions to ARCHITECTURE.md
- Specs folder becomes purely historical reference

## Related Issues

- Improved first-time user experience
- Clearer documentation structure
- Better separation of user vs. developer docs
- Compliance with open-source licensing best practices
