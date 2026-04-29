<script lang="ts">
	import { onMount } from 'svelte';
	import { apiClient, type ConfigResponse } from '$api';
	import { Button, Card, Input } from '$components';

	let config = $state<ConfigResponse | null>(null);
	let loading = $state(true);
	let saving = $state(false);
	let error = $state<string | null>(null);
	let successMessage = $state<string | null>(null);
	let qmkPath = $state('');

	onMount(async () => {
		try {
			config = await apiClient.getConfig();
			qmkPath = config.qmk_firmware_path || '';
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load config';
		} finally {
			loading = false;
		}
	});

	async function handleSave() {
		saving = true;
		successMessage = null;
		error = null;

		try {
			await apiClient.updateConfig({
				qmk_firmware_path: qmkPath || undefined
			});
			
			// Reload config
			config = await apiClient.getConfig();
			qmkPath = config.qmk_firmware_path || '';
			
			successMessage = 'Settings saved successfully';
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save settings';
		} finally {
			saving = false;
		}
	}
</script>

<div class="container mx-auto p-6">
	<div class="mb-8 flex items-center justify-between">
		<div>
			<h1 class="text-4xl font-bold mb-2">Settings</h1>
			<p class="text-lg font-medium mb-1">Workspace Setup</p>
			<p class="text-muted-foreground">
				Connect LazyQMK to your QMK folder and confirm where layouts live.
			</p>
		</div>
		<Button onclick={() => (window.location.href = '/')}>
			Back to Home
		</Button>
	</div>

	{#if loading}
		<p class="text-muted-foreground">Loading settings...</p>
	{:else if error && !config}
		<Card class="state-panel-error">
			<p class="state-eyebrow mb-3">Setup unavailable</p>
			<h2 class="text-2xl font-semibold text-destructive">Could not load settings</h2>
			<p class="mt-2 text-sm text-muted-foreground">{error}</p>
			<div class="mt-6">
				<Button onclick={() => window.location.reload()}>Retry Loading Settings</Button>
			</div>
		</Card>
	{:else}
		<div class="max-w-2xl space-y-6">
			<Card class="surface-subtle p-4">
				<div class="grid gap-4 md:grid-cols-2 text-sm">
					<div>
						<p class="font-medium">Basic</p>
						<p class="text-muted-foreground mt-1">Set required QMK path so keyboard discovery and builds work.</p>
					</div>
					<div>
						<p class="font-medium">Advanced</p>
						<p class="text-muted-foreground mt-1">Workspace root is informational here. Change it only when starting backend with a different workspace.</p>
					</div>
				</div>
			</Card>

			<!-- Success Message -->
			{#if successMessage}
				<Card class="p-4 bg-green-50 dark:bg-green-950 border-green-200 dark:border-green-800">
					<p class="text-green-800 dark:text-green-200">{successMessage}</p>
				</Card>
			{/if}

			<!-- Error Message -->
			{#if error && config}
				<Card class="p-4 bg-destructive/10 border-destructive">
					<p class="text-destructive">{error}</p>
				</Card>
			{/if}

			<!-- QMK Firmware Path -->
			<Card class="p-6">
				<div class="mb-4 flex items-center justify-between gap-4">
					<div>
						<p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">Basic</p>
						<h2 class="text-xl font-semibold mt-1">QMK firmware folder</h2>
					</div>
					<span class="rounded-full border px-3 py-1 text-xs text-muted-foreground">Required</span>
				</div>
				<p class="text-sm text-muted-foreground mb-4">
					Point LazyQMK at your local <code class="bg-muted px-1 rounded">qmk_firmware</code> folder so keyboard data and firmware builds work.
				</p>
				<Input
					bind:value={qmkPath}
					placeholder="/path/to/qmk_firmware"
					class="mb-4"
				/>
			</Card>

			<!-- Workspace Root -->
			<Card class="p-6">
				<div class="mb-4 flex items-center justify-between gap-4">
					<div>
						<p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">Advanced</p>
						<h2 class="text-xl font-semibold mt-1">Layout workspace</h2>
					</div>
					<span class="rounded-full border px-3 py-1 text-xs text-muted-foreground">Read-only</span>
				</div>
				<p class="text-sm text-muted-foreground mb-4">
					This is where LazyQMK looks for your layout files. Start backend with <code class="bg-muted px-1 rounded">--workspace</code> if you want a different folder.
				</p>
				<p class="text-sm">
					<code class="bg-muted px-2 py-1 rounded">{config?.workspace_root || 'Unknown'}</code>
				</p>
			</Card>

			<!-- Save Button -->
			<div class="flex justify-end">
				<Button
					onclick={handleSave}
					disabled={saving}
				>
					{saving ? 'Saving...' : 'Save Settings'}
				</Button>
			</div>
		</div>
	{/if}
</div>
