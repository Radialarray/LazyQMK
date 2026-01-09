# Changelog

All notable changes to LazyQMK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.13.0] - 2026-01-09

### Added

#### Web Editor & API Server
- **Full web-based keyboard layout editor** - Complete browser-based UI with feature parity to TUI
- **REST API server** (`lazyqmk-web` binary) - Axum-based backend serving both API and web UI
- **Single-binary deployment** - Frontend embedded using rust-embed for portable deployment
- **Template browser & creator** - Browse and apply layout templates, save custom templates
- **First-run onboarding flow** - Guided setup with template selection
- **Visual keyboard preview** - Interactive keyboard renderer with hover details and key metadata
- **Keyboard navigation** - TUI-like keyboard shortcuts for power users
- **Settings manager** - Configure QMK paths, workspace directory, and themes
- **Dark mode support** - Automatic dark mode via system preferences

#### Firmware Operations
- **Firmware generation** - Generate QMK config files (JSON, keymap.c) and download as ZIP
- **Firmware building** - Compile .uf2/.hex/.bin firmware directly from browser
- **Real-time build logs** - Live streaming logs with color-coded levels (INFO/WARN/ERROR)
- **Artifact management** - Download compiled firmware with SHA256 checksums
- **Build history** - View past builds filtered by layout
- **Process control** - Cancel running builds (kills underlying QMK compile process)
- **Automatic cleanup** - Remove old artifacts (7-day age limit, 50 max builds)

#### Developer Experience
- **Cross-platform dev tooling** - `npm run dev:web` starts both backend and frontend
- **Production builds** - `RUST_EMBED=true npm run build` for static site generation
- **Comprehensive documentation** - `docs/WEB_DEPLOYMENT.md` and `web/README.md`
- **CI/CD integration** - GitHub Actions builds lazyqmk-web binaries for all platforms

### Fixed
- Fixed SvelteKit adapter warning in production builds (use adapter-static with RUST_EMBED flag)
- Fixed artifact download 404 errors (missing /download suffix)
- Fixed artifact file existence check before canonicalize
- Fixed UF2 format display in Build tab description
- Fixed tab overflow on narrow screens (dropdown navigation for secondary tabs)

### Changed
- Updated to Ratatui 0.29, Crossterm 0.29 (archived/021-dependency-updates)
- Migrated from deprecated serde_yaml to serde_yml 0.0.12
- Refactored to remove all Vial generation code (deprecated)
- Build cancellation now kills process immediately (not just marks cancelled)
- Generate job system with ZIP packaging for firmware files

### Technical Details
- Added `web` feature flag for optional web dependencies
- Dependencies: Axum 0.8, Tower-HTTP 0.6, Tokio 1.x, rust-embed 8.5, mime_guess 2.0
- Frontend: SvelteKit, Tailwind CSS, shadcn-svelte components
- Development: Node.js 20+, Vite 6.0 with hot-reload
- Build artifacts stored in `.lazyqmk/build_output/{job_id}/`
- Logs stored in `.lazyqmk/build_logs/{job_id}.log`

### Documentation
- Added `docs/WEB_DEPLOYMENT.md` - Production deployment guide
- Updated `web/README.md` - Development and build instructions
- Added inline API documentation for all REST endpoints

## [0.12.2] - Previous Release

(See git tags for earlier releases)

[0.13.0]: https://github.com/Radialarray/LazyQMK/compare/v0.12.2...v0.13.0
