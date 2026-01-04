import type {
	HealthResponse,
	LayoutListResponse,
	Layout,
	KeycodeListResponse,
	CategoryListResponse,
	ConfigResponse,
	ConfigUpdateRequest,
	GeometryResponse,
	ApiError
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

	// Geometry Operations
	async getGeometry(keyboard: string, layout: string): Promise<GeometryResponse> {
		return this.request<GeometryResponse>(
			`/api/keyboards/${encodeURIComponent(keyboard)}/geometry/${encodeURIComponent(layout)}`
		);
	}
}

// Default instance
export const apiClient = new ApiClient();
