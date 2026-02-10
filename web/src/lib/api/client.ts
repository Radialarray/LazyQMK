import type {
	HealthResponse,
	LayoutListResponse,
	Layout,
	KeycodeListResponse,
	CategoryListResponse,
	ConfigResponse,
	ConfigUpdateRequest,
	SwapKeysRequest,
	PreflightResponse,
	GeometryResponse,
	ApiError,
	ValidationResponse,
	InspectResponse,
	ExportResponse,
	GenerateResponse,
	GenerateJob,
	GenerateJobStatusResponse,
	GenerateJobLogsResponse,
	CancelGenerateJobResponse,
	EffectsListResponse,
	TemplateListResponse,
	TemplateInfo,
	SaveTemplateRequest,
	ApplyTemplateRequest,
	KeyboardListResponse,
	LayoutVariantsResponse,
	CreateLayoutRequest,
	SwitchVariantResponse,
	StartBuildRequest,
	StartBuildResponse,
	JobStatusResponse,
	JobLogsResponse,
	CancelJobResponse,
	BuildJob,
	BuildArtifactsResponse,
	RenderMetadataResponse
} from './types';

interface LazyQmkApiWindow {
	__LAZYQMK_API_BASE_URL?: string;
}

export class ApiClient {
	private baseUrl: string;

	constructor(baseUrl?: string) {
		// Default to current origin, configurable for testing
		this.baseUrl = baseUrl || '';
	}

	private resolveBaseUrl(): string {
		const runtimeBaseUrl = (globalThis as LazyQmkApiWindow).__LAZYQMK_API_BASE_URL;
		if (runtimeBaseUrl && runtimeBaseUrl.length > 0) {
			return runtimeBaseUrl;
		}
		return this.baseUrl;
	}

	private async request<T>(endpoint: string, options?: RequestInit): Promise<T> {
		const url = `${this.resolveBaseUrl()}${endpoint}`;
		const response = await fetch(url, {
			...options,
			headers: {
				'Content-Type': 'application/json',
				...options?.headers
			}
		});

		if (!response.ok) {
			let errorData: ApiError;
			try {
				errorData = await response.json();
			} catch {
				errorData = {
					error: `HTTP ${response.status}: ${response.statusText}`
				};
			}
			throw new Error(errorData.details || errorData.error);
		}

		// Handle 204 No Content
		if (response.status === 204) {
			return undefined as T;
		}

		return response.json();
	}

	// Health Check
	async health(): Promise<HealthResponse> {
		return this.request<HealthResponse>('/health');
	}

	// Layout Operations
	async listLayouts(): Promise<LayoutListResponse> {
		return this.request<LayoutListResponse>('/api/layouts');
	}

	async getLayout(filename: string): Promise<Layout> {
		return this.request<Layout>(`/api/layouts/${encodeURIComponent(filename)}`);
	}

	async saveLayout(filename: string, layout: Layout): Promise<void> {
		return this.request<void>(`/api/layouts/${encodeURIComponent(filename)}`, {
			method: 'PUT',
			body: JSON.stringify(layout)
		});
	}

	async swapKeys(filename: string, request: SwapKeysRequest): Promise<void> {
		return this.request<void>(`/api/layouts/${encodeURIComponent(filename)}/swap-keys`, {
			method: 'POST',
			body: JSON.stringify(request)
		});
	}

	async validateLayout(filename: string): Promise<ValidationResponse> {
		return this.request<ValidationResponse>(
			`/api/layouts/${encodeURIComponent(filename)}/validate`
		);
	}

	async inspectLayout(filename: string): Promise<InspectResponse> {
		return this.request<InspectResponse>(`/api/layouts/${encodeURIComponent(filename)}/inspect`);
	}

	async exportLayout(filename: string): Promise<ExportResponse> {
		return this.request<ExportResponse>(`/api/layouts/${encodeURIComponent(filename)}/export`);
	}

	async generateFirmware(filename: string): Promise<GenerateResponse> {
		return this.request<GenerateResponse>(
			`/api/layouts/${encodeURIComponent(filename)}/generate`,
			{
				method: 'POST'
			}
		);
	}

	async getRenderMetadata(filename: string): Promise<RenderMetadataResponse> {
		return this.request<RenderMetadataResponse>(
			`/api/layouts/${encodeURIComponent(filename)}/render-metadata`
		);
	}

	// Keycode Operations
	async listKeycodes(search?: string, category?: string): Promise<KeycodeListResponse> {
		const params = new URLSearchParams();
		if (search) params.set('search', search);
		if (category) params.set('category', category);
		const query = params.toString();
		const endpoint = `/api/keycodes${query ? `?${query}` : ''}`;
		return this.request<KeycodeListResponse>(endpoint);
	}

	async listCategories(): Promise<CategoryListResponse> {
		return this.request<CategoryListResponse>('/api/keycodes/categories');
	}

	// Config Operations
	async getConfig(): Promise<ConfigResponse> {
		return this.request<ConfigResponse>('/api/config');
	}

	async updateConfig(config: ConfigUpdateRequest): Promise<void> {
		return this.request<void>('/api/config', {
			method: 'PUT',
			body: JSON.stringify(config)
		});
	}

	// Preflight check for onboarding
	async preflight(): Promise<PreflightResponse> {
		return this.request<PreflightResponse>('/api/preflight');
	}

	// Effects Operations
	async listEffects(): Promise<EffectsListResponse> {
		return this.request<EffectsListResponse>('/api/effects');
	}

	// Geometry Operations
	async getGeometry(keyboard: string, layout: string): Promise<GeometryResponse> {
		return this.request<GeometryResponse>(
			`/api/keyboards/${encodeURIComponent(keyboard)}/geometry/${encodeURIComponent(layout)}`
		);
	}

	// Template Operations
	async listTemplates(): Promise<TemplateListResponse> {
		return this.request<TemplateListResponse>('/api/templates');
	}

	async getTemplate(filename: string): Promise<Layout> {
		return this.request<Layout>(`/api/templates/${encodeURIComponent(filename)}`);
	}

	async saveAsTemplate(filename: string, request: SaveTemplateRequest): Promise<TemplateInfo> {
		return this.request<TemplateInfo>(
			`/api/layouts/${encodeURIComponent(filename)}/save-as-template`,
			{
				method: 'POST',
				body: JSON.stringify(request)
			}
		);
	}

	async applyTemplate(
		templateFilename: string,
		request: ApplyTemplateRequest
	): Promise<Layout> {
		return this.request<Layout>(
			`/api/templates/${encodeURIComponent(templateFilename)}/apply`,
			{
				method: 'POST',
				body: JSON.stringify(request)
			}
		);
	}

	// Keyboard & Setup Wizard Operations
	async listKeyboards(): Promise<KeyboardListResponse> {
		return this.request<KeyboardListResponse>('/api/keyboards');
	}

	async listKeyboardLayouts(keyboard: string): Promise<LayoutVariantsResponse> {
		return this.request<LayoutVariantsResponse>(
			`/api/keyboards/${encodeURIComponent(keyboard)}/layouts`
		);
	}

	async createLayout(request: CreateLayoutRequest): Promise<Layout> {
		return this.request<Layout>('/api/layouts', {
			method: 'POST',
			body: JSON.stringify(request)
		});
	}

	async switchLayoutVariant(filename: string, layoutVariant: string): Promise<SwitchVariantResponse> {
		return this.request<SwitchVariantResponse>(
			`/api/layouts/${encodeURIComponent(filename)}/switch-variant`,
			{
				method: 'POST',
				body: JSON.stringify({ layout_variant: layoutVariant })
			}
		);
	}

	// Build Job Operations
	async startBuild(layoutFilename: string): Promise<StartBuildResponse> {
		const request: StartBuildRequest = { layout_filename: layoutFilename };
		return this.request<StartBuildResponse>('/api/build/start', {
			method: 'POST',
			body: JSON.stringify(request)
		});
	}

	async listBuildJobs(): Promise<BuildJob[]> {
		return this.request<BuildJob[]>('/api/build/jobs');
	}

	async getBuildJob(jobId: string): Promise<JobStatusResponse> {
		return this.request<JobStatusResponse>(`/api/build/jobs/${encodeURIComponent(jobId)}`);
	}

	async getBuildLogs(jobId: string, offset: number = 0, limit: number = 100): Promise<JobLogsResponse> {
		const params = new URLSearchParams();
		params.set('offset', offset.toString());
		params.set('limit', limit.toString());
		return this.request<JobLogsResponse>(`/api/build/jobs/${encodeURIComponent(jobId)}/logs?${params.toString()}`);
	}

	async cancelBuild(jobId: string): Promise<CancelJobResponse> {
		return this.request<CancelJobResponse>(`/api/build/jobs/${encodeURIComponent(jobId)}/cancel`, {
			method: 'POST'
		});
	}

	async getBuildArtifacts(jobId: string): Promise<BuildArtifactsResponse> {
		return this.request<BuildArtifactsResponse>(`/api/build/jobs/${encodeURIComponent(jobId)}/artifacts`);
	}

	/**
	 * Returns the download URL for a specific build artifact.
	 * @param jobId The build job ID
	 * @param artifactId The artifact ID (e.g., "uf2", "hex", "bin")
	 * @returns Full URL to download the artifact
	 */
	getBuildArtifactDownloadUrl(jobId: string, artifactId: string): string {
		return `${this.resolveBaseUrl()}/api/build/jobs/${encodeURIComponent(jobId)}/artifacts/${encodeURIComponent(artifactId)}/download`;
	}

	// Generate Job Operations
	async listGenerateJobs(): Promise<GenerateJob[]> {
		return this.request<GenerateJob[]>('/api/generate/jobs');
	}

	async getGenerateJob(jobId: string): Promise<GenerateJobStatusResponse> {
		return this.request<GenerateJobStatusResponse>(`/api/generate/jobs/${encodeURIComponent(jobId)}`);
	}

	async getGenerateLogs(jobId: string, offset: number = 0, limit: number = 100): Promise<GenerateJobLogsResponse> {
		const params = new URLSearchParams();
		params.set('offset', offset.toString());
		params.set('limit', limit.toString());
		return this.request<GenerateJobLogsResponse>(`/api/generate/jobs/${encodeURIComponent(jobId)}/logs?${params.toString()}`);
	}

	async cancelGenerate(jobId: string): Promise<CancelGenerateJobResponse> {
		return this.request<CancelGenerateJobResponse>(`/api/generate/jobs/${encodeURIComponent(jobId)}/cancel`, {
			method: 'POST'
		});
	}

	/**
	 * Returns the download URL for a completed generate job's zip file.
	 * @param jobId The generate job ID
	 * @returns Full URL to download the generated zip file
	 */
	getGenerateDownloadUrl(jobId: string): string {
		return `${this.resolveBaseUrl()}/api/generate/jobs/${encodeURIComponent(jobId)}/download`;
	}
}

// Default instance
export const apiClient = new ApiClient();
