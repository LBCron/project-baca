---
type: Developer Guide
title: "Project Baca — Developer Guide & Local Operations"
description: "Step-by-step instructions for local development, database migrations, WASM compilation, and test execution."
tags: [guide, onboarding, developer, setup, workflow, okf]
---

# GUIDE.md — Developer & Operations Guide

Practical instructions to build, run, test, and develop **Project Baca** locally.

## 1. Prerequisites

1. **Rust Toolchain:**
   - Rust 1.80+ (stable).
   - WASM target: `rustup target add wasm32-unknown-unknown`.
   - Tooling: `rustup component add clippy rustfmt`.
2. **WASM Bundler:**
   - Trunk: `cargo install trunk` (or via OS package manager).
3. **Container Runtime:**
   - Docker & Docker Compose (v2.20+).
4. **Python Environment (for Ingestion Worker):**
   - Python 3.11+ with `uv` or `venv`.

## 2. Infrastructure Services

Orchestrate local dependencies via Docker Compose:

```bash
# Start background containers
make db-up

# Check container health
docker compose ps
```

### Port Mapping
* **PostgreSQL 17 + pgvector:** `localhost:5433` (`baca_user` / `project_baca_db`)
* **Redis 7 (Streams & Cache):** `localhost:6380`
* **MinIO Object Storage:** S3 API on `localhost:9005`, Console on `http://localhost:9006`
* **Mailpit (SMTP):** SMTP on `localhost:1025`, Web UI on `http://localhost:8025`

## 3. Database Migrations

Manage database schema via paired SQL files in `migrations/`:

```bash
make migrate-up     # Apply pending migrations
make migrate-down   # Rollback latest migration
make migrate-status # Show applied migration history
```

## 4. Running Application Components

### Development Commands
* **`make dev`:** Orchestrator overview showing running ports and service commands.
* **`make dev-server`:** Runs Axum backend with auto-reload via `cargo-watch` (port 8080).
* **`make dev-web`:** Runs Leptos WASM frontend with hot-reload via Trunk (port 3000).

### API Documentation & Swagger UI
* **Swagger UI:** Interactive testing UI at `http://localhost:8080/swagger-ui`.
* **OpenAPI Spec:** Raw JSON spec at `http://localhost:8080/api-docs/openapi.json`.
* Schema definitions are compiled directly from shared DTOs in `crates/shared`.

### Structured Logging & Observability
* **Human-readable text:** Default output for local development.
* **Structured JSON:** Enable via `LOG_FORMAT=json cargo run -p server`.
* **Request Correlation:** The `request_id_middleware` mints a fresh UUID `x-request-id` per request (client-supplied values are ignored) across all spans and responses.

### Server Environment Notes
* `PUBLIC_RATE_LIMIT_PER_MINUTE` caps unauthenticated catalog and insight reads per IP. Keep `0` locally (dev and in-memory tests share one fallback IP); production compose sets 600.
* Signup fails closed with HTTP 502 when SMTP is unreachable, so Mailpit must be up for the OTP flow.

### Ingestion Worker (Python)
```bash
make worker-install   # create venv + install deps (first time only)
make worker-test      # unit tests, no services needed (9/9)
make worker-test-live # DLQ E2E, needs live Redis + Postgres
make worker-once      # process a single queued job then exit
make worker           # long-running consumer
```
Worker settings via `.env`: `GEMINI_API_KEY`, `LLM_MODEL_NAME` (default `gemini-flash-latest`), `RECLAIM_IDLE_MS` (default 300000). Canonical stream `stream:epub_ingestion`, DLQ `stream:epub_ingestion:dlq`.
Test debris (pending PEL, draft probe rows, fake MinIO objects) accumulates on the shared dev stack; clean it with `make purge-test-debris` (dev-only).

## 5. Testing & Quality Assurance

Comprehensive workspace test suite (`cargo test --workspace`, expect 0 failures) plus worker unit tests. Functional server suites live in one binary (`crates/server/tests/it`); each `make test-*` target filters that binary by module:

```bash
make test-smoke        # Service boot, health endpoints, Swagger UI, network reachability
make test-integration  # Request ID minting, CORS allowlist, OpenAPI schema verification
make test-auth         # Signup/OTP/login/rotation/logout/revoke + guest merge
make test-catalog      # Catalog, reader, heartbeat anti-farm, badges
make test-semantic     # Quote search, save/card ownership, atomic cards, recaps
make test-admin        # Upload RBAC/validation, job monitor, drop-off analytics
make test-database     # Constraints, cascades, rollbacks, query plans
make test-performance  # Latency SLA benchmarks (REST p95 < 50ms, OTP p95 < 100ms)
make test-load-stress  # Concurrent load + rate-limit shedding
make test-api-boundary # 404/405, malformed payloads, pagination clamps
make test-security     # OWASP headers, lockout, revocation, error masking
make test-reliability  # Disconnect fallback, fault injection, oversized ids
make test-all          # Complete end-to-end test execution
make test-web          # Web pyramid (unit + e2e + component + integration) vs live :8080 + :3000
make test-web-full     # + visual goldens + axe a11y + firefox smoke
```

### Web Regression Gate
`make test-web` runs `crates/web/tests/` (pytest pyramid: e2e + component gallery + API contracts + unit); `make test-web-full` adds visual goldens, axe a11y, and a Firefox smoke. Run it after any `crates/web` change. Goldens live in `tests/visual/test_visual.py-snapshots/` and are captured at 1280x720 in Chromium with the English default; delete a golden to regenerate it after an intentional visual change, then review the new image before committing. The suite needs a published book with chapters (the Sitti Nurbaya seed) and Mailpit for the authed flow.

### Test Libraries
* **Mocking:** `mockall` (declarative mock generation for trait ports).
* **Assertions:** `pretty_assertions` (colored diffs) and `claims` (`assert_ok!`, `assert_err!`).
* **Parameterized Tests:** `rstest` (fixtures and table-driven test cases).

## 6. Troubleshooting

1. **Port 5433 Conflict:** Check active processes with `lsof -i :5433` and adjust `docker-compose.yml` if necessary.
2. **Missing WASM Target:** Run `rustup target add wasm32-unknown-unknown`.
3. **MinIO Connection Error:** Ensure buckets (`baca-epubs`, `baca-covers`) exist — the server creates them idempotently at startup; uploads fail closed with 500 when storage is unreachable.
4. **Redis Streams Group:** The worker auto-initializes the consumer group using `XGROUP CREATE` fallback.
