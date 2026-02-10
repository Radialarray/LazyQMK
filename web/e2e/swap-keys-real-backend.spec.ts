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

const LAYOUT_MARKDOWN = `---
name: Swap Test
description: Test layout for swap
author: Test User
created: 2024-01-01T00:00:00Z
modified: 2024-01-01T00:00:00Z
tags: []
is_template: false
version: '1.0'
keyboard: test_keyboard
layout_variant: LAYOUT_test
keymap_name: test
output_format: hex
---

# Swap Test

## Layer 0: Base
**ID**: test-layer
**Color**: #FFFFFF

| C0 | C1 |
|------|------|
| KC_Q{#FF0000} | KC_W{#00FF00} |
`;

test.describe('Swap Keys Mode - real backend', () => {
	let workspaceRoot: string;
	let backendProcess: Awaited<ReturnType<typeof startBackend>>['process'];
	let backendPort: number;
	const layoutFilename = 'swap_real_backend';
	const layoutFile = `${layoutFilename}.md`;

	test.beforeAll(async () => {
		workspaceRoot = await mkdtemp(join(tmpdir(), 'lazyqmk-e2e-'));
		backendPort = 3101 + Math.floor(Math.random() * 500);
		const qmkRoot = join(workspaceRoot, 'qmk_firmware');
		const keyboardDir = join(qmkRoot, 'keyboards', 'test_keyboard');
		await mkdir(keyboardDir, { recursive: true });
		await writeFile(join(qmkRoot, 'Makefile'), '# Minimal QMK Makefile for tests\n');
		await writeFile(join(keyboardDir, 'info.json'), JSON.stringify(QMK_INFO_JSON, null, 2));
		await writeFile(join(workspaceRoot, layoutFile), LAYOUT_MARKDOWN);
	});

	test.afterAll(async () => {
		await stopBackend(backendProcess);
		if (workspaceRoot) {
			await rm(workspaceRoot, { recursive: true, force: true });
		}
	});

	test('swap persists in UI and markdown file', async ({ page }) => {
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
		await page.goto(`http://localhost:4173/layouts/${layoutFilename}`);

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

		const savedLayout = await readFile(join(workspaceRoot, layoutFile), 'utf8');
		expect(savedLayout).toContain('KC_W{#00FF00} | KC_Q{#FF0000}');
	});
});
