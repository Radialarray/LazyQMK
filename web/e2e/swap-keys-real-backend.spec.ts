import { test, expect } from '@playwright/test';
import { mkdtemp, rm, writeFile, readFile, mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { startBackend, stopBackend } from './helpers/backend';

const QMK_INFO_JSON = {
	keyboard_name: 'test_keyboard',
	manufacturer: 'Test',
	maintainer: 'test',
	processor: 'atmega32u4',
	bootloader: 'atmel-dfu',
	usb: {
		vid: '0xFEED',
		pid: '0x0000',
		device_version: '1.0.0'
	},
	matrix_pins: {
		cols: ['F0', 'F1'],
		rows: ['D0']
	},
	diode_direction: 'COL2ROW',
	layouts: {
		LAYOUT_test: {
			layout: [
				{ matrix: [0, 0], x: 0, y: 0 },
				{ matrix: [0, 1], x: 1, y: 0 }
			]
		}
	}
};

const LAYOUT_JSON = {
	metadata: {
		name: 'Swap Test',
		description: 'Test layout for swap',
		author: 'Test User',
		created: '2024-01-01T00:00:00Z',
		modified: '2024-01-01T00:00:00Z',
		tags: [],
		is_template: false,
		version: '1.0',
		keyboard: 'test_keyboard',
		layout_variant: 'LAYOUT_test',
		keymap_name: 'test',
		output_format: 'hex'
	},
	layers: [
		{
			id: 'test-layer',
			number: 0,
			name: 'Base',
			default_color: { r: 255, g: 255, b: 255 },
			category_id: null,
			keys: [
				{
					position: { row: 0, col: 0 },
					keycode: 'KC_Q',
					label: null,
					color_override: { r: 255, g: 0, b: 0 },
					category_id: null,
					combo_participant: false
				},
				{
					position: { row: 0, col: 1 },
					keycode: 'KC_W',
					label: null,
					color_override: { r: 0, g: 255, b: 0 },
					category_id: null,
					combo_participant: false
				}
			],
			layer_colors_enabled: true
		}
	],
	categories: []
};

test.describe('Swap Keys Mode - real backend', () => {
	let workspaceRoot: string;
	let backendProcess: Awaited<ReturnType<typeof startBackend>>['process'];
	let backendPort: number;
	const layoutFilename = 'swap_real_backend';
	const layoutFile = join(layoutFilename, 'current.json');

	test.beforeAll(async () => {
		workspaceRoot = await mkdtemp(join(tmpdir(), 'lazyqmk-e2e-'));
		backendPort = 3101 + Math.floor(Math.random() * 500);
		const qmkRoot = join(workspaceRoot, 'qmk_firmware');
		const keyboardDir = join(qmkRoot, 'keyboards', 'test_keyboard');
		await mkdir(keyboardDir, { recursive: true });
		await writeFile(join(qmkRoot, 'Makefile'), '# Minimal QMK Makefile for tests\n');
		await writeFile(join(keyboardDir, 'info.json'), JSON.stringify(QMK_INFO_JSON, null, 2));
		await mkdir(join(workspaceRoot, layoutFilename), { recursive: true });
		await writeFile(join(workspaceRoot, layoutFile), JSON.stringify(LAYOUT_JSON));
	});

	test.afterAll(async () => {
		await stopBackend(backendProcess);
		if (workspaceRoot) {
			await rm(workspaceRoot, { recursive: true, force: true });
		}
	});

	test('swap persists in the UI and canonical layout JSON', async ({ page }) => {
		const apiBaseUrl = `http://127.0.0.1:${backendPort}`;
		await page.addInitScript(({ baseUrl }) => {
			(window as { __LAZYQMK_API_BASE_URL?: string }).__LAZYQMK_API_BASE_URL = baseUrl;
		}, { baseUrl: apiBaseUrl });
		const backend = await startBackend({
			workspaceRoot,
			configDir: workspaceRoot,
			port: backendPort
		});
		backendProcess = backend.process;
		const configResponse = await page.request.put(`${apiBaseUrl}/api/config`, {
			data: {
				qmk_firmware_path: join(workspaceRoot, 'qmk_firmware')
			}
		});
		expect(configResponse.ok()).toBe(true);
		const preflightResponse = await page.request.get(`${apiBaseUrl}/api/preflight`);
		expect(preflightResponse.ok()).toBe(true);
		const preflightData = await preflightResponse.json();
		expect(preflightData.qmk_configured).toBe(true);
		await page.goto(`${apiBaseUrl}/layouts/${layoutFilename}`);

		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.getByTestId('key-0')).toBeVisible();
		await expect(page.getByTestId('key-1')).toBeVisible();

		const getKeyLabel = async (visualIndex: number) => {
			const keyText = page
				.locator(`[data-testid="key-${visualIndex}"] text`)
				.first();
			await expect(keyText).toBeVisible();
			return keyText.textContent();
		};

		expect(await getKeyLabel(0)).toContain('Q');
		expect(await getKeyLabel(1)).toContain('W');

		await page.locator('.keyboard-preview').click();
		await page.keyboard.press('Shift+W');
		await expect(page.getByText('Swap mode - click first key to swap')).toBeVisible();

		await page.getByTestId('key-0').dispatchEvent('click');
		await expect(page.getByText('Swap mode - click second key to swap')).toBeVisible();
		await page.getByTestId('key-1').dispatchEvent('click');
		await expect(page.getByText('Keys swapped')).toBeVisible();

		expect(await getKeyLabel(0)).toContain('W');
		expect(await getKeyLabel(1)).toContain('Q');

		await page.reload();
		await expect(page.getByRole('heading', { name: 'Keyboard Preview' })).toBeVisible();
		await expect(page.getByTestId('key-0')).toBeVisible();
		await expect(page.getByTestId('key-1')).toBeVisible();

		expect(await getKeyLabel(0)).toContain('W');
		expect(await getKeyLabel(1)).toContain('Q');

		const savedLayout = JSON.parse(await readFile(join(workspaceRoot, layoutFile), 'utf8'));
		expect(savedLayout.layers[0].keys).toEqual([
			expect.objectContaining({
				position: { row: 0, col: 0 },
				keycode: 'KC_W',
				color_override: { r: 0, g: 255, b: 0 }
			}),
			expect.objectContaining({
				position: { row: 0, col: 1 },
				keycode: 'KC_Q',
				color_override: { r: 255, g: 0, b: 0 }
			})
		]);
	});
});
