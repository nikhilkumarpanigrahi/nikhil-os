# Contributing to NIKHIL//OS

Thanks for your interest in contributing. NIKHIL//OS is an unusual project:
the UI is a surface over a real simulated operating system. Please keep the
system's principles in mind when you contribute.

## Principles

- **Real state, never faked.** Do not introduce telemetry, processes, or
  filesystem values that do not come from the Rust core.
- **UI never mutates kernel state.** Applications talk to services and
  syscalls, not to internal core structures.
- **AI is a controlled service.** No arbitrary shell, filesystem, or database
  access. Tools are allowlisted and schema-validated.
- **Observe everything.** New subsystems must emit structured events.
- **Tests are not optional.** Every subsystem ships with unit and integration
  tests. AI features require retrieval/tool/prompt-injection tests.

## Getting started

1. Fork the repository.
2. Install prerequisites (see [README](README.md#quick-start)).
3. Create a branch: `git checkout -b feat/my-change`.
4. Make changes. Follow the [engineering prompts](docs/07-ENGINEERING-PROMPTS.md)
   as a design checklist for major subsystems.
5. Add or update tests.
6. Run the checks below.
7. Open a pull request against `main`.

## Checks

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-features
npm run build:core
cd web && npm install && npm run lint && npm run typecheck && npm test && npm run build
```

## Code style

- Rust: rustfmt, 4-space indent, `snake_case`, module-per-subsystem.
- TypeScript: Prettier (see `.prettierrc`), 2-space indent, `camelCase`.
- Match the comment density and idiom of surrounding code.
- Document public APIs and any behavior change in the docs.

## Submitting changes

- Keep PRs focused on one concern.
- Reference the issue you're addressing.
- For architecture-affecting decisions, add an ADR (see
  [docs/architecture/adr/](docs/architecture/adr/) and the template in
  [docs/08-ADR-TEMPLATE.md](docs/08-ADR-TEMPLATE.md)).

## Adding your own profile data

Want NIKHIL//OS for yourself? You don't need to contribute code — just fork
and replace `knowledge/data/`. See
[docs/contributions/adding-your-profile.md](docs/contributions/adding-your-profile.md).

## Code of Conduct

All participants agree to abide by [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
