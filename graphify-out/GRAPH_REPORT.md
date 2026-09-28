# Graph Report - project-baca  (2026-09-28)

## Corpus Check
- 178 files · ~297,282 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 15 file(s) not represented in the graph (top: (none) 5, .example 1, .prod 1)

## Summary
- 1794 nodes · 3280 edges · 169 communities (124 shown, 45 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 86 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `f6947416`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- load_stress_test.rs
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
- EmbeddingProvider
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
- Manual Testing — Milestone 07: QA Gates, M7 Tracks & Launch Readiness
- 2. Step-by-Step Test Procedure
- Model
- manual-test/README.md
- Manual Testing — Milestone 03: Catalog, Reader Engine & Gamification
- ai_rate_limit.rs
- Open Knowledge Format v0.2
- PostgreSQL 17 + pgvector
- Model
- otp.rs
- Manual Testing — Milestone 06: Leptos Web Reader (Rotaria P1–P7)
- AuthUser
- prelude
- Model
- Model
- Model
- chapters.rs
- reading_activity_logs.rs
- tags.rs
- Entity
- user_badges.rs
- main
- 2. Prerequisites & Service Status Verification
- routes/admin.rs
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
- get_atomic_cards
- semantic_ai_test.rs
- web/src/storage/mod.rs
- catalog_test.rs
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
- utc
- test_worker_live.py
- gamification.rs
- HttpError
- Ingestion Worker (`python_worker/`)
- reader.rs
- reliability_test.rs
- 09a — Planning Salvage (KERJAKAN PERTAMA)
- components/insights.rs
- shared
- web/src/lib.rs
- 10c — Worker Resilience (MED, OWASP API4)
- email.rs
- m6_reader.py
- 08_documentation_closeout.md
- 09_planning_absorption.md
- pages/admin.rs
- 08b — Delete `tasks/readme.md` Twin
- 08c — Delete capture-cli Twins
- 08d — Merge `project-log-kanonis.md` into `log.md`
- 08e — Backlog Hygiene
- 08f — Tech-Debt TD-08 Update
- Audit Web Menyeluruh — 2026-09-27 (pra-M7)
- 08g — Two One-Line Doc Fixes
- 08h — Final Verify & Log (WAJIB TERAKHIR)
- 09b — Guest Local Quote Saving + Sync-on-Login (BL-01)
- 09c — Batch Quote Save Endpoint (BL-02)
- pg_backup.sh
- apiresponse
- 09d — Quote-Sync RFC (ID-01 + ID-02, docs-only)
- 09e — Per-Device Refresh Tokens (ID-03)
- 09f — tldr_cache Covering Index (TD-01, conditional)
- 09g — Legacy Draft Audit (TD-02, TANPA penghapusan)
- 09h — Test Service Isolation (TD-03)
- 09j — Carousel Polish + Lighthouse (sisa research-genz + BL-12)
- 09k — Trademark Clearance (docs-only, butuh user)
- 09l — Delete planning/ (WAJIB TERAKHIR)
- 10a — Email Fail-Open → Fail-Closed + Resend (HIGH)
- 10d — DLQ Replayable + tldr Upsert (MED)
- 10f — SRS Contracts + Numbers (HIGH-docs, OWASP API9)
- 10g — Web a11y + Error Branches (MED)
- 10h — Web Completes (LOW, US-11/US-14 + MDN caching)
- sea_orm
- Task 08: Documentation Closeout (Epic)
- 09i — Launch Remaining Checklist (BL-10/BL-11 sisa + ID-04)
- Task 10: Research Gap Closure (Epic)
- 10b — Secret Hygiene (HIGH)
- 10e — Recap Context (MED, kualitas US-06)
- 3. Saved Quotes & Vintage Cards (Authenticated, Owner-Scoped)
- 2. Home: Brand, i18n, Hero, Catalog
- 6. Auth, Guest Merge, Profile, Admin
- AiConfig

## God Nodes (most connected - your core abstractions)
1. `AppError` - 94 edges
2. `AppState` - 58 edges
3. `HttpError` - 41 edges
4. `AuthUser` - 25 edges
5. `post()` - 22 edges
6. `login()` - 19 edges
7. `get()` - 19 edges
8. `AppConfig` - 18 edges
9. `seed_test_context()` - 18 edges
10. `signup()` - 17 edges

## Surprising Connections (you probably didn't know these)
- `2. Steps` --references--> `_gemini_post()`  [INFERRED]
  knowledge/tasks/10c_worker_resilience.md → python_worker/worker.py
- `Sub-Task 1.2: Modular `AppConfig` Sub-Configurations` --references--> `env()`  [INFERRED]
  knowledge/tasks/01_infrastructure_and_config.md → python_worker/worker.py
- `1. Goal` --references--> `_gemini_post()`  [INFERRED]
  knowledge/tasks/10c_worker_resilience.md → python_worker/worker.py
- `1. Goal` --references--> `embed_batch()`  [INFERRED]
  knowledge/tasks/10c_worker_resilience.md → python_worker/worker.py
- `Web Entry Point` --semantically_similar_to--> `Asset Catalog`  [INFERRED] [semantically similar]
  crates/web/index.html → assets/README.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Infrastructure Services** — redis_7 [EXTRACTED 1.00]
- **OKF v0.2 Knowledge Vault** — knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md [EXTRACTED 1.00]
- **MVP Implementation Sequence** — knowledge_tasks_01_infrastructure_and_config, knowledge_tasks_02_authentication_and_user, knowledge_tasks_03_catalog_and_reader_backend, knowledge_tasks_04_semantic_ai_and_insights, knowledge_tasks_05_ingestion_pipeline_and_admin, knowledge_tasks_06_frontend_leptos_web_reader, knowledge_tasks_07_quality_assurance_and_launch [EXTRACTED 1.00]
- **OKF Knowledge Vault** — knowledge_index_md, knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md, knowledge_log_md [EXTRACTED 1.00]
- **OKF v0.2 Specification Suite** — knowledge_index_md, knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md, knowledge_log_md [EXTRACTED 1.00]

## Communities (169 total, 45 thin omitted)

### Community 0 - "load_stress_test.rs"
Cohesion: 0.10
Nodes (4): body, http, instant, testharness

### Community 1 - "books"
Cohesion: 0.07
Nodes (43): idx_users_admin_role, idx_users_email, users, book_tags, books, idx_book_tags_reverse, idx_books_catalog_filter, idx_books_published_author_trgm (+35 more)

### Community 2 - "api/mod.rs"
Cohesion: 0.05
Nodes (115): B, chrono, ActiveProgressDto, ApiResponse, ApiResponse<T>, AtomicCardsDto, BadgeDto, BookCatalogQuery (+107 more)

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
Cohesion: 0.10
Nodes (51): aws_sdk_s3, activate_user_by_email(), create_inactive_user(), delete_user_by_id(), find_user_by_email(), find_user_by_id(), DatabaseConnection, Model (+43 more)

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

### Community 13 - "EmbeddingProvider"
Cohesion: 0.18
Nodes (8): EmbeddingProvider, FastEmbedProvider, GeminiEmbeddingProvider, Client, Result, Send, String, Sync

### Community 14 - "embedding.rs"
Cohesion: 0.15
Nodes (19): async_trait, build_embedding_provider(), GeminiBatchEmbedRequest, GeminiBatchEmbedResponse, GeminiContent, GeminiEmbedding, GeminiEmbedRequest, GeminiEmbedResponse (+11 more)

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
Cohesion: 0.21
Nodes (30): AppState, Client, MultiplexedConnection, Option, Result, auth_routes(), change_password(), delete_me() (+22 more)

### Community 29 - "security_owasp_test.rs"
Cohesion: 0.26
Nodes (10): String, test_owasp_login_brute_force_lockout(), test_owasp_otp_cooldown_and_email_bombing_prevention(), test_owasp_rate_limiting_headers_and_ip_extraction(), test_owasp_revoke_all_sessions(), test_owasp_security_headers_and_error_handling(), test_owasp_structured_validation_error_details(), test_owasp_timing_attack_mitigation_on_login() (+2 more)

### Community 30 - "Manual Testing — Milestone 07: QA Gates, M7 Tracks & Launch Readiness"
Cohesion: 0.11
Nodes (19): 1. Overview, 2. Part A — QA Gates, 3. Part B — M7 Session Track (BL-11), 4. Part C — M7 PWA Track (BL-12), 5. Part D — Remaining Launch Checklist (Not Done, Do Not Fake), 6. Regression Checklist (Copy-Paste per Pass), Manual Testing — Milestone 07: QA Gates, M7 Tracks & Launch Readiness, Step 7.10: Service Worker Caches Assets, Never `/api/*` (+11 more)

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
Cohesion: 0.10
Nodes (21): 1. Overview, 2. Quick Data Seeder for Manual Testing, 2. Web Client Verification (M6, `http://127.0.0.1:3000`), 3. Catalog Discovery & Typo-Tolerant Search, 4. Book Overview, Reader Content & Offline Bundle, 5. Reading Progress, CFI Anchors & Active Position, 6. Gamification: Heartbeats, Streaks & Badges, Manual Testing — Milestone 03: Catalog, Reader Engine & Gamification (+13 more)

### Community 43 - "ai_rate_limit.rs"
Cohesion: 0.19
Nodes (13): AI_MAX_REQUESTS, ai_rate_limit_middleware(), AI_WINDOW_SECONDS, get_client_ip(), Arc, Body, Next, Request (+5 more)

### Community 46 - "Model"
Cohesion: 0.33
Nodes (6): Model, DateTimeWithTimeZone, Json, Option, String, Uuid

### Community 47 - "otp.rs"
Cohesion: 0.07
Nodes (43): argon2, generate_and_store_otp(), generate_numeric_otp(), hash_otp(), MAX_OTP_ATTEMPTS, OTP_LOCK_SECONDS, OTP_TTL_SECONDS, OtpRecord (+35 more)

### Community 48 - "Manual Testing — Milestone 06: Leptos Web Reader (Rotaria P1–P7)"
Cohesion: 0.15
Nodes (13): 1. Overview, 3. Book Overview: Save Offline Side Effects, 4. Reader: Zones, CFI Anchor, Typography, Heartbeat, 5. Insights: Quote Finder, Atomic Cards, Recap, PNG Export, 7. Regression Checklist (Copy-Paste per Pass), Manual Testing — Milestone 06: Leptos Web Reader (Rotaria P1–P7), Step 6.10: Heartbeat + Streak Toast, Step 6.11: Quote Drawer States (+5 more)

### Community 49 - "AuthUser"
Cohesion: 0.11
Nodes (25): apperror, AuthUser, Arc, Result, Self, String, Uuid, require_admin() (+17 more)

### Community 50 - "prelude"
Cohesion: 0.11
Nodes (16): Model, Relation, DateTimeWithTimeZone, String, Relation, Model, Relation, Uuid (+8 more)

### Community 51 - "Model"
Cohesion: 0.40
Nodes (5): Model, DateTimeWithTimeZone, Option, String, Uuid

### Community 52 - "Model"
Cohesion: 0.40
Nodes (5): Model, Date, DateTimeWithTimeZone, Option, Uuid

### Community 53 - "Model"
Cohesion: 0.40
Nodes (5): Model, DateTimeWithTimeZone, Option, String, Uuid

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

### Community 59 - "main"
Cohesion: 0.16
Nodes (13): init_db_pool(), init_redis_client(), Client, DatabaseConnection, Result, main(), Box, Error (+5 more)

### Community 60 - "2. Prerequisites & Service Status Verification"
Cohesion: 0.14
Nodes (13): 1. Overview, 2. Prerequisites & Service Status Verification, 3. Server Health & OpenAPI Documentation, 4. Structured Log Inspection, Manual Testing — Milestone 01: Infrastructure & Configuration, Step 1.1: Verify Docker Containers, Step 1.2: Check PostgreSQL Schema & Extensions, Step 1.3: Check Redis Connectivity (+5 more)

### Community 61 - "routes/admin.rs"
Cohesion: 0.13
Nodes (13): activemodeltrait, axum, Callback, crate, DropoffQuery, MAX_EPUB_BYTES, Uuid, AuthModal() (+5 more)

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
Cohesion: 0.07
Nodes (29): ai, get_job_status(), INGESTION_GROUP, INGESTION_STREAM, job_key(), JOB_KEY_PREFIX, JOB_TTL_SECS, publish_ingestion_job() (+21 more)

### Community 91 - "Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya."
Cohesion: 0.40
Nodes (4): Answer, Outcome, Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya., Source Nodes

### Community 92 - "Master Task Roadmap to MVP — Project Baca"
Cohesion: 0.33
Nodes (6): 1. Task Execution Rules, 2. Phase Map & Dependencies, 3. Task Breakdown, 4. Operational Status, Master Task Roadmap to MVP — Project Baca, Micro-task Status Board (open work — update per task, not per epic)

### Community 93 - "get_atomic_cards"
Cohesion: 0.31
Nodes (13): get_atomic_cards(), get_chapter_recap(), insights_routes(), resolve_chapter(), Arc, IntoResponse, Option, Path (+5 more)

### Community 94 - "semantic_ai_test.rs"
Cohesion: 0.07
Nodes (52): AdminContext, BOUNDARY, multipart_body(), Body, Option, Request, String, Uuid (+44 more)

### Community 95 - "web/src/storage/mod.rs"
Cohesion: 0.15
Nodes (20): clear(), db(), DB_NAME, DB_VERSION, get_all(), OfflineChapterRecord, put(), Result (+12 more)

### Community 96 - "catalog_test.rs"
Cohesion: 0.33
Nodes (10): Result, String, Uuid, seed_test_catalog(), SeededCatalog, test_book_overview_chapter_and_offline_bundle(), test_catalog_listing_and_filtering(), test_catalog_typo_tolerant_fts_search() (+2 more)

### Community 97 - "Brain content"
Cohesion: 0.40
Nodes (4): Brain content, Provenance, Safety, Skills

### Community 103 - "worker.py"
Cohesion: 0.17
Nodes (21): dataclasses, datetime, html_parser, logging, convert_cover_to_webp(), ensure_group(), generate_json(), main() (+13 more)

### Community 104 - "extract_text"
Cohesion: 0.12
Nodes (9): HTMLParser, SanitizeTest, extract_text(), __init__(), Returns cleaned HTML containing only allowlisted semantic tags., Returns plain whitespace-normalized text (tags and entities resolved)., Allowlist sanitizer: keeps semantic tags, drops scripts/styles/attrs., sanitize_html() (+1 more)

### Community 105 - "Manual Testing — Milestone 05: Ingestion Pipeline & Admin"
Cohesion: 0.12
Nodes (15): 1. Overview, 2. Admin EPUB Upload, 3. Job Monitoring & Drop-Off Analytics, 4. Python Worker Pipeline, 5. End-to-End Checklist (M4+M5 Together), Manual Testing — Milestone 05: Ingestion Pipeline & Admin, Step 5.1: RBAC — Reader and Guest Are Rejected, Step 5.2: Validation — Type, Presence, Size, Metadata (+7 more)

### Community 106 - "progress.rs"
Cohesion: 0.29
Nodes (13): get_active_progress(), merge_guest_progress(), progress_routes(), Arc, Json, Path, Response, Result (+5 more)

### Community 107 - "test_worker.py"
Cohesion: 0.16
Nodes (11): io, ArchiveLimitsTest, ChunkTest, Unit tests for the ingestion worker's pure helpers (stdlib only). Run:…, check_archive_limits(), chunk_words(), _opf_path(), Splits text into ~target-word chunks at whitespace boundaries. A trailing… (+3 more)

### Community 108 - "backfill_chunks.rs"
Cohesion: 0.13
Nodes (17): arc, BATCH_SIZE, chunk_text(), CHUNK_WORDS, main(), MIN_CHARS, Box, Error (+9 more)

### Community 109 - "parse_epub"
Cohesion: 0.20
Nodes (9): make_epub(), ParseTest, Builds a minimal valid EPUB in memory., parse_epub(), ParsedChapter, ParsedEpub, Joins an OPF-relative href without letting `..`/absolute paths escape., Parses EPUB bytes into metadata, optional cover, and spine-ordered chapters. (+1 more)

### Community 110 - "routes/books.rs"
Cohesion: 0.36
Nodes (14): books_routes(), get_book(), get_chapter(), get_offline_bundle(), list_books(), Arc, Path, Query (+6 more)

### Community 111 - "utc"
Cohesion: 0.36
Nodes (9): list_badges(), list_user_badges(), DatabaseConnection, Result, Uuid, Vec, seed_default_badges_if_empty(), entities (+1 more)

### Community 112 - "test_worker_live.py"
Cohesion: 0.16
Nodes (9): minio, os, DlqTest, Live worker tests: poison jobs reach the DLQ after 3 attempts. Needs Redis +…, redis, Purge test debris from shared dev services (dev-only, never production).…, skipUnless, subprocess (+1 more)

### Community 113 - "gamification.rs"
Cohesion: 0.21
Nodes (15): admin, auth, books, gamification_routes(), HEARTBEAT_MIN_INTERVAL_SECS, list_badges(), list_user_badges(), record_heartbeat() (+7 more)

### Community 114 - "HttpError"
Cohesion: 0.28
Nodes (7): HttpError, From, IntoResponse, Response, Self, DbErr, ValidationErrors

### Community 115 - "Ingestion Worker (`python_worker/`)"
Cohesion: 0.25
Nodes (7): Environment, Ingestion Worker (`python_worker/`), Job lifecycle, Notes & deviations, Run, Setup, Unit tests (no services required)

### Community 117 - "reader.rs"
Cohesion: 0.14
Nodes (13): bookdetaildto, catchuprecap, chapterdetaildto, closure, BookPage(), IntoView, ProfilePage(), IntoView (+5 more)

### Community 119 - "reliability_test.rs"
Cohesion: 0.14
Nodes (14): assert_eq, automock, BookCatalogPort, Option, Result, Send, Sync, Uuid (+6 more)

### Community 120 - "09a — Planning Salvage (KERJAKAN PERTAMA)"
Cohesion: 0.20
Nodes (10): 09a — Planning Salvage (KERJAKAN PERTAMA), 1. Goal, 2. Steps, 3. Verify, 4. Arsip: Item Selesai (jangan dikerjakan ulang), 4. Do NOT (tambahan), 5. Arsip: Stub Phase 2 (belum jadi task; butuh keputusan user), 6. Arsip: Riset Yang Dikonsumsi + Sisa (+2 more)

### Community 121 - "components/insights.rs"
Cohesion: 0.39
Nodes (7): api, AtomicCards(), CatchupRecap(), QuoteFinder(), IntoView, RwSignal, String

### Community 122 - "shared"
Cohesion: 0.14
Nodes (13): authmodal, IntoView, SiteHeader(), fetch_catalog(), HERO_ART, HomePage(), IntoView, Option (+5 more)

### Community 123 - "web/src/lib.rs"
Cohesion: 0.20
Nodes (9): adminpage, bookpage, components, App(), IntoView, homepage, path, profilepage (+1 more)

### Community 124 - "10c — Worker Resilience (MED, OWASP API4)"
Cohesion: 0.24
Nodes (9): 10c — Worker Resilience (MED, OWASP API4), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When, embed_batch(), _gemini_post() (+1 more)

### Community 125 - "email.rs"
Cohesion: 0.36
Nodes (7): Option, Result, send_otp_email(), smtp_step(), TcpStream, timeout, tracing

### Community 126 - "m6_reader.py"
Cohesion: 0.28
Nodes (8): asyncio, api_book_id(), check(), main(), M6 web realtime E2E (Playwright, black-box) + Rotaria P1-P7. Requires: `make…, json, sys, urllib_request

### Community 129 - "08_documentation_closeout.md"
Cohesion: 0.25
Nodes (6): 08a — Baseline Ref Check (KERJAKAN PERTAMA), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 130 - "09_planning_absorption.md"
Cohesion: 0.25
Nodes (6): 1. Hasil Research Lead (ringkas; detail penuh di 09a), 2. Urutan Eksekusi, 3. Aturan Global, 4. Non-Goal, 5. Definition of Done Epic, Task 09: Planning Absorption (Epic)

### Community 131 - "pages/admin.rs"
Cohesion: 0.39
Nodes (7): AdminPage(), job_status(), IntoView, Result, String, upload_epub(), File

### Community 132 - "08b — Delete `tasks/readme.md` Twin"
Cohesion: 0.29
Nodes (6): 08b — Delete `tasks/readme.md` Twin, 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 133 - "08c — Delete capture-cli Twins"
Cohesion: 0.29
Nodes (6): 08c — Delete capture-cli Twins, 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 134 - "08d — Merge `project-log-kanonis.md` into `log.md`"
Cohesion: 0.29
Nodes (6): 08d — Merge `project-log-kanonis.md` into `log.md`, 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 135 - "08e — Backlog Hygiene"
Cohesion: 0.29
Nodes (6): 08e — Backlog Hygiene, 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 136 - "08f — Tech-Debt TD-08 Update"
Cohesion: 0.29
Nodes (6): 08f — Tech-Debt TD-08 Update, 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 137 - "Audit Web Menyeluruh — 2026-09-27 (pra-M7)"
Cohesion: 0.40
Nodes (4): Audit Web Menyeluruh — 2026-09-27 (pra-M7), Kontrak yang legitimate (bukan bug), Screenshot, Temuan & Perbaikan (8/8 fixed, 0 page-error)

### Community 138 - "08g — Two One-Line Doc Fixes"
Cohesion: 0.29
Nodes (6): 08g — Two One-Line Doc Fixes, 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 139 - "08h — Final Verify & Log (WAJIB TERAKHIR)"
Cohesion: 0.29
Nodes (6): 08h — Final Verify & Log (WAJIB TERAKHIR), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 140 - "09b — Guest Local Quote Saving + Sync-on-Login (BL-01)"
Cohesion: 0.29
Nodes (6): 09b — Guest Local Quote Saving + Sync-on-Login (BL-01), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 141 - "09c — Batch Quote Save Endpoint (BL-02)"
Cohesion: 0.29
Nodes (6): 09c — Batch Quote Save Endpoint (BL-02), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 145 - "09d — Quote-Sync RFC (ID-01 + ID-02, docs-only)"
Cohesion: 0.29
Nodes (6): 09d — Quote-Sync RFC (ID-01 + ID-02, docs-only), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 146 - "09e — Per-Device Refresh Tokens (ID-03)"
Cohesion: 0.29
Nodes (6): 09e — Per-Device Refresh Tokens (ID-03), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 147 - "09f — tldr_cache Covering Index (TD-01, conditional)"
Cohesion: 0.29
Nodes (6): 09f — tldr_cache Covering Index (TD-01, conditional), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 148 - "09g — Legacy Draft Audit (TD-02, TANPA penghapusan)"
Cohesion: 0.29
Nodes (6): 09g — Legacy Draft Audit (TD-02, TANPA penghapusan), 1. Goal, 2. Steps (semua SELECT; tidak ada DELETE/UPDATE), 3. Verify, 4. Do NOT, 5. Done When

### Community 149 - "09h — Test Service Isolation (TD-03)"
Cohesion: 0.29
Nodes (6): 09h — Test Service Isolation (TD-03), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 150 - "09j — Carousel Polish + Lighthouse (sisa research-genz + BL-12)"
Cohesion: 0.29
Nodes (6): 09j — Carousel Polish + Lighthouse (sisa research-genz + BL-12), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 151 - "09k — Trademark Clearance (docs-only, butuh user)"
Cohesion: 0.29
Nodes (6): 09k — Trademark Clearance (docs-only, butuh user), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 152 - "09l — Delete planning/ (WAJIB TERAKHIR)"
Cohesion: 0.29
Nodes (6): 09l — Delete planning/ (WAJIB TERAKHIR), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 153 - "10a — Email Fail-Open → Fail-Closed + Resend (HIGH)"
Cohesion: 0.29
Nodes (6): 10a — Email Fail-Open → Fail-Closed + Resend (HIGH), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 154 - "10d — DLQ Replayable + tldr Upsert (MED)"
Cohesion: 0.29
Nodes (6): 10d — DLQ Replayable + tldr Upsert (MED), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 155 - "10f — SRS Contracts + Numbers (HIGH-docs, OWASP API9)"
Cohesion: 0.29
Nodes (6): 10f — SRS Contracts + Numbers (HIGH-docs, OWASP API9), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 156 - "10g — Web a11y + Error Branches (MED)"
Cohesion: 0.29
Nodes (6): 10g — Web a11y + Error Branches (MED), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 157 - "10h — Web Completes (LOW, US-11/US-14 + MDN caching)"
Cohesion: 0.29
Nodes (6): 10h — Web Completes (LOW, US-11/US-14 + MDN caching), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 159 - "Task 08: Documentation Closeout (Epic)"
Cohesion: 0.33
Nodes (6): 1. Peta Redundansi (hasil audit Lead, 2026-09-28), 2. Urutan Eksekusi, 3. Aturan Global, 4. Non-Goal (JANGAN dihapus di Task 08), 5. Definition of Done Epic, Task 08: Documentation Closeout (Epic)

### Community 160 - "09i — Launch Remaining Checklist (BL-10/BL-11 sisa + ID-04)"
Cohesion: 0.33
Nodes (5): 09i — Launch Remaining Checklist (BL-10/BL-11 sisa + ID-04), 1. Goal, 2. Verify Global, 3. Do NOT, 4. Done When

### Community 161 - "Task 10: Research Gap Closure (Epic)"
Cohesion: 0.33
Nodes (6): 1. Temuan Terverifikasi (bukti file:line), 2. Riset Adopsi (sumber primer), 3. Urutan Eksekusi, 4. Aturan Global, 5. Definition of Done Epic, Task 10: Research Gap Closure (Epic)

### Community 162 - "10b — Secret Hygiene (HIGH)"
Cohesion: 0.33
Nodes (6): 10b — Secret Hygiene (HIGH), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 163 - "10e — Recap Context (MED, kualitas US-06)"
Cohesion: 0.33
Nodes (6): 10e — Recap Context (MED, kualitas US-06), 1. Goal, 2. Steps, 3. Verify, 4. Do NOT, 5. Done When

### Community 164 - "3. Saved Quotes & Vintage Cards (Authenticated, Owner-Scoped)"
Cohesion: 0.40
Nodes (5): 3. Saved Quotes & Vintage Cards (Authenticated, Owner-Scoped), Step 4.5: Save Requires Authentication, Step 4.6: Authenticated Save + Pair Validation, Step 4.7: Vintage SVG Card Is Owner-Only, Step 4.8: List Saved Quotes

### Community 165 - "2. Home: Brand, i18n, Hero, Catalog"
Cohesion: 0.40
Nodes (5): 2. Home: Brand, i18n, Hero, Catalog, Step 6.1: Brand Renders, Step 6.2: i18n Toggle Switches the Whole Page Without Reload, Step 6.3: Carousel Rotates and Pauses, Step 6.4: Search (250 ms Debounce) and Load More

### Community 166 - "6. Auth, Guest Merge, Profile, Admin"
Cohesion: 0.40
Nodes (5): 6. Auth, Guest Merge, Profile, Admin, Step 6.12: Guest Tap-to-Save + Auto-Merge on Login, Step 6.13: Profile (`/me`) — Badges, Quotes, Logout, Step 6.14: i18n of Action Labels (18-Key Spot Check), Step 6.15: Admin Upload + Job Monitor (`/admin`)

## Knowledge Gaps
- **408 isolated node(s):** `COMPLETION_MIN`, `COMPLETION_MAX`, `DAILY_THRESHOLD_SECONDS`, `BASE_HEARTBEAT_XP`, `STREAK_BONUS_XP` (+403 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 788 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **45 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppError` connect `AppError` to `api/mod.rs`, `AiConfig`, `quotes.rs`, `AppState`, `EmbeddingProvider`, `embedding.rs`, `utc`, `otp.rs`, `AppConfig`, `HttpError`, `quote_repository.rs`, `progress_repository.rs`, `AuthUser`, `get_atomic_cards`, `reliability_test.rs`, `queue.rs`, `main`, `email.rs`?**
  _High betweenness centrality (0.193) - this node is a cross-community bridge._
- **Are the 3 inferred relationships involving `HttpError` (e.g. with `ai_rate_limit_middleware()` and `.from_request_parts()`) actually correct?**
  _`HttpError` has 3 INFERRED edges - model-reasoned connections that need verification._
- **What connects `COMPLETION_MIN`, `COMPLETION_MAX`, `DAILY_THRESHOLD_SECONDS` to the rest of the system?**
  _408 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `load_stress_test.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.09846153846153846 - nodes in this community are weakly interconnected._
- **Should `books` be split into smaller, more focused modules?**
  _Cohesion score 0.07428571428571429 - nodes in this community are weakly interconnected._
- **Should `api/mod.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05137741046831956 - nodes in this community are weakly interconnected._
- **Should `server/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.10317460317460317 - nodes in this community are weakly interconnected._