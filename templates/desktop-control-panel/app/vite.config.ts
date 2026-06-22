import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
// Vitest types are imported via the triple-slash reference in tsconfig when
// present; the `test` block is only read by vitest, not by the Vite dev server.
/// <reference types="vitest" />

// Tauri sets TAURI_DEV_HOST when developing on a remote/networked target.
declare const process: { env: Record<string, string | undefined> };
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  // Vitest configuration — applies only during `vitest run` / `vitest watch`.
  // jsdom provides a browser-like DOM so component render tests work without
  // a real browser. The existing store + IPC tests (no DOM use) run fine under
  // jsdom because it doesn't break non-DOM code.
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test-setup.ts"],
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    target: "es2020",
    minify: "esbuild",
    sourcemap: true,
  },
  assetsInclude: ["**/*.txt"],
});
