# LazyQMK Web Editor - Features Documentation

> **Last Updated:** 2026-01-09  
> **Version:** 0.13.0

The LazyQMK Web Editor provides a modern browser-based interface for designing keyboard layouts with full feature parity to the terminal UI.

---

## Overview

The `lazyqmk-web` binary is a standalone web server that combines:
- **REST API Backend** - Layout management and firmware operations
- **Embedded Web Frontend** - SvelteKit-based UI compiled into the binary
- **Single Binary Deployment** - No separate installation or configuration needed

**Access:** `http://localhost:3001` (default) after running `lazyqmk-web`

---

## Core Features

### 1. Visual Keyboard Layout Editor

**Interactive Editing**
- Click any key to open the keycode picker
- Drag-and-drop keycode assignment (coming soon)
- Real-time visual preview of keyboard geometry
- Accurate physical key positioning based on QMK `info.json`
- Split keyboard support (Corne, Ergodox, Ferris Sweep, etc.)

**Keycode Picker**
- Fuzzy search through 600+ QMK keycodes
- Category-based filtering (Basic, Navigation, Symbols, Function, Media, Modifiers)
- Keyboard shortcuts for power users (arrow keys, Enter to select, Esc to cancel)
- Real-time search filtering
- Recent keycodes for quick access

### 2. Layer Management

**Layer Operations**
- Create new layers with custom names
- Delete layers (with confirmation)
- Switch between layers via tabs
- Visual layer tabs showing all layers
- Layer default color assignment
- Layer category assignment

**Layer Organization**
- Layer naming for easy identification
- Color-coded layer tabs
- Drag-to-reorder layers (coming soon)
- Duplicate layers (coming soon)

### 3. Color System

**Four-Level Color Priority**
1. **Individual key color override** (highest priority)
2. **Key category color** 
3. **Layer category color**
4. **Layer default color** (lowest priority)

**Color Picker**
- RGB channel sliders (0-255)
- Hex code input (#RRGGBB)
- Live color preview
- Recent colors palette
- Material Design color presets

**Category System**
- Create custom categories (navigation, symbols, numbers, etc.)
- Assign colors to categories
- Apply categories to keys or entire layers
- Category manager UI

### 4. Firmware Generation

**Generate QMK Files**
- Creates `keymap.json` and `keymap.c` files
- Generates RGB LED matrix definitions (`led_map.h`)
- Exports as `.zip` archive for easy download
- Preserves layout metadata in JSON

**Advanced Features**
- Idle effect screensaver configuration (timeout, duration, effect type)
- RGB matrix configuration
- Per-key RGB definitions
- Visual-to-matrix coordinate mapping

### 5. Firmware Building

**One-Click Compilation**
- Build firmware directly in the browser
- Download compiled artifacts (.uf2, .hex, .bin)
- Real-time build progress indicator
- Build cancellation support

**Build Logs**
- Live streaming compilation output
- Syntax highlighting for errors/warnings
- Auto-scroll to latest output
- Searchable log history
- Copy logs to clipboard

**Build History**
- Track all firmware builds with timestamps
- SHA256 checksums for artifact verification
- File size and format information
- Quick re-download of previous builds
- Delete old builds manually

**Artifact Management**
- Automatic cleanup (7 days old, 50 max builds)
- Manual cleanup via UI
- Download individual artifacts or full build ZIP
- Artifact metadata (size, checksum, format)

### 6. Settings Manager

**QMK Configuration**
- Set QMK firmware path
- Set keyboard name (e.g., `crkbd/rev1`)
- Set layout variant (e.g., `LAYOUT_split_3x6_3`)
- Validate paths and keyboard existence
- Auto-discover available keyboards

**Workspace Configuration**
- Set custom workspace directory
- Browse layouts in workspace
- Create new layouts from templates
- Import/export layouts

**Build Settings**
- Configure build artifacts directory
- Set compilation flags
- Enable/disable verbose build output

### 7. Layout File Management

**Supported Operations**
- List all layouts in workspace
- Create new layout from scratch or template
- Open existing layout
- Save layout (auto-save on major operations)
- Rename layout
- Delete layout (with confirmation)
- Export layout as Markdown documentation

**File Format**
- Human-readable Markdown with YAML frontmatter
- Version control friendly (plain text, diffable)
- Compatible with TUI (`lazyqmk`) layouts
- Schema version tracking

**Metadata**
- Name, description, author
- Creation and modification timestamps
- Tags for searchability
- Template flag
- Layout variant

### 8. Template System

**Template Browser**
- View available templates with metadata
- Filter templates by tags
- Create new layout from template
- Save current layout as template

**Template Sharing**
- Export template as `.md` file
- Import template from file
- Share via git repository
- Compatible with TUI templates

### 9. Export System

**Markdown Documentation Export**
- Generate visual keyboard documentation
- Layer-by-layer layout diagrams
- Color legend with category breakdown
- Keycode reference tables
- Metadata summary
- Printable format

**Export Formats**
- Markdown with embedded diagrams
- Plain text layout tables
- JSON schema (QMK-compatible)
- PDF (coming soon)

### 10. User Interface

**Modern Design**
- shadcn-svelte component library
- Tailwind CSS styling
- Responsive layout (desktop, tablet, mobile)
- Smooth animations and transitions
- Accessible components (ARIA labels, keyboard navigation)

**Dark Mode**
- Automatic theme detection from system preferences
- Manual theme toggle
- Persistent theme selection
- Syntax highlighting adapted to theme

**Keyboard Shortcuts**
- Arrow keys for key navigation
- Enter to edit key
- Esc to cancel dialogs
- Ctrl+S to save layout
- Ctrl+B to build firmware
- Ctrl+G to generate firmware files
- Tab/Shift+Tab to switch layers
- ? to show keyboard shortcuts help

**Navigation**
- Breadcrumb navigation
- Quick actions toolbar
- Sidebar for layout list
- Modal dialogs for focused tasks

### 11. Real-Time Updates

**Live Feedback**
- Instant visual updates on keycode changes
- Live color preview while editing
- Real-time validation of QMK paths
- Dynamic keyboard geometry rendering

**Progress Indicators**
- Build progress bar
- Loading spinners for async operations
- Toast notifications for actions
- Error messages with suggestions

### 12. Multi-Layout Support

**Workspace Management**
- Browse multiple layouts in workspace
- Quick-switch between layouts
- Recent layouts list
- Search layouts by name/tags

**Layout Organization**
- Group layouts by keyboard
- Filter by template status
- Sort by creation/modification date
- Tag-based filtering

---

## REST API

The web editor communicates with the backend via a comprehensive REST API. See [API Documentation](../web/README.md#api-endpoints) for details.

**Key Endpoints:**
- `GET /api/layouts` - List all layouts
- `GET /api/layouts/:id` - Get layout details
- `PUT /api/layouts/:id` - Update layout
- `POST /api/layouts` - Create new layout
- `DELETE /api/layouts/:id` - Delete layout
- `POST /api/layouts/:id/generate` - Generate firmware files
- `POST /api/layouts/:id/build` - Build firmware
- `GET /api/builds` - List build history
- `GET /api/builds/:id/logs` - Get build logs (SSE streaming)
- `DELETE /api/builds/:id` - Cancel/delete build
- `GET /api/settings` - Get settings
- `PUT /api/settings` - Update settings
- `GET /api/keyboards` - List available keyboards
- `GET /api/keyboards/:id/layouts` - Get keyboard layout variants

---

## Browser Compatibility

**Supported Browsers:**
- Chrome/Edge 90+ (recommended)
- Firefox 88+
- Safari 14+
- Opera 76+

**Required Features:**
- ES6+ JavaScript
- CSS Grid and Flexbox
- Fetch API
- Server-Sent Events (SSE) for build logs
- LocalStorage for theme persistence

---

## Performance

**Optimizations:**
- Lazy loading of components
- Virtual scrolling for large lists
- Debounced search inputs
- Efficient DOM updates via Svelte reactivity
- Code splitting for faster initial load

**Resource Usage:**
- Minimal memory footprint (~50MB for web server)
- Efficient CPU usage (idle when not building)
- Network traffic only for API calls (no polling)

---

## Security

**Local-Only by Default:**
- Binds to `127.0.0.1:3001` (localhost only)
- No external network access required
- No telemetry or analytics

**Production Deployment:**
- Use reverse proxy (nginx, Caddy) with HTTPS
- Add authentication if exposing to network
- See [WEB_DEPLOYMENT.md](WEB_DEPLOYMENT.md) for production setup

---

## Development

Want to contribute to the web frontend? See [web/README.md](../web/README.md) for:
- Development setup with hot-reload
- Frontend architecture (SvelteKit + Tailwind)
- Component structure
- Build system (`dev.mjs` script)
- Testing (Playwright E2E tests)

---

## Comparison: Web vs TUI

| Feature | Web Editor | TUI |
|---------|-----------|-----|
| **Interface** | Browser-based GUI | Terminal-based TUI |
| **Mouse Support** | ✅ Full | ❌ Keyboard only |
| **Keyboard Shortcuts** | ✅ Yes | ✅ Yes |
| **Build Logs** | ✅ Live streaming | ✅ Live display |
| **Firmware Download** | ✅ Direct download | ✅ Saved to disk |
| **Remote Access** | ✅ Over network | ⚠️ SSH required |
| **Resource Usage** | ~50MB | ~10MB |
| **Dependencies** | Web browser | Terminal emulator |
| **Deployment** | Single binary + browser | Single binary |
| **Feature Parity** | ✅ 100% | ✅ 100% |

**Recommendation:** Use whichever interface matches your workflow. Both share the same layout file format and can be used interchangeably!

---

## Troubleshooting

**Port Already in Use:**
```bash
# Use custom port
lazyqmk-web --port 8080
```

**Cannot Access from Another Device:**
```bash
# Bind to all interfaces (CAUTION: exposes to local network)
lazyqmk-web --host 0.0.0.0
```

**Build Fails:**
- Check QMK firmware path in Settings
- Verify QMK CLI is installed: `qmk --version`
- Check build logs for specific errors
- Ensure keyboard exists: `qmk list-keyboards | grep <keyboard>`

**Layout Not Saving:**
- Check workspace directory permissions
- Verify disk space available
- Check browser console for errors (F12)

**Blank Page:**
- Hard refresh browser (Ctrl+Shift+R)
- Clear browser cache
- Check browser console for errors
- Verify `lazyqmk-web` is running

---

## Future Features (Roadmap)

**Planned for Future Releases:**
- Drag-and-drop keycode assignment from palette
- Visual layer reordering (drag tabs)
- Duplicate layer function
- PDF export for documentation
- Multi-keyboard workspace (switch keyboards without restarting)
- Collaborative editing (multiple users)
- Cloud sync for layouts
- Mobile app (PWA)
- Advanced macro editor UI
- Combo key configuration UI
- Visual tap dance builder

See GitHub issues for tracking: https://github.com/Radialarray/LazyQMK/issues

---

## Additional Resources

- **Main README:** [README.md](../README.md) - Quick start and installation
- **Web Deployment:** [WEB_DEPLOYMENT.md](WEB_DEPLOYMENT.md) - Production deployment guide
- **Web Development:** [web/README.md](../web/README.md) - Frontend development setup
- **TUI Features:** [FEATURES.md](FEATURES.md) - Terminal UI feature documentation
- **Architecture:** [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture overview
