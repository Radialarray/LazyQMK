<script lang="ts">
	import type { KeyGeometryInfo, KeyAssignment, Layer, Category, KeyRenderMetadata } from '$api/types';
	import {
		transformGeometry,
		getKeyTransform,
		getKeyId,
		KEY_BORDER_RADIUS,
		type KeySvgData
	} from '$lib/utils/geometry';
	import { resolveKeyColor } from '$lib/utils/colorResolution';
	import { handleKeyboardNavigation } from '$lib/utils/keyboardNavigation';

	interface Props {
		/** Raw geometry data from the backend API */
		geometry: KeyGeometryInfo[];
		/** Key assignments for the current layer (optional) */
		keyAssignments?: KeyAssignment[];
		/** Currently selected key index (visual index) */
		selectedKeyIndex?: number | null;
		/** Set of selected key indices for multi-selection */
		selectedKeyIndices?: Set<number>;
		/** Swap mode active flag */
		swapMode?: boolean;
		/** First key selected in swap mode */
		swapFirstKey?: number | null;
		/** Current layer (for color resolution) */
		layer?: Layer;
		/** Categories (for color resolution) */
		categories?: Category[];
		/** Render metadata for rich key labels (optional) */
		renderMetadata?: KeyRenderMetadata[];
		/** Callback when a key is clicked */
		onKeyClick?: (visualIndex: number, matrixRow: number, matrixCol: number, shiftKey: boolean) => void;
		/** Callback when keyboard navigation occurs */
		onNavigate?: (newKeyIndex: number | null, newSelectedIndices: Set<number>) => void;
		/** Callback when a key is hovered */
		onKeyHover?: (visualIndex: number | null) => void;
		/** Custom class for the container */
		class?: string;
		/** Position to visual index mapping from backend (optional, for fallback lookups) */
		positionToVisualIndexMap?: Record<string, number>;
	}

	let {
		geometry,
		keyAssignments = [],
		selectedKeyIndex = null,
		selectedKeyIndices = new Set(),
		swapMode = false,
		swapFirstKey = null,
		layer,
		categories = [],
		renderMetadata = [],
		onKeyClick,
		onNavigate,
		onKeyHover,
		class: className = '',
		positionToVisualIndexMap
	}: Props = $props();
	
	let containerElement: HTMLDivElement;

	// Transform geometry data for SVG rendering
	const transformed = $derived(transformGeometry(geometry));

	// Build a mapping from visual position (row, col) to visual_index.
	// Prefer the backend-provided mapping if available, otherwise compute locally as fallback.
	const positionToVisualIndex = $derived.by(() => {
		// Use backend-provided mapping if available
		if (positionToVisualIndexMap) {
			return new Map(Object.entries(positionToVisualIndexMap).map(([k, v]) => [k, v]));
		}
		// Fallback: compute locally from geometry (brittle, for backwards compatibility)
		const map = new Map<string, number>();
		for (const key of geometry) {
			// Geometry y → row, x → col (quantized to grid)
			const row = Math.round(key.y);
			const col = Math.round(key.x);
			const posKey = `${row},${col}`;
			map.set(posKey, key.visual_index);
		}
		return map;
	});

	// Create a lookup map from visual index to keycode label
	// Layout keys have position.row/col which maps to geometry visual_y/visual_x
	const keycodeMap = $derived.by(() => {
		const map = new Map<number, string>();
		for (const assignment of keyAssignments) {
			// Try to get visual_index from assignment (newer API) or look it up from position
			let visualIndex = assignment.visual_index;
			if (visualIndex === undefined || visualIndex === null) {
				// Fall back to looking up from position using geometry mapping
				const pos = assignment.position;
				if (pos) {
					const posKey = `${pos.row},${pos.col}`;
					visualIndex = positionToVisualIndex.get(posKey) ?? -1;
				} else {
					visualIndex = -1;
				}
			}
			if (visualIndex >= 0) {
				map.set(visualIndex, formatKeycode(assignment.keycode));
			}
		}
		return map;
	});

	// Create a lookup map from visual index to render metadata
	const renderMetadataMap = $derived.by(() => {
		const map = new Map<number, KeyRenderMetadata>();
		for (const metadata of renderMetadata) {
			map.set(metadata.visual_index, metadata);
		}
		return map;
	});

	// Create a lookup map from visual index to resolved color
	const colorMap = $derived.by(() => {
		const map = new Map<number, string | undefined>();
		if (layer) {
			for (const assignment of keyAssignments) {
				// Try to get visual_index from assignment (newer API) or look it up from position
				let visualIndex = assignment.visual_index;
				if (visualIndex === undefined || visualIndex === null) {
					const pos = assignment.position;
					if (pos) {
						const posKey = `${pos.row},${pos.col}`;
						visualIndex = positionToVisualIndex.get(posKey) ?? -1;
					} else {
						visualIndex = -1;
					}
				}
				if (visualIndex >= 0) {
					const color = resolveKeyColor(assignment, layer, categories);
					map.set(visualIndex, color);
				}
			}
		}
		return map;
	});

	/**
	 * Formats a keycode for display on a key cap.
	 * Shortens common prefixes and handles special cases.
	 */
	function formatKeycode(keycode: string): string {
		if (!keycode || keycode === 'KC_NO' || keycode === 'XXXXXXX') {
			return '';
		}
		if (keycode === 'KC_TRNS' || keycode === '_______') {
			return '▽';
		}
		// Remove common prefixes for cleaner display
		let label = keycode
			.replace(/^KC_/, '')
			.replace(/^QK_/, '')
			.replace(/^RGB_/, 'RGB\n')
			.replace(/^MO\((\d+)\)/, 'MO($1)')
			.replace(/^TG\((\d+)\)/, 'TG($1)')
			.replace(/^TO\((\d+)\)/, 'TO($1)')
			.replace(/^LT\((\d+),\s*(.+)\)/, 'LT$1\n$2')
			.replace(/^MT\((.+),\s*(.+)\)/, '$1\n$2')
			.replace(/^TD\((\d+)\)/, 'TD($1)');

		// Truncate long labels
		if (label.length > 8 && !label.includes('\n')) {
			label = label.substring(0, 7) + '…';
		}

		return label;
	}

	function handleKeyClick(key: KeySvgData, event: MouseEvent) {
		onKeyClick?.(key.visualIndex, key.matrixRow, key.matrixCol, event.shiftKey);
	}

	function handleKeyHover(visualIndex: number | null) {
		onKeyHover?.(visualIndex);
	}
	
	/**
	 * Handle keyboard events for navigation
	 */
	function handleKeyDown(event: KeyboardEvent) {
		const result = handleKeyboardNavigation(
			event,
			selectedKeyIndex,
			transformed.keys,
			selectedKeyIndices
		);
		
		if (result.handled) {
			event.preventDefault();
			event.stopPropagation();
			onNavigate?.(result.newKeyIndex, result.newSelectedIndices);
		}
	}

	/**
	 * Computes the font size based on label length.
	 */
	function getFontSize(label: string): number {
		if (label.includes('\n')) {
			return 9;
		}
		if (label.length > 5) {
			return 10;
		}
		if (label.length > 3) {
			return 11;
		}
		return 12;
	}
</script>

<div
	bind:this={containerElement}
	class="keyboard-preview {className}"
	role="button"
	aria-label="Keyboard layout preview"
	tabindex="0"
	onkeydown={handleKeyDown}
>
	{#if transformed.keys.length > 0}
		<svg
			viewBox="0 0 {transformed.viewport.width} {transformed.viewport.height}"
			class="w-full h-auto"
			xmlns="http://www.w3.org/2000/svg"
		>
			<!-- Key definitions for reusable styles -->
			<defs>
				<filter id="key-shadow" x="-10%" y="-10%" width="120%" height="120%">
					<feDropShadow dx="0" dy="1" stdDeviation="1" flood-opacity="0.15" />
				</filter>
			</defs>

			<!-- Render each key -->
			{#each transformed.keys as key (getKeyId(key))}
				{@const isSelected = selectedKeyIndex === key.visualIndex || selectedKeyIndices.has(key.visualIndex)}
				{@const isSwapFirst = swapMode && swapFirstKey === key.visualIndex}
				{@const label = keycodeMap.get(key.visualIndex) ?? ''}
				{@const metadata = renderMetadataMap.get(key.visualIndex)}
				{@const transform = getKeyTransform(key)}
				{@const fontSize = getFontSize(label)}
				{@const resolvedColor = colorMap.get(key.visualIndex)}

				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<g
					class="key-group"
					transform={transform}
					onclick={(e) => handleKeyClick(key, e)}
					onmouseenter={() => handleKeyHover(key.visualIndex)}
					onmouseleave={() => handleKeyHover(null)}
					data-testid="key-{key.visualIndex}"
					data-visual-index={key.visualIndex}
					data-matrix-row={key.matrixRow}
					data-matrix-col={key.matrixCol}
				>
					<!-- Define clip path for label overflow prevention -->
					<defs>
						<clipPath id="clip-{key.visualIndex}">
							<rect
								x={key.x + 3}
								y={key.y + 2}
								width={key.width - 6}
								height={key.height - 5}
								rx={KEY_BORDER_RADIUS - 2}
								ry={KEY_BORDER_RADIUS - 2}
							/>
						</clipPath>
						<!-- RGB Glow filter (always apply when color exists, including selected keys) -->
						{#if resolvedColor}
							<filter id="glow-{key.visualIndex}" x="-100%" y="-100%" width="300%" height="300%">
								<feGaussianBlur in="SourceGraphic" stdDeviation="3.5" result="blur" />
								<feFlood flood-color={resolvedColor} flood-opacity="0.6" result="color" />
								<feComposite in="color" in2="blur" operator="in" result="glow" />
								<feMerge>
									<feMergeNode in="glow" />
									<feMergeNode in="SourceGraphic" />
								</feMerge>
							</filter>
						{/if}
					</defs>

					<!-- Key background -->
					<rect
						x={key.x}
						y={key.y}
						width={key.width}
						height={key.height}
						rx={KEY_BORDER_RADIUS}
						ry={KEY_BORDER_RADIUS}
						class="key-bg {isSelected ? 'selected' : ''} {isSwapFirst ? 'swap-first' : ''}"
						style={resolvedColor && !isSelected && !isSwapFirst ? `fill: ${resolvedColor}` : ''}
						filter={resolvedColor ? `url(#glow-${key.visualIndex})` : 'url(#key-shadow)'}
					/>

					<!-- Key top surface (slightly inset for 3D effect) -->
					<rect
						x={key.x + 2}
						y={key.y + 1}
						width={key.width - 4}
						height={key.height - 4}
						rx={KEY_BORDER_RADIUS - 1}
						ry={KEY_BORDER_RADIUS - 1}
						class="key-top {isSelected ? 'selected' : ''} {isSwapFirst ? 'swap-first' : ''}"
						style={resolvedColor && !isSelected && !isSwapFirst ? `fill: ${resolvedColor}; opacity: 0.9` : ''}
					/>

					<!-- Key label - use render metadata if available, otherwise fallback to formatted keycode -->
					<!-- Wrap labels in a group with clip-path to prevent overflow -->
					<g clip-path="url(#clip-{key.visualIndex})">
						{#if metadata}
							{@const primaryLabel = metadata.display.primary}
							{@const secondaryLabel = metadata.display.secondary}
							{@const tertiaryLabel = metadata.display.tertiary}
							{@const labelCount = [primaryLabel, secondaryLabel, tertiaryLabel].filter(Boolean).length}
							{@const labelFontSize = labelCount > 1 ? 9 : 12}
							
							{#if labelCount === 1}
								<!-- Single label: centered -->
								<text
									x={key.x + key.width / 2}
									y={key.y + key.height / 2 + labelFontSize / 3}
									text-anchor="middle"
									class="key-label"
									font-size={labelFontSize}
								>
									{primaryLabel}
								</text>
							{:else if labelCount === 2}
								<!-- Two labels: secondary on top, primary centered below -->
								<text
									x={key.x + key.width / 2}
									y={key.y + 10}
									text-anchor="middle"
									class="key-label secondary"
									font-size={labelFontSize}
								>
									{secondaryLabel}
								</text>
								<text
									x={key.x + key.width / 2}
									y={key.y + key.height / 2 + labelFontSize / 3 + 3}
									text-anchor="middle"
									class="key-label"
									font-size={labelFontSize}
								>
									{primaryLabel}
								</text>
							{:else if labelCount === 3}
								<!-- Three labels: secondary and tertiary on top row, primary centered -->
								<text
									x={key.x + key.width / 2}
									y={key.y + 9}
									text-anchor="middle"
									class="key-label secondary"
									font-size={8}
								>
									{secondaryLabel}
								</text>
								<text
									x={key.x + key.width / 2}
									y={key.y + 18}
									text-anchor="middle"
									class="key-label tertiary"
									font-size={7}
								>
									{tertiaryLabel}
								</text>
								<text
									x={key.x + key.width / 2}
									y={key.y + key.height / 2 + labelFontSize / 3 + 6}
									text-anchor="middle"
									class="key-label"
									font-size={labelFontSize}
								>
									{primaryLabel}
								</text>
							{/if}
						{:else if label}
							{@const lines = label.split('\n')}
							{#if lines.length === 1}
								<text
									x={key.x + key.width / 2}
									y={key.y + key.height / 2 + fontSize / 3}
									text-anchor="middle"
									class="key-label"
									font-size={fontSize}
								>
									{label}
								</text>
							{:else}
								{#each lines as line, i}
									<text
										x={key.x + key.width / 2}
										y={key.y + key.height / 2 + (i - (lines.length - 1) / 2) * (fontSize + 2)}
										text-anchor="middle"
										class="key-label"
										font-size={fontSize}
									>
										{line}
									</text>
								{/each}
							{/if}
						{/if}
					</g>
				</g>
			{/each}
		</svg>
	{:else}
		<div class="flex items-center justify-center h-32 text-muted-foreground">
			No geometry data available
		</div>
	{/if}
</div>

<style>
	.keyboard-preview {
		user-select: none;
		outline: none;
	}
	
	.keyboard-preview:focus-visible {
		outline: 2px solid hsl(var(--ring));
		outline-offset: 2px;
		border-radius: 8px;
	}

	.key-group {
		cursor: pointer;
		transition: transform 0.1s ease;
	}

	.key-group:hover {
		transform: translateY(-1px);
	}

	.key-bg {
		fill: hsl(var(--muted));
		stroke: hsl(var(--border));
		stroke-width: 1;
		transition: fill 0.15s ease, stroke 0.15s ease;
	}

	.key-bg.selected {
		fill: hsl(var(--primary));
		stroke: hsl(var(--primary));
	}

	.key-bg.swap-first {
		fill: hsl(48 96% 53%); /* yellow-400 */
		stroke: hsl(48 96% 53%);
		stroke-width: 2;
	}

	.key-top {
		fill: hsl(var(--card));
		transition: fill 0.15s ease;
	}

	.key-top.selected {
		fill: hsl(var(--primary) / 0.8);
	}

	.key-top.swap-first {
		fill: hsl(48 96% 53% / 0.8); /* yellow-400 with opacity */
	}

	.key-label {
		fill: hsl(var(--foreground));
		font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
		font-weight: 500;
		pointer-events: none;
	}

	.key-label.secondary {
		opacity: 0.7;
		font-weight: 400;
	}

	.key-label.tertiary {
		opacity: 0.5;
		font-weight: 400;
		font-size: 8px;
	}

	/* Hover effect */
	.key-group:hover .key-bg:not(.selected):not(.swap-first) {
		fill: hsl(var(--accent));
	}

	.key-group:hover .key-top:not(.selected):not(.swap-first) {
		fill: hsl(var(--accent) / 0.9);
	}
</style>
