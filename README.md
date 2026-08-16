# NIKHIL//OS

> **An AI-native personal computing environment.**

NIKHIL//OS turns a personal portfolio into an interactive, Unix-inspired
operating system. Visitors don't scroll a website — they **use** the system:
boot it, open a terminal, browse a virtual filesystem, inspect real process
and memory telemetry, and explore projects, experience, and evidence through
native applications.

It is built the way a real operating system is built — a shared Rust core
(compiled to WebAssembly for the browser), a typed IPC and syscall layer, a
service manager, a package manager, and a shell with real pipelines — with
the UI as a thin surface over genuinely observable state. **No telemetry is
faked.**

```
Presentation
    ↓
Application Runtime
    ↓
OS Services
    ↓
Rust Core (WASM)
```

## Editions

| Edition   | Status              | Description                                   |
| --------- | ------------------- | --------------------------------------------- |
| **Web**   | Alpha (this repo)   | No installation — runs entirely in the browser |
| Desktop   | Planned             | Native app via Tauri (macOS / Windows / Linux) |
| Kernel    | Planned             | Future bare-metal x86_64 systems project       |

## Try it

The Web Edition is deployed from `main` to GitHub Pages.

```text
https://nikhilkumarpanigrahi.github.io/nikhil-os/
```

> The profile data in `knowledge/data/` is Nikhil's own. To make NIKHIL//OS
> your own, fork the repo and replace that directory — see
> [docs/contributions/fork-your-own-os.md](docs/contributions/fork-your-own-os.md).

## What's inside

- **Rust shared core** (`crates/core`) — process manager, round-robin
  scheduler, virtual filesystem (with live `/proc` and `/sys`), memory
  simulation, permissions, typed IPC, syscalls, service manager, `pkgctl`
  package manager, event bus, structured logging.
- **`nish` shell** — lexer → parser → AST → executor. Pipes, redirection,
  environment variables, aliases, history, autocomplete, exit codes.
  `ps | grep ai` runs through a real pipeline.
- **Web Edition** (`web/`) — React + TypeScript desktop environment: landing
  page, real boot sequence, window manager, `Cmd/Ctrl + K` command palette,
  keyboard navigation, and applications (Terminal, Files, Projects, Resume,
  Recruiter Mode, System Monitor).
- **Knowledge core** (`knowledge/`) — the canonical entity/relationship schema
  and profile data backing every application.
- **Design system** — dark-first, restrained, high information density,
  inspired by modern developer tools.

## Quick start

Prerequisites: Node ≥ 20, Rust stable with the `wasm32-unknown-unknown`
target, and [wasm-pack](https://rustwasm.github.io/wasm-pack/).

```bash
# 1. Install prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# 2. Build the Rust core → WebAssembly
npm run build:core

# 3. Install and run the Web Edition
cd web
npm install
npm run dev
```

Open http://localhost:5173 and **enter the OS**.

### Testing

```bash
cargo test --all-features      # Rust core: boot, scheduler, VFS, shell pipeline
cd web && npm test             # Web UI components (Vitest + jsdom)
cd web && npm run test:e2e     # Browser smoke tests (Playwright + Chromium)
```

### Project structure

```text
crates/core/     Shared Rust OS core (compiled to WASM)
web/             Web Edition — React + TypeScript + Vite
knowledge/       Canonical schema + profile data
apps/            Application registry / manifests
desktop/         Tauri Desktop Edition (planned)
backend/         AI orchestration backend (planned)
kernel/          Bare-metal kernel (planned)
docs/            Product, architecture, ADRs, contribution guides
```

## Architecture

The system is layered: the UI never mutates kernel state directly.
Applications use services and syscalls; AI is a controlled service with
schema-validated, permission-checked tools; every subsystem emits structured
events consumed by Developer Mode.

See [docs/02-ARCHITECTURE.md](docs/02-ARCHITECTURE.md) for the full picture,
and [docs/architecture/adr/](docs/architecture/adr/) for the decision record.

## Security

- AI cannot execute arbitrary commands. Every tool action passes through
  intent → schema validation → permission validation → OS service → audit.
- Threat model covers prompt injection, malicious tool arguments, privilege
  escalation, and data exfiltration. See [SECURITY.md](SECURITY.md).

## License

MIT — see [LICENSE](LICENSE).

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) first.

---

> **The UI is the surface. The system underneath is the project.**
