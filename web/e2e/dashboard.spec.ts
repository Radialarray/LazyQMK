import { test, expect } from '@playwright/test';

// Mock preflight response for returning user (not first run)
const mockPreflightReturningUser = {
	qmk_configured: true,
	has_layouts: true,
	first_run: false,
	qmk_firmware_path: '/path/to/qmk_firmware'
};

// Mock preflight response for first-run user
const mockPreflightFirstRun = {
	qmk_configured: false,
	has_layouts: false,
	first_run: true,
	qmk_firmware_path: null
};

// Mock preflight response for user with layouts but no QMK config
const mockPreflightNoQmk = {
	qmk_configured: false,
	has_layouts: true,
	first_run: false,
	qmk_firmware_path: null
};

// Mock layouts
const mockLayouts = {
	layouts: [
		{
			filename: 'test-layout.md',
			name: 'Test Layout',
			description: 'A test keyboard layout',
			modified: '2024-01-03T12:00:00Z'
		},
		{
			filename: 'another-layout.md',
			name: 'Another Layout',
			description: 'Another test layout',
			modified: '2024-01-02T12:00:00Z'
		},
		{
			filename: 'old-layout.md',
			name: 'Old Layout',
			description: 'An older layout',
			modified: '2024-01-01T12:00:00Z'
		}
	]
};

test.describe('Home page - configured user', () => {
	test.beforeEach(async ({ page }) => {
		// Mock the preflight API to simulate returning user
		await page.route('**/api/preflight', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockPreflightReturningUser)
			});
		});

		// Mock the layouts API
		await page.route('**/api/layouts', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockLayouts)
			});
		});
	});

	test('loads the layout-focused home page', async ({ page }) => {
		await page.goto('/');

		// Check that the main heading is visible
		await expect(page.getByRole('heading', { name: 'LazyQMK', level: 1 })).toBeVisible();

		// Check primary actions are present
		await expect(page.locator('[data-testid="primary-actions"]')).toBeVisible();
		await expect(page.getByRole('heading', { name: 'Start Layout Setup' })).toBeVisible();
		await expect(page.getByRole('heading', { name: 'Open Layout Workspace' })).toBeVisible();
		await expect(page.getByRole('heading', { name: 'Main areas' })).toBeVisible();

		// Check recent layouts section is present
		await expect(page.locator('[data-testid="recent-layouts"]')).toBeVisible();
		await expect(page.getByRole('heading', { name: 'Recent Layouts' })).toBeVisible();
	});

	test('shows recent layouts sorted by date', async ({ page }) => {
		await page.goto('/');

		// Wait for layouts to load - use h3 heading elements for exact match
		await expect(page.locator('[data-testid="recent-layout-item"] h3').filter({ hasText: 'Test Layout' })).toBeVisible();
		await expect(page.locator('[data-testid="recent-layout-item"] h3').filter({ hasText: 'Another Layout' })).toBeVisible();
		await expect(page.locator('[data-testid="recent-layout-item"] h3').filter({ hasText: 'Old Layout' })).toBeVisible();
	});

	test('navigates to create new layout via primary action', async ({ page }) => {
		await page.goto('/');

		// Click the create new layout card
		await page.locator('[data-testid="create-layout-action"]').click();

		// Should navigate to /onboarding
		await expect(page).toHaveURL('/onboarding');
	});

	test('navigates to layouts list via primary action', async ({ page }) => {
		await page.goto('/');

		// Click the open existing layout card
		await page.locator('[data-testid="open-layout-action"]').click();

		// Should navigate to /layouts
		await expect(page).toHaveURL('/layouts');
	});

	test('navigates to layout editor when clicking recent layout', async ({ page }) => {
		await page.goto('/');

		// Click on a recent layout
		await page.locator('[data-testid="recent-layout-item"]').first().click();

		// Should navigate to layout editor
		await expect(page).toHaveURL(/\/layouts\/test-layout\.md/);
	});

	test('shows View all link when layouts exist', async ({ page }) => {
		await page.goto('/');

		await expect(page.getByRole('link', { name: 'View all' })).toBeVisible();
	});

	test('navigates to layouts via nav', async ({ page }) => {
		await page.goto('/');

		// Click the "Layouts" nav link
		await page.getByRole('link', { name: 'Layouts', exact: true }).click();

		// Should navigate to /layouts
		await expect(page).toHaveURL('/layouts');
		await expect(page.getByRole('heading', { name: 'Layouts' })).toBeVisible();
	});

	test('navigates to settings via header nav', async ({ page }) => {
		await page.goto('/');

		await page.getByRole('link', { name: 'Settings' }).click();

		// Should navigate to /settings
		await expect(page).toHaveURL('/settings');
		await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();
	});

	test('does not expose global build entry in header nav', async ({ page }) => {
		await page.goto('/');

		await expect(page.getByRole('link', { name: 'Build' })).toHaveCount(0);
		await expect(page.getByRole('button', { name: 'More' })).toHaveCount(0);
	});
});

test.describe('Home page - no layouts', () => {
	test.beforeEach(async ({ page }) => {
		await page.route('**/api/preflight', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockPreflightReturningUser)
			});
		});

		await page.route('**/api/layouts', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({ layouts: [] })
			});
		});
	});

	test('shows empty state for recent layouts', async ({ page }) => {
		await page.goto('/');

		await expect(page.getByText('No layouts yet')).toBeVisible();
		await expect(page.getByText('Create your first layout to get started')).toBeVisible();
	});

	test('does not show View all link when no layouts', async ({ page }) => {
		await page.goto('/');

		await expect(page.getByRole('link', { name: 'View all' })).not.toBeVisible();
	});
});

test.describe('Home page - first run redirect', () => {
	test('redirects to onboarding on first run', async ({ page }) => {
		// Mock preflight to indicate first run
		await page.route('**/api/preflight', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockPreflightFirstRun)
			});
		});

		await page.goto('/');

		// Should redirect to onboarding
		await expect(page).toHaveURL('/onboarding');
	});

	test('shows onboarding welcome message after redirect', async ({ page }) => {
		await page.route('**/api/preflight', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockPreflightFirstRun)
			});
		});

		await page.goto('/');

		// Should show welcome heading
		await expect(page.getByRole('heading', { name: 'Welcome to LazyQMK' })).toBeVisible();
	});

	test('redirects to onboarding when QMK not configured', async ({ page }) => {
		// Mock preflight to indicate QMK not configured
		await page.route('**/api/preflight', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockPreflightNoQmk)
			});
		});

		await page.goto('/');

		// Should redirect to onboarding
		await expect(page).toHaveURL('/onboarding');
	});
});

test.describe('Home page - error handling', () => {
	test('shows error state on API failure', async ({ page }) => {
		await page.route('**/api/preflight', async (route) => {
			await route.fulfill({
				status: 500,
				contentType: 'application/json',
				body: JSON.stringify({ error: 'Internal server error' })
			});
		});

		await page.goto('/');

		// Should show error message
		await expect(page.getByRole('heading', { name: 'Connection Error' })).toBeVisible();
		await expect(page.getByRole('button', { name: 'Retry' })).toBeVisible();
	});
});
