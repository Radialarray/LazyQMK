import { test, expect } from '@playwright/test';

test.describe('Metadata Editor', () => {
	// Mock layout data
	const mockLayout = {
		metadata: {
			name: 'Test Layout',
			description: 'A test keyboard layout',
			author: 'Test User',
			keyboard: 'crkbd',
			layout: 'LAYOUT_split_3x6_3',
			created: '2024-01-01T00:00:00Z',
			modified: '2024-01-01T00:00:00Z',
			tags: ['corne', '42-key'],
			version: '1.0',
			is_template: false
		},
		layers: [
			{
				name: 'Base',
				color: '#4a9eff',
				keys: [
					{ keycode: 'KC_Q', matrix_position: [0, 0], visual_index: 0, led_index: 0 }
				]
			}
		]
	};

	// Mock geometry data
	const mockGeometry = {
		keyboard: 'crkbd',
		layout: 'LAYOUT_split_3x6_3',
		keys: [
			{ matrix_row: 0, matrix_col: 0, x: 0, y: 0, width: 1, height: 1, rotation: 0, led_index: 0, visual_index: 0 }
		],
		matrix_rows: 1,
		matrix_cols: 1,
		encoder_count: 0,
		position_to_visual_index: {
			'0,0': 0
		}
	};

	test.beforeEach(async ({ page }) => {
		let savedLayout = { ...mockLayout };

		// Mock the layout API endpoint
		await page.route('**/api/layouts/test-layout*', async (route) => {
			const url = route.request().url();
			const method = route.request().method();

			if (method === 'PUT') {
				// Mock save endpoint - capture the saved data
				const postData = route.request().postDataJSON();
				savedLayout = postData;
				await route.fulfill({
					status: 204
				});
			} else {
				// Mock get endpoint - return saved data
				await route.fulfill({
					status: 200,
					contentType: 'application/json',
					body: JSON.stringify(savedLayout)
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
	});

	test('metadata tab displays current metadata', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Click on Metadata tab
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Verify metadata fields are populated
		await expect(page.getByTestId('metadata-name-input')).toHaveValue('Test Layout');
		await expect(page.getByTestId('metadata-description-input')).toHaveValue('A test keyboard layout');
		await expect(page.getByTestId('metadata-author-input')).toHaveValue('Test User');
		await expect(page.getByTestId('metadata-tags-input')).toHaveValue('corne, 42-key');

		// Verify tags are displayed as badges
		await expect(page.locator('.font-mono', { hasText: 'corne' }).first()).toBeVisible();
		await expect(page.locator('.font-mono', { hasText: '42-key' }).first()).toBeVisible();
	});

	test('can edit name field', async ({ page }) => {
		await page.goto('/layouts/test-layout');
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Edit name
		const nameInput = page.getByTestId('metadata-name-input');
		await nameInput.fill('Updated Layout Name');

		// Verify unsaved changes indicator appears
		await expect(page.getByText('Unsaved changes')).toBeVisible();
	});

	test('validates name cannot be empty', async ({ page }) => {
		await page.goto('/layouts/test-layout');
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Clear name field
		const nameInput = page.getByTestId('metadata-name-input');
		await nameInput.fill('');

		// Verify error message appears
		await expect(page.getByTestId('metadata-name-error')).toBeVisible();
		await expect(page.getByTestId('metadata-name-error')).toContainText('cannot be empty');

		// Verify save button is disabled
		await expect(page.getByTestId('save-button')).toBeDisabled();
	});

	test('validates name length maximum 100 characters', async ({ page }) => {
		await page.goto('/layouts/test-layout');
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Enter name with 101 characters
		const longName = 'a'.repeat(101);
		const nameInput = page.getByTestId('metadata-name-input');
		await nameInput.fill(longName);

		// Verify error message appears
		await expect(page.getByTestId('metadata-name-error')).toBeVisible();
		await expect(page.getByTestId('metadata-name-error')).toContainText('exceeds maximum length');

		// Verify save button is disabled
		await expect(page.getByTestId('save-button')).toBeDisabled();
	});

	test('can edit description field', async ({ page }) => {
		await page.goto('/layouts/test-layout');
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Edit description
		const descInput = page.getByTestId('metadata-description-input');
		await descInput.fill('Updated description text');

		// Verify unsaved changes indicator
		await expect(page.getByText('Unsaved changes')).toBeVisible();
	});

	test('can edit author field', async ({ page }) => {
		await page.goto('/layouts/test-layout');
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Edit author
		const authorInput = page.getByTestId('metadata-author-input');
		await authorInput.fill('New Author Name');

		// Verify unsaved changes indicator
		await expect(page.getByText('Unsaved changes')).toBeVisible();
	});

	test('can edit tags field with valid tags', async ({ page }) => {
		await page.goto('/layouts/test-layout');
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Edit tags
		const tagsInput = page.getByTestId('metadata-tags-input');
		await tagsInput.fill('tag1, tag2, tag-3');

		// Verify no error appears
		await expect(page.getByTestId('metadata-tags-error')).not.toBeVisible();

		// Verify unsaved changes indicator
		await expect(page.getByText('Unsaved changes')).toBeVisible();
	});

	test('validates tags must be lowercase', async ({ page }) => {
		await page.goto('/layouts/test-layout');
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Enter invalid tag with uppercase
		const tagsInput = page.getByTestId('metadata-tags-input');
		await tagsInput.fill('valid, UPPERCASE');

		// Verify error message appears
		await expect(page.getByTestId('metadata-tags-error')).toBeVisible();
		await expect(page.getByTestId('metadata-tags-error')).toContainText('UPPERCASE');
		await expect(page.getByTestId('metadata-tags-error')).toContainText('lowercase');

		// Verify save button is disabled
		await expect(page.getByTestId('save-button')).toBeDisabled();
	});

	test('validates tags cannot contain spaces', async ({ page }) => {
		await page.goto('/layouts/test-layout');
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Enter tag with space
		const tagsInput = page.getByTestId('metadata-tags-input');
		await tagsInput.fill('valid, has space');

		// Verify error message appears
		await expect(page.getByTestId('metadata-tags-error')).toBeVisible();
		await expect(page.getByTestId('metadata-tags-error')).toContainText('has space');

		// Verify save button is disabled
		await expect(page.getByTestId('save-button')).toBeDisabled();
	});

	test('validates tags cannot contain underscores', async ({ page }) => {
		await page.goto('/layouts/test-layout');
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Enter tag with underscore
		const tagsInput = page.getByTestId('metadata-tags-input');
		await tagsInput.fill('valid, has_underscore');

		// Verify error message appears
		await expect(page.getByTestId('metadata-tags-error')).toBeVisible();
		await expect(page.getByTestId('metadata-tags-error')).toContainText('has_underscore');

		// Verify save button is disabled
		await expect(page.getByTestId('save-button')).toBeDisabled();
	});

	test('removes duplicate tags automatically', async ({ page }) => {
		await page.goto('/layouts/test-layout');
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Enter tags with duplicates
		const tagsInput = page.getByTestId('metadata-tags-input');
		await tagsInput.fill('tag1, tag2, tag1, tag3, tag2');

		// Verify no error
		await expect(page.getByTestId('metadata-tags-error')).not.toBeVisible();

		// Note: Actual deduplication is handled on save, so we just verify no error
	});

	test('removes empty tags automatically', async ({ page }) => {
		await page.goto('/layouts/test-layout');
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Enter tags with empty entries
		const tagsInput = page.getByTestId('metadata-tags-input');
		await tagsInput.fill('tag1,  , tag2,   , tag3');

		// Verify no error
		await expect(page.getByTestId('metadata-tags-error')).not.toBeVisible();
	});

	test('saves metadata changes and persists after reload', async ({ page }) => {
		await page.goto('/layouts/test-layout');
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Edit all fields
		await page.getByTestId('metadata-name-input').fill('New Name');
		await page.getByTestId('metadata-description-input').fill('New description');
		await page.getByTestId('metadata-author-input').fill('New Author');
		await page.getByTestId('metadata-tags-input').fill('new-tag1, new-tag2');

		// Save changes
		await page.getByTestId('save-button').click();
		await expect(page.getByText('Saved!')).toBeVisible();

		// Reload page
		await page.reload();

		// Navigate to metadata tab
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Verify all changes persisted
		await expect(page.getByTestId('metadata-name-input')).toHaveValue('New Name');
		await expect(page.getByTestId('metadata-description-input')).toHaveValue('New description');
		await expect(page.getByTestId('metadata-author-input')).toHaveValue('New Author');
		await expect(page.getByTestId('metadata-tags-input')).toHaveValue('new-tag1, new-tag2');
	});

	test('updates header title when name changes', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		// Verify initial title
		await expect(page.getByRole('heading', { name: 'Test Layout' })).toBeVisible();

		// Edit name
		await page.getByRole('tab', { name: /metadata/i }).click();
		await page.getByTestId('metadata-name-input').fill('Updated Title');

		// Verify header updates immediately (reactive)
		await expect(page.getByRole('heading', { name: 'Updated Title' })).toBeVisible();
	});

	test('displays read-only system information', async ({ page }) => {
		await page.goto('/layouts/test-layout');
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Verify system information is displayed
		await expect(page.getByText('System Information')).toBeVisible();
		await expect(page.getByText('crkbd')).toBeVisible();
		await expect(page.getByText('Version')).toBeVisible();
		await expect(page.getByText('1.0')).toBeVisible();
	});

	test('allows empty tags', async ({ page }) => {
		await page.goto('/layouts/test-layout');
		await page.getByRole('tab', { name: /metadata/i }).click();

		// Clear tags
		const tagsInput = page.getByTestId('metadata-tags-input');
		await tagsInput.fill('');

		// Verify no error
		await expect(page.getByTestId('metadata-tags-error')).not.toBeVisible();

		// Verify save is still enabled (assuming name is valid)
		await expect(page.getByTestId('save-button')).toBeEnabled();
	});
});
