<script lang="ts">
	import { Button, Card, ColorPicker, Input } from '$components';
	import type { Layout, PaletteFxSettings, RgbColor, RgbOverlayRippleSettings } from '$api/types';

	interface Props {
		layout: Layout;
		onRgbGeneralChange: (field: string, value: number | boolean) => void;
		onIdleEffectChange: (field: string, value: number | boolean | string) => void;
		onOverlayRippleChange: (
			field: keyof RgbOverlayRippleSettings,
			value: number | boolean | string | RgbColor
		) => void;
		onPaletteFxChange: (field: keyof PaletteFxSettings, value: number | boolean | string) => void;
	}

	let { layout, onRgbGeneralChange, onIdleEffectChange, onOverlayRippleChange, onPaletteFxChange }: Props =
		$props();

	function clampInt(raw: string, min: number, max: number, fallback: number): number {
		const n = parseInt(raw, 10);
		if (Number.isNaN(n)) return fallback;
		return Math.min(max, Math.max(min, n));
	}

	function formatTimeoutLabel(ms: number): string {
		if (ms === 0) return 'Disabled';
		if (ms >= 60000 && ms % 60000 === 0) return `${ms / 60000} min`;
		if (ms >= 1000 && ms % 1000 === 0) return `${ms / 1000} sec`;
		return `${ms} ms`;
	}
</script>

<div class="space-y-6" data-testid="layout-settings-root">
	<!-- Background Lighting -->
	<Card class="p-6" data-testid="background-lighting-card">
		<div class="mb-4 flex items-center justify-between gap-4">
			<div>
				<p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
					Lighting
				</p>
				<h2 class="text-xl font-semibold mt-1">Background Lighting</h2>
			</div>
			<span class="rounded-full border px-3 py-1 text-xs text-muted-foreground">Basic</span>
		</div>
		<p class="text-sm text-muted-foreground mb-6">
			Controls the brightness and color shown behind keys that have no category, group, or layer color
			of their own.
		</p>

		<div class="space-y-5 max-w-2xl">
			<div class="flex items-center gap-3">
				<input
					type="checkbox"
					id="rgb-enabled"
					class="w-4 h-4"
					checked={layout.rgb_enabled ?? true}
					onchange={(e) => onRgbGeneralChange('rgb_enabled', e.currentTarget.checked)}
				/>
				<label for="rgb-enabled" class="text-sm font-medium">Turn on RGB lighting</label>
			</div>

			<div class="flex items-start gap-3">
				<input
					type="checkbox"
					id="base-colors-for-unassigned-keys"
					class="w-4 h-4 mt-0.5"
					checked={layout.show_base_layer_colors_for_unassigned_keys ?? false}
					onchange={(e) =>
						onRgbGeneralChange('show_base_layer_colors_for_unassigned_keys', e.currentTarget.checked)}
				/>
				<div>
					<label for="base-colors-for-unassigned-keys" class="text-sm font-medium">
						Use base-layer colors for unassigned keys
					</label>
					<p class="text-xs text-muted-foreground mt-1">
						Shows the Layer 0 color for KC_TRNS and KC_NO on higher layers. KC_NO stays disabled.
					</p>
				</div>
			</div>

			<div>
				<div class="flex items-center justify-between">
					<label for="uncolored-brightness" class="block text-sm font-medium text-muted-foreground">
						Brightness (uncolored keys)
					</label>
					<span class="text-sm font-mono">{layout.uncolored_key_behavior ?? 100}%</span>
				</div>
				<input
					id="uncolored-brightness"
					type="range"
					min="0"
					max="100"
					step="1"
					class="mt-2 w-full"
					value={layout.uncolored_key_behavior ?? 100}
					oninput={(e) =>
						onRgbGeneralChange('uncolored_key_behavior', clampInt(e.currentTarget.value, 0, 100, 100))}
					data-testid="uncolored-brightness-slider"
				/>
				<p class="text-xs text-muted-foreground mt-1">
					How bright keys render when no category, group, or layer color is assigned. Used by the
					layer color picker too.
				</p>
			</div>

			<div>
				<div class="flex items-center justify-between">
					<label for="rgb-saturation" class="block text-sm font-medium text-muted-foreground">
						Saturation
					</label>
					<span class="text-sm font-mono">{layout.rgb_saturation ?? 100}%</span>
				</div>
				<input
					id="rgb-saturation"
					type="range"
					min="0"
					max="200"
					step="1"
					class="mt-2 w-full"
					value={layout.rgb_saturation ?? 100}
					oninput={(e) => onRgbGeneralChange('rgb_saturation', clampInt(e.currentTarget.value, 0, 200, 100))}
				/>
				<p class="text-xs text-muted-foreground mt-1">
					0% turns colors into grayscale, 100% is the natural color, 200% is fully saturated.
				</p>
			</div>

			<div>
				<div class="flex items-center justify-between">
					<label for="rgb-speed" class="block text-sm font-medium text-muted-foreground">
						Matrix animation speed
					</label>
					<span class="text-sm font-mono">{layout.rgb_matrix_default_speed ?? 127}</span>
				</div>
				<input
					id="rgb-speed"
					type="range"
					min="0"
					max="255"
					step="1"
					class="mt-2 w-full"
					value={layout.rgb_matrix_default_speed ?? 127}
					oninput={(e) =>
						onRgbGeneralChange('rgb_matrix_default_speed', clampInt(e.currentTarget.value, 0, 255, 127))}
				/>
				<p class="text-xs text-muted-foreground mt-1">
					Used by the idle effect and ripple animations. Higher number = faster.
				</p>
			</div>

			<div>
				<label for="rgb-timeout" class="block text-sm font-medium text-muted-foreground mb-1">
					Auto-off after inactivity
				</label>
				<Input
					id="rgb-timeout"
					type="number"
					class="w-32"
					min="0"
					max="65535"
					value={Math.round((layout.rgb_timeout_ms ?? 60000) / 1000)}
					oninput={(e) =>
						onRgbGeneralChange(
							'rgb_timeout_ms',
							clampInt(e.currentTarget.value, 0, 65535, 60) * 1000
						)}
				/>
				<p class="text-xs text-muted-foreground mt-1">
					Current value: {formatTimeoutLabel(layout.rgb_timeout_ms ?? 60000)}. Set 0 to disable
					auto-off.
				</p>
			</div>
		</div>
	</Card>

	<!-- Idle Lighting -->
	<Card class="p-6" data-testid="idle-lighting-card">
		<h2 class="text-lg font-semibold mb-4">Idle Lighting</h2>
		<p class="text-muted-foreground text-sm mb-6">
			Choose what people see after your keyboard sits untouched for a while.
		</p>
		<details class="mb-6 rounded-lg border border-border bg-muted/20 p-4">
			<summary class="cursor-pointer font-medium">Show timing guidance</summary>
			<p class="mt-2 text-sm text-muted-foreground">
				Short timeouts feel lively on desk displays. Longer timeouts avoid distractions during
				active work.
			</p>
		</details>

		<div class="space-y-4 max-w-md">
			<div class="flex items-center gap-3">
				<input
					type="checkbox"
					id="idle-enabled"
					class="w-4 h-4"
					checked={layout.idle_effect_settings?.enabled ?? true}
					onchange={(e) => onIdleEffectChange('enabled', e.currentTarget.checked)}
				/>
				<label for="idle-enabled" class="text-sm font-medium">Turn on idle lighting</label>
			</div>

			<div>
				<label for="idle-timeout" class="block text-sm font-medium text-muted-foreground mb-1">
					Idle Timeout (seconds)
				</label>
				<Input
					id="idle-timeout"
					type="number"
					value={Math.round((layout.idle_effect_settings?.idle_timeout_ms ?? 60000) / 1000)}
					oninput={(e) =>
						onIdleEffectChange('idle_timeout_ms', parseInt(e.currentTarget.value) * 1000)}
					min="10"
					max="600"
				/>
				<p class="text-xs text-muted-foreground mt-1">
					Wait time after last key press before idle lighting begins (10-600 seconds).
				</p>
			</div>

			<div>
				<label for="idle-duration" class="block text-sm font-medium text-muted-foreground mb-1">
					Effect Duration (seconds)
				</label>
				<Input
					id="idle-duration"
					type="number"
					value={Math.round(
						(layout.idle_effect_settings?.idle_effect_duration_ms ?? 300000) / 1000
					)}
					oninput={(e) =>
						onIdleEffectChange(
							'idle_effect_duration_ms',
							parseInt(e.currentTarget.value) * 1000
						)}
					min="30"
					max="3600"
				/>
				<p class="text-xs text-muted-foreground mt-1">
					How long idle effect keeps running before lights turn off again (30-3600 seconds).
				</p>
			</div>

			<div>
				<label for="idle-effect-mode" class="block text-sm font-medium text-muted-foreground mb-1">
					Effect Mode
				</label>
				<select
					id="idle-effect-mode"
					class="w-full px-3 py-2 border border-border rounded-lg bg-background"
					value={layout.idle_effect_settings?.idle_effect_mode ?? 'Breathing'}
					onchange={(e) => onIdleEffectChange('idle_effect_mode', e.currentTarget.value)}
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
				<p class="text-xs text-muted-foreground mt-1">Visual pattern shown during idle period.</p>
			</div>
		</div>
	</Card>

	<!-- PaletteFX -->
	<Card class="p-6" data-testid="palette-fx-card">
		<h2 class="text-lg font-semibold mb-4">PaletteFX Effects</h2>
		<p class="text-muted-foreground text-sm mb-6">
			Replace custom ripple overlay with community module effects using professional color palettes.
		</p>
		<details class="mb-6 rounded-lg border border-border bg-muted/20 p-4">
			<summary class="cursor-pointer font-medium">About PaletteFX</summary>
			<p class="mt-2 text-sm text-muted-foreground">
				PaletteFX provides animated RGB effects driven by curated color palettes. Requires QMK 0.28+
				and the getreuer/palettefx community module installed in
				qmk_firmware/modules/getreuer/.
			</p>
		</details>
		<div class="space-y-4 max-w-md">
			<div class="flex items-center gap-3">
				<input
					type="checkbox"
					id="pfx-enabled"
					class="w-4 h-4"
					checked={layout.palette_fx?.enabled ?? false}
					onchange={(e) => onPaletteFxChange('enabled', e.currentTarget.checked)}
				/>
				<label for="pfx-enabled" class="text-sm font-medium">PaletteFX Enabled</label>
			</div>
			<div>
				<label for="pfx-effect" class="block text-sm font-medium text-muted-foreground mb-1">
					Default Effect
				</label>
				<select
					id="pfx-effect"
					class="w-full px-3 py-2 border border-border rounded-lg bg-background"
					value={layout.palette_fx?.default_effect ?? 'Flow'}
					onchange={(e) => onPaletteFxChange('default_effect', e.currentTarget.value)}
				>
					<option value="Gradient">Gradient</option>
					<option value="Flow">Flow</option>
					<option value="Ripple">Ripple</option>
					<option value="Sparkle">Sparkle</option>
					<option value="Vortex">Vortex</option>
					<option value="Reactive">Reactive</option>
				</select>
			</div>
			<div>
				<label for="pfx-palette" class="block text-sm font-medium text-muted-foreground mb-1">
					Default Palette
				</label>
				<select
					id="pfx-palette"
					class="w-full px-3 py-2 border border-border rounded-lg bg-background"
					value={layout.palette_fx?.default_palette ?? 'Synthwave'}
					onchange={(e) => onPaletteFxChange('default_palette', e.currentTarget.value)}
				>
					<option value="Afterburn">Afterburn</option>
					<option value="Amber">Amber</option>
					<option value="Bad Wolf">Bad Wolf</option>
					<option value="Carnival">Carnival</option>
					<option value="Classic">Classic</option>
					<option value="Dracula">Dracula</option>
					<option value="Groovy">Groovy</option>
					<option value="Not Pink">Not Pink</option>
					<option value="Phosphor">Phosphor</option>
					<option value="Polarized">Polarized</option>
					<option value="Rose Gold">Rose Gold</option>
					<option value="Sport">Sport</option>
					<option value="Synthwave">Synthwave</option>
					<option value="Thermal">Thermal</option>
					<option value="Viridis">Viridis</option>
					<option value="Watermelon">Watermelon</option>
				</select>
			</div>
			<div class="flex items-center gap-3">
				<input
					type="checkbox"
					id="pfx-all-effects"
					class="w-4 h-4"
					checked={layout.palette_fx?.enable_all_effects ?? true}
					onchange={(e) => onPaletteFxChange('enable_all_effects', e.currentTarget.checked)}
				/>
				<label for="pfx-all-effects" class="text-sm font-medium">Enable All Effects</label>
			</div>
			<div class="flex items-center gap-3">
				<input
					type="checkbox"
					id="pfx-all-palettes"
					class="w-4 h-4"
					checked={layout.palette_fx?.enable_all_palettes ?? true}
					onchange={(e) => onPaletteFxChange('enable_all_palettes', e.currentTarget.checked)}
				/>
				<label for="pfx-all-palettes" class="text-sm font-medium">Enable All Palettes</label>
			</div>
		</div>
	</Card>

	<!-- Ripple Lighting -->
	<Card class="p-6" data-testid="ripple-lighting-card">
		<h2 class="text-lg font-semibold mb-4">Ripple Lighting</h2>
		<p class="text-muted-foreground text-sm mb-6">
			Add motion on key press while keeping your normal layer colors underneath.
		</p>
		<details class="mb-6 rounded-lg border border-border bg-muted/20 p-4">
			<summary class="cursor-pointer font-medium">How to tune ripple lighting</summary>
			<p class="mt-2 text-sm text-muted-foreground">
				Turn ripple lighting on first. Open advanced settings only when you want to fine-tune motion,
				triggers, or ignore rules.
			</p>
		</details>

		<div class="space-y-6 max-w-2xl">
			<div class="flex items-center gap-3">
				<input
					type="checkbox"
					id="ripple-enabled"
					class="w-4 h-4"
					checked={layout.rgb_overlay_ripple?.enabled ?? false}
					onchange={(e) => onOverlayRippleChange('enabled', e.currentTarget.checked)}
				/>
				<label for="ripple-enabled" class="text-sm font-medium">Turn on ripple lighting</label>
			</div>

			<details
				class="rounded-lg border border-border p-4"
				open={layout.rgb_overlay_ripple?.enabled ?? false}
			>
				<summary class="cursor-pointer font-medium">Advanced ripple settings</summary>
				<div class="mt-4 grid grid-cols-1 md:grid-cols-2 gap-4">
					<div>
						<label for="max-ripples" class="block text-sm font-medium text-muted-foreground mb-1">
							Max Ripples
						</label>
						<Input
							id="max-ripples"
							type="number"
							value={layout.rgb_overlay_ripple?.max_ripples ?? 4}
							oninput={(e) =>
								onOverlayRippleChange(
									'max_ripples',
									clampInt(e.currentTarget.value, 1, 8, 4)
								)}
							min="1"
							max="8"
						/>
						<p class="text-xs text-muted-foreground mt-1">
							Maximum number of ripple animations visible at same time (1-8).
						</p>
					</div>

					<div>
						<label for="duration" class="block text-sm font-medium text-muted-foreground mb-1">
							Duration (ms)
						</label>
						<Input
							id="duration"
							type="number"
							value={layout.rgb_overlay_ripple?.duration_ms ?? 1500}
							oninput={(e) =>
								onOverlayRippleChange(
									'duration_ms',
									clampInt(e.currentTarget.value, 0, 65535, 1500)
								)}
							min="100"
							max="5000"
						/>
						<p class="text-xs text-muted-foreground mt-1">
							Length of one ripple animation in milliseconds.
						</p>
					</div>

					<div>
						<label for="speed" class="block text-sm font-medium text-muted-foreground mb-1">
							Speed
						</label>
						<Input
							id="speed"
							type="number"
							value={layout.rgb_overlay_ripple?.speed ?? 200}
							oninput={(e) =>
								onOverlayRippleChange('speed', clampInt(e.currentTarget.value, 0, 255, 200))}
							min="0"
							max="255"
						/>
						<p class="text-xs text-muted-foreground mt-1">
							How quickly ripple expands outward. Higher number = faster motion.
						</p>
					</div>

					<div>
						<label for="band-width" class="block text-sm font-medium text-muted-foreground mb-1">
							Band Width
						</label>
						<Input
							id="band-width"
							type="number"
							value={layout.rgb_overlay_ripple?.band_width ?? 30}
							oninput={(e) =>
								onOverlayRippleChange(
									'band_width',
									clampInt(e.currentTarget.value, 0, 255, 30)
								)}
							min="1"
							max="255"
						/>
						<p class="text-xs text-muted-foreground mt-1">
							Thickness of bright ripple band. Higher number = wider ring.
						</p>
					</div>

					<div>
						<label for="amplitude" class="block text-sm font-medium text-muted-foreground mb-1">
							Amplitude (%)
						</label>
						<Input
							id="amplitude"
							type="number"
							value={layout.rgb_overlay_ripple?.amplitude_pct ?? 50}
							oninput={(e) =>
								onOverlayRippleChange(
									'amplitude_pct',
									clampInt(e.currentTarget.value, 0, 100, 50)
								)}
							min="0"
							max="100"
						/>
						<p class="text-xs text-muted-foreground mt-1">
							Extra brightness added by ripple compared with base layer color.
						</p>
					</div>
				</div>

				<div class="mt-4">
					<label for="color-mode" class="block text-sm font-medium text-muted-foreground mb-1">
						Color Mode
					</label>
					<select
						id="color-mode"
						class="w-full px-3 py-2 border border-border rounded-lg bg-background"
						value={layout.rgb_overlay_ripple?.color_mode ?? 'Fixed Color'}
						onchange={(e) => onOverlayRippleChange('color_mode', e.currentTarget.value)}
					>
						<option value="Fixed Color">Fixed Color</option>
						<option value="Key Color">Key Color</option>
						<option value="Hue Shift">Hue Shift</option>
					</select>
					<p class="text-xs text-muted-foreground mt-1">
						{#if (layout.rgb_overlay_ripple?.color_mode ?? 'Fixed Color') === 'Fixed Color'}
							Use the same color for all ripples
						{:else if (layout.rgb_overlay_ripple?.color_mode ?? 'Fixed Color') === 'Key Color'}
							Use each key's base color from layer settings
						{:else}
							Shift hue from key's base color by fixed degrees
						{/if}
					</p>
				</div>

				{#if (layout.rgb_overlay_ripple?.color_mode ?? 'Fixed Color') === 'Fixed Color'}
					<div class="mt-4">
						<label class="block text-sm font-medium text-muted-foreground mb-2">Fixed Color</label>
						<ColorPicker
							color={layout.rgb_overlay_ripple?.fixed_color ?? { r: 0, g: 255, b: 255 }}
							onSelect={(color) => onOverlayRippleChange('fixed_color', color)}
						/>
					</div>
				{/if}

				{#if (layout.rgb_overlay_ripple?.color_mode ?? 'Fixed Color') === 'Hue Shift'}
					<div class="mt-4">
						<label for="hue-shift" class="block text-sm font-medium text-muted-foreground mb-1">
							Hue Shift (degrees)
						</label>
						<Input
							id="hue-shift"
							type="number"
							value={layout.rgb_overlay_ripple?.hue_shift_deg ?? 60}
							oninput={(e) =>
								onOverlayRippleChange(
									'hue_shift_deg',
									clampInt(e.currentTarget.value, -180, 180, 60)
								)}
							min="-180"
							max="180"
						/>
						<p class="text-xs text-muted-foreground mt-1">
							Color shift amount from base key color. Positive and negative move in opposite
							directions.
						</p>
					</div>
				{/if}

				<div class="mt-4 space-y-3">
					<h3 class="text-sm font-semibold">Trigger Options</h3>
					<div class="flex items-center gap-3">
						<input
							type="checkbox"
							id="trigger-press"
							class="w-4 h-4"
							checked={layout.rgb_overlay_ripple?.trigger_on_press ?? true}
							onchange={(e) => onOverlayRippleChange('trigger_on_press', e.currentTarget.checked)}
						/>
						<label for="trigger-press" class="text-sm">Trigger on key press</label>
					</div>
					<div class="flex items-center gap-3">
						<input
							type="checkbox"
							id="trigger-release"
							class="w-4 h-4"
							checked={layout.rgb_overlay_ripple?.trigger_on_release ?? false}
							onchange={(e) =>
								onOverlayRippleChange('trigger_on_release', e.currentTarget.checked)}
						/>
						<label for="trigger-release" class="text-sm">Trigger on key release</label>
					</div>
				</div>

				<div class="mt-4 space-y-3">
					<h3 class="text-sm font-semibold">Ignore Options</h3>
					<div class="flex items-center gap-3">
						<input
							type="checkbox"
							id="ignore-transparent"
							class="w-4 h-4"
							checked={layout.rgb_overlay_ripple?.ignore_transparent ?? true}
							onchange={(e) => onOverlayRippleChange('ignore_transparent', e.currentTarget.checked)}
						/>
						<label for="ignore-transparent" class="text-sm">
							Ignore lower-layer passthrough keys
							<span class="kbd-token ml-1">KC_TRNS</span>
						</label>
					</div>
					<div class="flex items-center gap-3">
						<input
							type="checkbox"
							id="ignore-modifiers"
							class="w-4 h-4"
							checked={layout.rgb_overlay_ripple?.ignore_modifiers ?? false}
							onchange={(e) => onOverlayRippleChange('ignore_modifiers', e.currentTarget.checked)}
						/>
						<label for="ignore-modifiers" class="text-sm">Ignore modifier keys</label>
					</div>
					<div class="flex items-center gap-3">
						<input
							type="checkbox"
							id="ignore-layer-switch"
							class="w-4 h-4"
							checked={layout.rgb_overlay_ripple?.ignore_layer_switch ?? false}
							onchange={(e) =>
								onOverlayRippleChange('ignore_layer_switch', e.currentTarget.checked)}
						/>
						<label for="ignore-layer-switch" class="text-sm">Ignore layer switch keys</label>
					</div>
				</div>
			</details>
		</div>
	</Card>
</div>
