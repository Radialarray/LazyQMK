import { describe, it, expect, vi, beforeEach } from 'vitest';
import { apiClient } from '$api';
import type { KeycodeListResponse, CategoryListResponse } from '$api/types';

// Mock the API client
vi.mock('$api', () => ({
	apiClient: {
		listKeycodes: vi.fn(),
		listCategories: vi.fn()
	}
}));

describe('KeycodePicker API Integration', () => {
	const mockCategories: CategoryListResponse = {
		categories: [
			{ id: 'basic', name: 'Basic', description: 'Basic keys' },
			{ id: 'modifier', name: 'Modifier', description: 'Modifier keys' },
			{ id: 'layer', name: 'Layer', description: 'Layer switching' }
		]
	};

	const mockKeycodes: KeycodeListResponse = {
		keycodes: [
			{ code: 'KC_A', name: 'A', category: 'basic', description: 'Letter A' },
			{ code: 'KC_B', name: 'B', category: 'basic', description: 'Letter B' },
			{ code: 'KC_LCTL', name: 'Left Control', category: 'modifier', description: 'Left Control key' },
			{ code: 'MO(1)', name: 'Momentary Layer 1', category: 'layer', description: 'Hold for layer 1' }
		],
		total: 4
	};

	beforeEach(() => {
		vi.clearAllMocks();
		vi.mocked(apiClient.listCategories).mockResolvedValue(mockCategories);
		vi.mocked(apiClient.listKeycodes).mockResolvedValue(mockKeycodes);
	});

	it('calls listCategories API', async () => {
		const result = await apiClient.listCategories();
		expect(result).toEqual(mockCategories);
		expect(apiClient.listCategories).toHaveBeenCalledTimes(1);
	});

	it('calls listKeycodes API with search parameter', async () => {
		const result = await apiClient.listKeycodes('control', undefined);
		expect(result).toEqual(mockKeycodes);
		expect(apiClient.listKeycodes).toHaveBeenCalledWith('control', undefined);
	});

	it('calls listKeycodes API with category parameter', async () => {
		const result = await apiClient.listKeycodes(undefined, 'modifier');
		expect(result).toEqual(mockKeycodes);
		expect(apiClient.listKeycodes).toHaveBeenCalledWith(undefined, 'modifier');
	});

	it('calls listKeycodes API with both parameters', async () => {
		const result = await apiClient.listKeycodes('ctrl', 'modifier');
		expect(result).toEqual(mockKeycodes);
		expect(apiClient.listKeycodes).toHaveBeenCalledWith('ctrl', 'modifier');
	});

	it('handles API errors gracefully', async () => {
		vi.mocked(apiClient.listKeycodes).mockRejectedValue(new Error('Failed to load'));
		
		await expect(apiClient.listKeycodes()).rejects.toThrow('Failed to load');
	});

	it('returns categories with expected structure', async () => {
		const result = await apiClient.listCategories();
		
		expect(result.categories).toHaveLength(3);
		expect(result.categories[0]).toHaveProperty('id');
		expect(result.categories[0]).toHaveProperty('name');
		expect(result.categories[0]).toHaveProperty('description');
	});

	it('returns keycodes with expected structure', async () => {
		const result = await apiClient.listKeycodes();
		
		expect(result.keycodes).toHaveLength(4);
		expect(result.keycodes[0]).toHaveProperty('code');
		expect(result.keycodes[0]).toHaveProperty('name');
		expect(result.keycodes[0]).toHaveProperty('category');
		expect(result.keycodes[0]).toHaveProperty('description');
		expect(result.total).toBe(4);
	});

	it('filters keycodes by category', async () => {
		const filteredKeycodes: KeycodeListResponse = {
			keycodes: [
				{ code: 'KC_LCTL', name: 'Left Control', category: 'modifier', description: 'Left Control key' }
			],
			total: 1
		};
		
		vi.mocked(apiClient.listKeycodes).mockResolvedValue(filteredKeycodes);
		
		const result = await apiClient.listKeycodes(undefined, 'modifier');
		
		expect(result.keycodes).toHaveLength(1);
		expect(result.keycodes[0].category).toBe('modifier');
	});

	it('filters keycodes by search term', async () => {
		const searchResults: KeycodeListResponse = {
			keycodes: [
				{ code: 'KC_LCTL', name: 'Left Control', category: 'modifier', description: 'Left Control key' }
			],
			total: 1
		};
		
		vi.mocked(apiClient.listKeycodes).mockResolvedValue(searchResults);
		
		const result = await apiClient.listKeycodes('control', undefined);
		
		expect(result.keycodes).toHaveLength(1);
		expect(result.keycodes[0].name).toContain('Control');
	});

	it('includes quick-filter categories used by picker UI', async () => {
		const result = await apiClient.listCategories();
		const ids = result.categories.map((category) => category.id);

		expect(ids).toContain('basic');
		expect(ids).toContain('modifier');
		expect(ids).toContain('layer');
	});
});
