import { test, expect } from '@playwright/test';

test.describe('Dashboard smoke tests', () => {
	test('loads the dashboard page', async ({ page }) => {
		await page.goto('/');
		
		// Check that the main heading is visible
		await expect(page.getByRole('heading', { name: 'LazyQMK Dashboard' })).toBeVisible();
		
		// Check that the backend status card is present
		await expect(page.getByText('Backend Status')).toBeVisible();
		
		// Check that navigation cards are present
		await expect(page.getByText('Layouts')).toBeVisible();
		await expect(page.getByText('Keycodes')).toBeVisible();
		await expect(page.getByText('Settings')).toBeVisible();
	});

	test('navigates to layouts page', async ({ page }) => {
		await page.goto('/');
		
		// Click the "View Layouts" button
		await page.getByRole('button', { name: 'View Layouts' }).click();
		
		// Should navigate to /layouts
		await expect(page).toHaveURL('/layouts');
		await expect(page.getByRole('heading', { name: 'Layouts' })).toBeVisible();
	});

	test('navigates to keycodes page', async ({ page }) => {
		await page.goto('/');
		
		// Click the "Browse Keycodes" button
		await page.getByRole('button', { name: 'Browse Keycodes' }).click();
		
		// Should navigate to /keycodes
		await expect(page).toHaveURL('/keycodes');
		await expect(page.getByRole('heading', { name: 'Keycodes Browser' })).toBeVisible();
	});

	test('navigates to settings page', async ({ page }) => {
		await page.goto('/');
		
		// Click the "Open Settings" button
		await page.getByRole('button', { name: 'Open Settings' }).click();
		
		// Should navigate to /settings
		await expect(page).toHaveURL('/settings');
		await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();
	});
});
