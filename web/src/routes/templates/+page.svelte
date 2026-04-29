<svelte:head>
	<title>Layout Templates - LazyQMK</title>
</svelte:head>

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { apiClient } from '$api';
	import { AccessibleDialog, Button, Card, Input } from '$components';

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

	function getUseCaseCue(template: TemplateInfo): string {
		const text = `${template.name} ${template.description} ${template.tags.join(' ')}`.toLowerCase();
		if (text.includes('gaming')) return 'Good for gaming-focused boards';
		if (text.includes('minimal') || text.includes('42') || text.includes('40')) return 'Good for compact minimal layouts';
		if (text.includes('split') || text.includes('corne') || text.includes('erg')) return 'Good for split ergonomic boards';
		return 'Good general-purpose starting point';
	}

	function getCompatibilityCue(template: TemplateInfo): string {
		if (template.layer_count >= 6) return 'Best when you rely on many layers';
		if (template.layer_count >= 3) return 'Balanced for daily multi-layer use';
		return 'Best for simple low-layer workflows';
	}
</script>

<div class="container mx-auto p-6">
	<div class="mb-8">
		<h1 class="text-4xl font-bold mb-2">Layout Templates</h1>
		<p class="text-lg font-medium mb-1">Starter Layouts</p>
		<p class="text-muted-foreground">Start from proven layouts, then tailor them to your board and workflow.</p>
	</div>

	<Card class="surface-subtle mb-6 p-4">
		<div class="grid gap-3 md:grid-cols-3 text-sm">
			<div>
				<p class="font-medium">Use-case cues</p>
				<p class="text-muted-foreground">Cards call out who each template fits best.</p>
			</div>
			<div>
				<p class="font-medium">Compatibility cues</p>
				<p class="text-muted-foreground">Layer depth hints show how much complexity each start point assumes.</p>
			</div>
			<div>
				<p class="font-medium">Fast handoff</p>
				<p class="text-muted-foreground">Choose one, name layout, then continue inside editor.</p>
			</div>
		</div>
	</Card>

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
						<div class="space-y-2 mb-4 text-xs">
							<div class="rounded-lg border border-border bg-muted/20 px-3 py-2">
								<p class="font-medium text-foreground">Use case</p>
								<p class="mt-1 text-muted-foreground">{getUseCaseCue(template)}</p>
							</div>
							<div class="rounded-lg border border-border bg-muted/20 px-3 py-2">
								<p class="font-medium text-foreground">Compatibility</p>
								<p class="mt-1 text-muted-foreground">{getCompatibilityCue(template)}</p>
							</div>
						</div>

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
		<AccessibleDialog
			open={showApplyDialog}
			title="Apply template"
			description={`Create new layout from ${selectedTemplate.name}.`}
			onClose={closeApplyDialog}
			titleId="apply-template-title"
		>
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

			<svelte:fragment slot="footer">
				<Button onclick={closeApplyDialog} disabled={applyLoading} variant="ghost">Cancel</Button>
				<Button onclick={applyTemplate} disabled={applyLoading || !newLayoutName.trim()}>
					{applyLoading ? 'Creating...' : 'Create Layout'}
				</Button>
			</svelte:fragment>
		</AccessibleDialog>
	{/if}
</div>
