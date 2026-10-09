# Graph Report - project-baca  (2026-10-09)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 3089 nodes · 7168 edges · 246 communities (110 shown, 136 thin omitted)
- Extraction: 89% EXTRACTED · 10% INFERRED · 0% AMBIGUOUS · INFERRED: 750 edges (avg confidence: 0.87)
- Token cost: 10,657 input · 2,872 output

## Graph Freshness
- Built from commit: `f14b672f`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Knowledge Management Rules
- Ingestion Queue Processing
- UI Animation Components
- Project Release Notes
- AI Embedding Providers
- Data Transfer Objects
- Quote Repository Operations
- Reading Metrics Logic
- API Boundary Tests
- Domain Status Logic
- Database Schema Migrations
- System Architecture Layers
- Admin Web Pages
- Frontend API Utilities
- JWT Token Management
- Authentication Routes
- Shell UI Components
- System Architecture Overview
- Design System Assets
- Axum Middleware Errors
- Server Initialization
- Admin Integration Tests
- Engineering Pillars
- Admin Route Constants
- Internationalization Module
- Design Audit Tasks
- PWA Web Infrastructure
- Home Page Components
- Architecture Decision Records
- Development Invariants
- Rate Limiting Middleware
- Book Repository Operations
- Reader Theme Preferences
- Security Policy Decisions
- UI Interaction Helpers
- Database Constraint Tests
- Application State Management
- Test Harness Utilities
- Client-Side Offline Storage
- Makefile Automation
- AI Integration Tests
- E2E Accessibility Tests
- Curation Bounded Context
- Ingestion Message Contract
- Database Table Clusters
- AI Insight Endpoints
- OTP Security Logic
- S3 Storage Service
- Auth Lifecycle Tests
- UI Icon Library
- Agent Operational Rules
- Reader Feature Backlog
- Pytest Shared Fixtures
- Python Worker Scripts
- Domain Context Definitions
- Database Entity Models
- Offline PWA Requirements
- Edge Security Architecture
- Test Environment Configuration
- Email Notification Service
- Application Configuration
- User Session E2E Tests
- Gamification Database Indexes
- HTML Sanitization Utilities
- Account Lifecycle Requirements
- User Reconciliation Requirements
- Book Cover Service
- Text Chunking Utility
- AI Insight Components
- Reader Bounded Context
- API Client Wrapper
- Audit Review Rounds
- Discovery Bounded Context
- Tauri and Service Worker
- Docker Deployment Config
- Project Reliability Tasks
- Public Book Routes
- Toast Notification System
- Visual Regression Helpers
- User Repository Operations
- Gamification API Routes
- AI Insight Routes
- OWASP Security Tests
- Axe Accessibility Scans
- Worker Resilience Fixes
- Server Route Modules
- Password Hashing Logic
- Environment Config Validation
- Progress Repository Operations
- Catalog Integration Tests
- AI Memory Skills
- EPUB Parsing Logic
- AI Retry Logic Tests
- Frontend Routing App
- Responsive Design Tests
- UI Component Storybook
- Data Dictionary Documentation
- Ingestion Worker Tests
- User Session State
- Database Schema & Search
- Authentication UI Components
- Audit Reports & Screenshots
- Database Entity Relations
- WASM Build Configuration
- Book Card UI
- Docker Production Infrastructure
- Badge Repository Logic
- Navigation Integration Tests
- Saved Quotes Model
- User Data Model
- Progress UI Components
- Reader Flow Tests
- Database Migration Scripts
- Chapter Data Model
- Tag Data Model
- Entity Relationship Definitions
- Generic Data Model
- Security & Request Middleware
- Financial Data Model
- Temporal Data Model
- API Response Wrapper
- Home Page Smoke Tests
- Knowledge Graph Query
- Project Architecture Layers
- Python Worker Dependencies
- Entity Relations
- Entity Relations
- Entity Relations
- Entity Relations
- Entity Relations
- Date-based Data Model
- Entity Relations
- Entity Relations
- Entity Relations
- String-based Data Model
- Entity Relations
- Entity Relations
- Entity Relations
- Basic String Model
- Avatar Validation
- ActiveModel Behavior
- ActiveModel Behavior
- ActiveModel Behavior
- ActiveModel Behavior
- ActiveModel Behavior
- ActiveModel Behavior
- ActiveModel Behavior
- ActiveModel Behavior
- ActiveModel Behavior
- ActiveModel Behavior
- ActiveModel Behavior
- ActiveModel Behavior
- ActiveModel Behavior
- Brain Router Skill
- Memory Care Skill
- Memory Recall Skill
- Knowledge Graph Output
- Code Commenting Standards
- Documentation Link Integrity
- Admin Sorting Visuals
- Admin Pigeonhole Visuals
- Reader View Visuals
- Reader Mascot Illustration
- Library Catalog Visuals
- Library Bookshelf Visuals
- Ingestion Monitor Visuals
- Manuscript Inspection Visuals
- AI Quote Illustration
- Discovery Rocket Illustration
- Brand Identity Mark
- Design Reference Guide
- Brand Iconography
- Apple Touch Assets
- PWA Icon Small
- PWA Icon Large
- PWA Maskable Icon
- Admin Visual Snapshots
- Home Visual Snapshots
- Book Overview Snapshots
- Guest Profile Snapshots
- Reader Visual Snapshots
- Home UI Screenshots
- Carousel Audit Screenshots
- Book Overview Screenshots
- Quote Finder Screenshots
- Reader UI Screenshots
- Reader Drawer UI
- Admin Upload Screenshots
- Reader View Logic
- Auth UI Screenshots
- Reader UI Styling
- Monorepo Directory Map
- Project Backlog
- Technical Debt

## God Nodes (most connected - your core abstractions)
1. `AppError` - 119 edges
2. `AppState` - 75 edges
3. `Unreleased` - 39 edges
4. `Project Baca — Decisions` - 38 edges
5. `assert_no_page_errors()` - 34 edges
6. `use_lang()` - 33 edges
7. `Release gate` - 32 edges
8. `2026-10-08 Task 25 Shipped: Paper Reader, Navigation, Hardening` - 30 edges
9. `TestHarness` - 29 edges
10. `Standing decision: Edge` - 29 edges

## Surprising Connections (you probably didn't know these)
- `Postgres for Everything (No Elasticsearch)` --semantically_similar_to--> `Postgres for Everything`  [INFERRED] [semantically similar]
  CLAUDE.md → knowledge/erd.md
- `1. Engineering Invariants & Data Philosophy` --references--> `books()`  [INFERRED]
  knowledge/erd.md → crates/web/tests/component/test_components.py
- `5. Migration History (`migrations/`)` --references--> `books()`  [INFERRED]
  knowledge/erd.md → crates/web/tests/component/test_components.py
- `B. Catalog & Taxonomy Cluster` --references--> `books()`  [INFERRED]
  knowledge/erd.md → crates/web/tests/component/test_components.py
- `Dev Services via make db-up (Postgres 5433, Redis 6380, MinIO 9005/9006, Mailpit 1025/8025)` --semantically_similar_to--> `docker-compose.prod.yml (production compose)`  [INFERRED] [semantically similar]
  GUIDE.md → docker-compose.prod.yml

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **EPUB Ingestion Worker Pipeline** — architecture_download_and_zip_guard, architecture_parse_opf_and_spine_with_webp_cover, architecture_xhtml_sanitizer, architecture_chunking_with_cfi_ranges, architecture_batch_embedding_768_dimensions, architecture_atomic_cards_and_spoiler_free_recap_generation, architecture_insert_publish_and_xack [EXTRACTED 1.00]
- **Fase 1 reliability tickets** — knowledge_tickets_readme_fase_1_keandalan, knowledge_tickets_tick_001, knowledge_tickets_tick_002 [EXTRACTED 1.00]
- **Infrastructure Services** — redis_7, minio [EXTRACTED 1.00]
- **Launch gate prerequisites** — knowledge_tickets_tick_014, knowledge_tickets_tick_001, knowledge_tickets_tick_003, knowledge_tickets_tick_005, knowledge_tickets_tick_007 [EXTRACTED 1.00]
- **Reader Page Turn Mechanism** — architecture_reader_column_layout, architecture_spine_hinged_strip_page_flip, architecture_one_column_clone_per_strip, architecture_damped_spring_page_turn, architecture_transform_and_opacity_only_frames, architecture_slo_reader_page_turn [EXTRACTED 1.00]
- **Token and session hardening lineage** — knowledge_decisions_adr_15_authentication_baseline, knowledge_decisions_owasp_asvs_error_sanitizing_lockouts_and_token_revocation, knowledge_decisions_adr_21_adr_22_fail_closed_security_beyond_auth, knowledge_decisions_adr_27_fail_closed_on_auth_and_ai_cost_paths, knowledge_decisions_adr_29_refresh_grace_window_one_limiter_engine_one_api_surface, knowledge_decisions_adr_31_per_device_sessions_lifecycle_curator_desk_strict_script_csp, knowledge_decisions_standing_decision_tokens_and_sessions [EXTRACTED 1.00]
- **Token and Session Security Lifecycle** — architecture_hs256_access_token, architecture_refresh_token_rotation, architecture_refresh_token_family_revocation, architecture_session_record, architecture_delete_me_sessions_id [EXTRACTED 1.00]
- **Auth, Redis and session hardening chain** — knowledge_tickets_tick_001, knowledge_tickets_tick_003, knowledge_tickets_tick_004, knowledge_tickets_tick_006 [INFERRED 0.75]
- **Curator ingestion flow** — knowledge_product_curate_flow, changelog_curator_desk, knowledge_product_fr_adm_01, knowledge_product_fr_adm_02, knowledge_product_fr_adm_03, knowledge_product_fr_adm_04 [INFERRED 0.85]
- **Guest-to-account reconciliation flow** — knowledge_product_guest_role, knowledge_product_guest_to_account_reconciliation, knowledge_product_auth_sheet, knowledge_product_fr_usr_01, knowledge_product_fr_usr_03, knowledge_product_us_05, knowledge_product_sign_in_flow [INFERRED 0.85]
- **Shared Engineering Invariants Across Agent Guides** — agents_zero_panics_in_rust, knowledge_erd_postgres_for_everything, agents_pragmatic_message_broker, agents_paired_sql_migrations, agents_centralized_makefile_automation, agents_edge_transport_termination, agents_zero_emoji_policy, claude_zero_panics_invariant, claude_postgres_for_everything, claude_redis_streams_for_async_tasks, claude_edge_transport_termination, gemini_rust_zero_panics, gemini_pragmatic_message_broker, gemini_seaorm_paired_migrations, architecture_invariants [INFERRED 0.95]

## Communities (246 total, 136 thin omitted)

### Community 0 - "Knowledge Management Rules"
Cohesion: 0.08
Nodes (63): .agents/rules/, Graphify Query, GUIDE.md, knowledge/manual-qa.md, knowledge/product.md, knowledge/tickets/README.md, Shared Agent Rules ~/.agents/rules, Open Security Work in Tickets (+55 more)

### Community 1 - "Ingestion Queue Processing"
Cohesion: 0.07
Nodes (28): dlq_entry(), get_job_status(), INGESTION_DLQ_STREAM, INGESTION_GROUP, INGESTION_STREAM, job_key(), JOB_KEY_PREFIX, JOB_TTL_SECS (+20 more)

### Community 2 - "UI Animation Components"
Cohesion: 0.07
Nodes (28): angle_for_travel(), BEND_MAX, clear_children(), column_template(), Dir, DRAG_SPRING, DragStart, FLICK_VELOCITY (+20 more)

### Community 3 - "Project Release Notes"
Cohesion: 0.06
Nodes (39): Center Tap Dims Bars, Covers Bucket, IndexedDB project_baca_db v3, Reader, Reader Finale Card, Reader Gestures and Keys, Reader Heartbeat Once per Minute, SLO Reader Page Turn (+31 more)

### Community 4 - "AI Embedding Providers"
Cohesion: 0.08
Nodes (22): API_KEY_HEADER, build_embedding_provider(), EmbeddingProvider, FastEmbedProvider, GeminiBatchEmbedRequest, GeminiBatchEmbedResponse, GeminiContent, GeminiEmbedding (+14 more)

### Community 5 - "Data Transfer Objects"
Cohesion: 0.11
Nodes (44): ActiveProgressDto, AdminBookPatchRequest, AdminBookRowDto, ApiResponse, AtomicCardsDto, BadgeDto, BookCatalogQuery, BookDetailDto (+36 more)

### Community 6 - "Quote Repository Operations"
Cohesion: 0.11
Nodes (25): ChunkSearchRow, find_saved_quote_duplicate(), get_chapter_recap(), get_saved_quote_by_id(), get_tldr_cache(), insert_quote(), list_saved_quotes(), QuoteSaveOutcome (+17 more)

### Community 7 - "Reading Metrics Logic"
Cohesion: 0.06
Nodes (32): reading_minutes(), roman(), WORDS_PER_MINUTE, anchor_index(), columns_for(), css_px(), Paddings, page_of() (+24 more)

### Community 8 - "API Boundary Tests"
Cohesion: 0.06
Nodes (4): get_cover(), test_cover_missing_and_invalid_names_are_404(), test_cover_roundtrip_through_storage(), test_signup_fails_closed_when_smtp_unreachable()

### Community 9 - "Domain Status Logic"
Cohesion: 0.06
Nodes (23): advance_streak(), BASE_HEARTBEAT_XP, BookStatus, COMPLETION_MAX, COMPLETION_MIN, DAILY_MILESTONE_BONUS_XP, DAILY_THRESHOLD_SECONDS, day() (+15 more)

### Community 10 - "Database Schema Migrations"
Cohesion: 0.07
Nodes (43): idx_users_admin_role, idx_users_email, users, book_tags, books, idx_book_tags_reverse, idx_books_catalog_filter, idx_books_published_author_trgm (+35 more)

### Community 11 - "System Architecture Layers"
Cohesion: 0.11
Nodes (37): Badge Eligibility, Crates and Layering, crates/domain, crates/infra, crates/server, crates/shared, crates/web, docker-compose.prod.yml (+29 more)

### Community 12 - "Admin Web Pages"
Cohesion: 0.09
Nodes (27): JobStatusDto, SiteHeader(), use_session(), ShellLayout(), use_toasts(), use_lang(), AdminPage(), CuratorDesk() (+19 more)

### Community 13 - "Frontend API Utilities"
Cohesion: 0.07
Nodes (31): ReadingProgressUpdateDto, ADMIN_PAGE_SIZE, api::signup, as_html_element(), asset_url(), change_password(), clear_token(), drain_pending() (+23 more)

### Community 14 - "JWT Token Management"
Cohesion: 0.17
Nodes (36): blacklist_access_token(), Claims, generate_access_token(), generate_access_token_issued_at(), generate_refresh_token(), invalidate_user_tokens(), is_token_blacklisted(), is_user_token_revoked() (+28 more)

### Community 15 - "Authentication Routes"
Cohesion: 0.19
Nodes (23): client_ip_from_headers(), auth_routes(), change_password(), delete_me(), get_me(), INVALID_CREDENTIALS, issue_token_pair(), list_my_sessions() (+15 more)

### Community 16 - "Shell UI Components"
Cohesion: 0.06
Nodes (9): current(), HOW_IT_WORKS_ID, ACCOUNT_ID, current(), TabBar(), current_url(), resume_chapter(), Space key, pinch zoom and chapter hash misbehave (+1 more)

### Community 17 - "System Architecture Overview"
Cohesion: 0.10
Nodes (36): Argon2id Password Hashing (19 MiB, 2 iterations), Auth Endpoints (/auth/signup, verify-otp, login, refresh, logout), Axum Middleware Layers (tracing, CORS, x-request-id, security headers), Catalog Endpoints (/books, /books/search, chapters, offline-bundle), Caddy Edge (TLS, HTTP/3, SPA from dist/, /api proxy), GET /covers/{file} (immutable cache), Cover Storage (covers/<book uuid>.<ext> in covers bucket), Data Exposure Rules (drafts hidden, values not echoed, secrets masked) (+28 more)

### Community 18 - "Design System Assets"
Cohesion: 0.10
Nodes (31): 404 Page, admin-sorting-pigeonholes, cozy-reader-armchair-owl, Home Two-Row Catalog Preview, Illustration Assets, library-bookshelf-ladder, manuscript-inspection-clothesline, retro-rocket-discovery (+23 more)

### Community 19 - "Axum Middleware Errors"
Cohesion: 0.09
Nodes (6): HttpError, AuthUser, get_active_progress(), merge_guest_progress(), progress_routes(), update_progress()

### Community 20 - "Server Initialization"
Cohesion: 0.07
Nodes (7): init_db_pool(), init_redis_client(), redacted_endpoint(), main(), shutdown_signal(), BookCatalogPort, test_reliability_mock_repository_fault_injection()

### Community 21 - "Admin Integration Tests"
Cohesion: 0.15
Nodes (27): admin_get(), admin_patch(), AdminContext, BOUNDARY, multipart_body(), seed_funnel(), seed_status_book(), seed_users() (+19 more)

### Community 22 - "Engineering Pillars"
Cohesion: 0.08
Nodes (18): Audit Trail Protocol (doc routing by change type), OKF v0.2 Knowledge Catalog (knowledge/ as SSOT), Rotaria Two-Surface Design (Espresso shell, Paper surface), [0.2.0] 2026-09-25, 768-dim Dual-Mode Embedding Engine, Edge Transport Architecture (Caddy HTTP/3), Open Knowledge Format (OKF v0.2), PostgreSQL FTS and pg_trgm Search Strategy (+10 more)

### Community 23 - "Admin Route Constants"
Cohesion: 0.18
Nodes (28): active_progress(), admin_books(), admin_dlq(), admin_dropoff(), admin_job(), admin_replay(), admin_set_status(), all_badges() (+20 more)

### Community 24 - "Internationalization Module"
Cohesion: 0.12
Nodes (9): apply_lang(), ENTRIES, Lang, LANG_KEY, lang_signal(), LangContext, provide_lang(), badge_description() (+1 more)

### Community 25 - "Design Audit Tasks"
Cohesion: 0.09
Nodes (27): Design canvas, adb reverse phone setup, Admin account setup, Appearance and language checks, Before you start, Device matrix, L1, L2 (+19 more)

### Community 26 - "PWA Web Infrastructure"
Cohesion: 0.08
Nodes (27): Design System (Espresso shell, Paper/Sepia/Espresso reading themes), Root Contexts (lang, Session, Toasts, ShellTheme), Service Worker (sw.js: shell and asset caches, /api never cached), boot.js bootstrap script (copied by Trunk), crates/web/index.html (Trunk entry page), SVG favicon /assets/rotaria-mark.svg, Google Fonts stylesheet (EB Garamond, Newsreader, Plus Jakarta Sans), style/main.css link (+19 more)

### Community 27 - "Home Page Components"
Cohesion: 0.10
Nodes (20): prefers_reduced_motion(), reveal(), scroll_to_hash(), scroll_to_location_hash(), catalog_columns(), CATALOG_PREVIEW_ROWS, HomePage(), LanguageFilter (+12 more)

### Community 28 - "Architecture Decision Records"
Cohesion: 0.13
Nodes (15): Ronde 2 finding TICK-001, Ronde 2 finding TICK-003, Project Baca — Decisions, OKF knowledge base, opencode.jsonc skill allowlist, Standing decision: API surface, Standing decision: Data access, Standing decision: Edge (+7 more)

### Community 29 - "Development Invariants"
Cohesion: 0.10
Nodes (16): make test-performance-release, Service Levels, SLO Catalog Search under 5 ms, SLO Login p95 under 150 ms and OTP under 100 ms, SLO REST API p95 under 50 ms, SLO Scoped Vector Search under 10 ms, 4-Tier Test Suite, Database Schema and Index Matrix (+8 more)

### Community 30 - "Rate Limiting Middleware"
Cohesion: 0.15
Nodes (17): AI_POLICY, ai_rate_limit_middleware(), AUTH_POLICY, enforce(), extract_client_ip(), FALLBACK_CLIENT_IP, forwarded_for_takes_the_first_parseable_hop(), public_policy() (+9 more)

### Community 31 - "Book Repository Operations"
Cohesion: 0.20
Nodes (18): ADMIN_ROW_SELECT, AdminBookRow, AdminBookRowDto, chapter_dropoff(), ensure_book_published(), get_book_by_id(), get_book_for_admin(), get_book_status() (+10 more)

### Community 32 - "Reader Theme Preferences"
Cohesion: 0.09
Nodes (13): FONT_SIZE_MAX, FONT_SIZE_MIN, LEADING_PRESETS, LINE_HEIGHT_MAX, LINE_HEIGHT_MIN, prefers_light(), READER_KEY, ReaderPrefs (+5 more)

### Community 33 - "Security Policy Decisions"
Cohesion: 0.11
Nodes (10): knowledge/decisions.md, Request Path, Test Layout, Audit Hardening (2026-10-07), Earlier Hardening (ADR-20 to ADR-27), Streak Endpoint and Public Read Cap, 2026-09-27 OpenCode Skill Allowlist and instructions Correction, 2026-09-27 Pre-M6 Debt Paydown: Deferred Security and p95 Benchmarks (+2 more)

### Community 34 - "UI Interaction Helpers"
Cohesion: 0.10
Nodes (19): eligible_badges(), active_element(), manage_sheet_focus(), crates/web/style/main.css, crates/web/tests/a11y/, crates/web/tests/e2e/, crates/web/tests/visual/, Ronde 2 finding TICK-010 (+11 more)

### Community 35 - "Database Constraint Tests"
Cohesion: 0.08
Nodes (4): DlqTest, main(), psql(), purge_seed_survivors()

### Community 36 - "Application State Management"
Cohesion: 0.12
Nodes (10): api_health_check(), ApiDoc, AppState, create_app(), health_check(), not_found_handler(), REQUEST_ID_HEADER, SecurityAddon (+2 more)

### Community 37 - "Test Harness Utilities"
Cohesion: 0.11
Nodes (3): CLIENT_IP_HEADER, Seeded, TestHarness

### Community 38 - "Client-Side Offline Storage"
Cohesion: 0.12
Nodes (16): clear(), db(), DB_NAME, DB_VERSION, delete(), get_all(), local_quote_key(), LocalQuoteRecord (+8 more)

### Community 39 - "Makefile Automation"
Cohesion: 0.11
Nodes (22): make check, make db-up migrate-up seed-dev, make dev-web, Makefile, Quick Commands, Deployment, make build, make prod-build prod-up prod-down prod-logs (+14 more)

### Community 40 - "AI Integration Tests"
Cohesion: 0.14
Nodes (20): one_hot(), seed_reader(), seed_test_context(), SeededAiContext, SeededReader, test_ai_rate_limiter_burst_enforcement(), test_atomic_cards_scoped_to_path_book(), test_atomic_insight_cards_cache_hit_and_miss() (+12 more)

### Community 41 - "E2E Accessibility Tests"
Cohesion: 0.17
Nodes (16): assert_no_page_errors(), goto(), test_aria_home_structure(), test_aria_nav_landmarks(), test_hero_plate_is_static(), test_home_catalog_shows_two_rows_then_everything(), test_i18n_toggle_and_persist(), test_overview_and_sheets() (+8 more)

### Community 42 - "Curation Bounded Context"
Cohesion: 0.14
Nodes (23): Admin Endpoints (/admin/books, upload, jobs, dlq, analytics), Curator Desk, Ronde 2 finding TICK-005, Bounded context: Curation, Dead-letter queue, EmbeddingProvider trait, Standing decision: Queue and ingestion, C1 (+15 more)

### Community 43 - "Ingestion Message Contract"
Cohesion: 0.13
Nodes (18): API Surface, Dead-Letter Stream stream:epub_ingestion:dlq, DELETE /me/sessions/{id}, API Error Codes, GET /admin/dlq, GET /me/sessions, Ingestion Message Contract, Job Hash job:{id} (+10 more)

### Community 44 - "Database Table Clusters"
Cohesion: 0.14
Nodes (21): Migration 01 init_extensions, Migration 04 create_chapters_and_chunks, Migration 05 create_tldr_cache, table book_chunks, table chapters, Chapters & Semantic Vectors Cluster, Project Baca ERD & PostgreSQL 17 Database Schema, Identity & Auth Cluster (+13 more)

### Community 45 - "AI Insight Endpoints"
Cohesion: 0.13
Nodes (24): Insight Endpoints (atomic-cards, recap), tldr_cache, Standing decision: AI cost, C3, C4, C5, Real book, AI, and quotes checks, AI cost metric (+16 more)

### Community 46 - "OTP Security Logic"
Cohesion: 0.11
Nodes (16): generate_and_store_otp(), generate_numeric_otp(), hash_otp(), MAX_OTP_ATTEMPTS, OTP_LOCK_SECONDS, OTP_TTL_SECONDS, OtpRecord, test_hash_otp_consistency() (+8 more)

### Community 47 - "S3 Storage Service"
Cohesion: 0.20
Nodes (4): StorageService, StoredObject, AppError, S3 client has only a connect timeout

### Community 48 - "Auth Lifecycle Tests"
Cohesion: 0.14
Nodes (13): login_req(), provision_verified_user(), refresh_req(), test_auth_concurrent_refresh_keeps_family_alive(), test_auth_email_aggregate_lockout_twenty_failures(), test_auth_pair_lockout_five_failures(), test_auth_password_change_kills_other_session_e2e(), test_auth_refresh_grace_window() (+5 more)

### Community 49 - "UI Icon Library"
Cohesion: 0.23
Nodes (22): arrow_down(), arrow_right(), back(), brand_mark(), cards(), check(), close(), download() (+14 more)

### Community 50 - "Agent Operational Rules"
Cohesion: 0.14
Nodes (16): Data Layer, Foreign-Key B-Tree Indexes, migrations, [0.2.1] 2026-09-26, Account Abuse Controls, Argon2id Thread Offloading, Database Index Optimization (Migration 07), Milestone 03 Catalog Backend and Reader API (+8 more)

### Community 51 - "Reader Feature Backlog"
Cohesion: 0.11
Nodes (23): load_chapter, paginate, persist, Ronde 2 finding TICK-002, p-N paragraph anchors, R7, R9, FR-RDR-03 (+15 more)

### Community 52 - "Pytest Shared Fixtures"
Cohesion: 0.14
Nodes (10): api_get(), book_with_chapters(), browser_context_args(), clean_page(), stable_book_id(), _drag(), _open_long_chapter(), test_drag_turns_like_paper() (+2 more)

### Community 53 - "Python Worker Scripts"
Cohesion: 0.15
Nodes (10): convert_cover_to_webp(), ensure_group(), main(), make_db(), make_minio(), make_redis(), _opf_path(), process_message() (+2 more)

### Community 54 - "Domain Context Definitions"
Cohesion: 0.13
Nodes (18): Bounded context: Engagement, Standing decision: Crates and layering, Covered by automation, make test-admin, make test-auth and test-security, make test-catalog, make test-domain, make test-live (+10 more)

### Community 55 - "Database Entity Models"
Cohesion: 0.13
Nodes (10): Relation, Relation, Model, Relation, Relation, Relation, Relation, Relation (+2 more)

### Community 56 - "Offline PWA Requirements"
Cohesion: 0.14
Nodes (21): Ronde 2 finding TICK-004, Ronde 2 finding TICK-009, A7, Install and update checks, O1, O2, O4, O5 (+13 more)

### Community 57 - "Edge Security Architecture"
Cohesion: 0.16
Nodes (13): Defense-in-Depth Security Architecture, Edge Transport Architecture (ADR-17), make purge-test-debris, Milestone 02 Authentication and Guest Reconciliation, OTP Lock Answers 429, Strict Script CSP, Strict Script CSP Change, 2026-09-26 Edge Transport Architecture: Caddy HTTP/3 Reverse Proxy (+5 more)

### Community 58 - "Test Environment Configuration"
Cohesion: 0.14
Nodes (19): Test Layout (tests/it single binary, separate perf and load), playwright, pytest>=8, pytest-xdist>=3, AppState, AppState as central Axum dependency-injection bridge, get_book, Graphify query: trace data path and dependencies across architectural boundaries (+11 more)

### Community 59 - "Email Notification Service"
Cohesion: 0.16
Nodes (10): EmailConfig, build_transport(), mailpit_accepts_cleartext_delivery_when_running(), mask_email(), otp_message(), otp_message_addresses_sender_and_recipient(), send_otp_email(), smtp_error() (+2 more)

### Community 60 - "Application Configuration"
Cohesion: 0.15
Nodes (6): AppConfig, AuthConfig, DatabaseConfig, RedisConfig, ServerConfig, StorageConfig

### Community 61 - "User Session E2E Tests"
Cohesion: 0.17
Nodes (11): _count_local_quotes(), _mailpit_otp(), _post(), _put_local_quote(), test_authed_autosave_shelf_cleanup(), test_guest_quote_merges_on_register(), call(), test_envelope_and_health() (+3 more)

### Community 62 - "Gamification Database Indexes"
Cohesion: 0.20
Nodes (20): Migration 02 create_users_and_roles, Migration 06 create_progress_and_gamification, Migration 07 optimize_indexes_and_foreign_keys, table badges, index idx_books_published_year_id, index idx_reading_activity_user_date, index idx_saved_quotes_user_book, index idx_tldr_cache_chapter_id (+12 more)

### Community 63 - "HTML Sanitization Utilities"
Cohesion: 0.12
Nodes (5): SanitizeTest, extract_text(), __init__(), sanitize_html(), _Sanitizer

### Community 64 - "Account Lifecycle Requirements"
Cohesion: 0.20
Nodes (16): A1, A3, A4, A5, A6, Accounts and sessions checks, H1, H2 (+8 more)

### Community 65 - "User Reconciliation Requirements"
Cohesion: 0.15
Nodes (16): A2, knowledge/product.md (PRD), Accounts and reconciliation requirements, Admin curator role, Auth sheet, Book clubs and social leaderboards, Curators segment, FR-USR-01 (+8 more)

### Community 66 - "Book Cover Service"
Cohesion: 0.14
Nodes (5): COVER_CACHE_CONTROL, cover_content_type(), covers_routes(), get_cover(), is_cover_file_name()

### Community 67 - "Text Chunking Utility"
Cohesion: 0.13
Nodes (7): BATCH_SIZE, chunk_text(), CHUNK_WORDS, main(), MIN_CHARS, strip_html(), Model

### Community 68 - "AI Insight Components"
Cohesion: 0.24
Nodes (9): AtomicCards(), CatchupRecap(), InsightList(), InsightText(), QuoteFinder(), share_quote(), Sheet(), string_field() (+1 more)

### Community 69 - "Reader Bounded Context"
Cohesion: 0.20
Nodes (17): Bounded context: Reading, Standing decision: Offline and deploy, R1, R2, R3, R4, R5, R6 (+9 more)

### Community 70 - "API Client Wrapper"
Cohesion: 0.21
Nodes (10): authed(), authed_send(), login(), parse_envelope(), patch(), post_once(), put(), revoke_all() (+2 more)

### Community 71 - "Audit Review Rounds"
Cohesion: 0.19
Nodes (13): Audit 2026-10-08, Ronde 1, Ronde 1 Backend sedang, Ronde 1 Backend tinggi, Ronde 1 Frontend sedang, Ronde 1 Frontend tinggi, Ronde 2, Ronde 2 finding TICK-006 (+5 more)

### Community 72 - "Discovery Bounded Context"
Cohesion: 0.17
Nodes (15): Bounded context: Discovery, Standing decision: Indexes, Standing decision: Storage and search, O3, Catalog and discovery requirements, Catalog search latency metric, Discover flow, FR-CAT-01 (+7 more)

### Community 73 - "Tauri and Service Worker"
Cohesion: 0.21
Nodes (14): apps/tauri/, Caddyfile, crates/web/index.html, SHELL_URL, TICK-009: Service Worker dan Font Mandiri, Font CSS never cached for offline, Render-blocking Google Fonts stylesheet, Shell cache grows per navigated URL (+6 more)

### Community 74 - "Docker Deployment Config"
Cohesion: 0.19
Nodes (16): Caddyfile.prod, is_production, docker-compose.yml, docker-compose.prod.yml, Dockerfile.server, Dockerfile.worker, .env.production.example, Fase 5: Peluncuran (+8 more)

### Community 75 - "Project Reliability Tasks"
Cohesion: 0.17
Nodes (16): crates/infra/src/security/*.rs, AppState.redis_conn, knowledge/tasks/ (old task board), Fase 1: Keandalan, Fase 2: Keamanan dan Sesi, Fase 4: Fitur, Old Task Number Mapping (Pemetaan nomor lama), Ticket Board (Tiket Kerja) (+8 more)

### Community 76 - "Public Book Routes"
Cohesion: 0.36
Nodes (6): books_routes(), get_book(), get_chapter(), get_offline_bundle(), list_books(), search_books()

### Community 77 - "Toast Notification System"
Cohesion: 0.20
Nodes (5): ERROR_MS, INFO_MS, Toast, Toasts, ToastStack()

### Community 78 - "Visual Regression Helpers"
Cohesion: 0.17
Nodes (5): check(), externalize(), replace(), _is_inline(), main()

### Community 79 - "User Repository Operations"
Cohesion: 0.37
Nodes (8): activate_user_by_email(), create_inactive_user(), delete_user_by_id(), find_user_by_email(), find_user_by_id(), update_inactive_credentials(), update_user_password(), update_user_profile()

### Community 80 - "Gamification API Routes"
Cohesion: 0.34
Nodes (6): gamification_routes(), get_my_streak(), HEARTBEAT_MIN_INTERVAL_SECS, list_badges(), list_user_badges(), record_heartbeat()

### Community 81 - "AI Insight Routes"
Cohesion: 0.29
Nodes (4): get_atomic_cards(), get_chapter_recap(), insights_routes(), resolve_chapter()

### Community 82 - "OWASP Security Tests"
Cohesion: 0.23
Nodes (11): test_owasp_login_brute_force_lockout(), test_owasp_otp_cooldown_and_email_bombing_prevention(), test_owasp_rate_limiting_headers_and_ip_extraction(), test_owasp_revoke_all_sessions(), test_owasp_security_headers_and_error_handling(), test_owasp_structured_validation_error_details(), test_owasp_timing_attack_mitigation_on_login(), test_owasp_token_revocation_on_logout() (+3 more)

### Community 83 - "Axe Accessibility Scans"
Cohesion: 0.28
Nodes (9): _assert_clean(), _scan(), test_axe_admin(), test_axe_home(), test_axe_overview(), test_axe_paper_surface(), test_axe_profile_guest(), test_axe_quote_modal() (+1 more)

### Community 84 - "Worker Resilience Fixes"
Cohesion: 0.18
Nodes (12): TICK-005: Ketahanan Worker Ingest, EPUB metadata not truncated to column width, Final publish ignores archived status, Gemini API key sent in URL query, Read-phase TimeoutError not retried, XAUTOCLAIM idle time not refreshed during long jobs, embed_batch(), _gemini_post() (+4 more)

### Community 86 - "Password Hashing Logic"
Cohesion: 0.31
Nodes (10): DUMMY_ARGON2_HASH, hash_password(), hash_password_async(), test_async_hash_and_verify(), test_dummy_argon2_hash_validity(), test_hash_and_verify_password_success(), test_verify_password_invalid_hash(), test_verify_password_wrong_password() (+2 more)

### Community 87 - "Environment Config Validation"
Cohesion: 0.19
Nodes (6): SmtpSecurity, test_app_config_from_env_clean_execution(), test_config_accessors(), test_default_app_config(), All guests share one bucket without proxy trust, Plaintext SMTP accepted in production

### Community 89 - "Progress Repository Operations"
Cohesion: 0.35
Nodes (7): check_and_award_badges(), get_active_progress(), get_streak(), record_heartbeat(), seconds_read_on(), update_progress(), ReadingHeartbeatResponse

### Community 90 - "Catalog Integration Tests"
Cohesion: 0.26
Nodes (10): seed_book_with_chapter(), seed_test_catalog(), SeededCatalog, test_book_overview_chapter_and_offline_bundle(), test_catalog_listing_and_filtering(), test_catalog_typo_tolerant_fts_search(), test_gamification_heartbeat_and_badges(), test_heartbeat_credit_clamped_and_gate_per_user() (+2 more)

### Community 91 - "AI Memory Skills"
Cohesion: 0.19
Nodes (8): Brain content, Provenance, Safety, Skills, brain-router skill, memory-care skill, memory-recall skill, recall tool (read-only, bounded result budget)

### Community 92 - "EPUB Parsing Logic"
Cohesion: 0.20
Nodes (6): make_epub(), ParseTest, parse_epub(), ParsedChapter, ParsedEpub, _safe_member()

### Community 93 - "AI Retry Logic Tests"
Cohesion: 0.16
Nodes (3): RetryAndRecapTests, backoff_seconds(), build_recap_input()

### Community 95 - "Responsive Design Tests"
Cohesion: 0.26
Nodes (9): assets/illustrations/, _page_at(), test_home_and_book_fit_viewport(), test_reader_paginates_long_chapter(), _visible(), Fase 3: Kualitas dan Bobot, TICK-011: Responsif Lanjutan, Illustration sources too low resolution for dense screens (+1 more)

### Community 96 - "UI Component Storybook"
Cohesion: 0.31
Nodes (7): _story(), test_atomic_cards_404_tolerant(), test_auth_sheet_modes(), test_curator_desk_tabs_rows_replay_and_funnel(), test_quote_finder_empty_state(), test_quote_finder_error_branch(), test_site_header_toggle_no_reload()

### Community 97 - "Data Dictionary Documentation"
Cohesion: 0.19
Nodes (13): books(), 1. Engineering Invariants & Data Philosophy, 2. Entity Relationship Diagram, 3. Data Dictionary & Table Specifications, 4. Index Matrix & Query Patterns, 5. Migration History (`migrations/`), A. Identity & Auth Cluster, B. Catalog & Taxonomy Cluster (+5 more)

### Community 98 - "Ingestion Worker Tests"
Cohesion: 0.19
Nodes (4): ArchiveLimitsTest, ChunkTest, check_archive_limits(), chunk_words()

### Community 100 - "Database Schema & Search"
Cohesion: 0.30
Nodes (12): Migration 03 create_books_and_tags, table book_tags, table books, Catalog & Taxonomy Cluster, Discovery Search Query (<5ms), index idx_book_tags_reverse, index idx_books_catalog_filter, index idx_books_published_author_trgm (+4 more)

### Community 101 - "Authentication UI Components"
Cohesion: 0.22
Nodes (4): AuthSheet(), friendly_error(), merge_local_quotes(), Mode

### Community 102 - "Audit Reports & Screenshots"
Cohesion: 0.18
Nodes (10): Audit Screenshot 05: Atomic Insights Modal, Audit Screenshot 08: Guest Profile, Audit Screenshot 10: Save Offline on Book Overview, Quote Finder Empty State Screenshot, Saved Offline Book Overview Screenshot, Guest Profile CTA Screenshot, Audit Web Menyeluruh — 2026-09-27 (pra-M7), Kontrak yang legitimate (bukan bug) (+2 more)

### Community 104 - "WASM Build Configuration"
Cohesion: 0.33
Nodes (9): Cargo.toml, crates/web/Trunk.toml, Ronde 2 finding TICK-008, Ronde 2 test results, Smaller WebAssembly bundle with size budget, TICK-008: Bundel WASM Ramping, No WASM release profile and wasm-opt skipped, WASM release size budget (+1 more)

### Community 105 - "Book Card UI"
Cohesion: 0.31
Nodes (5): BookCard(), Cover(), cover_tint(), COVER_TINTS, SkeletonCard()

### Community 106 - "Docker Production Infrastructure"
Cohesion: 0.53
Nodes (9): docker-compose.prod.yml (production compose), edge service (Dockerfile.edge, Caddy on 80 and 443 TCP and UDP), minio service (cgr.dev/chainguard/minio, internal only), named volumes (pgdata_prod, redisdata_prod, miniodata_prod, caddy_data, caddy_config), postgres service (pgvector/pgvector:pg17), redis service (redis:7-alpine), server healthcheck (wget /health, status healthy), server service (Dockerfile.server, APP_ENV production) (+1 more)

### Community 107 - "Badge Repository Logic"
Cohesion: 0.50
Nodes (3): list_badges(), list_user_badges(), seed_default_badges_if_empty()

### Community 108 - "Navigation Integration Tests"
Cohesion: 0.43
Nodes (5): _section_top(), test_cover_art_never_shows_a_broken_image(), test_header_marks_catalog_and_shelf(), test_how_it_works_scrolls_from_the_shelf(), test_tab_bar_marks_search_and_profile()

### Community 111 - "Progress UI Components"
Cohesion: 0.38
Nodes (3): ProgressBar(), ProgressRing(), RING_RADIUS

### Community 112 - "Reader Flow Tests"
Cohesion: 0.52
Nodes (4): _chapter_no(), test_end_of_book_notice_is_visible(), test_offline_save_read_badge(), test_reader_body_and_type_sheet()

### Community 113 - "Database Migration Scripts"
Cohesion: 0.76
Nodes (6): init_table(), migrate_down(), migrate_status(), migrate_up(), psql_cmd(), migrate.sh script

### Community 123 - "Knowledge Graph Query"
Cohesion: 0.40
Nodes (4): Answer, Outcome, Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya., Source Nodes

### Community 124 - "Project Architecture Layers"
Cohesion: 0.70
Nodes (5): domain, infra, server, shared, web

### Community 125 - "Python Worker Dependencies"
Cohesion: 0.40
Nodes (4): minio==7.2.20, pg8000==1.31.5, pillow==12.3.0, redis==8.1.0

## Ambiguous Edges - Review These
- `2026-09-28 Task 09 Absorbed planning/` → `Defense-in-Depth Security Architecture`  [AMBIGUOUS]
  CHANGELOG.md · relation: conceptually_related_to
- `ADR-29 Rotaria two-surface redesign` → `ADR-32 Open-book reader`  [AMBIGUOUS]
  knowledge/decisions.md · relation: conceptually_related_to
- `Release 0.2.0 (2026-09-25)` → `Version stays 0.1.0`  [AMBIGUOUS]
  AGENTS.md · relation: conceptually_related_to
- `Redis Streams as Single Queue` → `SMTP via lettre`  [AMBIGUOUS]
  CHANGELOG.md · relation: conceptually_related_to
- `Release 0.2.1 (2026-09-26)` → `Version stays 0.1.0`  [AMBIGUOUS]
  AGENTS.md · relation: conceptually_related_to
- `ADR-27 Separate production deploy and runtime-cache PWA` → `ADR-32 Open-book reader`  [AMBIGUOUS]
  knowledge/decisions.md · relation: conceptually_related_to
- `ADR-27 Separate production deploy and runtime-cache PWA` → `ADR-32 Trusted address header and API-served covers`  [AMBIGUOUS]
  knowledge/decisions.md · relation: conceptually_related_to
- `ADR-32 Open-book reader` → `Standing decision: Offline and deploy`  [AMBIGUOUS]
  knowledge/decisions.md · relation: rationale_for
- `Standing decision: Offline and deploy` → `ADR-32 Trusted address header and API-served covers`  [AMBIGUOUS]
  knowledge/decisions.md · relation: rationale_for

## Knowledge Gaps
- **247 isolated node(s):** `Mode`, `Relation`, `Relation`, `Relation`, `Relation` (+242 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 804 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **136 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `2026-09-28 Task 09 Absorbed planning/` and `Defense-in-Depth Security Architecture`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `ADR-29 Rotaria two-surface redesign` and `ADR-32 Open-book reader`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `Release 0.2.0 (2026-09-25)` and `Version stays 0.1.0`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `Redis Streams as Single Queue` and `SMTP via lettre`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `Release 0.2.1 (2026-09-26)` and `Version stays 0.1.0`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `ADR-27 Separate production deploy and runtime-cache PWA` and `ADR-32 Open-book reader`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `ADR-27 Separate production deploy and runtime-cache PWA` and `ADR-32 Trusted address header and API-served covers`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._