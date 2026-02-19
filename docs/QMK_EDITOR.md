# QMK Editor Integration

This guide explains how to use the official QMK Configurator alongside LazyQMK in the Docker Compose stack.

## Overview

The LazyQMK Docker stack includes the official [QMK Configurator](https://github.com/qmk/qmk_configurator), a web-based visual editor for creating QMK keymaps. This provides an alternative workflow for users who prefer a visual interface over LazyQMK's TUI or want to leverage both tools together.

## Services in the Stack

| Service | Port | Description |
|---------|------|-------------|
| **LazyQMK Backend** | 3001 | REST API for LazyQMK layout management |
| **LazyQMK Frontend** | 5173 | SvelteKit web UI for LazyQMK |
| **QMK Editor** | 8080 | Official QMK Configurator web interface |

## Quick Start

### 1. Start the Stack

Start all services including QMK Editor:

```bash
docker compose up -d
```

Or start only specific services:

```bash
# Start LazyQMK only
docker compose up -d backend frontend

# Start QMK Editor only
docker compose up -d qmk-editor
```

### 2. Access the Services

- **LazyQMK Web UI**: [http://localhost:5173](http://localhost:5173)
- **QMK Configurator**: [http://localhost:8080](http://localhost:8080)
- **LazyQMK API**: [http://localhost:3001](http://localhost:3001)

### 3. Stop the Services

```bash
# Stop all services
docker compose down

# Stop only QMK Editor
docker compose stop qmk-editor
```

## Using QMK Configurator

### Basic Workflow

1. **Open QMK Editor**: Navigate to [http://localhost:8080](http://localhost:8080)
2. **Select Keyboard**: Choose your keyboard from the dropdown or search
3. **Configure Layout**: Click keys to assign keycodes using the visual interface
4. **Compile Firmware**: Click "Compile" to generate firmware on QMK's servers
5. **Download Firmware**: Download the compiled `.hex` or `.bin` file
6. **Flash Keyboard**: Use QMK Toolbox or `qmk flash` to flash your keyboard

### API Configuration

By default, the QMK Configurator uses the public QMK API (`https://api.qmk.fm`) for:
- Keyboard metadata (supported keyboards, layouts)
- Firmware compilation

You can override this to use a local QMK API instance:

```yaml
# docker-compose.yml
qmk-editor:
  environment:
    - VITE_API_URL=http://localhost:5001  # Your local qmk_api instance
```

**Note**: Running a local QMK API requires additional setup. See the [qmk_api repository](https://github.com/qmk/qmk_api) for details.

## Workflow Comparison: LazyQMK vs QMK Configurator

### LazyQMK Strengths

- **Fast iteration**: Immediate visual feedback in TUI
- **Advanced features**: Tap Dance, Combos, RGB effects with state machine generation
- **Layout management**: Store multiple layouts as human-readable Markdown files
- **Offline-first**: No internet required, works fully locally
- **Version control friendly**: Markdown layouts work well with Git

### QMK Configurator Strengths

- **Visual interface**: Intuitive drag-and-drop keymap editing
- **Beginner-friendly**: No command-line knowledge required
- **Official support**: Maintained by QMK team, always up-to-date
- **Cloud compilation**: No local QMK setup needed
- **Cross-platform**: Works in any modern browser

### Recommended Hybrid Workflow

1. **Initial setup**: Use QMK Configurator to create a basic keymap visually
2. **Export JSON**: Download the keymap JSON from QMK Configurator
3. **Import to LazyQMK**: Convert the JSON to LazyQMK's Markdown format (feature planned)
4. **Advanced features**: Add Tap Dance, Combos, RGB effects in LazyQMK
5. **Iterate**: Use LazyQMK TUI for quick tweaks, QMK Configurator for major layout changes

## Port Configuration

### Default Ports

The default configuration uses these ports:

- `8080`: QMK Editor web interface
- `5173`: LazyQMK frontend
- `3001`: LazyQMK backend API

### Customizing Ports

Edit `docker-compose.yml` to change ports:

```yaml
qmk-editor:
  ports:
    - "9090:80"  # Access QMK Editor at http://localhost:9090
```

Or use environment variables:

```bash
# Create .env file
echo "QMK_EDITOR_PORT=9090" > .env

# Update docker-compose.yml to use:
# ports:
#   - "${QMK_EDITOR_PORT:-8080}:80"
```

## Troubleshooting

### QMK Editor Not Loading

**Problem**: Browser shows "Connection refused" at `http://localhost:8080`

**Solutions**:

1. Check if the container is running:
   ```bash
   docker compose ps qmk-editor
   ```

2. View container logs:
   ```bash
   docker compose logs qmk-editor
   ```

3. Restart the service:
   ```bash
   docker compose restart qmk-editor
   ```

### Compilation Fails

**Problem**: "Compilation failed" error in QMK Configurator

**Possible causes**:
- Invalid keycode combinations
- Unsupported keyboard features
- QMK API service unavailable

**Solutions**:
1. Check the QMK API status: [https://api.qmk.fm/v1/healthcheck](https://api.qmk.fm/v1/healthcheck)
2. Verify your keymap is valid (no conflicting keycodes)
3. Try compiling with LazyQMK as an alternative

### Port Already in Use

**Problem**: `Error starting userland proxy: listen tcp4 0.0.0.0:8080: bind: address already in use`

**Solution**: Another service is using port 8080. Either:

1. Stop the conflicting service:
   ```bash
   lsof -ti:8080 | xargs kill
   ```

2. Change QMK Editor's port in `docker-compose.yml`:
   ```yaml
   qmk-editor:
     ports:
       - "8081:80"  # Use port 8081 instead
   ```

## Advanced Configuration

### Building a Custom Image

To build QMK Configurator from source with custom modifications:

1. Clone the repository:
   ```bash
   git clone https://github.com/qmk/qmk_configurator.git
   cd qmk_configurator
   ```

2. Make your changes to the source code

3. Update `docker-compose.yml`:
   ```yaml
   qmk-editor:
     build:
       context: ./qmk_configurator
       dockerfile: Dockerfile
     # Remove the 'image' line
   ```

4. Rebuild and restart:
   ```bash
   docker compose build qmk-editor
   docker compose up -d qmk-editor
   ```

### Using a Specific Version

Pin to a specific QMK Configurator version:

```yaml
qmk-editor:
  image: qmkfm/qmk_configurator:v1.0.0  # Replace with desired version
```

Find available versions at: [Docker Hub - qmkfm/qmk_configurator](https://hub.docker.com/r/qmkfm/qmk_configurator/tags)

### Resource Limits

Limit CPU and memory usage:

```yaml
qmk-editor:
  deploy:
    resources:
      limits:
        cpus: '0.5'
        memory: 512M
      reservations:
        cpus: '0.25'
        memory: 256M
```

## Security Considerations

### Network Isolation

By default, all services share the `lazyqmk-network` bridge network. This allows:
- Communication between services (e.g., frontend → backend)
- Internet access for downloading resources

To isolate QMK Editor from other services:

```yaml
qmk-editor:
  networks:
    - qmk-network  # Separate network

networks:
  qmk-network:
    driver: bridge
```

### Firewall Rules

If exposing services to your LAN:

```bash
# Allow only from specific IP range
sudo ufw allow from 192.168.1.0/24 to any port 8080
```

### HTTPS with Reverse Proxy

For production deployments, use a reverse proxy (nginx, Caddy, Traefik) to add HTTPS:

```nginx
server {
    listen 443 ssl http2;
    server_name qmk-editor.local;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## Integration Ideas

### Future Integration Features (Planned)

- **Import QMK JSON**: Load QMK Configurator JSON keymaps into LazyQMK
- **Export to QMK JSON**: Export LazyQMK layouts for use in QMK Configurator
- **Unified firmware compilation**: Use QMK Editor's layout with LazyQMK's advanced features
- **Keymap comparison**: Visual diff between LazyQMK and QMK Configurator layouts

### Community Contributions Welcome

If you have ideas for better integration between LazyQMK and QMK Configurator, please:
- Open an issue: [LazyQMK Issues](https://github.com/svenlochner/LazyQMK/issues)
- Submit a PR with implementation
- Share your workflow in Discussions

## Related Documentation

- [QMK Configurator Repository](https://github.com/qmk/qmk_configurator)
- [QMK Configurator Live Site](https://config.qmk.fm)
- [LazyQMK Web Features](./WEB_FEATURES.md)
- [LazyQMK Web Deployment](./WEB_DEPLOYMENT.md)
- [Docker Compose Documentation](https://docs.docker.com/compose/)

## Contributing

### Reporting Issues

If you encounter issues with:
- **QMK Editor itself**: Report to [qmk/qmk_configurator](https://github.com/qmk/qmk_configurator/issues)
- **Docker integration**: Report to [LazyQMK](https://github.com/svenlochner/LazyQMK/issues)

### Improving This Documentation

Found a typo or have a better explanation? PRs welcome!

```bash
# Edit this file
vim docs/QMK_EDITOR.md

# Submit PR with your improvements
git checkout -b docs/improve-qmk-editor-docs
git commit -m "docs: improve QMK Editor integration guide"
git push origin docs/improve-qmk-editor-docs
```
