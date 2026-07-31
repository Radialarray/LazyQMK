<script lang="ts">
	import { Card } from '$components';
	import type { LayerRefLayer, LayerRefWarning } from '$api/types';

	interface Props {
		layers: LayerRefLayer[];
		totalInbound: number;
		totalWarnings: number;
	}

	let { layers, totalInbound, totalWarnings }: Props = $props();

	const layersWithRefs = $derived(layers.filter((l) => l.inbound_count > 0));
	const layersWithWarnings = $derived(layers.filter((l) => l.warnings.length > 0));
	const orphans = $derived(layers.filter((l) => l.inbound_count === 0 && l.number > 0));
</script>

<Card class="p-6" data-testid="layer-refs-panel">
	<div class="mb-4 flex items-center justify-between gap-4">
		<div>
			<p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
				Cross-Layer Analysis
			</p>
			<h2 class="text-xl font-semibold mt-1">Layer References & Transparency Warnings</h2>
		</div>
		<div class="flex gap-2 text-xs">
			<span class="rounded-full border px-3 py-1 text-muted-foreground" data-testid="layer-refs-total">
				{totalInbound} inbound {totalInbound === 1 ? 'ref' : 'refs'}
			</span>
			{#if totalWarnings > 0}
				<span
					class="rounded-full border border-destructive/40 bg-destructive/10 px-3 py-1 text-destructive"
					data-testid="layer-refs-warnings-badge"
				>
					{totalWarnings} warning{totalWarnings === 1 ? '' : 's'}
				</span>
			{:else}
				<span
					class="rounded-full border border-green-500/40 bg-green-500/10 px-3 py-1 text-green-700 dark:text-green-300"
				>
					No conflicts
				</span>
			{/if}
		</div>
	</div>

	<p class="text-sm text-muted-foreground mb-6">
		Inbound references show which keys on other layers activate this layer (via
		<kbd class="kbd-token">MO</kbd>/<kbd class="kbd-token">LT</kbd>/<kbd class="kbd-token">TG</kbd>/...).
		Warnings flag positions where a hold-like reference lands on a non-transparent key, which
		will trigger the wrong keycode while the layer is held.
	</p>

	{#if totalInbound === 0 && totalWarnings === 0}
		<div class="rounded-lg border border-border bg-muted/20 p-4 text-sm text-muted-foreground">
			This layout has no layer-switching keycodes and no transparency conflicts. Layer 0
			(Base) is always accessible; other layers are reachable only by direct selection.
		</div>
	{:else}
		{#if layersWithWarnings.length > 0}
			<div class="mb-6 space-y-3" data-testid="layer-refs-warnings-section">
				<h3 class="text-sm font-semibold">Transparency Conflicts</h3>
				{#each layersWithWarnings as layer (layer.number)}
					<div
						class="rounded-lg border border-destructive/40 bg-destructive/5 p-4"
						data-testid="layer-refs-warning-block"
					>
						<p class="text-sm font-medium">
							Layer {layer.number}
							<span class="text-muted-foreground font-normal">— {layer.name}</span>
						</p>
						<ul class="mt-2 space-y-1.5 text-sm">
							{#each layer.warnings as warning: LayerRefWarning (warning.from_layer + '-' + warning.row + '-' + warning.col)}
								<li
									class="text-destructive"
									data-testid="layer-refs-warning-row"
								>
									Position
									<code class="bg-muted text-foreground px-1 rounded"
										>[{warning.row},{warning.col}]</code
									>
									holds
									<code class="bg-muted text-foreground px-1 rounded">{warning.keycode}</code
									>
									but Layer {layer.number} has
									<code class="bg-muted text-foreground px-1 rounded"
										>{warning.target_keycode}</code
									>
									there — switch the slot to <kbd class="kbd-token">KC_TRNS</kbd> to avoid
									triggering both.
								</li>
							{/each}
						</ul>
					</div>
				{/each}
			</div>
		{/if}

		{#if layersWithRefs.length > 0}
			<div class="space-y-3" data-testid="layer-refs-inbound-section">
				<h3 class="text-sm font-semibold">Inbound References</h3>
				{#each layersWithRefs as layer (layer.number)}
					<details class="rounded-lg border border-border bg-background" open>
						<summary
							class="cursor-pointer px-4 py-2 text-sm font-medium flex items-center justify-between"
							data-testid="layer-refs-layer-summary"
						>
							<span>
								Layer {layer.number}
								<span class="text-muted-foreground font-normal">— {layer.name}</span>
							</span>
							<span class="text-xs text-muted-foreground">
								{layer.inbound_count} {layer.inbound_count === 1 ? 'ref' : 'refs'}
							</span>
						</summary>
						<ul class="border-t border-border divide-y divide-border/60">
							{#each layer.inbound_refs as ref (ref.from_layer + '-' + ref.row + '-' + ref.col)}
								<li
									class="px-4 py-2 text-sm flex items-center justify-between gap-3"
									data-testid="layer-refs-inbound-row"
								>
									<div class="flex items-center gap-3 min-w-0">
										<span
											class="text-xs font-mono bg-muted text-muted-foreground px-2 py-0.5 rounded shrink-0"
										>
											L{ref.from_layer} [{ref.row},{ref.col}]
										</span>
										<code
											class="font-mono text-xs bg-background border border-border px-1.5 py-0.5 rounded truncate"
										>
											{ref.keycode}
										</code>
									</div>
									<span class="text-xs text-muted-foreground shrink-0">{ref.kind}</span>
								</li>
							{/each}
						</ul>
					</details>
				{/each}
			</div>
		{/if}

		{#if orphans.length > 0}
			<div class="mt-6 rounded-lg border border-border bg-muted/20 p-4 text-sm text-muted-foreground">
				<p class="font-medium text-foreground">Orphan layers (no inbound refs)</p>
				<p class="mt-1">
					{orphans.map((l) => `L${l.number} (${l.name})`).join(', ')} {orphans.length === 1
						? 'is'
						: 'are'} only reachable by direct selection — no key on another layer
					switches to {orphans.length === 1 ? 'it' : 'them'}.
				</p>
			</div>
		{/if}
	{/if}
</Card>