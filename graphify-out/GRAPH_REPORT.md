# Graph Report - project-baca  (2026-10-08)

## Corpus Check
- 204 files · ~405,390 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 17 file(s) not represented in the graph (top: (none) 5, .example 2, .prod 1)

## Summary
- 2891 nodes · 6095 edges · 216 communities (126 shown, 90 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 397 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `a380e390`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- shared/src/lib.rs
- axe.min.js
- A
- Lang
- domain/src/lib.rs
- routes/admin.rs
- quotes.rs
- e
- books
- routes/auth.rs
- otp.rs
- AppError
- config.rs
- T
- reader/mod.rs
- Tp
- server/src/lib.rs
- api/mod.rs
- serde
- theme.rs
- HttpError
- conftest.py
- worker.py
- web/src/storage/mod.rs
- String
- get_atomic_cards
- embedding.rs
- quote_repository.rs
- auth_test.rs
- flip.rs
- icons.rs
- home.rs
- backfill_chunks.rs
- post
- extract_text
- api_boundary_test.rs
- prelude
- security_owasp_test.rs
- routes/progress.rs
- Fh
- Toasts
- AuthUser
- AppState
- queue.rs
- n
- test_worker_live.py
- admin_test.rs
- m
- reliability_test.rs
- Session
- TestHarness
- parse_epub
- book_card.rs
- test_sessions.py
- Value
- test_worker.py
- components/progress.rs
- shared
- web/src/lib.rs
- chapters.rs
- test_axe.py
- playwright_sync_api
- semantic_ai_test.rs
- rate_limit.rs
- RetryAndRecapTests
- email.rs
- Model
- test_components.py
- Changelog
- externalize_inline_scripts.py
- components/insights.rs
- Log — Project Baca Knowledge Bundle
- Chronological Decision Records
- sea_orm
- use_lang
- test_responsive.py
- Ingestion Worker (`python_worker/`)
- progress_repository.rs
- manage_sheet_focus
- Log — Project Baca Knowledge Bundle
- test_reader_flip.py
- books
- Manual Testing — Milestone 03: Catalog, Reader Engine & Gamification
- validate_avatar_url
- Manual Testing — Milestone 07: QA Gates, M7 Tracks & Launch Readiness
- Vec
- Manual Testing — Milestone 06: Leptos Web Reader (Rotaria P1–P7)
- 3. REST API Endpoint Contracts
- 2. Step-by-Step Test Procedure
- Manual Testing — Milestone 04: Semantic AI, Quotes & Insights
- Manual Testing — Milestone 05: Ingestion Pipeline & Admin
- PRD — Project Baca (MVP: Reader + AI Insights)
- user_repository.rs
- migrate.sh
- 3. Screen Wireframes
- catalog_test.rs
- Entity
- Model
- user_badges.rs
- test_reader_flow.py
- 2. Prerequisites & Service Status Verification
- manual-test/README.md
- Audit menyeluruh Rotaria, 8 Oktober 2026
- 3. Data Dictionary & Table Specifications
- generate_json
- GeminiEmbeddingProvider
- tasks/README.md
- 2. Module 2: Reflowable Reader & Offline Engine
- AGENTS.md — AI Agent Guidelines & Governance
- CLAUDE.md — Claude Code Guidelines for Project Baca
- mo
- Model
- Model
- users.rs
- assert_no_page_errors
- Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya.
- Knowledge Catalog Index & Directory Architecture Map (OKF v0.2)
- 22 — Tauri Shell
- Project Baca (Rotaria)
- 3. Module 3: Authentication, Users & Reconciliation
- shared
- 4. Module 4: Semantic AI & Insights
- Knowledge Index
- Entity
- Entity
- Entity
- Entity
- Entity
- reading_activity_logs.rs
- Entity
- 4. Reader: Zones, CFI Anchor, Typography, Heartbeat
- Entity
- Entity
- Entity
- Entity
- Entity
- Brain content
- Brain content
- 20 — Launch Remaining and Trademark
- 21 — Rate-Limit Follow-ups
- 24 — Responsive Follow-ups
- Rotaria Windmill Icon
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
- 26 — Redis reconnect, boot fail-fast, outbound timeouts
- pg_backup.sh
- 27 — Auth follow-ups
- Admin Sorting Illustration
- Reader View Illustration
- Library Catalog Illustration
- Ingestion Monitor Illustration
- AI Quote Finder Illustration
- 29 — Edge and compose hygiene
- GBrain Design Reference
- 30 — Slim the WASM bundle
- 31 — Reader position integrity
- 32 — Session and API client follow-ups
- Production Compose Configuration
- 33 — Shell and accessibility polish
- 34 — Service worker and fonts
- Project Baca — Operational Status & Backlog Log
- GEMINI.md — Gemini & Google Antigravity Guidelines
- brain-router/SKILL.md
- memory-care/SKILL.md
- memory-recall/SKILL.md

## God Nodes (most connected - your core abstractions)
1. `AppError` - 119 edges
2. `AppState` - 75 edges
3. `n()` - 62 edges
4. `Log — Project Baca Knowledge Bundle` - 51 edges
5. `HttpError` - 50 edges
6. `Chronological Decision Records` - 37 edges
7. `assert_no_page_errors()` - 34 edges
8. `Log — Project Baca Knowledge Bundle` - 34 edges
9. `AuthUser` - 33 edges
10. `use_lang()` - 33 edges

## Surprising Connections (you probably didn't know these)
- `2026-09-27 — Rotaria P1–P7: Rebrand, i18n, Hero, Gamification, Social, Profile, Admin` --references--> `me()`  [INFERRED]
  knowledge/log.md → crates/web/tests/a11y/axe.min.js
- `2026-09-27 — Rotaria P1–P7: Rebrand, i18n, Hero, Gamification, Social, Profile, Admin` --references--> `me()`  [INFERRED]
  knowledge/opencode/mirror-backup/kb-log.md → crates/web/tests/a11y/axe.min.js
- `2026-09-27 — Milestone 06: Leptos WASM Frontend Complete (ADR-25)` --references--> `T()`  [INFERRED]
  MEMORY.md → crates/web/tests/a11y/axe.min.js
- `1. Engineering Invariants & Data Philosophy` --references--> `books()`  [INFERRED]
  knowledge/erd.md → crates/web/tests/component/test_components.py
- `5. Migration History (`migrations/`)` --references--> `books()`  [INFERRED]
  knowledge/erd.md → crates/web/tests/component/test_components.py

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Infrastructure Services** — redis_7, minio [EXTRACTED 1.00]

## Communities (216 total, 90 thin omitted)

### Community 0 - "shared/src/lib.rs"
Cohesion: 0.12
Nodes (42): ActiveProgressDto, AdminBookPatchRequest, AdminBookRowDto, ApiResponse, BadgeDto, BookCatalogQuery, BookDetailDto, BookSearchQuery (+34 more)

### Community 1 - "axe.min.js"
Cohesion: 0.02
Nodes (39): ae(), be(), ce(), de(), df(), ei(), ep(), ff() (+31 more)

### Community 2 - "A"
Cohesion: 0.07
Nodes (39): A(), Am(), As(), bg(), Bm(), bs(), cg(), e0() (+31 more)

### Community 3 - "Lang"
Cohesion: 0.12
Nodes (9): apply_lang(), ENTRIES, Lang, LANG_KEY, lang_signal(), LangContext, provide_lang(), badge_description() (+1 more)

### Community 4 - "domain/src/lib.rs"
Cohesion: 0.06
Nodes (24): advance_streak(), BASE_HEARTBEAT_XP, BookStatus, COMPLETION_MAX, COMPLETION_MIN, DAILY_MILESTONE_BONUS_XP, DAILY_THRESHOLD_SECONDS, day() (+16 more)

### Community 5 - "routes/admin.rs"
Cohesion: 0.14
Nodes (14): forbidden(), require_admin(), admin_routes(), AdminCatalogQuery, DlqQuery, dropoff_analytics(), DropoffQuery, ingestion_status() (+6 more)

### Community 6 - "quotes.rs"
Cohesion: 0.22
Nodes (11): card_url_for(), get_quote_card(), handle_list_saved_quotes(), handle_save_quote(), handle_save_quotes_batch(), outcome(), quotes_routes(), render_vintage_quote_svg() (+3 more)

### Community 7 - "e"
Cohesion: 0.12
Nodes (37): bc(), bh(), bl(), cf(), cp(), D(), dl(), dp() (+29 more)

### Community 8 - "books"
Cohesion: 0.07
Nodes (43): idx_users_admin_role, idx_users_email, users, book_tags, books, idx_book_tags_reverse, idx_books_catalog_filter, idx_books_published_author_trgm (+35 more)

### Community 9 - "routes/auth.rs"
Cohesion: 0.18
Nodes (22): auth_routes(), change_password(), delete_me(), get_me(), INVALID_CREDENTIALS, issue_token_pair(), list_my_sessions(), login() (+14 more)

### Community 10 - "otp.rs"
Cohesion: 0.05
Nodes (27): generate_and_store_otp(), generate_numeric_otp(), hash_otp(), MAX_OTP_ATTEMPTS, OTP_LOCK_SECONDS, OTP_TTL_SECONDS, OtpRecord, test_hash_otp_consistency() (+19 more)

### Community 12 - "AppError"
Cohesion: 0.06
Nodes (60): list_badges(), list_user_badges(), seed_default_badges_if_empty(), ADMIN_ROW_SELECT, AdminBookRow, AdminBookRowDto, chapter_dropoff(), ensure_book_published() (+52 more)

### Community 13 - "config.rs"
Cohesion: 0.10
Nodes (11): AiConfig, AppConfig, AuthConfig, DatabaseConfig, EmailConfig, RedisConfig, SmtpSecurity, StorageConfig (+3 more)

### Community 14 - "T"
Cohesion: 0.12
Nodes (29): a1(), af(), cc(), dc(), fe(), g1(), gc(), h1() (+21 more)

### Community 15 - "reader/mod.rs"
Cohesion: 0.08
Nodes (22): anchor_index(), columns_for(), css_px(), Paddings, page_of(), paragraphs(), TWO_COLUMN_MIN_WIDTH, ANCHOR_END (+14 more)

### Community 16 - "Tp"
Cohesion: 0.11
Nodes (23): _0(), Ap(), bf(), r(), dg(), Ef(), eu(), gf() (+15 more)

### Community 17 - "server/src/lib.rs"
Cohesion: 0.11
Nodes (11): api_health_check(), ApiDoc, create_app(), health_check(), not_found_handler(), REQUEST_ID_HEADER, request_id_middleware(), security_headers_middleware() (+3 more)

### Community 18 - "api/mod.rs"
Cohesion: 0.09
Nodes (19): ADMIN_PAGE_SIZE, as_html_element(), asset_url(), change_password(), clear_token(), encode_param(), is_authed(), logout() (+11 more)

### Community 19 - "serde"
Cohesion: 0.12
Nodes (6): Model, Relation, Model, Relation, Model, Relation

### Community 20 - "theme.rs"
Cohesion: 0.09
Nodes (13): FONT_SIZE_MAX, FONT_SIZE_MIN, LEADING_PRESETS, LINE_HEIGHT_MAX, LINE_HEIGHT_MIN, prefers_light(), READER_KEY, ReaderPrefs (+5 more)

### Community 21 - "HttpError"
Cohesion: 0.16
Nodes (7): HttpError, gamification_routes(), get_my_streak(), HEARTBEAT_MIN_INTERVAL_SECS, list_badges(), list_user_badges(), record_heartbeat()

### Community 22 - "conftest.py"
Cohesion: 0.13
Nodes (10): api_get(), book_with_chapters(), browser_context_args(), clean_page(), stable_book_id(), call(), test_envelope_and_health(), test_merge_cap_boundary() (+2 more)

### Community 23 - "worker.py"
Cohesion: 0.16
Nodes (10): convert_cover_to_webp(), ensure_group(), main(), make_db(), make_minio(), make_redis(), process_message(), run() (+2 more)

### Community 24 - "web/src/storage/mod.rs"
Cohesion: 0.08
Nodes (16): clear(), db(), DB_NAME, DB_VERSION, delete(), get_all(), local_quote_key(), LocalQuoteRecord (+8 more)

### Community 25 - "String"
Cohesion: 0.21
Nodes (23): active_progress(), admin_books(), admin_dlq(), admin_dropoff(), admin_job(), admin_replay(), admin_set_status(), all_badges() (+15 more)

### Community 26 - "get_atomic_cards"
Cohesion: 0.31
Nodes (4): get_atomic_cards(), get_chapter_recap(), insights_routes(), resolve_chapter()

### Community 27 - "embedding.rs"
Cohesion: 0.16
Nodes (14): API_KEY_HEADER, build_embedding_provider(), GeminiBatchEmbedRequest, GeminiBatchEmbedResponse, GeminiContent, GeminiEmbedding, GeminiEmbedRequest, GeminiEmbedResponse (+6 more)

### Community 28 - "quote_repository.rs"
Cohesion: 0.25
Nodes (14): ChunkSearchRow, find_saved_quote_duplicate(), get_chapter_recap(), get_saved_quote_by_id(), get_tldr_cache(), insert_quote(), list_saved_quotes(), QuoteSaveOutcome (+6 more)

### Community 29 - "auth_test.rs"
Cohesion: 0.14
Nodes (13): login_req(), provision_verified_user(), refresh_req(), test_auth_concurrent_refresh_keeps_family_alive(), test_auth_email_aggregate_lockout_twenty_failures(), test_auth_pair_lockout_five_failures(), test_auth_password_change_kills_other_session_e2e(), test_auth_refresh_grace_window() (+5 more)

### Community 30 - "flip.rs"
Cohesion: 0.07
Nodes (28): angle_for_travel(), BEND_MAX, clear_children(), column_template(), Dir, DRAG_SPRING, DragStart, FLICK_VELOCITY (+20 more)

### Community 31 - "icons.rs"
Cohesion: 0.23
Nodes (22): arrow_down(), arrow_right(), back(), brand_mark(), cards(), check(), close(), download() (+14 more)

### Community 32 - "home.rs"
Cohesion: 0.10
Nodes (20): prefers_reduced_motion(), reveal(), scroll_to_hash(), scroll_to_location_hash(), catalog_columns(), CATALOG_PREVIEW_ROWS, HomePage(), LanguageFilter (+12 more)

### Community 33 - "backfill_chunks.rs"
Cohesion: 0.12
Nodes (8): BATCH_SIZE, chunk_text(), CHUNK_WORDS, main(), MIN_CHARS, strip_html(), Model, Relation

### Community 34 - "post"
Cohesion: 0.13
Nodes (20): ReadingProgressUpdateDto, authed(), authed_send(), drain_pending(), heartbeat(), login(), merge_guest_progress(), parse_envelope() (+12 more)

### Community 35 - "extract_text"
Cohesion: 0.12
Nodes (5): SanitizeTest, extract_text(), __init__(), sanitize_html(), _Sanitizer

### Community 36 - "api_boundary_test.rs"
Cohesion: 0.06
Nodes (3): get_cover(), test_cover_missing_and_invalid_names_are_404(), test_cover_roundtrip_through_storage()

### Community 37 - "prelude"
Cohesion: 0.09
Nodes (9): current(), HOW_IT_WORKS_ID, ACCOUNT_ID, current(), TabBar(), ERROR_MS, INFO_MS, gallery_body() (+1 more)

### Community 38 - "security_owasp_test.rs"
Cohesion: 0.23
Nodes (11): test_owasp_login_brute_force_lockout(), test_owasp_otp_cooldown_and_email_bombing_prevention(), test_owasp_rate_limiting_headers_and_ip_extraction(), test_owasp_revoke_all_sessions(), test_owasp_security_headers_and_error_handling(), test_owasp_structured_validation_error_details(), test_owasp_timing_attack_mitigation_on_login(), test_owasp_token_revocation_on_logout() (+3 more)

### Community 39 - "routes/progress.rs"
Cohesion: 0.26
Nodes (4): get_active_progress(), merge_guest_progress(), progress_routes(), update_progress()

### Community 40 - "Fh"
Cohesion: 0.31
Nodes (13): Ah(), ch(), Dh(), Eh(), Fh(), lh(), Nh(), S() (+5 more)

### Community 41 - "Toasts"
Cohesion: 0.24
Nodes (3): Toast, Toasts, ToastStack()

### Community 43 - "AppState"
Cohesion: 0.24
Nodes (7): AppState, books_routes(), get_book(), get_chapter(), get_offline_bundle(), list_books(), search_books()

### Community 44 - "queue.rs"
Cohesion: 0.12
Nodes (12): dlq_entry(), get_job_status(), INGESTION_DLQ_STREAM, INGESTION_GROUP, INGESTION_STREAM, job_key(), JOB_KEY_PREFIX, JOB_TTL_SECS (+4 more)

### Community 45 - "n"
Cohesion: 0.10
Nodes (55): b(), bi(), bp(), c(), Ci(), i(), eb(), r() (+47 more)

### Community 46 - "test_worker_live.py"
Cohesion: 0.15
Nodes (4): DlqTest, main(), psql(), purge_seed_survivors()

### Community 48 - "admin_test.rs"
Cohesion: 0.15
Nodes (27): admin_get(), admin_patch(), AdminContext, BOUNDARY, multipart_body(), seed_funnel(), seed_status_book(), seed_users() (+19 more)

### Community 49 - "m"
Cohesion: 0.20
Nodes (14): ao(), co(), m(), io(), ji(), jm(), lo(), oo() (+6 more)

### Community 50 - "reliability_test.rs"
Cohesion: 0.13
Nodes (3): BookCatalogPort, test_reliability_mock_repository_fault_injection(), test_signup_fails_closed_when_smtp_unreachable()

### Community 52 - "TestHarness"
Cohesion: 0.11
Nodes (3): CLIENT_IP_HEADER, Seeded, TestHarness

### Community 53 - "parse_epub"
Cohesion: 0.18
Nodes (7): 2026-09-28 — M7 Sequential Execution: Hardening + BL-10 + BL-11 + BL-12, make_epub(), ParseTest, parse_epub(), ParsedChapter, ParsedEpub, _safe_member()

### Community 54 - "book_card.rs"
Cohesion: 0.31
Nodes (5): BookCard(), Cover(), cover_tint(), COVER_TINTS, SkeletonCard()

### Community 55 - "test_sessions.py"
Cohesion: 0.27
Nodes (7): _count_local_quotes(), _mailpit_otp(), _post(), _put_local_quote(), test_authed_autosave_shelf_cleanup(), test_guest_profile_and_admin_guard(), test_guest_quote_merges_on_register()

### Community 56 - "Value"
Cohesion: 0.29
Nodes (3): ApiResponse<T>, AtomicCardsDto, ChapterRecapDto

### Community 57 - "test_worker.py"
Cohesion: 0.16
Nodes (5): ArchiveLimitsTest, ChunkTest, check_archive_limits(), chunk_words(), _opf_path()

### Community 58 - "components/progress.rs"
Cohesion: 0.38
Nodes (3): ProgressBar(), ProgressRing(), RING_RADIUS

### Community 59 - "shared"
Cohesion: 0.09
Nodes (6): AuthSheet(), friendly_error(), merge_local_quotes(), Mode, current_url(), resume_chapter()

### Community 62 - "test_axe.py"
Cohesion: 0.28
Nodes (9): _assert_clean(), _scan(), test_axe_admin(), test_axe_home(), test_axe_overview(), test_axe_paper_surface(), test_axe_profile_guest(), test_axe_quote_modal() (+1 more)

### Community 63 - "playwright_sync_api"
Cohesion: 0.23
Nodes (6): assert_golden(), test_visual_admin(), test_visual_home(), test_visual_overview(), test_visual_profile_guest(), test_visual_reader()

### Community 64 - "semantic_ai_test.rs"
Cohesion: 0.14
Nodes (20): one_hot(), seed_reader(), seed_test_context(), SeededAiContext, SeededReader, test_ai_rate_limiter_burst_enforcement(), test_atomic_cards_scoped_to_path_book(), test_atomic_insight_cards_cache_hit_and_miss() (+12 more)

### Community 65 - "rate_limit.rs"
Cohesion: 0.14
Nodes (19): ServerConfig, AI_POLICY, ai_rate_limit_middleware(), AUTH_POLICY, client_ip_from_headers(), enforce(), extract_client_ip(), FALLBACK_CLIENT_IP (+11 more)

### Community 67 - "RetryAndRecapTests"
Cohesion: 0.16
Nodes (3): RetryAndRecapTests, backoff_seconds(), build_recap_input()

### Community 68 - "email.rs"
Cohesion: 0.16
Nodes (9): build_transport(), mailpit_accepts_cleartext_delivery_when_running(), mask_email(), otp_message(), otp_message_addresses_sender_and_recipient(), send_otp_email(), smtp_error(), SMTP_TIMEOUT (+1 more)

### Community 69 - "Model"
Cohesion: 0.18
Nodes (3): Entity, Model, Relation

### Community 70 - "test_components.py"
Cohesion: 0.31
Nodes (7): _story(), test_atomic_cards_404_tolerant(), test_auth_sheet_modes(), test_curator_desk_tabs_rows_replay_and_funnel(), test_quote_finder_empty_state(), test_quote_finder_error_branch(), test_site_header_toggle_no_reload()

### Community 75 - "Changelog"
Cohesion: 0.44
Nodes (5): Architecture Blueprint & Technical Design, Asset Catalog, Changelog, Distributed Processing & Ingestion Pipeline, Developer & Operations Guide

### Community 76 - "externalize_inline_scripts.py"
Cohesion: 0.20
Nodes (7): 2026-09-27 — Milestone 05 Complete: Ingestion Pipeline & Admin (ADR-19), 2026-09-27 — Milestone 05 Complete: Ingestion Pipeline & Admin (ADR-19), check(), externalize(), replace(), _is_inline(), main()

### Community 78 - "components/insights.rs"
Cohesion: 0.24
Nodes (9): AtomicCards(), CatchupRecap(), InsightList(), InsightText(), QuoteFinder(), share_quote(), Sheet(), string_field() (+1 more)

### Community 79 - "Log — Project Baca Knowledge Bundle"
Cohesion: 0.04
Nodes (47): 2026-09-25 — Core Engineering Invariants & Zero Emoji Policy, 2026-09-25 — Dual-Mode Embeddings, Triton Elimination & Private Gitignore, 2026-09-25 — OKF v0.2 Knowledge Vault & Graphify Knowledge Graph, 2026-09-26 — Auth & User Management Performance Optimization, Query Architecture & Vulnerability Test Suite, 2026-09-26 — Cargo Workspace Initialization & 5-Crate Skeleton, 2026-09-26 — Database Performance Optimization: Index Pruning, Foreign Key Coverage & Catalog Zero-Sort, 2026-09-26 — Edge Transport Architecture: Caddy HTTP/3 (QUIC) Reverse Proxy & Auto-HTTPS, 2026-09-26 — Graphify Knowledge Graph & Obsidian Vault Synchronization (+39 more)

### Community 80 - "Chronological Decision Records"
Cohesion: 0.05
Nodes (38): 2026-09-25 — 6 Core Engineering Pillars & Zero Emoji Policy, 2026-09-25 — Data Access: SeaORM & Paired SQL Migrations, 2026-09-25 — Database Selection: "Postgres for Everything" vs Elasticsearch, 2026-09-25 — Eliminated Triton Server & Adopted 768-Dim Dual-Mode Embeddings, 2026-09-25 — OKF v0.2 Implementation & Obsidian Integration, 2026-09-25 — Queue Architecture: Redis Streams vs RedPanda / RabbitMQ, 2026-09-25 — Visual Identity: Vintage Literary (1900–1950), 2026-09-26 — Adoption of Caddy Reverse Proxy for HTTP/3 (QUIC) Edge Termination with Automatic HTTP/2 Fallback (ADR-17) (+30 more)

### Community 83 - "sea_orm"
Cohesion: 0.11
Nodes (5): init_db_pool(), init_redis_client(), redacted_endpoint(), main(), shutdown_signal()

### Community 84 - "use_lang"
Cohesion: 0.09
Nodes (27): JobStatusDto, SiteHeader(), use_session(), ShellLayout(), use_toasts(), use_lang(), AdminPage(), CuratorDesk() (+19 more)

### Community 85 - "test_responsive.py"
Cohesion: 0.46
Nodes (4): _page_at(), test_home_and_book_fit_viewport(), test_reader_paginates_long_chapter(), _visible()

### Community 88 - "Ingestion Worker (`python_worker/`)"
Cohesion: 0.33
Nodes (5): Environment, Ingestion Worker (`python_worker/`), Notes, Setup and run, Tests

### Community 96 - "progress_repository.rs"
Cohesion: 0.19
Nodes (9): Model, Relation, check_and_award_badges(), get_active_progress(), get_streak(), record_heartbeat(), seconds_read_on(), update_progress() (+1 more)

### Community 99 - "Log — Project Baca Knowledge Bundle"
Cohesion: 0.06
Nodes (31): 2026-09-25 — Core Engineering Invariants & Zero Emoji Policy, 2026-09-25 — Dual-Mode Embeddings, Triton Elimination & Private Gitignore, 2026-09-25 — OKF v0.2 Knowledge Vault & Graphify Knowledge Graph, 2026-09-26 — Auth & User Management Performance Optimization, Query Architecture & Vulnerability Test Suite, 2026-09-26 — Cargo Workspace Initialization & 5-Crate Skeleton, 2026-09-26 — Database Performance Optimization: Index Pruning, Foreign Key Coverage & Catalog Zero-Sort, 2026-09-26 — Edge Transport Architecture: Caddy HTTP/3 (QUIC) Reverse Proxy & Auto-HTTPS, 2026-09-26 — Graphify Knowledge Graph & Obsidian Vault Synchronization (+23 more)

### Community 100 - "test_reader_flip.py"
Cohesion: 0.42
Nodes (5): _drag(), _open_long_chapter(), test_drag_turns_like_paper(), test_keyboard_turns_and_dims(), test_reduced_motion_turns_without_paper()

### Community 101 - "books"
Cohesion: 0.09
Nodes (26): q(), books(), 1. Module 1: Catalog & Discovery, 5. Module 5: Gamification & Retention, 6. Module 6: Ingestion Pipeline & Admin, 7. Module 7: User Interface Internationalization (i18n), 8. Requirements Traceability Matrix (RTM), FR-ADM-01: EPUB Upload & Validation (+18 more)

### Community 102 - "Manual Testing — Milestone 03: Catalog, Reader Engine & Gamification"
Cohesion: 0.10
Nodes (21): 1. Overview, 2. Quick Data Seeder for Manual Testing, 2. Web Client Verification (M6, `http://127.0.0.1:3000`), 3. Catalog Discovery & Typo-Tolerant Search, 4. Book Overview, Reader Content & Offline Bundle, 5. Reading Progress, CFI Anchors & Active Position, 6. Gamification: Heartbeats, Streaks & Badges, Manual Testing — Milestone 03: Catalog, Reader Engine & Gamification (+13 more)

### Community 105 - "Manual Testing — Milestone 07: QA Gates, M7 Tracks & Launch Readiness"
Cohesion: 0.11
Nodes (19): 1. Overview, 2. Part A — QA Gates, 3. Part B — M7 Session Track (BL-11), 4. Part C — M7 PWA Track (BL-12), 5. Part D — Remaining Launch Checklist (Not Done, Do Not Fake), 6. Regression Checklist (Copy-Paste per Pass), Manual Testing — Milestone 07: QA Gates, M7 Tracks & Launch Readiness, Step 7.10: Service Worker Caches Assets, Never `/api/*` (+11 more)

### Community 108 - "Vec"
Cohesion: 0.20
Nodes (4): EmbeddingProvider, FastEmbedProvider, MockEmbeddingProvider, test_mock_provider()

### Community 109 - "Manual Testing — Milestone 06: Leptos Web Reader (Rotaria P1–P7)"
Cohesion: 0.11
Nodes (18): 1. Overview, 2. Home: Brand, i18n, Hero, Catalog, 3. Book Overview: Save Offline Side Effects, 5. Insights: Quote Finder, Atomic Cards, Recap, PNG Export, 6. Auth, Guest Merge, Profile, Admin, 7. Regression Checklist (Copy-Paste per Pass), Manual Testing — Milestone 06: Leptos Web Reader (Rotaria P1–P7), Step 6.11: Quote Drawer States (+10 more)

### Community 110 - "3. REST API Endpoint Contracts"
Cohesion: 0.11
Nodes (18): 1. Security Architecture, 2. API Response & Error Standards, 3. REST API Endpoint Contracts, 4. Non-Functional Requirements (NFR), A. Health, B. Authentication and account, Credentials and tokens, D. Insights and quotes (+10 more)

### Community 111 - "2. Step-by-Step Test Procedure"
Cohesion: 0.12
Nodes (16): 1. Overview, 2. Step-by-Step Test Procedure, Manual Testing — Milestone 02: Authentication, Security & Guest Progress, Step 2.10: Logout (Revoke Session & Blacklist Access Token), Step 2.11: Revoke All Sessions (`POST /api/v1/auth/revoke-all`), Step 2.12: Throttling, Cooldown & Brute-Force Lockout, Step 2.1: Register a New User Account, Step 2.2: Retrieve OTP from Mailpit (+8 more)

### Community 112 - "Manual Testing — Milestone 04: Semantic AI, Quotes & Insights"
Cohesion: 0.12
Nodes (16): 1. Overview, 2. Scoped Quote Search (guest-open), 3. Saved Quotes & Vintage Cards (Authenticated, Owner-Scoped), 4. Atomic Cards & Spoiler-Free Recaps, 5. Honest Expectations for Stub Content, Manual Testing — Milestone 04: Semantic AI, Quotes & Insights, Step 4.10: Recap Unavailable for Chapter 1 (Spoiler-Free Rule), Step 4.1: Successful Guest Search (`POST /api/v1/books/{book_id}/quotes/search`) (+8 more)

### Community 113 - "Manual Testing — Milestone 05: Ingestion Pipeline & Admin"
Cohesion: 0.12
Nodes (15): 1. Overview, 2. Admin EPUB Upload, 3. Job Monitoring & Drop-Off Analytics, 4. Python Worker Pipeline, 5. End-to-End Checklist (M4+M5 Together), Manual Testing — Milestone 05: Ingestion Pipeline & Admin, Step 5.1: RBAC — Reader and Guest Are Rejected, Step 5.2: Validation — Type, Presence, Size, Metadata (+7 more)

### Community 114 - "PRD — Project Baca (MVP: Reader + AI Insights)"
Cohesion: 0.12
Nodes (16): 10. Engineering Invariants, 1. Product Vision, 2. Target Users, 3. Content Strategy & Public Domain Licensing, 4. User Roles & Access Model, 5. Key Features & User Stories, 6. Visual Design Identity (Vintage Literary 1900–1950), 7. Success Metrics (+8 more)

### Community 115 - "user_repository.rs"
Cohesion: 0.37
Nodes (8): activate_user_by_email(), create_inactive_user(), delete_user_by_id(), find_user_by_email(), find_user_by_id(), update_inactive_credentials(), update_user_password(), update_user_profile()

### Community 116 - "migrate.sh"
Cohesion: 0.76
Nodes (6): init_table(), migrate_down(), migrate_status(), migrate_up(), psql_cmd(), migrate.sh script

### Community 117 - "3. Screen Wireframes"
Cohesion: 0.13
Nodes (15): 1. Core UX Philosophy, 2. User Journey Flowchart, 3. Screen Wireframes, 4. State Transitions & Offline Synchronization, 5. Leptos WASM Component Mapping (`crates/web`), Guest to Account Reconciliation (Auto-Merge), Offline Mode, Screen 1: Home & Library Catalog (`/`) (+7 more)

### Community 118 - "catalog_test.rs"
Cohesion: 0.26
Nodes (10): seed_book_with_chapter(), seed_test_catalog(), SeededCatalog, test_book_overview_chapter_and_offline_bundle(), test_catalog_listing_and_filtering(), test_catalog_typo_tolerant_fts_search(), test_gamification_heartbeat_and_badges(), test_heartbeat_credit_clamped_and_gate_per_user() (+2 more)

### Community 122 - "test_reader_flow.py"
Cohesion: 0.52
Nodes (4): _chapter_no(), test_end_of_book_notice_is_visible(), test_offline_save_read_badge(), test_reader_body_and_type_sheet()

### Community 123 - "2. Prerequisites & Service Status Verification"
Cohesion: 0.14
Nodes (13): 1. Overview, 2. Prerequisites & Service Status Verification, 3. Server Health & OpenAPI Documentation, 4. Structured Log Inspection, Manual Testing — Milestone 01: Infrastructure & Configuration, Step 1.1: Verify Docker Containers, Step 1.2: Check PostgreSQL Schema & Extensions, Step 1.3: Check Redis Connectivity (+5 more)

### Community 124 - "manual-test/README.md"
Cohesion: 0.18
Nodes (8): 1. Directory Structure & Milestone Modules, 2. Server & Service URLs Quick Reference, 3. General Testing Workflow, Project Baca — Manual Testing Documentation, Audit Web Menyeluruh — 2026-09-27 (pra-M7), Kontrak yang legitimate (bukan bug), Screenshot, Temuan & Perbaikan (8/8 fixed, 0 page-error)

### Community 125 - "Audit menyeluruh Rotaria, 8 Oktober 2026"
Cohesion: 0.17
Nodes (10): Audit menyeluruh Rotaria, 8 Oktober 2026, Bukti pengujian yang dijalankan, Rencana kerja yang diusulkan (menunggu persetujuan), Audit ronde 2, 2026-10-08, Celah backend (task 26 sampai 29, tambahan task 21), Celah frontend (task 30 sampai 34), Dokumen, Hasil test (+2 more)

### Community 126 - "3. Data Dictionary & Table Specifications"
Cohesion: 0.17
Nodes (12): 1. Engineering Invariants & Data Philosophy, 2. Entity Relationship Diagram, 3. Data Dictionary & Table Specifications, 4. Index Matrix & Query Patterns, 5. Migration History (`migrations/`), A. Identity & Auth Cluster, B. Catalog & Taxonomy Cluster, C. Chapters & Semantic Vectors Cluster (+4 more)

### Community 127 - "generate_json"
Cohesion: 0.20
Nodes (8): 1. Findings (audit 2026-10-08, round 2), 28 — Worker follow-ups, 2. Steps, 3. Verify, embed_batch(), _gemini_post(), generate_json(), _strip_fences()

### Community 130 - "tasks/README.md"
Cohesion: 0.25
Nodes (6): 1. Goal, 23 — Reader Enhancements, 2. Verify, Board, Order, Task Board

### Community 131 - "2. Module 2: Reflowable Reader & Offline Engine"
Cohesion: 0.29
Nodes (7): 2. Module 2: Reflowable Reader & Offline Engine, FR-RDR-01: Multi-Column Reflowable Text Rendering, FR-RDR-02: Tap-to-Turn Navigation, FR-RDR-03: DOM CFI Position Locking, FR-RDR-04: Typography Controls, FR-RDR-05: Offline Storage via IndexedDB, FR-RDR-06: Connectivity Detection & Offline Fallback

### Community 132 - "AGENTS.md — AI Agent Guidelines & Governance"
Cohesion: 0.33
Nodes (6): 1. Single Source of Truth (SSOT), 2. Non-Negotiable Core Invariants, 3. Engineering Pillars, 4. Audit Trail Protocol, 5. Zero Emoji Policy & Link Integrity, AGENTS.md — AI Agent Guidelines & Governance

### Community 133 - "CLAUDE.md — Claude Code Guidelines for Project Baca"
Cohesion: 0.33
Nodes (6): 1. Primary Commands (Makefile Orchestration), 2. Monorepo Architecture Overview, 3. Strict Coding Invariants, 4. Working rules, 5. Documentation & Audit Trail Workflow, CLAUDE.md — Claude Code Guidelines for Project Baca

### Community 134 - "mo"
Cohesion: 0.33
Nodes (6): ancestry(), ho(), mo(), Ph(), Rp(), Y()

### Community 138 - "assert_no_page_errors"
Cohesion: 0.16
Nodes (16): assert_no_page_errors(), goto(), test_aria_home_structure(), test_aria_nav_landmarks(), test_hero_plate_is_static(), test_home_catalog_shows_two_rows_then_everything(), test_i18n_toggle_and_persist(), test_overview_and_sheets() (+8 more)

### Community 139 - "Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya."
Cohesion: 0.40
Nodes (4): Answer, Outcome, Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya., Source Nodes

### Community 140 - "Knowledge Catalog Index & Directory Architecture Map (OKF v0.2)"
Cohesion: 0.33
Nodes (5): 1. Directory Role & Responsibility Map, 2. Core Specification Documents, 3. Progressive Disclosure Hierarchy, 4. Engineering Invariants, Knowledge Catalog Index & Directory Architecture Map (OKF v0.2)

### Community 141 - "22 — Tauri Shell"
Cohesion: 0.33
Nodes (5): 1. Goal, 22 — Tauri Shell, 2. Preconditions from the web build, 3. Steps, 4. Verify

### Community 142 - "Project Baca (Rotaria)"
Cohesion: 0.33
Nodes (6): Content, Documentation, Project Baca (Rotaria), Quickstart, Routes, Stack

### Community 143 - "3. Module 3: Authentication, Users & Reconciliation"
Cohesion: 0.40
Nodes (5): 3. Module 3: Authentication, Users & Reconciliation, FR-USR-01: Guest Access (Local-First), FR-USR-02: Registration, Login & Email OTP, FR-USR-03: Guest-to-Cloud Auto-Merge, FR-USR-04: Role-Based Access Control (RBAC)

### Community 144 - "shared"
Cohesion: 0.70
Nodes (5): domain, infra, server, shared, web

### Community 145 - "4. Module 4: Semantic AI & Insights"
Cohesion: 0.40
Nodes (5): 4. Module 4: Semantic AI & Insights, FR-AI-01: Scoped Semantic Quote Finder, FR-AI-02: Vintage Quote Card Export, FR-AI-03: Spoiler-Free Catch-up Recap, FR-AI-04: Atomic Insight Cards

### Community 146 - "Knowledge Index"
Cohesion: 0.40
Nodes (5): 1. Directory map, 2. Documents, 3. Reading order, 4. Invariants, Knowledge Index

### Community 154 - "4. Reader: Zones, CFI Anchor, Typography, Heartbeat"
Cohesion: 0.40
Nodes (5): 4. Reader: Zones, CFI Anchor, Typography, Heartbeat, Step 6.10: Heartbeat + Streak Toast, Step 6.7: Tap Zones Turn Pages, Step 6.8: Anchor Survives Reload, Step 6.9: Typography Drawer Bounds

### Community 160 - "Brain content"
Cohesion: 0.40
Nodes (4): Brain content, Provenance, Safety, Skills

### Community 162 - "Brain content"
Cohesion: 0.40
Nodes (4): Brain content, Provenance, Safety, Skills

### Community 163 - "20 — Launch Remaining and Trademark"
Cohesion: 0.40
Nodes (4): 1. Goal, 20 — Launch Remaining and Trademark, 2. Steps, 3. Verify

### Community 164 - "21 — Rate-Limit Follow-ups"
Cohesion: 0.40
Nodes (4): 1. Findings (audit 2026-10-07), 21 — Rate-Limit Follow-ups, 2. Steps, 3. Verify

### Community 165 - "24 — Responsive Follow-ups"
Cohesion: 0.40
Nodes (4): 1. Findings (viewport matrix, 2026-10-08), 24 — Responsive Follow-ups, 2. Steps, 3. Verify

### Community 180 - "26 — Redis reconnect, boot fail-fast, outbound timeouts"
Cohesion: 0.40
Nodes (4): 1. Findings (audit 2026-10-08, round 2), 26 — Redis reconnect, boot fail-fast, outbound timeouts, 2. Steps, 3. Verify

### Community 183 - "27 — Auth follow-ups"
Cohesion: 0.40
Nodes (4): 1. Findings (audit 2026-10-08, round 2), 27 — Auth follow-ups, 2. Steps, 3. Verify

### Community 189 - "29 — Edge and compose hygiene"
Cohesion: 0.40
Nodes (4): 1. Findings, 29 — Edge and compose hygiene, 2. Steps, 3. Verify

### Community 191 - "30 — Slim the WASM bundle"
Cohesion: 0.40
Nodes (4): 1. Measurement (2026-10-08, `trunk build --release`), 2. Steps, 30 — Slim the WASM bundle, 3. Verify

### Community 192 - "31 — Reader position integrity"
Cohesion: 0.40
Nodes (4): 1. Findings (audit 2026-10-08, round 2), 2. Steps, 31 — Reader position integrity, 3. Verify

### Community 203 - "32 — Session and API client follow-ups"
Cohesion: 0.40
Nodes (4): 1. Findings (audit 2026-10-08, round 2), 2. Steps, 32 — Session and API client follow-ups, 3. Verify

### Community 206 - "33 — Shell and accessibility polish"
Cohesion: 0.40
Nodes (4): 1. Findings (audit 2026-10-08, round 2), 2. Steps, 33 — Shell and accessibility polish, 3. Verify

### Community 207 - "34 — Service worker and fonts"
Cohesion: 0.40
Nodes (4): 1. Findings (audit 2026-10-08, round 2), 2. Steps, 34 — Service worker and fonts, 3. Verify

### Community 208 - "Project Baca — Operational Status & Backlog Log"
Cohesion: 0.40
Nodes (5): 1. Current State, 2. Shipped Milestones (Snapshot), 3. Open Work (Task Board 20–34), 4. Architecture Decision Records, Project Baca — Operational Status & Backlog Log

### Community 209 - "GEMINI.md — Gemini & Google Antigravity Guidelines"
Cohesion: 0.50
Nodes (4): 1. Navigation & Source of Truth (SSOT), 2. Non-Negotiable Engineering Invariants, 3. Engineering Discipline & Zero Emoji Policy, GEMINI.md — Gemini & Google Antigravity Guidelines

## Knowledge Gaps
- **494 isolated node(s):** `COMPLETION_MIN`, `COMPLETION_MAX`, `DAILY_THRESHOLD_SECONDS`, `BASE_HEARTBEAT_XP`, `STREAK_BONUS_XP` (+489 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 1102 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **90 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Log — Project Baca Knowledge Bundle` connect `Log — Project Baca Knowledge Bundle` to `index.md`, `parse_epub`, `externalize_inline_scripts.py`, `books`?**
  _High betweenness centrality (0.211) - this node is a cross-community bridge._
- **Why does `_safe_member()` connect `parse_epub` to `worker.py`?**
  _High betweenness centrality (0.185) - this node is a cross-community bridge._
- **Why does `2026-09-28 — M7 Sequential Execution: Hardening + BL-10 + BL-11 + BL-12` connect `parse_epub` to `Log — Project Baca Knowledge Bundle`?**
  _High betweenness centrality (0.184) - this node is a cross-community bridge._
- **Are the 23 inferred relationships involving `n()` (e.g. with `axe.min.js` and `A()`) actually correct?**
  _`n()` has 23 INFERRED edges - model-reasoned connections that need verification._
- **Are the 2 inferred relationships involving `HttpError` (e.g. with `.from_request_parts()` and `enforce()`) actually correct?**
  _`HttpError` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `COMPLETION_MIN`, `COMPLETION_MAX`, `DAILY_THRESHOLD_SECONDS` to the rest of the system?**
  _494 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `shared/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.11529411764705882 - nodes in this community are weakly interconnected._