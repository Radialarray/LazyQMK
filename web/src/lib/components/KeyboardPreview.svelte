<script lang="ts">
	import type { KeyGeometryInfo, KeyAssignment } from '$api/types';
	import {
		transformGeometry,
		getKeyTransform,
		getKeyId,
		KEY_BORDER_RADIUS,
		type KeySvgData
	} from '$lib/utils/geometry';

	interface Props {
		/** Raw geometry data from the backend API */
		geometry: KeyGeometryInfo[];
		/** Key assignments for the current layer (optional) */
		keyAssignments?: KeyAssignment[];
		/** Currently selected key index (visual index) */
		selectedKeyIndex?: number | null;
		/** Callback when a key is clicked */
		onKeyClick?: (visualIndex: number, matrixRow: number, matrixCol: number) => void;
		/** Custom class for the container */
		class?: string;
	}

	let {
		geometry,
		keyAssignments = [],
		selectedKeyIndex = null,
		onKeyClick,
		class: className = ''
	}: Props = $props();

	// Transform geometry data for SVG rendering
	const transformed = $derived(transformGeometry(geometry));

	// Create a lookup map from visual index to keycode label
	const keycodeMap = $derived.by(() => {
		const map = new Map<number, string>();
		for (const assignment of keyAssignments) {
			map.set(assignment.visual_index, formatKeycode(assignment.keycode));
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

	function handleKeyClick(key: KeySvgData) {
		onKeyClick?.(key.visualIndex, key.matrixRow, key.matrixCol);
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
	class="keyboard-preview {className}"
	role="application"
	aria-label="Keyboard layout preview"
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
				{@const isSelected = selectedKeyIndex === key.visualIndex}
				{@const label = keycodeMap.get(key.visualIndex) ?? ''}
				{@const transform = getKeyTransform(key)}
				{@const fontSize = getFontSize(label)}

				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<g
					class="key-group"
					transform={transform}
					onclick={() => handleKeyClick(key)}
					data-testid="key-{key.visualIndex}"
					data-visual-index={key.visualIndex}
					data-matrix-row={key.matrixRow}
					data-matrix-col={key.matrixCol}
				>
					<!-- Key background -->
					<rect
						x={key.x}
						y={key.y}
						width={key.width}
						height={key.height}
						rx={KEY_BORDER_RADIUS}
						ry={KEY_BORDER_RADIUS}
						class="key-bg {isSelected ? 'selected' : ''}"
						filter="url(#key-shadow)"
					/>

					<!-- Key top surface (slightly inset for 3D effect) -->
					<rect
						x={key.x + 2}
						y={key.y + 1}
						width={key.width - 4}
						height={key.height - 4}
						rx={KEY_BORDER_RADIUS - 1}
						ry={KEY_BORDER_RADIUS - 1}
						class="key-top {isSelected ? 'selected' : ''}"
					/>

					<!-- Key label -->
					{#if label}
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

	.key-top {
		fill: hsl(var(--card));
		transition: fill 0.15s ease;
	}

	.key-top.selected {
		fill: hsl(var(--primary) / 0.8);
	}

	.key-label {
		fill: hsl(var(--foreground));
		font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
		font-weight: 500;
		pointer-events: none;
	}

	.key-bg.selected + .key-top + .key-label,
	.key-bg.selected ~ .key-label {
		fill: hsl(var(--primary-foreground));
	}

	/* Hover effect */
	.key-group:hover .key-bg:not(.selected) {
		fill: hsl(var(--accent));
	}

	.key-group:hover .key-top:not(.selected) {
		fill: hsl(var(--accent) / 0.9);
	}
</style>
