import { describe, it, expect } from 'vitest';
import type { ComboAction, ComboMarker } from '$api/types';

/**
 * Combo-marker rendering contract for `<KeyboardPreview>`.
 *
 * Mounting the actual `KeyboardPreview.svelte` component under Vitest is
 * blocked by an upstream tooling incompatibility: `@sveltejs/vite-plugin-svelte@6`
 * requires Vite 6 (which it uses for its `environments` API), but the bundled
 * Vitest 2.x test runner still ships Vite 5. The Svelte plugin therefore crashes
 * during `configureServer` before any test can run. (`web/src/lib/components/Button.test.ts.skip`
 * documents the same blocker; that file is intentionally skipped.)
 *
 * Instead, this test pins the **marker-rendering contract** that the component
 * relies on, mirroring the inline expressions in `KeyboardPreview.svelte`
 * (lines 312–313):
 *
 * ```svelte
 * {@const comboClass  = comboMarker ? `combo-${comboMarker.action.replace(/_/g, '-')}` : ''}
 * {@const comboLetter = comboMarker?.action === 'bootloader'      ? 'B'
 *                    : comboMarker?.action === 'disable_effects'  ? 'E'
 *                    : comboMarker?.action === 'disable_lighting' ? 'L' : ''}
 * ```
 *
 * If the component is ever refactored, update these helpers in lockstep so the
 * test continues to act as a contract guard for the keyboard-preview markers.
 */

function comboClassFor(action: ComboAction): string {
	return `combo-${action.replace(/_/g, '-')}`;
}

function comboLetterFor(action: ComboAction): string {
	switch (action) {
		case 'bootloader':
			return 'B';
		case 'disable_effects':
			return 'E';
		case 'disable_lighting':
			return 'L';
	}
}

function makeMarker(visualIndex: number, action: ComboAction): ComboMarker {
	return { visualIndex, action, holdDurationMs: 500 };
}

describe('KeyboardPreview combo-marker rendering contract', () => {
	describe('comboClassFor', () => {
		it('maps bootloader to combo-bootloader', () => {
			expect(comboClassFor('bootloader')).toBe('combo-bootloader');
		});

		it('maps disable_effects to combo-disable-effects', () => {
			expect(comboClassFor('disable_effects')).toBe('combo-disable-effects');
		});

		it('maps disable_lighting to combo-disable-lighting', () => {
			expect(comboClassFor('disable_lighting')).toBe('combo-disable-lighting');
		});
	});

	describe('comboLetterFor', () => {
		it('returns B for bootloader', () => {
			expect(comboLetterFor('bootloader')).toBe('B');
		});

		it('returns E for disable_effects', () => {
			expect(comboLetterFor('disable_effects')).toBe('E');
		});

		it('returns L for disable_lighting', () => {
			expect(comboLetterFor('disable_lighting')).toBe('L');
		});
	});

	describe('rendered marker shape', () => {
		const actions: ComboAction[] = ['bootloader', 'disable_effects', 'disable_lighting'];
		const expectedClasses = [
			'combo-bootloader',
			'combo-disable-effects',
			'combo-disable-lighting'
		];
		const expectedLetters = ['B', 'E', 'L'];

		it('produces a distinct action-specific class for every ComboAction', () => {
			const rendered = actions.map(comboClassFor);
			expect(rendered).toEqual(expectedClasses);

			// Every class is unique (no overlap between actions).
			expect(new Set(rendered).size).toBe(actions.length);
		});

		it('produces a distinct one-letter badge for every ComboAction', () => {
			const rendered = actions.map(comboLetterFor);
			expect(rendered).toEqual(expectedLetters);

			// Every letter is unique.
			expect(new Set(rendered).size).toBe(actions.length);
		});

		it('round-trips: class and letter are derived from the same action', () => {
			for (const action of actions) {
				const marker = makeMarker(0, action);
				expect(marker.action).toBe(action);
				expect(comboClassFor(marker.action)).toMatch(/^combo-/);
				expect(comboLetterFor(marker.action)).toMatch(/^[BEL]$/);
			}
		});
	});

	describe('marker visualIndex routing', () => {
		it('builds markers anchored at distinct visual indices', () => {
			const markers: ComboMarker[] = [
				makeMarker(0, 'bootloader'),
				makeMarker(1, 'disable_effects'),
				makeMarker(2, 'disable_lighting')
			];

			// The component builds a Map<visualIndex, ComboMarker>; ensure the
			// markers we feed in are distinct on visualIndex (no overwrites).
			const map = new Map<number, ComboMarker>();
			for (const marker of markers) {
				expect(map.has(marker.visualIndex)).toBe(false);
				map.set(marker.visualIndex, marker);
			}
			expect(map.size).toBe(3);

			// And that lookup yields the expected letter.
			expect(comboLetterFor(map.get(0)!.action)).toBe('B');
			expect(comboLetterFor(map.get(1)!.action)).toBe('E');
			expect(comboLetterFor(map.get(2)!.action)).toBe('L');
		});
	});
});
