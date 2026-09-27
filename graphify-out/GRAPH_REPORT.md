# Graph Report - project-baca  (2026-09-27)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 1271 nodes · 2452 edges · 118 communities (80 shown, 38 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 69 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `9cdbf481`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- reliability_test.rs
- books
- shared/src/lib.rs
- server/src/lib.rs
- 2. Work Breakdown
- domain/src/lib.rs
- AppError
- 2. Work Breakdown
- AuthUser
- 2. Work Breakdown
- 2. Work Breakdown
- 2. Work Breakdown
- tasks/README.md
- otp.rs
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
- .find_book_by_id
- 3. Server Health & OpenAPI Documentation
- quotes.rs
- serde
- Model
- Model
- users.rs
- chapters.rs
- reading_activity_logs.rs
- tags.rs
- Entity
- user_badges.rs
- badges.rs
- 2. Prerequisites & Service Status Verification
- 3. Catalog Discovery & Typo-Tolerant Search
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
- 4. Book Overview, Reader Content & Offline Bundle
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
- get_atomic_cards
- semantic_ai_test.rs
- catalog_test.rs
- utc
- Brain content
- brain-router/SKILL.md
- Asset Catalog
- appconfig
- memory-care/SKILL.md
- memory-recall/SKILL.md
- worker.py
- extract_text
- progress.rs
- routes/books.rs
- test_worker.py
- backfill_chunks.rs
- parse_epub
- gamification.rs
- test_worker_live.py
- HttpError
- ai_rate_limit_middleware
- Ingestion Worker (`python_worker/`)
- shared
- web/src/lib.rs

## God Nodes (most connected - your core abstractions)
1. `AppError` - 90 edges
2. `AppState` - 59 edges
3. `HttpError` - 39 edges
4. `AuthUser` - 23 edges
5. `seed_test_context()` - 18 edges
6. `AppConfig` - 17 edges
7. `parse_epub()` - 15 edges
8. `change_password()` - 15 edges
9. `login()` - 15 edges
10. `signup()` - 15 edges

## Surprising Connections (you probably didn't know these)
- `Sub-Task 1.2: Modular `AppConfig` Sub-Configurations` --references--> `env()`  [INFERRED]
  knowledge/tasks/01_infrastructure_and_config.md → python_worker/worker.py
- `Web Entry Point` --semantically_similar_to--> `Asset Catalog`  [INFERRED] [semantically similar]
  crates/web/index.html → assets/README.md
- `main()` --calls--> `build_embedding_provider()`  [INFERRED]
  crates/infra/examples/backfill_chunks.rs → crates/infra/src/ai/embedding.rs
- `main()` --calls--> `init_db_pool()`  [INFERRED]
  crates/infra/examples/backfill_chunks.rs → crates/infra/src/pool.rs
- `ai_rate_limit_middleware()` --calls--> `verify_access_token()`  [INFERRED]
  crates/server/src/middleware/ai_rate_limit.rs → crates/infra/src/security/jwt.rs

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Infrastructure Services** — redis_7 [EXTRACTED 1.00]
- **OKF v0.2 Knowledge Vault** — knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md [EXTRACTED 1.00]
- **MVP Implementation Sequence** — knowledge_tasks_01_infrastructure_and_config, knowledge_tasks_02_authentication_and_user, knowledge_tasks_03_catalog_and_reader_backend, knowledge_tasks_04_semantic_ai_and_insights, knowledge_tasks_05_ingestion_pipeline_and_admin, knowledge_tasks_06_frontend_leptos_web_reader, knowledge_tasks_07_quality_assurance_and_launch [EXTRACTED 1.00]
- **OKF Knowledge Vault** — knowledge_index_md, knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md, knowledge_log_md [EXTRACTED 1.00]
- **OKF v0.2 Specification Suite** — knowledge_index_md, knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md, knowledge_log_md [EXTRACTED 1.00]

## Communities (118 total, 38 thin omitted)

### Community 0 - "reliability_test.rs"
Cohesion: 0.09
Nodes (7): arc, assert_eq, automock, body, http, instant, testharness

### Community 1 - "books"
Cohesion: 0.07
Nodes (43): idx_users_admin_role, idx_users_email, users, book_tags, books, idx_book_tags_reverse, idx_books_catalog_filter, idx_books_published_author_trgm (+35 more)

### Community 2 - "shared/src/lib.rs"
Cohesion: 0.11
Nodes (45): chrono, ActiveProgressDto, ApiResponse, ApiResponse<T>, AtomicCardsDto, BadgeDto, BookCatalogQuery, BookDetailDto (+37 more)

### Community 3 - "server/src/lib.rs"
Cohesion: 0.10
Nodes (25): cors, api_health_check(), ApiDoc, create_app(), health_check(), not_found_handler(), REQUEST_ID_HEADER, request_id_middleware() (+17 more)

### Community 4 - "2. Work Breakdown"
Cohesion: 0.20
Nodes (10): 1. Summary, 2. Work Breakdown, 3. Success Criteria & Verification, Cross-Domain Matrix, Sub-Task 2.1: Password Hashing (Argon2id) & OTP [COMPLETED], Sub-Task 2.2: JWT Tokens & Refresh Rotation [COMPLETED], Sub-Task 2.3: REST API Endpoints (SRS 1–7) [COMPLETED], Sub-Task 2.4: Guest Reconciliation (`POST /api/v1/progress/merge`) [COMPLETED] (+2 more)

### Community 5 - "domain/src/lib.rs"
Cohesion: 0.07
Nodes (29): advance_streak(), BASE_HEARTBEAT_XP, BookStatus, COMPLETION_MAX, COMPLETION_MIN, DAILY_MILESTONE_BONUS_XP, DAILY_THRESHOLD_SECONDS, day() (+21 more)

### Community 6 - "AppError"
Cohesion: 0.10
Nodes (45): aws_sdk_s3, activate_user_by_email(), create_inactive_user(), delete_user_by_id(), find_user_by_email(), find_user_by_id(), DatabaseConnection, Model (+37 more)

### Community 7 - "2. Work Breakdown"
Cohesion: 0.15
Nodes (13): 1. Summary, 2. Work Breakdown, 3. Success Criteria, Cross-Domain Matrix, Sub-Task 6.1: Vintage Literary Design System (1900–1950), Sub-Task 6.2: API Client & Local-First IndexedDB (`rexie`), Sub-Task 6.3: Home Page (Catalog & Search), Sub-Task 6.4: Book Overview Page (+5 more)

### Community 8 - "AuthUser"
Cohesion: 0.08
Nodes (33): activemodeltrait, apperror, auth, books, AuthUser, Arc, Result, Self (+25 more)

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
Cohesion: 0.13
Nodes (26): argon2, generate_and_store_otp(), generate_numeric_otp(), hash_otp(), MAX_OTP_ATTEMPTS, OTP_TTL_SECONDS, OtpRecord, MultiplexedConnection (+18 more)

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
Cohesion: 0.09
Nodes (24): AppConfig, AuthConfig, DatabaseConfig, EmailConfig, RedisConfig, Result, Self, String (+16 more)

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

### Community 27 - "AppState"
Cohesion: 0.25
Nodes (26): AppState, Client, MultiplexedConnection, Option, Result, auth_routes(), change_password(), delete_me() (+18 more)

### Community 29 - "security_owasp_test.rs"
Cohesion: 0.26
Nodes (10): String, test_owasp_login_brute_force_lockout(), test_owasp_otp_cooldown_and_email_bombing_prevention(), test_owasp_rate_limiting_headers_and_ip_extraction(), test_owasp_revoke_all_sessions(), test_owasp_security_headers_and_error_handling(), test_owasp_structured_validation_error_details(), test_owasp_timing_attack_mitigation_on_login() (+2 more)

### Community 31 - "2. Step-by-Step Test Procedure"
Cohesion: 0.15
Nodes (13): 2. Step-by-Step Test Procedure, Step 2.10: Logout (Revoke Session & Blacklist Access Token), Step 2.11: Revoke All Sessions (`POST /api/v1/auth/revoke-all`), Step 2.12: Throttling, Cooldown & Brute-Force Lockout, Step 2.1: Register a New User Account, Step 2.2: Retrieve OTP from Mailpit, Step 2.3: Verify OTP and Activate Account, Step 2.4: Inspect Authenticated User Profile (+5 more)

### Community 33 - "Model"
Cohesion: 0.18
Nodes (9): Entity, Model, Relation, DateTimeWithTimeZone, Option, Related, RelationDef, String (+1 more)

### Community 34 - "manual-test/README.md"
Cohesion: 0.20
Nodes (6): 1. Overview, Manual Testing — Milestone 02: Authentication, Security & Guest Progress, 1. Directory Structure & Milestone Modules, 2. Server & Service URLs Quick Reference, 3. General Testing Workflow, Project Baca — Manual Testing Documentation

### Community 35 - "Manual Testing — Milestone 03: Catalog, Reader Engine & Gamification"
Cohesion: 0.20
Nodes (10): 1. Overview, 2. Quick Data Seeder for Manual Testing, 5. Reading Progress, CFI Anchors & Active Position, 6. Gamification: Heartbeats, Streaks & Badges, Manual Testing — Milestone 03: Catalog, Reader Engine & Gamification, Step 3.10: Send Reading Heartbeat (`POST /api/v1/activity/heartbeat`), Step 3.11: View Master Badges (`GET /api/v1/badges`), Step 3.12: View User Unlocked Badges (`GET /api/v1/me/badges`) (+2 more)

### Community 43 - "sea_orm"
Cohesion: 0.12
Nodes (18): init_db_pool(), init_redis_client(), Client, DatabaseConnection, Result, main(), Box, Error (+10 more)

### Community 46 - "Model"
Cohesion: 0.33
Nodes (6): Model, DateTimeWithTimeZone, Json, Option, String, Uuid

### Community 47 - ".find_book_by_id"
Cohesion: 0.25
Nodes (7): BookCatalogPort, Option, Result, Send, Sync, Uuid, test_reliability_mock_repository_fault_injection()

### Community 48 - "3. Server Health & OpenAPI Documentation"
Cohesion: 0.40
Nodes (5): 3. Server Health & OpenAPI Documentation, Step 1.5: Start the Axum Server, Step 1.6: Query Server Health Endpoint, Step 1.7: Query Infrastructure Connectivity Endpoint, Step 1.8: Access Swagger UI in Browser

### Community 49 - "quotes.rs"
Cohesion: 0.14
Nodes (34): chapter_dropoff(), get_book_by_id(), get_chapter_book_id(), get_chapter_by_number(), get_chapter_number(), get_offline_bundle(), get_tags_for_book(), list_books() (+26 more)

### Community 50 - "serde"
Cohesion: 0.21
Nodes (8): Relation, Model, Relation, Uuid, Relation, Relation, prelude, serde

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

### Community 59 - "badges.rs"
Cohesion: 0.40
Nodes (4): Model, Relation, DateTimeWithTimeZone, String

### Community 60 - "2. Prerequisites & Service Status Verification"
Cohesion: 0.25
Nodes (8): 1. Overview, 2. Prerequisites & Service Status Verification, 4. Structured Log Inspection, Manual Testing — Milestone 01: Infrastructure & Configuration, Step 1.1: Verify Docker Containers, Step 1.2: Check PostgreSQL Schema & Extensions, Step 1.3: Check Redis Connectivity, Step 1.4: Check Mailpit & MinIO HTTP Endpoints

### Community 61 - "3. Catalog Discovery & Typo-Tolerant Search"
Cohesion: 0.40
Nodes (5): 3. Catalog Discovery & Typo-Tolerant Search, Step 3.1: Browse Catalog (`GET /api/v1/books`), Step 3.2: Filter by Theme & Language, Step 3.3: Typo-Tolerant FTS Search (`GET /api/v1/books/search`), Step 3.4: Query Validation

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

### Community 74 - "4. Book Overview, Reader Content & Offline Bundle"
Cohesion: 0.50
Nodes (4): 4. Book Overview, Reader Content & Offline Bundle, Step 3.5: Book Overview & Chapter Summary (`GET /api/v1/books/{id}`), Step 3.6: Reader Chapter Delivery (`GET /api/v1/books/{id}/chapters/{chapter_number}`), Step 3.7: Offline Bundle Synchronization (`GET /api/v1/books/{id}/offline-bundle`)

### Community 89 - "Project Baca"
Cohesion: 0.25
Nodes (8): 1. What is Project Baca?, 2. Why Project Baca? (The Problem We Solve), 3. Reader Experience & Key Features, 4. Architectural Highlights & Technology, 5. Quickstart for Developers & Self-Hosters, 6. Project Documentation & Specifications, 7. Open Access & Heritage Notice, Project Baca

### Community 90 - "queue.rs"
Cohesion: 0.08
Nodes (32): ai, apiresponse, asynccommands, get_job_status(), INGESTION_GROUP, INGESTION_STREAM, job_key(), JOB_KEY_PREFIX (+24 more)

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
Cohesion: 0.06
Nodes (54): Model, DateTimeWithTimeZone, String, Uuid, AdminContext, BOUNDARY, multipart_body(), Body (+46 more)

### Community 95 - "catalog_test.rs"
Cohesion: 0.33
Nodes (10): Result, String, Uuid, seed_test_catalog(), SeededCatalog, test_book_overview_chapter_and_offline_bundle(), test_catalog_listing_and_filtering(), test_catalog_typo_tolerant_fts_search() (+2 more)

### Community 96 - "utc"
Cohesion: 0.36
Nodes (9): list_badges(), list_user_badges(), DatabaseConnection, Result, Uuid, Vec, seed_default_badges_if_empty(), entities (+1 more)

### Community 97 - "Brain content"
Cohesion: 0.40
Nodes (4): Brain content, Provenance, Safety, Skills

### Community 103 - "worker.py"
Cohesion: 0.14
Nodes (26): dataclasses, datetime, html_parser, json, logging, convert_cover_to_webp(), embed_batch(), ensure_group() (+18 more)

### Community 104 - "extract_text"
Cohesion: 0.12
Nodes (9): HTMLParser, SanitizeTest, extract_text(), __init__(), Returns cleaned HTML containing only allowlisted semantic tags., Returns plain whitespace-normalized text (tags and entities resolved)., Allowlist sanitizer: keeps semantic tags, drops scripts/styles/attrs., sanitize_html() (+1 more)

### Community 105 - "progress.rs"
Cohesion: 0.23
Nodes (15): crate, get_active_progress(), merge_guest_progress(), progress_routes(), Arc, Json, Path, Response (+7 more)

### Community 106 - "routes/books.rs"
Cohesion: 0.36
Nodes (14): books_routes(), get_book(), get_chapter(), get_offline_bundle(), list_books(), Arc, Path, Query (+6 more)

### Community 107 - "test_worker.py"
Cohesion: 0.18
Nodes (10): ArchiveLimitsTest, ChunkTest, Unit tests for the ingestion worker's pure helpers (stdlib only). Run:…, check_archive_limits(), chunk_words(), _opf_path(), Splits text into ~target-word chunks at whitespace boundaries. A trailing…, Rejects zip bombs before any entry is extracted. (+2 more)

### Community 108 - "backfill_chunks.rs"
Cohesion: 0.19
Nodes (12): BATCH_SIZE, chunk_text(), CHUNK_WORDS, main(), MIN_CHARS, Box, Error, Result (+4 more)

### Community 109 - "parse_epub"
Cohesion: 0.24
Nodes (7): make_epub(), ParseTest, Builds a minimal valid EPUB in memory., parse_epub(), ParsedChapter, ParsedEpub, Parses EPUB bytes into metadata, optional cover, and spine-ordered chapters.

### Community 110 - "gamification.rs"
Cohesion: 0.40
Nodes (10): gamification_routes(), list_badges(), list_user_badges(), record_heartbeat(), Arc, Json, Response, Result (+2 more)

### Community 112 - "test_worker_live.py"
Cohesion: 0.20
Nodes (7): os, DlqTest, Live worker tests: poison jobs reach the DLQ after 3 attempts. Needs Redis +…, redis, skipUnless, subprocess, sys

### Community 113 - "HttpError"
Cohesion: 0.28
Nodes (7): HttpError, From, IntoResponse, Response, Self, DbErr, ValidationErrors

### Community 114 - "ai_rate_limit_middleware"
Cohesion: 0.28
Nodes (9): ai_rate_limit_middleware(), get_client_ip(), Arc, Body, Next, Request, Response, State (+1 more)

### Community 115 - "Ingestion Worker (`python_worker/`)"
Cohesion: 0.25
Nodes (7): Environment, Ingestion Worker (`python_worker/`), Job lifecycle, Notes & deviations, Run, Setup, Unit tests (no services required)

### Community 116 - "shared"
Cohesion: 0.29
Nodes (5): axum, AI_MAX_REQUESTS, AI_WINDOW_SECONDS, shared, verify_access_token

## Knowledge Gaps
- **177 isolated node(s):** `ApiDoc`, `Relation`, `Relation`, `Relation`, `Relation` (+172 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 499 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **38 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppError` connect `AppError` to `utc`, `shared/src/lib.rs`, `AuthUser`, `sea_orm`, `otp.rs`, `embedding.rs`, `.find_book_by_id`, `AppConfig`, `quotes.rs`, `quote_repository.rs`, `progress_repository.rs`, `HttpError`, `queue.rs`, `AppState`, `get_atomic_cards`?**
  _High betweenness centrality (0.159) - this node is a cross-community bridge._
- **Why does `env()` connect `2. Work Breakdown` to `worker.py`?**
  _High betweenness centrality (0.113) - this node is a cross-community bridge._
- **What connects `ApiDoc`, `Relation`, `Relation` to the rest of the system?**
  _177 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `reliability_test.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.08912655971479501 - nodes in this community are weakly interconnected._
- **Should `books` be split into smaller, more focused modules?**
  _Cohesion score 0.07428571428571429 - nodes in this community are weakly interconnected._
- **Should `shared/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.11436170212765957 - nodes in this community are weakly interconnected._
- **Should `server/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.09971509971509972 - nodes in this community are weakly interconnected._