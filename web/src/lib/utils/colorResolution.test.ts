import { describe, it, expect } from 'vitest';
import { resolveKeyColor, rgbToHex, hexToRgb, isValidHex } from './colorResolution';
import type { RgbColor, KeyAssignment, Layer, Category } from '$api/types';

describe('colorResolution', () => {
	const redRgb: RgbColor = { r: 255, g: 0, b: 0 };
	const greenRgb: RgbColor = { r: 0, g: 255, b: 0 };
	const blueRgb: RgbColor = { r: 0, g: 0, b: 255 };
	const yellowRgb: RgbColor = { r: 255, g: 255, b: 0 };

	const navigationCategory: Category = {
		id: 'navigation',
		name: 'Navigation',
		color: greenRgb
	};

	const symbolsCategory: Category = {
		id: 'symbols',
		name: 'Symbols',
		color: blueRgb
	};

	const categories: Category[] = [navigationCategory, symbolsCategory];

	describe('resolveKeyColor', () => {
		it('should return key color override (highest priority)', () => {
			const key: KeyAssignment = {
				keycode: 'KC_A',
				matrix_position: [0, 0],
				visual_index: 0,
				led_index: 0,
				color_override: redRgb,
				category_id: 'navigation'
			};

			const layer: Layer = {
				name: 'Base',
				color: '#FFFFFF',
				default_color: yellowRgb,
				category_id: 'symbols',
				layer_colors_enabled: true,
				keys: [key]
			};

			const result = resolveKeyColor(key, layer, categories);
			expect(result).toBe('#FF0000'); // Red from key override
		});

		it('should return key category color when no override', () => {
			const key: KeyAssignment = {
				keycode: 'KC_A',
				matrix_position: [0, 0],
				visual_index: 0,
				led_index: 0,
				category_id: 'navigation'
			};

			const layer: Layer = {
				name: 'Base',
				color: '#FFFFFF',
				default_color: yellowRgb,
				category_id: 'symbols',
				layer_colors_enabled: true,
				keys: [key]
			};

			const result = resolveKeyColor(key, layer, categories);
			expect(result).toBe('#00FF00'); // Green from navigation category
		});

		it('should return layer category color when no key override/category', () => {
			const key: KeyAssignment = {
				keycode: 'KC_A',
				matrix_position: [0, 0],
				visual_index: 0,
				led_index: 0
			};

			const layer: Layer = {
				name: 'Base',
				color: '#FFFFFF',
				default_color: yellowRgb,
				category_id: 'symbols',
				layer_colors_enabled: true,
				keys: [key]
			};

			const result = resolveKeyColor(key, layer, categories);
			expect(result).toBe('#0000FF'); // Blue from symbols category
		});

		it('should return layer default color when no other colors', () => {
			const key: KeyAssignment = {
				keycode: 'KC_A',
				matrix_position: [0, 0],
				visual_index: 0,
				led_index: 0
			};

			const layer: Layer = {
				name: 'Base',
				color: '#FFFFFF',
				default_color: yellowRgb,
				layer_colors_enabled: true,
				keys: [key]
			};

			const result = resolveKeyColor(key, layer, categories);
			expect(result).toBe('#FFFF00'); // Yellow from layer default
		});

		it('should return undefined when no colors available', () => {
			const key: KeyAssignment = {
				keycode: 'KC_A',
				matrix_position: [0, 0],
				visual_index: 0,
				led_index: 0
			};

			const layer: Layer = {
				name: 'Base',
				color: '#FFFFFF',
				keys: [key]
			};

			const result = resolveKeyColor(key, layer, []);
			expect(result).toBeUndefined();
		});

		it('should skip layer colors when layer_colors_enabled is false', () => {
			const key: KeyAssignment = {
				keycode: 'KC_A',
				matrix_position: [0, 0],
				visual_index: 0,
				led_index: 0
			};

			const layer: Layer = {
				name: 'Base',
				color: '#FFFFFF',
				default_color: yellowRgb,
				category_id: 'symbols',
				layer_colors_enabled: false,
				keys: [key]
			};

			const result = resolveKeyColor(key, layer, categories);
			expect(result).toBeUndefined(); // Layer colors disabled
		});

		it('should still use key colors when layer_colors_enabled is false', () => {
			const key: KeyAssignment = {
				keycode: 'KC_A',
				matrix_position: [0, 0],
				visual_index: 0,
				led_index: 0,
				color_override: redRgb
			};

			const layer: Layer = {
				name: 'Base',
				color: '#FFFFFF',
				default_color: yellowRgb,
				layer_colors_enabled: false,
				keys: [key]
			};

			const result = resolveKeyColor(key, layer, categories);
			expect(result).toBe('#FF0000'); // Key override still works
		});

		it('should handle missing category gracefully', () => {
			const key: KeyAssignment = {
				keycode: 'KC_A',
				matrix_position: [0, 0],
				visual_index: 0,
				led_index: 0,
				category_id: 'nonexistent'
			};

			const layer: Layer = {
				name: 'Base',
				color: '#FFFFFF',
				default_color: yellowRgb,
				keys: [key]
			};

			const result = resolveKeyColor(key, layer, categories);
			expect(result).toBe('#FFFF00'); // Falls back to layer default
		});

		it('uses matching base-layer colors for transparent and no-op keys when enabled', () => {
			const baseKeys: KeyAssignment[] = [
				{
					keycode: 'KC_A',
					matrix_position: [0, 0],
					visual_index: 0,
					led_index: 0,
					color_override: redRgb
				},
				{
					keycode: 'KC_B',
					matrix_position: [0, 1],
					visual_index: 1,
					led_index: 1,
					color_override: greenRgb
				}
			];
			const baseLayer: Layer = {
				name: 'Base',
				number: 0,
				color: '#FFFFFF',
				default_color: blueRgb,
				keys: baseKeys
			};
			const layer: Layer = {
				name: 'Navigation',
				number: 1,
				color: '#FFFFFF',
				default_color: blueRgb,
				keys: []
			};

			const transparent = { ...baseKeys[0], keycode: 'KC_TRNS' };
			const noOp = { ...baseKeys[1], keycode: 'KC_NO' };

			expect(resolveKeyColor(transparent, layer, [], baseLayer, true)).toBe('#FF0000');
			expect(resolveKeyColor(noOp, layer, [], baseLayer, true)).toBe('#00FF00');
			expect(resolveKeyColor(transparent, layer, [], baseLayer, false)).toBe('#FFFFFF');
			expect(resolveKeyColor(noOp, layer, [], baseLayer, false)).toBe('#FFFFFF');
		});
	});

	describe('rgbToHex', () => {
		it('should convert RGB to hex', () => {
			expect(rgbToHex({ r: 255, g: 0, b: 0 })).toBe('#FF0000');
			expect(rgbToHex({ r: 0, g: 255, b: 0 })).toBe('#00FF00');
			expect(rgbToHex({ r: 0, g: 0, b: 255 })).toBe('#0000FF');
			expect(rgbToHex({ r: 0, g: 0, b: 0 })).toBe('#000000');
			expect(rgbToHex({ r: 255, g: 255, b: 255 })).toBe('#FFFFFF');
		});

		it('should pad single-digit hex values', () => {
			expect(rgbToHex({ r: 1, g: 2, b: 3 })).toBe('#010203');
		});
	});

	describe('hexToRgb', () => {
		it('should convert hex to RGB', () => {
			expect(hexToRgb('#FF0000')).toEqual({ r: 255, g: 0, b: 0 });
			expect(hexToRgb('#00FF00')).toEqual({ r: 0, g: 255, b: 0 });
			expect(hexToRgb('#0000FF')).toEqual({ r: 0, g: 0, b: 255 });
			expect(hexToRgb('#000000')).toEqual({ r: 0, g: 0, b: 0 });
			expect(hexToRgb('#FFFFFF')).toEqual({ r: 255, g: 255, b: 255 });
		});

		it('should handle hex without # prefix', () => {
			expect(hexToRgb('FF0000')).toEqual({ r: 255, g: 0, b: 0 });
		});

		it('should handle lowercase hex', () => {
			expect(hexToRgb('#ff0000')).toEqual({ r: 255, g: 0, b: 0 });
		});
	});

	describe('isValidHex', () => {
		it('should validate valid hex colors', () => {
			expect(isValidHex('#FF0000')).toBe(true);
			expect(isValidHex('FF0000')).toBe(true);
			expect(isValidHex('#00ff00')).toBe(true);
			expect(isValidHex('0000FF')).toBe(true);
		});

		it('should reject invalid hex colors', () => {
			expect(isValidHex('#FFF')).toBe(false);
			expect(isValidHex('#GGGGGG')).toBe(false);
			expect(isValidHex('FFFFFFFF')).toBe(false);
			expect(isValidHex('')).toBe(false);
			expect(isValidHex('#')).toBe(false);
		});
	});
});
