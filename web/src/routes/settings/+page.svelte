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
			<p class="text-muted-foreground">
				Configure LazyQMK workspace and paths
			</p>
		</div>
		<Button onclick={() => (window.location.href = '/')}>
			Back to Dashboard
		</Button>
	</div>

	{#if loading}
		<p class="text-muted-foreground">Loading settings...</p>
	{:else if error && !config}
		<Card class="p-6">
			<div class="text-destructive">
				<p class="font-medium">Error loading settings</p>
				<p class="text-sm">{error}</p>
			</div>
		</Card>
	{:else}
		<div class="max-w-2xl space-y-6">
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
				<h2 class="text-xl font-semibold mb-4">QMK Firmware Path</h2>
				<p class="text-sm text-muted-foreground mb-4">
					Path to your QMK firmware directory (for keyboard metadata and compilation)
				</p>
				<Input
					bind:value={qmkPath}
					placeholder="/path/to/qmk_firmware"
					class="mb-4"
				/>
				<p class="text-xs text-muted-foreground">
					Current: {config?.qmk_firmware_path || 'Not configured'}
				</p>
			</Card>

			<!-- Workspace Root -->
			<Card class="p-6">
				<h2 class="text-xl font-semibold mb-4">Workspace Root</h2>
				<p class="text-sm text-muted-foreground mb-4">
					Directory containing your layout files (configured when starting the backend)
				</p>
				<p class="text-sm">
					Current workspace: <code class="bg-muted px-2 py-1 rounded">{config?.output_dir || 'Unknown'}</code>
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
