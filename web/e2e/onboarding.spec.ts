import { test, expect } from '@playwright/test';

// Mock responses
const mockPreflightFirstRun = {
	qmk_configured: false,
	has_layouts: false,
	first_run: true,
	qmk_firmware_path: null
};

const mockPreflightQmkConfigured = {
	qmk_configured: true,
	has_layouts: false,
	first_run: false,
	qmk_firmware_path: '/path/to/qmk_firmware'
};

const mockTemplates = {
	templates: [
		{
			filename: 'test-template.md',
			name: 'Test Template',
			description: 'A test template for testing',
			author: 'Test Author',
			tags: ['test', 'demo'],
			created: '2024-01-01T00:00:00Z',
			layer_count: 3
		}
	]
};

const mockKeyboards = {
	keyboards: [
		{ path: 'crkbd', layout_count: 2 },
		{ path: 'splitkb/halcyon/corne', layout_count: 1 },
		{ path: 'planck', layout_count: 3 }
	]
};

const mockVariants = {
	keyboard: 'crkbd',
	variants: [
		{ name: 'LAYOUT_split_3x6_3', key_count: 42 }
	]
};

test.describe('Onboarding flow - First run', () => {
	test.beforeEach(async ({ page }) => {
		// Mock preflight for first run
		await page.route('**/api/preflight', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockPreflightFirstRun)
			});
		});
	});

	test('shows welcome page on first run', async ({ page }) => {
		await page.goto('/onboarding');

		// Should show welcome heading
		await expect(page.getByRole('heading', { name: 'Welcome to LazyQMK' })).toBeVisible();

		// Should show QMK configuration step
		await expect(page.getByRole('heading', { name: 'Configure QMK Firmware' })).toBeVisible();

		// Should have QMK path input
		await expect(page.locator('input#qmk-path')).toBeVisible();
	});

	test('shows step 1 indicator for QMK config', async ({ page }) => {
		await page.goto('/onboarding');

		// Canonical flow card should show current step
		await expect(page.getByText('Canonical setup flow')).toBeVisible();
		await expect(page.getByText(/Step 1 of 2/)).toBeVisible();
	});

	test('has Go to Home link', async ({ page }) => {
		await page.goto('/onboarding');

		// Should have home link
		await expect(page.getByRole('link', { name: /Go to Home/i })).toBeVisible();
	});

	test('home link navigates to home', async ({ page }) => {
		await page.goto('/onboarding');

		// Mock preflight to return configured state so home doesn't redirect back
		await page.route('**/api/preflight', async (route) => {
			// After clicking home, return configured state so home loads
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({
					qmk_configured: true,
					has_layouts: false,
					first_run: false,
					qmk_firmware_path: '/path/to/qmk_firmware'
				})
			});
		});

		// Mock layouts for home page
		await page.route('**/api/layouts', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({ layouts: [] })
			});
		});

		// Click home link
		await page.getByRole('link', { name: /Go to Home/i }).click();

		// Should navigate to home (/) without redirecting back to onboarding
		await expect(page).toHaveURL('/');
	});
});

test.describe('Onboarding flow - QMK configured', () => {
	test.beforeEach(async ({ page }) => {
		// Mock preflight for QMK configured
		await page.route('**/api/preflight', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockPreflightQmkConfigured)
			});
		});

		// Mock templates
		await page.route('**/api/templates', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockTemplates)
			});
		});

		// Mock keyboards
		await page.route('**/api/keyboards', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockKeyboards)
			});
		});
	});

	test('shows choose step when QMK is configured', async ({ page }) => {
		await page.goto('/onboarding');

		// Should show canonical start-point heading
		await expect(page.getByRole('heading', { name: 'Choose your starting point' })).toBeVisible();

		// Should show template and scratch options
		await expect(page.getByRole('heading', { name: 'Start from Template' })).toBeVisible();
		await expect(page.getByRole('heading', { name: 'Start from Scratch' })).toBeVisible();
		await expect(page.getByText(/After layout opens, core editing stays in workspace tabs\./i)).toBeVisible();
	});

	test('shows available templates', async ({ page }) => {
		await page.goto('/onboarding');

		// Should show template list
		await expect(page.getByRole('heading', { name: 'Available Templates' })).toBeVisible();
		// Use button role to target the template card specifically
		await expect(page.getByRole('button', { name: /Test Template/ })).toBeVisible();
	});

	test('shows template unavailable message on choose step when templates missing', async ({ page }) => {
		await page.route('**/api/templates', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({ templates: [] })
			});
		});

		await page.goto('/onboarding');

		await expect(page.getByText('Template start unavailable')).toBeVisible();
		await expect(
			page.getByText(/Choose “From Scratch” instead of falling back automatically\./)
		).toBeVisible();
	});

	test('can select a template', async ({ page }) => {
		await page.goto('/onboarding');

		// Click on the template card button
		await page.getByRole('button', { name: /Test Template/ }).click();

		// Should show Apply Template step
		await expect(page.getByRole('heading', { name: 'Apply Template' })).toBeVisible();
		await expect(page.getByText('Creating from:')).toBeVisible();
	});

	test('can navigate to create from scratch', async ({ page }) => {
		await page.goto('/onboarding');

		// Click From Scratch button
		await page.getByRole('button', { name: 'Start from Scratch' }).click();

		// Should show Create New Layout step
		await expect(page.getByRole('heading', { name: 'Create New Layout' })).toBeVisible();

		// Recognition mode is default; exact search starts hidden
		await expect(page.getByRole('button', { name: 'Recognize by board family' })).toBeVisible();
		await expect(page.getByText('Split and ergonomic')).toBeVisible();
		await expect(page.locator('input#keyboard-search')).toHaveClass(/sr-only/);
	});
});

test.describe('Onboarding flow - Create from scratch', () => {
	test.beforeEach(async ({ page }) => {
		await page.route('**/api/preflight', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockPreflightQmkConfigured)
			});
		});

		await page.route('**/api/templates', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({ templates: [] }) // No templates
			});
		});

		await page.route('**/api/keyboards', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockKeyboards)
			});
		});

		await page.route('**/api/keyboards/crkbd/layouts', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockVariants)
			});
		});
	});

	test('can search keyboards after switching browse mode', async ({ page }) => {
		await page.goto('/onboarding');

		await page.getByRole('button', { name: 'Start from Scratch' }).click();
		await page.getByRole('button', { name: 'Search exact names' }).click();
		await expect(page.locator('input#keyboard-search')).toBeVisible();
		await page.locator('input#keyboard-search').fill('crkbd');
		await expect(page.getByText('crkbd').first()).toBeVisible();
	});

	test('can select keyboard and variant', async ({ page }) => {
		await page.goto('/onboarding');

		// Navigate to create from scratch
		await page.getByRole('button', { name: 'Start from Scratch' }).click();

		// Select crkbd from recognition cards
		await page.getByRole('button', { name: /crkbd.*2 layout variants available/i }).first().click();

		// Should show variant selection
		await expect(page.getByText('LAYOUT_split_3x6_3')).toBeVisible();
	});

	test('shows layout name input after variant selection', async ({ page }) => {
		await page.goto('/onboarding');

		// Navigate to create from scratch
		await page.getByRole('button', { name: 'Start from Scratch' }).click();

		// Select keyboard
		await page.getByRole('button', { name: /crkbd.*2 layout variants available/i }).first().click();

		// Select variant
		await page.getByText('LAYOUT_split_3x6_3').click();

		// Should show layout name input
		await expect(page.locator('input#layout-name-create')).toBeVisible();
	});

	test('can go back from create step', async ({ page }) => {
		await page.goto('/onboarding');

		// Navigate to create from scratch
		await page.getByRole('button', { name: 'Start from Scratch' }).click();

		// Click cancel/back button
		await page.getByRole('button', { name: 'Cancel' }).click();

		// Should be back on choose step
		await expect(page.getByRole('heading', { name: 'Choose your starting point' })).toBeVisible();
	});
});

test.describe('Onboarding navigation header', () => {
	test('onboarding page does not show navigation header', async ({ page }) => {
		await page.route('**/api/preflight', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockPreflightFirstRun)
			});
		});

		await page.goto('/onboarding');

		// Should NOT see the nav header
		await expect(page.locator('header nav')).not.toBeVisible();
	});

	test('home page shows navigation header', async ({ page }) => {
		await page.route('**/api/preflight', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockPreflightQmkConfigured)
			});
		});

		await page.route('**/api/layouts', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({ layouts: [] })
			});
		});

		await page.goto('/');

		// Should see the nav header with LazyQMK link
		await expect(page.locator('header a').filter({ hasText: 'LazyQMK' })).toBeVisible();
	});
});

test.describe('Navigation header - promoted primary destinations', () => {
	test.beforeEach(async ({ page }) => {
		await page.route('**/api/preflight', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockPreflightQmkConfigured)
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

	test('header shows primary destinations directly', async ({ page }) => {
		await page.goto('/');

		const nav = page.getByRole('navigation');
		await expect(nav.getByRole('link', { name: 'My Layouts' })).toBeVisible();
		await expect(nav.getByRole('link', { name: 'Create Layout' })).toBeVisible();
		await expect(nav.getByRole('link', { name: 'Starter Layouts' })).toBeVisible();
		await expect(nav.getByRole('link', { name: 'Keycode Reference' })).toBeVisible();
		await expect(nav.getByRole('link', { name: 'Workspace Setup' })).toBeVisible();
	});

	test('can navigate to Settings from header', async ({ page }) => {
		await page.goto('/');

		await page.getByRole('link', { name: 'Workspace Setup' }).click();

		// Should navigate to settings
		await expect(page).toHaveURL('/settings');
	});

	test('can navigate to Keycodes from header', async ({ page }) => {
		await page.goto('/');

		await page.getByRole('link', { name: 'Keycode Reference' }).click();

		// Should navigate to keycodes
		await expect(page).toHaveURL('/keycodes');
	});
});

test.describe('Deep links still work', () => {
	test.beforeEach(async ({ page }) => {
		await page.route('**/api/preflight', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockPreflightQmkConfigured)
			});
		});
	});

	test('can access /settings directly', async ({ page }) => {
		await page.goto('/settings');

		// Should be on settings page
		await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();
	});

	test('can access /keycodes directly', async ({ page }) => {
		await page.goto('/keycodes');

		// Should be on keycodes page
		await expect(page.getByRole('heading', { name: 'Keycodes Browser' })).toBeVisible();
	});

	test('can access /templates directly', async ({ page }) => {
		await page.route('**/api/templates', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockTemplates)
			});
		});

		await page.goto('/templates');

		// Should be on templates page
		await expect(page.getByRole('heading', { name: 'Layout Templates' })).toBeVisible();
	});

	test('can access /layouts directly', async ({ page }) => {
		await page.route('**/api/layouts', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({ layouts: [] })
			});
		});

		await page.goto('/layouts');

		// Should be on layouts page
		await expect(page.getByRole('heading', { name: 'Layouts' })).toBeVisible();
	});
});
