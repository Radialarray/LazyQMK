import { test, expect } from '@playwright/test';

/**
 * E2E tests for TUI-like keyboard navigation and shortcuts in the keyboard preview.
 * 
 * Tests cover:
 * - Arrow key navigation within keyboard preview
 * - Shift+Arrow for selection extension
 * - Enter to open keycode picker
 * - Escape to close picker or clear selection
 * - [ and ] to cycle layers
 * - Suppression of shortcuts when typing in inputs
 */

test.describe('Keyboard Navigation and Shortcuts', () => {
	// Mock layout data with multiple layers
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
			},
			{
				name: 'Raise',
				color: '#4aff4a',
				keys: [
					{ keycode: 'KC_EXLM', matrix_position: [0, 0], visual_index: 0, led_index: 0 },
					{ keycode: 'KC_AT', matrix_position: [0, 1], visual_index: 1, led_index: 1 },
					{ keycode: 'KC_HASH', matrix_position: [0, 2], visual_index: 2, led_index: 2 },
					{ keycode: 'KC_DLR', matrix_position: [1, 0], visual_index: 3, led_index: 3 },
					{ keycode: 'KC_PERC', matrix_position: [1, 1], visual_index: 4, led_index: 4 },
					{ keycode: 'KC_CIRC', matrix_position: [1, 2], visual_index: 5, led_index: 5 }
				]
			}
		]
	};

	// Mock geometry data with 2x3 grid layout
	const mockGeometry = {
		keyboard: 'crkbd',
		layout: 'LAYOUT_split_3x6_3',
		keys: [
			// Row 0
			{ matrix_row: 0, matrix_col: 0, x: 0, y: 0, width: 1, height: 1, rotation: 0, led_index: 0, visual_index: 0 },
			{ matrix_row: 0, matrix_col: 1, x: 1, y: 0, width: 1, height: 1, rotation: 0, led_index: 1, visual_index: 1 },
			{ matrix_row: 0, matrix_col: 2, x: 2, y: 0, width: 1, height: 1, rotation: 0, led_index: 2, visual_index: 2 },
			// Row 1
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

	// Mock keycodes
	const mockKeycodes = {
		keycodes: [
			{ code: 'KC_A', name: 'A', category: 'basic', description: 'Letter A' },
			{ code: 'KC_B', name: 'B', category: 'basic', description: 'Letter B' },
			{ code: 'KC_C', name: 'C', category: 'basic', description: 'Letter C' }
		],
		total: 3
	};

	// Mock categories
	const mockCategories = {
		categories: [
			{ id: 'basic', name: 'Basic', description: 'Basic keys' }
		]
	};

	test.beforeEach(async ({ page }) => {
		// Mock the layout API endpoint
		await page.route('**/api/layouts/test-layout*', async (route) => {
			const method = route.request().method();
			if (method === 'PUT') {
				await route.fulfill({ status: 204 });
			} else {
				await route.fulfill({
					status: 200,
					contentType: 'application/json',
					body: JSON.stringify(mockLayout)
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

		// Mock the keycodes API endpoint
		await page.route('**/api/keycodes*', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockKeycodes)
			});
		});

		// Mock the categories API endpoint
		await page.route('**/api/keycodes/categories', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockCategories)
			});
		});
	});

	test('Tab into keyboard preview and ArrowRight moves active key', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Focus the keyboard preview container by tabbing or clicking it
		const preview = page.locator('.keyboard-preview');
		await preview.click();

		// Verify initial focus/selection - click first key to select it and close picker
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.keyboard.press('Escape');
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Now press ArrowRight to move to the next key
		await page.keyboard.press('ArrowRight');

		// Verify that key-1 is now active (check for selection class or focus)
		// The active key should be highlighted - we can check the "selected" class or data attribute
		const key1 = page.locator('[data-testid="key-1"]');
		
		// Check if key-1's rect has the "selected" class
		const key1Rect = key1.locator('rect.key-bg.selected');
		await expect(key1Rect).toHaveCount(1);
	});

	test('Shift+ArrowRight extends selection', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Select first key
		await page.locator('[data-testid="key-0"]').click();
		await page.keyboard.press('Escape'); // Close picker if opened

		// Press Shift+ArrowRight to extend selection
		await page.keyboard.press('Shift+ArrowRight');

		// Verify multi-selection indicator appears showing 2 keys selected
		await expect(page.locator('text=2 keys selected')).toBeVisible();

		// Both key-0 and key-1 should have selected class
		const key0Selected = page.locator('[data-testid="key-0"] rect.key-bg.selected');
		const key1Selected = page.locator('[data-testid="key-1"] rect.key-bg.selected');
		
		await expect(key0Selected).toHaveCount(1);
		await expect(key1Selected).toHaveCount(1);
	});

	test('Enter opens keycode picker for active key', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Select first key and close picker
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.keyboard.press('Escape');
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Press Enter to reopen picker
		await page.keyboard.press('Enter');

		// Verify keycode picker opens
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await expect(page.getByText('Select Keycode')).toBeVisible();
	});

	test('Escape closes picker if open', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Open picker by clicking a key
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();

		// Press Escape to close picker
		await page.keyboard.press('Escape');

		// Verify picker is closed
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();
	});

	test('Escape clears selection when picker is not open', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Select a key and close picker
		await page.locator('[data-testid="key-0"]').click();
		await page.keyboard.press('Escape'); // Close picker
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Extend selection with Shift+Click
		await page.locator('[data-testid="key-1"]').click({ modifiers: ['Shift'] });
		await expect(page.locator('text=2 keys selected')).toBeVisible();

		// Press Escape again to clear selection
		await page.keyboard.press('Escape');

		// Verify selection is cleared (no "keys selected" text or only "Selected:" text)
		await expect(page.locator('text=2 keys selected')).not.toBeVisible();
		
		// Check that keys no longer have selected class
		const key0Selected = page.locator('[data-testid="key-0"] rect.key-bg.selected');
		await expect(key0Selected).toHaveCount(0);
	});

	test('[ and ] cycle layers when preview focused', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Verify we start on Base layer (default)
		await expect(page.getByRole('button', { name: 'Base' })).toBeVisible();

		// Focus the keyboard preview
		const preview = page.locator('.keyboard-preview');
		await preview.click();

		// Press ] to cycle to next layer
		await page.keyboard.press(']');

		// Verify we're now on Lower layer
		// The layer button should show active state, or we can check the displayed keycodes
		// Check that key-0 now shows "1" (KC_1 from Lower layer)
		// We can verify by checking the SVG text content
		const key0Text = page.locator('[data-testid="key-0"] text');
		await expect(key0Text).toHaveText('1');

		// Press ] again to cycle to Raise layer
		await page.keyboard.press(']');

		// Verify we're on Raise layer (key-0 should show "EXLM" or "!")
		// Keycodes like KC_EXLM get formatted, check for presence
		const key0TextRaise = page.locator('[data-testid="key-0"] text');
		await expect(key0TextRaise).toHaveText('EXLM');

		// Press [ to cycle back to Lower
		await page.keyboard.press('[');

		// Verify we're back on Lower (key-0 shows "1")
		const key0TextLower = page.locator('[data-testid="key-0"] text');
		await expect(key0TextLower).toHaveText('1');

		// Press [ again to cycle back to Base
		await page.keyboard.press('[');

		// Verify we're back on Base (key-0 shows "Q")
		const key0TextBase = page.locator('[data-testid="key-0"] text');
		await expect(key0TextBase).toHaveText('Q');
	});

	test('Shortcuts do not trigger while text input is focused', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Verify Base layer initially
		let key0Text = page.locator('[data-testid="key-0"] text');
		await expect(key0Text).toHaveText('Q');

		// Open the keycode picker to access its search input
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();

		// Find the search input in the keycode picker
		const searchInput = page.locator('[data-testid="keycode-search-input"]');
		await expect(searchInput).toBeVisible();
		await searchInput.click();

		// Press ] while focused in the input - should NOT cycle layers
		await page.keyboard.press(']');

		// Verify we're still on Base layer (key-0 still shows "Q")
		// Close picker first to check the layer
		await page.keyboard.press('Escape');
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();
		
		key0Text = page.locator('[data-testid="key-0"] text');
		await expect(key0Text).toHaveText('Q');

		// Open picker again and test [ key
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await searchInput.click();

		// Press [ while in input - should NOT cycle layers
		await page.keyboard.press('[');

		// Close picker and verify still on Base layer
		await page.keyboard.press('Escape');
		await expect(key0Text).toHaveText('Q');

		// Test arrow keys don't navigate while typing
		// Open picker and focus search input
		await page.locator('[data-testid="key-0"]').click();
		await searchInput.fill('test');
		
		// Press ArrowRight while in input - should move cursor, not navigate keys
		await page.keyboard.press('ArrowRight');
		
		// Type another character to verify we're still in input
		await page.keyboard.type('x');
		
		// The input should contain our text with x added
		const inputValue = await searchInput.inputValue();
		expect(inputValue).toContain('x');

		// Close picker and verify shortcuts work again
		await page.keyboard.press('Escape');
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();
		
		// Focus back on keyboard preview
		const preview = page.locator('.keyboard-preview');
		await preview.click();

		// Now ] should work to cycle layers
		await page.keyboard.press(']');

		// Verify layer changed (key-0 now shows "1" from Lower layer)
		key0Text = page.locator('[data-testid="key-0"] text');
		await expect(key0Text).toHaveText('1');
	});

	test('Arrow keys navigate in all directions', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Click key-0 (top-left) and close picker
		await page.locator('[data-testid="key-0"]').click();
		await page.keyboard.press('Escape');

		// Press ArrowDown - should move to key-3 (directly below)
		await page.keyboard.press('ArrowDown');
		
		const key3Selected = page.locator('[data-testid="key-3"] rect.key-bg.selected');
		await expect(key3Selected).toHaveCount(1);

		// Press ArrowRight - should move to key-4
		await page.keyboard.press('ArrowRight');
		
		const key4Selected = page.locator('[data-testid="key-4"] rect.key-bg.selected');
		await expect(key4Selected).toHaveCount(1);

		// Press ArrowUp - should move to key-1 (above)
		await page.keyboard.press('ArrowUp');
		
		const key1Selected = page.locator('[data-testid="key-1"] rect.key-bg.selected');
		await expect(key1Selected).toHaveCount(1);

		// Press ArrowLeft - should move back to key-0
		await page.keyboard.press('ArrowLeft');
		
		const key0Selected = page.locator('[data-testid="key-0"] rect.key-bg.selected');
		await expect(key0Selected).toHaveCount(1);
	});

	test('Multi-selection can be extended in multiple directions', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Select key-0
		await page.locator('[data-testid="key-0"]').click();
		await page.keyboard.press('Escape');

		// Extend selection right with Shift+ArrowRight
		await page.keyboard.press('Shift+ArrowRight');
		await expect(page.locator('text=2 keys selected')).toBeVisible();

		// Extend selection down with Shift+ArrowDown (from key-1 to key-4)
		await page.keyboard.press('Shift+ArrowDown');
		await expect(page.locator('text=3 keys selected')).toBeVisible();

		// Verify key-0, key-1, and key-4 are all selected
		await expect(page.locator('[data-testid="key-0"] rect.key-bg.selected')).toHaveCount(1);
		await expect(page.locator('[data-testid="key-1"] rect.key-bg.selected')).toHaveCount(1);
		await expect(page.locator('[data-testid="key-4"] rect.key-bg.selected')).toHaveCount(1);
	});

	test('Enter opens picker even with multiple keys selected (active key editing)', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Select first key
		await page.locator('[data-testid="key-0"]').click();
		await page.keyboard.press('Escape');

		// Extend selection
		await page.keyboard.press('Shift+ArrowRight');
		await expect(page.locator('text=2 keys selected')).toBeVisible();

		// Press Enter - should open picker for the active key (even with multi-selection)
		await page.keyboard.press('Enter');

		// Verify picker opens (allows editing the active key within a multi-selection)
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		
		// Close picker
		await page.keyboard.press('Escape');
	});

	test('Layer cycling wraps around from last to first layer', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Focus keyboard preview
		const preview = page.locator('.keyboard-preview');
		await preview.click();

		// Verify starting on Base layer (key-0 shows "Q")
		let key0Text = page.locator('[data-testid="key-0"] text');
		await expect(key0Text).toHaveText('Q');

		// Press [ to cycle backwards - should wrap to Raise (last layer)
		await page.keyboard.press('[');

		// Verify we're on Raise (last layer) - key-0 shows "EXLM"
		await expect(key0Text).toHaveText('EXLM');

		// Press ] three times to cycle forward through all layers and wrap back to Base
		await page.keyboard.press(']'); // Raise -> Base
		await expect(key0Text).toHaveText('Q');

		await page.keyboard.press(']'); // Base -> Lower
		await expect(key0Text).toHaveText('1');

		await page.keyboard.press(']'); // Lower -> Raise
		await expect(key0Text).toHaveText('EXLM');

		await page.keyboard.press(']'); // Raise -> Base (wrap around)
		await expect(key0Text).toHaveText('Q');
	});

	test('Keyboard navigation works after closing and reopening picker', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Select a key
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();

		// Close picker with Escape
		await page.keyboard.press('Escape');
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Navigate with arrow key
		await page.keyboard.press('ArrowRight');
		
		const key1Selected = page.locator('[data-testid="key-1"] rect.key-bg.selected');
		await expect(key1Selected).toHaveCount(1);

		// Open picker with Enter
		await page.keyboard.press('Enter');
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();

		// Close picker and navigate again
		await page.keyboard.press('Escape');
		await page.keyboard.press('ArrowDown');
		
		const key4Selected = page.locator('[data-testid="key-4"] rect.key-bg.selected');
		await expect(key4Selected).toHaveCount(1);
	});
});
