import type {
	HealthResponse,
	LayoutListResponse,
	Layout,
	KeycodeListResponse,
	CategoryListResponse,
	ConfigResponse,
	ConfigUpdateRequest,
	GeometryResponse,
	ApiError,
	ValidationResponse,
	InspectResponse,
	ExportResponse,
	GenerateResponse,
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
	RenderMetadataResponse
} from './types';

export class ApiClient {
	private baseUrl: string;

	constructor(baseUrl?: string) {
		// Default to current origin, configurable for testing
		this.baseUrl = baseUrl || '';
	}

	private async request<T>(endpoint: string, options?: RequestInit): Promise<T> {
		const url = `${this.baseUrl}${endpoint}`;
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
}

// Default instance
export const apiClient = new ApiClient();
