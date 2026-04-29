<script lang="ts">
	import { Button, Input } from '$components';
	import { apiClient } from '$api';
	import type { KeycodeInfo, CategoryInfo } from '$api/types';

	interface Props {
		/** Whether the picker is open */
		open: boolean;
		/** Callback when the picker is closed */
		onClose: () => void;
		/** Callback when a keycode is selected */
		onSelect: (keycode: string) => void;
		/** Optional current keycode to highlight */
		currentKeycode?: string;
	}

	let { open = $bindable(), onClose, onSelect, currentKeycode = '' }: Props = $props();

	// State
	let searchQuery = $state('');
	let selectedCategory = $state<string>('');
	let keycodes = $state<KeycodeInfo[]>([]);
	let categories = $state<CategoryInfo[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	const quickFilters = ['basic', 'modifier', 'layer', 'media', 'rgb'];

	const filteredCategories = $derived(
		quickFilters
			.map((id) => categories.find((category) => category.id === id))
			.filter((category): category is CategoryInfo => Boolean(category))
	);

	const activeCategory = $derived(
		categories.find((category) => category.id === selectedCategory) ?? null
	);

	const keycodeCountLabel = $derived(
		loading ? 'Loading results…' : `${keycodes.length} ${keycodes.length === 1 ? 'result' : 'results'}`
	);

	// Load categories on mount
	$effect(() => {
		if (open && categories.length === 0) {
			loadCategories();
		}
	});

	// Load keycodes when search or category changes
	$effect(() => {
		if (open) {
			loadKeycodes();
		}
	});

	async function loadCategories() {
		try {
			const response = await apiClient.listCategories();
			categories = response.categories;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load categories';
		}
	}

	async function loadKeycodes() {
		loading = true;
		error = null;
		try {
			const response = await apiClient.listKeycodes(
				searchQuery || undefined,
				selectedCategory || undefined
			);
			keycodes = response.keycodes;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load keycodes';
			keycodes = [];
		} finally {
			loading = false;
		}
	}

	function handleSelectKeycode(keycode: string) {
		onSelect(keycode);
		onClose();
	}

	function handleClear() {
		onSelect('KC_TRNS');
		onClose();
	}

	function handleClose() {
		onClose();
	}

	function resetFilters() {
		searchQuery = '';
		selectedCategory = '';
	}

	function handleCategoryChange(categoryId: string) {
		selectedCategory = categoryId;
	}

	function handleSearchInput(event: Event) {
		const target = event.target as HTMLInputElement;
		searchQuery = target.value;
	}

	// Close on Escape key
	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			handleClose();
		}
	}
</script>

{#if open}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={handleClose}
		onkeydown={handleKeydown}
		role="dialog"
		aria-modal="true"
		aria-labelledby="keycode-picker-title"
		data-testid="keycode-picker-overlay"
		tabindex="-1"
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="bg-background border border-border rounded-lg shadow-lg w-full max-w-6xl max-h-[80vh] flex flex-col"
			onclick={(e) => e.stopPropagation()}
			data-testid="keycode-picker-dialog"
		>
			<!-- Header -->
			<div class="flex items-start justify-between gap-4 p-4 border-b border-border">
				<div>
					<h2 id="keycode-picker-title" class="text-lg font-semibold">Choose key behavior</h2>
					<p class="mt-1 text-sm text-muted-foreground">
						Search by code or plain-language meaning. Start with quick groups, then refine.
					</p>
				</div>
				<Button variant="ghost" size="icon" onclick={handleClose} title="Close">
					<span class="text-xl">✕</span>
				</Button>
			</div>

			<!-- Search and Filters -->
			<div class="p-4 border-b border-border space-y-4">
				<div class="grid gap-3 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-end">
					<div>
						<label for="keycode-search" class="mb-2 block text-sm font-medium">Search</label>
						<Input
							id="keycode-search"
							type="text"
							placeholder="Try: enter, control, rgb, layer…"
							value={searchQuery}
							oninput={handleSearchInput}
							class="w-full"
							data-testid="keycode-search-input"
						/>
					</div>
					<div class="flex items-center gap-2 text-sm text-muted-foreground">
						<span data-testid="keycode-results-count">{keycodeCountLabel}</span>
						{#if searchQuery || selectedCategory}
							<Button variant="ghost" size="sm" onclick={resetFilters} data-testid="keycode-reset-filters">
								Reset
							</Button>
						{/if}
					</div>
				</div>

				<div>
					<p class="mb-2 text-xs font-medium uppercase tracking-wide text-muted-foreground">Quick groups</p>
					<div class="flex gap-2 flex-wrap">
						<button
							onclick={() => handleCategoryChange('')}
							class="px-3 py-1.5 rounded-md text-sm transition-colors
								{selectedCategory === ''
								? 'bg-primary text-primary-foreground'
								: 'bg-muted hover:bg-muted/80'}"
							data-testid="category-all"
						>
							All
						</button>
						{#each filteredCategories as category}
							<button
								onclick={() => handleCategoryChange(category.id)}
								class="px-3 py-1.5 rounded-md text-sm transition-colors
									{selectedCategory === category.id
									? 'bg-primary text-primary-foreground'
									: 'bg-muted hover:bg-muted/80'}"
								data-testid="category-{category.id}"
							>
								{category.name}
							</button>
						{/each}
					</div>
				</div>

				<div>
					<p class="mb-2 text-xs font-medium uppercase tracking-wide text-muted-foreground">All categories</p>
					<div class="flex gap-2 flex-wrap max-h-24 overflow-y-auto pr-1">
					<button
						onclick={() => handleCategoryChange('')}
						class="px-3 py-1.5 rounded-md text-sm transition-colors
							{selectedCategory === ''
							? 'bg-primary text-primary-foreground'
							: 'bg-muted hover:bg-muted/80'}"
						data-testid="category-all-categories"
					>
						All
					</button>
					{#each categories as category}
						<button
							onclick={() => handleCategoryChange(category.id)}
							class="px-3 py-1.5 rounded-md text-sm transition-colors
								{selectedCategory === category.id
								? 'bg-primary text-primary-foreground'
								: 'bg-muted hover:bg-muted/80'}"
							data-testid="category-{category.id}"
						>
							{category.name}
						</button>
					{/each}
				</div>
				</div>

				{#if activeCategory}
					<p class="text-xs text-muted-foreground" data-testid="active-category-description">
						<strong class="text-foreground">{activeCategory.name}:</strong>
						{activeCategory.description || 'Category selected.'}
					</p>
				{/if}
			</div>

			<!-- Keycode List -->
			<div class="flex-1 overflow-y-auto p-4">
				{#if loading}
					<div class="flex items-center justify-center h-32 text-muted-foreground">
						Loading keycodes...
					</div>
				{:else if error}
					<div class="flex items-center justify-center h-32 text-destructive">
						{error}
					</div>
				{:else if keycodes.length === 0}
					<div class="flex items-center justify-center h-32 text-muted-foreground">
						No keycodes found
					</div>
				{:else}
					<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2">
						{#each keycodes as keycode}
							{@const isCurrent = keycode.code === currentKeycode}
							<button
								onclick={() => handleSelectKeycode(keycode.code)}
								class="p-3 text-left rounded-lg border border-border hover:bg-accent transition-colors
									{isCurrent ? 'ring-2 ring-primary' : ''}"
								data-testid="keycode-option-{keycode.code}"
							>
								<div class="flex items-start justify-between gap-3">
									<div>
										<div class="font-mono text-sm font-medium">{keycode.code}</div>
										<div class="text-xs text-muted-foreground mt-1">
											{keycode.name}
											{#if isCurrent}
												<span class="ml-2 text-primary">(current)</span>
											{/if}
										</div>
									</div>
									<span class="shrink-0 rounded bg-muted px-2 py-0.5 text-[11px] uppercase tracking-wide text-muted-foreground">
										{keycode.category}
									</span>
								</div>
								<div class="text-xs text-muted-foreground mt-2">
									Press to assign this keycode
								</div>
								{#if keycode.description}
									<div class="text-xs text-muted-foreground mt-1 line-clamp-2">
										{keycode.description}
									</div>
								{/if}
							</button>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Footer -->
			<div class="flex items-center justify-between p-4 border-t border-border">
				<Button variant="outline" onclick={handleClear} data-testid="clear-key-button">
					Use lower layer key <span class="kbd-token ml-2">KC_TRNS</span>
				</Button>
				<Button variant="ghost" onclick={handleClose}>Cancel</Button>
			</div>
		</div>
	</div>
{/if}

<style>
	.line-clamp-2 {
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
</style>
