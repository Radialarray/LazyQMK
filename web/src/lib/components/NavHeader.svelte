<script lang="ts">
	import { page } from '$app/stores';
	import { Button } from '$components';

	let showSettingsMenu = $state(false);

	// Secondary pages that go under More menu
	const morePages = [
		{ href: '/templates', label: 'Templates', description: 'Layout templates' },
		{ href: '/keycodes', label: 'Keycodes', description: 'Browse keycodes' },
		{ href: '/settings', label: 'Settings', description: 'Configure QMK path' }
	];

	// Check if current page is a More menu page
	let isMorePage = $derived(
		morePages.some((p) => $page.url.pathname === p.href)
	);

	function toggleMoreMenu() {
		showSettingsMenu = !showSettingsMenu;
	}

	function closeMoreMenu() {
		showSettingsMenu = false;
	}
</script>

<header class="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 sticky top-0 z-50">
	<div class="container mx-auto px-4 h-14 flex items-center justify-between">
		<!-- Logo/Home -->
		<a href="/" class="flex items-center gap-2 font-bold text-lg hover:opacity-80">
			<span class="text-2xl">⌨️</span>
			<span>LazyQMK</span>
		</a>

		<!-- Main Nav -->
		<nav class="flex items-center gap-1">
			<a href="/layouts" class="px-3 py-2 text-sm rounded-md hover:bg-muted {$page.url.pathname.startsWith('/layouts') ? 'bg-muted font-medium' : 'text-muted-foreground'}">
				Layouts
			</a>
			<a href="/onboarding" class="px-3 py-2 text-sm rounded-md hover:bg-muted {$page.url.pathname === '/onboarding' ? 'bg-muted font-medium' : 'text-muted-foreground'}">
				New
			</a>

			<!-- More Dropdown -->
			<div class="relative">
				<button
					class="px-3 py-2 text-sm rounded-md hover:bg-muted flex items-center gap-1 {isMorePage ? 'bg-muted font-medium' : 'text-muted-foreground'}"
					onclick={toggleMoreMenu}
					aria-expanded={showSettingsMenu}
					aria-haspopup="true"
				>
					More
					<svg class="w-4 h-4 transition-transform {showSettingsMenu ? 'rotate-180' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
					</svg>
				</button>

				{#if showSettingsMenu}
					<!-- Backdrop -->
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="fixed inset-0 z-40" onclick={closeMoreMenu}></div>

					<!-- Dropdown Menu -->
					<div class="absolute right-0 top-full mt-1 w-48 bg-popover border rounded-md shadow-lg z-50 py-1">
						{#each morePages as page}
							<a
								href={page.href}
								class="block px-4 py-2 text-sm hover:bg-muted"
								onclick={closeMoreMenu}
							>
								<div class="font-medium">{page.label}</div>
								<div class="text-xs text-muted-foreground">{page.description}</div>
							</a>
						{/each}
					</div>
				{/if}
			</div>
		</nav>
	</div>
</header>
