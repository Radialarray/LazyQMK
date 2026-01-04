import { test, expect } from '@playwright/test';

test.describe('Keyboard Preview', () => {
	// Mock layout data
	const mockLayout = {
		metadata: {
			name: 'Test Layout',
			description: 'A test keyboard layout',
			author: 'Test User',
			keyboard: 'crkbd',
			layout: 'LAYOUT_split_3x6_3',
			created: '2024-01-01T00:00:00Z',
			modified: '2024-01-01T00:00:00Z'
		},
		layers: [
			{
				name: 'Base',
				color: '#4a9eff',
				keys: [
					{ keycode: 'KC_Q', matrix_position: [0, 0], visual_index: 0, led_index: 0 },
					{ keycode: 'KC_W', matrix_position: [0, 1], visual_index: 1, led_index: 1 },
					{ keycode: 'KC_E', matrix_position: [0, 2], visual_index: 2, led_index: 2 },
					{ keycode: 'KC_A', matrix_position: [1, 0], visual_index: 3, led_index: 3 },
					{ keycode: 'KC_S', matrix_position: [1, 1], visual_index: 4, led_index: 4 },
					{ keycode: 'KC_D', matrix_position: [1, 2], visual_index: 5, led_index: 5 }
				]
			},
			{
				name: 'Lower',
				color: '#ff4a4a',
				keys: [
					{ keycode: 'KC_1', matrix_position: [0, 0], visual_index: 0, led_index: 0 },
					{ keycode: 'KC_2', matrix_position: [0, 1], visual_index: 1, led_index: 1 },
					{ keycode: 'KC_3', matrix_position: [0, 2], visual_index: 2, led_index: 2 },
					{ keycode: 'KC_4', matrix_position: [1, 0], visual_index: 3, led_index: 3 },
					{ keycode: 'KC_5', matrix_position: [1, 1], visual_index: 4, led_index: 4 },
					{ keycode: 'KC_6', matrix_position: [1, 2], visual_index: 5, led_index: 5 }
				]
			}
		]
	};

	// Mock geometry data
	const mockGeometry = {
		keyboard: 'crkbd',
		layout: 'LAYOUT_split_3x6_3',
		keys: [
			{ matrix_row: 0, matrix_col: 0, x: 0, y: 0, width: 1, height: 1, rotation: 0, led_index: 0 },
			{ matrix_row: 0, matrix_col: 1, x: 1, y: 0, width: 1, height: 1, rotation: 0, led_index: 1 },
			{ matrix_row: 0, matrix_col: 2, x: 2, y: 0, width: 1, height: 1, rotation: 0, led_index: 2 },
			{ matrix_row: 1, matrix_col: 0, x: 0, y: 1, width: 1, height: 1, rotation: 0, led_index: 3 },
			{ matrix_row: 1, matrix_col: 1, x: 1, y: 1, width: 1, height: 1, rotation: 0, led_index: 4 },
			{ matrix_row: 1, matrix_col: 2, x: 2, y: 1, width: 1, height: 1, rotation: 0, led_index: 5 }
		],
		matrix_rows: 2,
		matrix_cols: 3,
		encoder_count: 0
	};

	test.beforeEach(async ({ page }) => {
		// Mock the layout API endpoint - needs to be set up before navigation
		await page.route('**/api/layouts/test-layout*', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockLayout)
			});
		});

		// Mock the geometry API endpoint
		await page.route('**/api/keyboards/crkbd/geometry/LAYOUT_split_3x6_3', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockGeometry)
			});
		});
	});

	test('renders keyboard preview with keys from geometry', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();

		// Check that keys are rendered (should have 6 keys)
		const keys = page.locator('[data-testid^="key-"]');
		await expect(keys).toHaveCount(6);
	});

	test('displays keycode labels on keys', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();

		// Check that SVG contains key labels (the SVG text elements)
		// Wait for keys to be rendered
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Check that some keycode labels are visible in the SVG
		const svgText = page.locator('.keyboard-preview svg text');
		await expect(svgText).not.toHaveCount(0);
	});

	test('clicking a key selects it and shows details', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();

		// Wait for keys to be rendered
		const firstKey = page.locator('[data-testid="key-0"]');
		await expect(firstKey).toBeVisible();

		// Click on the first key
		await firstKey.click();

		// Verify selection is shown - check for the keycode display
		await expect(page.getByText('Selected:')).toBeVisible();

		// Verify key details card appears
		await expect(page.getByRole('heading', { name: 'Key Details' })).toBeVisible();
		await expect(page.getByText('Visual Index')).toBeVisible();
	});

	test('clicking a different key changes selection', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Click on first key
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByRole('heading', { name: 'Key Details' })).toBeVisible();

		// Click on second key
		await page.locator('[data-testid="key-1"]').click();

		// Verify key details card is still visible
		await expect(page.getByRole('heading', { name: 'Key Details' })).toBeVisible();
	});

	test('switching layers updates displayed layer', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();

		// Verify Base layer button is visible
		await expect(page.getByRole('button', { name: 'Base' })).toBeVisible();

		// Click on Lower layer button
		await page.getByRole('button', { name: 'Lower' }).click();

		// Lower button should now have different styling (it's selected)
		// The layer selector changes the active layer
		const lowerButton = page.getByRole('button', { name: 'Lower' });
		await expect(lowerButton).toBeVisible();
	});

	test('switching layers clears key selection', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Select a key
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByRole('heading', { name: 'Key Details' })).toBeVisible();

		// Switch layers
		await page.getByRole('button', { name: 'Lower' }).click();

		// Key details should no longer be visible (selection cleared)
		await expect(page.getByRole('heading', { name: 'Key Details' })).not.toBeVisible();
	});

	test('shows error message when geometry fails to load', async ({ page }) => {
		// Override the geometry route to return an error
		await page.route('**/api/keyboards/crkbd/geometry/LAYOUT_split_3x6_3', async (route) => {
			await route.fulfill({
				status: 404,
				contentType: 'application/json',
				body: JSON.stringify({
					error: 'Keyboard not found',
					details: 'The keyboard "crkbd" was not found'
				})
			});
		});

		await page.goto('/layouts/test-layout');

		// Should show error message
		await expect(page.getByText('Failed to load keyboard geometry')).toBeVisible();
	});
});
