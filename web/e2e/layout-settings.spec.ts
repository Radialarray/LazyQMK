import { test, expect } from '@playwright/test';

test.describe('Layout Settings Tab', () => {
	const mockLayout = {
		metadata: {
			name: 'Settings Test',
			description: 'Verifies Background Lighting controls',
			author: 'Test User',
			keyboard: 'crkbd',
			layout: 'LAYOUT_split_3x6_3',
			layout_variant: 'LAYOUT_split_3x6_3',
			created: '2024-01-01T00:00:00Z',
			modified: '2024-01-01T00:00:00Z'
		},
		rgb_enabled: true,
		rgb_brightness: 100,
		rgb_saturation: 150,
		rgb_matrix_default_speed: 127,
		rgb_timeout_ms: 60000,
		uncolored_key_behavior: 40,
		idle_effect_settings: {
			enabled: true,
			idle_timeout_ms: 60000,
			idle_effect_duration_ms: 300000,
			idle_effect_mode: 'Breathing'
		},
		palette_fx: {
			enabled: true,
			default_effect: 'Flow',
			default_palette: 'Synthwave',
			enable_all_effects: true,
			enable_all_palettes: true
		},
		rgb_overlay_ripple: {
			enabled: false,
			max_ripples: 4,
			duration_ms: 1500,
			speed: 200,
			band_width: 30,
			amplitude_pct: 50,
			color_mode: 'Fixed Color',
			fixed_color: { r: 0, g: 255, b: 255 },
			hue_shift_deg: 60,
			trigger_on_press: true,
			trigger_on_release: false,
			ignore_transparent: true,
			ignore_modifiers: false,
			ignore_layer_switch: false
		},
		layers: [
			{
				name: 'Base',
				number: 0,
				color: '#888888',
				keys: [
					{ keycode: 'KC_Q', matrix_position: [0, 0], visual_index: 0, led_index: 0 },
					{ keycode: 'KC_W', matrix_position: [0, 1], visual_index: 1, led_index: 1 },
					{ keycode: 'KC_E', matrix_position: [0, 2], visual_index: 2, led_index: 2 }
				]
			}
		],
		categories: []
	};

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
		position_to_visual_index: { '0,0': 0, '0,70': 1, '0,140': 2 }
	};

	const mockRenderMetadata = {
		filename: 'settings-test',
		layers: [
			{
				number: 0,
				name: 'Base',
				keys: [
					{ visual_index: 0, display: { primary: 'Q' }, details: [] },
					{ visual_index: 1, display: { primary: 'W' }, details: [] },
					{ visual_index: 2, display: { primary: 'E' }, details: [] }
				]
			}
		]
	};

	test.beforeEach(async ({ page }) => {
		let currentLayout = JSON.parse(JSON.stringify(mockLayout));

		await page.route('**/api/layouts/settings-test*', async (route) => {
			const method = route.request().method();
			if (method === 'PUT') {
				const body = route.request().postData();
				if (body) currentLayout = JSON.parse(body);
				await route.fulfill({ status: 204 });
			} else {
				await route.fulfill({
					status: 200,
					contentType: 'application/json',
					body: JSON.stringify(currentLayout)
				});
			}
		});

		await page.route('**/api/keyboards/crkbd/geometry/LAYOUT_split_3x6_3', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockGeometry)
			});
		});

		await page.route('**/api/layouts/settings-test/render-metadata', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockRenderMetadata)
			});
		});

		await page.goto('/layouts/settings-test');
		await page.waitForLoadState('networkidle');
	});

	test('exposes the Settings tab in primary tab navigation', async ({ page }) => {
		const settingsTab = page.locator('[data-testid="tab-settings"]');
		await expect(settingsTab).toBeVisible();
		await expect(settingsTab).toContainText('Settings');
	});

	test('renders all four lighting sections', async ({ page }) => {
		await page.click('[data-testid="tab-settings"]');

		await expect(page.locator('[data-testid="background-lighting-card"]')).toBeVisible();
		await expect(page.locator('[data-testid="idle-lighting-card"]')).toBeVisible();
		await expect(page.locator('[data-testid="palette-fx-card"]')).toBeVisible();
		await expect(page.locator('[data-testid="ripple-lighting-card"]')).toBeVisible();
	});

	test('reflects current global brightness on first paint', async ({ page }) => {
		await page.click('[data-testid="tab-settings"]');
		const slider = page.locator('[data-testid="uncolored-brightness-slider"]');
		await expect(slider).toBeVisible();
		await expect(slider).toHaveValue('40');
	});

	test('moving the uncolored brightness slider updates the percent label', async ({ page }) => {
		await page.click('[data-testid="tab-settings"]');

		const slider = page.locator('[data-testid="uncolored-brightness-slider"]');
		await slider.evaluate((el) => {
			const input = el as HTMLInputElement;
			input.value = '75';
			input.dispatchEvent(new Event('input', { bubbles: true }));
		});

		await expect(page.locator('text=75%').first()).toBeVisible();
	});

	test('toggling rgb_enabled marks the layout dirty', async ({ page }) => {
		await page.click('[data-testid="tab-settings"]');
		await page.uncheck('#rgb-enabled');

		const saveButton = page.locator('[data-testid="save-button"]');
		await expect(saveButton).toBeEnabled();
	});

	test('saving persists uncolored_key_behavior value', async ({ page }) => {
		await page.click('[data-testid="tab-settings"]');

		const slider = page.locator('[data-testid="uncolored-brightness-slider"]');
		await slider.evaluate((el) => {
			const input = el as HTMLInputElement;
			input.value = '65';
			input.dispatchEvent(new Event('input', { bubbles: true }));
		});

		await page.click('[data-testid="save-button"]');
		await expect(page.locator('text=Saved!')).toBeVisible();

		await page.reload();
		await page.waitForLoadState('networkidle');
		await page.click('[data-testid="tab-settings"]');

		const restoredSlider = page.locator('[data-testid="uncolored-brightness-slider"]');
		await expect(restoredSlider).toHaveValue('65');
	});

	test('idle lighting section keeps its existing controls', async ({ page }) => {
		await page.click('[data-testid="tab-settings"]');
		await expect(page.locator('#idle-enabled')).toBeVisible();
		await expect(page.locator('#idle-timeout')).toBeVisible();
		await expect(page.locator('#idle-duration')).toBeVisible();
		await expect(page.locator('#idle-effect-mode')).toBeVisible();
	});

	test('palette fx section keeps its existing controls', async ({ page }) => {
		await page.click('[data-testid="tab-settings"]');
		await expect(page.locator('#pfx-enabled')).toBeVisible();
		await expect(page.locator('#pfx-effect')).toBeVisible();
		await expect(page.locator('#pfx-palette')).toBeVisible();
	});

	test('legacy tabs (idle-effect, palette-fx, overlay-ripple) are removed', async ({ page }) => {
		await expect(page.locator('[data-testid="tab-idle-effect"]')).toHaveCount(0);
		await expect(page.locator('[data-testid="tab-palette-fx"]')).toHaveCount(0);
		await expect(page.locator('[data-testid="tab-overlay-ripple"]')).toHaveCount(0);
	});
});