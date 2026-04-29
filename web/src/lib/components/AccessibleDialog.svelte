<script lang="ts">
	import { Button, Card } from '$components';

	interface $$Slots {
		default: Record<string, never>;
		footer: Record<string, never>;
	}

	interface Props {
		open: boolean;
		title: string;
		description?: string;
		onClose: () => void;
		titleId?: string;
		panelClass?: string;
		closeLabel?: string;
		showCloseButton?: boolean;
	}

	let {
		open,
		title,
		description = '',
		onClose,
		titleId = 'dialog-title',
		panelClass = 'max-w-md',
		closeLabel = 'Close dialog',
		showCloseButton = true
	}: Props = $props();

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			onClose();
		}
	}
</script>

{#if open}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
		onclick={onClose}
		onkeydown={handleKeydown}
		role="dialog"
		aria-modal="true"
		aria-labelledby={titleId}
		tabindex="-1"
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class={`w-full ${panelClass}`} onclick={(event: MouseEvent) => event.stopPropagation()}>
			<Card class="p-6">
				<div class="flex items-start justify-between gap-4">
					<div>
						<h2 id={titleId} class="text-lg font-semibold">{title}</h2>
						{#if description}
							<p class="mt-2 text-sm text-muted-foreground">{description}</p>
						{/if}
					</div>
					{#if showCloseButton}
						<Button variant="ghost" size="icon" onclick={onClose} title={closeLabel}>
							<span class="text-xl">✕</span>
						</Button>
					{/if}
				</div>

				<div class="mt-4">
					<slot />
				</div>

				<div class="mt-6 flex flex-wrap justify-end gap-2">
					<slot name="footer" />
				</div>
			</Card>
		</div>
	</div>
{/if}
