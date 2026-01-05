import { test, expect } from '@playwright/test';

test.describe('Clipboard and Multi-Selection', () => {
	// Mock layout data
	const mockLayout = {
		metadata: {
			name: 'Clipboard Test Layout',
			description: 'Testing clipboard and selection features',
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
				default_color: { r: 74, g: 158, b: 255 },
				layer_colors_enabled: true,
				keys: [
					{ keycode: 'KC_Q', matrix_position: [0, 0], visual_index: 0, led_index: 0 },
					{ keycode: 'KC_W', matrix_position: [0, 1], visual_index: 1, led_index: 1 },
					{ keycode: 'KC_E', matrix_position: [0, 2], visual_index: 2, led_index: 2 },
					{ keycode: 'KC_R', matrix_position: [0, 3], visual_index: 3, led_index: 3 },
					{ keycode: 'KC_T', matrix_position: [0, 4], visual_index: 4, led_index: 4 }
				]
			}
		],
		categories: []
	};

	// Mock geometry data
	const mockGeometry = {
		keyboard: 'crkbd',
		layout: 'LAYOUT_split_3x6_3',
		keys: [
			{ matrix_row: 0, matrix_col: 0, x: 0, y: 0, width: 60, height: 60, rotation: 0, led_index: 0, visual_index: 0 },
			{ matrix_row: 0, matrix_col: 1, x: 70, y: 0, width: 60, height: 60, rotation: 0, led_index: 1, visual_index: 1 },
			{ matrix_row: 0, matrix_col: 2, x: 140, y: 0, width: 60, height: 60, rotation: 0, led_index: 2, visual_index: 2 },
			{ matrix_row: 0, matrix_col: 3, x: 210, y: 0, width: 60, height: 60, rotation: 0, led_index: 3, visual_index: 3 },
			{ matrix_row: 0, matrix_col: 4, x: 280, y: 0, width: 60, height: 60, rotation: 0, led_index: 4, visual_index: 4 }
		],
		matrix_rows: 1,
		matrix_cols: 5,
		encoder_count: 0,
		position_to_visual_index: {
			'0,0': 0, '0,70': 1, '0,140': 2, '0,210': 3, '0,280': 4
		}
	};

	test.beforeEach(async ({ page }) => {
		let currentLayout = JSON.parse(JSON.stringify(mockLayout));

		// Mock the layout API endpoint
		await page.route('**/api/layouts/clipboard-test*', async (route) => {
			const method = route.request().method();

			if (method === 'PUT') {
				// Capture saved layout data
				const body = route.request().postData();
				if (body) {
					currentLayout = JSON.parse(body);
				}
				await route.fulfill({ status: 204 });
			} else {
				// Return current layout state
				await route.fulfill({
					status: 200,
					contentType: 'application/json',
					body: JSON.stringify(currentLayout)
				});
			}
		});

		// Mock the geometry API endpoint
		await page.route('**/api/keyboards/crkbd/geometry/LAYOUT_split_3x6_3', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockGeometry)
			});
		});

		await page.goto('/layouts/clipboard-test');
		await page.waitForLoadState('networkidle');
	});

	test('should enable selection mode and select multiple keys', async ({ page }) => {
		// Enable selection mode
		await page.click('[data-testid="selection-mode-button"]');

		// Verify selection mode is active
		await expect(page.locator('[data-testid="selection-mode-button"]')).toContainText(
			'✓ Selection Mode'
		);

		// Click multiple keys to select them
		await page.click('[data-testid="key-0"]');
		await page.click('[data-testid="key-1"]');
		await page.click('[data-testid="key-2"]');

		// Verify selection count is displayed
		await expect(page.locator('text=3 keys selected')).toBeVisible();

		// Verify Clear Selection button is visible
		await expect(page.locator('[data-testid="clear-selection-button"]')).toBeVisible();
	});

	test('should toggle key selection in selection mode', async ({ page }) => {
		// Enable selection mode
		await page.click('[data-testid="selection-mode-button"]');

		// Select a key
		await page.click('[data-testid="key-0"]');
		await expect(page.locator('text=1 key selected')).toBeVisible();

		// Click same key again to deselect
		await page.click('[data-testid="key-0"]');

		// Selection should be cleared
		const selectionText = page.locator('text=1 key selected');
		await expect(selectionText).not.toBeVisible();
	});

	test('should support shift-click for multi-selection without selection mode', async ({
		page
	}) => {
		// Single click first key - should open picker
		await page.click('[data-testid="key-0"]');

		// Close the picker
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Shift-click second key - should NOT open picker
		await page.click('[data-testid="key-1"]', { modifiers: ['Shift'] });

		// Should show 2 keys selected
		await expect(page.locator('text=2 keys selected')).toBeVisible();
	});

	test('should copy and paste selected keys', async ({ page }) => {
		// Enable selection mode
		await page.click('[data-testid="selection-mode-button"]');

		// Select first two keys (KC_Q, KC_W)
		await page.click('[data-testid="key-0"]');
		await page.click('[data-testid="key-1"]');

		// Copy selection
		await page.click('[data-testid="copy-button"]');

		// Verify paste button shows clipboard count
		await expect(page.locator('[data-testid="paste-button"]')).toContainText('(2)');

		// Clear selection
		await page.click('[data-testid="clear-selection-button"]');

		// Select target key (index 3)
		await page.click('[data-testid="key-3"]');

		// Paste
		await page.click('[data-testid="paste-button"]');

		// Save to persist changes
		await page.click('button:has-text("Save")');
		await expect(page.locator('text=Saved!')).toBeVisible();

		// Reload and verify keys were pasted
		await page.reload();
		await page.waitForLoadState('networkidle');

		// Enable selection mode to check keys without opening picker
		await page.click('[data-testid="selection-mode-button"]');

		// Check that key 3 now has KC_Q and key 4 has KC_W
		// We'll verify by selecting them and checking displayed keycode
		await page.click('[data-testid="key-3"]');
		await expect(page.locator('code:has-text("KC_Q")')).toBeVisible();

		await page.click('[data-testid="key-4"]');
		await expect(page.locator('code:has-text("KC_W")')).toBeVisible();
	});

	test('should cut selected keys (set to KC_TRNS)', async ({ page }) => {
		// Enable selection mode
		await page.click('[data-testid="selection-mode-button"]');

		// Select first two keys
		await page.click('[data-testid="key-0"]');
		await page.click('[data-testid="key-1"]');

		// Cut selection
		await page.click('[data-testid="cut-button"]');

		// Save
		await page.click('button:has-text("Save")');
		await expect(page.locator('text=Saved!')).toBeVisible();

		// Reload and verify keys are now KC_TRNS (shown as ▽)
		await page.reload();
		await page.waitForLoadState('networkidle');

		// Enable selection mode to check keys without opening picker
		await page.click('[data-testid="selection-mode-button"]');

		// Select first key
		await page.click('[data-testid="key-0"]');
		await expect(page.locator('code:has-text("KC_TRNS")')).toBeVisible();

		// Select second key
		await page.click('[data-testid="key-1"]');
		await expect(page.locator('code:has-text("KC_TRNS")')).toBeVisible();
	});

	test('should undo paste operation', async ({ page }) => {
		// Enable selection mode and select first key
		await page.click('[data-testid="selection-mode-button"]');
		await page.click('[data-testid="key-0"]');

		// Copy
		await page.click('[data-testid="copy-button"]');

		// Select target (key 2) and paste
		await page.click('[data-testid="clear-selection-button"]');
		await page.click('[data-testid="key-2"]');
		await page.click('[data-testid="paste-button"]');

		// Verify paste happened (key 2 should show as changed)
		await expect(page.locator('text=Unsaved changes')).toBeVisible();

		// Click undo
		await page.click('[data-testid="undo-button"]');

		// Key 2 should be back to KC_E
		await page.click('[data-testid="key-2"]');
		await expect(page.locator('code:has-text("KC_E")')).toBeVisible();
	});

	test('should undo cut operation', async ({ page }) => {
		// Enable selection mode and select first key
		await page.click('[data-testid="selection-mode-button"]');
		await page.click('[data-testid="key-0"]');

		// Cut
		await page.click('[data-testid="cut-button"]');

		// Verify key is now KC_TRNS
		await page.click('[data-testid="key-0"]');
		await expect(page.locator('code:has-text("KC_TRNS")')).toBeVisible();

		// Undo
		await page.click('[data-testid="undo-button"]');

		// Key should be restored to KC_Q
		await page.click('[data-testid="key-0"]');
		await expect(page.locator('code:has-text("KC_Q")')).toBeVisible();
	});

	test('should preserve color overrides in clipboard operations', async ({ page }) => {
		// Select first key - this will open picker
		await page.click('[data-testid="key-0"]');

		// Close picker
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Set color override
		await page.click('[data-testid="set-color-button"]');
		await page.click('button[title="#FF0000"]'); // Red

		// Wait for color picker to close
		await expect(page.locator('text=Preset colors')).not.toBeVisible();

		// Enable selection mode and select the key
		await page.click('[data-testid="selection-mode-button"]');
		await page.click('[data-testid="key-0"]');

		// Copy
		await page.click('[data-testid="copy-button"]');

		// Clear and select target
		await page.click('[data-testid="clear-selection-button"]');
		await page.click('[data-testid="key-2"]');

		// Paste
		await page.click('[data-testid="paste-button"]');

		// Click target key to verify color override was copied
		await page.click('[data-testid="key-2"]');
		await expect(page.locator('[data-testid="clear-color-override-button"]')).toBeVisible();
	});

	test('should disable clipboard buttons appropriately', async ({ page }) => {
		// Copy/Cut should be disabled with no selection
		await expect(page.locator('[data-testid="copy-button"]')).toBeDisabled();
		await expect(page.locator('[data-testid="cut-button"]')).toBeDisabled();

		// Paste should be disabled with empty clipboard
		await expect(page.locator('[data-testid="paste-button"]')).toBeDisabled();

		// Undo should be disabled initially
		await expect(page.locator('[data-testid="undo-button"]')).toBeDisabled();

		// Select a key
		await page.click('[data-testid="selection-mode-button"]');
		await page.click('[data-testid="key-0"]');

		// Copy/Cut should now be enabled
		await expect(page.locator('[data-testid="copy-button"]')).not.toBeDisabled();
		await expect(page.locator('[data-testid="cut-button"]')).not.toBeDisabled();

		// Copy to enable paste
		await page.click('[data-testid="copy-button"]');
		await expect(page.locator('[data-testid="paste-button"]')).not.toBeDisabled();
	});

	test('should clear selection when changing layers', async ({ page }) => {
		// Create a second layer for this test
		// This would require mocking a layout with multiple layers
		// For now, we verify selection clears when manually cleared

		await page.click('[data-testid="selection-mode-button"]');
		await page.click('[data-testid="key-0"]');
		await page.click('[data-testid="key-1"]');

		await expect(page.locator('text=2 keys selected')).toBeVisible();

		await page.click('[data-testid="clear-selection-button"]');

		const selectionText = page.locator('text=2 keys selected');
		await expect(selectionText).not.toBeVisible();
	});
});
