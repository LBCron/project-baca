# Graph Report - project-baca  (2026-11-03)

## Corpus Check
- 205 files · ~177,166 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 50 file(s) not represented in the graph (top: .xml 11, (none) 10, .woff2 10)

## Summary
- 3335 nodes · 7146 edges · 253 communities (138 shown, 115 thin omitted)
- Extraction: 91% EXTRACTED · 9% INFERRED · 0% AMBIGUOUS · INFERRED: 638 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ff7a995b`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- knowledge/log.md
- routes/admin.rs
- UI Animation Components
- Unreleased
- embedding.rs
- Data Transfer Objects
- quote_repository.rs
- Reading Metrics Logic
- Domain Status Logic
- Database Schema Migrations
- System Architecture Layers
- use_lang
- refresh
- JWT Token Management
- Authentication Routes
- prelude
- System Architecture Overview
- Web App
- routes/progress.rs
- server/src/main.rs
- Admin Integration Tests
- Engineering Pillars
- api/mod.rs
- Internationalization Module
- minio
- crates/web/index.html (Trunk entry page)
- Home Page Components
- Records
- Development Invariants
- Rate Limiting Middleware
- Book Repository Operations
- Reader Theme Preferences
- 2026-10-07 Audit Hardening + Rotaria Redesign
- manage_sheet_focus
- database_test.rs
- Application State Management
- Test Harness Utilities
- Client-Side Offline Storage
- Release 0.2.0 (2026-09-25)
- AI Integration Tests
- E2E Accessibility Tests
- with
- Worker Pipeline per Message
- Project Baca ERD & PostgreSQL 17 Database Schema
- environment
- otp.rs
- AppError
- Auth Lifecycle Tests
- UI Icon Library
- CHANGELOG.md
- tauri.conf.json
- test_reader_flip.py
- Python Worker Scripts
- quotes.rs
- deserialize
- queue.rs
- Security Model
- GUIDE.md: Developer Guide
- Email Notification Service
- AppConfig
- test_sessions.py
- Index Matrix (17 query-critical indexes)
- HTML Sanitization Utilities
- test_worker_live.py
- Product
- Book Cover Service
- backfill_chunks.rs
- AI Insight Components
- reliability_test.rs
- API Client Wrapper
- BuildTask.kt
- server
- TICK-013: Shell Tauri untuk Android, iOS, dan Desktop
- services
- Ticket Board (Tiket Kerja)
- HttpError
- Toast Notification System
- Visual Regression Helpers
- unit_tests.rs
- AppState
- AI Insight Routes
- OWASP Security Tests
- Axe Accessibility Scans
- Settings
- conftest.py
- Password Hashing Logic
- SmtpSecurity
- Progress Repository Operations
- Catalog Integration Tests
- AI Memory Skills
- EPUB Parsing Logic
- _gemini_post
- NOTICE.md: Copyright and Legal Licenses
- Responsive Design Tests
- UI Component Storybook
- Data Dictionary Documentation
- test_worker.py
- User Session State
- Database Schema & Search
- AuthSheet
- Audit Reports & Screenshots
- Entity
- 3. Checklist
- components/auth.rs
- test_offline_shell.py
- with
- Navigation Integration Tests
- release_notes.py
- users.rs
- Relation
- test_reader_flow.py
- Database Migration Scripts
- Chapter Data Model
- MainActivity.kt
- Entity Relationship Definitions
- tldr_cache.rs
- Release 0.1.0 (2026-09-25)
- GeminiEmbeddingProvider
- TcpRelay
- images
- table users
- Knowledge Graph Query
- Project Architecture Layers
- Python Worker Dependencies
- Entity Relations
- Entity Relations
- Entity Relations
- Entity Relations
- Entity Relations
- reading_activity_logs.rs
- Entity Relations
- Entity Relations
- Entity Relations
- user_badges.rs
- Entity Relations
- Entity Relations
- Entity Relations
- tauri/src/lib.rs
- keep_claimed
- EmbeddingProvider
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
- test_api_contract.py
- steps
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
- steps
- publish
- profile.rs
- table book_chunks
- purge_test_debris.py
- knowledge/index.md
- book_chunks.rs
- minio
- MockEmbeddingProvider
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
- covers_test.rs
- edge
- default.json
- release.yml
- notes
- app/build.gradle.kts
- pg_backup.sh
- rotaria-app

## God Nodes (most connected - your core abstractions)
1. `AppError` - 143 edges
2. `AppState` - 86 edges
3. `HttpError` - 62 edges
4. `use_lang()` - 48 edges
5. `assert_no_page_errors()` - 40 edges
6. `Unreleased` - 39 edges
7. `AuthUser` - 37 edges
8. `ReaderPage()` - 37 edges
9. `Records` - 37 edges
10. `goto()` - 34 edges

## Surprising Connections (you probably didn't know these)
- `Postgres for Everything (No Elasticsearch)` --semantically_similar_to--> `Postgres for Everything`  [INFERRED] [semantically similar]
  CLAUDE.md → knowledge/erd.md
- `1. Engineering Invariants & Data Philosophy` --references--> `books()`  [INFERRED]
  knowledge/erd.md → crates/web/tests/component/test_components.py
- `5. Migration History (`migrations/`)` --references--> `books()`  [INFERRED]
  knowledge/erd.md → crates/web/tests/component/test_components.py
- `B. Catalog & Taxonomy Cluster` --references--> `books()`  [INFERRED]
  knowledge/erd.md → crates/web/tests/component/test_components.py
- `postgres service (pgvector/pgvector:pg17)` --implements--> `Postgres for Everything`  [INFERRED]
  docker-compose.prod.yml → knowledge/erd.md

## Import Cycles
- 1-file cycle: `crates/infra/src/pool.rs -> crates/infra/src/pool.rs`
- 1-file cycle: `crates/infra/src/repositories/book_repository.rs -> crates/infra/src/repositories/book_repository.rs`

## Hyperedges (group relationships)
- **EPUB Ingestion Worker Pipeline** — architecture_download_and_zip_guard, architecture_parse_opf_and_spine_with_webp_cover, architecture_xhtml_sanitizer, architecture_chunking_with_cfi_ranges, architecture_batch_embedding_768_dimensions, architecture_atomic_cards_and_spoiler_free_recap_generation, architecture_insert_publish_and_xack [EXTRACTED 1.00]
- **Infrastructure Services** — redis_7, minio [EXTRACTED 1.00]
- **Reader Page Turn Mechanism** — architecture_reader_column_layout, architecture_spine_hinged_strip_page_flip, architecture_one_column_clone_per_strip, architecture_damped_spring_page_turn, architecture_transform_and_opacity_only_frames, architecture_slo_reader_page_turn [EXTRACTED 1.00]
- **Token and Session Security Lifecycle** — architecture_hs256_access_token, architecture_refresh_token_rotation, architecture_refresh_token_family_revocation, architecture_session_record, architecture_delete_me_sessions_id [EXTRACTED 1.00]
- **Shared Engineering Invariants Across Agent Guides** — agents_zero_panics_in_rust, knowledge_erd_postgres_for_everything, agents_pragmatic_message_broker, agents_paired_sql_migrations, agents_centralized_makefile_automation, agents_edge_transport_termination, agents_zero_emoji_policy, claude_zero_panics_invariant, claude_postgres_for_everything, claude_redis_streams_for_async_tasks, claude_edge_transport_termination, gemini_rust_zero_panics, gemini_pragmatic_message_broker, gemini_seaorm_paired_migrations, architecture_invariants [INFERRED 0.95]

## Communities (253 total, 115 thin omitted)

### Community 0 - "knowledge/log.md"
Cohesion: 0.10
Nodes (38): Playwright Test Pyramid, Reading order (orientation, product, contracts, history), 2026-09-26 Milestone 02: Authentication, JWT and Argon2id, Redis OTP, Guest Reconciliation, 2026-09-26 Milestone 03: Catalog Backend, Typo-Tolerant FTS, Reader API and Streak, 2026-09-26 Milestone Breakdown & Task Governance, 2026-09-27 Doc-SSOT Resync and Task 04 Honesty Fix, 2026-09-27 Full-System Audit Pass: Error Handling, Security, Performance, 2026-09-27 M6 Docs Audit: Specs Aligned to Code (+30 more)

### Community 1 - "routes/admin.rs"
Cohesion: 0.13
Nodes (14): forbidden(), require_admin(), admin_routes(), AdminCatalogQuery, DlqQuery, dropoff_analytics(), DropoffQuery, ingestion_status() (+6 more)

### Community 2 - "UI Animation Components"
Cohesion: 0.07
Nodes (33): angle_for_travel(), BEND_MAX, clear_children(), column_template(), Dir, Back, Forward, DRAG_SPRING (+25 more)

### Community 3 - "Unreleased"
Cohesion: 0.06
Nodes (35): Admin Endpoints (/admin/books, upload, jobs, dlq, analytics), Unversioned /api/* Alias Removed, Cover Delivery, Curator Desk, Earlier Hardening (ADR-20 to ADR-27), Fix Anchors, Focus and Aria Labels, Fix Quote API Contracts, Fix Reader Pagination of Long Chapters (+27 more)

### Community 4 - "embedding.rs"
Cohesion: 0.14
Nodes (14): API_KEY_HEADER, build_embedding_provider(), GeminiBatchEmbedRequest, GeminiBatchEmbedResponse, GeminiContent, GeminiEmbedding, GeminiEmbedRequest, GeminiEmbedResponse (+6 more)

### Community 5 - "Data Transfer Objects"
Cohesion: 0.09
Nodes (37): ActiveProgressDto, AdminBookRowDto, ApiResponse, ApiResponse<T>, ValidationError, AtomicCardsDto, BadgeDto, BOOK_TEXT_HITS_MAX (+29 more)

### Community 6 - "quote_repository.rs"
Cohesion: 0.23
Nodes (16): ChunkSearchRow, find_saved_quote_duplicate(), get_chapter_recap(), get_saved_quote_by_id(), get_tldr_cache(), insert_quote(), list_saved_quotes(), QuoteSaveOutcome (+8 more)

### Community 7 - "Reading Metrics Logic"
Cohesion: 0.05
Nodes (44): BookTextHitDto, close_after_event(), reading_minutes(), reading_minutes_never_zero(), roman(), roman_numerals_match_print_convention(), WORDS_PER_MINUTE, anchor_index() (+36 more)

### Community 9 - "Domain Status Logic"
Cohesion: 0.06
Nodes (35): advance_streak(), badge_matrix_matches_repository_rules(), BASE_HEARTBEAT_XP, BookStatus, Archived, Draft, Processing, Published (+27 more)

### Community 10 - "Database Schema Migrations"
Cohesion: 0.07
Nodes (43): idx_users_admin_role, idx_users_email, users, book_tags, books, idx_book_tags_reverse, idx_books_catalog_filter, idx_books_published_author_trgm (+35 more)

### Community 11 - "System Architecture Layers"
Cohesion: 0.11
Nodes (33): Badge Eligibility, Crates and Layering, crates/domain, crates/infra, crates/server, crates/shared, crates/web, Percentage (+25 more)

### Community 12 - "use_lang"
Cohesion: 0.16
Nodes (20): JobStatusDto, use_lang(), AdminPage(), CuratorDesk(), Desk, Manuscripts, Queue, Retention (+12 more)

### Community 13 - "refresh"
Cohesion: 0.16
Nodes (13): change_password(), clear_token(), is_refusal(), logout(), refresh(), refresh_once(), refresh_token(), save_progress_on_exit() (+5 more)

### Community 14 - "JWT Token Management"
Cohesion: 0.14
Nodes (41): init_redis_conn(), RedisConn, blacklist_access_token(), Claims, generate_access_token(), generate_access_token_issued_at(), generate_refresh_token(), invalidate_user_tokens() (+33 more)

### Community 15 - "Authentication Routes"
Cohesion: 0.12
Nodes (35): hit_window(), activate_user_by_email(), create_inactive_user(), delete_user_by_id(), find_user_by_email(), find_user_by_id(), update_inactive_credentials(), update_user_password() (+27 more)

### Community 16 - "prelude"
Cohesion: 0.14
Nodes (6): HOW_IT_WORKS_ID, ProgressBar(), ProgressRing(), RING_RADIUS, ACCOUNT_ID, TabBar()

### Community 17 - "System Architecture Overview"
Cohesion: 0.08
Nodes (37): Auth Endpoints (/auth/signup, verify-otp, login, refresh, logout), Axum Middleware Layers (tracing, CORS, x-request-id, security headers), Catalog Endpoints (/books, /books/search, chapters, offline-bundle), Caddy Edge (TLS, HTTP/3, SPA from dist/, /api proxy), GET /covers/{file} (immutable cache), Cover Storage (covers/<book uuid>.<ext> in covers bucket), Data Layer, Foreign-Key B-Tree Indexes (+29 more)

### Community 18 - "Web App"
Cohesion: 0.06
Nodes (39): 404 Page, admin-sorting-pigeonholes, Center Tap Dims Bars, cozy-reader-armchair-owl, Home Two-Row Catalog Preview, Illustration Assets, IndexedDB project_baca_db v3, library-bookshelf-ladder (+31 more)

### Community 19 - "routes/progress.rs"
Cohesion: 0.20
Nodes (6): get_active_progress(), merge_guest_progress(), progress_routes(), update_progress(), GuestMergeRequest, UpdateProgressRequest

### Community 20 - "server/src/main.rs"
Cohesion: 0.14
Nodes (6): init_db_pool(), init_redis_client(), redacted_endpoint(), redacted_endpoint_drops_credentials_and_path(), main(), shutdown_signal()

### Community 21 - "Admin Integration Tests"
Cohesion: 0.15
Nodes (29): admin_get(), admin_patch(), AdminContext, BOUNDARY, multipart_body(), seed_funnel(), seed_status_book(), seed_users() (+21 more)

### Community 22 - "Engineering Pillars"
Cohesion: 0.10
Nodes (11): OKF v0.2 Knowledge Catalog (knowledge/ as SSOT), [0.2.0] 2026-09-25, 768-dim Dual-Mode Embedding Engine, Edge Transport Architecture (Caddy HTTP/3), Open Knowledge Format (OKF v0.2), PostgreSQL FTS and pg_trgm Search Strategy, Redis Streams Message Broker, CLAUDE.md: Claude Code Guidelines (+3 more)

### Community 23 - "api/mod.rs"
Cohesion: 0.05
Nodes (30): BookSearchQuery, admin_books(), admin_dlq(), admin_dropoff(), admin_job(), ADMIN_PAGE_SIZE, admin_set_status(), all_badges() (+22 more)

### Community 24 - "Internationalization Module"
Cohesion: 0.11
Nodes (11): apply_lang(), ENTRIES, Lang, En, Id, LANG_KEY, lang_signal(), LangContext (+3 more)

### Community 25 - "minio"
Cohesion: 0.05
Nodes (42): MINIO_ROOT_PASSWORD, MINIO_ROOT_USER, POSTGRES_DB, POSTGRES_PASSWORD, POSTGRES_USER, interval, retries, test (+34 more)

### Community 26 - "crates/web/index.html (Trunk entry page)"
Cohesion: 0.19
Nodes (13): Design System (Espresso shell, Paper/Sepia/Espresso reading themes), Root Contexts (lang, Session, Toasts, ShellTheme), Service Worker (sw.js: shell and asset caches, /api never cached), boot.js bootstrap script (copied by Trunk), crates/web/index.html (Trunk entry page), SVG favicon /assets/rotaria-mark.svg, Google Fonts stylesheet (EB Garamond, Newsreader, Plus Jakarta Sans), style/main.css link (+5 more)

### Community 27 - "Home Page Components"
Cohesion: 0.07
Nodes (32): BookSummaryDto, prefers_reduced_motion(), reveal(), scroll_to_hash(), scroll_to_location_hash(), BookCard(), Cover(), cover_tint() (+24 more)

### Community 28 - "Records"
Cohesion: 0.04
Nodes (46): 2026-09-25 pillars: Six engineering pillars and no emoji, 2026-09-25 postgres-everything: PostgreSQL for relational, lexical, and vector data, 2026-09-25 redis-streams: Redis Streams instead of a broker, 2026-09-25 seaorm-migrations: SeaORM with paired SQL migrations, 2026-09-25 vintage-literary: Vintage Literary visual identity, 2026-09-26 ADR-14: Typed configuration, pools, and SeaORM entities, 2026-09-26 ADR-15: Authentication baseline, 2026-09-26 ADR-16: Catalog, reader, and streak backend (+38 more)

### Community 29 - "Development Invariants"
Cohesion: 0.09
Nodes (12): make test-performance-release, Service Levels, SLO Catalog Search under 5 ms, SLO Login p95 under 150 ms and OTP under 100 ms, SLO REST API p95 under 50 ms, SLO Scoped Vector Search under 10 ms, Database Schema and Index Matrix, Draft and Archived Books Answer 404 (+4 more)

### Community 30 - "Rate Limiting Middleware"
Cohesion: 0.20
Nodes (20): ServerConfig, AI_POLICY, ai_rate_limit_middleware(), AUTH_POLICY, client_ip_from_headers(), enforce(), extract_client_ip(), FALLBACK_CLIENT_IP (+12 more)

### Community 31 - "Book Repository Operations"
Cohesion: 0.13
Nodes (26): ADMIN_ROW_SELECT, AdminBookRow, AdminBookRowDto, chapter_dropoff(), character_references_decode_like_a_browser(), decode_entities(), ensure_book_published(), get_book_by_id() (+18 more)

### Community 32 - "Reader Theme Preferences"
Cohesion: 0.08
Nodes (18): FONT_SIZE_MAX, FONT_SIZE_MIN, LEADING_PRESETS, LINE_HEIGHT_MAX, LINE_HEIGHT_MIN, prefers_light(), READER_KEY, ReaderPrefs (+10 more)

### Community 33 - "2026-10-07 Audit Hardening + Rotaria Redesign"
Cohesion: 0.14
Nodes (7): knowledge/decisions.md, docker-compose.prod.yml, Request Path, Audit Hardening (2026-10-07), Edge Transport Architecture (ADR-17), 2026-09-26 Edge Transport Architecture: Caddy HTTP/3 Reverse Proxy, 2026-10-07 Audit Hardening + Rotaria Redesign

### Community 34 - "manage_sheet_focus"
Cohesion: 0.23
Nodes (5): active_element(), dialog_focusables(), FOCUSABLE, manage_sheet_focus(), trap_tab()

### Community 35 - "database_test.rs"
Cohesion: 0.09
Nodes (4): Model, Relation, Books, Tags

### Community 36 - "Application State Management"
Cohesion: 0.11
Nodes (10): api_health_check(), ApiDoc, create_app(), health_check(), not_found_handler(), REQUEST_ID_HEADER, request_id_middleware(), security_headers_middleware() (+2 more)

### Community 37 - "Test Harness Utilities"
Cohesion: 0.13
Nodes (5): CLIENT_IP_HEADER, PROVISIONED_PASSWORD, relay_connection(), Seeded, TestHarness

### Community 38 - "Client-Side Offline Storage"
Cohesion: 0.09
Nodes (23): ReadingProgressUpdateDto, drain_pending(), pending_count(), PendingProgress, queue_pending_progress(), save_progress(), save_quotes_batch(), merge_local_quotes() (+15 more)

### Community 39 - "Release 0.2.0 (2026-09-25)"
Cohesion: 0.06
Nodes (37): .agents/rules/, Graphify Query, GUIDE.md, knowledge/manual-qa.md, knowledge/product.md, knowledge/tickets/README.md, Shared Agent Rules ~/.agents/rules, FastEmbed Stub (+29 more)

### Community 40 - "AI Integration Tests"
Cohesion: 0.16
Nodes (20): one_hot(), seed_reader(), seed_test_context(), SeededAiContext, SeededReader, test_ai_rate_limiter_burst_enforcement(), test_atomic_cards_scoped_to_path_book(), test_atomic_insight_cards_cache_hit_and_miss() (+12 more)

### Community 41 - "E2E Accessibility Tests"
Cohesion: 0.14
Nodes (14): goto(), test_hero_plate_is_static(), test_home_catalog_shows_two_rows_then_everything(), test_i18n_toggle_and_persist(), test_overview_and_sheets(), test_search_returns_cards(), test_home_brand_and_hero(), test_home_renders_catalog() (+6 more)

### Community 42 - "with"
Cohesion: 0.06
Nodes (42): name, runs-on, steps, name, runs-on, steps, concurrency, cancel-in-progress (+34 more)

### Community 43 - "Worker Pipeline per Message"
Cohesion: 0.11
Nodes (10): Covers Bucket, Dead-Letter Stream stream:epub_ingestion:dlq, GET /admin/dlq, HNSW Cosine Index on book_chunks.embedding, Ingestion Message Contract, Job Hash job:{id}, POST /admin/dlq/{id}/replay, raw-epubs Storage Path (+2 more)

### Community 44 - "Project Baca ERD & PostgreSQL 17 Database Schema"
Cohesion: 0.29
Nodes (8): Migration 01 init_extensions, Migration 02 create_users_and_roles, Project Baca ERD & PostgreSQL 17 Database Schema, Identity & Auth Cluster, pg_trgm extension (trigram fuzzy search), pgvector extension (HNSW), SeaORM (crates/infra), uuid-ossp extension

### Community 45 - "environment"
Cohesion: 0.08
Nodes (33): APP_ENV, APP_PORT, CORS_ALLOWED_ORIGINS, DATABASE_URL, EMBEDDING_DIMENSION, EMBEDDING_MODEL_NAME, EMBEDDING_PROVIDER, GEMINI_API_KEY (+25 more)

### Community 46 - "otp.rs"
Cohesion: 0.18
Nodes (12): attempts_key(), generate_and_store_otp(), generate_numeric_otp(), hash_otp(), MAX_OTP_ATTEMPTS, otp_key(), OTP_LOCK_SECONDS, OTP_TTL_SECONDS (+4 more)

### Community 47 - "AppError"
Cohesion: 0.10
Nodes (17): StorageConfig, S3_CONNECT_TIMEOUT, S3_OPERATION_TIMEOUT, StorageService, StoredObject, AppError, BadRequest, Conflict (+9 more)

### Community 48 - "Auth Lifecycle Tests"
Cohesion: 0.13
Nodes (23): provision_verified_user(), json_post(), login_req(), plant_known_otp(), refresh_req(), signup_body(), test_auth_concurrent_refresh_keeps_family_alive(), test_auth_email_aggregate_lockout_twenty_failures() (+15 more)

### Community 49 - "UI Icon Library"
Cohesion: 0.23
Nodes (22): arrow_down(), arrow_right(), back(), brand_mark(), cards(), check(), close(), download() (+14 more)

### Community 50 - "CHANGELOG.md"
Cohesion: 0.12
Nodes (20): Audit Trail Protocol (doc routing by change type), Login Lockouts (5 per email+IP, 20 per email, 15 min), [0.1.0] 2026-09-25, [0.2.1] 2026-09-26, Account Abuse Controls, Argon2id Thread Offloading, Keep a Changelog 1.0.0, Database Index Optimization (Migration 07) (+12 more)

### Community 51 - "tauri.conf.json"
Cohesion: 0.06
Nodes (30): app, security, windows, cwd, script, cwd, script, wait (+22 more)

### Community 52 - "test_reader_flip.py"
Cohesion: 0.40
Nodes (7): _drag(), _open_long_chapter(), test_drag_turns_like_paper(), test_keyboard_turns_and_dims(), test_reduced_motion_turns_without_paper(), test_space_on_a_control_activates_it_and_pinch_zoom_stays(), test_spread_starts_at_the_desktop_breakpoint()

### Community 53 - "Python Worker Scripts"
Cohesion: 0.12
Nodes (8): dead_letter(), main(), _opf_path(), _raise_shutdown(), release_job(), run(), set_job(), WorkerShutdown

### Community 54 - "quotes.rs"
Cohesion: 0.15
Nodes (13): REDIS_RETRY_AFTER_SECS, auth_state_unavailable(), AuthUser, card_url_for(), get_quote_card(), handle_list_saved_quotes(), handle_save_quote(), handle_save_quotes_batch() (+5 more)

### Community 55 - "deserialize"
Cohesion: 0.07
Nodes (18): Model, Relation, UserBadges, Model, Relation, Books, Chapters, Users (+10 more)

### Community 56 - "queue.rs"
Cohesion: 0.13
Nodes (17): dlq_entry(), entry_fields(), get_job_status(), INGESTION_DLQ_STREAM, ingestion_entries_trim_the_stream_approximately(), ingestion_entry(), INGESTION_GROUP, INGESTION_STREAM (+9 more)

### Community 57 - "Security Model"
Cohesion: 0.09
Nodes (17): API Surface, Argon2id Password Hashing (19 MiB, 2 iterations), Data Exposure Rules (drafts hidden, values not echoed, secrets masked), DELETE /me/sessions/{id}, API Error Codes, GET /me/sessions, Open Security Work in Tickets, POST /auth/revoke-all (+9 more)

### Community 58 - "GUIDE.md: Developer Guide"
Cohesion: 0.06
Nodes (55): make check, make db-up migrate-up seed-dev, make dev-web, Makefile, Quick Commands, Deployment, make build, make prod-build prod-up prod-down prod-logs (+47 more)

### Community 59 - "Email Notification Service"
Cohesion: 0.18
Nodes (11): EmailConfig, build_transport(), mailpit_accepts_cleartext_delivery_when_running(), mask_email(), mask_email_hides_the_local_part(), otp_message(), otp_message_addresses_sender_and_recipient(), send_otp_email() (+3 more)

### Community 60 - "AppConfig"
Cohesion: 0.12
Nodes (12): AppConfig, AuthConfig, cleartext_smtp_is_refused_in_production_only(), DatabaseConfig, production(), production_shape_passes_validation(), public_limit_without_proxy_trust_is_refused_in_production(), RedisConfig (+4 more)

### Community 61 - "test_sessions.py"
Cohesion: 0.17
Nodes (13): api_post(), mailpit_otp(), register_reader(), _count_local_quotes(), _put_local_quote(), _sign_in_storage(), test_authed_autosave_shelf_cleanup(), test_drain_keeps_writes_queued_meanwhile_and_never_doubles() (+5 more)

### Community 62 - "Index Matrix (17 query-critical indexes)"
Cohesion: 0.15
Nodes (17): Migration 05 create_tldr_cache, Migration 07 optimize_indexes_and_foreign_keys, index idx_book_tags_reverse, index idx_books_catalog_filter, index idx_books_published_year_id, index idx_reading_activity_user_date, index idx_saved_quotes_user_book, index idx_tldr_cache_chapter_id (+9 more)

### Community 63 - "HTML Sanitization Utilities"
Cohesion: 0.12
Nodes (5): SanitizeTest, extract_text(), __init__(), sanitize_html(), _Sanitizer

### Community 64 - "test_worker_live.py"
Cohesion: 0.14
Nodes (6): 2026-10-08 ADR-31 (absorbs ADR-23, 2026-09-27): Tests own their rows, ClaimAndShutdownLiveTest, DlqTest, probe_settings(), silent_listener(), ensure_group()

### Community 65 - "Product"
Cohesion: 0.10
Nodes (20): 10. Design identity, 11. Success metrics, 12. Scope, 1. Vision, 2. Target users, 3. Content and licensing, 4. Roles and access, 5. User stories (+12 more)

### Community 66 - "Book Cover Service"
Cohesion: 0.20
Nodes (7): accepts_only_uuid_named_images(), content_type_prefers_stored_image_type(), COVER_CACHE_CONTROL, cover_content_type(), covers_routes(), get_cover(), is_cover_file_name()

### Community 67 - "backfill_chunks.rs"
Cohesion: 0.12
Nodes (9): BATCH_SIZE, chunk_text(), CHUNK_WORDS, main(), MIN_CHARS, strip_html(), list_badges(), list_user_badges() (+1 more)

### Community 68 - "AI Insight Components"
Cohesion: 0.16
Nodes (11): AtomicCards(), CatchupRecap(), InsightList(), InsightText(), keep_quote(), QuoteFinder(), Sheet(), string_field() (+3 more)

### Community 69 - "reliability_test.rs"
Cohesion: 0.15
Nodes (9): BookCatalogPort, get(), get_bearer(), offline_app(), test_redis_outage_answers_503_and_heals_without_restart(), test_reliability_disconnected_database_fallback(), test_reliability_mock_repository_fault_injection(), test_signup_fails_closed_when_smtp_unreachable() (+1 more)

### Community 70 - "API Client Wrapper"
Cohesion: 0.28
Nodes (11): admin_replay(), api_base(), authed(), authed_send(), parse_envelope(), patch(), post(), post_once() (+3 more)

### Community 71 - "BuildTask.kt"
Cohesion: 0.15
Nodes (3): BuildTask, Config, RustPlugin

### Community 72 - "server"
Cohesion: 0.14
Nodes (18): API_BASE_URL, args, context, dockerfile, postgres, redis, build, condition (+10 more)

### Community 73 - "TICK-013: Shell Tauri untuk Android, iOS, dan Desktop"
Cohesion: 0.20
Nodes (7): apps/tauri/, Fase 4: Fitur, TICK-013: Shell Tauri untuk Android, iOS, dan Desktop, API origin baked at build time, navigator.share unavailable in some WebViews, No rotaria deep link scheme, Service worker always registered

### Community 74 - "services"
Cohesion: 0.15
Nodes (12): image, restart, volumes, services, redis, volumes, caddy_config, caddy_data (+4 more)

### Community 75 - "Ticket Board (Tiket Kerja)"
Cohesion: 0.13
Nodes (18): assets/illustrations/, .env.production.example, knowledge/tasks/ (old task board), Audit 2026-10-08, Fase 1: Keandalan, Fase 2: Keamanan dan Sesi, Fase 3: Kualitas dan Bobot, Fase 5: Peluncuran (+10 more)

### Community 76 - "HttpError"
Cohesion: 0.14
Nodes (9): HttpError, redis_unavailable(), get_book(), get_chapter(), get_offline_bundle(), list_books(), search_book_text(), search_books() (+1 more)

### Community 77 - "Toast Notification System"
Cohesion: 0.12
Nodes (7): ERROR_MS, INFO_MS, Toast, Toasts, ToastStack(), App(), NotFound()

### Community 78 - "Visual Regression Helpers"
Cohesion: 0.27
Nodes (5): check(), externalize(), replace(), _is_inline(), main()

### Community 79 - "unit_tests.rs"
Cohesion: 0.13
Nodes (6): entry_count(), local_quote_key(), LocalQuoteRecord, OfflineChapterRecord, test_i18n_dictionary_complete_in_both_langs(), test_local_quote_key_is_stable_and_distinct()

### Community 80 - "AppState"
Cohesion: 0.16
Nodes (12): AppState, auth_routes(), user_routes(), books_routes(), gamification_routes(), get_my_streak(), HEARTBEAT_MIN_INTERVAL_SECS, list_badges() (+4 more)

### Community 81 - "AI Insight Routes"
Cohesion: 0.42
Nodes (4): get_atomic_cards(), get_chapter_recap(), insights_routes(), resolve_chapter()

### Community 82 - "OWASP Security Tests"
Cohesion: 0.23
Nodes (13): test_counter_keys_always_expire(), test_owasp_login_brute_force_lockout(), test_owasp_otp_cooldown_and_email_bombing_prevention(), test_owasp_rate_limiting_headers_and_ip_extraction(), test_owasp_revoke_all_sessions(), test_owasp_security_headers_and_error_handling(), test_owasp_structured_validation_error_details(), test_owasp_timing_attack_mitigation_on_login() (+5 more)

### Community 83 - "Axe Accessibility Scans"
Cohesion: 0.25
Nodes (11): _assert_clean(), _scan(), test_axe_admin(), test_axe_home(), test_axe_overview(), test_axe_paper_surface(), test_axe_profile_guest(), test_axe_quote_modal() (+3 more)

### Community 84 - "Settings"
Cohesion: 0.23
Nodes (8): ReclaimTest, embed_batch(), make_db(), make_minio(), make_redis(), process_message(), reclaim_idle(), Settings

### Community 85 - "conftest.py"
Cohesion: 0.15
Nodes (7): book_with_chapters(), browser_context_args(), clean_page(), install_fetch_shim(), long_chapter_html(), reset_signup_cap(), stable_book_id()

### Community 86 - "Password Hashing Logic"
Cohesion: 0.21
Nodes (6): hash_password(), test_dummy_argon2_hash_validity(), test_hash_and_verify_password_success(), test_verify_password_invalid_hash(), test_verify_password_wrong_password(), verify_password()

### Community 87 - "SmtpSecurity"
Cohesion: 0.40
Nodes (3): SmtpSecurity, None, StartTls

### Community 89 - "Progress Repository Operations"
Cohesion: 0.20
Nodes (11): Model, check_and_award_badges(), get_active_progress(), get_streak(), record_heartbeat(), seconds_read_on(), update_progress(), ReadingHeartbeatRequest (+3 more)

### Community 90 - "Catalog Integration Tests"
Cohesion: 0.25
Nodes (13): seed_book_with_chapter(), seed_test_catalog(), SeededCatalog, test_book_overview_chapter_and_offline_bundle(), test_book_text_search_finds_paragraphs(), test_catalog_listing_and_filtering(), test_catalog_typo_tolerant_fts_search(), test_gamification_heartbeat_and_badges() (+5 more)

### Community 91 - "AI Memory Skills"
Cohesion: 0.19
Nodes (8): Brain content, Provenance, Safety, Skills, brain-router skill, memory-care skill, memory-recall skill, recall tool (read-only, bounded result budget)

### Community 92 - "EPUB Parsing Logic"
Cohesion: 0.15
Nodes (9): ColumnWidthTest, make_epub(), ParseTest, clip(), convert_cover_to_webp(), parse_epub(), ParsedChapter, ParsedEpub (+1 more)

### Community 93 - "_gemini_post"
Cohesion: 0.10
Nodes (8): GeminiTransportTest, RetryAndRecapTests, fake_urlopen(), backoff_seconds(), build_recap_input(), _gemini_post(), generate_json(), _strip_fences()

### Community 94 - "NOTICE.md: Copyright and Legal Licenses"
Cohesion: 0.16
Nodes (14): Rotaria Two-Surface Design (Espresso shell, Paper surface), Vintage Literary Aesthetic, Rotaria Two-Surface Design, Berne Convention, Disclaimer of Warranty, NOTICE.md: Copyright and Legal Licenses, Indonesian Law No. 28/2014 on Copyright, Project Gutenberg (+6 more)

### Community 95 - "Responsive Design Tests"
Cohesion: 0.47
Nodes (6): _page_at(), test_home_and_book_fit_viewport(), test_phone_rail_starts_on_the_content_edge(), test_reader_paginates_long_chapter(), test_two_screen_spread_keeps_text_off_the_hinge(), _visible()

### Community 96 - "UI Component Storybook"
Cohesion: 0.22
Nodes (10): _story(), test_atomic_cards_404_tolerant(), test_auth_sheet_modes(), test_curator_desk_tabs_rows_replay_and_funnel(), test_quote_finder_empty_state(), test_quote_finder_error_branch(), test_site_header_toggle_no_reload(), assert_no_page_errors() (+2 more)

### Community 97 - "Data Dictionary Documentation"
Cohesion: 0.19
Nodes (13): books(), 1. Engineering Invariants & Data Philosophy, 2. Entity Relationship Diagram, 3. Data Dictionary & Table Specifications, 4. Index Matrix & Query Patterns, 5. Migration History (`migrations/`), A. Identity & Auth Cluster, B. Catalog & Taxonomy Cluster (+5 more)

### Community 98 - "test_worker.py"
Cohesion: 0.12
Nodes (7): ArchivedBookTest, ArchiveLimitsTest, ChunkTest, FakeCursor, StreamCapTest, check_archive_limits(), chunk_words()

### Community 99 - "User Session State"
Cohesion: 0.28
Nodes (3): is_authed(), me(), Session

### Community 100 - "Database Schema & Search"
Cohesion: 0.47
Nodes (9): Migration 03 create_books_and_tags, table book_tags, table books, Catalog & Taxonomy Cluster, Discovery Search Query (<5ms), index idx_books_published_author_trgm, index idx_books_published_fts, index idx_books_published_title_trgm (+1 more)

### Community 101 - "AuthSheet"
Cohesion: 0.17
Nodes (10): LoginRequest, PasswordChangedDto, SignupRequest, TokenResponse, VerifyOtpRequest, login(), merge_guest_progress(), signup() (+2 more)

### Community 102 - "Audit Reports & Screenshots"
Cohesion: 0.11
Nodes (16): Audit Screenshot 05: Atomic Insights Modal, Audit Screenshot 08: Guest Profile, Audit Screenshot 10: Save Offline on Book Overview, Quote Finder Empty State Screenshot, Saved Offline Book Overview Screenshot, Guest Profile CTA Screenshot, Audit Web Menyeluruh — 2026-09-27 (pra-M7), Kontrak yang legitimate (bukan bug) (+8 more)

### Community 104 - "3. Checklist"
Cohesion: 0.13
Nodes (14): 1. Before you start, 2. Covered by automation, 3. Checklist, 4. Release gate, Accessibility, Accounts and sessions, App shell (Android device and desktop), Appearance and language (+6 more)

### Community 105 - "components/auth.rs"
Cohesion: 0.14
Nodes (5): friendly_error(), Mode, Login, Otp, Register

### Community 106 - "test_offline_shell.py"
Cohesion: 0.27
Nodes (7): _controlled(), _new_page(), test_home_and_reader_load_nothing_from_third_parties(), test_house_fonts_offline_on_home_and_reader(), test_native_shell_sets_the_api_base_and_skips_the_worker(), test_shell_cache_keeps_one_navigation_entry(), test_unhashed_files_are_fresh_on_the_next_visit()

### Community 107 - "with"
Cohesion: 0.15
Nodes (13): with, cache-from, cache-to, context, images, labels, name, password (+5 more)

### Community 108 - "Navigation Integration Tests"
Cohesion: 0.24
Nodes (9): _section_top(), test_cover_art_never_shows_a_broken_image(), test_focused_search_field_clears_the_sticky_header(), test_header_marks_catalog_and_shelf(), test_how_it_works_scrolls_from_the_shelf(), test_tab_bar_marks_search_and_profile(), test_tab_stays_inside_an_open_sheet(), test_theme_toggle_names_its_target_and_follows_the_system() (+1 more)

### Community 109 - "release_notes.py"
Cohesion: 0.22
Nodes (4): main(), changelog_section(), main(), workspace_version()

### Community 110 - "users.rs"
Cohesion: 0.20
Nodes (7): Model, Relation, ReadingActivityLogs, SavedQuotes, UserBadges, UserReadingProgress, UserReadingStreak

### Community 111 - "Relation"
Cohesion: 0.17
Nodes (9): Model, Relation, BookChunks, BookTags, Chapters, ReadingActivityLogs, SavedQuotes, TldrCache (+1 more)

### Community 112 - "test_reader_flow.py"
Cohesion: 0.20
Nodes (25): api_get(), _account_anchor(), _chapter_no(), _context(), _guest_record(), _open(), _reader_context(), _reopens_saved_page() (+17 more)

### Community 113 - "Database Migration Scripts"
Cohesion: 0.76
Nodes (6): init_table(), migrate_down(), migrate_status(), migrate_up(), psql_cmd(), migrate.sh script

### Community 114 - "Chapter Data Model"
Cohesion: 0.22
Nodes (6): Model, Relation, BookChunks, Books, SavedQuotes, TldrCache

### Community 117 - "tldr_cache.rs"
Cohesion: 0.25
Nodes (4): Model, Relation, Books, Chapters

### Community 118 - "Release 0.1.0 (2026-09-25)"
Cohesion: 0.20
Nodes (9): assets/README.md, DISTRIBUTED.md, docker-compose.yml, Documentation Consolidated (2026-10-09), Local Infrastructure docker-compose.yml, Multi-Agent Guidelines, Project Initialization, Release 0.1.0 (2026-09-25) (+1 more)

### Community 121 - "images"
Cohesion: 0.20
Nodes (10): name, needs, runs-on, strategy, dockerfile, image, images, include (+2 more)

### Community 122 - "table users"
Cohesion: 0.42
Nodes (9): Migration 06 create_progress_and_gamification, table badges, table reading_activity_logs, Reading Progress & Gamification Cluster, table saved_quotes, table user_badges, table user_reading_progress, table user_reading_streaks (+1 more)

### Community 123 - "Knowledge Graph Query"
Cohesion: 0.40
Nodes (4): Answer, Outcome, Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya., Source Nodes

### Community 124 - "Project Architecture Layers"
Cohesion: 0.70
Nodes (5): domain, infra, server, shared, web

### Community 125 - "Python Worker Dependencies"
Cohesion: 0.40
Nodes (4): minio==7.2.20, pg8000==1.31.5, pillow==12.3.0, redis==8.1.0

### Community 131 - "reading_activity_logs.rs"
Cohesion: 0.25
Nodes (4): Model, Relation, Books, Users

### Community 135 - "user_badges.rs"
Cohesion: 0.29
Nodes (4): Model, Relation, Badges, Users

### Community 139 - "tauri/src/lib.rs"
Cohesion: 0.12
Nodes (8): API_BASE, book_links_open_the_book_and_nothing_else(), build(), GateState, LinkGate, navigate(), route(), route_for()

### Community 140 - "keep_claimed"
Cohesion: 0.22
Nodes (3): ClaimAndShutdownTest, claim_heartbeat_seconds(), keep_claimed()

### Community 161 - "test_api_contract.py"
Cohesion: 0.33
Nodes (5): call(), test_envelope_and_health(), test_merge_cap_boundary(), test_save_quote_requires_auth(), test_unknown_book_search_404()

### Community 162 - "steps"
Cohesion: 0.22
Nodes (8): description, name, runs, steps, using, name, run, shell

### Community 176 - "steps"
Cohesion: 0.28
Nodes (9): GH_TOKEN, VERSION, steps, steps, steps, env, id, run (+1 more)

### Community 177 - "publish"
Cohesion: 0.22
Nodes (9): permissions, publish, permissions, contents, packages, name, needs, permissions (+1 more)

### Community 178 - "profile.rs"
Cohesion: 0.14
Nodes (16): active_progress(), copy_text(), share_text(), SiteHeader(), share_quote(), use_session(), ShellLayout(), use_toasts() (+8 more)

### Community 179 - "table book_chunks"
Cohesion: 0.36
Nodes (8): Migration 04 create_chapters_and_chunks, table book_chunks, table chapters, Chapters & Semantic Vectors Cluster, index idx_book_chunks_hnsw_embedding, index idx_book_chunks_lookup, Scoped Quote Search Query (<10ms), index uq_chapters_book_number

### Community 180 - "purge_test_debris.py"
Cohesion: 0.33
Nodes (3): main(), psql(), purge_seed_survivors()

### Community 184 - "knowledge/index.md"
Cohesion: 0.29
Nodes (4): GBrain Skill Pack, knowledge/audits/, Knowledge Documents Table, Root Documents Table

### Community 186 - "book_chunks.rs"
Cohesion: 0.25
Nodes (4): Model, Relation, Books, Chapters

### Community 187 - "minio"
Cohesion: 0.25
Nodes (8): MINIO_ROOT_PASSWORD, MINIO_ROOT_USER, command, environment, image, restart, volumes, minio

### Community 227 - "covers_test.rs"
Cohesion: 0.48
Nodes (3): get_cover(), test_cover_missing_and_invalid_names_are_404(), test_cover_roundtrip_through_storage()

### Community 228 - "edge"
Cohesion: 0.29
Nodes (7): depends_on, environment, ports, restart, volumes, DOMAIN, edge

### Community 229 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 230 - "release.yml"
Cohesion: 0.33
Nodes (5): jobs, name, on, push, tags

### Community 240 - "notes"
Cohesion: 0.40
Nodes (5): notes, name, outputs, runs-on, version

## Ambiguous Edges - Review These
- `2026-09-28 Task 09 Absorbed planning/` → `Defense-in-Depth Security Architecture`  [AMBIGUOUS]
  CHANGELOG.md · relation: conceptually_related_to
- `Release 0.2.0 (2026-09-25)` → `Version stays 0.1.0`  [AMBIGUOUS]
  AGENTS.md · relation: conceptually_related_to
- `Redis Streams as Single Queue` → `SMTP via lettre`  [AMBIGUOUS]
  CHANGELOG.md · relation: conceptually_related_to
- `Release 0.2.1 (2026-09-26)` → `Version stays 0.1.0`  [AMBIGUOUS]
  AGENTS.md · relation: conceptually_related_to

## Knowledge Gaps
- **211 isolated node(s):** `rotaria-app`, `Draft`, `Processing`, `Published`, `Archived` (+206 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 1098 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **115 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `TICK-013: Shell Tauri untuk Android, iOS, dan Desktop` connect `TICK-013: Shell Tauri untuk Android, iOS, dan Desktop` to `Ticket Board (Tiket Kerja)`, `api/mod.rs`?**
  _High betweenness centrality (0.216) - this node is a cross-community bridge._
- **Are the 30 inferred relationships involving `use_lang()` (e.g. with `AuthSheet()` and `BookCard()`) actually correct?**
  _`use_lang()` has 30 INFERRED edges - model-reasoned connections that need verification._
- **What connects `rotaria-app`, `Draft`, `Processing` to the rest of the system?**
  _211 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `knowledge/log.md` be split into smaller, more focused modules?**
  _Cohesion score 0.0971322849213691 - nodes in this community are weakly interconnected._
- **Why does `Ticket Board (Tiket Kerja)` connect `Ticket Board (Tiket Kerja)` to `TICK-013: Shell Tauri untuk Android, iOS, dan Desktop`?**
  _High betweenness centrality (0.208) - this node is a cross-community bridge._

### Low-confidence Hints
_AMBIGUOUS edges — the extractor was unsure. Verify before acting on these._

- **What is the exact relationship between `2026-09-28 Task 09 Absorbed planning/` and `Defense-in-Depth Security Architecture`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `Release 0.2.0 (2026-09-25)` and `Version stays 0.1.0`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._