import "@testing-library/jest-dom/vitest";

// Global vitest setup. The WASM core cannot run inside jsdom (Node can't
// parse .wasm), so individual test files mock `src/core/wasm` (or the pkg
// module itself) with the stubs they need. Real-core coverage lives in the
// Rust test suite; these tests cover the UI logic around it.
