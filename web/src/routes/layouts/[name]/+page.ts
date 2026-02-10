import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import type { Layout, ApiError } from '$api/types';
import { addRecentLayout } from '$lib/utils/recentLayouts';

// Disable SSR for this page to allow proper API mocking in tests
// and client-side rendering with live geometry updates
export const ssr = false;

export const load: PageLoad = async ({ params, fetch }) => {
	try {
		const runtimeBaseUrl = (globalThis as { __LAZYQMK_API_BASE_URL?: string })
			.__LAZYQMK_API_BASE_URL;
		const endpoint = `/api/layouts/${encodeURIComponent(params.name)}`;
		const requestUrl = runtimeBaseUrl ? `${runtimeBaseUrl}${endpoint}` : endpoint;
		// Use SvelteKit's fetch which works in both SSR and client-side
		const response = await fetch(requestUrl);

		if (!response.ok) {
			let errorData: ApiError;
			try {
				errorData = await response.json();
			} catch {
				errorData = {
					error: `HTTP ${response.status}: ${response.statusText}`
				};
			}
			throw new Error(errorData.details || errorData.error);
		}

		const layout: Layout = await response.json();
		
		// Track this layout as recently opened
		addRecentLayout(params.name, layout.metadata.name);
		
		return {
			layout,
			filename: params.name
		};
	} catch (e) {
		throw error(404, {
			message: e instanceof Error ? e.message : 'Failed to load layout'
		});
	}
};
