# ADR-0003: Canonical Profile Data Embedded in the Core

**Status:** Accepted
**Date:** 2026-08-16

## Context

Every application — Projects, Resume, Recruiter Mode, and later the knowledge
graph and RAG — reads the same profile facts (projects, skills, experience,
evidence). If data is duplicated across UI components, the system will
silently diverge and the "evidence engine" promise breaks. Separately, the
project is open-sourced so others can use it for their own portfolio: the
data must be replaceable without touching application logic.

## Decision

Keep a single canonical profile dataset in `knowledge/data/` (versioned,
human-readable JSON), embedded into the Rust core at build time via
`include_str!` and exposed through a typed `KnowledgeService`. The UI reads
profile facts only through that service boundary.

## Alternatives Considered

### Option A — Data in the frontend (TypeScript modules)

Pros: simplest to edit.

Cons: no single source of truth; core cannot validate or serve it; the
"system underneath" does not own the knowledge it exposes.

### Option B — Data in a database

Pros: mutable, queryable.

Cons: heavy for a zero-install static Web Edition; adds a backend dependency
that the alpha explicitly avoids.

## Rationale

Embedding the canonical data in the core keeps one source of truth, lets the
core validate relationships and serve evidence-backed answers, and keeps the
Web Alpha static and zero-install. Forks replace one directory — and the
schema in `knowledge/schema/` documents exactly what is expected.

## Consequences

### Positive

- Single source of truth for every app.
- Data and logic are separable; forking is a data swap, not a rewrite.
- The core remains self-contained and testable without a UI.

### Negative

- Changing profile data requires a rebuild (acceptable for a portfolio whose
  data changes rarely).
- No live editing.

### Risks

- Users want a JSON-at-runtime override; deferred to the AI/backend phase.

## Validation

Projects, Resume, and Recruiter Mode render identical facts from one dataset;
a test asserts every application's data comes from the service.

## Related Components

- `knowledge/`, `crates/core` (KnowledgeService), `web/` apps

## Follow-up

- Runtime data loading if the backend phase introduces one.
