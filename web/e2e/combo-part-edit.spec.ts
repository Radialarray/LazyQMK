import { test, expect } from '@playwright/test';

const BASE_URL = process.env.LAYOUT_REFS_BASE_URL ?? 'http://localhost:3031';
const LAYOUT = 'corne_choc_pro_enhanced';

test.describe('Combo Part Edit (frontend parser)', () => {
	test('exposes parameterized keycodes that can be split-edited', async ({ request }) => {
		const body = await (
			await request.get(`${BASE_URL}/api/keycodes?search=LT`)
		).json();
		const lt = body.keycodes.find((k: { code: string }) => k.code === 'LT()');
		expect(lt).toBeTruthy();
		expect(lt.parameterized).toBe(true);
		expect(lt.params.map((p: { type: string }) => p.type)).toEqual(['layer', 'keycode']);
	});

	test('LT() on the layer picker triggers a 2-step chain (layer, keycode)', async ({
		request
	}) => {
		// Validate the API contract the WebUI chain logic depends on:
		// the first param of LT() is layer, the second is keycode.
		// The +page.svelte chain handler relies on this ordering.
		const body = await (
			await request.get(`${BASE_URL}/api/keycodes?search=LT()`)
		).json();
		const lt = body.keycodes.find((k: { code: string }) => k.code === 'LT()');
		expect(lt.params[0].type).toBe('layer');
		expect(lt.params[1].type).toBe('keycode');
	});
});