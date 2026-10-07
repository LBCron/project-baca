# Project Baca: Architecture Blueprint & Technical Design

Technical architecture, polyglot monorepo structure, Rust-friendly Domain-Driven Design (DDD), AI/NLP pipeline, infrastructure layout, and core engineering invariants for **Project Baca**.

## 1. Product Vision

1. **Deep Reading Experience (Kindle & Apple Books Style):** Clean typography, reflowable layout, tap-to-turn pagination, distraction-free reading, and offline-first storage via IndexedDB.
2. **Atomic Insights & Quote Discovery (Blinkist & Deepstash Style):** Chapter-level atomic insight cards, spoiler-free recap of previous chapters, and scoped quote finder on book overview pages.
3. **Public Domain Catalog:** Curated, legal catalog from open sources (Standard Ebooks, Project Gutenberg, Wikisource).

## 2. Polyglot Monorepo Structure

```text
project-baca/
├── Cargo.toml                       # Workspace manifest (Rust)
├── Makefile                         # Developer automation (dev, test, migrations)
├── docker-compose.yml               # PostgreSQL 17 pgvector, Redis, MinIO, Mailpit
├── README.md                        # Project overview and quickstart
├── ARCHITECTURE.md                  # Monorepo architecture blueprint (this document)
├── PROJECT_LOG.md                   # Operational sprint backlog and milestones
├── AGENTS.md                        # AI coding agent operational guidelines
├── CLAUDE.md                        # Claude Code guidelines
├── MEMORY.md                        # Architectural Decision Records (ADRs)
├── GEMINI.md                        # Antigravity & Gemini guidelines
├── GUIDE.md                         # Developer guide and local setup
├── DISTRIBUTED.md                   # Ingestion worker and distributed architecture
├── CHANGELOG.md                     # SemVer release history
├── NOTICE.md                        # Legal and public domain notices
│
├── .agents/                         # Persistent AI agent rules and workflows
│   ├── rules/                       # okf_memory.md, graphify.md
│   └── workflows/                   # graphify.md
│
├── assets/                          # Design assets and illustrations
│   ├── README.md
│   ├── illustrations/               # Pen-and-ink engravings
│   └── references/                  # UI references
│
├── knowledge/                       # Canonical specifications (OKF v0.2 SSOT)
│   ├── index.md                     # Master catalog
│   ├── prd.md                       # Product Requirements Document
│   ├── erd.md                       # Database schema and index matrix
│   ├── ux-flow.md                   # Interaction design and wireframes
│   ├── frd.md                       # Functional Requirements Document
│   ├── srs.md                       # System Requirements Specification
│   ├── log.md                       # Chronological audit trail
│   └── tasks/                       # Milestone execution specifications
│
├── migrations/                      # Paired SQL migrations (.up.sql & .down.sql)
│
├── crates/                          # Rust workspace crates (domain, shared, infra, server, web)
│   ├── domain/                      # Pure business rules and invariants, zero I/O
│   ├── shared/                      # DTOs, validation, and API contracts (Axum & WASM)
│   ├── infra/                       # Database pool (SeaORM), Redis, MinIO client
│   ├── server/                      # HTTP API gateway (Axum REST, OpenAPI, Auth)
│   └── web/                         # Client frontend (Leptos 0.7 WASM PWA)
│
├── python_worker/                   # Python AI ingestion and NLP worker
```

## 3. Domain Core (`crates/domain`)

* **Owns cross-context invariants, zero I/O:** `Percentage` (0.0-100.0, finished at 100), `BookStatus` lifecycle (`draft -> processing -> published`, `archived` from any active state), `advance_streak` transition rules, `eligible_badges` matrix, `merge_percentage` (guest-to-cloud `GREATEST` without SQL). Dependencies limited to `shared` (error contract), `chrono`, `thiserror`.
* **Ports deferred per YAGNI:** repository traits (e.g., `trait BookRepository`) stay out until a second consumer needs the same model with different meaning (ADR-18 trigger). Repositories live in `crates/infra/src/repositories/` and call domain functions for rule evaluation.
* **Layering:** `server` (handlers/DTOs) -> `infra` (adapters/SeaORM) -> `domain` (rules). `server` must not depend on `domain` directly.
* **Shared DTOs (`crates/shared`):** Single contract definitions compiled to native code for Axum and WebAssembly for Leptos.

## 4. Ingestion Pipeline & Semantic AI Architecture

```text
1. INGESTION PIPELINE
   [Admin Upload EPUB] -> [Axum API] -> Store raw EPUB to S3/MinIO
                                │
                                ▼
                       [Redis Streams Queue]
                                │
                                ▼
                       [Python NLP Worker]
                         ├── Parse EPUB XHTML per chapter
                         ├── Clean HTML & detect scene breaks
                         ├── Extract metadata, cover, & calculate word count
                         ├── Split text into chunks (~300-500 words)
                         ├── Generate embeddings (Google GenAI / FastEmbed 768-dim)
                         └── Persist to PostgreSQL 17 pgvector (book_chunks)

2. CATALOG SEARCH (PostgreSQL 17 FTS + pg_trgm)
   [User: Query "Sherlock" or "Adventure"]
             │
             ▼
   [Axum: GET /api/books?q=sherlock&lang=en]
             │
             ▼
   [PostgreSQL GIN FTS + Trigram Index] (<5ms latency)
   * Typo-tolerant lexical search without Elasticsearch JVM overhead.

3. SCOPED QUOTE FINDER (PostgreSQL 17 pgvector HNSW)
   [User: "Find quote about love and sacrifice"]
             │
             ▼
   [Axum: POST /api/books/{id}/quotes/search]
             │
             ▼
   [PostgreSQL HNSW Vector Query] (<10ms latency)
     WHERE book_id = $1 ORDER BY embedding <=> $query_vector LIMIT 5;

4. ATOMIC INSIGHT CARDS & CATCH-UP RECAP
   [User: Open chapter or click "Recap Previous Chapters"]
             │
             ▼
   [Check tldr_cache in PostgreSQL]
     ├── Cache Hit: Return cached summary instantly (0 token cost)
     └── Cache Miss: Worker generates atomic cards via LLM
```

## 5. Frontend Leptos WASM & PWA Architecture

* **Shell and reader split:** `ShellLayout` (header, phone tab bar, auth sheet, toasts) wraps `/`, `/book/:id`, `/me`, `/admin`; `/read/:id` renders without chrome. Root contexts: one language signal (`i18n::provide_lang`), `Session` (token, profile, sheet state), `Toasts`, and the shell theme (`theme::ShellTheme`).
* **Paginated reader:** chapter HTML flows into CSS columns whose gap equals twice the side padding, so one page stride is exactly the viewport width (one column, two from 1024px). Turns: tap zones, swipe, arrow keys. The position is anchored to the first visible paragraph (`p-N`), saved debounced after each turn (account via `PUT /progress/{book}`, guest via IndexedDB), and heartbeats report real elapsed seconds once a minute while the tab is visible.
* **Offline Storage:** Downloaded chapters and book metadata stored in browser IndexedDB via `rexie`; the reader and shelf fall back to them when the network is gone.
* **Reactive i18n (`[ ID | EN ]`):** Table-driven dictionary in `crates/web/src/i18n/mod.rs` (register: literate, "kamu"); default follows the browser language, persisted as `rotaria_lang`. Reader preferences persist as `rotaria_reader_prefs`, the shell theme as `rotaria_theme` (applied by `boot.js` before first paint).

## 6. Visual Design Identity (Vintage Literary, two surfaces)

* **Espresso shell:** `--bg: #1F1916`, elevated `#29211C`, border `#362C27`, text `#F0EAE1`, clay accent `#CE734E`; Paper shell available via the header toggle (`data-theme="paper"`).
* **Reading surfaces:** Paper `#F9F6F0`/`#2B2625`/`#9D5A3C`, Sepia `#EFE4CF`, Espresso `#1F1916`, chosen in the reader's type sheet.
* **Typography:** `EB Garamond` display (`--font-display`), `Newsreader` reading body (`--font-reading`), `Plus Jakarta Sans` interface labels (`--font-ui`); fleuron `❖` as ornament; chapter numerals in roman.
* **Shape and motion:** 20px card radius, pill controls, 44px touch targets; fade-up on mount, sheet slide-in, page-turn fade, all disabled under `prefers-reduced-motion`.
* **Illustrations:** Classic pen-and-ink engravings in `assets/illustrations/`, shown as paper plates over the dark shell.

## 7. Infrastructure Services (`docker-compose.yml`)

1. **PostgreSQL 17 + pgvector (Port 5433):** Relational tables and HNSW vector index.
2. **Redis 7 (Port 6380):** Cache, rate limiting, and Redis Streams message broker.
3. **MinIO (Port 9005, Console 9006):** S3-compatible storage for EPUB files and covers.
4. **Mailpit (SMTP 1025, Web UI 8025):** Local transactional email testing.
5. **Caddy Edge Gateway (Port 80/443 TCP & UDP):** Reverse proxy terminating HTTP/3 (QUIC) and HTTP/2 with automatic TLS, emitting `Alt-Svc` headers, and proxying upstream to Axum (8080) and Leptos PWA (3000/dist). Caddy also sets the SPA's security headers (CSP, HSTS, nosniff, frame denial, referrer and permissions policy) since Axum only covers API responses. *(Real-domain `up --build` and `caddy validate` still pending, see `knowledge/tasks`.)*


## 8. Data Layer, Migrations, and Automation

* **SeaORM:** Async Tokio/SQLx-based ORM in `crates/infra` providing type-safe queries and compatibility with `pgvector` and `pg_trgm`.
* **Paired SQL Migrations (`migrations/`):** Schema changes managed via explicit `<timestamp>_<name>.up.sql` and `<timestamp>_<name>.down.sql` scripts.
* **Makefile Automation:** Centralized command runners (`make dev-server`, `make dev-web`, `make migrate-up`, `make test-all`).
* **API Documentation (`utoipa`):** Compile-time checked OpenAPI 3.1 schema serving Swagger UI at `/swagger-ui`.
* **Structured Observability:** Tracing with automatic `x-request-id` propagation and NDJSON format via `LOG_FORMAT=json`.
* **Test Layout:** Functional suites are modules of one binary, `crates/server/tests/it` (smoke, integration, auth, catalog, semantic, admin, database, api-boundary, security, reliability); `performance_test` and `load_stress_test` stay separate so parallel functional tests cannot skew latency assertions. Debug builds keep `line-tables-only` debuginfo for workspace code and none for dependencies.
* **Rate Limiting:** One Redis fixed-window engine (`middleware/rate_limit.rs`) behind three policies: auth (20/min per IP), AI search (10/min per user or guest IP), and public reads (`PUBLIC_RATE_LIMIT_PER_MINUTE`, off in dev, 600 in production compose).

## 9. Development Intelligence Architecture (Quad-Layer System One)

* **Layer 0 (Jev / Laya):** Fast reflex gate (<0.2ms) for intent routing and shell safety guardrails.
* **Layer 1 (Graphify):** Codebase AST and symbol graph in `graphify-out/`.
* **Layer 2 (OKF Vault v0.2):** Specifications in `knowledge/` serving as single source of truth.
* **Layer 3 (GBrain):** Cross-session persistent memory in PostgreSQL 17 pgvector.

## 10. Core Engineering Invariants

1. **ROBUST:**
   - Zero panics in production Rust code (`unwrap()` and `expect()` prohibited in `crates/server`, `crates/domain`, `crates/infra`, `crates/shared`, `crates/web`). Errors handled through `Result<T, AppError>`.
   - Foreign key cascading constraints and check constraints enforced in PostgreSQL.
   - Graceful offline fallback in Leptos WASM via IndexedDB.
2. **SCALABLE:**
   - Stateless Axum backend enabling horizontal scaling.
   - Scoped semantic search per book (`WHERE book_id = $1`).
   - Partial GIN index filtering only active books (`WHERE status = 'published'`).
3. **EASY TO MAINTAIN:**
   - Modular crate boundaries (`domain`, `shared`, `infra`, `server`, `web`).
   - Paired reversible SQL migrations.
   - Centralized Makefile automation.
4. **DRY (Don't Repeat Yourself):**
   - Shared DTOs and validation rules in `crates/shared`.
   - Unified reactive i18n dictionary.
5. **KISS (Keep It Simple, Stupid):**
   - "Postgres for Everything": Relational data, FTS with Trigrams, and vector search in a single engine. No separate Elasticsearch cluster.
   - Redis Streams for asynchronous task queues instead of heavyweight brokers (Kafka/RabbitMQ).
   - Pragmatic, borrow-checker-friendly domain models without unnecessary OOP abstractions.
6. **YAGNI (You Aren't Gonna Need It):**
   - Strict focus on MVP requirements: public domain reading, clean typography, atomic summaries, quote discovery, and offline support. Deferring future enhancements until later phases.
