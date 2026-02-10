import { spawn, type ChildProcessWithoutNullStreams } from 'node:child_process';
import { request } from 'node:http';

type BackendProcess = ChildProcessWithoutNullStreams | null;

interface BackendOptions {
	workspaceRoot: string;
	port?: number;
	host?: string;
	configDir?: string;
}

const DEFAULT_PORT = 3001;
const DEFAULT_HOST = '127.0.0.1';

function waitForHealth(url: string, timeoutMs = 120_000): Promise<void> {
	const deadline = Date.now() + timeoutMs;

	return new Promise((resolve, reject) => {
		const attempt = () => {
			const req = request(url, (res) => {
				if (res.statusCode === 200) {
					res.resume();
					resolve();
					return;
				}
				res.resume();
				if (Date.now() > deadline) {
					reject(new Error(`Backend health check failed with status ${res.statusCode}`));
					return;
				}
				setTimeout(attempt, 500);
			});

			req.on('error', () => {
				if (Date.now() > deadline) {
					reject(new Error('Backend health check timed out'));
					return;
				}
				setTimeout(attempt, 500);
			});

			req.end();
		};

		attempt();
	});
}

export async function startBackend(
	options: BackendOptions
): Promise<{ process: BackendProcess; baseUrl: string }> {
	const port = options.port ?? DEFAULT_PORT;
	const host = options.host ?? DEFAULT_HOST;
	const args = [
		'run',
		'--features',
		'web',
		'--',
		'web',
		'--port',
		String(port),
		'--host',
		host,
		'--workspace',
		options.workspaceRoot
	];

	const env = {
		...process.env,
		LAZYQMK_CONFIG_DIR: options.configDir ?? options.workspaceRoot
	};

	const child = spawn('cargo', args, {
		cwd: '..',
		env,
		stdio: 'pipe'
	});

	child.stdout.on('data', (chunk) => process.stdout.write(chunk));
	child.stderr.on('data', (chunk) => process.stderr.write(chunk));

	const baseUrl = `http://${host}:${port}`;
	await waitForHealth(`${baseUrl}/health`);

	return { process: child, baseUrl };
}

export async function stopBackend(child: BackendProcess): Promise<void> {
	if (!child || child.killed) {
		return;
	}

	await new Promise<void>((resolve) => {
		child.once('exit', () => resolve());
		child.kill('SIGTERM');
		setTimeout(() => {
			if (!child.killed) {
				child.kill('SIGKILL');
			}
			resolve();
		}, 3000);
	});
}
