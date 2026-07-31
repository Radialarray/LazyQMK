<script lang="ts">
	import { AccessibleDialog, Button } from '$components';

	interface LayerOption {
		index: number;
		name: string;
		id?: string;
	}

	interface Props {
		open: boolean;
		layers: LayerOption[];
		onSelect: (layerRef: { index: number; name: string; id?: string }) => void;
		onClose: () => void;
		title?: string;
		description?: string;
	}

	let { open, layers, onSelect, onClose, title = 'Select layer', description }: Props = $props();

	function handleSelect(layer: LayerOption) {
		// Prefer UUID when available so the saved keycode is robust to layer reordering.
		if (layer.id) {
			onSelect({ index: layer.index, name: layer.name, id: layer.id });
		} else {
			onSelect({ index: layer.index, name: layer.name });
		}
	}
</script>

<AccessibleDialog
	{open}
	{title}
	description={description ?? 'Pick which layer this keycode should target.'}
	{onClose}
	titleId="layer-picker-title"
	panelClass="max-w-md"
>
	{#if layers.length === 0}
		<p class="text-sm text-muted-foreground" data-testid="layer-picker-empty">
			This layout has only the base layer. Add another layer first.
		</p>
	{:else}
		<ul class="space-y-1 max-h-96 overflow-y-auto" data-testid="layer-picker-list">
			{#each layers as layer (layer.index)}
				<li>
					<button
						type="button"
						onclick={() => handleSelect(layer)}
						class="w-full text-left px-3 py-2 rounded border border-border hover:bg-accent transition-colors flex items-center justify-between gap-2"
						data-testid="layer-picker-option-{layer.index}"
					>
						<span class="font-medium text-sm">
							Layer {layer.index}
							<span class="text-muted-foreground font-normal">— {layer.name}</span>
						</span>
						{#if layer.id}
							<span
								class="text-[10px] font-mono text-muted-foreground bg-muted px-1.5 py-0.5 rounded truncate max-w-[120px]"
								title={layer.id}
							>
								{layer.id.slice(0, 8)}
							</span>
						{/if}
					</button>
				</li>
			{/each}
		</ul>
	{/if}
	<svelte:fragment slot="footer">
		<Button onclick={onClose} variant="ghost">Cancel</Button>
	</svelte:fragment>
</AccessibleDialog>