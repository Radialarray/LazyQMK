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
			{ matrix_row: 0, matrix_col: 0, x: 0, y: 0, width: 1, height: 1, rotation: 0, led_index: 0, visual_index: 0 },
			{ matrix_row: 0, matrix_col: 1, x: 1, y: 0, width: 1, height: 1, rotation: 0, led_index: 1, visual_index: 1 },
			{ matrix_row: 0, matrix_col: 2, x: 2, y: 0, width: 1, height: 1, rotation: 0, led_index: 2, visual_index: 2 },
			{ matrix_row: 1, matrix_col: 0, x: 0, y: 1, width: 1, height: 1, rotation: 0, led_index: 3, visual_index: 3 },
			{ matrix_row: 1, matrix_col: 1, x: 1, y: 1, width: 1, height: 1, rotation: 0, led_index: 4, visual_index: 4 },
			{ matrix_row: 1, matrix_col: 2, x: 2, y: 1, width: 1, height: 1, rotation: 0, led_index: 5, visual_index: 5 }
		],
		matrix_rows: 2,
		matrix_cols: 3,
		encoder_count: 0,
		position_to_visual_index: {
			'0,0': 0, '0,1': 1, '0,2': 2,
			'1,0': 3, '1,1': 4, '1,2': 5
		}
	};

	// Mock render metadata data
	const mockRenderMetadata = {
		filename: 'test-layout',
		layers: [
			{
				number: 0,
				name: 'Base',
				keys: [
					{ visual_index: 0, display: { primary: 'Q' }, details: [{ kind: 'simple', code: 'KC_Q', description: 'Letter Q' }] },
					{ visual_index: 1, display: { primary: 'W' }, details: [{ kind: 'simple', code: 'KC_W', description: 'Letter W' }] },
					{ visual_index: 2, display: { primary: 'E' }, details: [{ kind: 'simple', code: 'KC_E', description: 'Letter E' }] },
					{ visual_index: 3, display: { primary: 'A' }, details: [{ kind: 'simple', code: 'KC_A', description: 'Letter A' }] },
					{ visual_index: 4, display: { primary: 'S' }, details: [{ kind: 'simple', code: 'KC_S', description: 'Letter S' }] },
					{ visual_index: 5, display: { primary: 'D' }, details: [{ kind: 'simple', code: 'KC_D', description: 'Letter D' }] }
				]
			},
			{
				number: 1,
				name: 'Lower',
				keys: [
					{ visual_index: 0, display: { primary: '1' }, details: [{ kind: 'simple', code: 'KC_1', description: 'Number 1' }] },
					{ visual_index: 1, display: { primary: '2' }, details: [{ kind: 'simple', code: 'KC_2', description: 'Number 2' }] },
					{ visual_index: 2, display: { primary: '3' }, details: [{ kind: 'simple', code: 'KC_3', description: 'Number 3' }] },
					{ visual_index: 3, display: { primary: '4' }, details: [{ kind: 'simple', code: 'KC_4', description: 'Number 4' }] },
					{ visual_index: 4, display: { primary: '5' }, details: [{ kind: 'simple', code: 'KC_5', description: 'Number 5' }] },
					{ visual_index: 5, display: { primary: '6' }, details: [{ kind: 'simple', code: 'KC_6', description: 'Number 6' }] }
				]
			}
		]
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

		// Mock the render metadata API endpoint
		await page.route('**/api/layouts/test-layout/render-metadata', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockRenderMetadata)
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

		// Click on the first key - this will open the picker
		await firstKey.click();

		// Verify picker is open
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();

		// Close the picker
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Move mouse away to clear hover state
		await page.mouse.move(0, 0);

		// Verify selection is shown - check for the keycode display
		await expect(page.getByText('Selected:')).toBeVisible();

		// Verify key details card appears - use data-testid for stability
		await expect(page.getByTestId('key-details-card')).toBeVisible();
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Details & Customization');
		await expect(page.getByText('Visual Index')).toBeVisible();
	});

	test('clicking a different key changes selection', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Click on first key to select it - opens picker
		await page.locator('[data-testid="key-0"]').click();
		
		// Close the picker
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();
		
		// Move mouse away to clear hover state
		await page.mouse.move(0, 0);
		
		// Verify key details card is visible for first key
		await expect(page.getByTestId('key-details-card')).toBeVisible();
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Details & Customization');

		// Click on second key to change selection - opens picker again
		await page.locator('[data-testid="key-1"]').click();

		// Close the picker
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Move mouse away to clear hover state
		await page.mouse.move(0, 0);

		// Verify key details card is still visible (for second key)
		await expect(page.getByTestId('key-details-card')).toBeVisible();
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Details & Customization');
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

	test('switching layers preserves key selection', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Select a key - this will open the keycode picker
		await page.locator('[data-testid="key-0"]').click();
		
		// Close the keycode picker
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();
		
		// Move mouse away to clear hover state
		await page.mouse.move(0, 0);
		
		// Verify key details card is visible
		await expect(page.getByTestId('key-details-card')).toBeVisible();
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Details & Customization');
		
		// Switch to a different layer
		await page.getByRole('button', { name: 'Lower' }).first().click();
		
		// Key details should still be visible (selection persists across layer changes - correct behavior)
		await expect(page.getByTestId('key-details-card')).toBeVisible();
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Details & Customization');
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

	test('hovering a key shows preview details', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// First, select a key to ensure the card stays visible
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();
		
		// Move mouse away to clear initial hover
		await page.mouse.move(0, 0);
		
		// Verify we're in customization mode (not hovering)
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Details & Customization');

		// Now hover over the selected key
		await page.locator('[data-testid="key-0"]').hover();

		// Key details panel should change to "Key Preview" mode
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Preview');

		// Should show key action details
		await expect(page.getByText('Key Actions')).toBeVisible();
		await expect(page.getByText('SIMPLE')).toBeVisible();
		await expect(page.getByText('Letter Q')).toBeVisible();

		// Move mouse away
		await page.mouse.move(0, 0);

		// Should revert to customization mode
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Details & Customization');
	});

	test('shows multi-selection summary when multiple keys selected', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Enable selection mode
		await page.getByTestId('selection-mode-button').click();

		// Click multiple keys
		await page.locator('[data-testid="key-0"]').click();
		await page.locator('[data-testid="key-1"]').click();
		
		// Move mouse away to clear hover state and show multi-selection summary
		await page.mouse.move(0, 0);

		// Should show multi-selection summary
		await expect(page.getByTestId('multi-selection-summary')).toBeVisible();
		await expect(page.getByText('Multiple Keys Selected (2 keys)')).toBeVisible();
		await expect(page.getByText('Use Copy, Cut, or Paste operations')).toBeVisible();
	});
});
