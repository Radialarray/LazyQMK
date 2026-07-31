import { test, expect } from '@playwright/test';

test.describe('Layer Default Color', () => {
	// Mock layout data with a layer without default color
	const mockLayout = {
		metadata: {
			name: 'Layer Color Test',
			description: 'Testing layer default color features',
			author: 'Test User',
			keyboard: 'crkbd',
			layout: 'LAYOUT_split_3x6_3',
			layout_variant: 'LAYOUT_split_3x6_3',
			created: '2024-01-01T00:00:00Z',
			modified: '2024-01-01T00:00:00Z'
		},
		layers: [
			{
				name: 'Base',
				number: 0,
				color: '#4a9eff',
				layer_colors_enabled: true,
				keys: [
					{ keycode: 'KC_Q', matrix_position: [0, 0], visual_index: 0, led_index: 0 },
					{ keycode: 'KC_W', matrix_position: [0, 1], visual_index: 1, led_index: 1 },
					{ keycode: 'KC_E', matrix_position: [0, 2], visual_index: 2, led_index: 2 }
				]
			},
			{
				name: 'Lower',
				number: 1,
				color: '#ff9e4a',
				default_color: { r: 255, g: 158, b: 74 },
				layer_colors_enabled: true,
				keys: [
					{ keycode: 'KC_1', matrix_position: [0, 0], visual_index: 0, led_index: 0 },
					{ keycode: 'KC_2', matrix_position: [0, 1], visual_index: 1, led_index: 1 },
					{ keycode: 'KC_3', matrix_position: [0, 2], visual_index: 2, led_index: 2 }
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
			{
				matrix_row: 0,
				matrix_col: 0,
				x: 0,
				y: 0,
				width: 60,
				height: 60,
				rotation: 0,
				led_index: 0,
				visual_index: 0
			},
			{
				matrix_row: 0,
				matrix_col: 1,
				x: 70,
				y: 0,
				width: 60,
				height: 60,
				rotation: 0,
				led_index: 1,
				visual_index: 1
			},
			{
				matrix_row: 0,
				matrix_col: 2,
				x: 140,
				y: 0,
				width: 60,
				height: 60,
				rotation: 0,
				led_index: 2,
				visual_index: 2
			}
		],
		matrix_rows: 1,
		matrix_cols: 3,
		encoder_count: 0,
		position_to_visual_index: {
			'0,0': 0,
			'0,70': 1,
			'0,140': 2
		}
	};

	// Mock render metadata
	const mockRenderMetadata = {
		filename: 'layer-color-test',
		layers: [
			{
				number: 0,
				name: 'Base',
				keys: [
					{ visual_index: 0, display: { primary: 'Q' }, details: [] },
					{ visual_index: 1, display: { primary: 'W' }, details: [] },
					{ visual_index: 2, display: { primary: 'E' }, details: [] }
				]
			},
			{
				number: 1,
				name: 'Lower',
				keys: [
					{ visual_index: 0, display: { primary: '1' }, details: [] },
					{ visual_index: 1, display: { primary: '2' }, details: [] },
					{ visual_index: 2, display: { primary: '3' }, details: [] }
				]
			}
		]
	};

	test.beforeEach(async ({ page }) => {
		let currentLayout = JSON.parse(JSON.stringify(mockLayout));

		// Mock the layout API endpoint
		await page.route('**/api/layouts/layer-color-test*', async (route) => {
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

		// Mock the render metadata API endpoint
		await page.route('**/api/layouts/layer-color-test/render-metadata', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockRenderMetadata)
			});
		});

		await page.goto('/layouts/layer-color-test');
		await page.waitForLoadState('networkidle');
	});

	test('should display Set Color button for layer without default color', async ({ page }) => {
		// Navigate to Layers tab
		await page.click('text=Layers');

		// First layer (Base) should show "Set Color" button (no default color set)
		const layer0ColorButton = page.locator('[data-testid="layer-0-color-button"]');
		await expect(layer0ColorButton).toBeVisible();
		await expect(layer0ColorButton).toContainText('Set Color');
	});

	test('should display Color button with swatch for layer with default color', async ({ page }) => {
		// Navigate to Layers tab
		await page.click('text=Layers');

		// Second layer (Lower) should show color swatch (has default color)
		const layer1ColorButton = page.locator('[data-testid="layer-1-color-button"]');
		await expect(layer1ColorButton).toBeVisible();
		await expect(layer1ColorButton).toContainText('Color');
		
		// Should have a color swatch
		const swatch = layer1ColorButton.locator('span.rounded');
		await expect(swatch).toBeVisible();
	});

	test('should open color picker when clicking Set Color button', async ({ page }) => {
		// Navigate to Layers tab
		await page.click('text=Layers');

		// Click Set Color button for first layer
		await page.click('[data-testid="layer-0-color-button"]');

		// Color picker should be visible inside a modal
		await expect(page.locator('[data-testid="layer-color-picker"]')).toBeVisible();
		await expect(page.locator('#layer-color-picker-title')).toHaveText('Layer Default Color');
		await expect(page.locator('text=Set the default color for Base')).toBeVisible();
		await expect(page.locator('text=Preset colors')).toBeVisible();
	});

	test('should set layer default color via preset palette', async ({ page }) => {
		// Navigate to Layers tab
		await page.click('text=Layers');

		// Click Set Color button for first layer
		await page.click('[data-testid="layer-0-color-button"]');

		// Wait for color picker to be visible
		await expect(page.locator('[data-testid="layer-color-picker"]')).toBeVisible();

		// Click a preset color (red) — aria-label embeds the hex
		await page.click('button[aria-label*="#FF0000"]');

		// Color picker should close
		await expect(page.locator('[data-testid="layer-color-picker"]')).not.toBeVisible();

		// Button should now show color swatch
		const layer0ColorButton = page.locator('[data-testid="layer-0-color-button"]');
		await expect(layer0ColorButton).toContainText('Color');
		const swatch = layer0ColorButton.locator('span.rounded');
		await expect(swatch).toBeVisible();
	});

	test('should clear layer default color', async ({ page }) => {
		// Navigate to Layers tab
		await page.click('text=Layers');

		// Click Color button for second layer (which has a default color)
		await page.click('[data-testid="layer-1-color-button"]');

		// Wait for color picker to be visible
		await expect(page.locator('[data-testid="layer-color-picker"]')).toBeVisible();

		// Click Clear button
		await page.click('[data-testid="color-picker-clear-button"]');

		// Color picker should close
		await expect(page.locator('[data-testid="layer-color-picker"]')).not.toBeVisible();

		// Button should now show "Set Color" (no swatch)
		const layer1ColorButton = page.locator('[data-testid="layer-1-color-button"]');
		await expect(layer1ColorButton).toContainText('Set Color');
	});

	test('should persist layer default color after save', async ({ page }) => {
		// Navigate to Layers tab
		await page.click('text=Layers');

		// Click Set Color button for first layer
		await page.click('[data-testid="layer-0-color-button"]');

		// Wait for color picker to be visible
		await expect(page.locator('[data-testid="layer-color-picker"]')).toBeVisible();

		// Click a preset color (green) — aria-label embeds the hex
		await page.click('button[aria-label*="#00FF00"]');

		// Save layout
		await page.click('[data-testid="save-button"]');
		await expect(page.locator('text=Saved!')).toBeVisible();

		// Reload page
		await page.reload();
		await page.waitForLoadState('networkidle');

		// Navigate to Layers tab
		await page.click('text=Layers');

		// Verify color persisted - should show "Color" with swatch
		const layer0ColorButton = page.locator('[data-testid="layer-0-color-button"]');
		await expect(layer0ColorButton).toContainText('Color');
		const swatch = layer0ColorButton.locator('span.rounded');
		await expect(swatch).toBeVisible();
	});

	test('should show explanation text in color picker', async ({ page }) => {
		// Navigate to Layers tab
		await page.click('text=Layers');

		// Click Set Color button
		await page.click('[data-testid="layer-0-color-button"]');

		// Verify explanation text is visible
		await expect(
			page.locator("text=Applies to keys on this layer that have no individual or category color.")
		).toBeVisible();
	});

	test('should show global-brightness caption inside the modal', async ({ page }) => {
		await page.click('text=Layers');
		await page.click('[data-testid="layer-0-color-button"]');

		const caption = page.locator('[data-testid="layer-color-brightness-caption"]');
		await expect(caption).toBeVisible();
		await expect(caption).toContainText('Brightness for uncolored keys on this layer follows the global setting');
		await expect(caption).toContainText('Settings → Background Lighting');
	});

	test('should close modal via Escape key', async ({ page }) => {
		await page.click('text=Layers');
		await page.click('[data-testid="layer-0-color-button"]');

		await expect(page.locator('[data-testid="layer-color-picker"]')).toBeVisible();

		await page.keyboard.press('Escape');

		await expect(page.locator('[data-testid="layer-color-picker"]')).not.toBeVisible();
	});

	test('should close modal via backdrop click', async ({ page }) => {
		await page.click('text=Layers');
		await page.click('[data-testid="layer-1-color-button"]');

		await expect(page.locator('[data-testid="layer-color-picker"]')).toBeVisible();

		// Click far outside the modal content (top-left corner of backdrop)
		await page.locator('[role="dialog"]').click({ position: { x: 5, y: 5 } });

		await expect(page.locator('[data-testid="layer-color-picker"]')).not.toBeVisible();
	});

	test('should close modal via Done button', async ({ page }) => {
		await page.click('text=Layers');
		await page.click('[data-testid="layer-0-color-button"]');

		await expect(page.locator('[data-testid="layer-color-picker"]')).toBeVisible();

		await page.click('[data-testid="layer-color-picker-done"]');

		await expect(page.locator('[data-testid="layer-color-picker"]')).not.toBeVisible();
	});
});
