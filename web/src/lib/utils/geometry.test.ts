import { describe, it, expect } from 'vitest';
import {
	transformGeometry,
	getKeyCenter,
	getKeyTransform,
	getKeyId,
	findKeyByMatrix,
	findKeyByVisualIndex,
	KEY_UNIT_SIZE,
	KEYBOARD_PADDING,
	KEY_GAP
} from './geometry';
import type { KeyGeometryInfo } from '$api/types';

describe('transformGeometry', () => {
	it('returns empty result for empty keys array', () => {
		const result = transformGeometry([]);
		expect(result.keys).toHaveLength(0);
		expect(result.viewport.width).toBe(0);
		expect(result.viewport.height).toBe(0);
	});

	it('transforms a single key correctly', () => {
		const keys: KeyGeometryInfo[] = [
			{
				matrix_row: 0,
				matrix_col: 0,
				x: 0,
				y: 0,
				width: 1,
				height: 1,
				rotation: 0,
				visual_index: 0
			}
		];

		const result = transformGeometry(keys);

		expect(result.keys).toHaveLength(1);
		const key = result.keys[0];

		// Key should be positioned at padding offset
		expect(key.x).toBe(KEYBOARD_PADDING);
		expect(key.y).toBe(KEYBOARD_PADDING);

		// Key size should be unit size minus gap
		expect(key.width).toBe(KEY_UNIT_SIZE - KEY_GAP);
		expect(key.height).toBe(KEY_UNIT_SIZE - KEY_GAP);

		// Viewport should encompass key plus padding
		expect(result.viewport.width).toBe(KEY_UNIT_SIZE - KEY_GAP + 2 * KEYBOARD_PADDING);
		expect(result.viewport.height).toBe(KEY_UNIT_SIZE - KEY_GAP + 2 * KEYBOARD_PADDING);
	});

	it('transforms multiple keys with correct positions', () => {
		const keys: KeyGeometryInfo[] = [
			{ matrix_row: 0, matrix_col: 0, x: 0, y: 0, width: 1, height: 1, rotation: 0, visual_index: 0 },
			{ matrix_row: 0, matrix_col: 1, x: 1, y: 0, width: 1, height: 1, rotation: 0, visual_index: 1 },
			{ matrix_row: 1, matrix_col: 0, x: 0, y: 1, width: 1, height: 1, rotation: 0, visual_index: 2 }
		];

		const result = transformGeometry(keys);

		expect(result.keys).toHaveLength(3);

		// First key at origin
		expect(result.keys[0].x).toBe(KEYBOARD_PADDING);
		expect(result.keys[0].y).toBe(KEYBOARD_PADDING);

		// Second key one unit to the right
		expect(result.keys[1].x).toBe(KEYBOARD_PADDING + KEY_UNIT_SIZE);
		expect(result.keys[1].y).toBe(KEYBOARD_PADDING);

		// Third key one unit down
		expect(result.keys[2].x).toBe(KEYBOARD_PADDING);
		expect(result.keys[2].y).toBe(KEYBOARD_PADDING + KEY_UNIT_SIZE);
	});

	it('handles wide keys (1.5u, 2u)', () => {
		const keys: KeyGeometryInfo[] = [
			{ matrix_row: 0, matrix_col: 0, x: 0, y: 0, width: 1.5, height: 1, rotation: 0, visual_index: 0 },
			{ matrix_row: 0, matrix_col: 1, x: 1.5, y: 0, width: 2, height: 1, rotation: 0, visual_index: 1 }
		];

		const result = transformGeometry(keys);

		expect(result.keys[0].width).toBe(1.5 * KEY_UNIT_SIZE - KEY_GAP);
		expect(result.keys[1].width).toBe(2 * KEY_UNIT_SIZE - KEY_GAP);
	});

	it('preserves rotation values', () => {
		const keys: KeyGeometryInfo[] = [
			{ matrix_row: 0, matrix_col: 0, x: 0, y: 0, width: 1, height: 1, rotation: 15, visual_index: 0 }
		];

		const result = transformGeometry(keys);
		expect(result.keys[0].rotation).toBe(15);
	});

	it('preserves LED index when present', () => {
		const keys: KeyGeometryInfo[] = [
			{ matrix_row: 0, matrix_col: 0, x: 0, y: 0, width: 1, height: 1, rotation: 0, led_index: 5, visual_index: 0 }
		];

		const result = transformGeometry(keys);
		expect(result.keys[0].ledIndex).toBe(5);
	});

	it('handles negative coordinates (normalizes to positive)', () => {
		const keys: KeyGeometryInfo[] = [
			{ matrix_row: 0, matrix_col: 0, x: -1, y: -1, width: 1, height: 1, rotation: 0, visual_index: 0 }
		];

		const result = transformGeometry(keys);

		// Should still start at padding
		expect(result.keys[0].x).toBe(KEYBOARD_PADDING);
		expect(result.keys[0].y).toBe(KEYBOARD_PADDING);
	});

	it('assigns visual index based on array order', () => {
		const keys: KeyGeometryInfo[] = [
			{ matrix_row: 1, matrix_col: 2, x: 0, y: 0, width: 1, height: 1, rotation: 0, visual_index: 0 },
			{ matrix_row: 0, matrix_col: 1, x: 1, y: 0, width: 1, height: 1, rotation: 0, visual_index: 1 }
		];

		const result = transformGeometry(keys);

		expect(result.keys[0].visualIndex).toBe(0);
		expect(result.keys[1].visualIndex).toBe(1);
	});

	it('allows custom unit size', () => {
		const keys: KeyGeometryInfo[] = [
			{ matrix_row: 0, matrix_col: 0, x: 0, y: 0, width: 1, height: 1, rotation: 0, visual_index: 0 }
		];

		const customUnitSize = 100;
		const result = transformGeometry(keys, customUnitSize);

		expect(result.keys[0].width).toBe(customUnitSize - KEY_GAP);
	});
});

describe('getKeyCenter', () => {
	it('computes center point correctly', () => {
		const key = {
			matrixRow: 0,
			matrixCol: 0,
			x: 10,
			y: 20,
			width: 50,
			height: 50,
			rotation: 0,
			visualIndex: 0
		};

		const center = getKeyCenter(key);
		expect(center.cx).toBe(35);
		expect(center.cy).toBe(45);
	});

	it('handles non-square keys', () => {
		const key = {
			matrixRow: 0,
			matrixCol: 0,
			x: 0,
			y: 0,
			width: 100,
			height: 50,
			rotation: 0,
			visualIndex: 0
		};

		const center = getKeyCenter(key);
		expect(center.cx).toBe(50);
		expect(center.cy).toBe(25);
	});
});

describe('getKeyTransform', () => {
	it('returns empty string for no rotation', () => {
		const key = {
			matrixRow: 0,
			matrixCol: 0,
			x: 10,
			y: 10,
			width: 50,
			height: 50,
			rotation: 0,
			visualIndex: 0
		};

		expect(getKeyTransform(key)).toBe('');
	});

	it('returns rotation transform for rotated key', () => {
		const key = {
			matrixRow: 0,
			matrixCol: 0,
			x: 10,
			y: 10,
			width: 50,
			height: 50,
			rotation: 15,
			visualIndex: 0
		};

		const transform = getKeyTransform(key);
		expect(transform).toBe('rotate(15 35 35)');
	});
});

describe('getKeyId', () => {
	it('generates ID from matrix position', () => {
		const key = {
			matrixRow: 2,
			matrixCol: 5,
			x: 0,
			y: 0,
			width: 50,
			height: 50,
			rotation: 0,
			visualIndex: 0
		};

		expect(getKeyId(key)).toBe('key-2-5');
	});
});

describe('findKeyByMatrix', () => {
	const keys = [
		{ matrixRow: 0, matrixCol: 0, x: 0, y: 0, width: 50, height: 50, rotation: 0, visualIndex: 0 },
		{ matrixRow: 0, matrixCol: 1, x: 50, y: 0, width: 50, height: 50, rotation: 0, visualIndex: 1 },
		{ matrixRow: 1, matrixCol: 0, x: 0, y: 50, width: 50, height: 50, rotation: 0, visualIndex: 2 }
	];

	it('finds key by matrix position', () => {
		const key = findKeyByMatrix(keys, 0, 1);
		expect(key).toBeDefined();
		expect(key?.visualIndex).toBe(1);
	});

	it('returns undefined for non-existent position', () => {
		const key = findKeyByMatrix(keys, 5, 5);
		expect(key).toBeUndefined();
	});
});

describe('findKeyByVisualIndex', () => {
	const keys = [
		{ matrixRow: 0, matrixCol: 0, x: 0, y: 0, width: 50, height: 50, rotation: 0, visualIndex: 0 },
		{ matrixRow: 0, matrixCol: 1, x: 50, y: 0, width: 50, height: 50, rotation: 0, visualIndex: 1 }
	];

	it('finds key by visual index', () => {
		const key = findKeyByVisualIndex(keys, 1);
		expect(key).toBeDefined();
		expect(key?.matrixCol).toBe(1);
	});

	it('returns undefined for non-existent index', () => {
		const key = findKeyByVisualIndex(keys, 99);
		expect(key).toBeUndefined();
	});
});
