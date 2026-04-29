<script lang="ts">
	import { page } from '$app/stores';

	const primaryPages = [
		{ href: '/layouts', label: 'Layouts', match: (pathname: string) => pathname.startsWith('/layouts') },
		{ href: '/onboarding', label: 'New Layout', match: (pathname: string) => pathname === '/onboarding' || pathname === '/setup' },
		{ href: '/templates', label: 'Templates', match: (pathname: string) => pathname === '/templates' },
		{ href: '/keycodes', label: 'Keycodes', match: (pathname: string) => pathname === '/keycodes' },
		{ href: '/settings', label: 'Settings', match: (pathname: string) => pathname === '/settings' }
	];
</script>

<header class="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 sticky top-0 z-50">
	<div class="container mx-auto px-4 h-14 flex items-center justify-between">
		<!-- Logo/Home -->
		<a href="/" class="flex items-center gap-2 font-bold text-lg hover:opacity-80">
			<span class="icon-chip h-8 w-8 rounded-lg text-xs">LQ</span>
			<div class="flex flex-col leading-none">
				<span>LazyQMK</span>
				<span class="text-[10px] font-medium uppercase tracking-[0.22em] text-muted-foreground">layout studio</span>
			</div>
		</a>

		<!-- Main Nav -->
		<nav class="flex flex-wrap items-center justify-end gap-1">
			{#each primaryPages as navItem}
				<a
					href={navItem.href}
					class="px-3 py-2 text-sm rounded-md hover:bg-muted {navItem.match($page.url.pathname) ? 'bg-muted font-medium' : 'text-muted-foreground'}"
				>
					{navItem.label}
				</a>
			{/each}
		</nav>
	</div>
</header>
