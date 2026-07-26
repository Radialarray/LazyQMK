<script lang="ts">
	import { AccessibleDialog, Button } from '$components';
	import { layoutSync } from '$stores/layoutSync.svelte';
	import type { Layout } from '$lib/api/types';

	/**
	 * Modal that appears when the file watcher detects an external
	 * change to the open layout while the user has unsaved local
	 * edits. Mirrors the TUI's `ExternalChangePrompt`.
	 *
	 * The modal wires directly into the global `layoutSync` store; it
	 * does not need any props. When `layoutSync.pendingReload ===
	 * 'conflict'`, the modal opens and offers the three resolution
	 * paths:
	 *
	 *   1. **Reload from disk** — discard local edits, fetch fresh
	 *      from the server.
	 *   2. **Keep mine, overwrite disk** — push local edits back.
	 *   3. **Save mine first, then reload** — back up to
	 *      `<name>.json.local` then reload.
	 *
	 * `Cancel` simply dismisses the modal without touching the
	 * server; a subsequent external change will re-trigger it.
	 */

	let busy = $state(false);
	let lastError = $state<string | null>(null);

	const open = $derived(layoutSync.pendingReload === 'conflict');
	const filename = $derived(layoutSync.conflictFilename ?? layoutSync.filename ?? '');
	const layout = $derived(layoutSync.current as Layout | null);

	async function handleReload() {
		if (busy) return;
		busy = true;
		lastError = null;
		try {
			await layoutSync.requestReload();
		} catch (e) {
			lastError = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}

	async function handleOverwrite() {
		if (busy) return;
		busy = true;
		lastError = null;
		try {
			await layoutSync.overwrite();
		} catch (e) {
			lastError = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}

	async function handleSaveThenReload() {
		if (busy) return;
		busy = true;
		lastError = null;
		try {
			await layoutSync.saveThenReload();
		} catch (e) {
			lastError = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}

	function handleCancel() {
		layoutSync.dismissConflict();
	}

	function handleKeydown(event: KeyboardEvent) {
		if (!open || busy) return;
		if (event.key === 'r' || event.key === 'R') {
			event.preventDefault();
			void handleReload();
		} else if (event.key === 'k' || event.key === 'K') {
			event.preventDefault();
			void handleOverwrite();
		} else if (event.key === 's' || event.key === 'S') {
			event.preventDefault();
			void handleSaveThenReload();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<AccessibleDialog
	{open}
	title="External change detected"
	description={filename
		? `The file ${filename} was modified by another process. Choose how to resolve the conflict.`
		: 'A layout was modified by another process.'}
	onClose={handleCancel}
	titleId="hot-reload-conflict-title"
	panelClass="max-w-lg"
	showCloseButton={!busy}
>
	{#if layout}
		<p class="text-sm text-muted-foreground">
			Local layout: <span class="font-mono">{layout.metadata?.name ?? '<unnamed>'}</span>
		</p>
	{/if}
	{#if lastError}
		<p class="mt-2 text-sm text-destructive">{lastError}</p>
	{/if}

	<svelte:fragment slot="footer">
		<Button variant="outline" disabled={busy} onclick={handleCancel}>Cancel</Button>
		<Button variant="secondary" disabled={busy} onclick={handleSaveThenReload}>
			Save then reload
		</Button>
		<Button variant="secondary" disabled={busy} onclick={handleOverwrite}>
			Keep mine
		</Button>
		<Button variant="default" disabled={busy} onclick={handleReload}>Reload from disk</Button>
	</svelte:fragment>
</AccessibleDialog>
