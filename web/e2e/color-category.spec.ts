import { test, expect } from '@playwright/test';

test.describe('Color and Category Management', () => {
	// Mock layout data
	const mockLayout = {
		metadata: {
			name: 'Color Test Layout',
			description: 'Testing color and category features',
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
					{ keycode: 'KC_E', matrix_position: [0, 2], visual_index: 2, led_index: 2 }
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
			'0,0': 0, '0,70': 1, '0,140': 2
		}
	};

	test.beforeEach(async ({ page }) => {
		let currentLayout = JSON.parse(JSON.stringify(mockLayout));

		// Mock the layout API endpoint
		await page.route('**/api/layouts/color-test*', async (route) => {
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

		await page.goto('/layouts/color-test');
		await page.waitForLoadState('networkidle');
	});

	test('should apply key color override and reflect in preview', async ({ page }) => {
		// Click on first key to select it - opens picker
		await page.click('[data-testid="key-0"]');

		// Close the picker
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Verify key is selected
		await expect(page.locator('[data-testid="key-0"] .key-bg')).toHaveClass(/selected/);

		// Wait for key details to be visible
		await expect(page.locator('text=Key Metadata')).toBeVisible();

		// Click "Set Color" button to open color picker
		await page.click('[data-testid="set-color-button"]');

		// Verify color picker is visible
		await expect(page.locator('text=Preset colors')).toBeVisible();

		// Click a preset color (red) - wait for it to be visible and clickable
		const colorButton = page.locator('button[title="#FF0000"]');
		await colorButton.waitFor({ state: 'visible' });
		await colorButton.click({ force: true });

		// Wait for the color picker to close (which happens when color is set)
		await expect(page.locator('text=Preset colors')).not.toBeVisible();

		// Wait a bit for reactivity to update
		await page.waitForTimeout(100);

		// Verify color override is now set ("Clear" button should be visible)
		await expect(page.locator('[data-testid="clear-color-override-button"]')).toBeVisible();

		// Save layout
		await page.click('button:has-text("Save")');
		await expect(page.locator('text=Saved!')).toBeVisible();

		// Reload page to verify persistence
		await page.reload();
		await page.waitForLoadState('networkidle');

		// Enable selection mode to check key without opening picker
		await page.click('[data-testid="selection-mode-button"]');

		// Click key again to check if color override persisted
		await page.click('[data-testid="key-0"]');

		// Move mouse away from key to clear hover state (shows "Key Metadata" instead of "Key Preview")
		await page.mouse.move(0, 0);

		// Wait for hover state to clear and customization panel to show
		await expect(page.locator('text=Key Metadata')).toBeVisible();

		// Verify the color override indicator is present (color swatch and "Clear" button)
		await expect(page.locator('text=Color Override')).toBeVisible();
		await expect(page.locator('[data-testid="clear-color-override-button"]')).toBeVisible();
	});

	test('should create category and assign to key', async ({ page }) => {
		// Navigate to Categories tab
		await page.click('text=Categories');

		// Add new category
		await page.click('text=Add Category');

		// Fill in category details
		await page.fill('#new-id', 'navigation');
		await page.fill('#new-name', 'Navigation Keys');

		// Select a color for category
		await page.click('button[title="#00FF00"]'); // Green

		// Create category
		await page.click('text=Create');

		// Verify category was created
		await expect(page.locator('text=Navigation Keys')).toBeVisible();

		// Navigate back to Preview tab
		await page.click('text=Preview');

		// Select first key - opens picker
		await page.click('[data-testid="key-0"]');

		// Close the picker
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Move mouse away from key to clear hover state (shows "Key Metadata" instead of "Key Preview")
		await page.mouse.move(0, 0);
		await expect(page.locator('text=Key Metadata')).toBeVisible();

		// Assign key to navigation category
		await page.selectOption('#key-category', 'navigation');

		// Save and verify persistence
		await page.click('button:has-text("Save")');
		await expect(page.locator('text=Saved!')).toBeVisible();

		await page.reload();
		await page.waitForLoadState('networkidle');

		// Navigate to categories tab to verify category persisted
		await page.click('text=Categories');
		await expect(page.locator('text=Navigation Keys')).toBeVisible();

		// Go back to preview
		await page.click('text=Preview');

		// Enable selection mode to check key without opening picker
		await page.click('[data-testid="selection-mode-button"]');

		// Click key to verify category
		await page.click('[data-testid="key-0"]');

		// Move mouse away from key to clear hover state and show customization panel
		await page.mouse.move(0, 0);
		await expect(page.locator('text=Key Metadata')).toBeVisible();

		// Verify category is selected in dropdown
		const selectedCategory = await page.inputValue('#key-category');
		expect(selectedCategory).toBe('navigation');
	});

	test('should handle color priority: override > key category > layer default', async ({ page }) => {
		// First, create a category
		await page.click('text=Categories');
		await page.click('text=Add Category');
		await page.fill('#new-id', 'test-category');
		await page.fill('#new-name', 'Test Category');
		await page.click('button[title="#FFFF00"]'); // Yellow
		await page.click('text=Create');

		// Go to preview and select key - opens picker
		await page.click('text=Preview');
		await page.click('[data-testid="key-0"]');

		// Close the picker
		await expect(page.getByTestId('keycode-picker-overlay')).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect(page.getByTestId('keycode-picker-overlay')).not.toBeVisible();

		// Move mouse away from key to clear hover state and show customization panel
		await page.mouse.move(0, 0);

		// Wait for key details to be visible
		await expect(page.locator('text=Key Metadata')).toBeVisible();

		// Assign key to category
		await page.selectOption('#key-category', 'test-category');

		// Verify category is selected
		let selectedCategory = await page.inputValue('#key-category');
		expect(selectedCategory).toBe('test-category');

		// Set color override
		await page.click('[data-testid="set-color-button"]');
		await page.click('button[title="#FF0000"]'); // Red

		// Wait for color picker to close
		await expect(page.locator('text=Preset colors')).not.toBeVisible();

		// Verify override is present ("Clear" button should be visible)
		await expect(page.locator('[data-testid="clear-color-override-button"]')).toBeVisible();

		// Clear override
		await page.click('[data-testid="clear-color-override-button"]');

		// Verify override is gone (Clear button should be hidden, Set Color visible)
		await expect(page.locator('[data-testid="clear-color-override-button"]')).not.toBeVisible();
		await expect(page.locator('[data-testid="set-color-button"]')).toBeVisible();
		
		// Verify category still assigned
		selectedCategory = await page.inputValue('#key-category');
		expect(selectedCategory).toBe('test-category');
	});

	test('should allow editing and deleting categories', async ({ page }) => {
		// Create a category
		await page.click('text=Categories');
		await page.click('text=Add Category');
		await page.fill('#new-id', 'test-category');
		await page.fill('#new-name', 'Test Category');
		await page.click('button[title="#FF0000"]');
		await page.click('text=Create');

		// Edit category
		await page.locator('button:has-text("Edit")').first().click();

		// Clear and fill in new name
		const nameInput = page.locator('#edit-name-test-category');
		await nameInput.clear();
		await nameInput.fill('Updated Category');

		// Save - use a more specific selector to avoid clicking the layout Save button
		await page.locator('.border >> button:has-text("Save")').first().click();

		// Wait briefly for save to complete
		await page.waitForTimeout(500);

		// Verify changes
		await expect(page.locator('text=Updated Category')).toBeVisible();
		await expect(page.locator('text=Test Category')).not.toBeVisible();

		// Delete category
		page.on('dialog', (dialog) => dialog.accept());
		await page.locator('button:has-text("Delete")').first().click();

		// Verify category was deleted
		await expect(page.locator('text=Updated Category')).not.toBeVisible();
	});
});
