# Changelog — Project Baca

All notable changes are documented chronologically following [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security
- Client address comes from one header: with `TRUST_PROXY_HEADERS=true` only `TRUSTED_IP_HEADER` (default `x-forwarded-for`) is read; `Caddyfile.prod` strips the other address headers. The default JWT secret is refused whenever proxy headers are trusted.
- Worker sanitizer re-escapes decoded character references, so markup hidden in entities stays text.
- Draft and archived books answer 404 on chapter, insight, and offline-bundle routes.
- SMTP through `lettre` with STARTTLS (`SMTP_SECURITY=starttls|none`); Gemini key sent in `x-goog-api-key`; connection strings, recipients, and OTPs masked in logs.
- Swagger and OpenAPI unmounted in production; `/health` answers 503 without a database; `Cache-Control: no-store` on auth and bearer responses; validation errors stop echoing values; search queries capped at 200 characters.
- Logout blacklists the session's last access token; heartbeat gate is one atomic `SET NX EX`; a duplicate quote save returns the existing row; `PATCH /admin/books/{id}` to `processing` is 409.
- Strict script CSP: a Trunk post-build hook externalizes the inline bootstrap, `script-src` drops `unsafe-inline`, `make test-web-full` fails on any inline script.
- OTP lock answers 429 with `Retry-After` after three wrong codes.
- Audit hardening (2026-10-07): 50 MB EPUB uploads past axum's default body limit; atomic refresh rotation (`GETDEL`, 30 s grace); unverified accounts behave like a wrong password; signup fails closed on SMTP (502); password change returns a fresh pair; heartbeat credit clamped to 90 s per user; admin guard re-reads the user row; HSTS on the API; SPA CSP in both Caddyfiles.
- Earlier hardening (ADR-20 to ADR-27): SHA-256 refresh storage with family revocation on replay; two-layer login lockout; signup SMTP cap; OTP lock without deletion; fail-closed 503 on Redis outage for auth and AI paths; EPUB magic check and worker zip guard; CORS allowlist; server-minted request ids; avatar URL allowlist; guest merge transactional with a 100-record cap.

### Added
- Reader as an open book: paper frame with running heads and folios; pages turn as spine-hinged strips driven by a damped spring from a drag, tap, or key; one-column clones keep a turn cheap; the last page resists and a finale card closes the book; center tap dims the bars; haptic on landing; reduced motion keeps a crossfade; the chapter title links to the chapter list.
- Home catalog preview: two rows (2x2 to 2x5 by width) and a "see the whole catalog" button; search and filters show everything with paging.
- Idle illustrations placed: clothesline (empty curator lists), pigeonholes (empty signed-in shelf), rocket (404 and empty search), owl (finale).
- Navigation markers: `aria-current` from hash and query, header dot, "Cara kerja" arrow, "Rak saya", a seven-stroke brand mark in the header and as favicon, system color scheme on first visit.
- Cover delivery: `GET /api/v1/covers/{file}` streams from the covers bucket with an immutable cache; covers render as CSS backgrounds with a typographic fallback.
- Curator desk: manuscripts (all statuses, counts, filters, archive, upload with a live job card), queue (dead-letter replay), retention (drop-off funnel). Endpoints `GET /admin/books`, `PATCH /admin/books/{id}`, `GET /admin/dlq`, `POST /admin/dlq/{id}/replay`.
- Worker resilience: Gemini retries with quadratic backoff and `Retry-After`; recap built from prior key concepts within the prompt budget; `tldr_cache` upsert on replay.
- Per-device sessions: `GET /me/sessions`, `DELETE /me/sessions/{id}`, `POST /auth/revoke-all?keep_current=true`; the shelf lists devices with per-row sign-out.
- Guest quotes kept on the device (IndexedDB v3) and merged on sign-in through `POST /quotes/save-batch` (up to 50, per-item outcome).
- Test isolation: `TestHarness` tracks seeded rows and `cleanup()` restores counts; `make seed-dev` installs four CC0 works with fixed ids; the janitor sweeps survivors.
- Rotaria redesign and responsive pass (2026-10-07 and 08): two-surface design system, phone tab bar, bottom sheets, action dock, paginated reader with type sheet, shelf with streak and sessions, table-driven ID/EN dictionary, WebP illustrations, safe-area insets, five-column catalog from 1440 px.
- `GET /api/v1/me/streak`; `PUBLIC_RATE_LIMIT_PER_MINUTE` public read cap on the shared limiter engine; functional server suites consolidated in `crates/server/tests/it` with lean debug profiles.
- Milestones 04 to 07 (2026-09-27 and 28): dual-mode embeddings and scoped HNSW quote search; atomic cards and recaps from `tldr_cache`; saved quotes with SVG cards; MinIO storage, admin upload (202), Python worker with DLQ and reclaim; job monitor and drop-off funnel; Leptos web reader with five routes; 401 auto-refresh; PWA manifest, service worker, offline reader, pending-write queue; production images, `docker-compose.prod.yml`, `Caddyfile.prod`, `.env.production.example`; `crates/domain` with DB-free tests; `make purge-test-debris`; `scripts/pg_backup.sh`.

### Fixed
- Anchors scroll after content loads and stop under the sticky header; the search tab focuses the field; sheets close on Escape and return focus; the upload input is keyboard-reachable; remaining aria labels translated.
- Signals read after awaits use `try_get`; toasts dedupe with `role=alert` for errors; catalog, book, and reader show a network error with retry; the admin poll stops on unmount; shelf skeletons; service worker shell fallback and asset pruning; Paper contrast passes AA.
- Reader root used `min-height` and never paginated long chapters; it is a fixed-height flex column; insight sheets fetch on open; measure capped at 1440 px.
- Quote API contracts: unknown book 404, invalid (book, chapter) pair 400, saving and cards require the owner.
- Save Offline key mismatch (`book_id` vs `id`, IndexedDB v2) and typed chapter records; shared header and button styles; hero dot labels and faint contrasts.
- Docs and comments: spec tracers removed from code, SSOT corrections in SRS and ERD, history re-sorted newest first.

### Removed
- The unversioned `/api/*` alias; `/api/v1/*` is the only API surface.

## [0.2.1] — 2026-09-26

### Added
- **Milestone 01 (Infrastructure & Entities):** Strongly-typed modular configuration system (`ServerConfig`, `DatabaseConfig`, `RedisConfig`, `AuthConfig`, `StorageConfig`, `EmailConfig`, `AiConfig`) loaded via `dotenvy`; dynamic SeaORM and Redis connection pools; 13 SeaORM entity models with `postgres-vector` and `PgVector` 768-dim support.
- **Milestone 02 (Authentication & Guest Reconciliation):** Complete authentication lifecycle (`/api/v1/auth/signup`, `verify-otp`, `login`, `refresh`, `logout`, `/api/v1/me`); Argon2id password hashing; Redis OTP with SHA-256 and 3-attempt limit; single-use refresh token rotation; async SMTP delivery via Mailpit; guest progress reconciliation (`/api/v1/progress/merge`) with SQL upsert and `GREATEST` progress resolution; RBAC middleware (`require_admin`).
- **Milestone 03 (Catalog Backend & Reader API):** Public catalog with cursor-based pagination and multi-filter criteria (`/api/v1/books`); typo-tolerant search (<3ms via `pg_trgm` and `word_similarity`); chapter delivery and offline reading bundles (`/api/v1/books/{id}/offline-bundle`); EPUB CFI reading progress synchronization (`/api/v1/progress/{book_id}`) and active position retrieval; daily streak gamification with heartbeat logging (`/api/v1/activity/heartbeat`); and badges system (`/api/v1/badges`, `/api/v1/me/badges`).
- **Database Index Optimization (Migration 07):** Added dedicated B-tree indexes for all Foreign Key columns to eliminate sequential scans during cascading deletes; consolidated unique indexes; and added `idx_books_published_year_id` on `books (publication_year DESC NULLS LAST, id ASC) WHERE status = 'published'` to enable zero-sort catalog pagination.
- **Automated Testing Modules:** Expanded test suite to 67 automated tests covering database integrity (`database_test.rs`), load and stress concurrency (`load_stress_test.rs`), API boundary and error envelope contracts (`api_boundary_test.rs`), and OWASP ASVS compliance (`security_owasp_test.rs`).
- **Edge Transport Architecture:** Adopted Caddy Reverse Proxy for HTTP/3 (QUIC over UDP 443) edge termination with automatic TLS and HTTP/2 over TCP fallback; preserved stateless TCP Axum backend on port 8080 (ADR-17).

### Changed
- **Redis Connection Optimization:** Pre-initialized multiplexed Tokio Redis connection in `AppState` (`state.get_redis_conn().await`), eliminating socket recreation overhead across requests.
- **Argon2id Thread Offloading:** Offloaded CPU-intensive password hashing to Tokio blocking thread pool (`tokio::task::spawn_blocking`) with calibrated parameters (19MB, 2 iterations, 1 lane) and optimized dev compilation flags (`opt-level = 3`), reducing login p95 latency from >600ms to 52ms.
- **README Redesign:** Rewrote [README.md](README.md) to provide an accessible, welcoming overview of Project Baca's literary vision, problem statement, reader features, and architecture for a general audience.

### Security
- **OWASP ASVS Hardening:** Injected mandatory global security headers (`X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Strict-Transport-Security`, `Alt-Svc`); sanitized internal database errors (CWE-209); implemented IP spoofing defense prioritizing Cloudflare `CF-Connecting-IP`; added standard RFC rate-limit headers (`X-RateLimit-*`, `Retry-After`).
- **Account Abuse Controls:** 60-second OTP email cooldown; 15-minute brute-force lockout after 5 failed login attempts; JWT `jti` blacklisting on logout; multi-session revocation (`/api/v1/auth/revoke-all`); and constant-time dummy password verification preventing user email timing enumeration.

## [0.2.0] — 2026-09-25

### Added
- **Open Knowledge Format (OKF v0.2):** Full SSOT specification vault in `knowledge/` (`prd.md`, `erd.md`, `ux-flow.md`, `frd.md`, `srs.md`, `index.md`, `log.md`).
- **Graphify Knowledge Graph:** Codebase graph extraction (nodes, edges, communities) in `graphify-out/`.
- **Obsidian Vault Export:** Structured markdown vault notes in `obsidian-vault/`.
- **Workspace Agent Rules:** Agent governance in `.agents/rules/okf_memory.md`, `.agents/rules/graphify.md`, and `.agents/workflows/graphify.md`.
- **Multi-Agent Guidelines:** Added [AGENTS.md](AGENTS.md), [CLAUDE.md](CLAUDE.md), [MEMORY.md](MEMORY.md), [GEMINI.md](GEMINI.md), [GUIDE.md](GUIDE.md), [DISTRIBUTED.md](DISTRIBUTED.md), [NOTICE.md](NOTICE.md).
- **System Specifications:** 23 REST API contracts, multi-layer security model in `knowledge/srs.md`.
- **Database Schema & Index Matrix:** 16 PostgreSQL 17 indexes including partial GIN and HNSW in `knowledge/erd.md`.
- **UX Flows & Wireframes:** Navigation layout and guest-to-cloud sync flows in `knowledge/ux-flow.md`.
- **Functional Requirements:** Specifications across 7 system modules in `knowledge/frd.md`.
- **OpenAPI & Swagger UI:** Integrated `utoipa` with Swagger UI at `/swagger-ui` and raw JSON spec at `/api-docs/openapi.json`.
- **4-Tier Test Suite:** Smoke, integration, performance SLA (p95 < 50ms), and reliability tests with `mockall`, `claims`, `pretty_assertions`, and `rstest`.

### Changed
- **Embedding Engine:** Standardized on 768-dim Dual-Mode Provider (Google GenAI API and FastEmbed CPU ONNX), eliminating external Triton inference servers and halving HNSW memory usage.
- **Search Strategy:** Adopted PostgreSQL 17 FTS + `pg_trgm` GIN indexes (<3ms) in place of Elasticsearch to save memory and eliminate dual-write overhead.
- **Message Broker:** Adopted Redis Streams as the single queue for ingestion and email delivery.
- **Style Policy:** Enforced Zero Emoji Policy across all documentation and codebase.
- **Visual Theme:** Standardized on Vintage Literary (1900–1950) editorial theme with reactive bilingual switching `[ID|EN]`.

### Security
- Defense-in-depth architecture: Cloudflare edge rate limits, Redis sliding-window limits, SameSite CSRF cookies, Argon2id password hashing, and SHA-256 hashed 6-digit OTPs.
- Environment isolation: Added `.env.example` and gitignore rules for private specifications.

## [0.1.0] — 2026-09-25

### Added
- **Project Initialization:** Created monorepo blueprint [ARCHITECTURE.md](ARCHITECTURE.md) and [README.md](README.md).
- **Local Infrastructure:** Configured [docker-compose.yml](docker-compose.yml) for PostgreSQL 17 + pgvector, Redis 7, MinIO, and Mailpit.
- **Legal Copyright Analysis:** Verified public domain copyright compliance (life of author + 70 years under Indonesian Law 28/2014 and international terms).
- **Visual Asset Catalog:** Cataloged 5 vintage pen-and-ink illustrations in `assets/README.md`.
