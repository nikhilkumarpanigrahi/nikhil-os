# Knowledge Schema

The knowledge layer is the single source of truth for the profile. Data lives in
[`knowledge/data/profile.json`](../data/profile.json), is embedded into the Rust core
at build time (`include_str!`), and is served to the UI through the WASM bridge
(`core.profile()`).

## Entities

| Entity | Fields | Notes |
| --- | --- | --- |
| `Person` | name, role, location, summary, contact | Canonical identity. |
| `Contact` | email, github, linkedin, website | Optional fields may be empty. |
| `Skill` | name, category, level | `level` is 0–100 (self-assessed). |
| `Technology` | name | A flat list of technologies used. |
| `Project` | id, title, category, summary, description, architecture, technologies, highlights, repo, demo, evidence | `category` is `systems` / `ai-ml` / `backend` / `open-source`. |
| `Experience` | role, organization, period, summary, highlights | Reverse-chronological. |
| `Education` | degree, institution, period | |
| `Certification` | name, issuer, year | |
| `Achievement` | title, description, evidence | |
| `Contribution` | repo, description | Open-source contributions. |
| `Claim` | claim, evidence, confidence | Verifiable statements; `evidence` references project ids. |
| `Evidence` | title, url | A URL that supports a project or claim. |

## Relationships

The entities form an implicit graph used by later AI features:

- `Person` — `HAS_SKILL` → `Skill`
- `Person` — `WORKED_AT` → `Experience.organization`
- `Person` — `STUDIED_AT` → `Education.institution`
- `Person` — `BUILT` → `Project`
- `Project` — `USES` → `Technology` / `Skill`
- `Project` — `SUPPORTED_BY` → `Evidence`
- `Claim` — `SUPPORTED_BY` → `Project.id` (via `evidence`)

## Adding your own profile

1. Copy `knowledge/data/profile.json`.
2. Replace every `REPLACE-*` field with your own data.
3. Replace the placeholder projects (`your-project-1`, `your-project-2`) with real ones,
   or delete them.
4. Rebuild the core (`wasm-pack build`) — the UI picks the new data up automatically.

See [`docs/contributions/`](../../docs/contributions/) for a full walkthrough.
