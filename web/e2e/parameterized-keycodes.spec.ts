import { test, expect } from '@playwright/test';

test.describe('Parameterized Keycode API', () => {
	test('LT() surfaces its layer + keycode params via the keycodes endpoint', async ({ request }) => {
		const body = await (
			await request.get('http://localhost:3031/api/keycodes?search=LT()')
		).json();
		const lt = body.keycodes.find((k: { code: string }) => k.code === 'LT()');
		expect(lt).toBeTruthy();
		expect(lt.parameterized).toBe(true);
		expect(lt.params).toEqual([
			{ type: 'layer', name: 'layer', description: expect.any(String) },
			{ type: 'keycode', name: 'keycode', description: expect.any(String) }
		]);
	});

	test('MO() surfaces single layer param', async ({ request }) => {
		const body = await (
			await request.get('http://localhost:3031/api/keycodes?search=MO()')
		).json();
		const mo = body.keycodes.find((k: { code: string }) => k.code === 'MO()');
		expect(mo).toBeTruthy();
		expect(mo.parameterized).toBe(true);
		expect(mo.params.length).toBe(1);
		expect(mo.params[0].type).toBe('layer');
	});

	test('LCG() surfaces keycode-only param (modifier is in the prefix)', async ({ request }) => {
		const body = await (
			await request.get('http://localhost:3031/api/keycodes?search=LCG()')
		).json();
		const lcg = body.keycodes.find((k: { code: string }) => k.code === 'LCG()');
		expect(lcg).toBeTruthy();
		expect(lcg.parameterized).toBe(true);
		expect(lcg.params.map((p: { type: string }) => p.type)).toEqual(['keycode']);
	});

	test('LM() surfaces layer + modifier params', async ({ request }) => {
		const body = await (
			await request.get('http://localhost:3031/api/keycodes?search=LM()')
		).json();
		const lm = body.keycodes.find((k: { code: string }) => k.code === 'LM()');
		expect(lm).toBeTruthy();
		expect(lm.params.map((p: { type: string }) => p.type)).toEqual(['layer', 'modifier']);
	});

	test('non-parameterized keycodes do not expose params field', async ({ request }) => {
		const body = await (
			await request.get('http://localhost:3031/api/keycodes?search=KC_A&category=basic')
		).json();
		const kcA = body.keycodes.find((k: { code: string }) => k.code === 'KC_A');
		expect(kcA).toBeTruthy();
		expect(kcA.parameterized).toBeUndefined();
		expect(kcA.params).toBeUndefined();
	});
});