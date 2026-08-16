import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import { configDefaults } from "vitest/config";
import wasm from "vite-plugin-wasm";

// The WASM core ships as a bundler-target wasm-bindgen package in `./pkg`.
// vite-plugin-wasm lets us import the generated .wasm module directly.
// (The new wasm-bindgen bundler output is synchronous — no top-level await.)
export default defineConfig({
  plugins: [react(), wasm()],
  // Relative base: the app must work from any subpath (GitHub Pages serves
  // it under /<repo>/), not just the domain root.
  base: "./",
  server: {
    // In dev, VITE_API_URL is unset so the Contact app calls `/api/...`.
    // Proxy those calls to the local Rust backend — no CORS, no config.
    proxy: {
      "/api": {
        target: "http://localhost:8080",
        changeOrigin: true,
      },
      "/admin": {
        target: "http://localhost:8080",
        changeOrigin: true,
      },
      "/healthz": {
        target: "http://localhost:8080",
        changeOrigin: true,
      },
    },
  },
  build: {
    target: "esnext",
    assetsInlineLimit: 1024 * 1024, // keep the .wasm inline for single-file deploys
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    css: true,
    // Playwright specs live in ./e2e and must not run under vitest.
    exclude: [...configDefaults.exclude, "e2e/**"],
  },
});
