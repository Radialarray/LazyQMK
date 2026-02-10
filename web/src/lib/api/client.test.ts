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

	describe('startBuild', () => {
		it('starts a build job', async () => {
			const mockResponse = {
				job: {
					id: 'job-123',
					status: 'pending',
					layout_filename: 'test.md',
					keyboard: 'crkbd',
					keymap: 'default',
					created_at: '2024-01-01T00:00:00Z',
					progress: 0
				}
			};
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			const result = await client.startBuild('test.md');
			expect(result).toEqual(mockResponse);
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/build/start',
				expect.objectContaining({
					method: 'POST',
					body: JSON.stringify({ layout_filename: 'test.md' })
				})
			);
		});
	});

	describe('listBuildJobs', () => {
		it('fetches all build jobs', async () => {
			const mockJobs = [
				{
					id: 'job-1',
					status: 'completed',
					layout_filename: 'test.md',
					keyboard: 'crkbd',
					keymap: 'default',
					created_at: '2024-01-01T00:00:00Z',
					progress: 100
				}
			];
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockJobs
			});

			const result = await client.listBuildJobs();
			expect(result).toEqual(mockJobs);
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/build/jobs',
				expect.any(Object)
			);
		});
	});

	describe('getBuildJob', () => {
		it('fetches a specific build job', async () => {
			const mockResponse = {
				job: {
					id: 'job-123',
					status: 'running',
					layout_filename: 'test.md',
					keyboard: 'crkbd',
					keymap: 'default',
					created_at: '2024-01-01T00:00:00Z',
					progress: 50
				}
			};
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			const result = await client.getBuildJob('job-123');
			expect(result).toEqual(mockResponse);
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/build/jobs/job-123',
				expect.any(Object)
			);
		});

		it('encodes job ID in URL', async () => {
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => ({ job: {} })
			});

			await client.getBuildJob('job with spaces');
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/build/jobs/job%20with%20spaces',
				expect.any(Object)
			);
		});
	});

	describe('getBuildLogs', () => {
		it('fetches build logs with default pagination', async () => {
			const mockResponse = {
				job_id: 'job-123',
				logs: [
					{ timestamp: '2024-01-01T00:00:00Z', level: 'INFO', message: 'Build started' }
				],
				has_more: false
			};
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			const result = await client.getBuildLogs('job-123');
			expect(result).toEqual(mockResponse);
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/build/jobs/job-123/logs?offset=0&limit=100',
				expect.any(Object)
			);
		});

		it('fetches build logs with custom pagination', async () => {
			const mockResponse = { job_id: 'job-123', logs: [], has_more: true };
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			await client.getBuildLogs('job-123', 50, 25);
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/build/jobs/job-123/logs?offset=50&limit=25',
				expect.any(Object)
			);
		});
	});

	describe('cancelBuild', () => {
		it('cancels a build job', async () => {
			const mockResponse = { success: true, message: 'Build cancelled' };
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			const result = await client.cancelBuild('job-123');
			expect(result).toEqual(mockResponse);
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/build/jobs/job-123/cancel',
				expect.objectContaining({
					method: 'POST'
				})
			);
		});

		it('handles cancellation failure', async () => {
			const mockResponse = { success: false, message: 'Cannot cancel completed job' };
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			const result = await client.cancelBuild('job-123');
			expect(result.success).toBe(false);
		});
	});

	// Generate Job Tests
	describe('listGenerateJobs', () => {
		it('fetches all generate jobs', async () => {
			const mockJobs = [
				{
					id: 'gen-1',
					status: 'completed',
					layout_filename: 'test.md',
					keyboard: 'crkbd',
					layout_variant: 'LAYOUT_split_3x6_3',
					created_at: '2024-01-01T00:00:00Z',
					progress: 100
				}
			];
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockJobs
			});

			const result = await client.listGenerateJobs();
			expect(result).toEqual(mockJobs);
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/generate/jobs',
				expect.any(Object)
			);
		});
	});

	describe('getGenerateJob', () => {
		it('fetches a specific generate job', async () => {
			const mockResponse = {
				job: {
					id: 'gen-123',
					status: 'running',
					layout_filename: 'test.md',
					keyboard: 'crkbd',
					layout_variant: 'LAYOUT_split_3x6_3',
					created_at: '2024-01-01T00:00:00Z',
					progress: 50
				}
			};
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			const result = await client.getGenerateJob('gen-123');
			expect(result).toEqual(mockResponse);
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/generate/jobs/gen-123',
				expect.any(Object)
			);
		});

		it('encodes job ID in URL', async () => {
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => ({ job: {} })
			});

			await client.getGenerateJob('job with spaces');
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/generate/jobs/job%20with%20spaces',
				expect.any(Object)
			);
		});
	});

	describe('getGenerateLogs', () => {
		it('fetches generate logs with default pagination', async () => {
			const mockResponse = {
				job_id: 'gen-123',
				logs: [
					{ timestamp: '2024-01-01T00:00:00Z', level: 'INFO', message: 'Generation started' }
				],
				has_more: false
			};
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			const result = await client.getGenerateLogs('gen-123');
			expect(result).toEqual(mockResponse);
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/generate/jobs/gen-123/logs?offset=0&limit=100',
				expect.any(Object)
			);
		});

		it('fetches generate logs with custom pagination', async () => {
			const mockResponse = { job_id: 'gen-123', logs: [], has_more: true };
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			await client.getGenerateLogs('gen-123', 50, 25);
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/generate/jobs/gen-123/logs?offset=50&limit=25',
				expect.any(Object)
			);
		});
	});

	describe('cancelGenerate', () => {
		it('cancels a generate job', async () => {
			const mockResponse = { success: true, message: 'Generation cancelled' };
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			const result = await client.cancelGenerate('gen-123');
			expect(result).toEqual(mockResponse);
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/generate/jobs/gen-123/cancel',
				expect.objectContaining({
					method: 'POST'
				})
			);
		});

		it('handles cancellation failure', async () => {
			const mockResponse = { success: false, message: 'Cannot cancel completed job' };
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			const result = await client.cancelGenerate('gen-123');
			expect(result.success).toBe(false);
		});
	});

	describe('getGenerateDownloadUrl', () => {
		it('returns the correct download URL', () => {
			const url = client.getGenerateDownloadUrl('gen-123');
			expect(url).toBe('http://localhost:3000/api/generate/jobs/gen-123/download');
		});

		it('encodes job ID in URL', () => {
			const url = client.getGenerateDownloadUrl('job with spaces');
			expect(url).toBe('http://localhost:3000/api/generate/jobs/job%20with%20spaces/download');
		});
	});

	// Build Artifacts Tests
	describe('getBuildArtifacts', () => {
		it('fetches build artifacts', async () => {
			const mockResponse = {
				job_id: 'job-123',
				artifacts: [
					{
						filename: 'firmware.hex',
						artifact_type: 'hex',
						size: 12345,
						hash: 'abc123',
						created_at: '2024-01-01T00:00:00Z'
					}
				]
			};
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			const result = await client.getBuildArtifacts('job-123');
			expect(result).toEqual(mockResponse);
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/build/jobs/job-123/artifacts',
				expect.any(Object)
			);
		});

		it('encodes job ID in URL', async () => {
			const mockResponse = { job_id: 'job', artifacts: [] };
			(global.fetch as any).mockResolvedValueOnce({
				ok: true,
				json: async () => mockResponse
			});

			await client.getBuildArtifacts('job with spaces');
			expect(global.fetch).toHaveBeenCalledWith(
				'http://localhost:3000/api/build/jobs/job%20with%20spaces/artifacts',
				expect.any(Object)
			);
		});
	});

	describe('getBuildArtifactDownloadUrl', () => {
		it('returns the correct download URL', () => {
			const url = client.getBuildArtifactDownloadUrl('job-123', 'firmware.hex');
			expect(url).toBe('http://localhost:3000/api/build/jobs/job-123/artifacts/firmware.hex/download');
		});

		it('encodes job ID and filename in URL', () => {
			const url = client.getBuildArtifactDownloadUrl('job with spaces', 'my firmware.hex');
			expect(url).toBe('http://localhost:3000/api/build/jobs/job%20with%20spaces/artifacts/my%20firmware.hex/download');
		});
	});
});
