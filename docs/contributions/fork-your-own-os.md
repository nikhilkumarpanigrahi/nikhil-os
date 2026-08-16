# Make NIKHIL//OS your own

NIKHIL//OS is designed to be forked into your own portfolio OS. The profile data is
cleanly separated from the engine, so you never touch Rust or React internals to make
it yours.

## Step 1 — Fork the repository

1. Click **Fork** on [github.com/nikhilkumarpanigrahi/nikhil-os](https://github.com/nikhilkumarpanigrahi/nikhil-os).
2. `git clone` your fork, `cd` into it.

## Step 2 — Replace the profile data

Everything the OS *shows about you* lives in one file:

```
knowledge/data/profile.json
```

Open it and replace every field:

- **`person`** — your name, role, location, summary, contact (email / GitHub / LinkedIn / website).
- **`highlights`** — 3–5 words/phrases that describe you.
- **`skills`** — name, category (`Systems` / `Frontend` / `Backend` / `Data` / `Tooling`), and a 0–100 level.
- **`technologies`** — the stack you want shown on the Projects page.
- **`projects`** — the two placeholder entries (`your-project-1`, `your-project-2`) are templates:
  set `title`, `summary`, `description`, `architecture`, `technologies`, `highlights`,
  and `repo` / `demo` URLs. The `evidence` array is where you back claims with links.
- **`experience`, `education`, `certifications`, `achievements`, `contributions`** — your history.
- **`claims`** — optional verifiable statements ("This project runs a real OS in the
  browser"), each referencing a project id as evidence. The AI Core app will eventually
  reason over these.

> The `id` of a project is used by `claims[].evidence` and by the `Projects` app for
> deep links — keep ids stable and URL-friendly.

## Step 3 — Rebuild (the UI is generated, not hand-edited)

The core embeds `profile.json` at build time. After editing:

```bash
cd crates/core
wasm-pack build --target bundler --out-dir ../../web/pkg --features wasm
cd ../../web
npm ci && npm run dev
```

Open `http://localhost:5173` — the OS now shows **your** data. Terminal, Files,
Projects, Resume, Recruiter, and System Monitor all read from the same source.

## Step 4 — Make it yours beyond the data

- **Colors / fonts** — edit the design tokens in [`web/src/styles/tokens.css`](../../web/src/styles/tokens.css).
- **Apps** — add your own app under `web/src/apps/` and register it in
  [`web/src/apps/registry.tsx`](../../web/src/apps/registry.tsx).
- **Shell** — add builtins in [`crates/core/src/shell/builtins.rs`](../../crates/core/src/shell/builtins.rs).
- **Kernel behavior** — boot order in `crates/core/src/system.rs`, scheduler in `crates/core/src/scheduler.rs`.

## Step 5 — Ship it

Deployment is a GitHub Pages action: push to `main` and your fork is live at
`https://<you>.github.io/<repo>/`. Your fork's README badge and the `demo` URL in
`profile.json` should point there.

See [`docs/09-README.md`](../09-README.md) and [`docs/07-ENGINEERING-PROMPTS.md`](../07-ENGINEERING-PROMPTS.md)
for the full picture.
