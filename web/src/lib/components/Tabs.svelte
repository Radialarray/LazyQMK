<script lang="ts">
	interface TabItem {
		id: string;
		label: string;
		icon?: string;
	}

	interface Props {
		tabs: TabItem[];
		activeTab: string;
		onTabChange: (tabId: string) => void;
		class?: string;
	}

	let { tabs, activeTab, onTabChange, class: className = '' }: Props = $props();
</script>

<div class="tabs-container {className}">
	<div class="flex border-b border-border">
		{#each tabs as tab}
			<button
				onclick={() => onTabChange(tab.id)}
				class="tab-button px-4 py-2 text-sm font-medium transition-colors
					{activeTab === tab.id
					? 'border-b-2 border-primary text-primary'
					: 'text-muted-foreground hover:text-foreground hover:bg-accent'}"
				aria-selected={activeTab === tab.id}
				role="tab"
			>
				{#if tab.icon}
					<span class="mr-2 text-xs font-semibold uppercase tracking-wide text-current/80">{tab.icon}</span>
				{/if}
				{tab.label}
			</button>
		{/each}
	</div>
</div>

<style>
	.tabs-container {
		width: 100%;
	}

	.tab-button {
		position: relative;
		white-space: nowrap;
	}

	.tab-button:focus {
		outline: 2px solid hsl(var(--ring));
		outline-offset: -2px;
	}
</style>
