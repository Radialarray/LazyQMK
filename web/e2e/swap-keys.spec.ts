import { test, expect } from '@playwright/test';

test.describe('Swap Keys Mode', () => {
	const mockLayout = {
		metadata: {
			name: 'Test Layout',
			description: 'Swap test layout',
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
					{ keycode: 'KC_W', matrix_position: [0, 1], visual_index: 1, led_index: 1 }
				]
			}
		]
	};

	const swappedLayout = {
		...mockLayout,
		layers: [
			{
				...mockLayout.layers[0],
				keys: [
					{ keycode: 'KC_W', matrix_position: [0, 0], visual_index: 0, led_index: 0 },
					{ keycode: 'KC_Q', matrix_position: [0, 1], visual_index: 1, led_index: 1 }
				]
			}
		]
	};

	const mockGeometry = {
		keyboard: 'crkbd',
		layout: 'LAYOUT_split_3x6_3',
		keys: [
			{ matrix_row: 0, matrix_col: 0, x: 0, y: 0, width: 1, height: 1, rotation: 0, led_index: 0, visual_index: 0 },
			{ matrix_row: 0, matrix_col: 1, x: 1, y: 0, width: 1, height: 1, rotation: 0, led_index: 1, visual_index: 1 }
		],
		matrix_rows: 1,
		matrix_cols: 2,
		encoder_count: 0,
		position_to_visual_index: {
			'0,0': 0,
			'0,1': 1
		}
	};

	const mockKeycodes = {
		keycodes: [
			{ code: 'KC_Q', name: 'Q', category: 'basic', description: 'Letter Q' },
			{ code: 'KC_W', name: 'W', category: 'basic', description: 'Letter W' }
		],
		total: 2
	};

	const mockCategories = {
		categories: [
			{ id: 'basic', name: 'Basic', description: 'Basic keys' }
		]
	};

	const mockRenderMetadata = {
		layers: [
			{
				number: 0,
				keys: [
					{ visual_index: 0, display: { primary: 'Q' } },
					{ visual_index: 1, display: { primary: 'W' } }
				]
			}
		]
	};

	const mockSwappedRenderMetadata = {
		layers: [
			{
				number: 0,
				keys: [
					{ visual_index: 0, display: { primary: 'W' } },
					{ visual_index: 1, display: { primary: 'Q' } }
				]
			}
		]
	};

	const setupMocks = async (page) => {
		let layoutState = mockLayout;
		let renderMetadataState = mockRenderMetadata;

		await page.route('**/api/layouts/**', async (route) => {
			const method = route.request().method();
			const url = route.request().url();
			if (method === 'PUT') {
				await route.fulfill({ status: 204 });
				return;
			}
			if (method === 'GET' && url.includes('/render-metadata')) {
				await route.fulfill({
					status: 200,
					contentType: 'application/json',
					body: JSON.stringify(renderMetadataState)
				});
				return;
			}
			if (method === 'POST' && url.includes('/swap-keys')) {
				layoutState = swappedLayout;
				renderMetadataState = mockSwappedRenderMetadata;
				await route.fulfill({ status: 204 });
				return;
			}
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(layoutState)
			});
		});

		await page.route('**/api/keyboards/crkbd/geometry/LAYOUT_split_3x6_3', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockGeometry)
			});
		});

		await page.route('**/api/keycodes*', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockKeycodes)
			});
		});

		await page.route('**/api/keycodes/categories', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockCategories)
			});
		});

	};

		test('should activate swap mode when Shift+W is pressed', async ({ page }) => {
			await setupMocks(page);
			await page.goto('/layouts/test-layout');

			await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
			await expect(page.getByTestId('key-0')).toBeVisible();
			await page.locator('.keyboard-preview').click();

			await page.keyboard.press('Shift+W');

			await expect(page.locator('text=Swap mode - click first key to swap')).toBeVisible();
			await expect(page.getByTestId('swap-mode-button')).toHaveText('✓ Swap Mode');
			await expect(page.locator('[data-testid="key-0"] rect.key-bg.swap-first')).toHaveCount(0);
		});

		test('should swap two keys when clicked in swap mode', async ({ page }) => {
			await setupMocks(page);
			await page.goto('/layouts/test-layout');

			await expect(page.getByTestId('key-0')).toBeVisible();
			await page.locator('.keyboard-preview').click();

			await page.keyboard.press('Shift+W');
			await expect(page.locator('text=Swap mode - click first key to swap')).toBeVisible();

		await page.getByTestId('key-0').dispatchEvent('click');
		await expect(page.locator('text=Swap mode - click second key to swap')).toBeVisible();
		await expect(page.locator('[data-testid="key-0"] rect.key-bg.swap-first')).toHaveCount(1);

		await page.getByTestId('key-1').dispatchEvent('click');

		await expect(page.locator('text=Keys swapped')).toBeVisible();
		await expect(page.locator('[data-testid="key-0"] rect.key-bg.swap-first')).toHaveCount(0);
	});

		test('should exit swap mode after successful swap', async ({ page }) => {
			await setupMocks(page);
			await page.goto('/layouts/test-layout');
			await page.locator('.keyboard-preview').click();

			await page.keyboard.press('Shift+W');
		await page.getByTestId('key-0').dispatchEvent('click');
		await page.getByTestId('key-1').dispatchEvent('click');

		await expect(page.getByTestId('swap-mode-button')).toHaveText('Swap Mode');
		await expect(page.locator('[data-testid="key-0"] rect.key-bg.swap-first')).toHaveCount(0);
	});

		test('should show error when trying to swap key with itself', async ({ page }) => {
			await setupMocks(page);
			await page.goto('/layouts/test-layout');
			await page.locator('.keyboard-preview').click();

			await page.keyboard.press('Shift+W');
		await page.getByTestId('key-0').dispatchEvent('click');
		await page.getByTestId('key-0').dispatchEvent('click');

		await expect(page.locator('text=Cannot swap a key with itself')).toBeVisible();
		await expect(page.getByTestId('swap-mode-button')).toHaveText('✓ Swap Mode');
		await expect(page.locator('[data-testid="key-0"] rect.key-bg.swap-first')).toHaveCount(1);
	});
});
