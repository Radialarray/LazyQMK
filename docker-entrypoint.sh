#!/bin/bash
set -e

# Fix ownership of bind-mounted volumes so the lazyqmk user can read/write.
# This runs as root on container start, then drops privileges via gosu.
echo "[entrypoint] Fixing permissions for bind-mounted volumes..."
chown -R lazyqmk:lazyqmk /app/workspace /app/qmk_firmware 2>/dev/null || true
# Also ensure the config directory is owned correctly (named volume may already be fine)
chown -R lazyqmk:lazyqmk /home/lazyqmk 2>/dev/null || true

# Drop to lazyqmk user and exec the main binary
echo "[entrypoint] Starting LazyQMK as user lazyqmk (uid $(id -u lazyqmk))..."
exec gosu lazyqmk lazyqmk "$@"
