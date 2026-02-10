import { defineConfig, devices } from '@playwright/test';

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
	testDir: './e2e',
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 2 : 0,
	workers: process.env.CI ? 1 : undefined,
	reporter: 'html',
	use: {
		baseURL: 'http://localhost:4173',
		trace: 'on-first-retry'
	},

	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'] }
		}
	],

	webServer: [
		// Start backend first and wait for health check
		{
			command: 'cd .. && cargo run --features web -- web',
			url: 'http://localhost:3001/health',
			reuseExistingServer: !process.env.CI,
			timeout: 120000 // 2 minutes for cargo build
		},
		// Start frontend after backend is healthy
		{
			command: 'npm run build && npm run preview',
			port: 4173,
			reuseExistingServer: !process.env.CI
		}
	]
});
