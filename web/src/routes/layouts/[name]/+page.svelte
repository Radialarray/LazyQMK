<script lang="ts">
	import {
		Button,
		Card,
		KeyboardPreview,
		Input,
		Tabs,
		LayerManager,
		KeycodePicker,
		CategoryManager,
		ColorPicker
	} from '$components';
	import { apiClient } from '$api';
	import type { PageData } from './$types';
	import type {
		GeometryResponse,
		Layout,
		TapDance,
		Combo,
		ValidationResponse,
		InspectResponse,
		ExportResponse,
		GenerateResponse,
		GenerateJob,
		LogEntry,
		Category,
		RgbColor,
		LayoutVariantInfo,
		SwitchVariantResponse,
		RenderMetadataResponse,
		BuildJob,
		BuildArtifact
	} from '$api/types';
	import { ClipboardManager } from '$lib/utils/clipboard';
	import { validateName, parseAndValidateTags, type ValidationError } from '$lib/utils/metadata';
	import {
		shouldCycleLayer,
		shouldHandleEscape,
		shouldOpenPicker
	} from '$lib/utils/keyboardNavigation';
	import { onDestroy } from 'svelte';

	let { data }: { data: PageData } = $props();
	// Initialize layout as mutable state without referencing props
	// The $effect below syncs it with data.layout when data changes
	let layout = $state() as Layout;
	// React to data changes and sync layout
	$effect(() => {
		layout = data.layout;
	});
	let filename = $derived(data.filename);
	let isDirty = $state(false);
	let saveStatus = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');
	let saveError = $state<string | null>(null);

	// Tab navigation
	// Primary tabs - always visible in horizontal bar
	const primaryTabs = [
		{ id: 'preview', label: 'Editor', icon: '⌨️' },
		{ id: 'generate', label: 'Generate', icon: '⚙️' },
		{ id: 'build', label: 'Build', icon: '🔨' }
	];
	
	// Secondary tabs - accessible via "More..." dropdown
	const secondaryTabs = [
		{ id: 'metadata', label: 'Metadata', icon: '📝' },
		{ id: 'layers', label: 'Layers', icon: '📚' },
		{ id: 'categories', label: 'Categories', icon: '🎨' },
		{ id: 'tap-dance', label: 'Tap Dance', icon: '💃' },
		{ id: 'combos', label: 'Combos', icon: '🔗' },
		{ id: 'idle-effect', label: 'Idle Effect', icon: '💤' },
		{ id: 'validate', label: 'Validate', icon: '✓' },
		{ id: 'inspect', label: 'Inspect', icon: '🔍' },
		{ id: 'export', label: 'Export', icon: '📄' }
	];
	
	let activeTab = $state('preview');
	let dropdownOpen = $state(false);
	
	// Check if active tab is in secondary tabs
	const isActiveTabSecondary = $derived(secondaryTabs.some(tab => tab.id === activeTab));

	// Metadata editing state
	let metadataName = $state('');
	let metadataDescription = $state('');
	let metadataAuthor = $state('');
	let metadataTagsInput = $state('');
	let metadataErrors = $state<ValidationError[]>([]);

	// Initialize metadata fields when layout loads
	$effect(() => {
		if (layout) {
			metadataName = layout.metadata.name || '';
			metadataDescription = layout.metadata.description || '';
			metadataAuthor = layout.metadata.author || '';
			metadataTagsInput = (layout.metadata.tags || []).join(', ');
		}
	});

	// State for keyboard preview
	let geometry = $state<GeometryResponse | null>(null);
	let geometryError = $state<string | null>(null);
	let geometryLoading = $state(false);
	let selectedKeyIndex = $state<number | null>(null);
	let selectedLayerIndex = $state(0);
	let hoveredKeyIndex = $state<number | null>(null);

	// Multi-selection state
	let selectionMode = $state(false);
	let selectedKeyIndices = $state<Set<number>>(new Set());

	// Swap mode state
	let swapMode = $state(false);
	let swapFirstKey = $state<number | null>(null);

	// Clipboard state
	const clipboard = new ClipboardManager();
	let clipboardSize = $state(0);
	let canUndo = $state(false);

	// Update clipboard state for reactivity
	function updateClipboardState() {
		clipboardSize = clipboard.getClipboardSize();
		canUndo = clipboard.canUndo(selectedLayerIndex);
	}

	// State for keycode picker
	let keycodePickerOpen = $state(false);
	let editingKeyVisualIndex = $state<number | null>(null);

	// State for color/category editing
	let showKeyColorPicker = $state(false);
	let showLayerColorPicker = $state(false);
	let showLayerCategorySelector = $state(false);

	// State for validation/inspect/export/generate
	let validationResult = $state<ValidationResponse | null>(null);
	let validationLoading = $state(false);
	let inspectResult = $state<InspectResponse | null>(null);
	let inspectLoading = $state(false);
	let exportResult = $state<ExportResponse | null>(null);
	let exportLoading = $state(false);
	let generateResult = $state<GenerateResponse | null>(null);
	let generateLoading = $state(false);

	// Generate job polling state
	let generateJob = $state<GenerateJob | null>(null);
	let generateLogs = $state<LogEntry[]>([]);
	let generatePollingActive = $state(false);
	let generateTimeoutReached = $state(false);
	let generateCancelling = $state(false);
	let pollIntervalId = $state<ReturnType<typeof setInterval> | null>(null);
	let pollTimeoutId = $state<ReturnType<typeof setTimeout> | null>(null);
	const POLL_INTERVAL_MS = 1000; // Poll every 1 second
	const GENERATE_TIMEOUT_MS = 5 * 60 * 1000; // 5 minute timeout

	// Cleanup polling on component destroy
	onDestroy(() => {
		stopPolling();
		stopBuildPolling();
	});

	function stopPolling() {
		if (pollIntervalId) {
			clearInterval(pollIntervalId);
			pollIntervalId = null;
		}
		if (pollTimeoutId) {
			clearTimeout(pollTimeoutId);
			pollTimeoutId = null;
		}
		generatePollingActive = false;
	}

	function resetGenerateState() {
		stopPolling();
		generateResult = null;
		generateJob = null;
		generateLogs = [];
		generateTimeoutReached = false;
		generateCancelling = false;
		generateLoading = false;
	}

	async function pollJobStatus(jobId: string) {
		try {
			const response = await apiClient.getGenerateJob(jobId);
			generateJob = response.job;

			// Fetch logs (append new ones)
			const logsResponse = await apiClient.getGenerateLogs(jobId, generateLogs.length);
			if (logsResponse.logs.length > 0) {
				generateLogs = [...generateLogs, ...logsResponse.logs];
			}

			// Check if job is complete
			const terminalStates: GenerateJob['status'][] = ['completed', 'failed', 'cancelled'];
			if (terminalStates.includes(response.job.status)) {
				stopPolling();
				generateLoading = false;
			}
		} catch (e) {
			console.error('Error polling generate job:', e);
			// Don't stop polling on transient errors
		}
	}

	function startPolling(jobId: string) {
		stopPolling(); // Clear any existing polling
		generatePollingActive = true;
		generateTimeoutReached = false;

		// Start polling interval
		pollIntervalId = setInterval(() => {
			pollJobStatus(jobId);
		}, POLL_INTERVAL_MS);

		// Set timeout
		pollTimeoutId = setTimeout(() => {
			generateTimeoutReached = true;
			stopPolling();
		}, GENERATE_TIMEOUT_MS);

		// Initial poll immediately
		pollJobStatus(jobId);
	}

	async function cancelGenerateJob() {
		if (!generateJob) return;
		generateCancelling = true;
		try {
			const result = await apiClient.cancelGenerate(generateJob.id);
			if (result.success) {
				// Will be picked up by next poll, or manually refresh
				await pollJobStatus(generateJob.id);
			}
		} catch (e) {
			console.error('Error cancelling generate job:', e);
		} finally {
			generateCancelling = false;
		}
	}

	// State for save as template
	let showSaveTemplateDialog = $state(false);
	let templateName = $state('');
	let templateTags = $state('');
	let saveTemplateLoading = $state(false);
	let saveTemplateError = $state<string | null>(null);

	// State for render metadata (preview features)
	let renderMetadata = $state<RenderMetadataResponse | null>(null);
	let renderMetadataLoading = $state(false);
	let renderMetadataError = $state<string | null>(null);

	// State for switch layout variant
	let showVariantSwitchDialog = $state(false);
	let availableVariants = $state<LayoutVariantInfo[]>([]);
	let variantsLoading = $state(false);
	let variantsError = $state<string | null>(null);
	let switchingVariant = $state(false);
	let switchVariantError = $state<string | null>(null);
	let switchVariantWarning = $state<string | null>(null);

	// State for Build tab (firmware compilation)
	let buildJob = $state<BuildJob | null>(null);
	let buildLogs = $state<LogEntry[]>([]);
	let buildArtifacts = $state<BuildArtifact[]>([]);
	let buildHistory = $state<BuildJob[]>([]);
	let buildLoading = $state(false);
	let buildPollingActive = $state(false);
	let buildCancelling = $state(false);
	let buildAutoScroll = $state(true);
	let buildPollIntervalId = $state<ReturnType<typeof setInterval> | null>(null);
	let buildLogOffset = $state(0);
	let buildLogsElement: HTMLDivElement | undefined = $state();
	const BUILD_POLL_INTERVAL_MS = 1000; // Poll every 1 second

	// Build polling and management functions
	function stopBuildPolling() {
		if (buildPollIntervalId) {
			clearInterval(buildPollIntervalId);
			buildPollIntervalId = null;
		}
		buildPollingActive = false;
	}

	function resetBuildState() {
		stopBuildPolling();
		buildJob = null;
		buildLogs = [];
		buildArtifacts = [];
		buildLogOffset = 0;
		buildLoading = false;
		buildCancelling = false;
	}

	async function pollBuildJobStatus(jobId: string) {
		try {
			const response = await apiClient.getBuildJob(jobId);
			buildJob = response.job;

			// Fetch logs incrementally
			const logsResponse = await apiClient.getBuildLogs(jobId, buildLogOffset, 100);
			if (logsResponse.logs.length > 0) {
				buildLogs = [...buildLogs, ...logsResponse.logs];
				buildLogOffset += logsResponse.logs.length;
				
				// Auto-scroll logs if enabled
				if (buildAutoScroll && buildLogsElement) {
					setTimeout(() => {
						buildLogsElement?.scrollTo({ top: buildLogsElement.scrollHeight, behavior: 'smooth' });
					}, 50);
				}
			}

			// Check if job is complete
			const terminalStates: BuildJob['status'][] = ['completed', 'failed', 'cancelled'];
			if (terminalStates.includes(response.job.status)) {
				stopBuildPolling();
				buildLoading = false;
				
				// Fetch artifacts if completed
				if (response.job.status === 'completed') {
					await loadBuildArtifacts(jobId);
				}
			}
		} catch (e) {
			console.error('Error polling build job:', e);
			// Don't stop polling on transient errors
		}
	}

	function startBuildPolling(jobId: string) {
		stopBuildPolling(); // Clear any existing polling
		buildPollingActive = true;

		// Start polling interval
		buildPollIntervalId = setInterval(() => {
			pollBuildJobStatus(jobId);
		}, BUILD_POLL_INTERVAL_MS);

		// Initial poll immediately
		pollBuildJobStatus(jobId);
	}

	async function loadBuildArtifacts(jobId: string) {
		try {
			const response = await apiClient.getBuildArtifacts(jobId);
			buildArtifacts = response.artifacts;
		} catch (e) {
			console.error('Error loading build artifacts:', e);
			buildArtifacts = [];
		}
	}

	async function loadBuildHistory() {
		try {
			const jobs = await apiClient.listBuildJobs();
			// Filter to jobs for this layout
			buildHistory = jobs.filter(j => j.layout_filename === filename).slice(0, 10);
		} catch (e) {
			console.error('Error loading build history:', e);
			buildHistory = [];
		}
	}

	async function startBuild() {
		if (!filename) return;

		// Require save before build
		if (isDirty) {
			// This shouldn't happen - UI should prevent it - but just in case
			console.warn('Cannot start build: layout has unsaved changes');
			return;
		}

		resetBuildState();
		buildLoading = true;

		try {
			const response = await apiClient.startBuild(filename);
			buildJob = response.job;
			startBuildPolling(response.job.id);
			// Refresh history to show new job
			await loadBuildHistory();
		} catch (e) {
			buildLoading = false;
			console.error('Error starting build:', e);
		}
	}

	async function cancelBuildJob() {
		if (!buildJob) return;
		buildCancelling = true;
		try {
			const result = await apiClient.cancelBuild(buildJob.id);
			if (result.success) {
				// Will be picked up by next poll
				await pollBuildJobStatus(buildJob.id);
			}
		} catch (e) {
			console.error('Error cancelling build job:', e);
		} finally {
			buildCancelling = false;
		}
	}

	async function selectBuildJob(job: BuildJob) {
		resetBuildState();
		buildJob = job;
		
		// Load logs for this job
		try {
			const logsResponse = await apiClient.getBuildLogs(job.id, 0, 500);
			buildLogs = logsResponse.logs;
			buildLogOffset = logsResponse.logs.length;
		} catch (e) {
			console.error('Error loading build logs:', e);
		}

		// Load artifacts if completed
		if (job.status === 'completed') {
			await loadBuildArtifacts(job.id);
		}

		// Resume polling if job is active
		const activeStates: BuildJob['status'][] = ['pending', 'running'];
		if (activeStates.includes(job.status)) {
			buildLoading = true;
			startBuildPolling(job.id);
		}
	}

	function copyBuildLogs() {
		const logText = buildLogs.map(log => `[${log.level}] ${log.message}`).join('\n');
		navigator.clipboard.writeText(logText).then(() => {
			console.log('Build logs copied to clipboard');
		}).catch(e => {
			console.error('Failed to copy build logs:', e);
		});
	}

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}

	function getLogLevelColor(level: string): string {
		switch (level.toUpperCase()) {
			case 'ERROR':
				return 'text-red-500';
			case 'WARN':
			case 'WARNING':
				return 'text-yellow-500';
			case 'INFO':
				return 'text-blue-400';
			case 'DEBUG':
				return 'text-gray-500';
			default:
				return 'text-muted-foreground';
		}
	}

	function getBuildStatusBadge(status: string): { class: string; icon: string; text: string } {
		switch (status) {
			case 'pending':
				return { class: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200', icon: '⏳', text: 'Pending' };
			case 'running':
				return { class: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200', icon: '🔄', text: 'Running' };
			case 'completed':
				return { class: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200', icon: '✅', text: 'Completed' };
			case 'failed':
				return { class: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200', icon: '❌', text: 'Failed' };
			case 'cancelled':
				return { class: 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200', icon: '🚫', text: 'Cancelled' };
			default:
				return { class: 'bg-gray-100 text-gray-800', icon: '❓', text: status };
		}
	}

	// Load geometry when layout is available
	$effect(() => {
		if (layout?.metadata.keyboard && layout?.metadata.layout_variant) {
			loadGeometry(layout.metadata.keyboard, layout.metadata.layout_variant);
		} else if (layout?.metadata.keyboard && layout?.metadata.layout) {
			loadGeometry(layout.metadata.keyboard, layout.metadata.layout);
		}
	});

	// Load render metadata when filename or layer changes
	$effect(() => {
		if (filename) {
			loadRenderMetadata(filename);
		}
	});

	// Load build history when switching to Build tab
	$effect(() => {
		if (activeTab === 'build' && filename) {
			loadBuildHistory();
		}
	});

	async function loadGeometry(keyboard: string, layoutName: string) {
		geometryLoading = true;
		geometryError = null;
		
		const maxRetries = 3;
		const retryDelayMs = 500;
		
		for (let attempt = 1; attempt <= maxRetries; attempt++) {
			try {
				console.log(`[loadGeometry] Attempt ${attempt}/${maxRetries} - Loading geometry for ${keyboard}/${layoutName}`);
				geometry = await apiClient.getGeometry(keyboard, layoutName);
				console.log(`[loadGeometry] Success - Loaded ${geometry.keys.length} keys`);
				geometryLoading = false;
				return; // Success, exit early
			} catch (e) {
				const errorMsg = e instanceof Error ? e.message : 'Failed to load keyboard geometry';
				console.warn(`[loadGeometry] Attempt ${attempt}/${maxRetries} failed:`, errorMsg);
				
				// On last attempt, set error state and give up
				if (attempt === maxRetries) {
					geometryError = errorMsg;
					geometry = null;
					console.error(`[loadGeometry] All attempts failed. Final error:`, errorMsg);
				} else {
					// Wait before retry, but only log on first retry to avoid spam
					if (attempt === 1) {
						console.log(`[loadGeometry] Backend may still be starting, will retry...`);
					}
					await new Promise(resolve => setTimeout(resolve, retryDelayMs));
				}
			}
		}
		
		geometryLoading = false;
	}

	async function loadRenderMetadata(layoutFilename: string) {
		renderMetadataLoading = true;
		renderMetadataError = null;
		try {
			console.log(`[loadRenderMetadata] Loading render metadata for ${layoutFilename}`);
			renderMetadata = await apiClient.getRenderMetadata(layoutFilename);
			console.log(`[loadRenderMetadata] Success - Loaded metadata for ${renderMetadata.layers.length} layers`);
		} catch (e) {
			const errorMsg = e instanceof Error ? e.message : 'Failed to load render metadata';
			console.warn(`[loadRenderMetadata] Failed:`, errorMsg);
			renderMetadataError = errorMsg;
			renderMetadata = null;
		} finally {
			renderMetadataLoading = false;
		}
	}

	async function openVariantSwitch() {
		if (!layout?.metadata.keyboard) {
			variantsError = 'No keyboard defined for this layout';
			return;
		}
		showVariantSwitchDialog = true;
		variantsLoading = true;
		variantsError = null;
		switchVariantError = null;
		switchVariantWarning = null;
		try {
			const response = await apiClient.listKeyboardLayouts(layout.metadata.keyboard);
			availableVariants = response.variants;
		} catch (e) {
			variantsError = e instanceof Error ? e.message : 'Failed to load layout variants';
		} finally {
			variantsLoading = false;
		}
	}

	async function switchToVariant(variantName: string) {
		switchingVariant = true;
		switchVariantError = null;
		switchVariantWarning = null;
		try {
			const response: SwitchVariantResponse = await apiClient.switchLayoutVariant(filename, variantName);
			layout = response.layout;
			if (response.warning) {
				switchVariantWarning = response.warning;
			}
			isDirty = false; // Layout was saved by the backend
			showVariantSwitchDialog = false;
			// Reload geometry for the new variant
			if (layout.metadata.keyboard && layout.metadata.layout_variant) {
				await loadGeometry(layout.metadata.keyboard, layout.metadata.layout_variant);
			}
		} catch (e) {
			switchVariantError = e instanceof Error ? e.message : 'Failed to switch layout variant';
		} finally {
			switchingVariant = false;
		}
	}

	function handleKeyClick(visualIndex: number, matrixRow: number, matrixCol: number, shiftKey: boolean) {
		// Handle swap mode first
		if (swapMode) {
			handleSwap(visualIndex);
			return;
		}

		if (selectionMode || shiftKey) {
			// Toggle selection in selection mode
			const newSelection = new Set(selectedKeyIndices);
			
			// When shift-clicking without selection mode, include the previously selected key
			// This allows shift-click multi-selection to work without enabling selection mode first
			if (shiftKey && !selectionMode && selectedKeyIndex !== null && !newSelection.has(selectedKeyIndex)) {
				newSelection.add(selectedKeyIndex);
			}
			
			if (newSelection.has(visualIndex)) {
				newSelection.delete(visualIndex);
			} else {
				newSelection.add(visualIndex);
			}
			selectedKeyIndices = newSelection;
			// Keep single selection for key details panel
			selectedKeyIndex = visualIndex;
		} else {
			// Single selection mode (default behavior)
			selectedKeyIndex = visualIndex;
			selectedKeyIndices = new Set();
			// Immediately open keycode picker for TUI-like UX
			openKeycodePicker();
		}
	}
	
	/**
	 * Handle keyboard navigation from KeyboardPreview
	 */
	function handleKeyboardNavigation(newKeyIndex: number | null, newSelectedIndices: Set<number>) {
		selectedKeyIndex = newKeyIndex;
		selectedKeyIndices = newSelectedIndices;
	}

	/**
	 * Handle key hover from KeyboardPreview
	 */
	function handleKeyHover(newHoveredIndex: number | null) {
		hoveredKeyIndex = newHoveredIndex;
	}
	
	/**
	 * Handle global keyboard shortcuts (layer cycling, escape, enter)
	 */
	function handleGlobalKeydown(event: KeyboardEvent) {
		// Layer cycling with [ and ]
		const cycleDirection = shouldCycleLayer(event);
		if (cycleDirection !== null && layout) {
			event.preventDefault();
			const currentIndex = selectedLayerIndex;
			const layerCount = layout.layers.length;
			
			if (cycleDirection === 'prev') {
				handleLayerChange((currentIndex - 1 + layerCount) % layerCount);
			} else {
				handleLayerChange((currentIndex + 1) % layerCount);
			}
			return;
		}
		
		// Shift+W for swap mode
		if (event.shiftKey && event.key === 'W') {
			event.preventDefault();
			toggleSwapMode();
			return;
		}
		
		// Escape key: close picker, clear swap mode, or clear selection
		if (shouldHandleEscape(event)) {
			event.preventDefault();
			if (keycodePickerOpen) {
				handleKeycodePickerClose();
			} else if (swapMode) {
				swapMode = false;
				swapFirstKey = null;
			} else if (selectedKeyIndices.size > 0 || selectionMode) {
				clearSelection();
			}
			return;
		}
		
		// Enter key: open keycode picker if a key is selected
		if (shouldOpenPicker(event)) {
			event.preventDefault();
			if (selectedKeyIndex !== null && !keycodePickerOpen) {
				openKeycodePicker();
			}
			return;
		}
	}

	function toggleSelectionMode() {
		selectionMode = !selectionMode;
		if (!selectionMode) {
			selectedKeyIndices = new Set();
		}
	}

	function toggleSwapMode() {
		swapMode = !swapMode;
		if (!swapMode) {
			swapFirstKey = null;
		}
		// Exit selection mode when entering swap mode (modes are mutually exclusive)
		if (swapMode) {
			selectionMode = false;
			selectedKeyIndices = new Set();
		}
	}

	async function handleSwap(targetIndex: number) {
		if (swapFirstKey === null) {
			// First key selection
			swapFirstKey = targetIndex;
			saveStatus = 'idle';
			saveError = 'Swap mode - click second key to swap';
		} else if (swapFirstKey === targetIndex) {
			// Clicked same key - show error
			saveStatus = 'error';
			saveError = 'Cannot swap a key with itself';
		} else {
			// Perform swap via API
			const layer = layout.layers[selectedLayerIndex];
			const firstKey = layer.keys.find(k => k.visual_index === swapFirstKey);
			const secondKey = layer.keys.find(k => k.visual_index === targetIndex);
			
			if (firstKey && secondKey && firstKey.position && secondKey.position) {
				try {
					saveStatus = 'saving';
					saveError = null;
					
					// Call API to swap keys
					await apiClient.swapKeys(filename, {
						layer: selectedLayerIndex,
						first_position: { row: firstKey.position.row, col: firstKey.position.col },
						second_position: { row: secondKey.position.row, col: secondKey.position.col }
					});
					
					// Reload layout to get the swapped data
					layout = await apiClient.getLayout(filename);
					
					// Show status
					saveStatus = 'saved';
					saveError = 'Keys swapped';
					setTimeout(() => {
						if (saveStatus === 'saved' && saveError === 'Keys swapped') {
							saveStatus = 'idle';
							saveError = null;
						}
					}, 2000);
				} catch (e) {
					saveStatus = 'error';
					saveError = e instanceof Error ? e.message : 'Failed to swap keys';
				}
			} else {
				saveStatus = 'error';
				saveError = 'Could not find key positions for swap';
			}
			
			// Exit swap mode
			swapMode = false;
			swapFirstKey = null;
		}
	}

	function clearSelection() {
		selectedKeyIndices = new Set();
		selectedKeyIndex = null;
	}

	// Clipboard operations
	function handleCopy() {
		if (!layout) return;
		const count = clipboard.copyKeys(currentLayerKeys, selectedKeyIndices);
		updateClipboardState();
		console.log(`Copied ${count} keys`);
	}

	function handleCut() {
		if (!layout) return;
		const updatedKeys = clipboard.cutKeys(currentLayerKeys, selectedKeyIndices, selectedLayerIndex);
		layout.layers[selectedLayerIndex].keys = updatedKeys;
		layout.layers = [...layout.layers];
		isDirty = true;
		updateClipboardState();
		console.log(`Cut ${selectedKeyIndices.size} keys`);
	}

	function handlePaste() {
		if (!layout) return;
		const selection: Set<number> = selectedKeyIndices.size > 0 ? selectedKeyIndices : 
			(selectedKeyIndex !== null ? new Set([selectedKeyIndex]) : new Set());
		
		const updatedKeys = clipboard.pasteKeys(currentLayerKeys, selection, selectedLayerIndex);
		if (updatedKeys) {
			layout.layers[selectedLayerIndex].keys = updatedKeys;
			layout.layers = [...layout.layers];
			isDirty = true;
			updateClipboardState();
			console.log(`Pasted to ${selection.size} keys`);
		}
	}

	function handleUndo() {
		if (!layout) return;
		const undoKeys = clipboard.undo(currentLayerKeys, selectedLayerIndex);
		if (undoKeys) {
			layout.layers[selectedLayerIndex].keys = undoKeys;
			layout.layers = [...layout.layers];
			isDirty = true;
			updateClipboardState();
			console.log('Undo successful');
		}
	}

	function openKeycodePicker() {
		if (selectedKeyIndex === null) return;
		editingKeyVisualIndex = selectedKeyIndex;
		keycodePickerOpen = true;
	}

	function handleKeycodeSelect(keycode: string) {
		if (!layout || editingKeyVisualIndex === null) return;
		
		// Find the key in the current layer and update its keycode
		const keyIndex = layout.layers[selectedLayerIndex].keys.findIndex(
			(k) => k.visual_index === editingKeyVisualIndex
		);
		
		if (keyIndex !== -1) {
			layout.layers[selectedLayerIndex].keys[keyIndex].keycode = keycode;
			layout.layers = [...layout.layers]; // Trigger reactivity
			isDirty = true;
		}
		
		keycodePickerOpen = false;
		editingKeyVisualIndex = null;
	}

	function handleKeycodePickerClose() {
		keycodePickerOpen = false;
		editingKeyVisualIndex = null;
	}

	function handleLayerChange(index: number) {
		selectedLayerIndex = index;
		// Clear selection when changing layers
		selectedKeyIndices = new Set();
		// Preserve key selection when switching layers
	}

	// Get key assignments for the current layer
	const currentLayerKeys = $derived(layout?.layers[selectedLayerIndex]?.keys ?? []);

	// Get render metadata for the current layer
	const currentLayerRenderMetadata = $derived.by(() => {
		if (!renderMetadata || !renderMetadata.layers) return [];
		const layerMeta = renderMetadata.layers.find(l => l.number === selectedLayerIndex);
		return layerMeta?.keys ?? [];
	});

	// Get selected key details
	const selectedKey = $derived.by(() => {
		if (selectedKeyIndex === null || !currentLayerKeys.length) return null;
		return currentLayerKeys.find((k) => k.visual_index === selectedKeyIndex) ?? null;
	});

	// Get hovered key details (for preview panel)
	const hoveredKey = $derived.by(() => {
		if (hoveredKeyIndex === null || !currentLayerKeys.length) return null;
		return currentLayerKeys.find((k) => k.visual_index === hoveredKeyIndex) ?? null;
	});

	// Get active key for details panel (hover takes precedence over selection for preview)
	const activeKeyIndex = $derived(hoveredKeyIndex ?? selectedKeyIndex);
	const activeKey = $derived(hoveredKey ?? selectedKey);
	const activeKeyRenderMetadata = $derived.by(() => {
		if (activeKeyIndex === null || !currentLayerRenderMetadata.length) return null;
		return currentLayerRenderMetadata.find((k) => k.visual_index === activeKeyIndex) ?? null;
	});

	// Save functionality
	async function saveLayout() {
		if (!layout || !filename) return;
		saveStatus = 'saving';
		saveError = null;
		try {
			await apiClient.saveLayout(filename, layout);
			saveStatus = 'saved';
			isDirty = false;
			setTimeout(() => {
				if (saveStatus === 'saved') saveStatus = 'idle';
			}, 2000);
		} catch (e) {
			saveStatus = 'error';
			saveError = e instanceof Error ? e.message : 'Failed to save';
		}
	}

	// Tap Dance management
	function addTapDance() {
		if (!layout) return;
		const newTd: TapDance = {
			name: `TD_${(layout.tap_dances?.length ?? 0) + 1}`,
			single_tap: 'KC_NO',
			double_tap: undefined,
			hold: undefined
		};
		layout.tap_dances = [...(layout.tap_dances ?? []), newTd];
		isDirty = true;
	}

	function updateTapDance(index: number, field: keyof TapDance, value: string) {
		if (!layout?.tap_dances) return;
		const td = layout.tap_dances[index];
		if (field === 'name') td.name = value;
		else if (field === 'single_tap') td.single_tap = value;
		else if (field === 'double_tap') td.double_tap = value || undefined;
		else if (field === 'hold') td.hold = value || undefined;
		layout.tap_dances = [...layout.tap_dances];
		isDirty = true;
	}

	function removeTapDance(index: number) {
		if (!layout?.tap_dances) return;
		layout.tap_dances = layout.tap_dances.filter((_, i) => i !== index);
		isDirty = true;
	}

	// Combo management
	function addCombo() {
		if (!layout) return;
		const newCombo: Combo = {
			id: `combo_${(layout.combos?.length ?? 0) + 1}`,
			name: `Combo ${(layout.combos?.length ?? 0) + 1}`,
			keys: ['KC_A', 'KC_B'],
			output: 'KC_C'
		};
		layout.combos = [...(layout.combos ?? []), newCombo];
		isDirty = true;
	}

	function updateCombo(index: number, field: keyof Combo, value: string | string[]) {
		if (!layout?.combos) return;
		const combo = layout.combos[index];
		if (field === 'keys') {
			combo.keys = Array.isArray(value) ? value : value.split(',').map((k) => k.trim());
		} else if (field === 'id') {
			combo.id = value as string;
		} else if (field === 'name') {
			combo.name = value as string;
		} else if (field === 'output') {
			combo.output = value as string;
		}
		layout.combos = [...layout.combos];
		isDirty = true;
	}

	function removeCombo(index: number) {
		if (!layout?.combos) return;
		layout.combos = layout.combos.filter((_, i) => i !== index);
		isDirty = true;
	}

	// Idle effect settings
	function updateIdleEffect(field: string, value: boolean | number | string) {
		if (!layout) return;
		if (!layout.idle_effect_settings) {
			layout.idle_effect_settings = {
				enabled: true,
				idle_timeout_ms: 60000,
				idle_effect_duration_ms: 300000,
				idle_effect_mode: 'Breathing'
			};
		}
		if (field === 'enabled') {
			layout.idle_effect_settings.enabled = value as boolean;
		} else if (field === 'idle_timeout_ms') {
			layout.idle_effect_settings.idle_timeout_ms = value as number;
		} else if (field === 'idle_effect_duration_ms') {
			layout.idle_effect_settings.idle_effect_duration_ms = value as number;
		} else if (field === 'idle_effect_mode') {
			layout.idle_effect_settings.idle_effect_mode = value as string;
		}
		layout = { ...layout };
		isDirty = true;
	}

	// Layer management
	function handleLayersChange(newLayers: typeof layout.layers) {
		if (!layout) return;
		layout.layers = newLayers;
		layout = { ...layout };
		isDirty = true;
	}

	// Category management
	function handleCategoriesChange(newCategories: Category[]) {
		if (!layout) return;
		layout.categories = newCategories;
		layout = { ...layout };
		isDirty = true;
	}

	// Key color/category management
	function setKeyColorOverride(color: RgbColor) {
		if (!layout || selectedKeyIndex === null) return;
		const keyIndex = layout.layers[selectedLayerIndex].keys.findIndex(
			(k) => k.visual_index === selectedKeyIndex
		);
		if (keyIndex !== -1) {
			// Create new key object with color override
			const updatedKey = { ...layout.layers[selectedLayerIndex].keys[keyIndex], color_override: color };
			// Create new keys array
			const updatedKeys = [...layout.layers[selectedLayerIndex].keys];
			updatedKeys[keyIndex] = updatedKey;
			// Create new layers array with updated layer
			const updatedLayers = [...layout.layers];
			updatedLayers[selectedLayerIndex] = { ...layout.layers[selectedLayerIndex], keys: updatedKeys };
			// Assign to trigger reactivity
			layout.layers = updatedLayers;
			// Force full layout reactivity
			layout = { ...layout };
			isDirty = true;
		}
		showKeyColorPicker = false;
	}

	function clearKeyColorOverride() {
		if (!layout || selectedKeyIndex === null) return;
		const keyIndex = layout.layers[selectedLayerIndex].keys.findIndex(
			(k) => k.visual_index === selectedKeyIndex
		);
		if (keyIndex !== -1) {
			// Create new key object without color override
			const updatedKey = { ...layout.layers[selectedLayerIndex].keys[keyIndex] };
			delete updatedKey.color_override;
			// Create new keys array
			const updatedKeys = [...layout.layers[selectedLayerIndex].keys];
			updatedKeys[keyIndex] = updatedKey;
			// Create new layers array with updated layer
			const updatedLayers = [...layout.layers];
			updatedLayers[selectedLayerIndex] = { ...layout.layers[selectedLayerIndex], keys: updatedKeys };
			// Assign to trigger reactivity
			layout.layers = updatedLayers;
			// Force full layout reactivity
			layout = { ...layout };
			isDirty = true;
		}
		showKeyColorPicker = false;
	}

	function setKeyCategory(categoryId: string | undefined) {
		if (!layout || selectedKeyIndex === null) return;
		const keyIndex = layout.layers[selectedLayerIndex].keys.findIndex(
			(k) => k.visual_index === selectedKeyIndex
		);
		if (keyIndex !== -1) {
			// Create new key object with category
			const updatedKey = { ...layout.layers[selectedLayerIndex].keys[keyIndex], category_id: categoryId };
			// Create new keys array
			const updatedKeys = [...layout.layers[selectedLayerIndex].keys];
			updatedKeys[keyIndex] = updatedKey;
			// Create new layers array with updated layer
			const updatedLayers = [...layout.layers];
			updatedLayers[selectedLayerIndex] = { ...layout.layers[selectedLayerIndex], keys: updatedKeys };
			// Assign to trigger reactivity
			layout.layers = updatedLayers;
			// Force full layout reactivity
			layout = { ...layout };
			isDirty = true;
		}
	}

	function setKeyDescription(description: string | undefined) {
		if (!layout || selectedKeyIndex === null) return;
		const keyIndex = layout.layers[selectedLayerIndex].keys.findIndex(
			(k) => k.visual_index === selectedKeyIndex
		);
		if (keyIndex !== -1) {
			// Normalize empty string to undefined
			const normalizedDescription = description?.trim() || undefined;
			// Create new key object with description
			const updatedKey = { ...layout.layers[selectedLayerIndex].keys[keyIndex], description: normalizedDescription };
			// Create new keys array
			const updatedKeys = [...layout.layers[selectedLayerIndex].keys];
			updatedKeys[keyIndex] = updatedKey;
			// Create new layers array with updated layer
			const updatedLayers = [...layout.layers];
			updatedLayers[selectedLayerIndex] = { ...layout.layers[selectedLayerIndex], keys: updatedKeys };
			// Assign to trigger reactivity
			layout.layers = updatedLayers;
			// Force full layout reactivity
			layout = { ...layout };
			isDirty = true;
		}
	}

	// Layer color/category management
	function setLayerDefaultColor(color: RgbColor) {
		if (!layout) return;
		layout.layers[selectedLayerIndex].default_color = color;
		layout.layers = [...layout.layers];
		isDirty = true;
		showLayerColorPicker = false;
	}

	function setLayerCategory(categoryId: string | undefined) {
		if (!layout) return;
		layout.layers[selectedLayerIndex].category_id = categoryId;
		layout.layers = [...layout.layers];
		isDirty = true;
	}

	// Validate
	async function runValidation() {
		if (!filename) return;
		validationLoading = true;
		try {
			validationResult = await apiClient.validateLayout(filename);
		} catch (e) {
			validationResult = {
				valid: false,
				error: e instanceof Error ? e.message : 'Validation failed',
				warnings: []
			};
		} finally {
			validationLoading = false;
		}
	}

	// Inspect
	async function runInspect() {
		if (!filename) return;
		inspectLoading = true;
		try {
			inspectResult = await apiClient.inspectLayout(filename);
		} catch (e) {
			inspectResult = null;
		} finally {
			inspectLoading = false;
		}
	}

	// Export
	async function runExport() {
		if (!filename) return;
		exportLoading = true;
		try {
			exportResult = await apiClient.exportLayout(filename);
		} catch (e) {
			exportResult = null;
		} finally {
			exportLoading = false;
		}
	}

	function downloadExport() {
		if (!exportResult) return;
		const blob = new Blob([exportResult.markdown], { type: 'text/markdown' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = exportResult.suggested_filename;
		a.click();
		URL.revokeObjectURL(url);
	}

	// Generate
	async function runGenerate() {
		if (!filename) return;
		
		// Reset state for fresh generation
		resetGenerateState();
		generateLoading = true;
		
		try {
			generateResult = await apiClient.generateFirmware(filename);
			// The response now includes the job object
			if (generateResult.job) {
				generateJob = generateResult.job;
				// Start polling for status updates
				startPolling(generateResult.job.id);
			}
		} catch (e) {
			generateResult = {
				status: 'error',
				message: e instanceof Error ? e.message : 'Generation failed',
				job: null as unknown as GenerateJob // Type workaround for error case
			};
			generateLoading = false;
		}
	}

	// Save as template
	function openSaveTemplateDialog() {
		templateName = layout?.metadata.name || '';
		templateTags = '';
		saveTemplateError = null;
		showSaveTemplateDialog = true;
	}

	function closeSaveTemplateDialog() {
		showSaveTemplateDialog = false;
		templateName = '';
		templateTags = '';
		saveTemplateError = null;
	}

	async function saveAsTemplate() {
		if (!filename || !templateName.trim()) {
			saveTemplateError = 'Please enter a template name';
			return;
		}

		saveTemplateLoading = true;
		saveTemplateError = null;

		try {
			const tags = templateTags
				.split(',')
				.map((t) => t.trim())
				.filter((t) => t.length > 0);

			await apiClient.saveAsTemplate(filename, {
				name: templateName.trim(),
				tags
			});

			closeSaveTemplateDialog();
			// Show success message
			saveStatus = 'saved';
			saveError = 'Saved as template!';
			setTimeout(() => {
				if (saveStatus === 'saved') saveStatus = 'idle';
			}, 3000);
		} catch (e) {
			saveTemplateError = e instanceof Error ? e.message : 'Failed to save template';
		} finally {
			saveTemplateLoading = false;
		}
	}

	// Metadata editing functions
	function updateMetadataField(field: 'name' | 'description' | 'author' | 'tags', value: string) {
		if (!layout) return;
		
		if (field === 'name') {
			metadataName = value;
			layout.metadata.name = value;
		} else if (field === 'description') {
			metadataDescription = value;
			layout.metadata.description = value;
		} else if (field === 'author') {
			metadataAuthor = value;
			layout.metadata.author = value;
		} else if (field === 'tags') {
			metadataTagsInput = value;
			// Don't update layout.metadata.tags until validation passes
		}
		
		// Validate metadata
		validateMetadataFields();
		
		// Only mark dirty if validation passes for critical fields (name)
		if (field === 'name') {
			const nameError = validateName(value);
			if (!nameError) {
				isDirty = true;
			}
		} else {
			isDirty = true;
		}
	}

	function validateMetadataFields() {
		const errors: ValidationError[] = [];
		
		// Validate name
		const nameError = validateName(metadataName);
		if (nameError) {
			errors.push(nameError);
		}
		
		// Validate and parse tags
		const tagsResult = parseAndValidateTags(metadataTagsInput);
		if (!tagsResult.valid) {
			errors.push({ field: 'tags', message: tagsResult.error });
		} else if (layout) {
			// Update tags in layout if valid
			layout.metadata.tags = tagsResult.tags;
		}
		
		metadataErrors = errors;
	}

	function getFieldError(field: string): string | null {
		const error = metadataErrors.find(e => e.field === field);
		return error ? error.message : null;
	}

	// Check if save should be blocked due to metadata validation errors
	const canSave = $derived(!metadataErrors.some(e => e.field === 'name' || e.field === 'tags'));
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="container mx-auto p-6">
	<!-- Header -->
	<div class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold mb-1">
				{layout?.metadata.name || 'Loading...'}
			</h1>
			<p class="text-muted-foreground text-sm">
				{layout?.metadata.description || ''}
			</p>
		</div>
		<div class="flex items-center gap-3">
			{#if isDirty}
				<span class="text-sm text-yellow-500">Unsaved changes</span>
			{/if}
			{#if saveStatus === 'saved'}
				<span class="text-sm text-green-500">Saved!</span>
			{:else if saveStatus === 'error'}
				<span class="text-sm text-red-500">{saveError}</span>
			{/if}
			<Button onclick={saveLayout} disabled={!isDirty || !canSave || saveStatus === 'saving'} data-testid="save-button">
				{saveStatus === 'saving' ? 'Saving...' : 'Save'}
			</Button>
			<Button onclick={openSaveTemplateDialog} variant="outline" data-testid="save-template-button">Save as Template</Button>
			<a href="/layouts">
				<Button>Back</Button>
			</a>
		</div>
	</div>

	<!-- Tab Navigation with Dropdown -->
	<div class="tabs-container mb-6" data-testid="tab-navigation">
		<div class="flex border-b border-border relative">
			<!-- Primary Tabs -->
			{#each primaryTabs as tab}
				<button
					onclick={() => { activeTab = tab.id; dropdownOpen = false; }}
					class="tab-button px-4 py-2 text-sm font-medium transition-colors
						{activeTab === tab.id
						? 'border-b-2 border-primary text-primary'
						: 'text-muted-foreground hover:text-foreground hover:bg-accent'}"
					aria-selected={activeTab === tab.id}
					role="tab"
					data-testid="tab-{tab.id}"
				>
					{#if tab.icon}
						<span class="mr-2">{tab.icon}</span>
					{/if}
					{tab.label}
				</button>
			{/each}
			
			<!-- More... Dropdown Button -->
			<div class="relative">
				<button
					onclick={() => dropdownOpen = !dropdownOpen}
					class="tab-button px-4 py-2 text-sm font-medium transition-colors flex items-center gap-1
						{isActiveTabSecondary
						? 'border-b-2 border-primary text-primary'
						: 'text-muted-foreground hover:text-foreground hover:bg-accent'}"
					aria-haspopup="true"
					aria-expanded={dropdownOpen}
					data-testid="tab-more-dropdown"
				>
					<span class="mr-1">⋯</span>
					More...
					<svg class="w-4 h-4 ml-1 transition-transform {dropdownOpen ? 'rotate-180' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
					</svg>
				</button>
				
				<!-- Dropdown Menu -->
				{#if dropdownOpen}
					<!-- Click-outside handler -->
					<button
						class="fixed inset-0 z-10"
						onclick={() => dropdownOpen = false}
						onkeydown={(e) => e.key === 'Escape' && (dropdownOpen = false)}
						tabindex="-1"
						aria-label="Close dropdown"
					></button>
					
					<!-- Dropdown content -->
					<div
						class="absolute top-full left-0 mt-1 w-56 bg-popover border border-border rounded-lg shadow-lg z-20 py-1"
						role="menu"
						data-testid="tab-dropdown-menu"
					>
						{#each secondaryTabs as tab}
							<button
								onclick={() => { activeTab = tab.id; dropdownOpen = false; }}
								class="w-full px-4 py-2 text-sm text-left flex items-center gap-2 transition-colors
									{activeTab === tab.id
									? 'bg-accent text-primary font-medium'
									: 'text-foreground hover:bg-accent'}"
								role="menuitem"
								data-testid="dropdown-tab-{tab.id}"
							>
								{#if tab.icon}
									<span>{tab.icon}</span>
								{/if}
								{tab.label}
								{#if activeTab === tab.id}
									<span class="ml-auto text-primary">✓</span>
								{/if}
							</button>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	</div>

	<!-- Tab Content -->
	{#if layout}
		{#if activeTab === 'metadata'}
			<!-- Metadata Tab -->
			<Card class="p-6 max-w-3xl">
				<h2 class="text-lg font-semibold mb-4">Layout Metadata</h2>
				<p class="text-sm text-muted-foreground mb-6">
					Edit the basic information about this layout. Changes are saved when you click the Save button.
				</p>

				<div class="space-y-6">
					<!-- Name Field -->
					<div>
						<label for="metadata-name" class="block text-sm font-medium mb-2">
							Name <span class="text-destructive">*</span>
						</label>
						<Input
							id="metadata-name"
							type="text"
							value={metadataName}
							oninput={(e) => updateMetadataField('name', e.currentTarget.value)}
							placeholder="My Awesome Layout"
							data-testid="metadata-name-input"
							class="w-full"
						/>
						{#if getFieldError('name')}
							<p class="text-sm text-destructive mt-1" data-testid="metadata-name-error">
								{getFieldError('name')}
							</p>
						{/if}
						<p class="text-xs text-muted-foreground mt-1">
							Maximum 100 characters. This appears in the layout list and exports.
						</p>
					</div>

					<!-- Description Field -->
					<div>
						<label for="metadata-description" class="block text-sm font-medium mb-2">
							Description
						</label>
						<textarea
							id="metadata-description"
							value={metadataDescription}
							oninput={(e) => updateMetadataField('description', e.currentTarget.value)}
							placeholder="A brief description of this layout..."
							rows={3}
							data-testid="metadata-description-input"
							class="w-full px-3 py-2 border border-border rounded-lg bg-background resize-y"
						></textarea>
						<p class="text-xs text-muted-foreground mt-1">
							Optional. Describe the layout's purpose, features, or design philosophy.
						</p>
					</div>

					<!-- Author Field -->
					<div>
						<label for="metadata-author" class="block text-sm font-medium mb-2">
							Author
						</label>
						<Input
							id="metadata-author"
							type="text"
							value={metadataAuthor}
							oninput={(e) => updateMetadataField('author', e.currentTarget.value)}
							placeholder="Your Name"
							data-testid="metadata-author-input"
							class="w-full"
						/>
						<p class="text-xs text-muted-foreground mt-1">
							Optional. Your name or username.
						</p>
					</div>

					<!-- Tags Field -->
					<div>
						<label for="metadata-tags" class="block text-sm font-medium mb-2">
							Tags
						</label>
						<Input
							id="metadata-tags"
							type="text"
							value={metadataTagsInput}
							oninput={(e) => updateMetadataField('tags', e.currentTarget.value)}
							placeholder="corne, 42-key, minimal"
							data-testid="metadata-tags-input"
							class="w-full font-mono text-sm"
						/>
						{#if getFieldError('tags')}
							<p class="text-sm text-destructive mt-1" data-testid="metadata-tags-error">
								{getFieldError('tags')}
							</p>
						{/if}
						<p class="text-xs text-muted-foreground mt-1">
							Comma-separated tags. Tags must be lowercase with hyphens and alphanumeric characters only (e.g., "42-key", "gaming").
						</p>
						{#if layout.metadata.tags && layout.metadata.tags.length > 0}
							<div class="flex flex-wrap gap-2 mt-2">
								{#each layout.metadata.tags as tag}
									<span class="inline-flex items-center px-2 py-1 rounded bg-primary/10 text-primary text-xs font-mono">
										{tag}
									</span>
								{/each}
							</div>
						{/if}
					</div>

					<!-- Read-only Metadata -->
					<div class="border-t border-border pt-4">
						<h3 class="text-sm font-semibold mb-3">System Information</h3>
						<dl class="grid grid-cols-2 gap-4 text-sm">
							<div>
								<dt class="font-medium text-muted-foreground">Keyboard</dt>
								<dd class="font-mono text-xs">{layout.metadata.keyboard || 'N/A'}</dd>
							</div>
							<div>
								<dt class="font-medium text-muted-foreground">Layout Variant</dt>
								<dd class="font-mono text-xs flex items-center gap-2">
									{layout.metadata.layout_variant || 'N/A'}
									{#if layout.metadata.keyboard}
										<Button size="sm" variant="outline" onclick={openVariantSwitch}>
											Switch
										</Button>
									{/if}
								</dd>
							</div>
							<div>
								<dt class="font-medium text-muted-foreground">Created</dt>
								<dd>{new Date(layout.metadata.created).toLocaleString()}</dd>
							</div>
							<div>
								<dt class="font-medium text-muted-foreground">Modified</dt>
								<dd>{new Date(layout.metadata.modified).toLocaleString()}</dd>
							</div>
							<div>
								<dt class="font-medium text-muted-foreground">Version</dt>
								<dd>{layout.metadata.version || '1.0'}</dd>
							</div>
							<div>
								<dt class="font-medium text-muted-foreground">Template</dt>
								<dd>{layout.metadata.is_template ? 'Yes' : 'No'}</dd>
							</div>
						</dl>
					</div>
				</div>
			</Card>
		{:else if activeTab === 'preview'}
			<!-- Preview Tab -->
			<div class="space-y-6">
				<!-- Metadata Card -->
				<Card class="p-6">
					<h2 class="text-lg font-semibold mb-3">Metadata</h2>
					<dl class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
						<div>
							<dt class="font-medium text-muted-foreground">Keyboard</dt>
							<dd>{layout.metadata.keyboard || 'N/A'}</dd>
						</div>
						<div>
							<dt class="font-medium text-muted-foreground">Layout</dt>
							<dd>{layout.metadata.layout_variant || layout.metadata.layout || 'N/A'}</dd>
						</div>
						<div>
							<dt class="font-medium text-muted-foreground">Author</dt>
							<dd>{layout.metadata.author || 'N/A'}</dd>
						</div>
						<div>
							<dt class="font-medium text-muted-foreground">Modified</dt>
							<dd>{new Date(layout.metadata.modified).toLocaleDateString()}</dd>
						</div>
					</dl>
				</Card>

				<!-- Layer Selector -->
				<Card class="p-6">
					<h2 class="text-lg font-semibold mb-3">Layers</h2>
					<div class="flex gap-2 flex-wrap">
						{#each layout.layers as layer, i}
							<button
								onclick={() => handleLayerChange(i)}
								class="flex items-center gap-2 px-3 py-1.5 rounded-lg border transition-colors text-sm
									{selectedLayerIndex === i
									? 'bg-primary text-primary-foreground border-primary'
									: 'bg-background hover:bg-accent border-border'}"
							>
								<span
									class="w-2.5 h-2.5 rounded-full"
									style="background-color: {layer.color || '#888'}"
								></span>
								<span class="font-medium">{layer.name}</span>
							</button>
						{/each}
					</div>
				</Card>

				<!-- Keyboard Preview -->
				<Card class="p-6">
					<div class="flex items-center justify-between mb-4">
						<h2 class="text-lg font-semibold">Keyboard Preview</h2>
						<div class="flex items-center gap-3">
							{#if selectedKeyIndices.size > 0}
								<span class="text-sm text-muted-foreground">
									{selectedKeyIndices.size} {selectedKeyIndices.size === 1 ? 'key' : 'keys'} selected
								</span>
							{/if}
							{#if selectedKey}
								<span class="text-sm text-muted-foreground">
									Selected: <code class="px-2 py-0.5 bg-muted rounded">{selectedKey.keycode}</code>
								</span>
							{/if}
						</div>
					</div>

					<!-- Selection and Clipboard Controls -->
					<div class="mb-4 flex items-center gap-2 flex-wrap">
						<Button
							onclick={toggleSelectionMode}
							size="sm"
							variant={selectionMode ? 'default' : 'outline'}
							data-testid="selection-mode-button"
						>
							{selectionMode ? '✓ Selection Mode' : 'Selection Mode'}
						</Button>
						<Button
							onclick={toggleSwapMode}
							size="sm"
							variant={swapMode ? 'default' : 'outline'}
							data-testid="swap-mode-button"
							title="Swap mode (Shift+W) - Click two keys to swap their properties"
						>
							{swapMode ? '✓ Swap Mode' : 'Swap Mode'}
						</Button>
						{#if selectionMode || selectedKeyIndices.size > 0}
							<Button
								onclick={clearSelection}
								size="sm"
								variant="outline"
								data-testid="clear-selection-button"
							>
								Clear Selection
							</Button>
						{/if}
						<div class="h-4 w-px bg-border mx-1"></div>
						<Button
							onclick={handleCopy}
							size="sm"
							variant="outline"
							disabled={selectedKeyIndices.size === 0}
							data-testid="copy-button"
							title="Copy selected keys (Ctrl+C)"
						>
							Copy
						</Button>
						<Button
							onclick={handleCut}
							size="sm"
							variant="outline"
							disabled={selectedKeyIndices.size === 0}
							data-testid="cut-button"
							title="Cut selected keys (Ctrl+X)"
						>
							Cut
						</Button>
						<Button
							onclick={handlePaste}
							size="sm"
							variant="outline"
							disabled={clipboardSize === 0}
							data-testid="paste-button"
							title="Paste clipboard (Ctrl+V)"
						>
							Paste {clipboardSize > 0 ? `(${clipboardSize})` : ''}
						</Button>
						<Button
							onclick={handleUndo}
							size="sm"
							variant="outline"
							disabled={!canUndo}
							data-testid="undo-button"
							title="Undo last operation (Ctrl+Z)"
						>
							Undo
						</Button>
					</div>

					{#if geometryLoading}
						<div class="flex items-center justify-center h-40 text-muted-foreground">
							Loading keyboard geometry...
						</div>
					{:else if geometryError}
						<div class="flex flex-col items-center justify-center h-40 text-destructive">
							<p class="mb-2">Failed to load keyboard geometry</p>
							<p class="text-sm text-muted-foreground">{geometryError}</p>
						</div>
					{:else if geometry}
						<KeyboardPreview
							geometry={geometry.keys}
							keyAssignments={currentLayerKeys}
							{selectedKeyIndex}
							{selectedKeyIndices}
							{swapMode}
							{swapFirstKey}
							layer={layout.layers[selectedLayerIndex]}
							categories={layout.categories || []}
							renderMetadata={currentLayerRenderMetadata}
							onKeyClick={handleKeyClick}
							onNavigate={handleKeyboardNavigation}
							onKeyHover={handleKeyHover}
							positionToVisualIndexMap={geometry.position_to_visual_index}
							class="max-w-4xl mx-auto"
						/>
					{:else}
						<div class="flex items-center justify-center h-40 text-muted-foreground">
							No geometry data available.
						</div>
					{/if}
				</Card>

			<!-- Key Details Card - Fixed height to prevent scrollbar jumping -->
			<Card class="p-6" style="min-height: 400px;" data-testid="key-details-card">
				<!-- Heading is always visible (LazyQMK-mpm: no flicker) -->
				<h2 class="text-lg font-semibold mb-4" data-testid="key-details-heading">
					Key Metadata
				</h2>
				
				{#if selectedKeyIndices.size > 1 && hoveredKeyIndex === null}
					<!-- Multi-selection summary -->
					<div class="mb-4 p-4 bg-muted/30 rounded-lg" data-testid="multi-selection-summary">
						<p class="font-medium text-sm">Multiple Keys Selected ({selectedKeyIndices.size} keys)</p>
						<p class="text-xs text-muted-foreground mt-1">
							Use Copy, Cut, or Paste operations to modify the selection
						</p>
					</div>
				{:else if hoveredKeyIndex !== null && hoveredKey === null}
					<!-- Hover with missing key data fallback -->
					<div class="p-4 bg-muted/30 rounded-lg" data-testid="key-hover-fallback">
						<p class="text-sm text-muted-foreground">
							Hovering key index: <span class="font-mono">{hoveredKeyIndex}</span>
						</p>
						<p class="text-xs text-muted-foreground mt-1">
							Key data not available for this position.
						</p>
					</div>
				{:else if activeKey}
					<!-- Key Legend Display (LazyQMK-t47: show primary/secondary/tertiary) -->
					{#if activeKeyRenderMetadata?.display}
						<div class="mb-4 p-4 bg-muted/30 rounded-lg" data-testid="key-legend-display">
							<div class="flex items-center gap-4">
								<!-- Primary label (large) -->
								<div class="flex flex-col items-center justify-center min-w-[60px] h-[60px] bg-background border border-border rounded-lg shadow-sm" data-testid="key-legend-primary">
									<span class="text-2xl font-bold">{activeKeyRenderMetadata.display.primary}</span>
								</div>
								<!-- Secondary/Tertiary labels (if present) -->
								{#if activeKeyRenderMetadata.display.secondary || activeKeyRenderMetadata.display.tertiary}
									<div class="flex flex-col gap-1">
										{#if activeKeyRenderMetadata.display.secondary}
											<div class="flex items-center gap-2" data-testid="key-legend-secondary">
												<span class="text-xs font-medium text-muted-foreground uppercase">Hold:</span>
												<span class="text-sm font-medium px-2 py-0.5 bg-primary/10 text-primary rounded">{activeKeyRenderMetadata.display.secondary}</span>
											</div>
										{/if}
										{#if activeKeyRenderMetadata.display.tertiary}
											<div class="flex items-center gap-2" data-testid="key-legend-tertiary">
												<span class="text-xs font-medium text-muted-foreground uppercase">Double:</span>
												<span class="text-sm font-medium px-2 py-0.5 bg-secondary/10 text-secondary-foreground rounded">{activeKeyRenderMetadata.display.tertiary}</span>
											</div>
										{/if}
									</div>
								{/if}
							</div>
						</div>
					{/if}
					
					<!-- Single key details -->
					<dl class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm mb-4">
						<div>
							<dt class="font-medium text-muted-foreground">Visual Index</dt>
							<dd class="font-mono">{activeKey.visual_index}</dd>
						</div>
						<div>
							<dt class="font-medium text-muted-foreground">Matrix Position</dt>
							<dd class="font-mono">
								[{activeKey.matrix_position[0]}, {activeKey.matrix_position[1]}]
							</dd>
						</div>
						<div>
							<dt class="font-medium text-muted-foreground">LED Index</dt>
							<dd class="font-mono">{activeKey.led_index ?? 'N/A'}</dd>
						</div>
						<div>
							<dt class="font-medium text-muted-foreground">Keycode</dt>
							<dd class="font-mono">{activeKey.keycode}</dd>
						</div>
					</dl>

					{#if activeKeyRenderMetadata && activeKeyRenderMetadata.details.length > 0}
						<!-- Rich key action breakdown (LazyQMK-t47: full keycode DB descriptions) -->
						<div class="border-t border-border pt-4 mb-4">
							<h3 class="font-medium text-sm mb-3">Key Actions</h3>
							<div class="space-y-2">
								{#each activeKeyRenderMetadata.details as action}
									<div class="flex items-start gap-3 text-sm">
										<span class="inline-flex items-center px-2 py-0.5 rounded-md bg-primary/10 text-primary text-xs font-medium uppercase min-w-[80px] justify-center">
											{action.kind.replace('_', ' ')}
										</span>
										<div class="flex-1">
											<code class="text-xs bg-muted px-1.5 py-0.5 rounded">{action.code}</code>
											<p class="text-muted-foreground mt-1">{action.description}</p>
										</div>
									</div>
								{/each}
							</div>
						</div>
					{/if}

					{#if hoveredKeyIndex === null && selectedKey}
						<!-- Customization controls (only shown when not hovering, works in selection mode - LazyQMK-yij) -->
						<div class="border-t border-border pt-4 space-y-4">
							<h3 class="font-medium text-sm">Key Customization</h3>

							<!-- Edit Keycode Button -->
							<div>
								<p class="block text-xs font-medium text-muted-foreground mb-2">Keycode</p>
								<Button onclick={openKeycodePicker} size="sm">Edit Keycode</Button>
							</div>

							<!-- Key Color Override -->
							<div>
								<p class="block text-xs font-medium text-muted-foreground mb-2">Color Override</p>
								{#if selectedKey.color_override}
									<div class="flex items-center gap-2">
										<div
											class="w-8 h-8 rounded border border-border"
											style="background-color: rgb({selectedKey.color_override.r}, {selectedKey.color_override
												.g}, {selectedKey.color_override.b})"
										></div>
										<Button onclick={() => (showKeyColorPicker = !showKeyColorPicker)} size="sm" variant="outline">
											Change
										</Button>
										<Button onclick={clearKeyColorOverride} size="sm" variant="outline" data-testid="clear-color-override-button">Clear</Button>
									</div>
								{:else}
									<Button onclick={() => (showKeyColorPicker = !showKeyColorPicker)} size="sm" data-testid="set-color-button">Set Color</Button>
								{/if}
								{#if showKeyColorPicker}
									<div class="mt-3 p-4 border border-border rounded-lg">
										<ColorPicker
											color={selectedKey.color_override}
											onSelect={setKeyColorOverride}
											onClear={clearKeyColorOverride}
											label="Key Color Override"
											showClear={!!selectedKey.color_override}
										/>
									</div>
								{/if}
							</div>

							<!-- Key Category -->
							<div>
								<label for="key-category" class="block text-xs font-medium text-muted-foreground mb-2">Category</label>
								<select
									id="key-category"
									class="w-full px-3 py-2 border border-border rounded-lg bg-background"
									value={selectedKey.category_id || ''}
									onchange={(e) => setKeyCategory(e.currentTarget.value || undefined)}
								>
									<option value="">None</option>
									{#if layout.categories}
										{#each layout.categories as category}
											<option value={category.id}>{category.name}</option>
										{/each}
									{/if}
								</select>
								<p class="text-xs text-muted-foreground mt-1">
									Assign this key to a category for automatic coloring
								</p>
							</div>

							<!-- Key Description (LazyQMK-yij: editable in selection mode) -->
							<div>
								<label for="key-description" class="block text-xs font-medium text-muted-foreground mb-2">Description</label>
								<textarea
									id="key-description"
									class="w-full px-3 py-2 border border-border rounded-lg bg-background resize-none text-sm"
									rows="2"
									placeholder="Add a note about this key..."
									value={selectedKey.description || ''}
									onchange={(e) => setKeyDescription(e.currentTarget.value)}
									data-testid="key-description-input"
								></textarea>
								<p class="text-xs text-muted-foreground mt-1">
									Optional note or reminder for this key binding
								</p>
							</div>
						</div>
					{/if}
				{:else}
					<!-- Empty state when no key is selected or hovered -->
					<div class="p-4 text-center text-muted-foreground" data-testid="key-details-empty">
						<p class="text-sm">Select or hover over a key to view details</p>
					</div>
				{/if}
			</Card>
			</div>
		{:else if activeTab === 'layers'}
			<!-- Layers Tab -->
			<LayerManager
				layers={layout.layers}
				{selectedLayerIndex}
				onLayersChange={handleLayersChange}
				onLayerSelect={handleLayerChange}
			/>
		{:else if activeTab === 'categories'}
			<!-- Categories Tab -->
			<CategoryManager
				categories={layout.categories || []}
				onChange={handleCategoriesChange}
			/>
		{:else if activeTab === 'tap-dance'}
			<!-- Tap Dance Tab -->
			<Card class="p-6">
				<div class="flex items-center justify-between mb-4">
					<h2 class="text-lg font-semibold">Tap Dance Actions</h2>
					<Button onclick={addTapDance} size="sm">Add Tap Dance</Button>
				</div>

				{#if !layout.tap_dances?.length}
					<p class="text-muted-foreground text-sm">
						No tap dances defined. Click "Add Tap Dance" to create one.
					</p>
				{:else}
					<div class="space-y-4">
						{#each layout.tap_dances as td, i}
							<div class="border border-border rounded-lg p-4 space-y-3">
								<div class="flex items-center justify-between">
									<span class="font-mono text-sm font-medium">TD({td.name})</span>
									<Button onclick={() => removeTapDance(i)} size="sm" variant="destructive">
										Remove
									</Button>
								</div>
								<div class="grid grid-cols-2 md:grid-cols-4 gap-3">
									<div>
										<label for="td-name-{i}" class="block text-xs font-medium text-muted-foreground mb-1">Name</label>
										<Input
											id="td-name-{i}"
											value={td.name}
											oninput={(e) => updateTapDance(i, 'name', e.currentTarget.value)}
											placeholder="TD_NAME"
											class="font-mono text-sm"
										/>
									</div>
									<div>
										<label for="td-single-{i}" class="block text-xs font-medium text-muted-foreground mb-1"
											>Single Tap</label
										>
										<Input
											id="td-single-{i}"
											value={td.single_tap || td.tap || ''}
											oninput={(e) => updateTapDance(i, 'single_tap', e.currentTarget.value)}
											placeholder="KC_A"
											class="font-mono text-sm"
										/>
									</div>
									<div>
										<label for="td-double-{i}" class="block text-xs font-medium text-muted-foreground mb-1"
											>Double Tap</label
										>
										<Input
											id="td-double-{i}"
											value={td.double_tap || ''}
											oninput={(e) => updateTapDance(i, 'double_tap', e.currentTarget.value)}
											placeholder="KC_B"
											class="font-mono text-sm"
										/>
									</div>
									<div>
										<label for="td-hold-{i}" class="block text-xs font-medium text-muted-foreground mb-1">Hold</label>
										<Input
											id="td-hold-{i}"
											value={td.hold || ''}
											oninput={(e) => updateTapDance(i, 'hold', e.currentTarget.value)}
											placeholder="KC_LCTL"
											class="font-mono text-sm"
										/>
									</div>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</Card>
		{:else if activeTab === 'combos'}
			<!-- Combos Tab -->
			<Card class="p-6">
				<div class="flex items-center justify-between mb-4">
					<h2 class="text-lg font-semibold">Combos</h2>
					<Button onclick={addCombo} size="sm">Add Combo</Button>
				</div>

				{#if !layout.combos?.length}
					<p class="text-muted-foreground text-sm">
						No combos defined. Combos trigger a keycode when multiple keys are pressed
						simultaneously.
					</p>
				{:else}
					<div class="space-y-4">
						{#each layout.combos as combo, i}
							<div class="border border-border rounded-lg p-4 space-y-3">
								<div class="flex items-center justify-between">
									<span class="font-medium">{combo.name}</span>
									<Button onclick={() => removeCombo(i)} size="sm" variant="destructive">
										Remove
									</Button>
								</div>
								<div class="grid grid-cols-3 gap-3">
									<div>
										<label for="combo-name-{i}" class="block text-xs font-medium text-muted-foreground mb-1">Name</label>
										<Input
											id="combo-name-{i}"
											value={combo.name}
											oninput={(e) => updateCombo(i, 'name', e.currentTarget.value)}
											placeholder="Combo Name"
										/>
									</div>
									<div>
										<label for="combo-keys-{i}" class="block text-xs font-medium text-muted-foreground mb-1"
											>Trigger Keys (comma-separated)</label
										>
										<Input
											id="combo-keys-{i}"
											value={combo.keys.join(', ')}
											oninput={(e) => updateCombo(i, 'keys', e.currentTarget.value)}
											placeholder="KC_A, KC_B"
											class="font-mono text-sm"
										/>
									</div>
									<div>
										<label for="combo-output-{i}" class="block text-xs font-medium text-muted-foreground mb-1">Output</label
										>
										<Input
											id="combo-output-{i}"
											value={combo.output}
											oninput={(e) => updateCombo(i, 'output', e.currentTarget.value)}
											placeholder="KC_ESC"
											class="font-mono text-sm"
										/>
									</div>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</Card>
		{:else if activeTab === 'idle-effect'}
			<!-- Idle Effect Tab -->
			<Card class="p-6">
				<h2 class="text-lg font-semibold mb-4">Idle Effect Settings</h2>
				<p class="text-muted-foreground text-sm mb-6">
					Configure the RGB effect that plays when the keyboard is idle.
				</p>

				<div class="space-y-4 max-w-md">
					<div class="flex items-center gap-3">
						<input
							type="checkbox"
							id="idle-enabled"
							checked={layout.idle_effect_settings?.enabled ?? true}
							onchange={(e) => updateIdleEffect('enabled', e.currentTarget.checked)}
							class="w-4 h-4"
						/>
						<label for="idle-enabled" class="text-sm font-medium">Enable Idle Effect</label>
					</div>

					<div>
						<label for="idle-timeout" class="block text-sm font-medium text-muted-foreground mb-1"
							>Idle Timeout (seconds)</label
						>
						<Input
							id="idle-timeout"
							type="number"
							value={Math.round((layout.idle_effect_settings?.idle_timeout_ms ?? 60000) / 1000)}
							oninput={(e) =>
								updateIdleEffect('idle_timeout_ms', parseInt(e.currentTarget.value) * 1000)}
							min="10"
							max="600"
						/>
						<p class="text-xs text-muted-foreground mt-1">
							Time before idle effect starts (10-600 seconds)
						</p>
					</div>

					<div>
						<label for="idle-duration" class="block text-sm font-medium text-muted-foreground mb-1"
							>Effect Duration (seconds)</label
						>
						<Input
							id="idle-duration"
							type="number"
							value={Math.round(
								(layout.idle_effect_settings?.idle_effect_duration_ms ?? 300000) / 1000
							)}
							oninput={(e) =>
								updateIdleEffect('idle_effect_duration_ms', parseInt(e.currentTarget.value) * 1000)}
							min="30"
							max="3600"
						/>
						<p class="text-xs text-muted-foreground mt-1">
							How long the effect runs before RGB turns off (30-3600 seconds)
						</p>
					</div>

					<div>
						<label for="idle-effect-mode" class="block text-sm font-medium text-muted-foreground mb-1">Effect Mode</label>
						<select
							id="idle-effect-mode"
							class="w-full px-3 py-2 border border-border rounded-lg bg-background"
							value={layout.idle_effect_settings?.idle_effect_mode ?? 'Breathing'}
							onchange={(e) => updateIdleEffect('idle_effect_mode', e.currentTarget.value)}
						>
							<option value="Solid Color">Solid Color</option>
							<option value="Breathing">Breathing</option>
							<option value="Rainbow Moving Chevron">Rainbow Moving Chevron</option>
							<option value="Cycle All">Cycle All</option>
							<option value="Cycle Left/Right">Cycle Left/Right</option>
							<option value="Cycle Up/Down">Cycle Up/Down</option>
							<option value="Rainbow Beacon">Rainbow Beacon</option>
							<option value="Rainbow Pinwheels">Rainbow Pinwheels</option>
							<option value="Jellybean Raindrops">Jellybean Raindrops</option>
						</select>
					</div>
				</div>
			</Card>
		{:else if activeTab === 'validate'}
			<!-- Validate Tab -->
			<Card class="p-6">
				<div class="flex items-center justify-between mb-4">
					<h2 class="text-lg font-semibold">Validate Layout</h2>
					<Button onclick={runValidation} disabled={validationLoading}>
						{validationLoading ? 'Validating...' : 'Run Validation'}
					</Button>
				</div>

				{#if validationResult}
					<div
						class="p-4 rounded-lg {validationResult.valid
							? 'bg-green-500/10 border border-green-500/30'
							: 'bg-red-500/10 border border-red-500/30'}"
					>
						<div class="flex items-center gap-2 mb-2">
							<span class="text-lg">{validationResult.valid ? '✓' : '✗'}</span>
							<span class="font-medium">
								{validationResult.valid ? 'Layout is valid' : 'Layout has errors'}
							</span>
						</div>

						{#if validationResult.error}
							<p class="text-red-500 text-sm">{validationResult.error}</p>
						{/if}

						{#if validationResult.warnings.length > 0}
							<div class="mt-3">
								<p class="text-sm font-medium text-yellow-500 mb-1">Warnings:</p>
								<ul class="list-disc list-inside text-sm text-muted-foreground">
									{#each validationResult.warnings as warning}
										<li>{warning}</li>
									{/each}
								</ul>
							</div>
						{/if}
					</div>
				{:else}
					<p class="text-muted-foreground text-sm">
						Click "Run Validation" to check your layout for errors.
					</p>
				{/if}
			</Card>
		{:else if activeTab === 'inspect'}
			<!-- Inspect Tab -->
			<Card class="p-6">
				<div class="flex items-center justify-between mb-4">
					<h2 class="text-lg font-semibold">Inspect Layout</h2>
					<Button onclick={runInspect} disabled={inspectLoading}>
						{inspectLoading ? 'Loading...' : 'Refresh'}
					</Button>
				</div>

				{#if inspectResult}
					<div class="space-y-6">
						<!-- Metadata -->
						<div>
							<h3 class="font-medium mb-2">Metadata</h3>
							<dl class="grid grid-cols-2 md:grid-cols-3 gap-3 text-sm">
								<div>
									<dt class="text-muted-foreground">Layers</dt>
									<dd class="font-mono">{inspectResult.metadata.layer_count}</dd>
								</div>
								<div>
									<dt class="text-muted-foreground">Keys per layer</dt>
									<dd class="font-mono">{inspectResult.metadata.key_count}</dd>
								</div>
								<div>
									<dt class="text-muted-foreground">Categories</dt>
									<dd class="font-mono">{inspectResult.metadata.category_count}</dd>
								</div>
								<div>
									<dt class="text-muted-foreground">Tap Dances</dt>
									<dd class="font-mono">{inspectResult.metadata.tap_dance_count}</dd>
								</div>
								<div>
									<dt class="text-muted-foreground">Keyboard</dt>
									<dd class="font-mono">{inspectResult.metadata.keyboard || 'N/A'}</dd>
								</div>
								<div>
									<dt class="text-muted-foreground">Author</dt>
									<dd>{inspectResult.metadata.author || 'N/A'}</dd>
								</div>
							</dl>
						</div>

						<!-- Layers -->
						<div>
							<h3 class="font-medium mb-2">Layers</h3>
							<div class="overflow-x-auto">
								<table class="w-full text-sm">
									<thead>
										<tr class="border-b border-border">
											<th class="text-left py-2 px-2">#</th>
											<th class="text-left py-2 px-2">Name</th>
											<th class="text-left py-2 px-2">Keys</th>
											<th class="text-left py-2 px-2">Color</th>
											<th class="text-left py-2 px-2">Enabled</th>
										</tr>
									</thead>
									<tbody>
										{#each inspectResult.layers as layer}
											<tr class="border-b border-border/50">
												<td class="py-2 px-2 font-mono">{layer.number}</td>
												<td class="py-2 px-2">{layer.name}</td>
												<td class="py-2 px-2 font-mono">{layer.key_count}</td>
												<td class="py-2 px-2">
													<span
														class="inline-block w-4 h-4 rounded"
														style="background-color: {layer.default_color}"
													></span>
												</td>
												<td class="py-2 px-2">{layer.colors_enabled ? '✓' : '✗'}</td>
											</tr>
										{/each}
									</tbody>
								</table>
							</div>
						</div>

						<!-- Settings -->
						<div>
							<h3 class="font-medium mb-2">Settings</h3>
							<dl class="grid grid-cols-2 md:grid-cols-4 gap-3 text-sm">
								<div>
									<dt class="text-muted-foreground">RGB Enabled</dt>
									<dd>{inspectResult.settings.rgb_enabled ? 'Yes' : 'No'}</dd>
								</div>
								<div>
									<dt class="text-muted-foreground">Brightness</dt>
									<dd>{inspectResult.settings.rgb_brightness}%</dd>
								</div>
								<div>
									<dt class="text-muted-foreground">Idle Effect</dt>
									<dd>{inspectResult.settings.idle_effect_mode}</dd>
								</div>
								<div>
									<dt class="text-muted-foreground">Tap-Hold Preset</dt>
									<dd>{inspectResult.settings.tap_hold_preset}</dd>
								</div>
							</dl>
						</div>
					</div>
				{:else}
					<p class="text-muted-foreground text-sm">
						Click "Refresh" to load detailed layout information.
					</p>
				{/if}
			</Card>
		{:else if activeTab === 'export'}
			<!-- Export Tab -->
			<Card class="p-6">
				<div class="flex items-center justify-between mb-4">
					<h2 class="text-lg font-semibold">Export to Markdown</h2>
					<div class="flex gap-2">
						<Button onclick={runExport} disabled={exportLoading}>
							{exportLoading ? 'Exporting...' : 'Generate Export'}
						</Button>
						{#if exportResult}
							<Button onclick={downloadExport}>Download</Button>
						{/if}
					</div>
				</div>

				{#if exportResult}
					<div class="space-y-4">
						<p class="text-sm text-muted-foreground">
							Suggested filename: <code class="bg-muted px-2 py-0.5 rounded"
								>{exportResult.suggested_filename}</code
							>
						</p>
						<div class="border border-border rounded-lg overflow-hidden">
							<pre
								class="p-4 text-sm overflow-x-auto max-h-96 bg-muted/30">{exportResult.markdown}</pre>
						</div>
					</div>
				{:else}
					<p class="text-muted-foreground text-sm">
						Generate a markdown export with keyboard diagrams and configuration summary.
					</p>
				{/if}
			</Card>
		{:else if activeTab === 'generate'}
			<!-- Generate Tab -->
			<Card class="p-6">
				<div class="flex items-center justify-between mb-4">
					<h2 class="text-lg font-semibold">Generate Firmware</h2>
					<div class="flex gap-2">
						{#if generateJob && (generateJob.status === 'pending' || generateJob.status === 'running')}
							<Button 
								onclick={cancelGenerateJob} 
								disabled={generateCancelling}
								variant="destructive"
								data-testid="cancel-generate-button"
							>
								{generateCancelling ? 'Cancelling...' : 'Cancel'}
							</Button>
						{/if}
						<Button 
							onclick={runGenerate} 
							disabled={generateLoading || generatePollingActive}
							data-testid="generate-button"
						>
							{generateLoading || generatePollingActive ? 'Generating...' : 'Generate Firmware'}
						</Button>
					</div>
				</div>

				<!-- Timeout Warning -->
				{#if generateTimeoutReached}
					<div class="mb-4 p-4 rounded-lg bg-yellow-500/10 border border-yellow-500/30" data-testid="generate-timeout-warning">
						<p class="font-medium text-yellow-700 dark:text-yellow-300">Generation Timeout</p>
						<p class="text-sm text-muted-foreground">
							The generation process has exceeded the 5 minute timeout. The job may still be running in the background.
							Check back later or try generating again.
						</p>
					</div>
				{/if}

				<!-- Job Status -->
				{#if generateJob}
					<div class="mb-4">
						<!-- Status Badge -->
						<div class="flex items-center gap-3 mb-3">
							<span class="text-sm font-medium text-muted-foreground">Status:</span>
							{#if generateJob.status === 'pending'}
								<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200" data-testid="status-pending">
									⏳ Pending
								</span>
							{:else if generateJob.status === 'running'}
								<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200" data-testid="status-running">
									🔄 Running
								</span>
							{:else if generateJob.status === 'completed'}
								<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200" data-testid="status-completed">
									✅ Completed
								</span>
							{:else if generateJob.status === 'failed'}
								<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200" data-testid="status-failed">
									❌ Failed
								</span>
							{:else if generateJob.status === 'cancelled'}
								<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200" data-testid="status-cancelled">
									🚫 Cancelled
								</span>
							{/if}
						</div>

						<!-- Progress Bar -->
						{#if generateJob.status === 'running' || generateJob.status === 'pending'}
							<div class="mb-3">
								<div class="flex justify-between text-xs text-muted-foreground mb-1">
									<span>Progress</span>
									<span>{generateJob.progress}%</span>
								</div>
								<div class="w-full bg-muted rounded-full h-2">
									<div 
										class="bg-primary h-2 rounded-full transition-all duration-300" 
										style="width: {generateJob.progress}%"
										data-testid="progress-bar"
									></div>
								</div>
							</div>
						{/if}

						<!-- Error Message -->
						{#if generateJob.status === 'failed' && generateJob.error}
							<div class="mb-3 p-3 rounded-lg bg-red-500/10 border border-red-500/30" data-testid="generate-error">
								<p class="text-sm font-medium text-red-700 dark:text-red-300">Error:</p>
								<p class="text-sm text-red-600 dark:text-red-400">{generateJob.error}</p>
							</div>
						{/if}

						<!-- Download Button -->
						{#if generateJob.status === 'completed' && generateJob.download_url}
							<div class="mb-4 p-4 rounded-lg bg-green-500/10 border border-green-500/30">
								<div class="flex items-center justify-between">
									<div>
										<p class="font-medium text-green-700 dark:text-green-300">Generation Complete!</p>
										<p class="text-sm text-muted-foreground">Your firmware files are ready to download.</p>
									</div>
									<a 
										href={apiClient.getGenerateDownloadUrl(generateJob.id)}
										download
										class="inline-flex items-center px-4 py-2 rounded-lg bg-green-600 text-white hover:bg-green-700 transition-colors"
										data-testid="download-button"
									>
										📦 Download ZIP
									</a>
								</div>
							</div>
						{/if}

						<!-- Logs Section -->
						{#if generateLogs.length > 0}
							<div class="mt-4">
								<h3 class="text-sm font-medium mb-2">Generation Logs</h3>
								<div 
									class="bg-muted/50 rounded-lg p-3 max-h-64 overflow-y-auto font-mono text-xs"
									data-testid="generate-logs"
								>
									{#each generateLogs as log}
										<div class="flex gap-2 py-0.5 {log.level === 'ERROR' ? 'text-red-500' : log.level === 'WARN' ? 'text-yellow-500' : 'text-muted-foreground'}">
											<span class="text-muted-foreground/60 shrink-0">
												{new Date(log.timestamp).toLocaleTimeString()}
											</span>
											<span class="font-semibold shrink-0 w-12">[{log.level}]</span>
											<span class="break-all">{log.message}</span>
										</div>
									{/each}
								</div>
							</div>
						{/if}
					</div>
				{:else if !generateResult}
					<!-- Initial state - no job yet -->
					<div class="space-y-4">
						<p class="text-muted-foreground text-sm">
							Generate QMK firmware files for this layout. This will create a zip file containing
							keymap.c, config.h, and other required files.
						</p>
						<div class="bg-muted/30 p-4 rounded-lg">
							<p class="text-sm font-medium mb-2">CLI Alternative:</p>
							<code class="text-sm font-mono">lazyqmk generate {filename}</code>
		</div>
	</div>
{/if}

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
			</Card>
		{:else if activeTab === 'build'}
			<!-- Build Tab - Firmware Compilation -->
			<div class="space-y-6">
				<!-- Build Controls Card -->
				<Card class="p-6">
					<div class="flex items-center justify-between mb-4">
						<div>
							<h2 class="text-lg font-semibold">Build Firmware</h2>
							<p class="text-sm text-muted-foreground">
								Compile QMK firmware (.uf2/.hex/.bin) for this layout
							</p>
						</div>
						<div class="flex gap-2">
							{#if buildJob && (buildJob.status === 'pending' || buildJob.status === 'running')}
								<Button 
									onclick={cancelBuildJob} 
									disabled={buildCancelling}
									variant="destructive"
									data-testid="cancel-build-button"
								>
									{buildCancelling ? 'Cancelling...' : 'Cancel Build'}
								</Button>
							{/if}
							<Button 
								onclick={startBuild} 
								disabled={buildLoading || buildPollingActive || isDirty}
								data-testid="start-build-button"
							>
								{buildLoading || buildPollingActive ? 'Building...' : 'Start Build'}
							</Button>
						</div>
					</div>

					<!-- Save Warning -->
					{#if isDirty}
						<div class="mb-4 p-4 rounded-lg bg-yellow-500/10 border border-yellow-500/30" data-testid="build-save-warning">
							<p class="font-medium text-yellow-700 dark:text-yellow-300">Unsaved Changes</p>
							<p class="text-sm text-muted-foreground">
								Please save your layout before starting a build. Click the "Save" button in the header.
							</p>
						</div>
					{/if}

					<!-- Active Build Status -->
					{#if buildJob}
						{@const badge = getBuildStatusBadge(buildJob.status)}
						<div class="mb-4">
							<!-- Status Badge -->
							<div class="flex items-center gap-3 mb-3">
								<span class="text-sm font-medium text-muted-foreground">Status:</span>
								<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {badge.class}" data-testid="build-status">
									{badge.icon} {badge.text}
								</span>
							</div>

							<!-- Progress Bar -->
							{#if buildJob.status === 'running' || buildJob.status === 'pending'}
								<div class="mb-3">
									<div class="flex justify-between text-xs text-muted-foreground mb-1">
										<span>Progress</span>
										<span>{buildJob.progress}%</span>
									</div>
									<div class="w-full bg-muted rounded-full h-2">
										<div 
											class="bg-primary h-2 rounded-full transition-all duration-300" 
											style="width: {buildJob.progress}%"
											data-testid="build-progress-bar"
										></div>
									</div>
								</div>
							{/if}

							<!-- Error Message -->
							{#if buildJob.status === 'failed' && buildJob.error}
								<div class="mb-3 p-3 rounded-lg bg-red-500/10 border border-red-500/30" data-testid="build-error">
									<p class="text-sm font-medium text-red-700 dark:text-red-300">Build Error:</p>
									<p class="text-sm text-red-600 dark:text-red-400">{buildJob.error}</p>
								</div>
							{/if}
						</div>
					{:else}
						<!-- Initial state - no active build -->
						<div class="text-center py-8 text-muted-foreground" data-testid="build-empty-state">
							<p class="text-sm">Start a build to compile firmware for this layout.</p>
							<p class="text-xs mt-2">
								CLI Alternative: <code class="bg-muted px-2 py-0.5 rounded font-mono">lazyqmk build {filename}</code>
							</p>
						</div>
					{/if}
				</Card>

				<!-- Build Artifacts Card -->
				{#if buildArtifacts.length > 0}
					<Card class="p-6" data-testid="build-artifacts-card">
						<h3 class="text-lg font-semibold mb-4">Build Artifacts</h3>
						<p class="text-sm text-muted-foreground mb-4">
							Download compiled firmware files for flashing to your keyboard.
						</p>
						<div class="space-y-3">
							{#each buildArtifacts as artifact}
								<div class="flex items-center justify-between p-3 border border-border rounded-lg hover:bg-muted/30 transition-colors" data-testid="artifact-row">
									<div class="flex-1">
										<p class="font-medium text-sm font-mono">{artifact.filename}</p>
										<div class="flex gap-4 text-xs text-muted-foreground mt-1">
											<span>Type: {artifact.artifact_type}</span>
											<span>Size: {formatBytes(artifact.size)}</span>
											{#if artifact.sha256}
												<span title={artifact.sha256}>Hash: {artifact.sha256.substring(0, 8)}...</span>
											{/if}
										</div>
									</div>
									<a 
										href={buildJob ? apiClient.getBuildArtifactDownloadUrl(buildJob.id, artifact.id) : '#'}
										download={artifact.filename}
										class="inline-flex items-center px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors text-sm"
										data-testid="artifact-download"
									>
										📥 Download
									</a>
								</div>
							{/each}
						</div>
					</Card>
				{/if}

				<!-- Build Logs Card -->
				{#if buildLogs.length > 0 || buildJob}
					<Card class="p-6" data-testid="build-logs-card">
						<div class="flex items-center justify-between mb-4">
							<h3 class="text-lg font-semibold">Build Logs</h3>
							<div class="flex items-center gap-3">
								<label class="flex items-center gap-2 text-sm text-muted-foreground cursor-pointer">
									<input 
										type="checkbox" 
										bind:checked={buildAutoScroll}
										class="w-4 h-4 rounded"
									/>
									Auto-scroll
								</label>
								<Button 
									size="sm" 
									variant="outline" 
									onclick={copyBuildLogs}
									disabled={buildLogs.length === 0}
									data-testid="copy-logs-button"
								>
									📋 Copy Logs
								</Button>
							</div>
						</div>
						<div 
							bind:this={buildLogsElement}
							class="bg-gray-900 text-gray-100 p-4 rounded-lg font-mono text-sm h-80 overflow-y-auto"
							data-testid="build-logs"
						>
							{#if buildLogs.length === 0}
								<p class="text-gray-500">Waiting for logs...</p>
							{:else}
								{#each buildLogs as log}
									<div class="flex gap-2 py-0.5">
										<span class="text-gray-500 shrink-0 w-20">
											{new Date(log.timestamp).toLocaleTimeString()}
										</span>
										<span class="shrink-0 w-14 {getLogLevelColor(log.level)}">[{log.level}]</span>
										<span class="break-all">{log.message}</span>
									</div>
								{/each}
							{/if}
						</div>
					</Card>
				{/if}

				<!-- Build History Card -->
				<Card class="p-6" data-testid="build-history-card">
					<div class="flex items-center justify-between mb-4">
						<h3 class="text-lg font-semibold">Build History</h3>
						<Button 
							size="sm" 
							variant="outline" 
							onclick={loadBuildHistory}
							data-testid="refresh-history-button"
						>
							🔄 Refresh
						</Button>
					</div>
					{#if buildHistory.length === 0}
						<p class="text-sm text-muted-foreground text-center py-4">
							No previous builds for this layout. Start your first build above.
						</p>
					{:else}
						<div class="overflow-x-auto">
							<table class="w-full text-sm">
								<thead>
									<tr class="border-b border-border">
										<th class="text-left py-2 px-2">Status</th>
										<th class="text-left py-2 px-2">Created</th>
										<th class="text-left py-2 px-2">Keyboard</th>
										<th class="text-left py-2 px-2">Progress</th>
										<th class="text-right py-2 px-2">Actions</th>
									</tr>
								</thead>
								<tbody>
									{#each buildHistory as job}
										{@const badge = getBuildStatusBadge(job.status)}
										<tr class="border-b border-border/50 hover:bg-muted/30 transition-colors" data-testid="history-row">
											<td class="py-2 px-2">
												<span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {badge.class}">
													{badge.icon} {badge.text}
												</span>
											</td>
											<td class="py-2 px-2 text-muted-foreground">
												{new Date(job.created_at).toLocaleString(undefined, { 
													month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' 
												})}
											</td>
											<td class="py-2 px-2 font-mono text-xs">{job.keyboard}</td>
											<td class="py-2 px-2">{job.progress}%</td>
											<td class="py-2 px-2 text-right">
												<Button 
													size="sm" 
													variant="ghost" 
													onclick={() => selectBuildJob(job)}
													data-testid="view-job-button"
												>
													View
												</Button>
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					{/if}
				</Card>
			</div>
		{/if}
	{/if}
</div>

<!-- Keycode Picker Modal -->
<KeycodePicker
	bind:open={keycodePickerOpen}
	onClose={handleKeycodePickerClose}
	onSelect={handleKeycodeSelect}
	currentKeycode={editingKeyVisualIndex !== null
		? currentLayerKeys.find((k) => k.visual_index === editingKeyVisualIndex)?.keycode
		: undefined}
/>

<!-- Save as Template Dialog -->
{#if showSaveTemplateDialog}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
		onclick={closeSaveTemplateDialog}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div onclick={(e: MouseEvent) => e.stopPropagation()}>
			<Card class="p-6 max-w-md w-full">
				<h2 class="text-2xl font-bold mb-4">Save as Template</h2>
				<p class="text-sm text-muted-foreground mb-4">
					Save this layout as a reusable template for future projects.
				</p>

				<div class="space-y-4 mb-4">
					<div>
						<label for="template-name" class="block text-sm font-medium mb-2">
							Template Name
						</label>
						<Input
							id="template-name"
							type="text"
							placeholder="My Awesome Template"
							bind:value={templateName}
							class="w-full"
						/>
					</div>

					<div>
						<label for="template-tags" class="block text-sm font-medium mb-2">
							Tags (comma-separated)
						</label>
						<Input
							id="template-tags"
							type="text"
							placeholder="corne, 42-key, minimal"
							bind:value={templateTags}
							class="w-full"
						/>
						<p class="text-xs text-muted-foreground mt-1">
							Tags help organize and find templates later
						</p>
					</div>
				</div>

				{#if saveTemplateError}
					<div class="mb-4 p-3 bg-destructive/10 text-destructive text-sm rounded">
						{saveTemplateError}
					</div>
				{/if}

				<div class="flex gap-2">
					<Button
						onclick={saveAsTemplate}
						disabled={saveTemplateLoading || !templateName.trim()}
						class="flex-1"
					>
						{saveTemplateLoading ? 'Saving...' : 'Save Template'}
					</Button>
					<Button
						onclick={closeSaveTemplateDialog}
						disabled={saveTemplateLoading}
						class="flex-1"
						variant="ghost"
					>
						Cancel
					</Button>
				</div>
			</Card>
		</div>
	</div>
{/if}

<!-- Variant Switch Dialog -->
{#if showVariantSwitchDialog}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
		onclick={() => (showVariantSwitchDialog = false)}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div onclick={(e: MouseEvent) => e.stopPropagation()}>
			<Card class="p-6 max-w-lg w-full">
				<h2 class="text-2xl font-bold mb-4">Switch Layout Variant</h2>
				<p class="text-sm text-muted-foreground mb-4">
					Change the physical layout variant for this keyboard. This will adjust the key count.
				</p>

				{#if variantsLoading}
					<p class="text-muted-foreground">Loading variants...</p>
				{:else if variantsError}
					<div class="p-3 bg-destructive/10 text-destructive text-sm rounded mb-4">
						{variantsError}
					</div>
				{:else}
					<div class="space-y-2 mb-4 max-h-64 overflow-y-auto">
						{#each availableVariants as variant}
							{@const isCurrent = variant.name === layout.metadata.layout_variant}
							<button
								class="w-full p-3 text-left border rounded hover:bg-muted flex justify-between items-center
								{isCurrent ? 'bg-primary/10 border-primary' : ''}"
								onclick={() => !isCurrent && switchToVariant(variant.name)}
								disabled={switchingVariant || isCurrent}
							>
								<span class="font-mono text-sm">{variant.name}</span>
								<span class="text-xs text-muted-foreground">
									{variant.key_count} keys
									{#if isCurrent}
										<span class="ml-2 text-primary">(current)</span>
									{/if}
								</span>
							</button>
						{/each}
					</div>
				{/if}

				{#if switchVariantError}
					<div class="mb-4 p-3 bg-destructive/10 text-destructive text-sm rounded">
						{switchVariantError}
					</div>
				{/if}

				{#if switchVariantWarning}
					<div class="mb-4 p-3 bg-yellow-50 dark:bg-yellow-950 border border-yellow-200 dark:border-yellow-800 text-yellow-800 dark:text-yellow-200 text-sm rounded">
						{switchVariantWarning}
					</div>
				{/if}

				<div class="flex justify-end">
					<Button
						onclick={() => (showVariantSwitchDialog = false)}
						disabled={switchingVariant}
						variant="outline"
					>
						{switchingVariant ? 'Switching...' : 'Close'}
					</Button>
				</div>
			</Card>
		</div>
	</div>
{/if}
