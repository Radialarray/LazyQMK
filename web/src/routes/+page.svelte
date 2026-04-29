<svelte:head>
	<title>LazyQMK</title>
</svelte:head>

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { apiClient, type PreflightResponse, type LayoutSummary } from '$api';
	import { Button, Card } from '$components';
	import { getRecentLayouts, filterValidRecentLayouts } from '$lib/utils/recentLayouts';

	let preflight = $state<PreflightResponse | null>(null);
	let recentLayouts = $state<LayoutSummary[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let totalLayouts = $state(0);

	// Maximum number of recent layouts to show
	const MAX_RECENT_LAYOUTS = 5;

	onMount(async () => {
		try {
			// Check preflight first
			preflight = await apiClient.preflight();

			// If first run (no layouts and no QMK config), redirect to onboarding
			if (preflight.first_run) {
				goto('/onboarding');
				return;
			}

			// If QMK is not configured but has layouts, redirect to onboarding for setup
			if (!preflight.qmk_configured) {
				goto('/onboarding');
				return;
			}

			// Load all layouts from backend
			const response = await apiClient.listLayouts();
			totalLayouts = response.layouts.length;
			
			// Get recent layouts from localStorage (tracks when user actually opened them)
			const storedRecents = getRecentLayouts();
			
			if (storedRecents.length > 0) {
				// Validate stored recents against backend (filter out deleted layouts)
				const validFilenames = new Set(response.layouts.map(l => l.filename));
				const validRecents = filterValidRecentLayouts(storedRecents, validFilenames);
				
				// Map valid recent layout filenames to full LayoutSummary objects
				const recentMap = new Map(response.layouts.map(l => [l.filename, l]));
				recentLayouts = validRecents
					.map(r => recentMap.get(r.filename))
					.filter((l): l is LayoutSummary => l !== undefined)
					.slice(0, MAX_RECENT_LAYOUTS);
			} else {
				// Fall back to most recently modified layouts if no localStorage data
				recentLayouts = response.layouts
					.sort((a, b) => new Date(b.modified).getTime() - new Date(a.modified).getTime())
					.slice(0, MAX_RECENT_LAYOUTS);
			}

			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to connect to backend';
		} finally {
			loading = false;
		}
	});

	function formatDate(isoDate: string): string {
		const date = new Date(isoDate);
		const now = new Date();
		const diffMs = now.getTime() - date.getTime();
		const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

		if (diffDays === 0) {
			return 'Today';
		} else if (diffDays === 1) {
			return 'Yesterday';
		} else if (diffDays < 7) {
			return `${diffDays} days ago`;
		} else {
			return date.toLocaleDateString(undefined, {
				year: 'numeric',
				month: 'short',
				day: 'numeric'
			});
		}
	}
</script>

<div class="container mx-auto p-6">
	{#if loading}
		<div class="flex items-center justify-center h-64">
			<Card class="state-panel-loading max-w-xl w-full">
				<p class="state-eyebrow mb-3">Workspace</p>
				<h2 class="text-2xl font-semibold">Loading home workspace</h2>
				<p class="mt-2 text-muted-foreground">Checking setup, recent layouts, and editor entry points.</p>
			</Card>
		</div>
	{:else if error}
		<div class="max-w-2xl mx-auto">
			<Card class="state-panel-error">
				<p class="state-eyebrow mb-3">Backend unavailable</p>
				<h2 class="text-2xl font-semibold text-destructive">Could not connect to LazyQMK</h2>
				<p class="mt-2 text-muted-foreground mb-6">{error}</p>
				<Button onclick={() => window.location.reload()}>Retry</Button>
			</Card>
		</div>
	{:else}
		<!-- Layout-focused home page -->
		<div class="max-w-4xl mx-auto">
			<!-- Header -->
			<div class="text-center mb-12 rounded-3xl border border-border/80 bg-gradient-to-br from-card via-card to-primary/5 px-6 py-10 shadow-sm">
				<div class="brand-badge mb-4">LazyQMK · Web editor</div>
				<h1 class="text-4xl font-bold mb-2">LazyQMK</h1>
				<p class="text-lg font-medium mt-2">Build cleaner QMK layouts, faster</p>
				<p class="text-muted-foreground">
					Calm workflow for setup, editing, and firmware handoff.
				</p>
			</div>

			<!-- Primary Actions -->
			<div class="grid gap-6 mb-12 lg:grid-cols-[1.2fr_0.8fr]" data-testid="primary-actions">
				<div class="space-y-6">
					<a href="/onboarding" class="block" data-testid="create-layout-action">
					<Card class="option-card p-8 h-full cursor-pointer">
						<div class="icon-chip mb-4">NEW</div>
						<h2 class="text-xl font-semibold mb-2">Start Layout Setup</h2>
						<p class="text-muted-foreground">
							Use one guided flow for QMK setup, templates, and new layouts.
						</p>
						<p class="text-sm text-primary mt-4">Setup QMK → choose template or scratch → open editor</p>
					</Card>
					</a>

					<a href="/layouts" class="block" data-testid="open-layout-action">
					<Card class="option-card p-8 h-full cursor-pointer">
						<div class="icon-chip mb-4">OPEN</div>
						<h2 class="text-xl font-semibold mb-2">Open Layout Workspace</h2>
						<p class="text-muted-foreground">
							Browse saved layouts and jump back into editing.
						</p>
					</Card>
					</a>
				</div>

				<Card class="surface-subtle p-6 h-full">
					<h2 class="text-lg font-semibold mb-4">Workspace dashboard</h2>
					<div class="space-y-4 text-sm">
						<div class="rounded-lg border border-border bg-background/70 px-4 py-3">
							<p class="font-medium">Current state</p>
							<p class="text-muted-foreground mt-1">
								{totalLayouts === 0
									? 'No layouts yet. Guided setup is best next step.'
									: `${totalLayouts} saved ${totalLayouts === 1 ? 'layout is' : 'layouts are'} ready to open.`}
							</p>
						</div>
						<div>
							<p class="font-medium">Create</p>
							<p class="text-muted-foreground">Onboarding, templates, and keyboard setup in one place.</p>
						</div>
						<div>
							<p class="font-medium">Edit</p>
							<p class="text-muted-foreground">Core editor tools live inside each layout workspace.</p>
						</div>
						<div>
							<p class="font-medium">Firmware workflow</p>
							<p class="text-muted-foreground">Generate sources first, then build flashable firmware from one guided path.</p>
						</div>
						<div>
							<p class="font-medium">Helpful side paths</p>
							<div class="mt-2 flex flex-wrap gap-2">
								<a href="/templates" class="rounded-full border px-3 py-1 text-xs hover:bg-accent">Browse templates</a>
								<a href="/keycodes" class="rounded-full border px-3 py-1 text-xs hover:bg-accent">Learn keycodes</a>
								<a href="/settings" class="rounded-full border px-3 py-1 text-xs hover:bg-accent">Workspace settings</a>
							</div>
						</div>
					</div>
				</Card>
			</div>

			<!-- Recent Layouts -->
			<div data-testid="recent-layouts">
				<div class="flex items-center justify-between mb-4">
					<h2 class="text-xl font-semibold">Recent Layouts</h2>
					{#if recentLayouts.length > 0}
						<a href="/layouts" class="text-sm text-primary hover:underline">
							View all
						</a>
					{/if}
				</div>

				{#if recentLayouts.length === 0}
					<Card class="state-panel-empty">
						<p class="state-eyebrow mb-3">Nothing recent</p>
						<h3 class="text-xl font-semibold">Create first layout</h3>
						<p class="mt-2 text-muted-foreground text-center">
							Start setup flow to connect QMK and open first editable layout.
						</p>
						<div class="mt-6">
							<a href="/onboarding"><Button>Create Layout</Button></a>
						</div>
					</Card>
				{:else}
					<div class="space-y-2">
						{#each recentLayouts as layout}
							<a href="/layouts/{encodeURIComponent(layout.filename)}" class="block" data-testid="recent-layout-item">
								<Card class="p-4 hover:border-primary transition-colors cursor-pointer">
									<div class="flex items-center justify-between">
										<div class="min-w-0 flex-1">
											<h3 class="font-medium truncate">{layout.name}</h3>
											<p class="text-sm text-muted-foreground truncate">
												{layout.description || 'No description'}
											</p>
										</div>
										<div class="text-xs text-muted-foreground ml-4 whitespace-nowrap">
											{formatDate(layout.modified)}
										</div>
									</div>
								</Card>
							</a>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>
