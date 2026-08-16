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
