<script lang="ts">
	import { onMount } from 'svelte';
	import { apiClient, type KeycodeInfo, type CategoryInfo } from '$api';
	import { Button, Card, Input } from '$components';

	let keycodes = $state<KeycodeInfo[]>([]);
	let categories = $state<CategoryInfo[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let searchTerm = $state('');
	let selectedCategory = $state<string | null>(null);

	onMount(async () => {
		try {
			const [keycodesRes, categoriesRes] = await Promise.all([
				apiClient.listKeycodes(),
				apiClient.listCategories()
			]);
			keycodes = keycodesRes.keycodes;
			categories = categoriesRes.categories;
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load keycodes';
		} finally {
			loading = false;
		}
	});

	async function handleSearch() {
		loading = true;
		try {
			const response = await apiClient.listKeycodes(searchTerm, selectedCategory || undefined);
			keycodes = response.keycodes;
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to search keycodes';
		} finally {
			loading = false;
		}
	}

	function selectCategory(categoryId: string | null) {
		selectedCategory = categoryId;
		handleSearch();
	}
</script>

<div class="container mx-auto p-6">
	<div class="mb-8 flex items-center justify-between">
		<div>
			<h1 class="text-4xl font-bold mb-2">Keycodes Browser</h1>
			<p class="text-muted-foreground">
				Browse and search QMK keycodes
			</p>
		</div>
		<Button onclick={() => (window.location.href = '/')}>
			Back to Dashboard
		</Button>
	</div>

	<div class="grid gap-6 lg:grid-cols-4">
		<!-- Category Sidebar -->
		<div class="lg:col-span-1">
			<Card class="p-4">
				<h2 class="text-lg font-semibold mb-4">Categories</h2>
				<div class="space-y-2">
					<button
						class="w-full text-left px-3 py-2 rounded-md hover:bg-accent transition-colors {selectedCategory ===
						null
							? 'bg-accent'
							: ''}"
						onclick={() => selectCategory(null)}
					>
						All Categories
					</button>
					{#each categories as category}
						<button
							class="w-full text-left px-3 py-2 rounded-md hover:bg-accent transition-colors {selectedCategory ===
							category.id
								? 'bg-accent'
								: ''}"
							onclick={() => selectCategory(category.id)}
						>
							{category.name}
						</button>
					{/each}
				</div>
			</Card>
		</div>

		<!-- Keycode List -->
		<div class="lg:col-span-3">
			<!-- Search Bar -->
			<Card class="p-4 mb-6">
				<div class="flex gap-2">
					<Input
						bind:value={searchTerm}
						placeholder="Search keycodes..."
						class="flex-1"
						oninput={() => handleSearch()}
					/>
				</div>
			</Card>

			{#if loading}
				<p class="text-muted-foreground">Loading keycodes...</p>
			{:else if error}
				<Card class="p-6">
					<div class="text-destructive">
						<p class="font-medium">Error loading keycodes</p>
						<p class="text-sm">{error}</p>
					</div>
				</Card>
			{:else if keycodes.length === 0}
				<Card class="p-6">
					<p class="text-muted-foreground">
						No keycodes found matching your search.
					</p>
				</Card>
			{:else}
				<div class="space-y-2">
					{#each keycodes as keycode}
						<Card class="p-4 hover:border-primary transition-colors">
							<div class="flex items-start justify-between">
								<div class="flex-1">
									<div class="flex items-center gap-3 mb-1">
										<code class="text-sm font-mono bg-muted px-2 py-1 rounded">
											{keycode.code}
										</code>
										<span class="text-sm font-medium">{keycode.name}</span>
									</div>
									{#if keycode.description}
										<p class="text-sm text-muted-foreground">
											{keycode.description}
										</p>
									{/if}
								</div>
								<span class="text-xs text-muted-foreground">
									{keycode.category}
								</span>
							</div>
						</Card>
					{/each}
				</div>
			{/if}
		</div>
	</div>
</div>
