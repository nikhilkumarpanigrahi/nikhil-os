# ADR-0004: Production Backend in Rust (axum) for the Web Edition

**Status:** Accepted
**Date:** 2026-08-16

## Context

The Web Alpha is a fully self-contained static site (the WASM core runs the
"OS" in the browser tab) — that was the deliberate scope of
[ADR-0002](ADR-0002-web-alpha-scope.md). It is now time to turn the OS from a
portfolio into an application that is *useful to visitors* and *valuable to
the owner*:

- **Visitors** want to message Nikhil *from inside the OS* — no downloads, no
  third-party forms, and (per the offline-first contract in
  `docs/02-ARCHITECTURE.md`) the static site must keep working even when the
  service is unreachable.
- **Nikhil** wants a real inbox with instant alerts, and a knowledge API so
  the OS is backed by a live service rather than only embedded data.
- The deployment target is the **AWS free tier** — year-one cost target is $0.

The architecture spec (`docs/02-ARCHITECTURE.md`) had suggested a Python
backend; the `backend/README.md` placeholder named "Rust service (axum)". The
project's identity is **full-stack Rust** (the OS core is Rust compiled to
WASM), and the owner explicitly chose Rust. This ADR resolves the conflict in
favor of Rust and records the full stack decision.

## Decision

Build the backend as a **Rust + axum modular monolith** with PostgreSQL, deploy
it as three containers (Postgres + API + Caddy) on a free-tier EC2 instance,
and expose three surfaces:

| Surface | Purpose |
|---|---|
| `POST /api/v1/contact` | Visitor messaging → stored, rate-limited, owner notified |
| `/admin` + JWT API | Nikhil's inbox panel: list, triage (new/read/replied/archived), stats |
| `/api/v1/{profile,projects,skills,experience,claims}` | Live knowledge API serving the *same* `knowledge/data/profile.json` the WASM core embeds |

Stack, pinned:

- **axum 0.8** on **tokio 1** (full), **tower-http 0.6** middleware stack
- **sqlx 0.8** with compile-time checked queries and a committed offline cache
  (`.sqlx/`) so builds never touch a live database
- **governor** for per-IP rate limiting; **argon2** password hashes +
  **jsonwebtoken** HS256 15-minute admin sessions
- **PostgreSQL 16** (container, pgvector-ready for the AI phase) as canonical
  truth for messages and the audit trail
- **Telegram bot** for instant alerts (free), provider-abstracted behind a
  `NotificationSender` trait with a structured-log fallback
- **Caddy** for automatic HTTPS (Let's Encrypt) in front of the API
- **Docker Compose** on a free-tier `t2.micro`/`t3.micro` (1 GB RAM — sized
  down, ~1 GB swap added at provisioning)
- **GitHub Actions**: `backend-ci` (fmt/clippy/test against a Postgres service
  container/docker build) and `backend-deploy` (SSH → `git pull` →
  `docker compose up -d --build` → health poll)

The modular-monolith shape (flat `routes/` + `services/` modules over one
binary) keeps the seams ADR-0003's knowledge boundary already established, and
leaves room for the documented AI/RAG phase without a rewrite.

## Alternatives Considered

### Python + FastAPI (the architecture doc's earlier suggestion)

Pros: fast to write; familiar.

Cons: a second language in a full-stack-Rust project; no compile-time SQL
checks; weaker "recruiter signal" for this owner; deployment and typing
discipline are harder to keep production-grade.

### Node + Express/Fastify

Pros: huge ecosystem.

Cons: dynamic typing; concurrency model mismatch with the Rust core; weaker
per-request performance story for the "system underneath" branding.

### Go + chi/gin

Pros: excellent concurrency, simple deploys.

Cons: another language; the owner's explicit choice was Rust, and the core
already demonstrates the Rust pedigree.

### Serverless (Lambda + API Gateway + RDS/Postgres)

Pros: scales to zero, no box to manage.

Cons: not free in spirit (API Gateway/RDS costs mount beyond the free tier);
cold starts; PostgreSQL cost; vendor lock-in contradicts the open-source
"run it yourself" story; harder to run the full stack locally.

## Rationale

- **One language, one story.** The OS core, its tooling, and now the backend
  are Rust — the project reads as a deliberate, coherent engineering artifact,
  which is the strongest recruiter signal available to a portfolio.
- **Compile-time guarantees where they matter.** sqlx verifies every query
  against the real schema at build time; axum gives typed extractors and
  exhaustive routing. Both are checked by CI before anything deploys.
- **Security defaults.** argon2 for passwords, short-lived JWTs, per-IP rate
  limits, a honeypot on the public form, and an error type that never leaks
  internals — appropriate for a service strangers can hit.
- **Operationally boring.** One box, three containers, automatic TLS, and a
  deploy that is a `git pull` + `docker compose up`. Boring is a feature when
  the operator is one person with a free-tier budget.
- **Honest cost model.** Year one ≈ $0. The free tier expires after 12 months
  (~$7–8/mo for t3.micro thereafter), and every piece is documented in
  `backend/README.md` so the migration off free tier is a config change, not a
  rewrite.

## Consequences

- The web app gains its **first network call**, but the offline-first contract
  holds: the Contact app degrades to a `mailto:` fallback and the rest of the
  OS is untouched when the API is down.
- `knowledge/data/profile.json` is now compiled into **two** consumers (WASM
  core and backend). The risk of drift is mitigated by keeping one file, a
  canonical-shape test on both sides, and an ETag over the file content.
- The backend carries its own lockfile and workspace so the WASM core's build
  stays fast and independent.
- Contributors must run `cargo sqlx prepare -- --all-targets` after any SQL
  change and commit the cache — enforced implicitly by CI (offline builds fail
  loudly on an uncached query).

## Status

Accepted. Supersedes the "Python backend" implication in
`docs/02-ARCHITECTURE.md`; implemented in `backend/`.
