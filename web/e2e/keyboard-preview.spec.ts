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
					{ keycode: 'LT(1, KC_ESC)', matrix_position: [1, 0], visual_index: 3, led_index: 3 },
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
					{ visual_index: 3, display: { primary: 'ESC', secondary: 'L1' }, details: [{ kind: 'tap', code: 'KC_ESC', description: 'Tap: Escape' }, { kind: 'hold', code: 'Layer 1', description: 'Hold: Activate layer 1' }] },
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

		// Check that keys are rendered (should have 6 keys) - use more specific selector
		const keys = page.locator('.keyboard-preview [data-testid^="key-"]');
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
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');
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
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');

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
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');
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
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');
		
		// Switch to a different layer
		await page.getByRole('button', { name: 'Lower' }).first().click();
		
		// Key details should still be visible (selection persists across layer changes - correct behavior)
		await expect(page.getByTestId('key-details-card')).toBeVisible();
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');
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
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');

		// Now hover over the selected key
		await page.locator('[data-testid="key-0"]').hover();

		// Key details panel should still show "Key Metadata" (consistent heading)
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');

		// Should show key action details
		await expect(page.getByText('Key Actions')).toBeVisible();
		await expect(page.getByText('SIMPLE')).toBeVisible();
		await expect(page.getByText('Letter Q')).toBeVisible();

		// Move mouse away

		// Should still show Key Metadata
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');
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

	test('displays multi-action key with secondary label', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-3"]')).toBeVisible();

		// Key 3 should be LT(1, KC_ESC) with primary "ESC" and secondary "L1"
		// SVG text elements should include both labels
		const keyGroup = page.locator('[data-testid="key-3"]');
		const textElements = keyGroup.locator('text');
		
		// Should have 2 text elements (primary and secondary)
		await expect(textElements).toHaveCount(2);
		
		// Check labels are present
		await expect(keyGroup.locator('text', { hasText: 'ESC' })).toBeVisible();
		await expect(keyGroup.locator('text', { hasText: 'L1' })).toBeVisible();
	});

	test('hover panel shows full key action descriptions', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-3"]')).toBeVisible();

		// First select a simple key to ensure the card stays visible
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();
		
		// Move mouse away to clear initial hover
		await page.mouse.move(0, 0);

		// Now hover over the multi-action key (LT(1, KC_ESC))
		await page.locator('[data-testid="key-3"]').hover();

		// Key details panel should still show "Key Metadata"
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');

		// Should show key action details with both tap and hold
		const keyActionsSection = page.locator('.border-t.border-border.pt-4.mb-4');
		await expect(keyActionsSection.getByRole('heading', { name: 'Key Actions' })).toBeVisible();
		
		// Check for action badges and descriptions (more specific selectors)
		const actionBadges = keyActionsSection.locator('span.uppercase');
		await expect(actionBadges.filter({ hasText: 'TAP' })).toBeVisible();
		await expect(actionBadges.filter({ hasText: 'HOLD' })).toBeVisible();
		
		// Check descriptions are present
		await expect(keyActionsSection.getByText('Tap: Escape')).toBeVisible();
		await expect(keyActionsSection.getByText('Hold: Activate layer 1')).toBeVisible();
	});

	test('labels are clipped and do not overflow key bounds', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Check that SVG has clip-path definitions
		const svg = page.locator('.keyboard-preview svg');
		const clipPaths = svg.locator('clipPath');
		
		// Should have clip paths for keys (at least 6 for our mock data)
		const count = await clipPaths.count();
		expect(count).toBeGreaterThanOrEqual(6);

		// Verify label groups use clip-path
		const labelGroups = svg.locator('g[clip-path]');
		const labelGroupCount = await labelGroups.count();
		expect(labelGroupCount).toBeGreaterThanOrEqual(6);
	});

	test('hover panel shows even when key data is missing from layer', async ({ page }) => {
		// Create a mock with mismatched keys (geometry has keys but layer is missing one)
		const mockLayoutMissing = {
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
						// Note: visual_index 0 is missing to simulate a data gap
						{ keycode: 'KC_W', matrix_position: [0, 1], visual_index: 1, led_index: 1 },
						{ keycode: 'KC_E', matrix_position: [0, 2], visual_index: 2, led_index: 2 },
						{ keycode: 'KC_S', matrix_position: [1, 0], visual_index: 3, led_index: 3 },
						{ keycode: 'KC_D', matrix_position: [1, 1], visual_index: 4, led_index: 4 },
						{ keycode: 'KC_F', matrix_position: [1, 2], visual_index: 5, led_index: 5 }
					]
				}
			]
		};

		const mockRenderMetadataMissing = {
			filename: 'test-layout-missing',
			layers: [
				{
					number: 0,
					name: 'Base',
					keys: [
						// Note: visual_index 0 is missing
						{ visual_index: 1, display: { primary: 'W' }, details: [{ kind: 'simple', code: 'KC_W', description: 'Letter W' }] },
						{ visual_index: 2, display: { primary: 'E' }, details: [{ kind: 'simple', code: 'KC_E', description: 'Letter E' }] },
						{ visual_index: 3, display: { primary: 'S' }, details: [{ kind: 'simple', code: 'KC_S', description: 'Letter S' }] },
						{ visual_index: 4, display: { primary: 'D' }, details: [{ kind: 'simple', code: 'KC_D', description: 'Letter D' }] },
						{ visual_index: 5, display: { primary: 'F' }, details: [{ kind: 'simple', code: 'KC_F', description: 'Letter F' }] }
					]
				}
			]
		};

		// Override routes for this test
		await page.route('**/api/layouts/test-layout-missing*', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockLayoutMissing)
			});
		});

		await page.route('**/api/layouts/test-layout-missing/render-metadata', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockRenderMetadataMissing)
			});
		});

		await page.goto('/layouts/test-layout-missing');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// First select a key that exists to have the panel visible
		await page.locator('[data-testid="key-1"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();
		
		// Move mouse away initially
		await page.mouse.move(0, 0);
		
		// Verify key details card is visible
		await expect(page.getByTestId('key-details-card')).toBeVisible();

		// Now hover over key 0 which has no data in layer
		await page.locator('[data-testid="key-0"]').hover();

		// The hover panel should still be visible (even with fallback)
		await expect(page.getByTestId('key-details-card')).toBeVisible();
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');
		
		// Should show the fallback message for missing key data
		await expect(page.getByTestId('key-hover-fallback')).toBeVisible();
		await expect(page.getByText('Hovering key index:')).toBeVisible();
		await expect(page.getByText('Key data not available')).toBeVisible();
	});

	test('details panel has fixed height to prevent scrollbar jumping', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// The key details card should always be visible with fixed min-height
		await expect(page.getByTestId('key-details-card')).toBeVisible();

		// Select a key to show content in the details panel
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();
		
		// Move mouse away to clear hover state and show customization mode
		await page.mouse.move(0, 0);
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');
		
		// Get card height in customization mode
		const cardLocator = page.getByTestId('key-details-card');
		const customizationCardBox = await cardLocator.boundingBox();
		expect(customizationCardBox).not.toBeNull();
		const customizationCardHeight = customizationCardBox!.height;

		// Hover over the same key to switch to preview mode
		await page.locator('[data-testid="key-0"]').hover();
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');
		
		// Get card height in preview mode
		const previewCardBox = await cardLocator.boundingBox();
		expect(previewCardBox).not.toBeNull();
		const previewCardHeight = previewCardBox!.height;

		// Move mouse to a different key (still in preview mode)
		await page.locator('[data-testid="key-1"]').hover();
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');
		
		// Get card height with different key
		const differentKeyPreviewBox = await cardLocator.boundingBox();
		expect(differentKeyPreviewBox).not.toBeNull();
		const differentKeyPreviewHeight = differentKeyPreviewBox!.height;

		// Card height should remain stable across content changes (may expand but shouldn't collapse)
		// The key insight: the card should be at least min-height, and stable across mode switches
		expect(customizationCardHeight).toBeGreaterThanOrEqual(400);
		expect(previewCardHeight).toBeGreaterThanOrEqual(400);
		expect(differentKeyPreviewHeight).toBeGreaterThanOrEqual(400);
		
		// More importantly: height should be stable when switching between preview states
		expect(previewCardHeight).toBe(differentKeyPreviewHeight);
	});

	test('render metadata visual_index correctly maps to geometry keys (regression test for LazyQMK-eon)', async ({ page }) => {
		// This test verifies that the render metadata's visual_index values
		// correctly match the geometry's visual_index values, allowing the
		// frontend to look up key details by hovering/selecting keys.
		//
		// BUG: Previously, the backend used array enumeration index as visual_index
		// instead of computing it from the key's position using the geometry mapping.
		// This caused "Key data not available for this position" for every key.

		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Select a key to ensure the details card is visible
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Move mouse away first to clear hover
		await page.mouse.move(0, 0);
		await expect(page.getByTestId('key-details-card')).toBeVisible();

		// Hover over key 0 - should show key details, NOT "Key data not available"
		await page.locator('[data-testid="key-0"]').hover();
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');

		// The key should have proper details - check for "Key Actions" section
		// If visual_index mapping is broken, we'd see "Key data not available" instead
		await expect(page.getByText('Key Actions')).toBeVisible();
		await expect(page.getByText('Letter Q')).toBeVisible();

		// The fallback message should NOT be visible
		await expect(page.getByTestId('key-hover-fallback')).not.toBeVisible();

		// Verify multiple keys work correctly (not just the first one)
		await page.locator('[data-testid="key-3"]').hover();
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');

		// Key 3 is LT(1, KC_ESC) - should show both tap and hold actions
		await expect(page.getByText('Key Actions')).toBeVisible();
		await expect(page.getByText('Tap: Escape')).toBeVisible();
		await expect(page.getByText('Hold: Activate layer 1')).toBeVisible();

		// The fallback message should NOT be visible
		await expect(page.getByTestId('key-hover-fallback')).not.toBeVisible();
	});

	test('heading is always visible even when no key is selected (LazyQMK-mpm)', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();

		// Key details card should be visible with heading even without selection
		await expect(page.getByTestId('key-details-card')).toBeVisible();
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');
		
		// Empty state message should be shown
		await expect(page.getByTestId('key-details-empty')).toBeVisible();
		await expect(page.getByText('Select or hover over a key to view details')).toBeVisible();
	});

	test('key legend displays primary/secondary/tertiary labels (LazyQMK-t47)', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-3"]')).toBeVisible();

		// First, select a key to keep the details panel visible
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		
		// Move mouse away first
		await page.mouse.move(0, 0);
		
		// Hover over the multi-action key (LT(1, KC_ESC)) which has secondary label
		await page.locator('[data-testid="key-3"]').hover();

		// Key legend display should be visible
		await expect(page.getByTestId('key-legend-display')).toBeVisible();
		
		// Primary label should show "ESC"
		await expect(page.getByTestId('key-legend-primary')).toBeVisible();
		await expect(page.getByTestId('key-legend-primary')).toContainText('ESC');
		
		// Secondary label should show "L1" (for layer 1)
		await expect(page.getByTestId('key-legend-secondary')).toBeVisible();
		await expect(page.getByTestId('key-legend-secondary')).toContainText('L1');
	});

	test('key legend for simple key shows only primary label (LazyQMK-t47)', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Select a simple key to see its legend
		await page.locator('[data-testid="key-0"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		
		// Move mouse away to show customization mode
		await page.mouse.move(0, 0);
		
		// Hover over the same key to see legend
		await page.locator('[data-testid="key-0"]').hover();

		// Key legend display should be visible
		await expect(page.getByTestId('key-legend-display')).toBeVisible();
		
		// Primary label should show "Q"
		await expect(page.getByTestId('key-legend-primary')).toBeVisible();
		await expect(page.getByTestId('key-legend-primary')).toContainText('Q');
		
		// Secondary label should NOT be visible for simple keys
		await expect(page.getByTestId('key-legend-secondary')).not.toBeVisible();
	});

	test('description field is editable in selection mode (LazyQMK-yij)', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-0"]')).toBeVisible();

		// Enable selection mode
		await page.getByTestId('selection-mode-button').click();

		// Select a single key in selection mode
		await page.locator('[data-testid="key-0"]').click();
		
		// Move mouse away to exit hover state and show customization controls
		await page.mouse.move(0, 0);

		// Key details card should show customization section with description field
		await expect(page.getByTestId('key-details-card')).toBeVisible();
		await expect(page.getByTestId('key-details-heading')).toHaveText('Key Metadata');
		
		// Description input should be visible and editable
		const descriptionInput = page.getByTestId('key-description-input');
		await expect(descriptionInput).toBeVisible();
		
		// Type a description
		await descriptionInput.fill('My custom description for this key');
		
		// Trigger change event by blurring
		await descriptionInput.blur();
		
		// The value should be set
		await expect(descriptionInput).toHaveValue('My custom description for this key');
	});

	test('description field can be added when empty (LazyQMK-yij)', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Wait for keyboard preview to load
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.locator('[data-testid="key-1"]')).toBeVisible();

		// Select a key (not in selection mode - normal click opens picker)
		await page.locator('[data-testid="key-1"]').click();
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		
		// Move mouse away to show customization controls
		await page.mouse.move(0, 0);

		// Description input should be visible (empty initially)
		const descriptionInput = page.getByTestId('key-description-input');
		await expect(descriptionInput).toBeVisible();
		await expect(descriptionInput).toHaveValue('');
		
		// Placeholder should be visible
		await expect(descriptionInput).toHaveAttribute('placeholder', 'Add a note about this key...');
		
		// Add a new description
		await descriptionInput.fill('New description');
		await descriptionInput.blur();
		
		// Check that the value was set
		await expect(descriptionInput).toHaveValue('New description');
		
		// Unsaved changes indicator should appear
		await expect(page.getByText('Unsaved changes')).toBeVisible();
	});
});

