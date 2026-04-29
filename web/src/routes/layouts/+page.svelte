<svelte:head>
	<title>Layouts - LazyQMK</title>
</svelte:head>

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
			<h1 class="text-4xl font-bold mb-2">My Layouts</h1>
			<p class="text-muted-foreground">
				Pick layout to edit, inspect, generate, or build firmware.
			</p>
		</div>
		<a href="/onboarding">
			<Button>Create New Layout</Button>
		</a>
	</div>

	<Card class="surface-subtle p-4 mb-6">
		<div class="grid gap-4 md:grid-cols-3 text-sm">
			<div>
				<p class="font-medium mb-1">Edit layout</p>
				<p class="text-muted-foreground">Change keys, layers, combos, metadata, and visual behavior.</p>
			</div>
			<div>
				<p class="font-medium mb-1">Firmware workflow</p>
				<p class="text-muted-foreground">Inside layout editor, run guided Generate → Build workflow.</p>
			</div>
			<div>
				<p class="font-medium mb-1">Need new layout?</p>
				<p class="text-muted-foreground">Unified setup flow handles QMK path, templates, and scratch starts.</p>
			</div>
		</div>
	</Card>

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
				No layouts found yet. Start with unified setup flow to create one.
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
						<a href="/layouts/{layout.filename}">
							<Button size="sm">Open</Button>
						</a>
					</div>
				</Card>
			{/each}
		</div>
	{/if}
</div>
