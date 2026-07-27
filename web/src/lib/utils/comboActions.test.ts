import { describe, it, expect } from 'vitest';
import { normaliseComboAction } from './comboActions';

describe('normaliseComboAction', () => {
	it('passes through the flat-string shape unchanged', () => {
		expect(normaliseComboAction('bootloader')).toBe('bootloader');
		expect(normaliseComboAction('disable_effects')).toBe('disable_effects');
		expect(normaliseComboAction('disable_lighting')).toBe('disable_lighting');
	});

	it('unwraps the internally-tagged enum shape from the backend', () => {
		expect(normaliseComboAction({ type: 'bootloader' })).toBe('bootloader');
		expect(normaliseComboAction({ type: 'disable_effects' })).toBe('disable_effects');
		expect(normaliseComboAction({ type: 'disable_lighting' })).toBe('disable_lighting');
	});

	it('falls back to disable_effects for unknown string values', () => {
		expect(normaliseComboAction('explode')).toBe('disable_effects');
		expect(normaliseComboAction('')).toBe('disable_effects');
	});

	it('falls back to disable_effects for malformed objects', () => {
		expect(normaliseComboAction({ type: 'unknown' })).toBe('disable_effects');
		expect(normaliseComboAction({})).toBe('disable_effects');
		expect(normaliseComboAction(null)).toBe('disable_effects');
		expect(normaliseComboAction(undefined)).toBe('disable_effects');
		expect(normaliseComboAction(42)).toBe('disable_effects');
	});
});
