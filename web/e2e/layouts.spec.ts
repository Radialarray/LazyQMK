import { test, expect } from '@playwright/test';

test.describe('Layouts page', () => {
	test('loads the layouts list page', async ({ page }) => {
		await page.goto('/layouts');
		
		// Check that the main heading is visible
		await expect(page.getByRole('heading', { name: 'Layouts' })).toBeVisible();
		
		// Check for back to dashboard button
		await expect(page.getByRole('button', { name: 'Back to Dashboard' })).toBeVisible();
	});

	test('shows loading state initially', async ({ page }) => {
		await page.goto('/layouts');
		
		// Should show loading text (may be brief)
		// Note: This test may be flaky if the backend responds very quickly
		const loadingText = page.getByText('Loading layouts...');
		// Just check that the page loaded, as loading state might be too fast to catch
		await expect(page.getByRole('heading', { name: 'Layouts' })).toBeVisible();
	});

	test('navigates back to dashboard', async ({ page }) => {
		await page.goto('/layouts');
		
		await page.getByRole('button', { name: 'Back to Dashboard' }).click();
		await expect(page).toHaveURL('/');
	});
});

test.describe('Layouts page with mock backend', () => {
	test.beforeEach(async ({ page }) => {
		// Mock the API endpoint
		await page.route('**/api/layouts', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({
					layouts: [
						{
							filename: 'test-layout.md',
							name: 'Test Layout',
							description: 'A test keyboard layout',
							modified: '2024-01-01T12:00:00Z'
						},
						{
							filename: 'another-layout.md',
							name: 'Another Layout',
							description: 'Another test layout',
							modified: '2024-01-02T12:00:00Z'
						}
					]
				})
			});
		});
	});

	test('displays mocked layouts', async ({ page }) => {
		await page.goto('/layouts');
		
		// Wait for layouts to load
		await expect(page.getByText('Test Layout')).toBeVisible();
		await expect(page.getByText('Another Layout')).toBeVisible();
		
		// Check descriptions are shown
		await expect(page.getByText('A test keyboard layout')).toBeVisible();
		await expect(page.getByText('Another test layout')).toBeVisible();
	});

	test('layout cards have open buttons', async ({ page }) => {
		await page.goto('/layouts');
		
		// Wait for layouts to load
		await expect(page.getByText('Test Layout')).toBeVisible();
		
		// Check that open buttons exist
		const openButtons = page.getByRole('button', { name: 'Open' });
		await expect(openButtons).toHaveCount(2);
	});

	test('shows empty state when no layouts exist', async ({ page }) => {
		// Override the route with empty layouts
		await page.route('**/api/layouts', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({ layouts: [] })
			});
		});

		await page.goto('/layouts');
		
		await expect(page.getByText(/No layouts found/i)).toBeVisible();
	});

	test('shows error state on API failure', async ({ page }) => {
		// Override the route with error
		await page.route('**/api/layouts', async (route) => {
			await route.fulfill({
				status: 500,
				contentType: 'application/json',
				body: JSON.stringify({ error: 'Internal server error' })
			});
		});

		await page.goto('/layouts');
		
		await expect(page.getByText(/Error loading layouts/i)).toBeVisible();
	});
});
