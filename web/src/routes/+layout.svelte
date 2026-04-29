<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from 'mode-watcher';
	import { page } from '$app/stores';
	import { NavHeader } from '$components';

	let { children } = $props();

	// Pages that should NOT show the nav header (full-screen experiences)
	const noHeaderPages = ['/onboarding', '/setup'];
	let showHeader = $derived(!noHeaderPages.includes($page.url.pathname));
</script>

<ModeWatcher />
{#if showHeader}
	<NavHeader />
{/if}
<main class={showHeader ? '' : ''}>
	{@render children()}
</main>
