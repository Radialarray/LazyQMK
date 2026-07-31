<script lang="ts">
	import { apiClient } from '$lib/api/client';
	import type {
		LayoutDiff,
		RevisionSummary,
		DiffResponse
	} from '$lib/api/types';
	import { Button } from '$components';
	import { AccessibleDialog } from '$components';

	interface Props {
		open: boolean;
		layoutName: string;
		onClose: () => void;
		onChange?: () => void;
	}

	let { open, layoutName, onClose, onChange }: Props = $props();

	let revisions = $state<RevisionSummary[]>([]);
	let selected = $state<number>(0);
	let busy = $state(false);
	let error = $state<string | null>(null);
	let confirmAction = $state<'restore' | 'delete' | null>(null);
	let labelInput = $state('');
	let diff = $state<LayoutDiff | null>(null);
	let showDiff = $state(false);

	async function refresh() {
		busy = true;
		error = null;
		try {
			const resp = await apiClient.listVersions(layoutName);
			revisions = resp.revisions;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}

	$effect(() => {
		if (open && layoutName) {
			refresh();
		}
	});

	async function handleCreateSnapshot() {
		busy = true;
		error = null;
		try {
			await apiClient.createSnapshot(layoutName, {
				label: labelInput.trim() || undefined
			});
			labelInput = '';
			await refresh();
			onChange?.();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}

	async function handleRestore(revision: number) {
		confirmAction = 'restore';
		pendingRestore = revision;
	}

	let pendingRestore = $state<number | null>(null);
	let pendingDelete = $state<number | null>(null);

	async function confirmYes() {
		if (!confirmAction) return;
		busy = true;
		error = null;
		try {
			if (confirmAction === 'restore' && pendingRestore !== null) {
				await apiClient.restoreRevision(layoutName, pendingRestore);
			} else if (confirmAction === 'delete' && pendingDelete !== null) {
				await apiClient.deleteRevision(layoutName, pendingDelete);
			}
			confirmAction = null;
			pendingRestore = null;
			pendingDelete = null;
			await refresh();
			onChange?.();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}

	function confirmNo() {
		confirmAction = null;
		pendingRestore = null;
		pendingDelete = null;
	}

	async function handleDelete(revision: number) {
		confirmAction = 'delete';
		pendingDelete = revision;
	}

	async function handleDiff() {
		if (selected >= revisions.length) return;
		const current = revisions[selected];
		busy = true;
		error = null;
		try {
			// Diff against current revision (revision 1 in our model is initial).
			const resp: DiffResponse = await apiClient.diffRevisions(
				layoutName,
				1,
				current.revision
			);
			diff = resp.diff;
			showDiff = true;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}
</script>

<AccessibleDialog title="Versions · {layoutName}" open={open} onClose={onClose}>
	<div class="flex flex-col gap-3 p-4 min-w-[600px]">
		{#if error}
			<div class="text-error text-sm">{error}</div>
		{/if}

		<div class="flex gap-2 items-center">
			<input
				type="text"
				bind:value={labelInput}
				placeholder="Snapshot label (optional)"
				class="input input-bordered input-sm flex-1"
				disabled={busy}
			/>
			<Button onclick={handleCreateSnapshot} disabled={busy || !layoutName}>
				Save snapshot
			</Button>
		</div>

		<div class="border border-surface rounded-md">
			<div class="bg-surface px-3 py-2 text-sm font-medium">Revisions</div>
			<div class="max-h-64 overflow-y-auto">
				{#if revisions.length === 0}
					<div class="p-3 text-sm text-text-secondary">
						No revisions yet. Save one to start tracking changes.
					</div>
				{:else}
					{#each revisions as r, i}
						<button
							class="w-full text-left px-3 py-2 hover:bg-highlight-bg flex justify-between items-center {i ===
							selected
								? 'bg-highlight-bg'
								: ''}"
							onclick={() => (selected = i)}
						>
							<span class="font-mono text-xs">#{r.revision}</span>
							<span class="flex-1 mx-2 truncate">
								{r.label ?? '(no label)'}
							</span>
							<span class="text-xs text-text-secondary">
								{new Date(r.created).toLocaleString()}
							</span>
						</button>
					{/each}
				{/if}
			</div>
		</div>

		{#if revisions.length > 0}
			<div class="flex gap-2 justify-end">
				<Button onclick={handleDiff} disabled={busy}>
					Diff with initial
				</Button>
			<Button
				onclick={() => handleRestore(revisions[selected].revision)}
				disabled={busy}
				variant="default"
			>
				Restore
			</Button>
			<Button
				onclick={() => handleDelete(revisions[selected].revision)}
				disabled={busy}
				variant="destructive"
			>
				Delete
			</Button>
			</div>
		{/if}

		{#if confirmAction}
			<div
				class="border border-warning bg-warning/10 rounded-md p-3 text-sm"
				role="alert"
			>
				{#if confirmAction === 'restore' && pendingRestore !== null}
					Restore revision #{pendingRestore}? Current layout will be
					auto-snapshotted first.
				{:else if confirmAction === 'delete' && pendingDelete !== null}
					Delete revision #{pendingDelete}? This cannot be undone.
				{/if}
				<div class="flex gap-2 mt-2 justify-end">
					<Button onclick={confirmNo} disabled={busy}>Cancel</Button>
					<Button onclick={confirmYes} disabled={busy} variant="destructive">
						Confirm
					</Button>
				</div>
			</div>
		{/if}
	</div>
</AccessibleDialog>

{#if showDiff && diff}
	<AccessibleDialog title="Diff · #{diff.from_revision} → #{diff.to_revision}" open={showDiff} onClose={() => (showDiff = false)}>
		<div class="p-4 min-w-[500px]">
			<div class="text-sm space-y-1 mb-3">
				<div>Layers: +{diff.summary.layers_added} -{diff.summary.layers_removed} ~{diff.summary.keys_changed} keys</div>
				<div>
					Flags: rgb={diff.summary.rgb_changed ? 'yes' : 'no'} combos={diff.summary.combos_changed ? 'yes' : 'no'} tap_dances={diff.summary.tap_dances_changed ? 'yes' : 'no'} meta={diff.summary.metadata_changed ? 'yes' : 'no'}
				</div>
			</div>
			<div class="max-h-96 overflow-y-auto border border-surface rounded-md p-3 space-y-1">
				{#each diff.layer_changes as change}
					{#if change.kind === 'added'}
						<div class="text-success">+ Layer {change.index} {change.layer?.name} (added)</div>
					{:else if change.kind === 'removed'}
						<div class="text-error">- Layer {change.index} {change.name} (removed)</div>
					{:else if change.kind === 'renamed'}
						<div class="text-warning">~ Layer {change.index} renamed: {change.from} → {change.to}</div>
					{:else if change.kind === 'keys_changed'}
						<div class="text-warning">~ Layer {change.index} {change.name} ({change.changes?.length ?? 0} keys)</div>
						{#each change.changes ?? [] as kc}
							<div class="ml-4 font-mono text-xs">({kc.row},{kc.col}): {kc.from} → {kc.to}</div>
						{/each}
					{/if}
				{/each}
				{#if diff.setting_changes.length > 0}
					<div class="font-medium mt-2">Settings:</div>
					{#each diff.setting_changes as s}
						<div class="ml-2 text-xs">{s.path}: {s.from} → {s.to}</div>
					{/each}
				{/if}
			</div>
			<div class="flex justify-end mt-3">
				<Button onclick={() => (showDiff = false)}>Close</Button>
			</div>
		</div>
	</AccessibleDialog>
{/if}
