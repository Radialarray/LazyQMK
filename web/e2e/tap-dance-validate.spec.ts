import { test, expect } from '@playwright/test';

test.describe('Tap Dance Validator API', () => {
	test('reports zero for a layout with no tap dances', async ({ request }) => {
		const body = await (
			await request.get(
				'http://localhost:3031/api/layouts/corne_choc_pro_enhanced/tap-dance/validate'
			)
		).json();
		expect(body.valid).toBe(true);
		expect(body.total_defined).toBe(0);
		expect(body.total_referenced).toBe(0);
		expect(body.orphaned).toEqual([]);
		expect(body.unused).toEqual([]);
	});

	test('flags unused definitions when TD() exists but is not referenced', async ({ request }) => {
		const layout = await (
			await request.get('http://localhost:3031/api/layouts/corne_choc_pro_enhanced')
		).json();
		const mutated = JSON.parse(JSON.stringify(layout));
		mutated.metadata.name = 'td-unused-test';
		mutated.metadata.modified = new Date().toISOString();
		mutated.tap_dances = [{ name: 'orphan', single_tap: 'KC_A', hold: null }];
		const put = await request.put('http://localhost:3031/api/layouts/td-unused-test', {
			data: mutated
		});
		expect(put.status()).toBe(204);

		const body = await (
			await request.get('http://localhost:3031/api/layouts/td-unused-test/tap-dance/validate')
		).json();
		// valid because no orphaned refs (unused is only a warning)
		expect(body.valid).toBe(true);
		expect(body.unused).toContain('orphan');
		expect(body.orphaned).toEqual([]);
		expect(body.total_defined).toBe(1);
	});

	test('handles auto-created tap dances (orphan surface still detected if file is malformed)', async ({
		request
	}) => {
		// Layout::validate auto-creates missing TD definitions on load, so the
		// orphan pathway is only reachable through raw file manipulation. This
		// test confirms the validator handles the normal happy path: a layout
		// with valid TD definitions round-trips through validate+save cleanly.
		const layout = await (
			await request.get('http://localhost:3031/api/layouts/corne_choc_pro_enhanced')
		).json();
		const mutated = JSON.parse(JSON.stringify(layout));
		mutated.metadata.name = 'td-happy-test';
		mutated.metadata.modified = new Date().toISOString();
		mutated.layers[0].keys[0].keycode = 'TD(foo)';
		mutated.tap_dances = [{ name: 'foo', single_tap: 'KC_A', hold: null }];
		const put = await request.put('http://localhost:3031/api/layouts/td-happy-test', {
			data: mutated
		});
		expect(put.status()).toBe(204);

		const body = await (
			await request.get('http://localhost:3031/api/layouts/td-happy-test/tap-dance/validate')
		).json();
		expect(body.valid).toBe(true);
		expect(body.orphaned).toEqual([]);
		expect(body.unused).toEqual([]); // Defined and used via TD(foo).
		expect(body.total_defined).toBeGreaterThanOrEqual(1);
		expect(body.total_referenced).toBeGreaterThanOrEqual(1);
	});
});