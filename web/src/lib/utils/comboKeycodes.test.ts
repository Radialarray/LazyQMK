import { describe, it, expect } from 'vitest';
import { parseComboKeycode, reassembleCombo, describeHold } from './comboKeycodes';

describe('parseComboKeycode', () => {
	it('parses LT(layer, key) as layer-tap', () => {
		const result = parseComboKeycode('LT(2, KC_SPC)');
		expect(result).toEqual({
			kind: 'layer-tap',
			prefix: 'LT',
			hold: '2',
			tap: 'KC_SPC',
			holdIsLayer: true,
			holdIsModifier: false
		});
	});

	it('parses LT with UUID reference', () => {
		const result = parseComboKeycode(
			'LT(@f85996a8-8dbd-403d-a804-fac1f2bc751d, KC_R)'
		);
		expect(result?.kind).toBe('layer-tap');
		expect(result?.hold).toBe('@f85996a8-8dbd-403d-a804-fac1f2bc751d');
		expect(result?.tap).toBe('KC_R');
		expect(result?.holdIsLayer).toBe(true);
	});

	it('parses MT(mod, key) as mod-tap', () => {
		const result = parseComboKeycode('MT(MOD_LCTL, KC_A)');
		expect(result?.kind).toBe('mod-tap');
		expect(result?.hold).toBe('MOD_LCTL');
		expect(result?.tap).toBe('KC_A');
		expect(result?.holdIsModifier).toBe(true);
	});

	it('parses LCTL_T(key) as mod-tap with prefix carry', () => {
		const result = parseComboKeycode('LCTL_T(KC_A)');
		expect(result?.kind).toBe('mod-tap');
		expect(result?.hold).toBe('LCTL_T');
		expect(result?.tap).toBe('KC_A');
	});

	it('parses LCG(key) as mod-combo', () => {
		const result = parseComboKeycode('LCG(KC_B)');
		expect(result?.kind).toBe('mod-combo');
		expect(result?.prefix).toBe('LCG');
		expect(result?.tap).toBe('KC_B');
	});

	it('parses LM(layer, mod) as layer-mod', () => {
		const result = parseComboKeycode('LM(1, MOD_LSFT)');
		expect(result?.kind).toBe('layer-mod');
		expect(result?.hold).toBe('1');
		expect(result?.tap).toBe('MOD_LSFT');
		expect(result?.holdIsLayer).toBe(true);
		expect(result?.holdIsModifier).toBe(true);
	});

	it('parses TD(name) as tap-dance', () => {
		const result = parseComboKeycode('TD(quote_tap)');
		expect(result?.kind).toBe('tap-dance');
		expect(result?.hold).toBe('quote_tap');
	});

	it('returns null for plain keycodes', () => {
		expect(parseComboKeycode('KC_A')).toBeNull();
		expect(parseComboKeycode('KC_TRNS')).toBeNull();
		// MO/TG/TO layer-switching keycodes are not combos (single layer arg)
		expect(parseComboKeycode('MO(1)')).toBeNull();
	});

	it('handles nested parentheses in the tap arg', () => {
		const result = parseComboKeycode('LT(2, LCTL_T(KC_A))');
		expect(result?.kind).toBe('layer-tap');
		expect(result?.hold).toBe('2');
		expect(result?.tap).toBe('LCTL_T(KC_A)');
	});
});

describe('reassembleCombo', () => {
	it('preserves the unmodified side', () => {
		const combo = parseComboKeycode('LT(2, KC_SPC)');
		expect(combo).not.toBeNull();
		const rebuilt = reassembleCombo(combo!, { tap: 'KC_A' });
		expect(rebuilt).toBe('LT(2, KC_A)');
	});

	it('preserves the unmodified hold', () => {
		const combo = parseComboKeycode('MT(MOD_LCTL, KC_A)');
		const rebuilt = reassembleCombo(combo!, { hold: 'MOD_LALT' });
		expect(rebuilt).toBe('MT(MOD_LALT, KC_A)');
	});

	it('keeps named mod-taps single-argument', () => {
		const combo = parseComboKeycode('LCTL_T(KC_A)');
		expect(reassembleCombo(combo!, { tap: 'KC_B' })).toBe('LCTL_T(KC_B)');
		// Editing the hold swaps the prefix, since that is where the modifier lives.
		expect(reassembleCombo(combo!, { hold: 'LSFT_T' })).toBe('LSFT_T(KC_A)');
	});

	it('keeps modifier combos single-argument', () => {
		const combo = parseComboKeycode('LCG(KC_B)');
		expect(reassembleCombo(combo!, { tap: 'KC_C' })).toBe('LCG(KC_C)');
		expect(reassembleCombo(combo!, { hold: 'MEH' })).toBe('MEH(KC_B)');
	});

	it('keeps tap dances single-argument', () => {
		const combo = parseComboKeycode('TD(quote_tap)');
		expect(reassembleCombo(combo!, { hold: 'other_tap' })).toBe('TD(other_tap)');
	});
});

describe('describeHold', () => {
	it('resolves UUID layer references to their layer number', () => {
		const combo = parseComboKeycode('LT(@6f9af2ee-80b1-417c-addc-5eadc2ad95ad, DE_Z)');
		const resolve = (ref: string) => (ref === '@6f9af2ee-80b1-417c-addc-5eadc2ad95ad' ? '5' : null);
		expect(describeHold(combo!, resolve)).toBe('Layer 5');
	});

	it('falls back to a short UUID fragment when unresolvable', () => {
		const combo = parseComboKeycode('LT(@6f9af2ee-80b1-417c-addc-5eadc2ad95ad, DE_Z)');
		expect(describeHold(combo!, () => null)).toBe('Layer 6f9af2ee');
	});

	it('passes numeric layer references through', () => {
		const combo = parseComboKeycode('LT(2, KC_SPC)');
		expect(describeHold(combo!)).toBe('Layer 2');
	});
});