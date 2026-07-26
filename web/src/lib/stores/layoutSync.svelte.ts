/**
 * Global hot-reload state for the WebUI editor.
 *
 * This is the first piece of cross-component state in the WebUI
 * (LazyQMK-84o5.8). It exposes a small Svelte 5 runes-flavoured API
 * that the editor page, the list page, and the conflict modal all
 * share so they can agree on:
 *
 *  - which layout is currently loaded (`current`),
 *  - what the last-known on-disk mtime was (`lastSyncedMtime`),
 *  - whether a save is currently in flight (`pendingSaves`),
 *  - whether the user must resolve a conflict
 *    (`pendingReload === 'conflict'`).
 *
 * The store owns the EventSource subscription and a refetch method,
 * so callers never see the wire-format event directly.
 */

import { apiClient } from '$lib/api';
import type { Layout } from '$lib/api/types';
import { subscribeLayoutEvents, type LayoutEvent } from '$lib/api/events';

export type ReloadState =
	/** No external event has been received since the last sync. */
	| 'clean'
	/** An external change arrived but no local edits are pending — safe to auto-reload. */
	| 'safe'
	/** An external change arrived while local edits are in flight — user must choose. */
	| 'conflict';

class LayoutSyncStore {
	/** Current layout (the server's view, or local optimistic copy if dirty). */
	current = $state<Layout | null>(null);
	/** Filename this store is currently bound to. */
	filename = $state<string | null>(null);
	/** mtime of the last successful GET, used to detect "stale on disk". */
	lastSyncedMtime = $state<number>(0);
	/** Set of filenames with a save currently in flight. */
	pendingSaves = $state<Set<string>>(new Set());
	/** Current conflict-resolution state. Drives the modal. */
	pendingReload = $state<ReloadState>('clean');
	/** Filename of the layout the conflict applies to (always equals `filename` today). */
	conflictFilename = $state<string | null>(null);
	/** Last error from a refetch, surfaced to the editor for the user. */
	error = $state<string | null>(null);

	#unsubscribe: (() => void) | null = null;

	/**
	 * Whether the editor has unsaved changes (a save in flight counts
	 * as "dirty" for conflict-resolution purposes).
	 */
	get dirty(): boolean {
		return this.filename !== null && this.pendingSaves.has(this.filename);
	}

	/**
	 * Binds the store to a specific layout. Loads it from the
	 * server, starts watching the SSE stream, and returns the
	 * initial layout. Re-calling with the same filename is a no-op
	 * (idempotent).
	 */
	async subscribeToFile(filename: string): Promise<Layout> {
		if (this.filename === filename && this.current) {
			return this.current;
		}

		this.unsubscribe();
		this.filename = filename;
		this.conflictFilename = null;
		this.pendingReload = 'clean';
		this.error = null;

		const layout = await this.#fetch(filename);
		this.current = layout;
		this.lastSyncedMtime = Date.now() / 1000;

		this.#unsubscribe = subscribeLayoutEvents(
			(ev) => this.#onEvent(ev),
			() => {
				// EventSource auto-reconnects; nothing to do on error.
			}
		);

		return layout;
	}

	/**
	 * Stops watching. Call on `onDestroy`/effect cleanup.
	 */
	unsubscribe(): void {
		if (this.#unsubscribe) {
			this.#unsubscribe();
			this.#unsubscribe = null;
		}
		this.filename = null;
		this.current = null;
		this.conflictFilename = null;
		this.pendingReload = 'clean';
	}

	/**
	 * Marks a save as in flight. Call this right before firing
	 * `apiClient.saveLayout` so the conflict detector can pause
	 * auto-reload until the round-trip completes.
	 */
	markSaving(filename: string): void {
		const next = new Set(this.pendingSaves);
		next.add(filename);
		this.pendingSaves = next;
	}

	/**
	 * Marks a save as finished. If a conflict was deferred while the
	 * save was in flight, it will now be surfaced to the user.
	 */
	markSaved(filename: string): void {
		const next = new Set(this.pendingSaves);
		next.delete(filename);
		this.pendingSaves = next;
		if (
			this.filename === filename &&
			this.pendingReload === 'conflict' &&
			this.conflictFilename === filename
		) {
			// Re-arm the conflict: the user still needs to decide
			// what to do, since the on-disk content is newer than
			// what they were about to save.
			this.conflictFilename = filename;
		}
	}

	/**
	 * Re-fetches the layout from the server. Used by the editor to
	 * pick up a change after the user chooses "Reload", and by
	 * the conflict modal as the "auto-reload" path.
	 */
	async requestReload(): Promise<void> {
		if (!this.filename) return;
		try {
			const layout = await this.#fetch(this.filename);
			this.current = layout;
			this.lastSyncedMtime = Date.now() / 1000;
			this.pendingReload = 'clean';
			this.conflictFilename = null;
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
		}
	}

	/**
	 * Persists the current `current` layout to disk, overwriting
	 * whatever the external process wrote. Used by the "Keep mine"
	 * option of the conflict modal.
	 */
	async overwrite(): Promise<void> {
		if (!this.filename || !this.current) return;
		this.markSaving(this.filename);
		try {
			await apiClient.saveLayout(this.filename, this.current);
			// PUT returns void; assume the in-memory copy is what the
			// server now has.
			this.lastSyncedMtime = Date.now() / 1000;
			this.pendingReload = 'clean';
			this.conflictFilename = null;
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
		} finally {
			this.markSaved(this.filename);
		}
	}

	/**
	 * Backs up the current `current` to `<name>.json.local` then
	 * reloads from disk. Matches the TUI "Save then reload" branch.
	 */
	async saveThenReload(): Promise<void> {
		if (!this.filename || !this.current) return;
		const sidecar = `${this.filename}.local`;
		this.markSaving(sidecar);
		try {
			await apiClient.saveLayout(sidecar, this.current);
		} catch (e) {
			this.error = `Failed to back up to ${sidecar}: ${e instanceof Error ? e.message : String(e)}`;
			this.markSaved(sidecar);
			return;
		}
		this.markSaved(sidecar);
		await this.requestReload();
	}

	/**
	 * Dismisses the conflict without taking action. The local copy
	 * stays in memory; future external changes will re-trigger the
	 * prompt.
	 */
	dismissConflict(): void {
		this.pendingReload = 'clean';
		this.conflictFilename = null;
	}

	async #fetch(filename: string): Promise<Layout> {
		return await apiClient.getLayout(filename);
	}

	#onEvent(event: LayoutEvent): void {
		// Ignore events for other files.
		if (!this.filename || event.filename !== this.filename) return;

		// We don't compare mtimes here: the server already filters
		// out self-writes, so any event we receive is external by
		// construction. The mtime is just metadata for the UI.
		if (event.kind === 'removed') {
			if (this.dirty) {
				this.pendingReload = 'conflict';
				this.conflictFilename = event.filename;
			} else {
				// File is gone. The user can still keep editing in
				// memory; we just clear the current to force the
				// editor to render the "file missing" state.
				this.pendingReload = 'conflict';
				this.conflictFilename = event.filename;
			}
			return;
		}

		if (this.dirty) {
			this.pendingReload = 'conflict';
			this.conflictFilename = event.filename;
		} else {
			this.pendingReload = 'safe';
			// Fire-and-forget; the editor will see `current` change
			// once the fetch completes.
			void this.requestReload();
		}
	}
}

export const layoutSync = new LayoutSyncStore();
