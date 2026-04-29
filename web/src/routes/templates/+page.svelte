<svelte:head>
	<title>Layout Templates - LazyQMK</title>
</svelte:head>

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { apiClient } from '$api';
	import { Button, Card, Input } from '$components';

	interface TemplateInfo {
		filename: string;
		name: string;
		description: string;
		author: string;
		tags: string[];
		created: string;
		layer_count: number;
	}

	let templates = $state<TemplateInfo[]>([]);
	let filteredTemplates = $state<TemplateInfo[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let searchQuery = $state('');
	let selectedTemplate = $state<TemplateInfo | null>(null);
	let showApplyDialog = $state(false);
	let newLayoutName = $state('');
	let applyError = $state<string | null>(null);
	let applyLoading = $state(false);

	onMount(async () => {
		await loadTemplates();
	});

	async function loadTemplates() {
		loading = true;
		error = null;
		try {
			const response = await apiClient.listTemplates();
			templates = response.templates;
			filteredTemplates = templates;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load templates';
		} finally {
			loading = false;
		}
	}

	function filterTemplates() {
		const query = searchQuery.toLowerCase();
		filteredTemplates = templates.filter(
			(t) =>
				t.name.toLowerCase().includes(query) ||
				t.description.toLowerCase().includes(query) ||
				t.tags.some((tag: string) => tag.toLowerCase().includes(query))
		);
	}

	$effect(() => {
		searchQuery;
		filterTemplates();
	});

	function openApplyDialog(template: TemplateInfo) {
		selectedTemplate = template;
		newLayoutName = '';
		applyError = null;
		showApplyDialog = true;
	}

	function closeApplyDialog() {
		showApplyDialog = false;
		selectedTemplate = null;
		newLayoutName = '';
		applyError = null;
	}

	async function applyTemplate() {
		if (!selectedTemplate || !newLayoutName.trim()) {
			applyError = 'Please enter a layout name';
			return;
		}

		applyLoading = true;
		applyError = null;

		try {
			const filename = newLayoutName.trim().endsWith('.md')
				? newLayoutName.trim()
				: `${newLayoutName.trim()}.md`;

			await apiClient.applyTemplate(selectedTemplate.filename, {
				target_filename: filename
			});

			// Navigate to the new layout
			goto(`/layouts/${encodeURIComponent(filename)}`);
		} catch (e) {
			applyError = e instanceof Error ? e.message : 'Failed to apply template';
		} finally {
			applyLoading = false;
		}
	}

	function formatDate(dateStr: string): string {
		try {
			const date = new Date(dateStr);
			return date.toLocaleDateString();
		} catch {
			return dateStr;
		}
	}
</script>

<div class="container mx-auto p-6">
	<div class="mb-8">
		<h1 class="text-4xl font-bold mb-2">Starter Layouts</h1>
		<p class="text-muted-foreground">Start from proven layouts, then tailor them to your board and workflow.</p>
	</div>

	<!-- Search Bar -->
	<div class="mb-6">
		<Input
			type="text"
			placeholder="Search starter layouts by name, purpose, or tags..."
			bind:value={searchQuery}
			class="max-w-md"
		/>
	</div>

	<!-- Loading State -->
	{#if loading}
		<div data-testid="loading-state">
			<Card class="state-panel-loading">
				<p class="state-eyebrow mb-3">Starter layouts</p>
				<h2 class="text-2xl font-semibold">Loading templates</h2>
				<p class="mt-2 text-muted-foreground">Gathering reusable starting points for your next layout.</p>
			</Card>
		</div>
	{:else if error}
		<!-- Error State -->
		<div data-testid="error-state">
			<Card class="state-panel-error">
				<p class="state-eyebrow mb-3">Starter layouts unavailable</p>
				<h2 class="text-2xl font-semibold text-destructive">Could not load templates</h2>
				<p class="mt-2 text-sm text-muted-foreground">{error}</p>
				<div class="mt-6 flex gap-2">
					<Button onclick={loadTemplates}>Retry</Button>
					<a href="/onboarding"><Button variant="outline">Start from Scratch</Button></a>
				</div>
			</Card>
		</div>
	{:else if filteredTemplates.length === 0}
		<!-- Empty State -->
		<div data-testid="empty-state">
			<Card class="state-panel-empty">
				<p class="state-eyebrow mb-3">No starter layouts</p>
				<h2 class="text-2xl font-semibold">{searchQuery ? 'Try broader search terms' : 'Create next reusable starting point'}</h2>
				<p class="mt-2 text-muted-foreground mb-6">
					{searchQuery
						? 'No template matches this search yet. Clear filters or try another keyword.'
						: 'Save one of your existing layouts as a template so future boards start faster.'}
				</p>
				<div class="flex justify-center gap-2">
					{#if searchQuery}
						<Button variant="outline" onclick={() => (searchQuery = '')}>Clear Search</Button>
					{/if}
					<a href="/layouts">
						<Button>{searchQuery ? 'Browse Layouts' : 'Open Layouts'}</Button>
					</a>
				</div>
			</Card>
		</div>
	{:else}
		<!-- Templates Grid -->
		<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3" data-testid="templates-grid">
			{#each filteredTemplates as template}
				<Card class="p-6 hover:border-primary transition-colors">
					<div class="mb-4">
						<h3 class="text-xl font-semibold mb-2">{template.name}</h3>
						<p class="text-sm text-muted-foreground mb-3">{template.description}</p>

						<!-- Metadata -->
						<div class="space-y-1 text-xs text-muted-foreground mb-3">
							<div>
								<span class="font-medium">Layers:</span>
								{template.layer_count}
							</div>
							<div>
								<span class="font-medium">Author:</span>
								{template.author || 'Unknown'}
							</div>
							<div>
								<span class="font-medium">Created:</span>
								{formatDate(template.created)}
							</div>
						</div>

						<!-- Tags -->
						{#if template.tags.length > 0}
							<div class="flex flex-wrap gap-2 mb-4">
								{#each template.tags as tag}
									<span
										class="px-2 py-1 text-xs rounded bg-secondary text-secondary-foreground"
									>
										{tag}
									</span>
								{/each}
							</div>
						{/if}
					</div>

					<!-- Actions -->
					<div class="flex gap-2">
						<Button onclick={() => openApplyDialog(template)} class="flex-1">
							Use as Starting Point
						</Button>
					</div>
				</Card>
			{/each}
		</div>
	{/if}

	<!-- Apply Template Dialog -->
	{#if showApplyDialog && selectedTemplate}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
			onclick={closeApplyDialog}
		>
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div onclick={(e: MouseEvent) => e.stopPropagation()}>
				<Card class="p-6 max-w-md w-full">
					<h2 class="text-2xl font-bold mb-4">Apply Template</h2>
					<p class="text-sm text-muted-foreground mb-4">
						Create a new layout from template: <strong>{selectedTemplate.name}</strong>
					</p>

					<div class="mb-4">
						<label for="layout-name" class="block text-sm font-medium mb-2">
							New Layout Name
						</label>
						<Input
							id="layout-name"
							type="text"
							placeholder="my-awesome-layout"
							bind:value={newLayoutName}
							class="w-full"
						/>
						<p class="text-xs text-muted-foreground mt-1">
							Will be saved as: {newLayoutName.trim() || 'my-layout'}.md
						</p>
					</div>

					{#if applyError}
						<div class="mb-4 p-3 bg-destructive/10 text-destructive text-sm rounded">
							{applyError}
						</div>
					{/if}

					<div class="flex gap-2">
						<Button
							onclick={applyTemplate}
							disabled={applyLoading || !newLayoutName.trim()}
							class="flex-1"
						>
							{applyLoading ? 'Creating...' : 'Create Layout'}
						</Button>
						<Button
							onclick={closeApplyDialog}
							disabled={applyLoading}
							class="flex-1"
							variant="ghost"
						>
							Cancel
						</Button>
					</div>
				</Card>
			</div>
		</div>
	{/if}
</div>
