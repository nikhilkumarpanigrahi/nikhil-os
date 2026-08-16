# ADR-0002: Web Alpha as First Public Milestone

**Status:** Accepted
**Date:** 2026-08-16

## Context

The full vision spans a Web Edition, a Tauri Desktop Edition, and a future
bare-metal kernel — an estimated 4–6 months of work. The roadmap warns: "Do
not wait months for a perfect release. Build a useful Web Alpha early and
progressively deepen the system." The first public milestone must be real,
useful, and shippable, while leaving every later phase (AI runtime, ML,
desktop, kernel) structurally open.

## Decision

Ship a complete **Web Alpha** first: boot sequence, desktop, window manager,
command palette, Terminal + `nish` shell over the real WASM core, Files,
Projects, Resume, Recruiter Mode, and System Monitor — all driven by genuine
core state and canonical profile data. AI runtime, ML, Tauri desktop, backend,
and kernel are represented as documented placeholders to be deepened in
subsequent phases.

## Alternatives Considered

### Option A — Full AI runtime in the first build

Pros: most impressive immediately.

Cons: Rust core + WASM is already the heaviest option; adding RAG,
embeddings, and the knowledge graph on top would push any public release out
by months and risk shipping nothing.

### Option B — Scaffold plus proof of concept only

Pros: smallest possible first step.

Cons: under-delivers on the goal of a usable, forkable system; a POC does not
demonstrate the product to recruiters or contributors.

## Rationale

A working Web Alpha is immediately demoable as a portfolio, immediately
useful to a visitor, and immediately forkable. It matches the documented
"Week 8 — Public Web Alpha" milestone, and each deferred system (AI runtime,
ML, desktop, kernel) has its own roadmap phase and ADR hook.

## Consequences

### Positive

- A public, usable artifact exists quickly.
- Feedback loop on the core architecture starts early.

### Negative

- The AI/knowledge-graph experience that makes the project distinctive is not
  yet visible.

### Risks

- Scope creep back into the AI phase; guarded by the roadmap's phase
  boundaries.

## Validation

The GitHub Pages deployment of `main` serves a bootable Web Edition whose
terminal, filesystem, and telemetry are driven by the Rust core.

## Related Components

- `web/`, `crates/core`, `knowledge/`

## Follow-up

- Phase 5 (AI Core) and Phase 6 (ML) as defined in `docs/06-ROADMAP.md`.
