import path from "path"
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// The build version is derived at runtime from the SHA-256 of the served
// index.html (Vite already cache-busts asset filenames into it, so any
// source change yields a different document hash). The server injects
// `<meta name="app-version">` into the served HTML and exposes the same
// hash via /api/v1/version — no Vite-side env var to keep in sync.

// Target ex server for the dev proxy.
// Set EX_SERVER in frontend/.env.local to point at a remote instance.
// Example: EX_SERVER=https://chat.example.com
const serverTarget = process.env.EX_SERVER ?? 'http://localhost:8080';

// https://vite.dev/config/
export default defineConfig({
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
})
