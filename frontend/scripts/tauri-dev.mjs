import { spawn } from 'node:child_process';

const port = Number(process.env.EX_DESKTOP_DEV_PORT ?? process.env.PORT ?? 1430);
const host = process.env.HOST ?? '127.0.0.1';
const viteClientUrl = `http://${host}:${port}/@vite/client`;
const indexUrl = `http://${host}:${port}/`;
const bootstrapMarker = 'name="ex-desktop-bootstrap" content="wrapper-v1"';

async function hasReusableViteServer() {
  try {
    const [viteClientResponse, indexResponse] = await Promise.all([
      fetch(viteClientUrl, {
        signal: AbortSignal.timeout(1500),
      }),
      fetch(indexUrl, {
        signal: AbortSignal.timeout(1500),
      }),
    ]);

    if (!viteClientResponse.ok) {
      throw new Error(
        `Port ${port} is already in use, but ${viteClientUrl} returned status ${viteClientResponse.status}.`,
      );
    }

    if (!indexResponse.ok) {
      throw new Error(
        `Port ${port} is already in use, but ${indexUrl} returned status ${indexResponse.status}.`,
      );
    }

    const indexHtml = await indexResponse.text();
    if (!indexHtml.includes(bootstrapMarker)) {
      throw new Error(
        `Port ${port} is serving a different app. Stop the existing dev server on ${host}:${port} and rerun make dev.`,
      );
    }

    return true;
  } catch (error) {
    if (error instanceof Error) {
      const cause = /** @type {{ code?: string } | undefined} */ (error.cause);
      if (cause?.code === 'ECONNREFUSED' || cause?.code === 'UND_ERR_CONNECT_TIMEOUT') {
        return false;
      }
      if (error.name === 'TimeoutError') {
        throw new Error(
          `Port ${port} is open but did not respond as the ex desktop bootstrap server in time.`,
        );
      }
    }
    if (error instanceof TypeError) {
      return false;
    }
    throw error;
  }
}

if (await hasReusableViteServer()) {
  console.log(`Reusing existing Vite dev server on ${viteClientUrl}`);
  process.exit(0);
}

const child = spawn(
  process.platform === 'win32' ? 'npm.cmd' : 'npm',
  ['run', 'dev', '--', '--host', host, '--strictPort'],
  {
    cwd: new URL('..', import.meta.url),
    stdio: 'inherit',
  },
);

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 1);
});
