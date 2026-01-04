import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import type { Layout, ApiError } from '$api/types';

// Disable SSR for this page to allow proper API mocking in tests
// and client-side rendering with live geometry updates
export const ssr = false;

export const load: PageLoad = async ({ params, fetch }) => {
	try {
		// Use SvelteKit's fetch which works in both SSR and client-side
		const response = await fetch(`/api/layouts/${encodeURIComponent(params.name)}`);

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
		return {
			layout
		};
	} catch (e) {
		throw error(404, {
			message: e instanceof Error ? e.message : 'Failed to load layout'
		});
	}
};
