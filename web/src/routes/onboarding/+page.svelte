<svelte:head>
	<title>Welcome to LazyQMK</title>
</svelte:head>

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import {
		apiClient,
		type PreflightResponse,
		type TemplateInfo,
		type KeyboardInfo,
		type LayoutVariantInfo,
		type LayoutSummary
	} from '$api';
	import { Button, Card, Input } from '$components';

	// Preflight state
	let preflight = $state<PreflightResponse | null>(null);
	let preflightLoading = $state(true);
	let preflightError = $state<string | null>(null);

	// Step management
	type OnboardingStep = 'config' | 'choose' | 'template' | 'create';
	let currentStep = $state<OnboardingStep>('config');

	// Config step state
	let qmkPath = $state('');
	let configSaving = $state(false);
	let configError = $state<string | null>(null);

	// Templates state
	let templates = $state<TemplateInfo[]>([]);
	let templatesLoading = $state(false);
	let templatesError = $state<string | null>(null);
	let templateChoiceMessage = $state<string | null>(null);
	let selectedTemplate = $state<TemplateInfo | null>(null);
	let newLayoutName = $state('');
	let applyLoading = $state(false);
	let applyError = $state<string | null>(null);

	// Existing layouts state
	let existingLayouts = $state<LayoutSummary[]>([]);
	let existingLoading = $state(false);
	let existingError = $state<string | null>(null);

	// Create from scratch state (mini setup wizard)
	let keyboards = $state<KeyboardInfo[]>([]);
	let keyboardsLoading = $state(false);
	let keyboardsError = $state<string | null>(null);
	let keyboardSearch = $state('');
	let selectedKeyboard = $state<string | null>(null);
	let variants = $state<LayoutVariantInfo[]>([]);
	let variantsLoading = $state(false);
	let selectedVariant = $state<string | null>(null);
	let layoutName = $state('');
	let layoutFilename = $state('');
	let createLoading = $state(false);
	let createError = $state<string | null>(null);
	let keyboardBrowseMode = $state<'recognition' | 'search'>('recognition');

	// Derived state
	let filteredKeyboards = $derived(
		keyboardSearch
			? keyboards.filter((k) =>
					k.path.toLowerCase().includes(keyboardSearch.toLowerCase())
				)
			: keyboards
	);

	let qmkConfigured = $derived(preflight?.qmk_configured ?? false);
	let hasTemplates = $derived(templates.length > 0);
	let hasExistingLayouts = $derived(existingLayouts.length > 0);
	let currentStepNumber = $derived(currentStep === 'config' ? 1 : 2);
	let currentStepTitle = $derived(
		currentStep === 'config' ? 'Connect QMK' :
		currentStep === 'choose' ? 'Choose how to start' :
		currentStep === 'template' ? 'Create from template' :
		'Create from scratch'
	);

	const startOptions = {
		existing: { icon: 'OPEN', eyebrow: 'Resume work' },
		template: { icon: 'KIT', eyebrow: 'Starting point' },
		create: { icon: 'NEW', eyebrow: 'Blank layout' }
	};

	const featuredKeyboardGroups = $derived([
		{ title: 'Split and ergonomic', items: keyboards.filter((k) => /corne|crkbd|sofle|lily58|ergodox|split/i.test(k.path)).slice(0, 8) },
		{ title: 'Compact boards', items: keyboards.filter((k) => /40|42|min|planck|vial|ortho/i.test(k.path)).slice(0, 8) },
		{ title: 'Starter picks', items: keyboards.slice(0, 8) }
	].filter((group) => group.items.length > 0));

	onMount(async () => {
		await loadPreflight();
	});

	async function loadPreflight() {
		preflightLoading = true;
		try {
			preflight = await apiClient.preflight();
			qmkPath = preflight.qmk_firmware_path || '';

			// If QMK is configured, skip to choose step
			if (preflight.qmk_configured) {
				currentStep = 'choose';
				await Promise.all([loadTemplates(), loadExistingLayouts()]);
			}

			preflightError = null;
		} catch (e) {
			preflightError = e instanceof Error ? e.message : 'Failed to check application state';
		} finally {
			preflightLoading = false;
		}
	}

	async function saveQmkPath() {
		configSaving = true;
		configError = null;
		try {
			await apiClient.updateConfig({ qmk_firmware_path: qmkPath || undefined });
			// Reload preflight to verify
			preflight = await apiClient.preflight();
			if (preflight.qmk_configured) {
				currentStep = 'choose';
				await Promise.all([loadTemplates(), loadExistingLayouts()]);
			} else {
				configError = 'Path saved but QMK directory validation failed. Please check the path.';
			}
		} catch (e) {
			configError = e instanceof Error ? e.message : 'Failed to save QMK path';
		} finally {
			configSaving = false;
		}
	}

	async function loadTemplates() {
		templatesLoading = true;
		templatesError = null;
		templateChoiceMessage = null;
		try {
			const response = await apiClient.listTemplates();
			templates = response.templates;
			if (response.templates.length === 0) {
				templateChoiceMessage =
					'Templates are unavailable right now. Choose “From Scratch” instead of falling back automatically.';
			}
		} catch (e) {
			templatesError = e instanceof Error ? e.message : 'Failed to load templates';
			templateChoiceMessage =
				'Templates are unavailable right now. Choose “From Scratch” instead of falling back automatically.';
		} finally {
			templatesLoading = false;
		}
	}

	async function loadExistingLayouts() {
		existingLoading = true;
		existingError = null;
		try {
			const response = await apiClient.listLayouts();
			existingLayouts = response.layouts;
		} catch (e) {
			existingError = e instanceof Error ? e.message : 'Failed to load existing layouts';
		} finally {
			existingLoading = false;
		}
	}

	async function loadKeyboards() {
		keyboardsLoading = true;
		keyboardsError = null;
		try {
			const response = await apiClient.listKeyboards();
			keyboards = response.keyboards;
		} catch (e) {
			keyboardsError = e instanceof Error ? e.message : 'Failed to load keyboards';
		} finally {
			keyboardsLoading = false;
		}
	}

	async function loadVariants() {
		if (!selectedKeyboard) return;
		variantsLoading = true;
		try {
			const response = await apiClient.listKeyboardLayouts(selectedKeyboard);
			variants = response.variants;
			// Auto-select if only one variant
			if (variants.length === 1) {
				selectedVariant = variants[0].name;
			}
		} catch (e) {
			// Silently fail, user can retry
		} finally {
			variantsLoading = false;
		}
	}

	function selectTemplate(template: TemplateInfo) {
		selectedTemplate = template;
		newLayoutName = '';
		applyError = null;
		templateChoiceMessage = null;
		currentStep = 'template';
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

			// Navigate to the new layout editor
			goto(`/layouts/${encodeURIComponent(filename)}`);
		} catch (e) {
			applyError = e instanceof Error ? e.message : 'Failed to apply template';
		} finally {
			applyLoading = false;
		}
	}

	function startCreateFromScratch() {
		templateChoiceMessage = null;
		currentStep = 'create';
		loadKeyboards();
	}

	function handleTemplateEntry() {
		if (templatesLoading) return;
		if (hasTemplates) {
			templateChoiceMessage = null;
			document.getElementById('available-templates')?.scrollIntoView({ behavior: 'smooth' });
			return;
		}

		templateChoiceMessage =
			'Templates are unavailable right now. Choose “From Scratch” instead of falling back automatically.';
	}

	function selectKeyboard(path: string) {
		selectedKeyboard = path;
		selectedVariant = null;
		variants = [];
		loadVariants();
	}

	function useRecognitionMode() {
		keyboardBrowseMode = 'recognition';
	}

	function useSearchMode() {
		keyboardBrowseMode = 'search';
	}

	async function createLayout() {
		if (!selectedKeyboard || !selectedVariant || !layoutName.trim()) return;

		createLoading = true;
		createError = null;

		try {
			const filename = layoutFilename.trim() || layoutName.toLowerCase().replace(/[^a-z0-9]+/g, '_');
			const finalFilename = filename.endsWith('.md') ? filename : `${filename}.md`;

			await apiClient.createLayout({
				filename: finalFilename,
				name: layoutName,
				keyboard: selectedKeyboard,
				layout_variant: selectedVariant
			});

			// Navigate to the new layout editor
			goto(`/layouts/${encodeURIComponent(finalFilename)}`);
		} catch (e) {
			createError = e instanceof Error ? e.message : 'Failed to create layout';
		} finally {
			createLoading = false;
		}
	}

	// Auto-generate filename from name
	$effect(() => {
		if (layoutName && !layoutFilename) {
			layoutFilename = layoutName
				.toLowerCase()
				.replace(/[^a-z0-9]+/g, '_')
				.replace(/^_|_$/g, '');
		}
	});
</script>


<div class="min-h-screen bg-gradient-to-br from-background via-background to-primary/5 flex items-center justify-center p-6">
		<div class="w-full max-w-4xl">
			<!-- Header -->
			<div class="text-center mb-12 rounded-3xl border border-border/80 bg-gradient-to-br from-card via-card to-primary/10 px-6 py-10 shadow-sm">
				<div class="brand-badge mb-4">LazyQMK · Layout studio</div>
				<h1 class="text-5xl font-bold mb-4 bg-gradient-to-r from-foreground to-primary bg-clip-text text-transparent">
					Welcome to LazyQMK
				</h1>
				<p class="text-lg font-medium mb-2">Welcome to clearer keyboard setup</p>
				<p class="text-xl text-muted-foreground">
					Connect QMK once, pick how to start, then move straight into editing.
				</p>
			</div>

			{#if !preflightLoading && !preflightError}
				<Card class="surface-elevated p-5 mb-6">
					<div class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
						<div>
							<p class="text-sm font-medium text-primary">Canonical setup flow</p>
							<h2 class="text-lg font-semibold mt-1">Step {currentStepNumber} of 2 — {currentStepTitle}</h2>
							<p class="text-sm text-muted-foreground mt-1">
								Small orientation up front, faster handoff into real layout work.
							</p>
						</div>
						<div class="grid grid-cols-3 gap-2 text-xs md:text-sm">
							<div class="rounded-lg border px-3 py-2 {qmkConfigured ? 'border-primary/40 bg-primary/5 text-foreground' : 'text-muted-foreground'}">
								<p class="font-medium">1. Setup</p>
								<p>QMK path</p>
							</div>
							<div class="rounded-lg border px-3 py-2 {currentStep !== 'config' ? 'border-primary/40 bg-primary/5 text-foreground' : 'text-muted-foreground'}">
								<p class="font-medium">2. Start</p>
								<p>Template, scratch, open</p>
							</div>
							<div class="rounded-lg border px-3 py-2 text-muted-foreground">
								<p class="font-medium">3. Edit</p>
								<p>Inside layout workspace</p>
							</div>
						</div>
					</div>
				</Card>
			{/if}

		{#if preflightLoading}
			<!-- Loading State -->
			<Card class="p-12 text-center">
				<div class="animate-pulse">
					<div class="h-8 bg-muted rounded w-48 mx-auto mb-4"></div>
					<div class="h-4 bg-muted rounded w-64 mx-auto"></div>
				</div>
				<p class="text-muted-foreground mt-4">Checking application state...</p>
			</Card>
		{:else if preflightError}
			<!-- Error State -->
			<Card class="p-8 border-destructive">
				<h2 class="text-2xl font-semibold mb-4 text-destructive">Connection Error</h2>
				<p class="text-muted-foreground mb-6">{preflightError}</p>
				<Button onclick={loadPreflight}>Retry</Button>
			</Card>
		{:else if currentStep === 'config'}
			<!-- Step 1: QMK Configuration -->
			<Card class="surface-elevated p-8">
				<div class="flex items-center gap-4 mb-6">
					<div class="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center text-primary text-xl font-bold">
						1
					</div>
						<div>
							<h2 class="text-2xl font-semibold">Configure QMK Firmware</h2>
							<p class="text-muted-foreground">Set path to your local QMK firmware folder</p>
						</div>
				</div>

				<div class="space-y-4">
					<div>
						<label for="qmk-path" class="block text-sm font-medium mb-2">
							QMK firmware path
						</label>
						<Input
							id="qmk-path"
							bind:value={qmkPath}
							placeholder="/path/to/qmk_firmware"
							class="font-mono"
						/>
						<p class="text-xs text-muted-foreground mt-1">
							Point to folder that contains your QMK firmware clone.
						</p>
					</div>

					{#if configError}
						<div class="p-3 bg-destructive/10 border border-destructive rounded text-sm text-destructive">
							{configError}
						</div>
					{/if}

					<div class="flex justify-end pt-4">
						<Button onclick={saveQmkPath} disabled={configSaving || !qmkPath.trim()}>
							{configSaving ? 'Saving...' : 'Continue'}
						</Button>
					</div>
				</div>
			</Card>
		{:else if currentStep === 'choose'}
			<!-- Step 2: Choose Path -->
			<div class="space-y-6">
				<Card class="surface-elevated p-8">
					<div class="flex items-center gap-4 mb-6">
						<div class="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center text-primary text-xl font-bold">
							2
						</div>
						<div>
							<h2 class="text-2xl font-semibold">Choose your starting point</h2>
							<p class="text-muted-foreground">One place for opening existing work or creating a new layout.</p>
						</div>
					</div>

					<div class="surface-subtle rounded-lg p-4 mb-6 text-sm">
						<p class="font-medium mb-1">What happens next</p>
						<p class="text-muted-foreground">
							After layout opens, core editing stays in workspace tabs. Firmware tasks follow one steady Generate → Build flow.
						</p>
					</div>

					<div class="grid md:grid-cols-3 gap-6">
						<!-- Load Existing Layout -->
						{#if hasExistingLayouts}
							<button
								class="p-6 border-2 rounded-lg text-left hover:border-primary hover:bg-primary/5 transition-all group"
								onclick={() => {
									document.getElementById('your-layouts')?.scrollIntoView({ behavior: 'smooth' });
								}}
								disabled={existingLoading}
							>
								<div class="icon-chip mb-4">{startOptions.existing.icon}</div>
								<p class="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">{startOptions.existing.eyebrow}</p>
								<h3 class="text-xl font-semibold mb-2 group-hover:text-primary">
									Open Existing Layout
								</h3>
								<p class="text-sm text-muted-foreground">
									Continue working on your saved layouts
								</p>
								{#if existingLoading}
									<p class="text-xs text-muted-foreground mt-2">Loading layouts...</p>
								{:else}
									<p class="text-xs text-muted-foreground mt-2">
										{existingLayouts.length} {existingLayouts.length === 1 ? 'layout' : 'layouts'} found
									</p>
								{/if}
							</button>
						{/if}

					<!-- From Template -->
					<button
						class="p-6 border-2 rounded-lg text-left hover:border-primary hover:bg-primary/5 transition-all group"
						onclick={handleTemplateEntry}
						disabled={templatesLoading}
					>
							<div class="icon-chip mb-4">{startOptions.template.icon}</div>
							<p class="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">{startOptions.template.eyebrow}</p>
								<h3 class="text-xl font-semibold mb-2 group-hover:text-primary">
									Start from Template
							</h3>
							<p class="text-sm text-muted-foreground">
								Start with a pre-configured layout template and customize it
							</p>
							{#if templatesLoading}
								<p class="text-xs text-muted-foreground mt-2">Loading templates...</p>
							{:else if !hasTemplates}
								<p class="text-xs text-amber-600 dark:text-amber-400 mt-2">
									No templates available yet — no automatic fallback
								</p>
							{/if}
						</button>

						<!-- From Scratch -->
						<button
							class="p-6 border-2 rounded-lg text-left hover:border-primary hover:bg-primary/5 transition-all group"
							onclick={startCreateFromScratch}
						>
							<div class="icon-chip mb-4">{startOptions.create.icon}</div>
							<p class="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">{startOptions.create.eyebrow}</p>
								<h3 class="text-xl font-semibold mb-2 group-hover:text-primary">
									Start from Scratch
							</h3>
							<p class="text-sm text-muted-foreground">
								Create a new layout by selecting your keyboard model
							</p>
						</button>
					</div>

					{#if templateChoiceMessage}
						<div class="mt-6 rounded-lg border border-amber-500/40 bg-amber-500/10 p-4 text-sm text-amber-700 dark:text-amber-300">
							<p class="font-medium">Template start unavailable</p>
							<p class="mt-1">{templateChoiceMessage}</p>
						</div>
					{/if}
				</Card>

				<!-- Templates Grid (if available) -->
				{#if hasTemplates}
					<Card class="surface-elevated p-6">
						<h3 id="available-templates" class="text-lg font-semibold mb-4">Available Templates</h3>
						<div class="grid md:grid-cols-2 lg:grid-cols-3 gap-4">
							{#each templates as template}
								<button
									class="p-4 border rounded-lg text-left hover:border-primary hover:bg-primary/5 transition-all"
									onclick={() => selectTemplate(template)}
								>
									<h4 class="font-semibold mb-1">{template.name}</h4>
									<p class="text-xs text-muted-foreground mb-2 line-clamp-2">
										{template.description}
									</p>
									<div class="flex justify-between text-xs text-muted-foreground">
										<span>{template.layer_count} layers</span>
										<span>{template.author || 'Unknown'}</span>
									</div>
								</button>
							{/each}
						</div>
					</Card>
				{/if}

				<!-- Existing Layouts Grid (if available) -->
				{#if hasExistingLayouts}
					<Card class="surface-elevated p-6">
						<h3 id="your-layouts" class="text-lg font-semibold mb-4">Your Layouts</h3>
						<div class="grid md:grid-cols-2 lg:grid-cols-3 gap-4">
							{#each existingLayouts as layout}
								<button
									class="p-4 border rounded-lg text-left hover:border-primary hover:bg-primary/5 transition-all"
									onclick={() => goto(`/layouts/${encodeURIComponent(layout.filename)}`)}
								>
									<h4 class="font-semibold mb-1">{layout.name}</h4>
									<p class="text-xs text-muted-foreground mb-2 line-clamp-2">
										{layout.description || 'No description'}
									</p>
									<div class="flex justify-between text-xs text-muted-foreground">
										{#if layout.modified}
											<span>Modified {new Date(layout.modified).toLocaleDateString()}</span>
										{:else}
											<span>&nbsp;</span>
										{/if}
									</div>
								</button>
							{/each}
						</div>
					</Card>
				{/if}
			</div>
		{:else if currentStep === 'template'}
			<!-- Apply Template Step -->
			<Card class="surface-elevated p-8">
				<div class="flex items-center gap-4 mb-6">
					<button
						class="w-10 h-10 rounded-full border hover:bg-muted flex items-center justify-center"
						onclick={() => (currentStep = 'choose')}
						aria-label="Go back"
					>
						←
					</button>
					<div>
						<h2 class="text-2xl font-semibold">Apply Template</h2>
						<p class="text-muted-foreground">
							Creating from: <strong>{selectedTemplate?.name}</strong>
						</p>
					</div>
				</div>

				<div class="space-y-4">
					{#if selectedTemplate}
						<div class="surface-subtle p-4 rounded-lg">
							<h4 class="font-medium mb-1">{selectedTemplate.name}</h4>
							<p class="text-sm text-muted-foreground">{selectedTemplate.description}</p>
							<div class="flex gap-4 mt-2 text-xs text-muted-foreground">
								<span>{selectedTemplate.layer_count} layers</span>
								<span>by {selectedTemplate.author || 'Unknown'}</span>
							</div>
						</div>
					{/if}

					<div>
						<label for="new-layout-name" class="block text-sm font-medium mb-2">
							Layout Name
						</label>
						<Input
							id="new-layout-name"
							bind:value={newLayoutName}
							placeholder="my-awesome-layout"
						/>
						<p class="text-xs text-muted-foreground mt-1">
							Will be saved as: {newLayoutName.trim() || 'my-layout'}.md
						</p>
					</div>

					{#if applyError}
						<div class="p-3 bg-destructive/10 border border-destructive rounded text-sm text-destructive">
							{applyError}
						</div>
					{/if}

					<div class="flex justify-end gap-2 pt-4">
						<Button variant="outline" onclick={() => (currentStep = 'choose')}>
							Cancel
						</Button>
						<Button onclick={applyTemplate} disabled={applyLoading || !newLayoutName.trim()}>
							{applyLoading ? 'Creating...' : 'Create Layout'}
						</Button>
					</div>
				</div>

			</Card>
		{:else if currentStep === 'create'}
			<!-- Create from Scratch -->
			<Card class="surface-elevated p-8">
				<div class="flex items-center gap-4 mb-6">
					<button
						class="w-10 h-10 rounded-full border hover:bg-muted flex items-center justify-center"
						onclick={() => (currentStep = 'choose')}
						aria-label="Go back"
					>
						←
					</button>
					<div>
						<h2 class="text-2xl font-semibold">Create New Layout</h2>
						<p class="text-muted-foreground">Select your keyboard and configure the layout</p>
					</div>
				</div>

				<div class="space-y-6">
					<!-- Keyboard Selection -->
					<div>
						<div class="flex items-center justify-between gap-4 mb-2">
							<label for="keyboard-search" class="block text-sm font-medium">Keyboard</label>
							<div class="flex gap-2 text-xs">
								<button class="rounded-full border px-3 py-1 {keyboardBrowseMode === 'recognition' ? 'bg-primary text-primary-foreground border-primary' : 'text-muted-foreground'}" onclick={useRecognitionMode}>Recognize by board family</button>
								<button class="rounded-full border px-3 py-1 {keyboardBrowseMode === 'search' ? 'bg-primary text-primary-foreground border-primary' : 'text-muted-foreground'}" onclick={useSearchMode}>Search exact names</button>
							</div>
						</div>
						{#if keyboardsLoading}
							<p class="text-muted-foreground">Loading keyboards...</p>
						{:else if keyboardsError}
							<div class="p-3 bg-destructive/10 border border-destructive rounded text-sm text-destructive mb-2">
								{keyboardsError}
							</div>
							<Button onclick={loadKeyboards} size="sm">Retry</Button>
						{:else}
							<Input id="keyboard-search" bind:value={keyboardSearch} placeholder="Search keyboards by exact name..." class="mb-2 {keyboardBrowseMode === 'recognition' ? 'sr-only' : ''}" />
							{#if keyboardBrowseMode === 'recognition'}
								<div class="space-y-4">
									<p class="text-sm text-muted-foreground">
										Start with board families you recognize. Switch to exact search only if needed.
									</p>
									{#each featuredKeyboardGroups as group}
										<div>
											<p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground mb-2">{group.title}</p>
											<div class="grid gap-2 md:grid-cols-2">
												{#each group.items as keyboard}
													<button
														class="rounded-lg border p-3 text-left hover:bg-muted {selectedKeyboard === keyboard.path ? 'border-primary bg-primary/10' : ''}"
														onclick={() => selectKeyboard(keyboard.path)}
													>
														<p class="font-medium">{keyboard.path.split('/').at(-1)}</p>
														<p class="mt-1 text-xs text-muted-foreground font-mono">{keyboard.path}</p>
														<p class="mt-2 text-xs text-muted-foreground">{keyboard.layout_count} layout {keyboard.layout_count === 1 ? 'variant' : 'variants'} available</p>
													</button>
												{/each}
											</div>
										</div>
									{/each}
								</div>
							{:else}
								<div class="max-h-48 overflow-y-auto border rounded">
									{#if filteredKeyboards.length === 0}
										<p class="p-4 text-muted-foreground text-center text-sm">No keyboards found</p>
									{:else}
										{#each filteredKeyboards.slice(0, 50) as keyboard}
											<button
												class="w-full p-2 text-left hover:bg-muted border-b last:border-b-0 text-sm {selectedKeyboard === keyboard.path ? 'bg-primary/10 border-l-4 border-l-primary' : ''}"
												onclick={() => selectKeyboard(keyboard.path)}
											>
												<span class="font-mono">{keyboard.path}</span>
											</button>
										{/each}
										{#if filteredKeyboards.length > 50}
											<p class="p-2 text-xs text-muted-foreground text-center">Showing first 50 of {filteredKeyboards.length} results</p>
										{/if}
									{/if}
								</div>
							{/if}
						{/if}
					</div>

					<!-- Variant Selection -->
					{#if selectedKeyboard}
						<fieldset>
							<legend class="block text-sm font-medium mb-2">Layout Variant</legend>
							{#if variantsLoading}
								<p class="text-muted-foreground text-sm">Loading variants...</p>
							{:else if variants.length === 0}
								<p class="text-muted-foreground text-sm">No variants available</p>
							{:else}
								<div class="grid gap-2" role="radiogroup" aria-label="Layout variant selection">
									{#each variants as variant}
										<button
											class="p-3 border rounded text-left hover:bg-muted flex justify-between items-center
											{selectedVariant === variant.name ? 'bg-primary/10 border-primary border-2' : ''}"
											onclick={() => (selectedVariant = variant.name)}
											role="radio"
											aria-checked={selectedVariant === variant.name}
										>
											<span class="font-mono text-sm">{variant.name}</span>
											<span class="text-xs text-muted-foreground">{variant.key_count} keys</span>
										</button>
									{/each}
								</div>
							{/if}
						</fieldset>
					{/if}

					<!-- Layout Details -->
					{#if selectedVariant}
						<div class="space-y-4 pt-2 border-t">
							<div>
								<label for="layout-name-create" class="block text-sm font-medium mb-2">
									Layout Name
								</label>
								<Input
									id="layout-name-create"
									bind:value={layoutName}
									placeholder="My Custom Layout"
								/>
							</div>

							<div>
								<label for="layout-filename-create" class="block text-sm font-medium mb-2">
									Filename
								</label>
								<Input
									id="layout-filename-create"
									bind:value={layoutFilename}
									placeholder="my_custom_layout"
								/>
								<p class="text-xs text-muted-foreground mt-1">
									Will be saved as: {layoutFilename || 'filename'}.md
								</p>
							</div>
						</div>
					{/if}

					{#if createError}
						<div class="p-3 bg-destructive/10 border border-destructive rounded text-sm text-destructive">
							{createError}
						</div>
					{/if}

					<div class="flex justify-end gap-2 pt-4">
						<Button variant="outline" onclick={() => (currentStep = 'choose')}>
							Cancel
						</Button>
						<Button
							onclick={createLayout}
							disabled={createLoading || !selectedKeyboard || !selectedVariant || !layoutName.trim()}
						>
							{createLoading ? 'Creating...' : 'Create Layout'}
						</Button>
					</div>
				</div>
			</Card>
		{/if}

		{#if currentStep === 'config' || currentStep === 'choose'}
			<div class="text-center mt-8">
				<a href="/" class="text-sm text-muted-foreground hover:text-foreground">Go to Home</a>
			</div>
		{/if}
	</div>
</div>
