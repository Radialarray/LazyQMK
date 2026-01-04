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
}

export interface IdleEffect {
	timeout: number;
	duration: number;
	effect: string;
}

export interface Layer {
	name: string;
	color: string;
	keys: KeyAssignment[];
}

export interface KeyAssignment {
	keycode: string;
	matrix_position: [number, number];
	visual_index: number;
	led_index: number;
}

export interface TapDance {
	id: string;
	name: string;
	tap: string;
	hold: string;
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
}

export interface ConfigUpdateRequest {
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
}

export interface GeometryResponse {
	keyboard: string;
	layout: string;
	keys: KeyGeometryInfo[];
	matrix_rows: number;
	matrix_cols: number;
	encoder_count: number;
}
