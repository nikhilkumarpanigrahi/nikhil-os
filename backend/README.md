# NIKHIL//OS Backend

The production service behind the OS: a **visitor contact inbox with instant
alerts**, a **JWT-protected admin panel**, and a **live knowledge API** serving
the same canonical `knowledge/data/profile.json` the WASM core embeds.

Built with **Rust + axum + PostgreSQL**, deployed on the **AWS free tier** as
three containers (Postgres + API + Caddy) via Docker Compose. See
[`docs/architecture/adr/ADR-0004-rust-axum-backend.md`](../docs/architecture/adr/ADR-0004-rust-axum-backend.md)
for the rationale.

| Layer | Tech |
|---|---|
| API | Rust, axum 0.8, tokio, tower-http, sqlx 0.8 (offline macros) |
| Auth | argon2 (password) + HS256 JWT (15-min sessions) |
| Rate limiting | governor (per-IP, in-memory) |
| Alerts | Telegram bot (provider-abstracted, degrades to structured logs) |
| DB | PostgreSQL 16 (container, free-tier tuned) |
| TLS / ingress | Caddy (auto-HTTPS via Let's Encrypt) |
| CI/CD | GitHub Actions — lint + test + docker build; SSH deploy on push |

---

## API reference

### Public (no auth)

| Method | Path | Notes |
|---|---|---|
| `GET` | `/healthz` | Liveness — `200 {}` |
| `GET` | `/readyz` | Readiness — pings Postgres |
| `GET` | `/api/v1/profile` | Full canonical profile (same data as the WASM core) |
| `GET` | `/api/v1/projects` | `{ "projects": [...] }` |
| `GET` | `/api/v1/skills` | `{ "skills": [...] }` |
| `GET` | `/api/v1/experience` | `{ "experience": [...] }` |
| `GET` | `/api/v1/claims` | `{ "claims": [...] }` |
| `POST` | `/api/v1/contact` | Submit a message → `201 {id, status}` |

Knowledge endpoints return `ETag` + `Cache-Control: public, max-age=300`.

**`POST /api/v1/contact`** body:

```jsonc
{
  "name": "Ada Lovelace",       // 2–80 chars, required
  "email": "ada@example.com",   // strict-but-minimal validation, required
  "subject": "Let's build",     // ≤120 chars, optional
  "topic": "collaboration",     // general|collaboration|opportunity|feedback|recruiting
  "body": "…",                  // 10–2000 chars, required
  "website": ""                 // HONEYPOT — must stay empty. Bots that fill it
                                // get a fake 201 and nothing is stored.
}
```

Per-IP rate limit: `CONTACT_RATE_LIMIT`/min. Every message is stored and the
owner is notified off the request path.

### Admin (Bearer JWT)

| Method | Path | Notes |
|---|---|---|
| `POST` | `/admin/api/login` | `{ "password" }` → `{ token, expires_in }` (rate-limited) |
| `GET` | `/admin/api/messages` | `?status=&limit=&offset=` |
| `PATCH` | `/admin/api/messages/{id}` | `{ "status" }` ∈ new\|read\|replied\|archived |
| `GET` | `/admin/api/stats` | Counts per status + 14-day volume + knowledge etag |
| `GET` | `/admin` | Self-contained dark-themed inbox panel (single HTML file) |

Sensitive actions are audit-logged to `admin_events`. Errors always serialize
as `{ "error": { "code", "message" } }` — internals never leak.

---

## Local development

Prereqs: Rust (stable), a local Postgres, `sqlx-cli`.

```bash
# 1. Create the DB + role once
psql -U postgres -c "CREATE ROLE nikhilos LOGIN PASSWORD 'nikhilos_dev_pw' CREATEDB;"
psql -U postgres -c "CREATE DATABASE nikhilos OWNER nikhilos;"
psql -U postgres -c "CREATE DATABASE nikhilos_test OWNER nikhilos;"

# 2. Configure
cp .env.example .env
#   DATABASE_URL=postgres://nikhilos:nikhilos_dev_pw@localhost:5432/nikhilos
#   ADMIN_PASSWORD_HASH=$(cargo run --bin hashgen)   # prompts for the password
#   ADMIN_JWT_SECRET=$(openssl rand -base64 48)

# 3. Run migrations (also runs automatically at startup)
cargo sqlx migrate run

# 4. Refresh the offline query cache after editing any SQL
cargo sqlx prepare -- --all-targets

# 5. Run it
cargo run
#    → http://localhost:8080/healthz
#    → admin panel at http://localhost:8080/admin
```

### Tests

```bash
DATABASE_URL=postgres://nikhilos:nikhilos_dev_pw@localhost:5432/nikhilos_test cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Integration tests in [`tests/api.rs`](tests/api.rs) drive the real router with
`tower::ServiceExt::oneshot` against a real Postgres. The suite is
**concurrency-safe**: tests use unique email/IP markers instead of truncating
tables.

---

## Deploy to AWS free tier ($0/month)

One EC2 instance (`t2.micro`/`t3.micro`, free tier), three containers, Caddy
terminates TLS. Full one-time provisioning:

```bash
# On a fresh Ubuntu 24.04 LTS free-tier EC2, as root:
sudo bash scripts/provision.sh            # opens ports, Docker, swap, deploy user

# As the deploy user, clone + configure:
#   git clone git@github.com:nikhilkumarpanigrahi/nikhil-os.git
#   cd nikhil-os/backend
#   cp .env.example .env   # fill ADMIN_PASSWORD_HASH, ADMIN_JWT_SECRET, DOMAIN,
#                          # POSTGRES_PASSWORD, TELEGRAM_BOT_TOKEN/CHAT_ID, CORS
#   docker compose up -d --build

# Point your domain's A record at the EC2 public IP (is-a.dev or DuckDNS),
# then Caddy grabs a certificate automatically:
#   curl https://api.nikhil.is-a.dev/healthz
```

Every push to `main` touching `backend/**` runs the `backend-deploy` workflow:
SSH → `git pull` → rewrite `.env` from GitHub Secrets → `docker compose up -d
--build` → poll `/healthz` until green.

**GitHub Secrets** (repository settings → Actions → secrets):

```
AWS_EC2_HOST          # EC2 public IP or hostname
AWS_EC2_USER          # deploy user created by provision.sh
AWS_EC2_KEY           # private SSH key (PEM) for that user
DOMAIN                # api.nikhil.is-a.dev
POSTGRES_PASSWORD     # long random string
ADMIN_PASSWORD_HASH   # output of cargo run --bin hashgen
ADMIN_JWT_SECRET      # openssl rand -base64 48
TELEGRAM_BOT_TOKEN
TELEGRAM_CHAT_ID
CORS_ALLOWED_ORIGINS  # https://nikhilkumarpanigrahi.github.io,http://localhost:5173
CONTACT_RATE_LIMIT    # 10
```

**Cost truth:** year one ≈ $0 (free-tier EC2, container Postgres, free domain,
free Telegram). After the 12-month free tier, a t3.micro is ~$7–8/mo — either
budget for it or migrate. Nothing here is irreversibly coupled to the free tier.

---

## Layout

```
backend/
├── Cargo.toml            # standalone workspace (own lockfile)
├── Dockerfile            # multi-stage, non-root, offline sqlx build
├── docker-compose.yml    # db + api + caddy (free-tier tuned)
├── Caddyfile             # {DOMAIN} → api:8080, auto-HTTPS
├── .env.example          # full configuration contract
├── migrations/           # SQL migrations (embedded via sqlx::migrate!)
├── .sqlx/                # committed offline query cache
├── src/
│   ├── main.rs           # bootstrap + graceful shutdown
│   ├── app.rs            # router + middleware order
│   ├── config.rs         # 12-factor env config, fails fast
│   ├── db.rs             # pool + migrations
│   ├── error.rs          # AppError → {error:{code,message}} JSON
│   ├── auth.rs           # argon2 verify + JWT issue/verify + Bearer extractor
│   ├── knowledge.rs      # typed profile.json (OnceLock) + etag
│   ├── routes/           # health, knowledge, contact, admin
│   ├── services/         # inbox (persistence), notify (Telegram/log)
│   └── admin/            # self-contained inbox panel (index.html)
├── tests/api.rs          # integration tests (real Postgres, oneshot router)
└── .cargo/config.toml    # SQLX_OFFLINE=true
```

## Design notes

- **Modular monolith** (per `docs/02-ARCHITECTURE.md` §6): one deployable with
  clean seams for future AI/RAG, guestbook, etc.
- **Offline-first contract** (per docs): the web app treats the API as an
  enhancement. If it's unreachable, the OS still works and the Contact app
  degrades to a `mailto:` fallback.
- **Single source of truth for knowledge**: `knowledge/data/profile.json` is
  embedded by both the WASM core (`include_str!`) and this service — no drift.
- **sqlx offline workflow**: every checked query is cached in `.sqlx/`; builds
  (local, CI, Docker) never need a live database. After changing SQL, run
  `cargo sqlx prepare -- --all-targets` and commit the new cache.
