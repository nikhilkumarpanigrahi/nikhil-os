import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";

// The WASM core ships as a bundler-target wasm-bindgen package in `./pkg`.
// vite-plugin-wasm lets us import the generated .wasm module directly.
// (The new wasm-bindgen bundler output is synchronous — no top-level await.)
export default defineConfig({
  plugins: [react(), wasm()],
  build: {
    target: "esnext",
    assetsInlineLimit: 1024 * 1024, // keep the .wasm inline for single-file deploys
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    css: true,
  },
});
