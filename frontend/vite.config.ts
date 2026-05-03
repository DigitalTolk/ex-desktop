import path from "path"
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

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
    port: Number(process.env.EX_DESKTOP_DEV_PORT ?? process.env.PORT ?? 1430),
    strictPort: true,
  },
})
