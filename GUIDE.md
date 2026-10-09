---
type: Developer Guide
title: Developer Guide
tags:
  - developer
  - guide
  - okf
  - onboarding
  - setup
  - workflow
---

# Developer Guide

Build, run, and test Project Baca locally. Deployment is in [ARCHITECTURE.md](ARCHITECTURE.md) section 8.

## 1. Prerequisites

- Rust stable with `rustup target add wasm32-unknown-unknown` and `rustup component add clippy rustfmt`.
- Trunk (`cargo install trunk`). `cargo-watch` is optional; `make dev-server` falls back to `cargo run`.
- Docker Compose v2.
- Python 3.11+ and `uv` (worker and janitor).
- Web suite: a venv at `~/.venvs/webapp-testing` with `pytest`, `pytest-playwright`, `pillow`, and Playwright Chromium and Firefox installed.

## 2. Services (`make db-up`)

| Service | Image | Host port |
|---|---|---|
| PostgreSQL 17 + pgvector | `pgvector/pgvector:pg17` | 5433 |
| Redis 7 | `redis:7-alpine` | 6380 |
| MinIO | `cgr.dev/chainguard/minio` | 9005 (S3 API), 9006 (console) |
| Mailpit | `axllent/mailpit` | 1025 (SMTP), 8025 (web UI) |

Credentials and bucket names are in `.env.example`. The server creates the buckets at boot.

## 3. Configuration (`.env`)

| Variable | Default | Meaning |
|---|---|---|
| `APP_ENV`, `APP_HOST`, `APP_PORT` | `development`, `0.0.0.0`, `8080` | `production` refuses the default JWT secret and unmounts Swagger |
| `CORS_ALLOWED_ORIGINS` | localhost:3000 origins | comma-separated allowlist |
| `LOG_FORMAT` | `text` | `json` for NDJSON logs |
| `TRUST_PROXY_HEADERS`, `TRUSTED_IP_HEADER` | `false`, `x-forwarded-for` | with trust on, the client address is read from that one header only (`cf-connecting-ip` behind Cloudflare) |
| `PUBLIC_RATE_LIMIT_PER_MINUTE` | `0` (off) | per-IP cap on unauthenticated reads; production compose sets 600 |
| `JWT_SECRET`, `JWT_ACCESS_EXPIRY_MINUTES`, `JWT_REFRESH_EXPIRY_DAYS` | dev secret, `1440`, `14` | production compose pins access tokens to 60 minutes |
| `DATABASE_URL`, `DATABASE_MAX_CONNECTIONS`, `DATABASE_MIN_CONNECTIONS`, `DATABASE_CONNECT_TIMEOUT_SECS`, `DATABASE_IDLE_TIMEOUT_SECS` | dev URL, `20`, `5`, `10`, `300` | SeaORM pool |
| `REDIS_URL` | `redis://localhost:6380/0` | OTP, sessions, rate limits, streams |
| `S3_ENDPOINT`, `S3_REGION`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`, `S3_BUCKET_EPUBS`, `S3_BUCKET_COVERS`, `S3_FORCE_PATH_STYLE` | MinIO dev values | any S3-compatible store |
| `SMTP_HOST`, `SMTP_PORT`, `SMTP_SECURITY`, `SMTP_USER`, `SMTP_PASSWORD`, `SMTP_FROM_EMAIL`, `SMTP_FROM_NAME` | Mailpit, `starttls` | `SMTP_SECURITY=none` for Mailpit; credentials sent only when `SMTP_USER` is set; signup answers 502 when the relay is unreachable |
| `EMBEDDING_PROVIDER`, `GEMINI_API_KEY`, `EMBEDDING_MODEL_NAME`, `EMBEDDING_DIMENSION` | `gemini`, none, `gemini-embedding-2`, `768` | shared by server (query embeddings) and worker; `AI_*` aliases accepted |
| `LLM_MODEL_NAME`, `RECLAIM_IDLE_MS` | `gemini-flash-latest`, `300000` | worker only |
| `API_BASE_URL` | `http://localhost:8080/api/v1` | baked into the WASM bundle at build time (`make build API_BASE_URL=...`) |

## 4. Run

```bash
make migrate-up seed-dev
make dev-server        # cargo-watch when installed, else cargo run -p server
make dev-web           # trunk serve on 127.0.0.1:3000
make worker            # after make worker-install
```

- Swagger UI `/swagger-ui`, OpenAPI `/api-docs/openapi.json` outside production.
- Every response carries a server-minted `x-request-id`; client values are ignored.
- Migrations: `make migrate-up`, `make migrate-down`, `make migrate-status` over paired SQL files in `migrations/`.
- Dev data: `make seed-dev` installs four CC0 books with fixed ids (the web suite pins `a0000000-0000-4000-8000-000000000001`). `make purge-test-debris` removes rows, stream entries, and objects left by crashed test runs. Never run either against production.

## 5. Tests

| Target | Scope | Needs |
|---|---|---|
| `make check` | `cargo check` native and wasm, clippy with `-D warnings`, `fmt --check` | nothing |
| `make test-domain` | 16 pure domain tests | nothing |
| `cargo test -p infra`, `cargo test -p server --lib` | config, repositories, helpers (25 + 6) | nothing |
| `make test-<module>` | one module of `crates/server/tests/it`: smoke, integration, auth, catalog, semantic, admin, database, api-boundary, security, reliability | `db-up` |
| `cargo test -p server --test it` | all functional modules, 94 tests (9 live tests ignored) | `db-up` |
| `make test-live` | the 9 ignored live tests, single-threaded | `db-up` |
| `make test-performance`, `make test-performance-release` | 4 p95 latency benchmarks | `db-up`, idle machine |
| `make test-load-stress` | 200 concurrent tasks and rate-limit saturation | `db-up` |
| `make worker-test`, `make worker-test-live` | 16 worker unit tests; DLQ end-to-end | nothing; `db-up` |
| `make test-web` | `cargo test -p web --lib` plus Playwright e2e, component, integration | `db-up`, API on :8080, web on :3000, Mailpit |
| `make test-web-full` | adds visual goldens, axe a11y, Firefox smoke, and the inline-script check on `crates/web/dist` | same |
| `make test-all` | `cargo test --workspace` (includes performance and load) | `db-up` |

Notes:

- Functional tests build the Axum app in-process against the shared dev services and delete every row they create (`TestHarness::track_*`, `harness.cleanup()`), so row counts match before and after a run.
- The web suite runs from `crates/web/tests` with `--browser chromium`. Contexts pin `color_scheme=dark`; the a11y suite also scans the Paper theme. Goldens are 1280x720 under `tests/visual/test_visual.py-snapshots/`; delete one to regenerate it after an intended change and review the image before committing.
- Benchmarks follow machine load. Rerun on an idle machine before reading a failure as a regression.
- Do not run the web suite while heavy backend suites run; Playwright `networkidle` waits can time out.
- Signup is capped at 10 per hour per IP. Delete the Redis key `otp:signup:ip:127.0.0.1` if local auth tests answer 429.

## 6. Troubleshooting

- Port collisions: the dev stack uses 5433, 6380, 9005, 9006, 1025, 8025.
- OTP mail never arrives: set `SMTP_SECURITY=none`; signup answers 502 while SMTP is unreachable.
- Uploads fail with 500: MinIO is unreachable; buckets are created at boot.
- Worker never consumes: check `XINFO GROUPS stream:epub_ingestion`; the group is created when the worker starts.
- Frontend change not visible: Trunk watches `src`, `style`, and `index.html` of `crates/web` only; a `crates/shared` change needs a Trunk restart.

## 7. Reader controls

- Turn a page: drag anywhere on a touch screen, or from the outer 22% of the page with a mouse. Release past the middle or flick to finish; release early to fall back. Tap the edges, use the edge arrows on wide screens, or press Space / Shift+Space, the arrow keys, PageUp / PageDown. Home and End jump within the chapter.
- Center tap dims the bars; tap again to restore them. The chapter title in the bar opens the chapter list on the book page. The last page resists, and a forward tap opens the finale card.
- Reduced motion: a short crossfade replaces the paper animation.
