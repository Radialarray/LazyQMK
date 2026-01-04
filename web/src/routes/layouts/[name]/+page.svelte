<script lang="ts">
	import { Button, Card, KeyboardPreview } from '$components';
	import { apiClient } from '$api';
	import type { PageData } from './$types';
	import type { GeometryResponse } from '$api/types';

	let { data }: { data: PageData } = $props();
	let layout = $derived(data.layout);

	// State for keyboard preview
	let geometry = $state<GeometryResponse | null>(null);
	let geometryError = $state<string | null>(null);
	let geometryLoading = $state(false);
	let selectedKeyIndex = $state<number | null>(null);
	let selectedLayerIndex = $state(0);

	// Load geometry when layout is available
	$effect(() => {
		if (layout?.metadata.keyboard && layout?.metadata.layout) {
			loadGeometry(layout.metadata.keyboard, layout.metadata.layout);
		}
	});

	async function loadGeometry(keyboard: string, layoutName: string) {
		geometryLoading = true;
		geometryError = null;
		try {
			geometry = await apiClient.getGeometry(keyboard, layoutName);
		} catch (e) {
			geometryError = e instanceof Error ? e.message : 'Failed to load keyboard geometry';
			geometry = null;
		} finally {
			geometryLoading = false;
		}
	}

	function handleKeyClick(visualIndex: number, matrixRow: number, matrixCol: number) {
		selectedKeyIndex = visualIndex;
	}

	function handleLayerChange(index: number) {
		selectedLayerIndex = index;
		// Clear selection when switching layers
		selectedKeyIndex = null;
	}

	// Get key assignments for the current layer
	const currentLayerKeys = $derived(layout?.layers[selectedLayerIndex]?.keys ?? []);

	// Get selected key details
	const selectedKey = $derived.by(() => {
		if (selectedKeyIndex === null || !currentLayerKeys.length) return null;
		return currentLayerKeys.find((k) => k.visual_index === selectedKeyIndex) ?? null;
	});
</script>

<div class="container mx-auto p-6">
	<div class="mb-8 flex items-center justify-between">
		<div>
			<h1 class="text-4xl font-bold mb-2">
				{layout?.metadata.name || 'Loading...'}
			</h1>
			<p class="text-muted-foreground">
				{layout?.metadata.description || ''}
			</p>
		</div>
		<a href="/layouts">
			<Button>Back to Layouts</Button>
		</a>
	</div>

	{#if layout}
		<div class="space-y-6">
			<!-- Metadata Card -->
			<Card class="p-6">
				<h2 class="text-xl font-semibold mb-4">Metadata</h2>
				<dl class="grid grid-cols-2 gap-4">
					<div>
						<dt class="text-sm font-medium text-muted-foreground">Keyboard</dt>
						<dd class="text-sm">{layout.metadata.keyboard}</dd>
					</div>
					<div>
						<dt class="text-sm font-medium text-muted-foreground">Layout</dt>
						<dd class="text-sm">{layout.metadata.layout}</dd>
					</div>
					<div>
						<dt class="text-sm font-medium text-muted-foreground">Author</dt>
						<dd class="text-sm">{layout.metadata.author}</dd>
					</div>
					<div>
						<dt class="text-sm font-medium text-muted-foreground">Created</dt>
						<dd class="text-sm">
							{new Date(layout.metadata.created).toLocaleDateString()}
						</dd>
					</div>
				</dl>
			</Card>

			<!-- Layer Selector -->
			<Card class="p-6">
				<h2 class="text-xl font-semibold mb-4">Layers</h2>
				<div class="flex gap-2 flex-wrap">
					{#each layout.layers as layer, i}
						<button
							onclick={() => handleLayerChange(i)}
							class="flex items-center gap-2 px-4 py-2 rounded-lg border transition-colors
								{selectedLayerIndex === i
								? 'bg-primary text-primary-foreground border-primary'
								: 'bg-background hover:bg-accent border-border'}"
						>
							<span
								class="w-3 h-3 rounded-full"
								style="background-color: {layer.color}"
							></span>
							<span class="font-medium">{layer.name}</span>
						</button>
					{/each}
				</div>
			</Card>

			<!-- Keyboard Preview -->
			<Card class="p-6">
				<div class="flex items-center justify-between mb-4">
					<h2 class="text-xl font-semibold">Keyboard Preview</h2>
					{#if selectedKey}
						<div class="text-sm text-muted-foreground">
							Selected: <code class="px-2 py-1 bg-muted rounded">{selectedKey.keycode}</code>
							<span class="ml-2 text-xs">
								(Position: {selectedKey.matrix_position[0]},{selectedKey.matrix_position[1]})
							</span>
						</div>
					{/if}
				</div>

				{#if geometryLoading}
					<div class="flex items-center justify-center h-48 text-muted-foreground">
						Loading keyboard geometry...
					</div>
				{:else if geometryError}
					<div class="flex flex-col items-center justify-center h-48 text-destructive">
						<p class="mb-2">Failed to load keyboard geometry</p>
						<p class="text-sm text-muted-foreground">{geometryError}</p>
					</div>
				{:else if geometry}
					<KeyboardPreview
						geometry={geometry.keys}
						keyAssignments={currentLayerKeys}
						{selectedKeyIndex}
						onKeyClick={handleKeyClick}
						class="max-w-4xl mx-auto"
					/>
				{:else}
					<div class="flex items-center justify-center h-48 text-muted-foreground">
						No geometry data available. Ensure QMK firmware path is configured.
					</div>
				{/if}
			</Card>

			<!-- Key Details (when selected) -->
			{#if selectedKey}
				<Card class="p-6">
					<h2 class="text-xl font-semibold mb-4">Key Details</h2>
					<dl class="grid grid-cols-2 gap-4">
						<div>
							<dt class="text-sm font-medium text-muted-foreground">Keycode</dt>
							<dd class="text-sm font-mono">{selectedKey.keycode}</dd>
						</div>
						<div>
							<dt class="text-sm font-medium text-muted-foreground">Visual Index</dt>
							<dd class="text-sm">{selectedKey.visual_index}</dd>
						</div>
						<div>
							<dt class="text-sm font-medium text-muted-foreground">Matrix Position</dt>
							<dd class="text-sm">
								Row {selectedKey.matrix_position[0]}, Column {selectedKey.matrix_position[1]}
							</dd>
						</div>
						<div>
							<dt class="text-sm font-medium text-muted-foreground">LED Index</dt>
							<dd class="text-sm">{selectedKey.led_index}</dd>
						</div>
					</dl>
				</Card>
			{/if}
		</div>
	{/if}
</div>
