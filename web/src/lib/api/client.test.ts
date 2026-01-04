import { describe, it, expect, beforeEach, vi } from 'vitest';
import { ApiClient } from './client';

// Mock fetch
global.fetch = vi.fn();

describe('ApiClient', () => {
	let client: ApiClient;

	beforeEach(() => {
		client = new ApiClient('http://localhost:3000');
		vi.clearAllMocks();
	});

	describe('health', () => {
		it('fetches health status', async () => {
			const mockResponse = { status: 'healthy', version: '0.12.0' };
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			const result = await client.health();
			expect(result).toEqual(mockResponse);
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/health',
				expect.objectContaining({
					headers: expect.objectContaining({
						'Content-Type': 'application/json'
					})
				})
			);
		});
	});

	describe('listLayouts', () => {
		it('fetches layout list', async () => {
			const mockResponse = {
				layouts: [
					{
						filename: 'test.md',
						name: 'Test Layout',
						description: 'A test',
						modified: '2024-01-01T00:00:00Z'
					}
				]
			};
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			const result = await client.listLayouts();
			expect(result).toEqual(mockResponse);
		});
	});

	describe('getLayout', () => {
		it('fetches a specific layout', async () => {
			const mockLayout = {
				metadata: {
					name: 'Test',
					description: 'Test layout',
					author: 'Test User',
					keyboard: 'crkbd',
					layout: 'LAYOUT_split_3x6_3',
					created: '2024-01-01T00:00:00Z',
					modified: '2024-01-01T00:00:00Z'
				},
				layers: []
			};
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockLayout
			});

			const result = await client.getLayout('test.md');
			expect(result).toEqual(mockLayout);
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/layouts/test.md',
				expect.any(Object)
			);
		});

		it('encodes filename in URL', async () => {
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => ({})
			});

			await client.getLayout('my layout.md');
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/layouts/my%20layout.md',
				expect.any(Object)
			);
		});
	});

	describe('error handling', () => {
		it('throws error on HTTP error', async () => {
			(global.fetch as any).mockResolvedValueOnce({
				ok: false,
				status: 404,
				json: async () => ({ error: 'Not found' })
			});

			await expect(client.health()).rejects.toThrow('Not found');
		});

		it('handles malformed error responses', async () => {
			(global.fetch as any).mockResolvedValueOnce({
				ok: false,
				status: 500,
				statusText: 'Internal Server Error',
				json: async () => {
					throw new Error('Invalid JSON');
				}
			});

			await expect(client.health()).rejects.toThrow('HTTP 500');
		});
	});

	describe('listKeycodes', () => {
		it('fetches keycodes without filters', async () => {
			const mockResponse = { keycodes: [], total: 0 };
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			await client.listKeycodes();
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/keycodes',
				expect.any(Object)
			);
		});

		it('fetches keycodes with search filter', async () => {
			const mockResponse = { keycodes: [], total: 0 };
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			await client.listKeycodes('KC_A');
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/keycodes?search=KC_A',
				expect.any(Object)
			);
		});

		it('fetches keycodes with category filter', async () => {
			const mockResponse = { keycodes: [], total: 0 };
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			await client.listKeycodes(undefined, 'basic');
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/keycodes?category=basic',
				expect.any(Object)
			);
		});
	});
});
