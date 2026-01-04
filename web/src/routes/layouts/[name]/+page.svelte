<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { apiClient, type Layout } from '$api';
	import { Button, Card } from '$components';

	let layout = $state<Layout | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let filename = $derived($page.params.name);

	onMount(async () => {
		try {
			layout = await apiClient.getLayout(filename);
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load layout';
		} finally {
			loading = false;
		}
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
		<Button onclick={() => (window.location.href = '/layouts')}>
			Back to Layouts
		</Button>
	</div>

	{#if loading}
		<p class="text-muted-foreground">Loading layout...</p>
	{:else if error}
		<Card class="p-6">
			<div class="text-destructive">
				<p class="font-medium">Error loading layout</p>
				<p class="text-sm">{error}</p>
			</div>
		</Card>
	{:else if layout}
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

			<!-- Layers Card -->
			<Card class="p-6">
				<h2 class="text-xl font-semibold mb-4">Layers</h2>
				<div class="space-y-2">
					{#each layout.layers as layer, i}
						<div class="flex items-center gap-4 p-3 rounded-lg border">
							<div
								class="w-4 h-4 rounded"
								style="background-color: {layer.color}"
							></div>
							<div class="flex-1">
								<p class="font-medium">{layer.name}</p>
								<p class="text-sm text-muted-foreground">
									{layer.keys.length} keys
								</p>
							</div>
							<span class="text-sm text-muted-foreground">Layer {i}</span>
						</div>
					{/each}
				</div>
			</Card>

			<!-- Editor Placeholder -->
			<Card class="p-6">
				<div class="text-center py-12">
					<h3 class="text-xl font-semibold mb-2">Visual Editor</h3>
					<p class="text-muted-foreground mb-4">
						The visual keyboard editor will be implemented here.
					</p>
					<p class="text-sm text-muted-foreground">
						For now, please use the TUI or CLI to edit layouts.
					</p>
				</div>
			</Card>
		</div>
	{/if}
</div>
