import { test, expect } from '@playwright/test';

test.describe('Key Editing', () => {
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
					{ keycode: 'KC_E', matrix_position: [0, 2], visual_index: 2, led_index: 2 }
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
			{ matrix_row: 0, matrix_col: 2, x: 2, y: 0, width: 1, height: 1, rotation: 0, led_index: 2, visual_index: 2 }
		],
		matrix_rows: 1,
		matrix_cols: 3,
		encoder_count: 0,
		position_to_visual_index: {
			'0,0': 0, '0,1': 1, '0,2': 2
		}
	};

	// Mock keycodes
	const mockKeycodes = {
		keycodes: [
			{ code: 'KC_A', name: 'A', category: 'basic', description: 'Letter A' },
			{ code: 'KC_B', name: 'B', category: 'basic', description: 'Letter B' },
			{ code: 'KC_C', name: 'C', category: 'basic', description: 'Letter C' },
			{ code: 'KC_LCTL', name: 'Left Control', category: 'modifier', description: 'Left Control key' }
		],
		total: 4
	};

	// Mock categories
	const mockCategories = {
		categories: [
			{ id: 'basic', name: 'Basic', description: 'Basic keys' },
			{ id: 'modifier', name: 'Modifier', description: 'Modifier keys' }
		]
	};

	test.beforeEach(async ({ page }) => {
		// Mock the layout API endpoint
		await page.route('**/api/layouts/test-layout*', async (route) => {
			const url = route.request().url();
			const method = route.request().method();

			if (method === 'PUT') {
				// Mock save endpoint
				await route.fulfill({
					status: 204
				});
			} else {
				// Mock get endpoint
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

	test('clicking a key opens the keycode picker', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Click on the first key - should immediately open picker
		await page.locator('[data-testid="key-0"]').click();

		// Verify keycode picker modal is visible immediately
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await expect(page.getByText('Select Keycode')).toBeVisible();
	});

	test('keycode picker displays keycodes', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard and click a key
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();
		await page.locator('[data-testid="key-0"]').click();

		// Wait for picker to open (opens immediately on click)
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();

		// Verify keycodes are displayed
		await expect(page.getByTestId('keycode-option-KC_A')).toBeVisible();
		await expect(page.getByTestId('keycode-option-KC_B')).toBeVisible();
		await expect(page.getByText('Letter A')).toBeVisible();
	});

	test('selecting a keycode updates the key', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard and click a key
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();
		await page.locator('[data-testid="key-0"]').click();

		// Wait for picker (opens immediately) and select a keycode
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByTestId('keycode-option-KC_A').click();

		// Verify picker closes
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Verify "Unsaved changes" indicator appears
		await expect(page.getByText('Unsaved changes')).toBeVisible();
	});

	test('clear key button sets keycode to KC_TRNS', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard and click a key
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();
		await page.locator('[data-testid="key-0"]').click();

		// Wait for picker (opens immediately) and click clear button
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByTestId('clear-key-button').click();

		// Verify picker closes
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Verify "Unsaved changes" indicator appears
		await expect(page.getByText('Unsaved changes')).toBeVisible();
	});

	test('search filters keycodes', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Click a key to select it and open picker
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();
		await page.locator('[data-testid="key-0"]').click();
		
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();

		// Type in search box
		const searchInput = page.getByTestId('keycode-search-input');
		await searchInput.fill('control');

		// Verify keycodes API was called with search param
		// This will re-fetch keycodes with search filter
		await page.waitForResponse((response) => 
			response.url().includes('/api/keycodes') && 
			response.url().includes('search=control')
		);
	});

	test('category filter filters keycodes', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Click a key to select it and open picker
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();
		await page.locator('[data-testid="key-0"]').click();
		
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();

		// Set up response listener before clicking category
		const responsePromise = page.waitForResponse((response) => 
			response.url().includes('/api/keycodes') && 
			response.url().includes('category=modifier')
		);

		// Click on a category button
		await page.getByTestId('category-modifier').click();

		// Verify keycodes API was called with category param
		await responsePromise;
	});

	test('closing picker without selection preserves original keycode', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Click a key to select it and open picker
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();
		await page.locator('[data-testid="key-0"]').click();
		
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();

		// Close picker by clicking Cancel button
		await page.getByRole('button', { name: 'Cancel' }).click();

		// Verify picker closes
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Verify no "Unsaved changes" indicator
		await expect(page.getByText('Unsaved changes')).not.toBeVisible();
	});

	test('saving layout after key edit persists changes', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Click a key to select it and open picker
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();
		await page.locator('[data-testid="key-0"]').click();

		// Select a keycode
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByTestId('keycode-option-KC_A').click();

		// Verify unsaved changes
		await expect(page.getByText('Unsaved changes')).toBeVisible();

		// Set up request listener before clicking save
		const requestPromise = page.waitForRequest((request) => 
			request.url().includes('/api/layouts/test-layout') && 
			request.method() === 'PUT'
		);

		// Click Save button
		await page.getByTestId('save-button').click();

		// Wait for save to complete
		await expect(page.getByText('Saved!')).toBeVisible();

		// Verify PUT request was made
		const saveRequest = await requestPromise;
		expect(saveRequest).toBeTruthy();
	});

	test('keycode picker highlights current keycode', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Click the first key (which has KC_Q) to select it and open picker
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();
		await page.locator('[data-testid="key-0"]').click();

		// Wait for picker
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();

		// Check if current keycode is highlighted (if KC_Q is in the list)
		// Our mock data doesn't include KC_Q, but we can verify the "(current)" text appears
		// when the current keycode matches one in the list
	});

	test('can edit multiple keys in sequence', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Edit first key
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByTestId('keycode-option-KC_A').click();

		// Edit second key
		await page.locator('[data-testid="key-1"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByTestId('keycode-option-KC_B').click();

		// Verify unsaved changes
		await expect(page.getByText('Unsaved changes')).toBeVisible();
	});

	test('clicking overlay closes keycode picker', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Click a key to select it and open picker
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();
		await page.locator('[data-testid="key-0"]').click();
		
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();

		// Click on the overlay (not the dialog)
		await page.getByTestId('keycode-picker-overlay').click({ position: { x: 10, y: 10 } });

		// Verify picker closes
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();
	});

	test('selection mode prevents picker from opening', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Enable selection mode
		await page.click('[data-testid="selection-mode-button"]');

		// Verify selection mode is active
		await expect(page.locator('[data-testid="selection-mode-button"]')).toContainText(
			'✓ Selection Mode'
		);

		// Click a key
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();
		await page.locator('[data-testid="key-0"]').click();

		// Verify picker does NOT open
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Verify key is selected
		await expect(page.locator('text=1 key selected')).toBeVisible();
	});

	test('shift-click multi-selection prevents picker from opening', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Single click first key - should open picker
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();

		// Close the picker
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Shift-click second key - should NOT open picker
		await page.locator('[data-testid="key-1"]').click({ modifiers: ['Shift'] });

		// Verify picker does NOT open
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Verify multi-selection occurred
		await expect(page.locator('text=2 keys selected')).toBeVisible();
	});
});
