# LazyQMK Documentation

This directory contains comprehensive documentation for LazyQMK development and deployment.

## 📋 Documentation Index

### User Guides
- **[QUICKSTART.md](../QUICKSTART.md)** - Getting started with LazyQMK (terminal and web)
- **[README.md](../README.md)** - Project overview, installation, and basic usage

### Deployment & Configuration
- **[DOCKER_QMK_SETUP.md](DOCKER_QMK_SETUP.md)** - Docker Compose and QMK firmware integration guide
  - QMK firmware setup options (submodule, external, volume)
  - Environment variable configuration
  - Volume mounting strategies
  - Troubleshooting common issues
- **[WEB_DEPLOYMENT.md](WEB_DEPLOYMENT.md)** - Production deployment for web interface
  - Building standalone binary with embedded frontend
  - Systemd service configuration
  - Reverse proxy setup (nginx, Caddy)
  - Security considerations

### Development
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Technical architecture and design decisions
- **[FEATURES.md](FEATURES.md)** - Complete feature list and implementation details
- **[TESTING.md](TESTING.md)** - Testing strategy and guidelines
- **[AGENTS.md](../AGENTS.md)** - Development workflow and contribution guide

### Web Interface
- **[web/SETUP.md](../web/SETUP.md)** - Web editor setup for development
  - Local development with Vite
  - Docker deployment
  - Tauri desktop application
- **[WEB_FEATURES.md](WEB_FEATURES.md)** - Web interface feature documentation

### Reference
- **[EXPORT_FORMAT.md](EXPORT_FORMAT.md)** - Layout export format specification
- **[BRANDING.md](BRANDING.md)** - Project branding guidelines
- **[RIPPLE_VALIDATION.md](RIPPLE_VALIDATION.md)** - RGB ripple effect validation

## 🚀 Quick Links by Use Case

### "I want to run LazyQMK with Docker"
1. Read [DOCKER_QMK_SETUP.md](DOCKER_QMK_SETUP.md) for QMK firmware setup
2. Copy `.env.example` to `.env` and customize
3. Run `git submodule update --init --recursive qmk_firmware`
4. Run `docker compose up -d`

### "I want to develop the web frontend"
1. Read [web/SETUP.md](../web/SETUP.md) for development setup
2. Follow the "Single-Command Development" section
3. Visit http://localhost:5173

### "I want to deploy LazyQMK to production"
1. Read [WEB_DEPLOYMENT.md](WEB_DEPLOYMENT.md) for building standalone binary
2. Configure systemd or Docker for your environment
3. Set up reverse proxy if needed

### "I want to understand the codebase"
1. Read [ARCHITECTURE.md](ARCHITECTURE.md) for high-level design
2. Read [AGENTS.md](../AGENTS.md) for development guidelines
3. Read [FEATURES.md](FEATURES.md) for feature implementation details

### "I want to contribute"
1. Read [AGENTS.md](../AGENTS.md) for development workflow
2. Read [ARCHITECTURE.md](ARCHITECTURE.md) for architecture overview
3. Read [TESTING.md](TESTING.md) for testing requirements
4. Follow the coding standards in AGENTS.md

## 📦 Docker Documentation Tree

```
Docker Compose Setup
├── DOCKER_QMK_SETUP.md (this is the main guide)
│   ├── Approach 1: Git Submodule (recommended)
│   ├── Approach 2: External QMK Clone
│   └── Approach 3: Docker Volume (advanced)
├── .env.example (environment variable template)
├── docker-compose.yml (orchestration with inline comments)
└── web/SETUP.md (development and alternative deployment)
```

## 🔧 Development Workflow

### Before Starting Development
- Read [AGENTS.md](../AGENTS.md) for coding standards
- Read [ARCHITECTURE.md](ARCHITECTURE.md) for system design
- Run `cargo test` to ensure environment is working

### Testing Requirements
- All tests must pass: `cargo test`
- Clippy must pass: `cargo clippy --all-features -- -D warnings`
- Pre-release tests: See [TESTING.md](TESTING.md)

### Committing Changes
- Follow Conventional Commits format (see AGENTS.md)
- Run tests before committing
- Use meaningful commit messages

## 📖 External Resources

- [QMK Firmware Documentation](https://docs.qmk.fm/) - Official QMK docs
- [Custom QMK Fork](https://github.com/Radialarray/qmk_firmware) - LazyQMK's QMK fork with RGB support
- [Ratatui Documentation](https://ratatui.rs/) - TUI framework docs
- [SvelteKit Documentation](https://kit.svelte.dev/) - Web framework docs

## 🤝 Need Help?

- **Bug reports**: [GitHub Issues](https://github.com/Radialarray/LazyQMK/issues)
- **Questions**: Open a GitHub Discussion
- **Contributing**: See [AGENTS.md](../AGENTS.md) and [ARCHITECTURE.md](ARCHITECTURE.md)

## 📝 Documentation Style

When adding new documentation:
- Use clear section headers with emoji for visual hierarchy
- Include code examples with comments
- Add troubleshooting sections for common issues
- Link to related documentation
- Keep language concise and accessible
- Use tables for comparisons
- Include "Quick Start" sections for practical tasks
