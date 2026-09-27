# Graph Report - project-baca  (2026-09-27)

## Corpus Check
- 141 files · ~237,018 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 9 file(s) not represented in the graph (top: (none) 5, .example 1, .toml 1)

## Summary
- 1461 nodes · 2805 edges · 129 communities (88 shown, 41 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 78 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `5006faaa`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- load_stress_test.rs
- books
- shared/src/lib.rs
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
- otp.rs
- Vec
- migrate.sh
- shared
- AppConfig
- 2. Work Breakdown
- quote_repository.rs
- catalog_test.rs
- web/src/main.rs
- Reader View Illustration
- Library Catalog Illustration
- Ingestion Monitor Illustration
- 2. Work Breakdown
- entities/mod.rs
- AppState
- README.md
- security_owasp_test.rs
- 2. Step-by-Step Test Procedure
- Model
- manual-test/README.md
- Manual Testing — Milestone 03: Catalog, Reader Engine & Gamification
- sea_orm
- Open Knowledge Format v0.2
- PostgreSQL 17 + pgvector
- Model
- reliability_test.rs
- Approved Feature Backlog
- admin.rs
- prelude
- Model
- Model
- users.rs
- chapters.rs
- reading_activity_logs.rs
- Model
- Entity
- Model
- embedding.rs
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
- queue.rs
- Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya.
- Master Task Roadmap to MVP — Project Baca
- book_repository.rs
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
- Model
- AuthUser
- 3. Catalog Discovery & Typo-Tolerant Search
- 3. Saved Quotes & Vintage Cards (Authenticated, Owner-Scoped)
- 2. Admin EPUB Upload
- components/insights.rs
- home.rs
- web/src/lib.rs
- email.rs
- MockEmbeddingProvider
- AiConfig

## God Nodes (most connected - your core abstractions)
1. `AppError` - 94 edges
2. `AppState` - 60 edges
3. `HttpError` - 39 edges
4. `AuthUser` - 25 edges
5. `AppConfig` - 18 edges
6. `seed_test_context()` - 18 edges
7. `post()` - 18 edges
8. `refresh()` - 16 edges
9. `signup()` - 15 edges
10. `verify_otp()` - 15 edges

## Surprising Connections (you probably didn't know these)
- `Sub-Task 1.2: Modular `AppConfig` Sub-Configurations` --references--> `env()`  [INFERRED]
  knowledge/tasks/01_infrastructure_and_config.md → python_worker/worker.py
- `Web Entry Point` --semantically_similar_to--> `Asset Catalog`  [INFERRED] [semantically similar]
  crates/web/index.html → assets/README.md
- `main()` --calls--> `build_embedding_provider()`  [INFERRED]
  crates/infra/examples/backfill_chunks.rs → crates/infra/src/ai/embedding.rs
- `main()` --calls--> `init_db_pool()`  [INFERRED]
  crates/infra/examples/backfill_chunks.rs → crates/infra/src/pool.rs
- `main()` --calls--> `build_embedding_provider()`  [INFERRED]
  crates/server/src/main.rs → crates/infra/src/ai/embedding.rs

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Infrastructure Services** — redis_7 [EXTRACTED 1.00]
- **OKF v0.2 Knowledge Vault** — knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md [EXTRACTED 1.00]
- **MVP Implementation Sequence** — knowledge_tasks_01_infrastructure_and_config, knowledge_tasks_02_authentication_and_user, knowledge_tasks_03_catalog_and_reader_backend, knowledge_tasks_04_semantic_ai_and_insights, knowledge_tasks_05_ingestion_pipeline_and_admin, knowledge_tasks_06_frontend_leptos_web_reader, knowledge_tasks_07_quality_assurance_and_launch [EXTRACTED 1.00]
- **OKF Knowledge Vault** — knowledge_index_md, knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md, knowledge_log_md [EXTRACTED 1.00]
- **OKF v0.2 Specification Suite** — knowledge_index_md, knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md, knowledge_log_md [EXTRACTED 1.00]

## Communities (129 total, 41 thin omitted)

### Community 0 - "load_stress_test.rs"
Cohesion: 0.09
Nodes (6): assert_eq, body, http, instant, testharness, uuid

### Community 1 - "books"
Cohesion: 0.07
Nodes (43): idx_users_admin_role, idx_users_email, users, book_tags, books, idx_book_tags_reverse, idx_books_catalog_filter, idx_books_published_author_trgm (+35 more)

### Community 2 - "shared/src/lib.rs"
Cohesion: 0.06
Nodes (85): B, chrono, ActiveProgressDto, ApiResponse, ApiResponse<T>, AtomicCardsDto, BadgeDto, BookCatalogQuery (+77 more)

### Community 3 - "server/src/lib.rs"
Cohesion: 0.11
Nodes (25): cors, api_health_check(), ApiDoc, create_app(), health_check(), not_found_handler(), REQUEST_ID_HEADER, request_id_middleware() (+17 more)

### Community 4 - "2. Work Breakdown"
Cohesion: 0.20
Nodes (10): 1. Summary, 2. Work Breakdown, 3. Success Criteria & Verification, Cross-Domain Matrix, Sub-Task 2.1: Password Hashing (Argon2id) & OTP [COMPLETED], Sub-Task 2.2: JWT Tokens & Refresh Rotation [COMPLETED], Sub-Task 2.3: REST API Endpoints (SRS 1–7) [COMPLETED], Sub-Task 2.4: Guest Reconciliation (`POST /api/v1/progress/merge`) [COMPLETED] (+2 more)

### Community 5 - "domain/src/lib.rs"
Cohesion: 0.07
Nodes (29): advance_streak(), BASE_HEARTBEAT_XP, BookStatus, COMPLETION_MAX, COMPLETION_MIN, DAILY_MILESTONE_BONUS_XP, DAILY_THRESHOLD_SECONDS, day() (+21 more)

### Community 6 - "AppError"
Cohesion: 0.10
Nodes (51): aws_sdk_s3, activate_user_by_email(), create_inactive_user(), delete_user_by_id(), find_user_by_email(), find_user_by_id(), DatabaseConnection, Model (+43 more)

### Community 7 - "2. Work Breakdown"
Cohesion: 0.15
Nodes (13): 1. Summary, 2. Work Breakdown, 3. Success Criteria, Cross-Domain Matrix, Sub-Task 6.1: Vintage Literary Design System (1900–1950), Sub-Task 6.2: API Client & Local-First IndexedDB (`rexie`), Sub-Task 6.3: Home Page (Catalog & Search), Sub-Task 6.4: Book Overview Page (+5 more)

### Community 8 - "quotes.rs"
Cohesion: 0.25
Nodes (17): get_quote_card(), handle_list_saved_quotes(), handle_save_quote(), quotes_routes(), render_vintage_quote_svg(), Arc, IntoResponse, Json (+9 more)

### Community 9 - "2. Work Breakdown"
Cohesion: 0.17
Nodes (12): 1. Summary, 2. Work Breakdown, 3. Success Criteria, Cross-Domain Matrix, Sub-Task 1.1: `dotenvy` Integration, Sub-Task 1.2: Modular `AppConfig` Sub-Configurations, Sub-Task 1.3: Dynamic Connection Pools, Sub-Task 1.4: SeaORM Entities for 13 Tables (+4 more)

### Community 10 - "2. Work Breakdown"
Cohesion: 0.17
Nodes (12): 1. Summary, 2. Work Breakdown, 3. Success Criteria, 4. Completion Evidence (2026-09-27), Cross-Domain Matrix, Sub-Task 5.1: S3/MinIO Storage Service (`crates/infra`), Sub-Task 5.2: Admin EPUB Upload Endpoint (SRS 21), Sub-Task 5.3: Ingestion Worker & EPUB Parsing (+4 more)

### Community 11 - "2. Work Breakdown"
Cohesion: 0.18
Nodes (11): 1. Summary, 2. Work Breakdown, 3. Success Criteria, Cross-Domain Matrix, Sub-Task 7.1: Smoke Testing Suite, Sub-Task 7.2: Unit Test Suite, Sub-Task 7.3: Integration Test Suite, Sub-Task 7.4: Performance SLA Benchmarks (+3 more)

### Community 13 - "otp.rs"
Cohesion: 0.12
Nodes (27): argon2, generate_and_store_otp(), generate_numeric_otp(), hash_otp(), MAX_OTP_ATTEMPTS, OTP_TTL_SECONDS, OtpRecord, MultiplexedConnection (+19 more)

### Community 14 - "Vec"
Cohesion: 0.19
Nodes (9): EmbeddingProvider, FastEmbedProvider, GeminiEmbeddingProvider, Client, Result, Send, String, Sync (+1 more)

### Community 15 - "migrate.sh"
Cohesion: 0.76
Nodes (6): init_table(), migrate_down(), migrate_status(), migrate_up(), psql_cmd(), migrate.sh script

### Community 16 - "shared"
Cohesion: 0.60
Nodes (5): domain, infra, server, shared, web

### Community 17 - "AppConfig"
Cohesion: 0.13
Nodes (16): AppConfig, AuthConfig, DatabaseConfig, EmailConfig, RedisConfig, Result, Self, String (+8 more)

### Community 18 - "2. Work Breakdown"
Cohesion: 0.22
Nodes (9): 1. Summary, 2. Work Breakdown, 3. Success Criteria & Verification, Cross-Domain Matrix, Sub-Task 3.1: Catalog Repository & FTS Lexical Search [COMPLETED], Sub-Task 3.2: Catalog & Chapter Endpoints (SRS 8–12) [COMPLETED], Sub-Task 3.3: Reading Progress & CFI Synchronization (SRS 13–14) [COMPLETED], Sub-Task 3.4: Reading Heartbeat, Streaks & Badges (SRS 15–16) [COMPLETED] (+1 more)

### Community 19 - "quote_repository.rs"
Cohesion: 0.30
Nodes (19): ChunkSearchRow, get_chapter_recap(), get_saved_quote_by_id(), get_tldr_cache(), list_saved_quotes(), QuoteSearchRow, DatabaseConnection, DateTime (+11 more)

### Community 20 - "catalog_test.rs"
Cohesion: 0.10
Nodes (34): Model, DateTimeWithTimeZone, String, Uuid, list_badges(), list_user_badges(), DatabaseConnection, Result (+26 more)

### Community 25 - "2. Work Breakdown"
Cohesion: 0.18
Nodes (11): 1. Summary, 2. Work Breakdown, 3. Success Criteria, 4. Known Gap (honest status), Cross-Domain Matrix, Sub-Task 4.1: Dual-Mode Embedding Provider (`crates/infra`), Sub-Task 4.2: Scoped Semantic Quote Finder (`POST /api/books/{id}/quotes/search`), Sub-Task 4.3: Chapter Atomic Insight Cards (SRS 18) (+3 more)

### Community 26 - "entities/mod.rs"
Cohesion: 0.14
Nodes (13): entity_as_badges, entity_as_bookchunks, entity_as_books, entity_as_booktags, entity_as_chapters, entity_as_readingactivitylogs, entity_as_savedquotes, entity_as_tags (+5 more)

### Community 27 - "AppState"
Cohesion: 0.25
Nodes (26): AppState, Client, MultiplexedConnection, Option, Result, auth_routes(), change_password(), delete_me() (+18 more)

### Community 29 - "security_owasp_test.rs"
Cohesion: 0.26
Nodes (10): String, test_owasp_login_brute_force_lockout(), test_owasp_otp_cooldown_and_email_bombing_prevention(), test_owasp_rate_limiting_headers_and_ip_extraction(), test_owasp_revoke_all_sessions(), test_owasp_security_headers_and_error_handling(), test_owasp_structured_validation_error_details(), test_owasp_timing_attack_mitigation_on_login() (+2 more)

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
Cohesion: 0.14
Nodes (14): 1. Overview, 2. Quick Data Seeder for Manual Testing, 4. Book Overview, Reader Content & Offline Bundle, 5. Reading Progress, CFI Anchors & Active Position, 6. Gamification: Heartbeats, Streaks & Badges, Manual Testing — Milestone 03: Catalog, Reader Engine & Gamification, Step 3.10: Send Reading Heartbeat (`POST /api/v1/activity/heartbeat`), Step 3.11: View Master Badges (`GET /api/v1/badges`) (+6 more)

### Community 43 - "sea_orm"
Cohesion: 0.12
Nodes (18): init_db_pool(), init_redis_client(), Client, DatabaseConnection, Result, main(), Box, Error (+10 more)

### Community 46 - "Model"
Cohesion: 0.33
Nodes (6): Model, DateTimeWithTimeZone, Json, Option, String, Uuid

### Community 47 - "reliability_test.rs"
Cohesion: 0.20
Nodes (9): automock, BookCatalogPort, Option, Result, Send, Sync, Uuid, test_reliability_mock_repository_fault_injection() (+1 more)

### Community 48 - "Approved Feature Backlog"
Cohesion: 0.07
Nodes (26): Approved Feature Backlog, BL-01: Guest Local Quote Saving (IndexedDB) + Sync-on-Login, BL-02: Batch Quote Save Endpoint, BL-03: Book Clubs & Social Leaderboards (PRD Phase 2), BL-04: Indie Author Self-Publishing Portal (PRD Phase 2), BL-05: Text-to-Speech / Audio Blinks (PRD Phase 2), BL-06: Mobile Native Wrapper — Tauri v2 Android/iOS (PRD Phase 2), Ditolak (dengan alasan + tanggal) (+18 more)

### Community 49 - "admin.rs"
Cohesion: 0.14
Nodes (21): activemodeltrait, require_admin(), Result, admin_routes(), dropoff_analytics(), DropoffQuery, ingestion_status(), MAX_EPUB_BYTES (+13 more)

### Community 50 - "prelude"
Cohesion: 0.11
Nodes (15): Model, Relation, DateTimeWithTimeZone, String, Relation, Model, Relation, Uuid (+7 more)

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

### Community 56 - "Model"
Cohesion: 0.50
Nodes (4): Model, DateTimeWithTimeZone, String, Uuid

### Community 57 - "Entity"
Cohesion: 0.40
Nodes (4): Entity, Option, Related, RelationDef

### Community 58 - "Model"
Cohesion: 0.50
Nodes (4): Model, DateTimeWithTimeZone, String, Uuid

### Community 59 - "embedding.rs"
Cohesion: 0.16
Nodes (17): async_trait, build_embedding_provider(), GeminiBatchEmbedRequest, GeminiBatchEmbedResponse, GeminiContent, GeminiEmbedding, GeminiEmbedRequest, GeminiEmbedResponse (+9 more)

### Community 60 - "2. Prerequisites & Service Status Verification"
Cohesion: 0.14
Nodes (13): 1. Overview, 2. Prerequisites & Service Status Verification, 3. Server Health & OpenAPI Documentation, 4. Structured Log Inspection, Manual Testing — Milestone 01: Infrastructure & Configuration, Step 1.1: Verify Docker Containers, Step 1.2: Check PostgreSQL Schema & Extensions, Step 1.3: Check Redis Connectivity (+5 more)

### Community 61 - "shared"
Cohesion: 0.25
Nodes (6): axum, Callback, AuthModal(), IntoView, RwSignal, shared

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
Nodes (11): 1. Overview, 2. Scoped Quote Search (guest-open), 4. Atomic Cards & Spoiler-Free Recaps, 5. Honest Expectations for Stub Content, Manual Testing — Milestone 04: Semantic AI, Quotes & Insights, Step 4.10: Recap Unavailable for Chapter 1 (Spoiler-Free Rule), Step 4.1: Successful Guest Search (`POST /api/v1/books/{book_id}/quotes/search`), Step 4.2: Validation Errors (+3 more)

### Community 89 - "Project Baca"
Cohesion: 0.25
Nodes (8): 1. What is Project Baca?, 2. Why Project Baca? (The Problem We Solve), 3. Reader Experience & Key Features, 4. Architectural Highlights & Technology, 5. Quickstart for Developers & Self-Hosters, 6. Project Documentation & Specifications, 7. Open Access & Heritage Notice, Project Baca

### Community 90 - "queue.rs"
Cohesion: 0.06
Nodes (44): ai, apiresponse, asynccommands, get_job_status(), INGESTION_GROUP, INGESTION_STREAM, job_key(), JOB_KEY_PREFIX (+36 more)

### Community 91 - "Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya."
Cohesion: 0.40
Nodes (4): Answer, Outcome, Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya., Source Nodes

### Community 92 - "Master Task Roadmap to MVP — Project Baca"
Cohesion: 0.40
Nodes (5): 1. Task Execution Rules, 2. Phase Map & Dependencies, 3. Task Breakdown, 4. Operational Status, Master Task Roadmap to MVP — Project Baca

### Community 93 - "book_repository.rs"
Cohesion: 0.16
Nodes (29): chapter_dropoff(), get_book_by_id(), get_chapter_book_id(), get_chapter_by_number(), get_chapter_number(), get_offline_bundle(), get_tags_for_book(), list_books() (+21 more)

### Community 94 - "semantic_ai_test.rs"
Cohesion: 0.07
Nodes (52): AdminContext, BOUNDARY, multipart_body(), Body, Option, Request, String, Uuid (+44 more)

### Community 95 - "web/src/storage/mod.rs"
Cohesion: 0.17
Nodes (18): clear(), db(), DB_NAME, DB_VERSION, get_all(), put(), Result, String (+10 more)

### Community 96 - "reader.rs"
Cohesion: 0.15
Nodes (12): bookdetaildto, catchuprecap, chapterdetaildto, closure, BookPage(), IntoView, query_param(), ReaderPage() (+4 more)

### Community 97 - "Brain content"
Cohesion: 0.40
Nodes (4): Brain content, Provenance, Safety, Skills

### Community 103 - "worker.py"
Cohesion: 0.13
Nodes (28): dataclasses, datetime, html_parser, json, logging, convert_cover_to_webp(), embed_batch(), ensure_group() (+20 more)

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
Cohesion: 0.21
Nodes (8): io, ArchiveLimitsTest, ChunkTest, Unit tests for the ingestion worker's pure helpers (stdlib only). Run:…, check_archive_limits(), chunk_words(), Splits text into ~target-word chunks at whitespace boundaries. A trailing…, Rejects zip bombs before any entry is extracted.

### Community 108 - "backfill_chunks.rs"
Cohesion: 0.19
Nodes (12): arc, BATCH_SIZE, chunk_text(), CHUNK_WORDS, main(), MIN_CHARS, Box, Error (+4 more)

### Community 109 - "parse_epub"
Cohesion: 0.24
Nodes (7): make_epub(), ParseTest, Builds a minimal valid EPUB in memory., parse_epub(), ParsedChapter, ParsedEpub, Parses EPUB bytes into metadata, optional cover, and spine-ordered chapters.

### Community 110 - "routes/books.rs"
Cohesion: 0.36
Nodes (14): books_routes(), get_book(), get_chapter(), get_offline_bundle(), list_books(), Arc, Path, Query (+6 more)

### Community 112 - "test_worker_live.py"
Cohesion: 0.16
Nodes (10): minio, os, DlqTest, Live worker tests: poison jobs reach the DLQ after 3 attempts. Needs Redis +…, redis, Purge test debris from shared dev services (dev-only, never production).…, skipUnless, subprocess (+2 more)

### Community 113 - "gamification.rs"
Cohesion: 0.23
Nodes (14): auth, books, gamification_routes(), HEARTBEAT_MIN_INTERVAL_SECS, list_badges(), list_user_badges(), record_heartbeat(), Arc (+6 more)

### Community 114 - "HttpError"
Cohesion: 0.28
Nodes (7): HttpError, From, IntoResponse, Response, Self, DbErr, ValidationErrors

### Community 115 - "Ingestion Worker (`python_worker/`)"
Cohesion: 0.25
Nodes (7): Environment, Ingestion Worker (`python_worker/`), Job lifecycle, Notes & deviations, Run, Setup, Unit tests (no services required)

### Community 116 - "Model"
Cohesion: 0.40
Nodes (5): Model, DateTimeWithTimeZone, String, Uuid, PgVector

### Community 117 - "AuthUser"
Cohesion: 0.19
Nodes (11): apperror, crate, AuthUser, Arc, Result, Self, String, Uuid (+3 more)

### Community 118 - "3. Catalog Discovery & Typo-Tolerant Search"
Cohesion: 0.40
Nodes (5): 3. Catalog Discovery & Typo-Tolerant Search, Step 3.1: Browse Catalog (`GET /api/v1/books`), Step 3.2: Filter by Theme & Language, Step 3.3: Typo-Tolerant FTS Search (`GET /api/v1/books/search`), Step 3.4: Query Validation

### Community 119 - "3. Saved Quotes & Vintage Cards (Authenticated, Owner-Scoped)"
Cohesion: 0.40
Nodes (5): 3. Saved Quotes & Vintage Cards (Authenticated, Owner-Scoped), Step 4.5: Save Requires Authentication, Step 4.6: Authenticated Save + Pair Validation, Step 4.7: Vintage SVG Card Is Owner-Only, Step 4.8: List Saved Quotes

### Community 120 - "2. Admin EPUB Upload"
Cohesion: 0.40
Nodes (5): 2. Admin EPUB Upload, Step 5.1: RBAC — Reader and Guest Are Rejected, Step 5.2: Validation — Type, Presence, Size, Metadata, Step 5.3: Happy Path — 202 Queued, Step 5.4: Failure Compensation (What to Look For)

### Community 121 - "components/insights.rs"
Cohesion: 0.33
Nodes (8): api, AtomicCards(), CatchupRecap(), QuoteFinder(), IntoView, RwSignal, String, jscast

### Community 122 - "home.rs"
Cohesion: 0.22
Nodes (9): authmodal, fetch_catalog(), HomePage(), IntoView, Option, Result, String, Vec (+1 more)

### Community 123 - "web/src/lib.rs"
Cohesion: 0.25
Nodes (7): bookpage, components, App(), IntoView, homepage, path, readerpage

### Community 124 - "email.rs"
Cohesion: 0.36
Nodes (7): Option, Result, send_otp_email(), smtp_step(), TcpStream, timeout, tracing

## Knowledge Gaps
- **228 isolated node(s):** `COMPLETION_MIN`, `COMPLETION_MAX`, `DAILY_THRESHOLD_SECONDS`, `BASE_HEARTBEAT_XP`, `STREAK_BONUS_XP` (+223 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 587 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **41 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppError` connect `AppError` to `shared/src/lib.rs`, `AppState`, `sea_orm`, `otp.rs`, `Vec`, `reliability_test.rs`, `AppConfig`, `HttpError`, `quote_repository.rs`, `catalog_test.rs`, `book_repository.rs`, `admin.rs`, `queue.rs`, `embedding.rs`, `email.rs`, `MockEmbeddingProvider`, `AiConfig`?**
  _High betweenness centrality (0.184) - this node is a cross-community bridge._
- **Why does `env()` connect `2. Work Breakdown` to `worker.py`?**
  _High betweenness centrality (0.157) - this node is a cross-community bridge._
- **What connects `COMPLETION_MIN`, `COMPLETION_MAX`, `DAILY_THRESHOLD_SECONDS` to the rest of the system?**
  _228 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `load_stress_test.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0907258064516129 - nodes in this community are weakly interconnected._
- **Should `books` be split into smaller, more focused modules?**
  _Cohesion score 0.07428571428571429 - nodes in this community are weakly interconnected._
- **Should `shared/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06332992849846783 - nodes in this community are weakly interconnected._
- **Should `server/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.10826210826210826 - nodes in this community are weakly interconnected._