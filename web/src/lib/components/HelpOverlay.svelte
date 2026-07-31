<script lang="ts">
	import { AccessibleDialog, Input, Button } from '$components';
	import type { HelpContext, HelpBinding } from '$api/types';

	interface Props {
		open: boolean;
		onClose: () => void;
		contexts: HelpContext[];
		appName: string;
	}

	let { open, onClose, contexts, appName }: Props = $props();

	let searchQuery = $state('');
	let activeContextId = $state<string>('');

	const filteredContexts = $derived.by(() => {
		if (!searchQuery.trim()) return contexts;
		const q = searchQuery.toLowerCase();
		return contexts
			.map((ctx) => ({
				...ctx,
				bindings: ctx.bindings.filter((b) =>
					`${b.action} ${b.keys.join(' ')} ${b.alt_keys?.join(' ') ?? ''}`
						.toLowerCase()
						.includes(q)
				)
			}))
			.filter((ctx) => ctx.bindings.length > 0 || ctx.name.toLowerCase().includes(q));
	});

	const activeContext = $derived(
		filteredContexts.find((c) => c.id === activeContextId) ?? filteredContexts[0]
	);

	function selectContext(id: string) {
		activeContextId = id;
	}

	function formatBinding(b: HelpBinding): string {
		const primary = b.keys.join('/');
		if (b.alt_keys && b.alt_keys.length > 0) {
			return `${primary} (${b.alt_keys.join('/')})`;
		}
		return primary;
	}
</script>

<AccessibleDialog
	{open}
	title="{appName} keyboard shortcuts"
	description="Press '?' any time to open this help. Use search to filter by action or key."
	{onClose}
	titleId="help-overlay-title"
	panelClass="max-w-3xl"
>
	<div class="space-y-4">
		<Input
			type="text"
			bind:value={searchQuery}
			placeholder="Filter by action or key (e.g. save, Ctrl+S, layer)"
			class="w-full"
			data-testid="help-search-input"
		/>

		<div class="grid grid-cols-1 md:grid-cols-[180px_1fr] gap-4 max-h-[60vh]">
			<!-- Context list -->
			<nav class="border-r border-border pr-2 overflow-y-auto max-h-[60vh]" data-testid="help-context-list">
				<ul class="space-y-0.5">
					{#each filteredContexts as ctx (ctx.id)}
						{@const isActive = (activeContext?.id ?? '') === ctx.id}
						<li>
							<button
								type="button"
								onclick={() => selectContext(ctx.id)}
								class="w-full text-left px-2 py-1.5 rounded text-sm transition-colors
									{isActive ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}"
								data-testid="help-context-{ctx.id}"
							>
								{ctx.name}
								{#if ctx.bindings.length > 0}
									<span
										class="ml-1 text-[10px] opacity-70"
										class:text-primary-foreground={isActive}
									>
										({ctx.bindings.length})
									</span>
								{/if}
							</button>
						</li>
					{/each}
				</ul>
			</nav>

			<!-- Bindings table -->
			<div class="overflow-y-auto max-h-[60vh]" data-testid="help-binding-list">
				{#if activeContext}
					<h3 class="text-sm font-semibold mb-1">{activeContext.name}</h3>
					{#if activeContext.description}
						<p class="text-xs text-muted-foreground mb-3">{activeContext.description}</p>
					{/if}
					{#if activeContext.bindings.length === 0}
						<p class="text-sm text-muted-foreground">
							No bindings match "{searchQuery}" in this context.
						</p>
					{:else}
						<table class="w-full text-sm">
							<thead>
								<tr class="border-b border-border text-xs uppercase tracking-wide text-muted-foreground">
									<th class="text-left py-1 w-1/3">Key</th>
									<th class="text-left py-1">Action</th>
								</tr>
							</thead>
							<tbody>
								{#each activeContext.bindings as binding (`${binding.keys.join(',')}-${binding.priority}`)}
									<tr class="border-b border-border/40" data-testid="help-binding-row">
										<td class="py-1.5 pr-3 align-top">
											<code
												class="font-mono bg-muted px-1.5 py-0.5 rounded text-xs whitespace-nowrap"
											>
												{formatBinding(binding)}
											</code>
										</td>
										<td class="py-1.5 align-top">
											<span>{binding.action}</span>
											{#if binding.hint}
												<span class="ml-1 text-xs text-muted-foreground"
													>({binding.hint})</span
												>
											{/if}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					{/if}
				{:else}
					<p class="text-sm text-muted-foreground">No contexts match "{searchQuery}".</p>
				{/if}
			</div>
		</div>
	</div>

	<svelte:fragment slot="footer">
		<Button onclick={onClose} data-testid="help-close-button">Close</Button>
	</svelte:fragment>
</AccessibleDialog>