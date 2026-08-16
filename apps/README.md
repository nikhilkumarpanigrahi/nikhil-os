# Applications

This directory is the **application registry** — machine-readable metadata for every
app the OS knows how to launch. In the Web Alpha the registry lives in
[`web/src/apps/registry.tsx`](../web/src/apps/registry.tsx); this directory is the
declarative home for richer manifests (icons, dependencies, permissions, sizes).

## Current Web Alpha apps

| App | Description | Driven by |
| --- | --- | --- |
| Terminal | `nish` shell over the real WASM core | core `run_command` / `autocomplete` |
| Files | browse the live VFS (`/proc`, `/sys` are real) | core `list_dir` / `read_file` |
| Projects | portfolio project cards with architecture + evidence | `profile()` |
| Resume | experience / education / skills timeline | `profile()` |
| Recruiter | concise candidate view for hiring managers | `profile()` |
| System Monitor | live process / memory / service telemetry | core `snapshot()` |
| Welcome | getting-started splash | static |

## Planned

- **AI Core** — chat over the knowledge graph (phase 5–6)
- **Knowledge / Graph** — browse the profile graph visually
- **Lab** — in-browser experiments, WASM demos
- **Packages** — `pkgctl` UI over the package registry

To add an app, implement it in `web/src/apps/`, register it in `registry.tsx`, and add
a manifest here.
