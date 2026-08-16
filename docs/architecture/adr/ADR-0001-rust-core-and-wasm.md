# ADR-0001: Rust Core Compiled to WebAssembly

**Status:** Accepted
**Date:** 2026-08-16

## Context

NIKHIL//OS is a Unix-inspired simulated operating system that must run
headless (native tests), in the browser (Web Edition), and eventually on
native desktop (Tauri). The presentation layer must be a thin surface over
genuine, observable system state — not a set of UI screens with fake
telemetry.

The choice of implementation language for the shared core determines whether
that promise is credible, and how much effort each edition costs.

## Decision

Implement the shared core as a Rust library crate (`nikhil-os-core`) compiled
to WebAssembly for the browser and to native code for tests and the future
Tauri desktop shell.

## Alternatives Considered

### Option A — TypeScript core

Pros: zero toolchain friction for contributors; fast iteration; same language
as the UI.

Cons: does not credibly signal "systems engineering" to the target audience
(systems engineers, students); a simulated OS in the UI language invites
treating the OS as a UI concern, which is exactly the trap the project exists
to avoid.

### Option B — Hybrid: TypeScript now, Rust later

Pros: fastest to a first demo.

Cons: a later rewrite would re-derive the interfaces from a TS implementation,
costing the claimed architecture; "Rust later" projects rarely migrate.

## Rationale

Rust provides strong type safety, explicit ownership, predictable
performance, first-class WASM support, and native integration — the exact
properties the architecture document calls for. Compiling the *same* library
to WASM and to native means the browser OS and the future desktop OS cannot
drift apart, and the core remains usable without any UI framework.

## Consequences

### Positive

- Single core serves Web, Desktop, and headless tests.
- Systems-programming credibility is real, not decorative.
- Strong types make IPC and syscall boundaries hard to violate.

### Negative

- Higher contributor barrier (Rust + `wasm32-unknown-unknown` toolchain).
- Slower initial progress than a TypeScript-first approach.

### Risks

- Contributors may be unfamiliar with Rust; mitigated by keeping subsystems
  small and documented.

## Validation

Web Alpha boots in the browser with real core state; `cargo test` covers the
core headlessly; the same crate compiles for both targets in CI.

## Related Components

- `crates/core`
- `web/` (WASM bridge)

## Follow-up

- Revisit the WASM build tooling (wasm-bindgen vs. growing needs) in the
  Developer Mode phase.
