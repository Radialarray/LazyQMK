// API Types matching Rust backend

export interface HealthResponse {
	status: string;
	version: string;
}

export interface LayoutSummary {
	filename: string;
	name: string;
	description: string;
	modified: string;
}

export interface LayoutListResponse {
	layouts: LayoutSummary[];
}

export interface Layout {
	metadata: LayoutMetadata;
	layers: Layer[];
	tap_dances?: TapDance[];
	combos?: Combo[];
	// RGB settings
	rgb_enabled?: boolean;
	rgb_brightness?: number;
	rgb_saturation?: number;
	rgb_matrix_default_speed?: number;
	rgb_timeout_ms?: number;
	uncolored_key_behavior?: number;
	// Idle effect settings
	idle_effect_settings?: IdleEffectSettings;
	// RGB overlay ripple settings
	rgb_overlay_ripple?: RgbOverlayRippleSettings;
	// Tap-hold settings
	tap_hold_settings?: TapHoldSettings;
	// Categories
	categories?: Category[];
}

export interface LayoutMetadata {
	name: string;
	description: string;
	author: string;
	keyboard: string;
	layout: string;
	created: string;
	modified: string;
	idle_effect?: IdleEffect;
	layout_variant?: string;
	keymap_name?: string;
	output_format?: string;
	tags?: string[];
	is_template?: boolean;
	version?: string;
}

export interface IdleEffect {
	timeout: number;
	duration: number;
	effect: string;
}

export interface IdleEffectSettings {
	enabled: boolean;
	idle_timeout_ms: number;
	idle_effect_duration_ms: number;
	idle_effect_mode: string;
}

export interface RgbOverlayRippleSettings {
	enabled: boolean;
	max_ripples: number;
	duration_ms: number;
	speed: number;
	band_width: number;
	amplitude_pct: number;
	color_mode: string;
	fixed_color: RgbColor;
	hue_shift_deg: number;
	trigger_on_press: boolean;
	trigger_on_release: boolean;
	ignore_transparent: boolean;
	ignore_modifiers: boolean;
	ignore_layer_switch: boolean;
}

export interface TapHoldSettings {
	tapping_term: number;
	quick_tap_term?: number;
	hold_mode: string;
	retro_tapping: boolean;
	tapping_toggle: number;
	flow_tap_term?: number;
	chordal_hold: boolean;
	preset: string;
}

export interface Category {
	id: string;
	name: string;
	color: RgbColor;
	description?: string;
}

export interface RgbColor {
	r: number;
	g: number;
	b: number;
}

export interface Layer {
	name: string;
	number?: number;
	id?: string;
	default_color?: RgbColor;
	category_id?: string;
	layer_colors_enabled?: boolean;
	color: string;
	keys: KeyAssignment[];
}

export interface KeyAssignment {
	keycode: string;
	matrix_position: [number, number];
	visual_index: number;
	led_index: number;
	position?: { row: number; col: number };
	color_override?: RgbColor;
	category_id?: string;
	description?: string;
}

export interface TapDance {
	id?: string;
	name: string;
	single_tap: string;
	tap?: string; // alias for single_tap
	hold?: string;
	double_tap?: string;
	tap_hold?: string;
}

export interface Combo {
	id: string;
	name: string;
	keys: string[];
	output: string;
}

export interface KeycodeInfo {
	code: string;
	name: string;
	category: string;
	description?: string;
}

export interface KeycodeListResponse {
	keycodes: KeycodeInfo[];
	total: number;
}

export interface CategoryInfo {
	id: string;
	name: string;
	description: string;
}

export interface CategoryListResponse {
	categories: CategoryInfo[];
}

export interface ConfigResponse {
	qmk_firmware_path?: string;
	output_dir: string;
	workspace_root: string;
}

export interface ConfigUpdateRequest {
	qmk_firmware_path?: string;
}

export interface SwapKeysRequest {
	/** Layer number (0-based) */
	layer: number;
	/** First key position */
	first_position: { row: number; col: number };
	/** Second key position */
	second_position: { row: number; col: number };
}

export interface PreflightResponse {
	/** Whether QMK firmware path is configured and valid */
	qmk_configured: boolean;
	/** Whether any layouts exist in the workspace */
	has_layouts: boolean;
	/** True if this appears to be a first-run (no layouts and no QMK config) */
	first_run: boolean;
	/** QMK firmware path if configured */
	qmk_firmware_path?: string;
}

export interface ApiError {
	error: string;
	details?: string;
}

export interface KeyGeometryInfo {
	matrix_row: number;
	matrix_col: number;
	x: number;
	y: number;
	width: number;
	height: number;
	rotation: number;
	led_index?: number;
	visual_index: number;
}

export interface GeometryResponse {
	keyboard: string;
	layout: string;
	keys: KeyGeometryInfo[];
	matrix_rows: number;
	matrix_cols: number;
	encoder_count: number;
	/** Mapping from visual position ("row,col") to visual_index (layout array index).
	 * This allows the frontend to look up the visual_index for keys that only have
	 * position data, avoiding brittle coordinate inference logic.
	 */
	position_to_visual_index?: Record<string, number>;
}

// Validation response
export interface ValidationResponse {
	valid: boolean;
	error?: string;
	warnings: string[];
}

// Inspect response
export interface InspectResponse {
	metadata: InspectMetadata;
	layers: InspectLayer[];
	tap_dances: InspectTapDance[];
	settings: InspectSettings;
}

export interface InspectMetadata {
	name: string;
	description: string;
	author: string;
	keyboard?: string;
	layout_variant?: string;
	created: string;
	modified: string;
	layer_count: number;
	key_count: number;
	category_count: number;
	tap_dance_count: number;
}

export interface InspectLayer {
	number: number;
	name: string;
	key_count: number;
	default_color: string;
	colors_enabled: boolean;
}

export interface InspectTapDance {
	name: string;
	single_tap: string;
	double_tap?: string;
	hold?: string;
	used_in_layers: number[];
}

export interface InspectSettings {
	rgb_enabled: boolean;
	rgb_brightness: number;
	rgb_saturation: number;
	rgb_matrix_default_speed: number;
	idle_effect_enabled: boolean;
	idle_timeout_ms: number;
	idle_effect_mode: string;
	tapping_term: number;
	tap_hold_preset: string;
}

// Export response
export interface ExportResponse {
	markdown: string;
	suggested_filename: string;
}

// Generate response (initial response from starting a generate job)
export interface GenerateResponse {
	status: string;
	message: string;
	job: GenerateJob;
}

// Generate Job types
export type GenerateJobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface GenerateJob {
	id: string;
	status: GenerateJobStatus;
	layout_filename: string;
	keyboard: string;
	layout_variant: string;
	created_at: string;
	started_at?: string;
	completed_at?: string;
	error?: string;
	zip_path?: string;
	download_url?: string;
	progress: number;
}

export interface GenerateJobStatusResponse {
	job: GenerateJob;
}

export interface GenerateJobLogsResponse {
	job_id: string;
	logs: LogEntry[];
	has_more: boolean;
}

export interface CancelGenerateJobResponse {
	success: boolean;
	message: string;
}

// Effects list
export interface EffectsListResponse {
	effects: EffectInfo[];
}

export interface EffectInfo {
	id: string;
	name: string;
}

// Template types
export interface TemplateInfo {
	filename: string;
	name: string;
	description: string;
	author: string;
	tags: string[];
	created: string;
	layer_count: number;
}

export interface TemplateListResponse {
	templates: TemplateInfo[];
}

export interface SaveTemplateRequest {
	name: string;
	tags?: string[];
}

export interface ApplyTemplateRequest {
	target_filename: string;
}

// Keyboard & Setup Wizard types
export interface KeyboardInfo {
	path: string;
	layout_count: number;
}

export interface KeyboardListResponse {
	keyboards: KeyboardInfo[];
}

export interface LayoutVariantInfo {
	name: string;
	key_count: number;
}

export interface LayoutVariantsResponse {
	keyboard: string;
	variants: LayoutVariantInfo[];
}

export interface CreateLayoutRequest {
	filename: string;
	name: string;
	keyboard: string;
	layout_variant: string;
	description?: string;
	author?: string;
}

export interface SwitchVariantRequest {
	layout_variant: string;
}

export interface SwitchVariantResponse {
	layout: Layout;
	keys_added: number;
	keys_removed: number;
	warning?: string;
}

// Build Job types
export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface BuildJob {
	id: string;
	status: JobStatus;
	layout_filename: string;
	keyboard: string;
	keymap: string;
	created_at: string;
	started_at?: string;
	completed_at?: string;
	error?: string;
	firmware_path?: string;
	progress: number;
}

export interface StartBuildRequest {
	layout_filename: string;
}

export interface StartBuildResponse {
	job: BuildJob;
}

export interface JobStatusResponse {
	job: BuildJob;
}

export interface LogEntry {
	timestamp: string;
	level: string;
	message: string;
}

export interface JobLogsResponse {
	job_id: string;
	logs: LogEntry[];
	has_more: boolean;
}

export interface CancelJobResponse {
	success: boolean;
	message: string;
}

// Build Artifacts
export interface BuildArtifact {
	id: string;
	filename: string;
	artifact_type: string;
	size: number;
	sha256?: string;
	download_url: string;
}

export interface BuildArtifactsResponse {
	job_id: string;
	artifacts: BuildArtifact[];
}

// Render Metadata Types (for Key Details Panel)

/** Display labels for a key (short form for in-key display) */
export interface KeyDisplayDto {
	/** Primary/main label for the key */
	primary: string;
	/** Secondary label (e.g., hold action) - optional */
	secondary?: string;
	/** Tertiary label (e.g., double-tap for tap-dance) - optional */
	tertiary?: string;
}

/** Type of action in a multi-action keycode */
export type ActionKindDto = 'tap' | 'hold' | 'double_tap' | 'layer' | 'modifier' | 'simple';

/** Detailed description of a single action within a keycode */
export interface KeyDetailActionDto {
	/** Type of action */
	kind: ActionKindDto;
	/** Raw keycode or parameter (e.g., "KC_A", "1", "MOD_LCTL") */
	code: string;
	/** Human-readable description */
	description: string;
}

/** Complete key render metadata for a single key */
export interface KeyRenderMetadata {
	/** Visual index (layout array index from info.json) */
	visual_index: number;
	/** Short labels for in-key display */
	display: KeyDisplayDto;
	/** Full action breakdown for Key Details panel */
	details: KeyDetailActionDto[];
}

/** Render metadata for a single layer */
export interface LayerRenderMetadata {
	/** Layer number */
	number: number;
	/** Layer name */
	name: string;
	/** Per-key render metadata */
	keys: KeyRenderMetadata[];
}

/** Response for layout render metadata */
export interface RenderMetadataResponse {
	/** Layout filename */
	filename: string;
	/** Layer-indexed key metadata */
	layers: LayerRenderMetadata[];
}
