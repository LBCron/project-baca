# Changelog — Project Baca

All notable changes are documented chronologically following [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Milestone 05 (Ingestion Pipeline & Admin):** MinIO `StorageService`; admin upload with EPUB/50MB validation (202 + Streams dispatch); Python worker (parse, sanitize, chunk, batch embeddings, atomic cards/recaps, WebP covers, DLQ + reclaim); job monitor and drop-off funnel endpoints with OpenAPI docs.

### Added
- **Milestone 04 (Semantic AI & Insights):** Dual-mode embedding provider (Gemini `text-embedding-004` REST + FastEmbed CPU stub); scoped HNSW quote search (`POST /api/v1/books/{id}/quotes/search`, SRS 17); atomic cards + spoiler-free recap from `tldr_cache` (SRS 18/19); saved quotes + vintage SVG card export (SRS 20b/20c/20d); per-user AI rate limiting (10 req/min).
- **Domain Core (ADR-18):** Restored `crates/domain` as pure business rules (`Percentage`, `BookStatus`, `advance_streak`, `eligible_badges`, `merge_percentage`), wired into `progress_repository.rs`; 13 DB-free unit tests.
- **Dev Backfill Tool:** `crates/infra/examples/backfill_chunks.rs` (idempotent chapter chunking + batched embeddings; replaced by the Task 05 worker later).

### Fixed
- **Quote API contracts:** Unknown-book search now returns contracted 404 (was 200 `[]`); quote save validates the (book, chapter) pair (404 unknown, 400 cross-book mismatch; was 500 FK violation).
- **Test honesty:** Semantic tests seed embedded chunks and assert ranked, scoped, non-empty results; added per-book isolation, 404, and pair-validation tests (95 green workspace-wide).
- **Docs SSOT:** SRS token lifetimes/Argon2 params, ERD migration 07, 13-table counts, canonical Redis Stream name, dual route-mount note, admin Screens 6–7 in UX flow.

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
