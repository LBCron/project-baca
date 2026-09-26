# Graph Report - project-baca  (2026-09-27)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 1045 nodes · 2011 edges · 103 communities (67 shown, 36 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 55 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `2f4cdcf9`
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
- TestHarness
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
- reliability_test.rs
- 3. Server Health & OpenAPI Documentation
- Model
- prelude
- Model
- Model
- users.rs
- chapters.rs
- reading_activity_logs.rs
- tags.rs
- Entity
- user_badges.rs
- serde
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
- ai_rate_limit.rs
- Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya.
- Master Task Roadmap to MVP — Project Baca
- get_atomic_cards
- semantic_ai_test.rs
- catalog_test.rs
- database_test.rs
- Brain content
- brain-router/SKILL.md
- Asset Catalog
- appconfig
- memory-care/SKILL.md
- memory-recall/SKILL.md

## God Nodes (most connected - your core abstractions)
1. `AppError` - 77 edges
2. `AppState` - 54 edges
3. `HttpError` - 36 edges
4. `AuthUser` - 20 edges
5. `AppConfig` - 17 edges
6. `change_password()` - 15 edges
7. `login()` - 15 edges
8. `signup()` - 15 edges
9. `verify_otp()` - 15 edges
10. `search_book_quotes()` - 14 edges

## Surprising Connections (you probably didn't know these)
- `Web Entry Point` --semantically_similar_to--> `Asset Catalog`  [INFERRED] [semantically similar]
  crates/web/index.html → assets/README.md
- `signup()` --calls--> `generate_and_store_otp()`  [INFERRED]
  crates/server/src/routes/auth.rs → crates/infra/src/security/otp.rs
- `verify_otp()` --calls--> `verify_and_consume_otp()`  [INFERRED]
  crates/server/src/routes/auth.rs → crates/infra/src/security/otp.rs
- `change_password()` --calls--> `hash_password_async()`  [INFERRED]
  crates/server/src/routes/auth.rs → crates/infra/src/security/password.rs
- `signup()` --calls--> `hash_password_async()`  [INFERRED]
  crates/server/src/routes/auth.rs → crates/infra/src/security/password.rs

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Infrastructure Services** — redis_7 [EXTRACTED 1.00]
- **OKF v0.2 Knowledge Vault** — knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md [EXTRACTED 1.00]
- **MVP Implementation Sequence** — knowledge_tasks_01_infrastructure_and_config, knowledge_tasks_02_authentication_and_user, knowledge_tasks_03_catalog_and_reader_backend, knowledge_tasks_04_semantic_ai_and_insights, knowledge_tasks_05_ingestion_pipeline_and_admin, knowledge_tasks_06_frontend_leptos_web_reader, knowledge_tasks_07_quality_assurance_and_launch [EXTRACTED 1.00]
- **OKF Knowledge Vault** — knowledge_index_md, knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md, knowledge_log_md [EXTRACTED 1.00]
- **OKF v0.2 Specification Suite** — knowledge_index_md, knowledge_prd_md, knowledge_erd_md, knowledge_frd_md, knowledge_srs_md, knowledge_ux_flow_md, knowledge_log_md [EXTRACTED 1.00]

## Communities (103 total, 36 thin omitted)

### Community 0 - "load_stress_test.rs"
Cohesion: 0.09
Nodes (6): assert_eq, body, http, instant, testharness, uuid

### Community 1 - "books"
Cohesion: 0.07
Nodes (43): idx_users_admin_role, idx_users_email, users, book_tags, books, idx_book_tags_reverse, idx_books_catalog_filter, idx_books_published_author_trgm (+35 more)

### Community 2 - "shared/src/lib.rs"
Cohesion: 0.10
Nodes (53): get_book_by_id(), get_chapter_by_number(), get_offline_bundle(), get_tags_for_book(), list_books(), DatabaseConnection, Result, String (+45 more)

### Community 3 - "server/src/lib.rs"
Cohesion: 0.10
Nodes (24): cors, api_health_check(), ApiDoc, create_app(), health_check(), not_found_handler(), REQUEST_ID_HEADER, request_id_middleware() (+16 more)

### Community 4 - "2. Work Breakdown"
Cohesion: 0.20
Nodes (10): 1. Summary, 2. Work Breakdown, 3. Success Criteria & Verification, Cross-Domain Matrix, Sub-Task 2.1: Password Hashing (Argon2id) & OTP [COMPLETED], Sub-Task 2.2: JWT Tokens & Refresh Rotation [COMPLETED], Sub-Task 2.3: REST API Endpoints (SRS 1–7) [COMPLETED], Sub-Task 2.4: Guest Reconciliation (`POST /api/v1/progress/merge`) [COMPLETED] (+2 more)

### Community 5 - "domain/src/lib.rs"
Cohesion: 0.28
Nodes (16): chrono, Book, BookChunk, BookStatus, Chapter, DomainError, ReadingProgress, ReadingStreak (+8 more)

### Community 6 - "AppError"
Cohesion: 0.15
Nodes (39): activate_user_by_email(), create_inactive_user(), delete_user_by_id(), find_user_by_email(), find_user_by_id(), DatabaseConnection, Model, Option (+31 more)

### Community 7 - "2. Work Breakdown"
Cohesion: 0.15
Nodes (13): 1. Summary, 2. Work Breakdown, 3. Success Criteria, Cross-Domain Matrix, Sub-Task 6.1: Vintage Literary Design System (1900–1950), Sub-Task 6.2: API Client & Local-First IndexedDB (`rexie`), Sub-Task 6.3: Home Page (Catalog & Search), Sub-Task 6.4: Book Overview Page (+5 more)

### Community 8 - "TestHarness"
Cohesion: 0.31
Nodes (8): Arc, Body, Request, Response, Router, Value, TestHarness, serviceext

### Community 9 - "2. Work Breakdown"
Cohesion: 0.18
Nodes (11): 1. Summary, 2. Work Breakdown, 3. Success Criteria, Cross-Domain Matrix, Sub-Task 1.1: `dotenvy` Integration, Sub-Task 1.2: Modular `AppConfig` Sub-Configurations, Sub-Task 1.3: Dynamic Connection Pools, Sub-Task 1.4: SeaORM Entities for 13 Tables (+3 more)

### Community 10 - "2. Work Breakdown"
Cohesion: 0.18
Nodes (11): 1. Summary, 2. Work Breakdown, 3. Success Criteria, Cross-Domain Matrix, Sub-Task 5.1: S3/MinIO Storage Service (`crates/infra`), Sub-Task 5.2: Admin EPUB Upload Endpoint (SRS 21), Sub-Task 5.3: Ingestion Worker & EPUB Parsing, Sub-Task 5.4: HTML Sanitization & Scene Chunking (+3 more)

### Community 11 - "2. Work Breakdown"
Cohesion: 0.18
Nodes (11): 1. Summary, 2. Work Breakdown, 3. Success Criteria, Cross-Domain Matrix, Sub-Task 7.1: Smoke Testing Suite, Sub-Task 7.2: Unit Test Suite, Sub-Task 7.3: Integration Test Suite, Sub-Task 7.4: Performance SLA Benchmarks (+3 more)

### Community 13 - "otp.rs"
Cohesion: 0.13
Nodes (26): argon2, generate_and_store_otp(), generate_numeric_otp(), hash_otp(), MAX_OTP_ATTEMPTS, OTP_TTL_SECONDS, OtpRecord, MultiplexedConnection (+18 more)

### Community 14 - "embedding.rs"
Cohesion: 0.09
Nodes (30): async_trait, build_embedding_provider(), EmbeddingProvider, FastEmbedProvider, GeminiBatchEmbedRequest, GeminiBatchEmbedResponse, GeminiContent, GeminiEmbedding (+22 more)

### Community 15 - "migrate.sh"
Cohesion: 0.76
Nodes (6): init_table(), migrate_down(), migrate_status(), migrate_up(), psql_cmd(), migrate.sh script

### Community 16 - "shared"
Cohesion: 0.70
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
Cohesion: 0.20
Nodes (10): 1. Summary, 2. Work Breakdown, 3. Success Criteria, Cross-Domain Matrix, Sub-Task 4.1: Dual-Mode Embedding Provider (`crates/infra`), Sub-Task 4.2: Scoped Semantic Quote Finder (`POST /api/books/{id}/quotes/search`), Sub-Task 4.3: Chapter Atomic Insight Cards (SRS 18), Sub-Task 4.4: Spoiler-Free Catch-up Recap (SRS 19) (+2 more)

### Community 26 - "entities/mod.rs"
Cohesion: 0.14
Nodes (13): entity_as_badges, entity_as_bookchunks, entity_as_books, entity_as_booktags, entity_as_chapters, entity_as_readingactivitylogs, entity_as_savedquotes, entity_as_tags (+5 more)

### Community 27 - "AppState"
Cohesion: 0.05
Nodes (103): apperror, axum, crate, HttpError, IntoResponse, Response, Self, AppState (+95 more)

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
Nodes (17): ai, Box, init_db_pool(), init_redis_client(), Client, DatabaseConnection, Result, main() (+9 more)

### Community 46 - "Model"
Cohesion: 0.25
Nodes (7): Model, Relation, DateTimeWithTimeZone, Json, Option, String, Uuid

### Community 47 - "reliability_test.rs"
Cohesion: 0.15
Nodes (13): arc, automock, BookCatalogPort, Option, Result, Send, Sync, Uuid (+5 more)

### Community 48 - "3. Server Health & OpenAPI Documentation"
Cohesion: 0.40
Nodes (5): 3. Server Health & OpenAPI Documentation, Step 1.5: Start the Axum Server, Step 1.6: Query Server Health Endpoint, Step 1.7: Query Infrastructure Connectivity Endpoint, Step 1.8: Access Swagger UI in Browser

### Community 49 - "Model"
Cohesion: 0.29
Nodes (6): Model, Relation, DateTimeWithTimeZone, String, Uuid, PgVector

### Community 50 - "prelude"
Cohesion: 0.20
Nodes (7): Model, Relation, Uuid, Relation, App(), IntoView, prelude

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

### Community 59 - "serde"
Cohesion: 0.33
Nodes (5): Model, Relation, DateTimeWithTimeZone, String, serde

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

### Community 90 - "ai_rate_limit.rs"
Cohesion: 0.11
Nodes (24): apiresponse, asynccommands, AI_MAX_REQUESTS, ai_rate_limit_middleware(), AI_WINDOW_SECONDS, get_client_ip(), Arc, Body (+16 more)

### Community 91 - "Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya."
Cohesion: 0.40
Nodes (4): Answer, Outcome, Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya., Source Nodes

### Community 92 - "Master Task Roadmap to MVP — Project Baca"
Cohesion: 0.40
Nodes (5): 1. Task Execution Rules, 2. Phase Map & Dependencies, 3. Task Breakdown, 4. Operational Status, Master Task Roadmap to MVP — Project Baca

### Community 93 - "get_atomic_cards"
Cohesion: 0.23
Nodes (15): auth, books, get_atomic_cards(), get_chapter_recap(), insights_routes(), resolve_chapter(), Arc, IntoResponse (+7 more)

### Community 94 - "semantic_ai_test.rs"
Cohesion: 0.27
Nodes (12): Result, String, Uuid, seed_test_context(), SeededAiContext, test_ai_rate_limiter_burst_enforcement(), test_atomic_insight_cards_cache_hit_and_miss(), test_chapter_recap_by_number_and_uuid() (+4 more)

### Community 95 - "catalog_test.rs"
Cohesion: 0.33
Nodes (10): Result, String, Uuid, seed_test_catalog(), SeededCatalog, test_book_overview_chapter_and_offline_bundle(), test_catalog_listing_and_filtering(), test_catalog_typo_tolerant_fts_search() (+2 more)

### Community 96 - "database_test.rs"
Cohesion: 0.14
Nodes (9): list_badges(), list_user_badges(), DatabaseConnection, Result, Uuid, Vec, seed_default_badges_if_empty(), entities (+1 more)

### Community 97 - "Brain content"
Cohesion: 0.40
Nodes (4): Brain content, Provenance, Safety, Skills

## Knowledge Gaps
- **155 isolated node(s):** `ApiDoc`, `Relation`, `Relation`, `Relation`, `Relation` (+150 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 408 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **36 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppError` connect `AppError` to `database_test.rs`, `shared/src/lib.rs`, `sea_orm`, `otp.rs`, `embedding.rs`, `reliability_test.rs`, `AppConfig`, `quote_repository.rs`, `progress_repository.rs`, `AppState`, `get_atomic_cards`?**
  _High betweenness centrality (0.152) - this node is a cross-community bridge._
- **Why does `AppState` connect `AppState` to `server/src/lib.rs`, `TestHarness`, `embedding.rs`, `AppConfig`, `ai_rate_limit.rs`, `get_atomic_cards`?**
  _High betweenness centrality (0.079) - this node is a cross-community bridge._
- **Why does `HttpError` connect `AppState` to `get_atomic_cards`, `AppError`?**
  _High betweenness centrality (0.040) - this node is a cross-community bridge._
- **What connects `ApiDoc`, `Relation`, `Relation` to the rest of the system?**
  _155 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `load_stress_test.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.09462365591397849 - nodes in this community are weakly interconnected._
- **Should `books` be split into smaller, more focused modules?**
  _Cohesion score 0.07428571428571429 - nodes in this community are weakly interconnected._
- **Should `shared/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.10025062656641603 - nodes in this community are weakly interconnected._