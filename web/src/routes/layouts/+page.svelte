<script lang="ts">
	import { onMount } from 'svelte';
	import { apiClient, type LayoutSummary } from '$api';
	import { Button, Card } from '$components';

	let layouts = $state<LayoutSummary[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			const response = await apiClient.listLayouts();
			layouts = response.layouts;
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load layouts';
		} finally {
			loading = false;
		}
	});

	function formatDate(isoDate: string): string {
		return new Date(isoDate).toLocaleDateString(undefined, {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}
</script>

<div class="container mx-auto p-6">
	<div class="mb-8 flex items-center justify-between">
		<div>
			<h1 class="text-4xl font-bold mb-2">Layouts</h1>
			<p class="text-muted-foreground">
				Manage your keyboard layouts
			</p>
		</div>
		<Button onclick={() => (window.location.href = '/')}>
			Back to Dashboard
		</Button>
	</div>

	{#if loading}
		<p class="text-muted-foreground">Loading layouts...</p>
	{:else if error}
		<Card class="p-6">
			<div class="text-destructive">
				<p class="font-medium">Error loading layouts</p>
				<p class="text-sm">{error}</p>
			</div>
		</Card>
	{:else if layouts.length === 0}
		<Card class="p-6">
			<p class="text-muted-foreground">
				No layouts found. Create one using the CLI or TUI.
			</p>
		</Card>
	{:else}
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each layouts as layout}
				<Card class="p-6 hover:border-primary transition-colors">
					<h3 class="text-lg font-semibold mb-2">{layout.name}</h3>
					<p class="text-sm text-muted-foreground mb-4">
						{layout.description}
					</p>
					<div class="flex items-center justify-between">
						<p class="text-xs text-muted-foreground">
							Modified: {formatDate(layout.modified)}
						</p>
						<Button
							size="sm"
							onclick={() => (window.location.href = `/layouts/${layout.filename}`)}
						>
							Open
						</Button>
					</div>
				</Card>
			{/each}
		</div>
	{/if}
</div>
