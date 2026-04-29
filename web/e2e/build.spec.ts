import { test, expect } from '@playwright/test';

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
			keys: [{ keycode: 'KC_Q', matrix_position: [0, 0], visual_index: 0, led_index: 0 }]
		}
	]
};

const mockGeometry = {
	keyboard: 'crkbd',
	layout: 'LAYOUT_split_3x6_3',
	keys: [
		{ matrix_row: 0, matrix_col: 0, x: 0, y: 0, width: 1, height: 1, rotation: 0, led_index: 0, visual_index: 0 }
	],
	matrix_rows: 1,
	matrix_cols: 1,
	encoder_count: 0,
	position_to_visual_index: { '0,0': 0 }
};

const mockBuildJob = {
	id: 'job-123',
	status: 'pending',
	layout_filename: 'test-layout',
	keyboard: 'crkbd',
	keymap: 'default',
	created_at: '2024-01-01T12:00:00Z',
	progress: 0
};

const mockRunningJob = {
	...mockBuildJob,
	status: 'running',
	started_at: '2024-01-01T12:00:01Z',
	progress: 50
};

const mockCompletedJob = {
	...mockBuildJob,
	status: 'completed',
	started_at: '2024-01-01T12:00:01Z',
	completed_at: '2024-01-01T12:01:00Z',
	progress: 100,
	firmware_path: '/tmp/crkbd_default.uf2'
};

const mockLogs = {
	job_id: 'job-123',
	logs: [
		{ timestamp: '2024-01-01T12:00:01Z', level: 'INFO', message: 'Build started' },
		{ timestamp: '2024-01-01T12:00:02Z', level: 'INFO', message: 'Compiling...' },
		{ timestamp: '2024-01-01T12:00:03Z', level: 'INFO', message: 'Build progress: 50%' }
	],
	has_more: false
};

const mockArtifacts = {
	job_id: 'job-123',
	artifacts: [
		{ id: 'firmware.uf2', filename: 'firmware.uf2', artifact_type: 'uf2', size: 1024, sha256: 'abcdef1234567890' }
	]
};

async function mockEditor(page: Parameters<typeof test.beforeEach>[0]['page']) {
	await page.route('**/api/layouts/test-layout*', async (route) => {
		if (route.request().method() === 'PUT') {
			await route.fulfill({ status: 204 });
			return;
		}

		await route.fulfill({
			status: 200,
			contentType: 'application/json',
			body: JSON.stringify(mockLayout)
		});
	});

	await page.route('**/api/keyboards/crkbd/geometry/LAYOUT_split_3x6_3', async (route) => {
		await route.fulfill({
			status: 200,
			contentType: 'application/json',
			body: JSON.stringify(mockGeometry)
		});
	});

	await page.route('**/api/build/jobs', async (route) => {
		await route.fulfill({
			status: 200,
			contentType: 'application/json',
			body: JSON.stringify([])
		});
	});
}

async function openBuildStep(page: Parameters<typeof test.beforeEach>[0]['page']) {
	await page.goto('/layouts/test-layout');
	await page.getByTestId('tab-firmware').click();
	await page.getByTestId('firmware-step-build').click();
}

test.describe('Firmware workflow build step', () => {
	test.beforeEach(async ({ page }) => {
		await mockEditor(page);
	});

	test('shows guided firmware workflow in editor', async ({ page }) => {
		await page.goto('/layouts/test-layout');

		await page.getByTestId('tab-firmware').click();
		await expect(page.getByTestId('firmware-workflow-summary')).toBeVisible();
		await expect(page.getByTestId('firmware-step-generate')).toContainText('Generate sources');
		await expect(page.getByTestId('firmware-step-build')).toContainText('Build firmware');
		await expect(page.getByText(/Run source generation first, then compile flashable firmware artifacts/i)).toBeVisible();
	});

	test('shows build step controls inside layout workspace', async ({ page }) => {
		await openBuildStep(page);

		await expect(page.getByRole('heading', { name: 'Step 2: Build firmware' })).toBeVisible();
		await expect(page.getByTestId('start-build-button')).toBeVisible();
		await expect(page.getByTestId('build-empty-state')).toBeVisible();
		await expect(page.getByTestId('build-history-card')).toBeVisible();
	});

	test('returns to generate step from build step', async ({ page }) => {
		await openBuildStep(page);

		await page.getByTestId('back-to-generate-button').click();
		await expect(page.getByRole('heading', { name: 'Step 1: Generate firmware sources' })).toBeVisible();
	});
});

test.describe('Build history and logs', () => {
	test.beforeEach(async ({ page }) => {
		await mockEditor(page);

		await page.route('**/api/build/jobs', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify([mockRunningJob])
			});
		});

		await page.route('**/api/build/jobs/job-123/logs**', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockLogs)
			});
		});
	});

	test('shows previous build in history', async ({ page }) => {
		await openBuildStep(page);

		await expect(page.getByTestId('history-row')).toBeVisible();
		await expect(page.getByText('crkbd')).toBeVisible();
	});

	test('can inspect build logs from history row', async ({ page }) => {
		await openBuildStep(page);

		await page.getByTestId('view-job-button').click();
		await expect(page.getByTestId('build-logs-card')).toBeVisible();
		await expect(page.getByTestId('build-logs')).toContainText('Build started');
		await expect(page.getByTestId('cancel-build-button')).toBeVisible();
	});
});

test.describe('Start and complete builds from workflow', () => {
	test.beforeEach(async ({ page }) => {
		await mockEditor(page);

		await page.route('**/api/build/start', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({ job: mockBuildJob })
			});
		});

		await page.route('**/api/build/jobs/job-123/logs**', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockLogs)
			});
		});
	});

	test('can start build from build step', async ({ page }) => {
		await openBuildStep(page);

		await page.getByTestId('start-build-button').click();
		await expect(page.getByTestId('build-status')).toBeVisible();
		await expect(page.getByTestId('build-logs-card')).toBeVisible();
	});

	test('shows artifacts for completed build', async ({ page }) => {
		await page.route('**/api/build/jobs', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify([mockCompletedJob])
			});
		});

		await page.route('**/api/build/jobs/job-123/artifacts', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify(mockArtifacts)
			});
		});

		await page.route('**/api/build/jobs/job-123/logs**', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({
					job_id: 'job-123',
					logs: [{ timestamp: '2024-01-01T12:01:00Z', level: 'INFO', message: 'Firmware built: /tmp/crkbd_default.uf2' }],
					has_more: false
				})
			});
		});

		await openBuildStep(page);
		await page.getByTestId('view-job-button').click();

		await expect(page.getByTestId('build-artifacts-card')).toBeVisible();
		await expect(page.getByTestId('artifact-row')).toContainText('firmware.uf2');
		await expect(page.getByTestId('cancel-build-button')).toHaveCount(0);
	});
});

test.describe('Build navigation acceptance', () => {
	test.beforeEach(async ({ page }) => {
		await page.route('**/api/preflight', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({
					qmk_configured: true,
					has_layouts: true,
					first_run: false,
					qmk_firmware_path: '/path/to/qmk_firmware'
				})
			});
		});

		await page.route('**/api/layouts', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({ layouts: [] })
			});
		});
	});

	test('home page keeps global build out of nav', async ({ page }) => {
		await page.goto('/');

		await expect(page.getByRole('button', { name: 'More' })).toHaveCount(0);
		await expect(page.getByRole('link', { name: 'Build' })).toHaveCount(0);
	});
});
