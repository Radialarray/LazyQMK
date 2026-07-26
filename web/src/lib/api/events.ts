/**
 * Typed wrapper around the `/api/events` Server-Sent Events stream.
 *
 * The backend broadcasts a `LayoutEvent` message every time a `.json`
 * file in the workspace changes on disk — for example because a
 * background agent ran `lazyqmk tap-dance add --layout foo.json` or
 * another TUI session saved the same file.
 *
 * `subscribeLayoutEvents` opens an `EventSource` and invokes
 * `onEvent` for every parsed message. It returns a cleanup function
 * that closes the connection; callers should call it on
 * `onDestroy`/`$effect` cleanup to avoid leaks.
 *
 * The browser's native `EventSource` automatically reconnects on
 * network errors, so we don't have to implement retry logic.
 */

/**
 * Wire-format `LayoutEvent` as serialised by the Rust backend.
 *
 * The `kind` tag is `"changed"` or `"removed"`. The `filename` is
 * the bare file name (e.g. `"my_layout.json"`), never an absolute
 * path.
 */
export type LayoutEvent =
	| { kind: 'changed'; filename: string; mtime: number }
	| { kind: 'removed'; filename: string };

/**
 * Subscribes to layout change events from the backend.
 *
 * @param onEvent Invoked once for every event received.
 * @param onError Optional callback for EventSource errors (e.g.
 *                network failure). The browser will auto-reconnect.
 * @returns A function that, when called, closes the EventSource.
 */
export function subscribeLayoutEvents(
	onEvent: (event: LayoutEvent) => void,
	onError?: (error: Event) => void
): () => void {
	const es = new EventSource('/api/events');

	es.addEventListener('layout', (raw: MessageEvent) => {
		try {
			const data = JSON.parse(raw.data) as LayoutEvent;
			// Defensive runtime check: backend always sends one of
			// the two known shapes, but the wire format could change.
			if (
				(data && typeof data === 'object' && 'kind' in data) &&
				((data.kind === 'changed' &&
					typeof data.filename === 'string' &&
					typeof data.mtime === 'number') ||
					(data.kind === 'removed' && typeof data.filename === 'string'))
			) {
				onEvent(data);
			}
		} catch {
			// Malformed payload — ignore. The next event will arrive
			// soon enough.
		}
	});

	if (onError) {
		es.addEventListener('error', onError);
	}

	return () => {
		es.close();
	};
}
