import { describe, expect, it } from 'vitest';
import { getNavigationTarget, shouldBlockNavigation } from './navigationGuard';

describe('navigationGuard', () => {
	it('builds target with path search and hash', () => {
		const url = new URL('https://example.com/layouts/test?tab=build#logs');

		expect(getNavigationTarget(url)).toBe('/layouts/test?tab=build#logs');
	});

	it('returns null when no target url', () => {
		expect(getNavigationTarget(null)).toBeNull();
	});

	it('blocks dirty navigation to different route state', () => {
		const currentUrl = new URL('https://example.com/layouts/test?tab=preview');
		const nextUrl = new URL('https://example.com/layouts/test?tab=build');

		expect(shouldBlockNavigation(true, false, currentUrl, nextUrl)).toBe(true);
	});

	it('allows navigation when not dirty', () => {
		const currentUrl = new URL('https://example.com/layouts/test');
		const nextUrl = new URL('https://example.com/layouts');

		expect(shouldBlockNavigation(false, false, currentUrl, nextUrl)).toBe(false);
	});

	it('allows navigation when bypass active', () => {
		const currentUrl = new URL('https://example.com/layouts/test');
		const nextUrl = new URL('https://example.com/layouts');

		expect(shouldBlockNavigation(true, true, currentUrl, nextUrl)).toBe(false);
	});
});
