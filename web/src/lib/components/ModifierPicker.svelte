<script lang="ts">
	import { AccessibleDialog, Button } from '$components';

	interface ModifierOption {
		value: string;
		label: string;
		description?: string;
	}

	const DEFAULT_OPTIONS: ModifierOption[] = [
		{ value: 'MOD_LCTL', label: 'Left Control', description: 'LCTL' },
		{ value: 'MOD_LSFT', label: 'Left Shift', description: 'LSFT' },
		{ value: 'MOD_LALT', label: 'Left Alt / Option', description: 'LALT' },
		{ value: 'MOD_LGUI', label: 'Left GUI / Cmd / Win', description: 'LGUI' },
		{ value: 'MOD_RCTL', label: 'Right Control', description: 'RCTL' },
		{ value: 'MOD_RSFT', label: 'Right Shift', description: 'RSFT' },
		{ value: 'MOD_RALT', label: 'Right Alt / AltGr', description: 'RALT' },
		{ value: 'MOD_RGUI', label: 'Right GUI', description: 'RGUI' },
		{ value: 'MOD_LCTL | MOD_LSFT', label: 'Ctrl + Shift (CS)', description: 'LCS' },
		{ value: 'MOD_LCTL | MOD_LALT', label: 'Ctrl + Alt (CA / AltGr)', description: 'CA' },
		{ value: 'MOD_LSFT | MOD_LALT', label: 'Shift + Alt (SA)', description: 'SA' },
		{ value: 'MOD_LCTL | MOD_LSFT | MOD_LALT', label: 'Meh (C+S+A)', description: 'MEH' },
		{
			value: 'MOD_LCTL | MOD_LSFT | MOD_LALT | MOD_LGUI',
			label: 'Hyper (C+S+A+G)',
			description: 'HYPR'
		}
	];

	interface Props {
		open: boolean;
		onSelect: (modifier: string) => void;
		onClose: () => void;
		title?: string;
		description?: string;
		options?: ModifierOption[];
	}

	let {
		open,
		onSelect,
		onClose,
		title = 'Select modifier',
		description,
		options = DEFAULT_OPTIONS
	}: Props = $props();
</script>

<AccessibleDialog
	{open}
	{title}
	description={description ?? 'Pick the modifier that wraps the next keycode.'}
	{onClose}
	titleId="modifier-picker-title"
	panelClass="max-w-md"
>
	<ul class="space-y-1 max-h-96 overflow-y-auto" data-testid="modifier-picker-list">
		{#each options as option (option.value)}
			<li>
				<button
					type="button"
					onclick={() => onSelect(option.value)}
					class="w-full text-left px-3 py-2 rounded border border-border hover:bg-accent transition-colors flex items-center justify-between gap-2"
					data-testid="modifier-picker-option"
				>
					<span class="font-medium text-sm">{option.label}</span>
					{#if option.description}
						<span
							class="text-[10px] font-mono text-muted-foreground bg-muted px-1.5 py-0.5 rounded"
						>
							{option.description}
						</span>
					{/if}
				</button>
			</li>
		{/each}
	</ul>
	<svelte:fragment slot="footer">
		<Button onclick={onClose} variant="ghost">Cancel</Button>
	</svelte:fragment>
</AccessibleDialog>