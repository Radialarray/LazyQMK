# Deploy Guide

How to deploy LazyQMK to a remote server using Docker Compose.

## Prerequisites

- Docker with `buildx` (Docker Desktop includes this)
- SSH access to the server (`ssh root@<server-ip>`)
- Server has: Docker, Docker Compose, `qmk_firmware/` directory, `docker-compose.yml`, `.env`

## Quick Deploy

```bash
# 1. Build for linux/amd64 (from repo root)
docker buildx build --platform linux/amd64 -t lazyqmk-backend:latest -f Dockerfile --load .

# 2. Save, transfer, load
docker save lazyqmk-backend:latest | gzip > /tmp/lazyqmk-backend.tar.gz
scp /tmp/lazyqmk-backend.tar.gz root@<server-ip>:/tmp/
ssh root@<server-ip> "docker load < /tmp/lazyqmk-backend.tar.gz"

# 3. Restart
ssh root@<server-ip> "cd /root/apps/lazyqmk && docker compose up -d"

# 4. Verify
ssh root@<server-ip> "curl -s http://localhost:3001/health"

# 5. Clean up
rm /tmp/lazyqmk-backend.tar.gz
ssh root@<server-ip> "rm /tmp/lazyqmk-backend.tar.gz"
```

## Why Build Locally?

The server has limited disk (8GB). Docker builds exhaust it. Building on your dev machine is fast (cached layers rebuild in seconds) and avoids disk pressure on the server.

## What Happens on Container Start

The `docker-entrypoint.sh` runs automatically and handles:

1. **Permissions** — `chown` on bind-mounted volumes (`workspace/`, `qmk_firmware/`) so the `lazyqmk` user (uid 999) can read/write
2. **Git submodule fix** — Recursively finds `.git` files (broken submodule pointers) in `qmk_firmware/` and converts them to standalone repos so `git describe` works during compilation
3. **Safe directory config** — Marks all git directories as `safe.directory` for both root and lazyqmk users
4. **QMK Python deps** — Installs `requirements.txt` on first run (creates `.qmk_setup_done` marker)
5. **Drop privileges** — Switches from root to `lazyqmk` user via `gosu`

All steps are idempotent — safe to run on every container restart.

## Frontend Image

The frontend (`web/Dockerfile`) follows the same process if you change frontend code:

```bash
docker buildx build --platform linux/amd64 -t lazyqmk-frontend:latest -f web/Dockerfile --load web/
docker save lazyqmk-frontend:latest | gzip > /tmp/lazyqmk-frontend.tar.gz
scp /tmp/lazyqmk-frontend.tar.gz root@<server-ip>:/tmp/
ssh root@<server-ip> "docker load < /tmp/lazyqmk-frontend.tar.gz"
ssh root@<server-ip> "cd /root/apps/lazyqmk && docker compose up -d"
```

## Cargo.lock

`Cargo.lock` is gitignored but required for Docker builds. It's baked into the image at build time, so you only need to worry about it locally. If you get dependency errors after adding/removing crates, run `cargo generate-lockfile` first.

## Server Architecture

```
Browser → Cloudflare Tunnel → cloudflared → Caddy :80
                                              ├── /api/* → backend:3001
                                              ├── /health → backend:3001
                                              └── /*     → frontend:5173
```

The server's `docker-compose.yml` defines four services: `backend`, `frontend`, `caddy` (reverse proxy), and `cloudflared` (tunnel).

## Troubleshooting

**Backend unhealthy after deploy** — Check logs: `ssh root@<server-ip> "docker logs lazyqmk-backend"`

**QMK compilation fails** — Verify git repos are intact: `docker exec lazyqmk-backend git -C /app/qmk_firmware describe` should return a version tag.

**Disk full on server** — Prune old images: `ssh root@<server-ip> "docker image prune -af && docker builder prune -af"`

**302 redirect on public URL** — Cloudflare Access policy is blocking. Disable it in Zero Trust dashboard → Access → Applications.

## Related Docs

- [DOCKER_BUILD.md](DOCKER_BUILD.md) — Rust version pinning, build optimization, Node.js setup
- [DOCKER_QMK_SETUP.md](DOCKER_QMK_SETUP.md) — QMK firmware integration details
- [WEB_DEPLOYMENT.md](WEB_DEPLOYMENT.md) — Standalone binary deployment (no Docker)
