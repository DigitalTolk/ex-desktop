import path from "path"
import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  // loadEnv reads .env, .env.local, .env.[mode], .env.[mode].local — the same
  // files Vite loads for import.meta.env, but made available to the config.
  // process.env is checked as a fallback for CI / shell-level overrides.
  const env = loadEnv(mode, process.cwd(), '');
  const serverTarget = env.EX_SERVER ?? process.env.EX_SERVER ?? 'http://localhost:8080';

  return {
    plugins: [react(), tailwindcss()],
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "./src"),
      },
    },
    // Required for Tauri: keep Vite output visible alongside Tauri logs and
    // pin the port so tauri.conf.json devUrl stays in sync.
    clearScreen: false,
    server: {
      strictPort: true,
      proxy: {
        '/api': {
          target: serverTarget,
          changeOrigin: true,
          ws: true,
        },
        '/auth': {
          target: serverTarget,
          changeOrigin: true,
        },
      },
    },
  };
})
