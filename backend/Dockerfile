# syntax=docker/dockerfile:1
#
# Multi-stage build. Build context is the REPO ROOT (docker build -f
# backend/Dockerfile .), not backend/ — the binary embeds
# knowledge/data/profile.json (include_str!) which lives at the repo root and
# is shared with the WASM core.
#
# The builder MIRRORS the repo layout (/app/backend/ + /app/knowledge/) rather
# than flattening it, so the include_str!("../../knowledge/data/profile.json")
# path in backend/src/knowledge.rs resolves exactly as it does on disk. Flatten
# it and the relative path breaks the compile.
#
# Builder: rust:1.97-alpine (musl static link). ring/rustls (sqlx + reqwest TLS)
# need a C toolchain, so install gcc + musl-dev. The Alpine rust image targets
# musl by default, so `cargo build --release` yields a static binary that runs
# on the slim glibc runtime below.
#
# The toolchain is pinned to 1.97 to match the stable rustc used to generate
# Cargo.lock — locked deps (e.g. time 0.3.55) require rustc ≥ 1.88, and older
# images fail the build. Keep this in sync with the local toolchain.
#
# LTO is an ARG (default `thin`) so docker-compose can override it to `off` on
# the free-tier t2.micro — a 1 GB box builds much faster/more safely without
# full LTO, at the cost of a slightly larger binary. CI's `docker build .`
# sanity check inherits the default.
FROM rust:1.97-alpine AS builder
ARG LTO=thin
RUN apk add --no-cache musl-dev gcc
# Offline query cache: the build must never touch a live database.
COPY backend/.cargo /app/backend/.cargo
COPY backend/Cargo.toml backend/Cargo.lock /app/backend/
COPY backend/migrations /app/backend/migrations
COPY backend/.sqlx /app/backend/.sqlx
COPY backend/src /app/backend/src
# Canonical profile data, shared with the WASM core — at the same repo-root
# position as on disk so knowledge.rs's include_str! resolves.
COPY knowledge /app/knowledge
WORKDIR /app/backend
ENV SQLX_OFFLINE=true
# The --config value must be a quoted TOML string — `profile.release.lto=thin`
# is a TOML parse error (bare words are not valid TOML values). The \" escapes
# the quotes so the shell hands cargo `profile.release.lto="thin"`.
RUN cargo build --release --locked --config "profile.release.lto=\"${LTO}\""

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system app \
    && useradd --system --gid app --home-dir /nonexistent --shell /usr/sbin/nologin app
COPY --from=builder /app/backend/target/release/nikhil-os-backend /usr/local/bin/nikhil-os-backend
USER app
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -fsS http://localhost:8080/healthz || exit 1
CMD ["nikhil-os-backend"]
