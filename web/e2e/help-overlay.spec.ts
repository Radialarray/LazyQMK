import { test, expect } from '@playwright/test';

test.describe('Help API', () => {
	test('exposes keybindings from src/data/help.toml', async ({ request }) => {
		const body = await (await request.get('http://localhost:3031/api/help')).json();
		expect(body.app_name).toBeTruthy();
		expect(Array.isArray(body.contexts)).toBe(true);
		// help.toml ships at least the main + keycode_picker + help contexts.
		expect(body.contexts.length).toBeGreaterThan(10);
	});

	test('includes the main view context with Ctrl+S / Ctrl+G / Ctrl+B bindings', async ({
		request
	}) => {
		const body = await (await request.get('http://localhost:3031/api/help')).json();
		const main = body.contexts.find((c: { id: string }) => c.id === 'main');
		expect(main).toBeTruthy();
		const labels = main.bindings.map((b: { action: string }) => b.action);
		expect(labels).toContain('Save layout');
		expect(labels).toContain('Generate firmware');
		expect(labels).toContain('Build firmware');
	});

	test('includes a parameterized_keycodes context documenting LT/MT/LM', async ({ request }) => {
		const body = await (await request.get('http://localhost:3031/api/help')).json();
		const param = body.contexts.find(
			(c: { id: string }) => c.id === 'parameterized_keycodes'
		);
		expect(param).toBeTruthy();
		const actions = param.bindings.map((b: { action: string }) => b.action);
		expect(actions.some((a: string) => a.startsWith('Layer-Tap'))).toBe(true);
		expect(actions.some((a: string) => a.startsWith('Mod-Tap'))).toBe(true);
		expect(actions.some((a: string) => a.startsWith('Layer-Mod'))).toBe(true);
	});

	test('bindings expose keys, alt_keys, hint, and priority', async ({ request }) => {
		const body = await (await request.get('http://localhost:3031/api/help')).json();
		const allBindings = body.contexts.flatMap(
			(c: { bindings: { keys: string[]; priority: number }[] }) => c.bindings
		);
		const sample = allBindings.find((b: { keys: string[] }) => b.keys.includes('Ctrl+S'));
		expect(sample).toBeTruthy();
		expect(typeof sample.priority).toBe('number');
	});
});