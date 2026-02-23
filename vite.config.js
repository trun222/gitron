import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;
const viteBase = process.env.VITE_BASE || '';
const isRemote = !!viteBase;
const apiPort = process.env.GITRON_API_PORT || 9417;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [sveltekit(), tailwindcss()],
  base: process.env.VITE_BASE || '/',

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: isRemote ? '0.0.0.0' : (host || false),
    allowedHosts: /** @type {true | string[]} */ (isRemote ? true : []),
    proxy: isRemote ? {
      [`${viteBase}/api`]: {
        target: `http://localhost:${apiPort}`,
        rewrite: (/** @type {string} */ path) => path.replace(new RegExp(`^${viteBase}`), ''),
      },
    } : !host ? {
      '/api': {
        target: `http://localhost:${apiPort}`,
      },
    } : undefined,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
