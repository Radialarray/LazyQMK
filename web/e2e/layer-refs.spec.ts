import { test, expect } from '@playwright/test';

const BASE_URL = process.env.LAYOUT_REFS_BASE_URL ?? 'http://localhost:3031';
const LAYOUT = 'corne_choc_pro_enhanced';

test.describe('Layer References API', () => {
	test('returns inbound refs and warnings for a real layout', async ({ request }) => {
		const response = await request.get(`${BASE_URL}/api/layouts/${LAYOUT}/layer-refs`);
		expect(response.ok()).toBeTruthy();
		const body = await response.json();
		expect(body).toHaveProperty('layers');
		expect(body).toHaveProperty('total_inbound_refs');
		expect(body).toHaveProperty('total_warnings');
		expect(Array.isArray(body.layers)).toBe(true);
		expect(body.layers.length).toBeGreaterThan(0);
	});

	test('resolves UUID-based layer targets against the layer list', async ({ request }) => {
		const body = await (
			await request.get(`${BASE_URL}/api/layouts/${LAYOUT}/layer-refs`)
		).json();

		const uuidRef = body.layers
			.flatMap((l: { inbound_refs: { keycode: string }[] }) => l.inbound_refs)
			.find((r: { keycode: string }) => r.keycode.includes('@'));

		// corne_choc_pro_enhanced uses LT(@uuid, ...) keys, so at least one
		// inbound reference should resolve through the UUID lookup.
		expect(uuidRef).toBeTruthy();
	});

	test('emits transparency warnings when a hold-like ref lands on a non-TRNS slot', async ({
		request
	}) => {
		const body = await (
			await request.get(`${BASE_URL}/api/layouts/${LAYOUT}/layer-refs`)
		).json();

		// The known bug on corne_choc_pro_enhanced: LT(@symbols, KC_R) lands on
		// a DE_ACUT slot on the Symbols layer, which produces a transparency
		// warning. This was the user's original report.
		expect(body.total_warnings).toBeGreaterThan(0);
		const warning = body.layers
			.flatMap((l: { warnings: { target_keycode: string }[] }) => l.warnings)
			.find((w: { target_keycode: string }) => w.target_keycode === 'DE_ACUT');
		expect(warning).toBeTruthy();
	});
});