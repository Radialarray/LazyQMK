<script lang="ts">
	import { Button } from '$components';
	import { hexToRgb, isValidHex } from '$lib/utils/colorResolution';
	import type { RgbColor } from '$api/types';

	interface Props {
		color?: RgbColor;
		onSelect: (color: RgbColor) => void;
		onClear?: () => void;
		label?: string;
		showClear?: boolean;
	}

	let { color, onSelect, onClear, label = 'Color', showClear = false }: Props = $props();

	// Predefined color palette
	const colorPalette = [
		{ hex: '#FF0000', name: 'Red' },
		{ hex: '#FF8800', name: 'Orange' },
		{ hex: '#FFFF00', name: 'Yellow' },
		{ hex: '#00FF00', name: 'Green' },
		{ hex: '#00FFFF', name: 'Cyan' },
		{ hex: '#0088FF', name: 'Blue' },
		{ hex: '#8800FF', name: 'Purple' },
		{ hex: '#FF00FF', name: 'Magenta' },
		{ hex: '#FFFFFF', name: 'White' },
		{ hex: '#CCCCCC', name: 'Light gray' },
		{ hex: '#888888', name: 'Gray' },
		{ hex: '#444444', name: 'Dark gray' }
	];

	let customHex = $state('');
	let customError = $state('');

	// Convert current color to hex for display
	const currentHex = $derived(
		color ? `#${color.r.toString(16).padStart(2, '0')}${color.g.toString(16).padStart(2, '0')}${color.b.toString(16).padStart(2, '0')}`.toUpperCase() : ''
	);

	function handlePaletteClick(hex: string) {
		onSelect(hexToRgb(hex));
	}

	function handleCustomInput(e: Event) {
		const input = e.currentTarget as HTMLInputElement;
		customHex = input.value;
		customError = '';
	}

	function handleCustomSubmit() {
		if (!customHex) {
			customError = 'Please enter a hex color';
			return;
		}

		if (!isValidHex(customHex)) {
			customError = 'Invalid hex color format (use #RRGGBB or RRGGBB)';
			return;
		}

		onSelect(hexToRgb(customHex));
		customHex = '';
		customError = '';
	}

	function handleClear() {
		onClear?.();
	}

	function isSelected(hex: string): boolean {
		return currentHex === hex;
	}
</script>

<div class="color-picker">
	<p class="block text-sm font-medium text-muted-foreground mb-2">{label}</p>

	{#if currentHex}
		<div class="mb-3 flex items-center gap-2">
			<span class="text-sm text-muted-foreground">Current:</span>
			<div class="w-8 h-8 rounded border border-border" style="background-color: {currentHex}"></div>
			<code class="text-xs font-mono bg-muted px-2 py-1 rounded">{currentHex}</code>
		</div>
	{/if}

	<!-- Predefined palette -->
	<div class="mb-4">
		<p class="text-xs text-muted-foreground mb-2">Preset colors:</p>
		<div class="grid grid-cols-6 gap-2">
			{#each colorPalette as paletteColor}
				<button
					type="button"
					onclick={() => handlePaletteClick(paletteColor.hex)}
					class="group relative h-10 w-10 rounded border transition-colors cursor-pointer {isSelected(paletteColor.hex) ? 'border-primary ring-2 ring-primary ring-offset-2 ring-offset-background' : 'border-border hover:border-primary'}"
					style="background-color: {paletteColor.hex}"
					title={`${paletteColor.name} (${paletteColor.hex})`}
					aria-label={`Select ${paletteColor.name} color ${paletteColor.hex}`}
					aria-pressed={isSelected(paletteColor.hex)}
				>
					<span class="sr-only">{paletteColor.name}</span>
				</button>
			{/each}
		</div>
		<p class="mt-2 text-xs text-muted-foreground">Named swatches improve screen reader and keyboard support.</p>
	</div>

	<!-- Custom hex input -->
	<div class="mb-3">
		<p class="text-xs text-muted-foreground mb-2">Custom hex color:</p>
		<div class="flex items-center gap-2">
			<input
				type="text"
				value={customHex}
				oninput={handleCustomInput}
				placeholder="#FF0000"
				class="flex-1 px-3 py-2 border border-border rounded-lg bg-background text-sm font-mono"
			/>
			<Button onclick={handleCustomSubmit} size="sm">Apply</Button>
		</div>
		{#if customError}
			<p class="text-xs text-red-500 mt-1">{customError}</p>
		{/if}
	</div>

	<!-- Clear button -->
	{#if showClear && onClear}
		<Button onclick={handleClear} variant="outline" size="sm" class="w-full" data-testid="color-picker-clear-button">Clear</Button>
	{/if}
</div>

<style>
	/* Component wrapper */
</style>
