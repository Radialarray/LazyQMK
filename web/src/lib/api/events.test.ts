/**
 * Unit tests for the SSE EventSource wrapper.
 *
 * We mock `EventSource` since the real implementation is browser-only.
 * The wrapper's logic is pure: open → handle message → close.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock EventSource before importing the module under test.
type Listener = (ev: unknown) => void;

class MockEventSource {
	static instances: MockEventSource[] = [];
	url: string;
	listeners: Map<string, Listener[]> = new Map();
	closed = false;
	withCredentials = false;
	readyState: 0 | 1 | 2 = 1;

	constructor(url: string) {
		this.url = url;
		MockEventSource.instances.push(this);
	}

	addEventListener(type: string, listener: Listener) {
		const arr = this.listeners.get(type) ?? [];
		arr.push(listener);
		this.listeners.set(type, arr);
	}

	removeEventListener() {}
	dispatchEvent(): boolean {
		return true;
	}
	close() {
		this.closed = true;
	}

	emit(type: string, ev: unknown) {
		const arr = this.listeners.get(type) ?? [];
		for (const fn of arr) fn(ev);
	}
}

beforeEach(() => {
	MockEventSource.instances = [];
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	(globalThis as any).EventSource = MockEventSource;
});

afterEach(() => {
	vi.restoreAllMocks();
});

describe('subscribeLayoutEvents', () => {
	it('opens an EventSource at /api/events', async () => {
		const { subscribeLayoutEvents } = await import('./events');
		const unsub = subscribeLayoutEvents(() => {});
		expect(MockEventSource.instances).toHaveLength(1);
		expect(MockEventSource.instances[0].url).toBe('/api/events');
		unsub();
	});

	it('parses a layout message into a typed event', async () => {
		const { subscribeLayoutEvents } = await import('./events');
		const onEvent = vi.fn();
		const unsub = subscribeLayoutEvents(onEvent);
		const es = MockEventSource.instances[0];

		es.emit('layout', { data: JSON.stringify({ kind: 'changed', filename: 'foo.json', mtime: 42 }) });

		expect(onEvent).toHaveBeenCalledTimes(1);
		expect(onEvent).toHaveBeenCalledWith({
			kind: 'changed',
			filename: 'foo.json',
			mtime: 42
		});
		unsub();
	});

	it('parses a removed event', async () => {
		const { subscribeLayoutEvents } = await import('./events');
		const onEvent = vi.fn();
		const unsub = subscribeLayoutEvents(onEvent);
		const es = MockEventSource.instances[0];

		es.emit('layout', { data: JSON.stringify({ kind: 'removed', filename: 'gone.json' }) });

		expect(onEvent).toHaveBeenCalledWith({ kind: 'removed', filename: 'gone.json' });
		unsub();
	});

	it('drops malformed payloads without throwing', async () => {
		const { subscribeLayoutEvents } = await import('./events');
		const onEvent = vi.fn();
		const unsub = subscribeLayoutEvents(onEvent);
		const es = MockEventSource.instances[0];

		es.emit('layout', { data: 'not json' });
		es.emit('layout', { data: JSON.stringify({ kind: 'unknown' }) });
		es.emit('layout', { data: JSON.stringify({ kind: 'changed', filename: 123 }) });

		expect(onEvent).not.toHaveBeenCalled();
		unsub();
	});

	it('returns a cleanup function that closes the source', async () => {
		const { subscribeLayoutEvents } = await import('./events');
		const unsub = subscribeLayoutEvents(() => {});
		const es = MockEventSource.instances[0];
		expect(es.closed).toBe(false);
		unsub();
		expect(es.closed).toBe(true);
	});

	it('forwards error events to the optional onError callback', async () => {
		const { subscribeLayoutEvents } = await import('./events');
		const onError = vi.fn();
		const unsub = subscribeLayoutEvents(() => {}, onError);
		const es = MockEventSource.instances[0];
		const errEv = new Event('error');
		es.emit('error', errEv);
		expect(onError).toHaveBeenCalledWith(errEv);
		unsub();
	});
});
