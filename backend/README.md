# Backend / AI Orchestration (placeholder)

> **Status: planned.** The Web Alpha is fully self-contained — the "backend" is the
> WASM core running in your browser tab. This directory will host the optional
> server-side AI orchestration layer described in the architecture spec.

## What it will do

- **AI runtime** — serve embeddings + retrieval over the knowledge graph for the
  AI Core app (a portfolio that can *answer questions about itself*).
- **Knowledge graph service** — a real graph store (e.g. Postgres + pgvector) for the
  entities in [`knowledge/`](../knowledge/).
- **Session / sync** — optional cloud sync so one profile powers web, desktop, and
  future native editions.
- **API** — the documented backend services, reached by the frontend only when a
  network connection is available (offline-first by design).

## Why it is optional

The portfolio is a static site today; a server would only add value for AI features
and sync. Keeping it separate means the Web Alpha deploys to GitHub Pages with zero
infrastructure.

## Status / roadmap

- [ ] Rust service (axum) skeleton
- [ ] Graph store + RAG pipeline
- [ ] AI chat API over profile claims

See [`docs/06-ROADMAP.md`](../docs/06-ROADMAP.md) for phase details.
