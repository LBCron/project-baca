# Graph Report - project-baca  (2026-09-28)

## Corpus Check
- 154 files · ~290,982 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 9 file(s) not represented in the graph (top: (none) 5, .example 1, .toml 1)

## Summary
- 1609 nodes · 3044 edges · 142 communities (102 shown, 40 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 82 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `e76cdbb6`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- reliability_test.rs
- books
- api/mod.rs
- server/src/lib.rs
- 2. Work Breakdown
- domain/src/lib.rs
- AppError
- 2. Work Breakdown
- quotes.rs
- 2. Work Breakdown
- 2. Work Breakdown
- 2. Work Breakdown
- tasks/README.md
- Log — Project Baca Knowledge Bundle
- embedding.rs
- migrate.sh
- shared
- AppConfig
- 2. Work Breakdown
- quote_repository.rs
- progress_repository.rs
- web/src/main.rs
- Reader View Illustration
- Library Catalog Illustration
- Ingestion Monitor Illustration
- 2. Work Breakdown
- entities/mod.rs
- routes/auth.rs
- README.md
- security_owasp_test.rs
- AuthUser
- 2. Step-by-Step Test Procedure
- Model
- manual-test/README.md
- Manual Testing — Milestone 03: Catalog, Reader Engine & Gamification
- queue.rs
- Open Knowledge Format v0.2
- PostgreSQL 17 + pgvector
- Model
- sea_orm
- Idea Parking Lot
- routes/admin.rs
- prelude
- Model
- Model
- users.rs
- chapters.rs
- reading_activity_logs.rs
- tags.rs
- Entity
- user_badges.rs
- AppState
- 2. Prerequisites & Service Status Verification
- shared
- middleware/mod.rs
- Entity
- Entity
- Entity
- Entity
- Entity
- Entity
- Entity
- Entity
- Entity
- Entity
- Entity
- Manual Testing — Milestone 04: Semantic AI, Quotes & Insights
- ActiveModel
- ActiveModel
- ActiveModel
- ActiveModel
- ActiveModel
- ActiveModel
- ActiveModel
- ActiveModel
- ActiveModel
- ActiveModel
- ActiveModel
- ActiveModel
- ActiveModel
- Project Baca
- rate_limit.rs
- Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya.
- Master Task Roadmap to MVP — Project Baca
- get_atomic_cards
- semantic_ai_test.rs
- web/src/storage/mod.rs
- reader.rs
- Brain content
- brain-router/SKILL.md
- Asset Catalog
- appconfig
- memory-care/SKILL.md
- memory-recall/SKILL.md
- worker.py
- extract_text
- Manual Testing — Milestone 05: Ingestion Pipeline & Admin
- progress.rs
- test_worker.py
- backfill_chunks.rs
- parse_epub
- routes/books.rs
- test_worker_live.py
- gamification.rs
- HttpError
- Ingestion Worker (`python_worker/`)
- book.rs
- 3. Catalog Discovery & Typo-Tolerant Search
- .find_book_by_id
- 2. Admin EPUB Upload
- components/insights.rs
- home.rs
- web/src/lib.rs
- catalog_test.rs
- load_stress_test.rs
- m6_reader.py
- Approved Feature Backlog
- Tech Debt Register
- pages/admin.rs
- planning/README.md
- Gen-Z Reader Engagement Research (Rotaria Rebrand Input)
- Rotaria Brand-Name Research
- Rotaria — Narasi Brand
- M7 Launch Gap Research
- Audit Web Menyeluruh — 2026-09-27 (pra-M7)
- Knowledge Catalog Index & Directory Architecture Map (OKF v0.2)
- Model
- Brain content
- 2. Scoped Quote Search (guest-open)

## God Nodes (most connected - your core abstractions)
1. `AppError` - 94 edges
2. `AppState` - 60 edges
3. `HttpError` - 39 edges
4. `Log — Project Baca Knowledge Bundle` - 33 edges
5. `AuthUser` - 25 edges
6. `post()` - 22 edges
7. `AppConfig` - 18 edges
8. `seed_test_context()` - 18 edges
9. `get()` - 17 edges
10. `refresh()` - 16 edges

## Surprising Connections (you probably didn't know these)
- `2026-09-27 — Milestone 05 Complete: Ingestion Pipeline & Admin (ADR-19)` --references--> `check()`  [INFERRED]
  knowledge/kb-log.md → crates/web/tests_e2e/m6_reader.py
- `Sub-Task 1.2: Modular `AppConfig` Sub-Configurations` --references--> `env()`  [INFERRED]
  knowledge/tasks/01_infrastructure_and_config.md → python_worker/worker.py
- `Web Entry Point` --semantically_similar_to--> `Asset Catalog`  [INFERRED] [semantically similar]
  crates/web/index.html → assets/README.md
- `main()` --calls--> `build_embedding_provider()`  [INFERRED]
  crates/infra/examples/backfill_chunks.rs → crates/infra/src/ai/embedding.rs
- `main()` --calls--> `init_db_pool()`  [INFERRED]
  crates/infra/examples/backfill_chunks.rs → crates/infra/src/pool.rs

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Infrastructure Services** — redis_7 [EXTRACTED 1.00]
- **OKF v0.2 Knowledge Vault** — knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md [EXTRACTED 1.00]
- **MVP Implementation Sequence** — knowledge_tasks_01_infrastructure_and_config, knowledge_tasks_02_authentication_and_user, knowledge_tasks_03_catalog_and_reader_backend, knowledge_tasks_04_semantic_ai_and_insights, knowledge_tasks_05_ingestion_pipeline_and_admin, knowledge_tasks_06_frontend_leptos_web_reader, knowledge_tasks_07_quality_assurance_and_launch [EXTRACTED 1.00]
- **OKF Knowledge Vault** — knowledge_index_md, knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md, knowledge_log_md [EXTRACTED 1.00]
- **OKF v0.2 Specification Suite** — knowledge_index_md, knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md, knowledge_log_md [EXTRACTED 1.00]

## Communities (142 total, 40 thin omitted)

### Community 0 - "reliability_test.rs"
Cohesion: 0.10
Nodes (6): assert_eq, automock, body, http, instant, testharness

### Community 1 - "books"
Cohesion: 0.07
Nodes (43): idx_users_admin_role, idx_users_email, users, book_tags, books, idx_book_tags_reverse, idx_books_catalog_filter, idx_books_published_author_trgm (+35 more)

### Community 2 - "api/mod.rs"
Cohesion: 0.06
Nodes (98): B, chrono, ActiveProgressDto, ApiResponse, ApiResponse<T>, AtomicCardsDto, BadgeDto, BookCatalogQuery (+90 more)

### Community 3 - "server/src/lib.rs"
Cohesion: 0.10
Nodes (26): cors, api_health_check(), ApiDoc, create_app(), health_check(), not_found_handler(), REQUEST_ID_HEADER, request_id_middleware() (+18 more)

### Community 4 - "2. Work Breakdown"
Cohesion: 0.20
Nodes (10): 1. Summary, 2. Work Breakdown, 3. Success Criteria & Verification, Cross-Domain Matrix, Sub-Task 2.1: Password Hashing (Argon2id) & OTP [COMPLETED], Sub-Task 2.2: JWT Tokens & Refresh Rotation [COMPLETED], Sub-Task 2.3: REST API Endpoints (SRS 1–7) [COMPLETED], Sub-Task 2.4: Guest Reconciliation (`POST /api/v1/progress/merge`) [COMPLETED] (+2 more)

### Community 5 - "domain/src/lib.rs"
Cohesion: 0.07
Nodes (29): advance_streak(), BASE_HEARTBEAT_XP, BookStatus, COMPLETION_MAX, COMPLETION_MIN, DAILY_MILESTONE_BONUS_XP, DAILY_THRESHOLD_SECONDS, day() (+21 more)

### Community 6 - "AppError"
Cohesion: 0.06
Nodes (78): argon2, aws_sdk_s3, activate_user_by_email(), create_inactive_user(), delete_user_by_id(), find_user_by_email(), find_user_by_id(), DatabaseConnection (+70 more)

### Community 7 - "2. Work Breakdown"
Cohesion: 0.15
Nodes (13): 1. Summary, 2. Work Breakdown, 3. Success Criteria, Cross-Domain Matrix, Sub-Task 6.1: Vintage Literary Design System (1900–1950), Sub-Task 6.2: API Client & Local-First IndexedDB (`rexie`), Sub-Task 6.3: Home Page (Catalog & Search), Sub-Task 6.4: Book Overview Page (+5 more)

### Community 8 - "quotes.rs"
Cohesion: 0.15
Nodes (33): chapter_dropoff(), get_book_by_id(), get_chapter_book_id(), get_chapter_by_number(), get_chapter_number(), get_offline_bundle(), get_tags_for_book(), list_books() (+25 more)

### Community 9 - "2. Work Breakdown"
Cohesion: 0.17
Nodes (12): 1. Summary, 2. Work Breakdown, 3. Success Criteria, Cross-Domain Matrix, Sub-Task 1.1: `dotenvy` Integration, Sub-Task 1.2: Modular `AppConfig` Sub-Configurations, Sub-Task 1.3: Dynamic Connection Pools, Sub-Task 1.4: SeaORM Entities for 13 Tables (+4 more)

### Community 10 - "2. Work Breakdown"
Cohesion: 0.17
Nodes (12): 1. Summary, 2. Work Breakdown, 3. Success Criteria, 4. Completion Evidence (2026-09-27), Cross-Domain Matrix, Sub-Task 5.1: S3/MinIO Storage Service (`crates/infra`), Sub-Task 5.2: Admin EPUB Upload Endpoint (SRS 21), Sub-Task 5.3: Ingestion Worker & EPUB Parsing (+4 more)

### Community 11 - "2. Work Breakdown"
Cohesion: 0.15
Nodes (13): 1. Summary, 2. Work Breakdown, 3. Success Criteria, Cross-Domain Matrix, Sub-Task 7.1: Smoke Testing Suite, Sub-Task 7.2: Unit Test Suite, Sub-Task 7.3: Integration Test Suite, Sub-Task 7.3b: Web E2E Regression Gate (M6, pre-M7 audit) (+5 more)

### Community 13 - "Log — Project Baca Knowledge Bundle"
Cohesion: 0.06
Nodes (32): 2026-09-25 — Core Engineering Invariants & Zero Emoji Policy, 2026-09-25 — Dual-Mode Embeddings, Triton Elimination & Private Gitignore, 2026-09-25 — OKF v0.2 Knowledge Vault & Graphify Knowledge Graph, 2026-09-26 — Auth & User Management Performance Optimization, Query Architecture & Vulnerability Test Suite, 2026-09-26 — Cargo Workspace Initialization & 5-Crate Skeleton, 2026-09-26 — Comprehensive Testing Modules: Database Integrity, Load & Stress, and API Boundary Conformance, 2026-09-26 — Database Performance Optimization: Index Pruning, Foreign Key Coverage & Catalog Zero-Sort, 2026-09-26 — Edge Transport Architecture: Caddy HTTP/3 (QUIC) Reverse Proxy & Auto-HTTPS (+24 more)

### Community 14 - "embedding.rs"
Cohesion: 0.09
Nodes (29): async_trait, build_embedding_provider(), EmbeddingProvider, FastEmbedProvider, GeminiBatchEmbedRequest, GeminiBatchEmbedResponse, GeminiContent, GeminiEmbedding (+21 more)

### Community 15 - "migrate.sh"
Cohesion: 0.76
Nodes (6): init_table(), migrate_down(), migrate_status(), migrate_up(), psql_cmd(), migrate.sh script

### Community 16 - "shared"
Cohesion: 0.60
Nodes (5): domain, infra, server, shared, web

### Community 17 - "AppConfig"
Cohesion: 0.06
Nodes (41): AppConfig, AuthConfig, DatabaseConfig, EmailConfig, RedisConfig, Result, Self, String (+33 more)

### Community 18 - "2. Work Breakdown"
Cohesion: 0.22
Nodes (9): 1. Summary, 2. Work Breakdown, 3. Success Criteria & Verification, Cross-Domain Matrix, Sub-Task 3.1: Catalog Repository & FTS Lexical Search [COMPLETED], Sub-Task 3.2: Catalog & Chapter Endpoints (SRS 8–12) [COMPLETED], Sub-Task 3.3: Reading Progress & CFI Synchronization (SRS 13–14) [COMPLETED], Sub-Task 3.4: Reading Heartbeat, Streaks & Badges (SRS 15–16) [COMPLETED] (+1 more)

### Community 19 - "quote_repository.rs"
Cohesion: 0.30
Nodes (19): ChunkSearchRow, get_chapter_recap(), get_saved_quote_by_id(), get_tldr_cache(), list_saved_quotes(), QuoteSearchRow, DatabaseConnection, DateTime (+11 more)

### Community 20 - "progress_repository.rs"
Cohesion: 0.22
Nodes (15): Model, DateTimeWithTimeZone, String, Uuid, check_and_award_badges(), get_active_progress(), record_heartbeat(), DatabaseConnection (+7 more)

### Community 25 - "2. Work Breakdown"
Cohesion: 0.18
Nodes (11): 1. Summary, 2. Work Breakdown, 3. Success Criteria, 4. Known Gap (honest status), Cross-Domain Matrix, Sub-Task 4.1: Dual-Mode Embedding Provider (`crates/infra`), Sub-Task 4.2: Scoped Semantic Quote Finder (`POST /api/books/{id}/quotes/search`), Sub-Task 4.3: Chapter Atomic Insight Cards (SRS 18) (+3 more)

### Community 26 - "entities/mod.rs"
Cohesion: 0.14
Nodes (13): entity_as_badges, entity_as_bookchunks, entity_as_books, entity_as_booktags, entity_as_chapters, entity_as_readingactivitylogs, entity_as_savedquotes, entity_as_tags (+5 more)

### Community 27 - "routes/auth.rs"
Cohesion: 0.40
Nodes (18): change_password(), delete_me(), get_me(), login(), logout(), refresh(), revoke_all(), Arc (+10 more)

### Community 29 - "security_owasp_test.rs"
Cohesion: 0.26
Nodes (10): String, test_owasp_login_brute_force_lockout(), test_owasp_otp_cooldown_and_email_bombing_prevention(), test_owasp_rate_limiting_headers_and_ip_extraction(), test_owasp_revoke_all_sessions(), test_owasp_security_headers_and_error_handling(), test_owasp_structured_validation_error_details(), test_owasp_timing_attack_mitigation_on_login() (+2 more)

### Community 30 - "AuthUser"
Cohesion: 0.21
Nodes (10): apperror, AuthUser, Arc, Result, Self, String, Uuid, FromRequestParts (+2 more)

### Community 31 - "2. Step-by-Step Test Procedure"
Cohesion: 0.14
Nodes (14): 2. Step-by-Step Test Procedure, Step 2.10: Logout (Revoke Session & Blacklist Access Token), Step 2.11: Revoke All Sessions (`POST /api/v1/auth/revoke-all`), Step 2.12: Throttling, Cooldown & Brute-Force Lockout, Step 2.1: Register a New User Account, Step 2.2: Retrieve OTP from Mailpit, Step 2.3: Verify OTP and Activate Account, Step 2.4: Inspect Authenticated User Profile (+6 more)

### Community 33 - "Model"
Cohesion: 0.22
Nodes (8): Entity, Model, DateTimeWithTimeZone, Option, Related, RelationDef, String, Uuid

### Community 34 - "manual-test/README.md"
Cohesion: 0.22
Nodes (6): 1. Overview, Manual Testing — Milestone 02: Authentication, Security & Guest Progress, 1. Directory Structure & Milestone Modules, 2. Server & Service URLs Quick Reference, 3. General Testing Workflow, Project Baca — Manual Testing Documentation

### Community 35 - "Manual Testing — Milestone 03: Catalog, Reader Engine & Gamification"
Cohesion: 0.13
Nodes (15): 1. Overview, 2. Quick Data Seeder for Manual Testing, 2. Web Client Verification (M6, `http://127.0.0.1:3000`), 4. Book Overview, Reader Content & Offline Bundle, 5. Reading Progress, CFI Anchors & Active Position, 6. Gamification: Heartbeats, Streaks & Badges, Manual Testing — Milestone 03: Catalog, Reader Engine & Gamification, Step 3.10: Send Reading Heartbeat (`POST /api/v1/activity/heartbeat`) (+7 more)

### Community 43 - "queue.rs"
Cohesion: 0.07
Nodes (32): ai, get_job_status(), INGESTION_GROUP, INGESTION_STREAM, job_key(), JOB_KEY_PREFIX, JOB_TTL_SECS, publish_ingestion_job() (+24 more)

### Community 46 - "Model"
Cohesion: 0.33
Nodes (6): Model, DateTimeWithTimeZone, Json, Option, String, Uuid

### Community 47 - "sea_orm"
Cohesion: 0.36
Nodes (9): list_badges(), list_user_badges(), DatabaseConnection, Result, Uuid, Vec, seed_default_badges_if_empty(), entities (+1 more)

### Community 48 - "Idea Parking Lot"
Cohesion: 0.22
Nodes (8): Ditolak (dengan alasan + tanggal), ID-01: Render Kartu SVG Lokal untuk Guest, ID-02: Resolusi Konflik Sync Kutipan, ID-03: Refresh Token Rotation per Perangkat, ID-04: K6 / Criterion untuk Klaim p95, Idea Parking Lot, Produk, Teknis

### Community 49 - "routes/admin.rs"
Cohesion: 0.13
Nodes (22): activemodeltrait, require_admin(), Result, admin_routes(), dropoff_analytics(), DropoffQuery, ingestion_status(), MAX_EPUB_BYTES (+14 more)

### Community 50 - "prelude"
Cohesion: 0.17
Nodes (10): Relation, Relation, Model, Relation, Uuid, Relation, Relation, Relation (+2 more)

### Community 51 - "Model"
Cohesion: 0.29
Nodes (6): Model, Relation, DateTimeWithTimeZone, Option, String, Uuid

### Community 52 - "Model"
Cohesion: 0.29
Nodes (6): Model, Relation, Date, DateTimeWithTimeZone, Option, Uuid

### Community 53 - "users.rs"
Cohesion: 0.29
Nodes (6): Model, Relation, DateTimeWithTimeZone, Option, String, Uuid

### Community 54 - "chapters.rs"
Cohesion: 0.33
Nodes (5): Model, Relation, DateTimeWithTimeZone, String, Uuid

### Community 55 - "reading_activity_logs.rs"
Cohesion: 0.33
Nodes (5): Model, Relation, Date, DateTimeWithTimeZone, Uuid

### Community 56 - "tags.rs"
Cohesion: 0.33
Nodes (5): Model, Relation, DateTimeWithTimeZone, String, Uuid

### Community 57 - "Entity"
Cohesion: 0.40
Nodes (4): Entity, Option, Related, RelationDef

### Community 58 - "user_badges.rs"
Cohesion: 0.33
Nodes (5): Model, Relation, DateTimeWithTimeZone, String, Uuid

### Community 59 - "AppState"
Cohesion: 0.28
Nodes (8): AppState, Client, MultiplexedConnection, Option, Result, auth_routes(), Router, user_routes()

### Community 60 - "2. Prerequisites & Service Status Verification"
Cohesion: 0.14
Nodes (13): 1. Overview, 2. Prerequisites & Service Status Verification, 3. Server Health & OpenAPI Documentation, 4. Structured Log Inspection, Manual Testing — Milestone 01: Infrastructure & Configuration, Step 1.1: Verify Docker Containers, Step 1.2: Check PostgreSQL Schema & Extensions, Step 1.3: Check Redis Connectivity (+5 more)

### Community 61 - "shared"
Cohesion: 0.20
Nodes (9): Callback, crate, AuthModal(), IntoView, RwSignal, ProfilePage(), IntoView, shared (+1 more)

### Community 62 - "middleware/mod.rs"
Cohesion: 0.40
Nodes (4): ai_rate_limit_middleware, authuser, rate_limit_middleware, require_admin

### Community 63 - "Entity"
Cohesion: 0.50
Nodes (3): Entity, Related, RelationDef

### Community 64 - "Entity"
Cohesion: 0.50
Nodes (3): Entity, Related, RelationDef

### Community 65 - "Entity"
Cohesion: 0.50
Nodes (3): Entity, Related, RelationDef

### Community 66 - "Entity"
Cohesion: 0.50
Nodes (3): Entity, Related, RelationDef

### Community 67 - "Entity"
Cohesion: 0.50
Nodes (3): Entity, Related, RelationDef

### Community 68 - "Entity"
Cohesion: 0.50
Nodes (3): Entity, Related, RelationDef

### Community 69 - "Entity"
Cohesion: 0.50
Nodes (3): Entity, Related, RelationDef

### Community 70 - "Entity"
Cohesion: 0.50
Nodes (3): Entity, Related, RelationDef

### Community 71 - "Entity"
Cohesion: 0.50
Nodes (3): Entity, Related, RelationDef

### Community 72 - "Entity"
Cohesion: 0.50
Nodes (3): Entity, Related, RelationDef

### Community 73 - "Entity"
Cohesion: 0.50
Nodes (3): Entity, Related, RelationDef

### Community 74 - "Manual Testing — Milestone 04: Semantic AI, Quotes & Insights"
Cohesion: 0.17
Nodes (11): 1. Overview, 3. Saved Quotes & Vintage Cards (Authenticated, Owner-Scoped), 4. Atomic Cards & Spoiler-Free Recaps, 5. Honest Expectations for Stub Content, Manual Testing — Milestone 04: Semantic AI, Quotes & Insights, Step 4.10: Recap Unavailable for Chapter 1 (Spoiler-Free Rule), Step 4.5: Save Requires Authentication, Step 4.6: Authenticated Save + Pair Validation (+3 more)

### Community 89 - "Project Baca"
Cohesion: 0.25
Nodes (8): 1. What is Project Baca?, 2. Why Project Baca? (The Problem We Solve), 3. Reader Experience & Key Features, 4. Architectural Highlights & Technology, 5. Quickstart for Developers & Self-Hosters, 6. Project Documentation & Specifications, 7. Open Access & Heritage Notice, Project Baca

### Community 90 - "rate_limit.rs"
Cohesion: 0.10
Nodes (27): apiresponse, asynccommands, AI_MAX_REQUESTS, ai_rate_limit_middleware(), AI_WINDOW_SECONDS, get_client_ip(), Arc, Body (+19 more)

### Community 91 - "Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya."
Cohesion: 0.40
Nodes (4): Answer, Outcome, Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya., Source Nodes

### Community 92 - "Master Task Roadmap to MVP — Project Baca"
Cohesion: 0.40
Nodes (5): 1. Task Execution Rules, 2. Phase Map & Dependencies, 3. Task Breakdown, 4. Operational Status, Master Task Roadmap to MVP — Project Baca

### Community 93 - "get_atomic_cards"
Cohesion: 0.31
Nodes (13): get_atomic_cards(), get_chapter_recap(), insights_routes(), resolve_chapter(), Arc, IntoResponse, Option, Path (+5 more)

### Community 94 - "semantic_ai_test.rs"
Cohesion: 0.07
Nodes (52): AdminContext, BOUNDARY, multipart_body(), Body, Option, Request, String, Uuid (+44 more)

### Community 95 - "web/src/storage/mod.rs"
Cohesion: 0.17
Nodes (18): clear(), db(), DB_NAME, DB_VERSION, get_all(), put(), Result, String (+10 more)

### Community 96 - "reader.rs"
Cohesion: 0.40
Nodes (4): catchuprecap, chapterdetaildto, closure, use_params_map

### Community 97 - "Brain content"
Cohesion: 0.40
Nodes (4): Brain content, Provenance, Safety, Skills

### Community 103 - "worker.py"
Cohesion: 0.16
Nodes (24): dataclasses, datetime, html_parser, logging, convert_cover_to_webp(), embed_batch(), ensure_group(), _gemini_post() (+16 more)

### Community 104 - "extract_text"
Cohesion: 0.12
Nodes (9): HTMLParser, SanitizeTest, extract_text(), __init__(), Returns cleaned HTML containing only allowlisted semantic tags., Returns plain whitespace-normalized text (tags and entities resolved)., Allowlist sanitizer: keeps semantic tags, drops scripts/styles/attrs., sanitize_html() (+1 more)

### Community 105 - "Manual Testing — Milestone 05: Ingestion Pipeline & Admin"
Cohesion: 0.18
Nodes (10): 1. Overview, 3. Job Monitoring & Drop-Off Analytics, 4. Python Worker Pipeline, 5. End-to-End Checklist (M4+M5 Together), Manual Testing — Milestone 05: Ingestion Pipeline & Admin, Step 5.5: Job Status (`GET /api/v1/admin/jobs/{job_id}`), Step 5.6: Drop-Off Funnel (`GET /api/v1/admin/analytics/drop-off`), Step 5.7: Run the Worker Once (+2 more)

### Community 106 - "progress.rs"
Cohesion: 0.26
Nodes (14): get_active_progress(), merge_guest_progress(), progress_routes(), Arc, Json, Path, Response, Result (+6 more)

### Community 107 - "test_worker.py"
Cohesion: 0.18
Nodes (10): ArchiveLimitsTest, ChunkTest, Unit tests for the ingestion worker's pure helpers (stdlib only). Run:…, check_archive_limits(), chunk_words(), _opf_path(), Splits text into ~target-word chunks at whitespace boundaries. A trailing…, Rejects zip bombs before any entry is extracted. (+2 more)

### Community 108 - "backfill_chunks.rs"
Cohesion: 0.13
Nodes (17): arc, BATCH_SIZE, chunk_text(), CHUNK_WORDS, main(), MIN_CHARS, Box, Error (+9 more)

### Community 109 - "parse_epub"
Cohesion: 0.24
Nodes (7): make_epub(), ParseTest, Builds a minimal valid EPUB in memory., parse_epub(), ParsedChapter, ParsedEpub, Parses EPUB bytes into metadata, optional cover, and spine-ordered chapters.

### Community 110 - "routes/books.rs"
Cohesion: 0.36
Nodes (14): books_routes(), get_book(), get_chapter(), get_offline_bundle(), list_books(), Arc, Path, Query (+6 more)

### Community 112 - "test_worker_live.py"
Cohesion: 0.16
Nodes (9): minio, os, DlqTest, Live worker tests: poison jobs reach the DLQ after 3 attempts. Needs Redis +…, redis, Purge test debris from shared dev services (dev-only, never production).…, skipUnless, subprocess (+1 more)

### Community 113 - "gamification.rs"
Cohesion: 0.35
Nodes (11): gamification_routes(), HEARTBEAT_MIN_INTERVAL_SECS, list_badges(), list_user_badges(), record_heartbeat(), Arc, Json, Response (+3 more)

### Community 114 - "HttpError"
Cohesion: 0.22
Nodes (8): axum, HttpError, From, IntoResponse, Response, Self, DbErr, ValidationErrors

### Community 115 - "Ingestion Worker (`python_worker/`)"
Cohesion: 0.25
Nodes (7): Environment, Ingestion Worker (`python_worker/`), Job lifecycle, Notes & deviations, Run, Setup, Unit tests (no services required)

### Community 117 - "book.rs"
Cohesion: 0.15
Nodes (11): admin, auth, bookdetaildto, books, BookPage(), OfflineChapterRecord, IntoView, String (+3 more)

### Community 118 - "3. Catalog Discovery & Typo-Tolerant Search"
Cohesion: 0.40
Nodes (5): 3. Catalog Discovery & Typo-Tolerant Search, Step 3.1: Browse Catalog (`GET /api/v1/books`), Step 3.2: Filter by Theme & Language, Step 3.3: Typo-Tolerant FTS Search (`GET /api/v1/books/search`), Step 3.4: Query Validation

### Community 119 - ".find_book_by_id"
Cohesion: 0.25
Nodes (7): BookCatalogPort, Option, Result, Send, Sync, Uuid, test_reliability_mock_repository_fault_injection()

### Community 120 - "2. Admin EPUB Upload"
Cohesion: 0.40
Nodes (5): 2. Admin EPUB Upload, Step 5.1: RBAC — Reader and Guest Are Rejected, Step 5.2: Validation — Type, Presence, Size, Metadata, Step 5.3: Happy Path — 202 Queued, Step 5.4: Failure Compensation (What to Look For)

### Community 121 - "components/insights.rs"
Cohesion: 0.39
Nodes (7): AtomicCards(), CatchupRecap(), QuoteFinder(), IntoView, RwSignal, String, jscast

### Community 122 - "home.rs"
Cohesion: 0.20
Nodes (10): api, authmodal, fetch_catalog(), HERO_ART, HomePage(), IntoView, Option, Result (+2 more)

### Community 123 - "web/src/lib.rs"
Cohesion: 0.20
Nodes (9): adminpage, bookpage, components, App(), IntoView, homepage, path, profilepage (+1 more)

### Community 124 - "catalog_test.rs"
Cohesion: 0.33
Nodes (10): Result, String, Uuid, seed_test_catalog(), SeededCatalog, test_book_overview_chapter_and_offline_bundle(), test_catalog_listing_and_filtering(), test_catalog_typo_tolerant_fts_search() (+2 more)

### Community 126 - "m6_reader.py"
Cohesion: 0.24
Nodes (9): asyncio, api_book_id(), check(), main(), M6 web realtime E2E (Playwright, black-box) + Rotaria P1-P7. Requires: `make…, json, 2026-09-27 — Milestone 05 Complete: Ingestion Pipeline & Admin (ADR-19), sys (+1 more)

### Community 129 - "Approved Feature Backlog"
Cohesion: 0.20
Nodes (10): Approved Feature Backlog, BL-01: Guest Local Quote Saving (IndexedDB) + Sync-on-Login, BL-02: Batch Quote Save Endpoint, BL-03: Book Clubs & Social Leaderboards (PRD Phase 2), BL-04: Indie Author Self-Publishing Portal (PRD Phase 2), BL-05: Text-to-Speech / Audio Blinks (PRD Phase 2), BL-06: Mobile Native Wrapper — Tauri v2 Android/iOS (PRD Phase 2), BL-07: Rotaria Rebrand — P1 Editorial & i18n + Hero (Gen-Z) (+2 more)

### Community 130 - "Tech Debt Register"
Cohesion: 0.14
Nodes (13): Ditutup / Dibayar (Closed — 2026-09-27), Ditutup / Ditolak Sadar (Closed — Keputusan Tercatat), TD-01: tldr_cache Tanpa Composite Covering Index, TD-02: 255 Legacy `Unpublished Draft Manuscript` Drafts, TD-03: Test Suite Menulis ke Layanan Dev Bersama, TD-04: Upaya Simplifikasi FTS Scorer — DIBATALKAN, TD-05: SLA OTP 40ms — DIKOREKSI ke 100ms, TD-06: Save Offline Web Mati Total — DIBAYAR saat audit pra-M7 (+5 more)

### Community 131 - "pages/admin.rs"
Cohesion: 0.39
Nodes (7): AdminPage(), job_status(), IntoView, Result, String, upload_epub(), File

### Community 132 - "planning/README.md"
Cohesion: 0.31
Nodes (3): Alur Hidup Item, Isi Direktori, Project Baca — Future Planning

### Community 133 - "Gen-Z Reader Engagement Research (Rotaria Rebrand Input)"
Cohesion: 0.29
Nodes (7): 1. Streaks & gamification (what works), 2. Social features (quotes, stats, clubs), 3. Hero / landing & micro-interaction patterns, 4. Vintage/retro that feels young, not museum-like, 5. ID/EN toggle for young Indonesians, 6. Carousel / auto-rotate hero (timing + a11y), Gen-Z Reader Engagement Research (Rotaria Rebrand Input)

### Community 134 - "Rotaria Brand-Name Research"
Cohesion: 0.29
Nodes (7): 1. Latin etymology chain [F unless marked], 2. Circulating libraries, 18th c. [F unless marked], 3. Modern "library circulation" meaning, 4. Trademark/collision screen (light, no legal conclusions), 5. Brand verdict, 6. Fact vs belum-terverifikasi (ringkasan), Rotaria Brand-Name Research

### Community 135 - "Rotaria — Narasi Brand"
Cohesion: 0.33
Nodes (6): Batasan jujur (dari verdict riset), Ikon, Nama, Narasi sirkulasi, Rotaria — Narasi Brand, Tagline (kunci, tampil di hero P1)

### Community 136 - "M7 Launch Gap Research"
Cohesion: 0.33
Nodes (6): 1. PWA gap, 2. Prod deploy gap, 3. Auth hardening gap, 4. CDN/static, 5. Severity ranking, M7 Launch Gap Research

### Community 137 - "Audit Web Menyeluruh — 2026-09-27 (pra-M7)"
Cohesion: 0.50
Nodes (4): Audit Web Menyeluruh — 2026-09-27 (pra-M7), Kontrak yang legitimate (bukan bug), Screenshot, Temuan & Perbaikan (8/8 fixed, 0 page-error)

### Community 138 - "Knowledge Catalog Index & Directory Architecture Map (OKF v0.2)"
Cohesion: 0.40
Nodes (5): 1. Directory Role & Responsibility Map, 2. Core Specification Documents, 3. Progressive Disclosure Hierarchy, 4. Engineering Invariants, Knowledge Catalog Index & Directory Architecture Map (OKF v0.2)

### Community 139 - "Model"
Cohesion: 0.67
Nodes (3): Model, DateTimeWithTimeZone, String

### Community 140 - "Brain content"
Cohesion: 0.40
Nodes (4): Brain content, Provenance, Safety, Skills

### Community 141 - "2. Scoped Quote Search (guest-open)"
Cohesion: 0.40
Nodes (5): 2. Scoped Quote Search (guest-open), Step 4.1: Successful Guest Search (`POST /api/v1/books/{book_id}/quotes/search`), Step 4.2: Validation Errors, Step 4.3: Unknown Book Returns 404 (Never Empty 200), Step 4.4: AI Burst Enforcement (11 Rapid Guest Requests)

## Knowledge Gaps
- **303 isolated node(s):** `COMPLETION_MIN`, `COMPLETION_MAX`, `DAILY_THRESHOLD_SECONDS`, `BASE_HEARTBEAT_XP`, `STREAK_BONUS_XP` (+298 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 674 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **40 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `env()` connect `2. Work Breakdown` to `worker.py`?**
  _High betweenness centrality (0.285) - this node is a cross-community bridge._
- **What connects `COMPLETION_MIN`, `COMPLETION_MAX`, `DAILY_THRESHOLD_SECONDS` to the rest of the system?**
  _303 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `reliability_test.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.09852216748768473 - nodes in this community are weakly interconnected._
- **Should `books` be split into smaller, more focused modules?**
  _Cohesion score 0.07428571428571429 - nodes in this community are weakly interconnected._
- **Should `api/mod.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0594059405940594 - nodes in this community are weakly interconnected._
- **Should `server/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.10317460317460317 - nodes in this community are weakly interconnected._
- **Should `domain/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0707070707070707 - nodes in this community are weakly interconnected._