import { error } from '@sveltejs/kit';
import { apiClient } from '$api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
	try {
		const layout = await apiClient.getLayout(params.name);
		return {
			layout
		};
	} catch (e) {
		throw error(404, {
			message: e instanceof Error ? e.message : 'Failed to load layout'
		});
	}
};
