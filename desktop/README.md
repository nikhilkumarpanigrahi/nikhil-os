# Desktop Edition (placeholder)

> **Status: planned.** The Web Alpha runs the same Rust core through the WASM bridge.
> This edition packages that core into a native desktop shell with **Tauri**.

## What it will do

- Run the identical `nikhil-os-core` crate — but compiled natively (`rlib`) instead of
  to WASM, for full-speed subsystem simulation.
- Provide a native window via Tauri, with the React desktop as the frontend.
- Add native extras the browser can't offer: real file access through the VFS,
  local-first AI models, system tray, offline installation.
- Support cross-platform packaging (macOS, Windows, Linux) via Tauri bundlers.

## Why Tauri

The web edition already proves the core + React stack. Tauri reuses both with a thin
Rust shell, keeping the binary small (~a few MB) and the attack surface minimal.

## Status / roadmap

- [ ] Tauri v2 scaffold (`cargo tauri init`)
- [ ] Wire the core crate as a native dependency
- [ ] Reuse `web/dist` as the packaged frontend
- [ ] Installer + code-signing pipeline

See [`docs/06-ROADMAP.md`](../docs/06-ROADMAP.md) for phase details.
