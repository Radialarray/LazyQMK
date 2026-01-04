# SvelteKit Frontend Setup Instructions

## Quick Start

```bash
# 1. Install dependencies
cd web
npm install

# 2. Start the backend (in another terminal)
cd ..
cargo run --features web --bin lazyqmk-web

# 3. Start the frontend dev server
cd web
npm run dev
```

Visit http://localhost:5173

## What Was Created

### Project Structure

```
web/
├── src/
│   ├── lib/
│   │   ├── api/              # API client
│   │   │   ├── types.ts      # TypeScript types matching Rust backend
│   │   │   ├── client.ts     # API client with all endpoints
│   │   │   ├── index.ts      # Exports
│   │   │   └── client.test.ts # API client tests
│   │   ├── components/       # UI components
│   │   │   ├── Button.svelte # Button component with variants
│   │   │   ├── Button.test.ts # Button tests
│   │   │   ├── Card.svelte   # Card container
│   │   │   ├── Input.svelte  # Input field
│   │   │   └── index.ts      # Exports
│   │   └── utils/
│   │       ├── cn.ts         # Class name utility (tailwind-merge + clsx)
│   │       └── index.ts      # Exports
│   ├── routes/
│   │   ├── +layout.svelte        # Root layout with dark mode
│   │   ├── +page.svelte          # Dashboard (/)
│   │   ├── layouts/
│   │   │   ├── +page.svelte      # Layout list (/layouts)
│   │   │   └── [name]/
│   │   │       └── +page.svelte  # Layout editor (/layouts/[name])
│   │   ├── keycodes/
│   │   │   └── +page.svelte      # Keycode browser (/keycodes)
│   │   └── settings/
│   │       └── +page.svelte      # Settings (/settings)
│   ├── test/
│   │   └── setup.ts          # Vitest setup
│   ├── app.html              # HTML template
│   └── app.css               # Global styles (Tailwind)
├── e2e/
│   ├── dashboard.spec.ts     # Dashboard E2E tests
│   └── layouts.spec.ts       # Layouts E2E tests with mocking
├── static/
│   └── favicon.png           # Favicon
├── package.json              # Dependencies
├── vite.config.ts            # Vite config with backend proxy
├── svelte.config.js          # SvelteKit config
├── tailwind.config.js        # Tailwind config
├── postcss.config.js         # PostCSS config
├── tsconfig.json             # TypeScript config
├── playwright.config.ts      # Playwright config
└── README.md                 # Full documentation
```

### Routes Implemented

1. **/ (Dashboard)**
   - Backend health check
   - Quick links to all sections
   - Connection status display

2. **/layouts (Layout List)**
   - Lists all layout files from workspace
   - Shows metadata (name, description, modified date)
   - Click to open layout editor

3. **/layouts/[name] (Layout Editor)**
   - View layout metadata
   - View layers with colors
   - Placeholder for visual editor (future)

4. **/keycodes (Keycode Browser)**
   - Browse all QMK keycodes
   - Filter by category
   - Search by name/code
   - Shows descriptions

5. **/settings (Settings)**
   - Configure QMK firmware path
   - View workspace root
   - Save configuration

### API Client

The `ApiClient` class provides methods for all backend endpoints:

```typescript
import { apiClient } from '$lib/api';

// Health check
const health = await apiClient.health();

// Layouts
const layouts = await apiClient.listLayouts();
const layout = await apiClient.getLayout('my-layout.md');
await apiClient.saveLayout('my-layout.md', layout);

// Keycodes
const keycodes = await apiClient.listKeycodes('KC_A', 'basic');
const categories = await apiClient.listCategories();

// Config
const config = await apiClient.getConfig();
await apiClient.updateConfig({ qmk_firmware_path: '/path/to/qmk' });

// Geometry
const geometry = await apiClient.getGeometry('crkbd', 'LAYOUT_split_3x6_3');
```

### Components

Three base components are included:

1. **Button** - Multiple variants (default, destructive, outline, secondary, ghost, link)
2. **Card** - Container with border and shadow
3. **Input** - Styled text input with Tailwind classes

All components use Svelte 5 runes ($props, $state, etc.)

### Testing

**Unit Tests (Vitest)**
```bash
npm run test              # Run once
npm run test:watch        # Watch mode
npm run test:ui           # Open UI
```

- API client tests (mocked fetch)
- Component tests (testing-library/svelte)

**E2E Tests (Playwright)**
```bash
npm run test:e2e          # Run tests
npm run test:e2e:ui       # Open UI
```

- Dashboard navigation tests
- Layouts page with mocked API responses
- Tests work with or without real backend

## Development Workflow

### 1. With Real Backend

```bash
# Terminal 1: Backend
cargo run --features web --bin lazyqmk-web

# Terminal 2: Frontend
cd web
npm run dev
```

### 2. With Mocked Backend (E2E Tests)

E2E tests automatically mock API responses:

```typescript
await page.route('**/api/layouts', async (route) => {
  await route.fulfill({
    status: 200,
    body: JSON.stringify({ layouts: [...] })
  });
});
```

### 3. Backend Proxy Configuration

Dev server proxies `/api` and `/health` to `http://localhost:3000` (see `vite.config.ts`).

To change backend URL:

```typescript
// vite.config.ts
server: {
  proxy: {
    '/api': {
      target: 'http://localhost:8080', // Change port
      changeOrigin: true
    }
  }
}
```

## Customization

### Adding a New Route

1. Create `src/routes/myroute/+page.svelte`
2. Import components: `import { Button, Card } from '$components'`
3. Use API client: `import { apiClient } from '$api'`
4. Add navigation link in dashboard

### Adding a New Component

1. Create `src/lib/components/MyComponent.svelte`
2. Export in `src/lib/components/index.ts`
3. Add tests in `src/lib/components/MyComponent.test.ts`
4. Use in routes: `import { MyComponent } from '$components'`

### Updating API Types

When backend types change:

1. Update `src/lib/api/types.ts` to match Rust types
2. Update `ApiClient` methods if endpoints changed
3. Run tests: `npm run test && npm run check`

## Production Build

```bash
npm run build       # Build to build/
npm run preview     # Preview production build
```

### Deployment Options

- **Static**: Vercel, Netlify, Cloudflare Pages (default adapter-auto)
- **Node.js**: Install `@sveltejs/adapter-node` and update svelte.config.js
- **Docker**: See README.md for Dockerfile example

## Troubleshooting

### "Cannot find module 'mode-watcher'"

Install dependencies:
```bash
npm install
```

### Backend Connection Failed

1. Check backend is running: `curl http://localhost:3000/health`
2. Check proxy config in `vite.config.ts`
3. Check CORS in backend (should allow `http://localhost:5173`)

### TypeScript Errors

```bash
npm run check          # Full type check
npm run check:watch    # Watch mode
```

### Dark Mode Not Working

Ensure `ModeWatcher` component is in `src/routes/+layout.svelte`

## Next Steps

This is a solid foundation. Future enhancements:

1. **Visual Layout Editor**
   - Canvas-based keyboard renderer
   - Drag-and-drop key assignment
   - Real-time preview

2. **Advanced Features**
   - Undo/redo for edits
   - Layout import/export
   - Firmware compilation from UI
   - Build log viewer

3. **UI Improvements**
   - Keyboard shortcuts
   - Toast notifications
   - Loading skeletons
   - Better mobile support

4. **State Management**
   - Svelte stores for global state
   - Optimistic updates
   - Offline support

## Architecture Notes

### Why This Stack?

- **SvelteKit**: Fast, modern, great DX
- **Svelte 5**: Latest features (runes, snippets)
- **Tailwind**: Utility-first, customizable
- **shadcn-svelte**: High-quality component patterns
- **Vitest**: Fast unit testing
- **Playwright**: Reliable E2E testing

### Design Decisions

1. **API Client Pattern**: Centralized client with typed responses
2. **Component Library**: Copy-paste approach (no heavy dependencies)
3. **Route Structure**: File-based routing (SvelteKit convention)
4. **Testing Strategy**: Unit tests for logic, E2E for user flows
5. **Styling**: Tailwind with design tokens for consistency

### Backend Integration

Frontend expects these backend routes (all implemented):

- `GET /health` - HealthResponse
- `GET /api/layouts` - LayoutListResponse
- `GET /api/layouts/{filename}` - Layout
- `PUT /api/layouts/{filename}` - void (204)
- `GET /api/keycodes?search=&category=` - KeycodeListResponse
- `GET /api/keycodes/categories` - CategoryListResponse
- `GET /api/config` - ConfigResponse
- `PUT /api/config` - void (204)
- `GET /api/keyboards/{keyboard}/geometry/{layout}` - GeometryResponse

All types in `src/lib/api/types.ts` match Rust backend definitions.

## Support

For issues or questions:
1. Check README.md in web/ directory
2. Review test files for examples
3. See ARCHITECTURE.md in project root (Rust backend docs)
