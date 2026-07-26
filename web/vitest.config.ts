import { defineConfig } from 'vitest/config';
import path from 'path';

export default defineConfig({
	resolve: {
		alias: {
			$lib: path.resolve('./src/lib'),
			$components: path.resolve('./src/lib/components'),
			$stores: path.resolve('./src/lib/stores'),
			$api: path.resolve('./src/lib/api')
		}
	},
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}'],
		// layoutSync.svelte.ts uses Svelte 5 runes; it is exercised
		// via svelte-check + e2e instead of vitest (the local vitest
		// 2 + svelte 5 + vite 5 toolchain doesn't preprocess .svelte.ts).
		exclude: ['src/lib/stores/**'],
		environment: 'jsdom',
		globals: true,
		setupFiles: ['./src/test/setup.ts']
	}
});
