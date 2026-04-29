<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	onMount(() => {
		goto('/onboarding', { replaceState: true });
	});
</script>

<svelte:head>
	<meta http-equiv="refresh" content="0;url=/onboarding" />
	<title>Redirecting to Setup - LazyQMK</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center p-6">
	<p class="text-sm text-muted-foreground">Redirecting to unified setup flow...</p>
</div>
