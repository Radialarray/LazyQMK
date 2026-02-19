# Docker Compose QMK Firmware Setup Guide

This guide explains how to integrate QMK firmware with LazyQMK's Docker Compose stack.

## Overview

LazyQMK requires access to QMK firmware for:
- Reading keyboard geometry and layout metadata (`info.json`)
- Generating firmware files (`keymap.c`, `keymap.json`)
- Compiling firmware binaries (`.hex`, `.bin`, `.uf2`)

The Docker Compose stack supports **three approaches** for providing QMK firmware:

1. **Git Submodule (Recommended)** - Use the included submodule
2. **External Clone** - Point to your own QMK firmware directory
3. **Docker Volume** - Clone into a persistent volume (advanced)

## Size Implications

| Component | Size | Notes |
|-----------|------|-------|
| QMK Firmware (full clone) | ~1.1 GB | Includes all keyboards, build system, toolchains |
| LazyQMK Backend Docker Image | ~100 MB | Does NOT include QMK firmware |
| Total Stack (backend + frontend + QMK editor) | ~250 MB | Images only, QMK firmware separate |

**Why QMK firmware is NOT embedded in the Docker image:**
- **Size**: Would increase image from 100 MB to 1.2+ GB
- **Updates**: Users can update QMK firmware independently
- **Flexibility**: Users can use custom QMK forks
- **Build time**: Avoids long image build times

## Approach 1: Git Submodule (Recommended)

The LazyQMK repository includes QMK firmware as a git submodule. This is the **simplest and recommended approach**.

### Initial Setup

```bash
# Clone LazyQMK with submodules
git clone --recurse-submodules https://github.com/Radialarray/LazyQMK.git
cd LazyQMK

# Or if already cloned without submodules:
git submodule update --init --recursive qmk_firmware
```

### Start Docker Compose

```bash
# Uses ./qmk_firmware by default
docker compose up -d

# Verify QMK firmware is mounted
docker compose exec backend ls -lh /app/qmk_firmware
```

The docker-compose.yml defaults to mounting `./qmk_firmware` (the submodule) into the backend container.

### Update QMK Firmware

```bash
# Update submodule to latest commit
cd qmk_firmware
git pull origin master
git submodule update --init --recursive
cd ..

# Restart containers to pick up changes
docker compose restart backend backend-dev
```

### Submodule Details

- **Repository**: https://github.com/Radialarray/qmk_firmware.git
- **Location**: `./qmk_firmware/` (relative to project root)
- **Commit**: Pinned to a known-good commit (see `.gitmodules`)
- **Custom fork**: Includes LazyQMK-specific features (LED/RGB support)

**Important:** Never commit changes to the `qmk_firmware/` submodule unless you intend to update the pinned commit. Add `qmk_firmware/` to your `.git/info/exclude` or global gitignore.

## Approach 2: External QMK Clone

If you already have QMK firmware cloned elsewhere (e.g., for other projects), you can point LazyQMK to it.

### Setup

```bash
# Clone QMK firmware to your preferred location
git clone --recurse-submodules https://github.com/qmk/qmk_firmware.git ~/qmk_firmware

# Or use the custom fork for full LazyQMK features:
git clone --recurse-submodules https://github.com/Radialarray/qmk_firmware.git ~/qmk_firmware
```

### Configure Docker Compose

Set the `QMK_FIRMWARE_PATH` environment variable:

```bash
# Option A: Export in shell
export QMK_FIRMWARE_PATH=~/qmk_firmware
docker compose up -d

# Option B: Create .env file
echo "QMK_FIRMWARE_PATH=$HOME/qmk_firmware" > .env
docker compose up -d

# Option C: Inline with docker-compose command
QMK_FIRMWARE_PATH=~/qmk_firmware docker compose up -d
```

### Verify Mount

```bash
# Check backend container has access
docker compose exec backend ls -lh /app/qmk_firmware/keyboards

# Should show directories like: crkbd/, ergodox_ez/, planck/, etc.
```

### Use Case

This approach is ideal if:
- You maintain multiple QMK-based projects
- You have custom QMK configurations or keyboards
- You want to share QMK firmware across tools (QMK CLI, LazyQMK, etc.)

## Approach 3: Docker Volume (Advanced)

For fully containerized setups where you don't want QMK firmware on the host filesystem.

### Setup

```bash
# Create a named volume for QMK firmware
docker volume create lazyqmk-qmk-firmware

# Clone QMK firmware into the volume (one-time setup)
docker run --rm -v lazyqmk-qmk-firmware:/qmk alpine sh -c "
  apk add --no-cache git && 
  git clone --recurse-submodules https://github.com/Radialarray/qmk_firmware.git /qmk
"
```

### Modify docker-compose.yml

Create a `docker-compose.override.yml` file:

```yaml
services:
  backend:
    volumes:
      # Replace the default QMK firmware bind mount with named volume
      - qmk-firmware-volume:/app/qmk_firmware:ro

  backend-dev:
    volumes:
      - qmk-firmware-volume:/app/qmk_firmware:ro

volumes:
  qmk-firmware-volume:
    external: true
    name: lazyqmk-qmk-firmware
```

### Start Services

```bash
docker compose up -d
```

### Update QMK Firmware

```bash
# Update firmware inside the volume
docker run --rm -v lazyqmk-qmk-firmware:/qmk alpine sh -c "
  apk add --no-cache git && 
  cd /qmk && 
  git pull origin master && 
  git submodule update --init --recursive
"

# Restart services
docker compose restart backend backend-dev
```

### Use Case

This approach is ideal for:
- Kubernetes or cloud deployments
- CI/CD pipelines
- Environments where host filesystem access is restricted
- Production servers without local QMK development

## Comparison Table

| Approach | Setup Complexity | Disk Usage | Update Process | Use Case |
|----------|------------------|------------|----------------|----------|
| **Git Submodule** | ⭐ Easy | Host: 1.1 GB | `git submodule update` | Local development, recommended |
| **External Clone** | ⭐⭐ Medium | Host: 1.1 GB | `git pull` in QMK dir | Shared QMK across projects |
| **Docker Volume** | ⭐⭐⭐ Advanced | Volume: 1.1 GB | Docker run command | Cloud/K8s deployments |

## Troubleshooting

### Backend shows "QMK firmware not found"

**Problem**: Container logs show errors about missing QMK firmware.

**Solution**:
```bash
# Check if qmk_firmware is initialized
ls -la qmk_firmware/

# If empty or missing, initialize submodule
git submodule update --init --recursive qmk_firmware

# Restart services
docker compose restart backend
```

### Keyboard not found in database

**Problem**: LazyQMK web UI doesn't show your keyboard.

**Solution**:
```bash
# Verify QMK firmware is mounted correctly
docker compose exec backend ls /app/qmk_firmware/keyboards/

# Check for your keyboard directory
docker compose exec backend ls /app/qmk_firmware/keyboards/crkbd/

# If missing, update QMK firmware (see approach-specific instructions above)
```

### Permission denied errors

**Problem**: Container can't read QMK firmware files.

**Solution**:
```bash
# Ensure host directory has read permissions
chmod -R a+rX qmk_firmware/

# For SELinux systems (Fedora, RHEL, CentOS)
chcon -Rt svirt_sandbox_file_t qmk_firmware/

# Restart services
docker compose restart backend
```

### Submodule is empty or uninitialized

**Problem**: `qmk_firmware/` directory exists but is empty.

**Solution**:
```bash
# Initialize and update submodule
git submodule update --init --recursive qmk_firmware

# Verify files exist
ls qmk_firmware/keyboards/

# Restart services
docker compose restart backend
```

### Volume mount not updating after QMK changes

**Problem**: Updated QMK firmware on host, but container still shows old files.

**Solution**:
```bash
# Restart services to remount volumes
docker compose restart backend backend-dev

# If that doesn't work, recreate containers
docker compose up -d --force-recreate backend backend-dev
```

### Docker volume is missing

**Problem**: Using volume approach, but volume doesn't exist.

**Solution**:
```bash
# List existing volumes
docker volume ls | grep qmk

# Create the volume if missing (see Approach 3 instructions)
docker volume create lazyqmk-qmk-firmware

# Re-run the git clone command to populate it
```

## Best Practices

### Development Workflow

1. **Use git submodule** for simplest setup
2. **Pin submodule commits** for reproducible builds
3. **Document QMK version** in your project (tag, commit hash)
4. **Test firmware updates** before committing submodule changes

### Production Deployment

1. **Use named volumes** for cloud deployments
2. **Mount read-only** (`:ro`) to prevent accidental modifications
3. **Version QMK firmware** with tags or commit hashes
4. **Backup volume** before major QMK updates

### QMK Firmware Updates

```bash
# Check current version
cd qmk_firmware && git log -1 --oneline && cd ..

# Test update in isolation first
cd qmk_firmware
git fetch origin
git checkout <new-commit-or-tag>
git submodule update --init --recursive
cd ..

# Test LazyQMK with updated firmware
docker compose up -d
# Test keyboard loading, firmware generation, etc.

# If successful, commit the submodule update
git add qmk_firmware
git commit -m "chore: update QMK firmware to <version>"
```

### Avoiding Submodule Commits

If you accidentally stage submodule changes:

```bash
# Unstage submodule changes
git reset qmk_firmware

# Or restore to current commit
git submodule update --init --recursive qmk_firmware
```

Add to `.git/info/exclude`:
```
qmk_firmware/
```

## Configuration Reference

### Environment Variables

Set in `.env` file or export in shell:

```bash
# Path to QMK firmware on host (default: ./qmk_firmware)
QMK_FIRMWARE_PATH=~/qmk_firmware

# Path to layout files on host (default: ./examples)
LAZYQMK_WORKSPACE=~/my-layouts

# Config directory (default: named volume)
LAZYQMK_CONFIG_DIR=~/.config/LazyQMK
```

### docker-compose.yml Volume Syntax

```yaml
volumes:
  # Bind mount (default): mounts host directory
  - ${QMK_FIRMWARE_PATH:-./qmk_firmware}:/app/qmk_firmware:ro
  
  # Named volume: uses Docker-managed volume
  - qmk-firmware-volume:/app/qmk_firmware:ro
  
  # Absolute path: explicitly specify host path
  - /home/user/qmk_firmware:/app/qmk_firmware:ro
```

### Health Check

The backend includes a health check endpoint. Verify QMK firmware is accessible:

```bash
# Check backend health
docker compose exec backend curl -f http://localhost:3001/health

# Check QMK firmware mount
docker compose exec backend test -d /app/qmk_firmware/keyboards && echo "OK" || echo "FAIL"
```

## Related Documentation

- [docker-compose.yml](../docker-compose.yml) - Full stack configuration
- [WEB_DEPLOYMENT.md](WEB_DEPLOYMENT.md) - Production deployment guide
- [web/SETUP.md](../web/SETUP.md) - Web editor setup
- [QUICKSTART.md](../QUICKSTART.md) - User guide
- [QMK Documentation](https://docs.qmk.fm/) - Official QMK docs

## Support

If you encounter issues not covered here:
- Check [GitHub Issues](https://github.com/Radialarray/LazyQMK/issues)
- Read the [Troubleshooting](#troubleshooting) section
- Open a new issue with:
  - Docker Compose version (`docker compose version`)
  - LazyQMK version (`docker compose exec backend lazyqmk-web --version`)
  - Output of `docker compose config` (sanitized)
  - Container logs (`docker compose logs backend`)
