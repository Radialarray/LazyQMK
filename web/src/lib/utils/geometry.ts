/**
 * Geometry transformation utilities for keyboard preview rendering.
 *
 * Transforms raw geometry data from the backend API into SVG coordinates,
 * handling unit conversion, padding, and layout bounds calculation.
 */

import type { KeyGeometryInfo } from '$api/types';

/** Default key unit size in SVG pixels */
export const KEY_UNIT_SIZE = 54;

/** Padding around the keyboard in SVG pixels */
export const KEYBOARD_PADDING = 10;

/** Gap between keys in SVG pixels */
export const KEY_GAP = 4;

/** Corner radius for key rectangles */
export const KEY_BORDER_RADIUS = 6;

/**
 * A key with computed SVG coordinates ready for rendering.
 */
export interface KeySvgData {
	/** Matrix row (for identification) */
	matrixRow: number;
	/** Matrix column (for identification) */
	matrixCol: number;
	/** SVG x coordinate (top-left) */
	x: number;
	/** SVG y coordinate (top-left) */
	y: number;
	/** SVG width */
	width: number;
	/** SVG height */
	height: number;
	/** Rotation angle in degrees (around center) */
	rotation: number;
	/** LED index if available */
	ledIndex?: number;
	/** Visual index (order in keys array) */
	visualIndex: number;
}

/**
 * Computed SVG viewport dimensions.
 */
export interface ViewportDimensions {
	/** Total SVG width */
	width: number;
	/** Total SVG height */
	height: number;
	/** Minimum x coordinate of any key */
	minX: number;
	/** Minimum y coordinate of any key */
	minY: number;
	/** Maximum x coordinate (including key width) */
	maxX: number;
	/** Maximum y coordinate (including key height) */
	maxY: number;
}

/**
 * Result of transforming geometry data for SVG rendering.
 */
export interface TransformedGeometry {
	/** Keys with SVG coordinates */
	keys: KeySvgData[];
	/** Viewport dimensions */
	viewport: ViewportDimensions;
}

/**
 * Transforms raw key geometry data from the backend into SVG coordinates.
 *
 * @param keys - Array of key geometry information from the API
 * @param unitSize - Size of one key unit in SVG pixels (default: 54)
 * @param padding - Padding around the keyboard (default: 10)
 * @param gap - Gap between keys (default: 4)
 * @returns Transformed geometry with SVG coordinates and viewport dimensions
 */
export function transformGeometry(
	keys: KeyGeometryInfo[],
	unitSize: number = KEY_UNIT_SIZE,
	padding: number = KEYBOARD_PADDING,
	gap: number = KEY_GAP
): TransformedGeometry {
	if (keys.length === 0) {
		return {
			keys: [],
			viewport: {
				width: 0,
				height: 0,
				minX: 0,
				minY: 0,
				maxX: 0,
				maxY: 0
			}
		};
	}

	// First pass: compute raw SVG coordinates for all keys
	const rawKeys: KeySvgData[] = keys.map((key) => ({
		matrixRow: key.matrix_row,
		matrixCol: key.matrix_col,
		x: key.x * unitSize,
		y: key.y * unitSize,
		width: key.width * unitSize - gap,
		height: key.height * unitSize - gap,
		rotation: key.rotation,
		ledIndex: key.led_index,
		visualIndex: key.visual_index
	}));

	// Compute bounds (accounting for rotation is complex, for now ignore rotation for bounds)
	let minX = Infinity;
	let minY = Infinity;
	let maxX = -Infinity;
	let maxY = -Infinity;

	for (const key of rawKeys) {
		minX = Math.min(minX, key.x);
		minY = Math.min(minY, key.y);
		maxX = Math.max(maxX, key.x + key.width);
		maxY = Math.max(maxY, key.y + key.height);
	}

	// Translate keys so minimum is at padding
	const offsetX = padding - minX;
	const offsetY = padding - minY;

	const translatedKeys: KeySvgData[] = rawKeys.map((key) => ({
		...key,
		x: key.x + offsetX,
		y: key.y + offsetY
	}));

	// Compute final viewport
	const viewport: ViewportDimensions = {
		width: maxX - minX + 2 * padding,
		height: maxY - minY + 2 * padding,
		minX: padding,
		minY: padding,
		maxX: maxX - minX + padding,
		maxY: maxY - minY + padding
	};

	return {
		keys: translatedKeys,
		viewport
	};
}

/**
 * Computes the center point of a key for rotation transforms.
 */
export function getKeyCenter(key: KeySvgData): { cx: number; cy: number } {
	return {
		cx: key.x + key.width / 2,
		cy: key.y + key.height / 2
	};
}

/**
 * Generates an SVG transform string for a rotated key.
 */
export function getKeyTransform(key: KeySvgData): string {
	if (key.rotation === 0) {
		return '';
	}
	const { cx, cy } = getKeyCenter(key);
	return `rotate(${key.rotation} ${cx} ${cy})`;
}

/**
 * Creates a unique key identifier from matrix position.
 */
export function getKeyId(key: KeySvgData): string {
	return `key-${key.matrixRow}-${key.matrixCol}`;
}

/**
 * Finds a key by matrix position.
 */
export function findKeyByMatrix(
	keys: KeySvgData[],
	row: number,
	col: number
): KeySvgData | undefined {
	return keys.find((k) => k.matrixRow === row && k.matrixCol === col);
}

/**
 * Finds a key by visual index.
 */
export function findKeyByVisualIndex(keys: KeySvgData[], visualIndex: number): KeySvgData | undefined {
	return keys.find((k) => k.visualIndex === visualIndex);
}
