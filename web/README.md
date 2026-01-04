# LazyQMK Web Frontend

Modern web-based frontend for LazyQMK keyboard layout editor, built with SvelteKit, Tailwind CSS, and shadcn-svelte.

## Features

- 🎨 **Modern UI**: Clean, responsive design with Tailwind CSS and shadcn-svelte components
- 🌙 **Dark Mode**: Automatic dark mode support via mode-watcher
- 📱 **Responsive**: Works on desktop, tablet, and mobile devices
- 🔌 **API Integration**: Connects to LazyQMK Axum backend
- ✅ **Tested**: Vitest unit tests and Playwright E2E tests

## Project Structure

```
web/
├── src/
│   ├── lib/
│   │   ├── api/           # API client and types
│   │   ├── components/    # Reusable UI components
│   │   ├── stores/        # Svelte stores (future)
│   │   └── utils/         # Utility functions
│   ├── routes/
│   │   ├── /              # Dashboard
│   │   ├── /layouts       # Layout list
│   │   ├── /layouts/[name] # Layout editor
│   │   ├── /keycodes      # Keycode browser
│   │   └── /settings      # Settings page
│   ├── test/              # Test utilities
│   ├── app.html           # HTML template
│   └── app.css            # Global styles
├── e2e/                   # Playwright E2E tests
├── static/                # Static assets
└── package.json
```

## Prerequisites

- Node.js 18+ (or Bun)
- LazyQMK backend running on port 3000 (or configured URL)

## Getting Started

### 1. Install Dependencies

```bash
cd web
npm install
# or
bun install
```

### 2. Start Backend

In a separate terminal, start the LazyQMK backend:

```bash
cd ..
cargo run --features web --bin lazyqmk-web
```

The backend will run on `http://localhost:3000` by default.

### 3. Start Development Server

```bash
npm run dev
```

The frontend will be available at `http://localhost:5173`

## Development

### Available Scripts

```bash
# Development
npm run dev              # Start dev server with hot reload
npm run build            # Build for production
npm run preview          # Preview production build

# Testing
npm run test             # Run unit tests with Vitest
npm run test:watch       # Run tests in watch mode
npm run test:ui          # Open Vitest UI
npm run test:e2e         # Run Playwright E2E tests
npm run test:e2e:ui      # Open Playwright UI

# Code Quality
npm run check            # Type-check with svelte-check
npm run check:watch      # Type-check in watch mode
```

### Backend Configuration

The frontend expects the backend at `http://localhost:3000` by default. To use a different URL:

1. **Development**: Edit `vite.config.ts` proxy configuration
2. **Production**: Set backend URL when initializing ApiClient

Example custom backend URL:

```typescript
import { ApiClient } from '$lib/api';

const client = new ApiClient('https://api.example.com');
```

### API Endpoints

The frontend connects to these backend endpoints:

- `GET /health` - Health check
- `GET /api/layouts` - List layouts
- `GET /api/layouts/{filename}` - Get layout
- `PUT /api/layouts/{filename}` - Save layout
- `GET /api/keycodes` - Search keycodes
- `GET /api/keycodes/categories` - List categories
- `GET /api/config` - Get configuration
- `PUT /api/config` - Update configuration
- `GET /api/keyboards/{keyboard}/geometry/{layout}` - Get keyboard geometry

## Routes

### `/` - Dashboard

Overview of LazyQMK with quick links to:
- Layouts management
- Keycode browser
- Settings
- Backend connection status

### `/layouts` - Layout List

Browse all keyboard layouts in the workspace:
- View layout metadata
- Open layouts for editing
- Filter and search (future)

### `/layouts/[name]` - Layout Editor

View and edit a specific layout:
- Layer visualization
- Key assignments
- Metadata editing
- Visual keyboard editor (placeholder)

### `/keycodes` - Keycode Browser

Browse and search QMK keycodes:
- Category filtering
- Search by name or code
- Keycode descriptions
- Copy to clipboard (future)

### `/settings` - Settings

Configure LazyQMK:
- QMK firmware path
- Workspace directory
- Build settings

## Testing

### Unit Tests (Vitest)

```bash
npm run test
```

Tests are colocated with source files (`.test.ts` extension):
- `src/lib/api/client.test.ts` - API client tests
- `src/lib/components/Button.test.ts` - Component tests

### E2E Tests (Playwright)

```bash
npm run test:e2e
```

E2E tests are in the `e2e/` directory:
- `e2e/dashboard.spec.ts` - Dashboard smoke tests
- `e2e/layouts.spec.ts` - Layout list with mocked backend

#### Testing Strategies

**With Real Backend:**
```bash
# Terminal 1: Start backend
cargo run --features web --bin lazyqmk-web

# Terminal 2: Run E2E tests
cd web
npm run test:e2e
```

**With Mocked Backend:**

Tests automatically mock API responses using Playwright's `route.fulfill()`:

```typescript
await page.route('**/api/layouts', async (route) => {
  await route.fulfill({
    status: 200,
    body: JSON.stringify({ layouts: [...] })
  });
});
```

## Styling

### Tailwind CSS

This project uses Tailwind CSS with a custom configuration:
- Custom color palette (light/dark mode)
- shadcn-svelte compatible design tokens
- Responsive breakpoints

### shadcn-svelte

UI components are built with shadcn-svelte principles:
- Copy-paste components (not a dependency)
- Full customization
- Built on bits-ui

Current components:
- `Button` - Button with variants (default, destructive, outline, etc.)
- `Card` - Container with border and shadow
- `Input` - Text input with styling

## Deployment

### Build for Production

```bash
npm run build
```

This generates a production build in the `build/` directory.

### Preview Production Build

```bash
npm run preview
```

### Deployment Options

#### Static Hosting (Vercel, Netlify, Cloudflare Pages)

The default adapter (`@sveltejs/adapter-auto`) works with most platforms.

#### Node.js Server

```bash
npm install @sveltejs/adapter-node
```

Update `svelte.config.js`:

```javascript
import adapter from '@sveltejs/adapter-node';
```

#### Docker

Create a `Dockerfile`:

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --production
COPY . .
RUN npm run build
EXPOSE 3000
CMD ["node", "build"]
```

## Troubleshooting

### Backend Connection Errors

**Problem**: `Failed to connect to backend`

**Solutions**:
1. Verify backend is running: `curl http://localhost:3000/health`
2. Check CORS configuration in backend
3. Update proxy in `vite.config.ts`

### Dark Mode Not Working

**Problem**: Theme not switching

**Solutions**:
1. Check `mode-watcher` is installed
2. Verify `ModeWatcher` component in `+layout.svelte`
3. Clear browser cache

### Type Errors

**Problem**: TypeScript errors in API types

**Solutions**:
1. Run `npm run check` to see all errors
2. Ensure types match backend Rust definitions
3. Regenerate types if backend changed

## Contributing

1. Follow existing code style
2. Add tests for new features
3. Update types when backend changes
4. Run `npm run check` before committing

## License

MIT - See LICENSE in project root
