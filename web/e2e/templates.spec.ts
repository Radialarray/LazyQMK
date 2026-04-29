import { test, expect } from '@playwright/test';

const mockTemplates = {
	templates: [
		{
			filename: 'gaming-split.md',
			name: 'Gaming Split',
			description: 'Split gaming starter with layered controls',
			author: 'Example Author',
			tags: ['gaming', 'split', 'corne'],
			created: '2024-01-01T00:00:00Z',
			layer_count: 4
		},
		{
			filename: 'minimal-42.md',
			name: 'Minimal 42',
			description: 'Compact daily driver for 42-key boards',
			author: 'Example Author',
			tags: ['minimal', '42-key'],
			created: '2024-01-02T00:00:00Z',
			layer_count: 2
		}
	]
};

test.describe('Templates Feature', () => {
	test.beforeEach(async ({ page }) => {
		await page.route('**/api/templates', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockTemplates)
			});
		});
	});

	test('shows IA cues on template cards', async ({ page }) => {
		await page.goto('/templates');

		await expect(page.getByRole('heading', { name: 'Layout Templates' })).toBeVisible();
		await expect(page.getByText('Use-case cues')).toBeVisible();
		await expect(page.getByText('Compatibility cues')).toBeVisible();
		await expect(page.getByText('Good for gaming-focused boards')).toBeVisible();
		await expect(page.getByText('Balanced for daily multi-layer use')).toBeVisible();
	});

	test('applies template from accessible dialog', async ({ page }) => {
		await page.route('**/api/templates/gaming-split.md/apply', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({ filename: 'my-gaming-layout.md' })
			});
		});

		await page.goto('/templates');
		await page.getByRole('button', { name: 'Use as Starting Point' }).first().click();

		await expect(page.getByRole('heading', { name: 'Apply template' })).toBeVisible();
		await page.getByLabel(/New Layout Name/i).fill('my-gaming-layout');
		await page.getByRole('button', { name: 'Create Layout' }).click();

		await expect(page).toHaveURL(/\/layouts\/my-gaming-layout/);
	});

	test('filters templates by search query', async ({ page }) => {
		await page.goto('/templates');
		await page.getByPlaceholder(/search starter layouts by name, purpose, or tags/i).fill('Minimal 42');

		await expect(page.getByText('Minimal 42')).toBeVisible();
		await expect(page.getByText('Gaming Split')).not.toBeVisible();
	});
});
