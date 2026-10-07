# Graph Report - project-baca-local-github-f0cac0  (2026-10-08)

## Corpus Check
- 149 files · ~310,677 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 17 file(s) not represented in the graph (top: (none) 5, .example 2, .prod 1)

## Summary
- 2178 nodes · 5101 edges · 168 communities (77 shown, 91 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 347 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `478305d4`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- shared/src/lib.rs
- axe.min.js
- n
- Lang
- domain/src/lib.rs
- routes/admin.rs
- quotes.rs
- e
- books
- routes/auth.rs
- otp.rs
- AppError
- AppConfig
- T
- reader.rs
- te
- AppState
- api/mod.rs
- serde
- theme.rs
- gamification.rs
- conftest.py
- worker.py
- web/src/storage/mod.rs
- String
- get_atomic_cards
- Vec
- quote_repository.rs
- auth_test.rs
- embedding.rs
- icons.rs
- home.rs
- backfill_chunks.rs
- post
- extract_text
- reliability_test.rs
- prelude
- security_owasp_test.rs
- routes/progress.rs
- Fh
- Toasts
- AuthUser
- HttpError
- queue.rs
- b
- test_worker_live.py
- admin_test.rs
- m
- catalog_test.rs
- Session
- TestHarness
- parse_epub
- book_card.rs
- test_sessions.py
- .error
- test_worker.py
- components/progress.rs
- components/auth.rs
- web/src/lib.rs
- chapters.rs
- test_axe.py
- playwright_sync_api
- semantic_ai_test.rs
- AiConfig
- RetryAndRecapTests
- email.rs
- Model
- assert_no_page_errors
- Project Baca
- externalize_inline_scripts.py
- components/insights.rs
- gi
- Asset Catalog
- sea_orm
- use_lang
- test_responsive.py
- Ingestion Worker (`python_worker/`)
- Model
- shared
- request_id_middleware
- Vec
- ReadingProgressUpdateDto
- test_aria.py
- validate_avatar_url
- migrate.sh
- MockEmbeddingProvider
- Entity
- Model
- user_badges.rs
- test_reader_flow.py
- rate_limit.rs
- Model
- Model
- users.rs
- goto
- Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya.
- shared
- Entity
- Entity
- Entity
- Entity
- Entity
- reading_activity_logs.rs
- Entity
- Entity
- Entity
- Entity
- Entity
- Entity
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
- pg_backup.sh
- Admin Sorting Illustration
- Reader View Illustration
- Library Catalog Illustration
- Ingestion Monitor Illustration
- AI Quote Finder Illustration
- GBrain Design Reference
- Production Compose Configuration

## God Nodes (most connected - your core abstractions)
1. `AppError` - 111 edges
2. `AppState` - 72 edges
3. `n()` - 62 edges
4. `HttpError` - 49 edges
5. `AuthUser` - 33 edges
6. `use_lang()` - 33 edges
7. `e()` - 31 edges
8. `A()` - 31 edges
9. `TestHarness` - 28 edges
10. `u()` - 28 edges

## Surprising Connections (you probably didn't know these)
- `Web Entry Point` --references--> `Rotaria Windmill Icon`  [EXTRACTED]
  crates/web/index.html → assets/illustrations/rotaria-windmill.svg
- `test_signup_fails_closed_when_smtp_unreachable()` --calls--> `signup()`  [INFERRED]
  crates/server/tests/it/reliability_test.rs → crates/server/src/routes/auth.rs
- `Changelog` --references--> `Architecture Blueprint & Technical Design`  [EXTRACTED]
  CHANGELOG.md → ARCHITECTURE.md
- `Changelog` --references--> `Distributed Processing & Ingestion Pipeline`  [EXTRACTED]
  CHANGELOG.md → DISTRIBUTED.md
- `Changelog` --references--> `Developer & Operations Guide`  [EXTRACTED]
  CHANGELOG.md → GUIDE.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Infrastructure Services** — redis_7, minio [EXTRACTED 1.00]

## Communities (168 total, 91 thin omitted)

### Community 0 - "shared/src/lib.rs"
Cohesion: 0.12
Nodes (40): ActiveProgressDto, AdminBookPatchRequest, AdminBookRowDto, ApiResponse, AtomicCardsDto, BadgeDto, BookCatalogQuery, BookDetailDto (+32 more)

### Community 1 - "axe.min.js"
Cohesion: 0.02
Nodes (44): ancestry(), be(), ce(), de(), df(), dg(), ei(), eu() (+36 more)

### Community 2 - "n"
Cohesion: 0.08
Nodes (62): A(), Am(), As(), bg(), Bm(), bs(), c(), cg() (+54 more)

### Community 3 - "Lang"
Cohesion: 0.12
Nodes (9): apply_lang(), ENTRIES, Lang, LANG_KEY, lang_signal(), LangContext, provide_lang(), badge_description() (+1 more)

### Community 4 - "domain/src/lib.rs"
Cohesion: 0.06
Nodes (24): advance_streak(), BASE_HEARTBEAT_XP, BookStatus, COMPLETION_MAX, COMPLETION_MIN, DAILY_MILESTONE_BONUS_XP, DAILY_THRESHOLD_SECONDS, day() (+16 more)

### Community 5 - "routes/admin.rs"
Cohesion: 0.17
Nodes (13): require_admin(), admin_routes(), AdminCatalogQuery, DlqQuery, dropoff_analytics(), DropoffQuery, ingestion_status(), list_catalog() (+5 more)

### Community 6 - "quotes.rs"
Cohesion: 0.12
Nodes (25): AdminBookRow, chapter_dropoff(), get_book_by_id(), get_book_status(), get_chapter_book_id(), get_chapter_by_number(), get_chapter_number(), get_offline_bundle() (+17 more)

### Community 7 - "e"
Cohesion: 0.13
Nodes (35): bh(), bl(), cf(), cp(), D(), dl(), dp(), e() (+27 more)

### Community 8 - "books"
Cohesion: 0.07
Nodes (43): idx_users_admin_role, idx_users_email, users, book_tags, books, idx_book_tags_reverse, idx_books_catalog_filter, idx_books_published_author_trgm (+35 more)

### Community 9 - "routes/auth.rs"
Cohesion: 0.18
Nodes (23): client_ip_from_headers(), auth_routes(), change_password(), delete_me(), get_me(), INVALID_CREDENTIALS, issue_token_pair(), list_my_sessions() (+15 more)

### Community 10 - "otp.rs"
Cohesion: 0.12
Nodes (19): generate_and_store_otp(), generate_numeric_otp(), hash_otp(), MAX_OTP_ATTEMPTS, OTP_LOCK_SECONDS, OTP_TTL_SECONDS, OtpRecord, test_hash_otp_consistency() (+11 more)

### Community 12 - "AppError"
Cohesion: 0.07
Nodes (53): check_and_award_badges(), get_active_progress(), get_streak(), record_heartbeat(), seconds_read_on(), update_progress(), activate_user_by_email(), create_inactive_user() (+45 more)

### Community 13 - "AppConfig"
Cohesion: 0.13
Nodes (10): AppConfig, AuthConfig, DatabaseConfig, EmailConfig, RedisConfig, ServerConfig, StorageConfig, test_app_config_from_env_clean_execution() (+2 more)

### Community 14 - "T"
Cohesion: 0.11
Nodes (31): a1(), af(), bc(), cc(), dc(), fc(), fe(), g1() (+23 more)

### Community 15 - "reader.rs"
Cohesion: 0.08
Nodes (17): reading_minutes(), roman(), WORDS_PER_MINUTE, ANCHOR_END, anchor_index(), cached_book(), cached_chapter(), HEARTBEAT_INTERVAL_SECS (+9 more)

### Community 16 - "te"
Cohesion: 0.09
Nodes (29): ae(), Ap(), bf(), r(), Ef(), ep(), f0(), ge() (+21 more)

### Community 17 - "AppState"
Cohesion: 0.12
Nodes (9): api_health_check(), ApiDoc, AppState, create_app(), health_check(), not_found_handler(), REQUEST_ID_HEADER, SecurityAddon (+1 more)

### Community 18 - "api/mod.rs"
Cohesion: 0.09
Nodes (22): TokenResponse, ADMIN_PAGE_SIZE, as_html_element(), change_password(), clear_token(), encode_param(), is_authed(), login() (+14 more)

### Community 19 - "serde"
Cohesion: 0.12
Nodes (6): Model, Relation, Model, Relation, Model, Relation

### Community 20 - "theme.rs"
Cohesion: 0.10
Nodes (12): FONT_SIZE_MAX, FONT_SIZE_MIN, LEADING_PRESETS, LINE_HEIGHT_MAX, LINE_HEIGHT_MIN, READER_KEY, ReaderPrefs, ReadingTheme (+4 more)

### Community 21 - "gamification.rs"
Cohesion: 0.37
Nodes (6): gamification_routes(), get_my_streak(), HEARTBEAT_MIN_INTERVAL_SECS, list_badges(), list_user_badges(), record_heartbeat()

### Community 22 - "conftest.py"
Cohesion: 0.13
Nodes (9): api_get(), book_with_chapters(), clean_page(), stable_book_id(), call(), test_envelope_and_health(), test_merge_cap_boundary(), test_save_quote_requires_auth() (+1 more)

### Community 23 - "worker.py"
Cohesion: 0.13
Nodes (14): convert_cover_to_webp(), embed_batch(), ensure_group(), _gemini_post(), generate_json(), main(), make_db(), make_minio() (+6 more)

### Community 24 - "web/src/storage/mod.rs"
Cohesion: 0.08
Nodes (16): clear(), db(), DB_NAME, DB_VERSION, delete(), get_all(), local_quote_key(), LocalQuoteRecord (+8 more)

### Community 25 - "String"
Cohesion: 0.24
Nodes (21): active_progress(), admin_books(), admin_dlq(), admin_dropoff(), admin_job(), admin_set_status(), all_badges(), atomic_cards() (+13 more)

### Community 26 - "get_atomic_cards"
Cohesion: 0.31
Nodes (4): get_atomic_cards(), get_chapter_recap(), insights_routes(), resolve_chapter()

### Community 27 - "Vec"
Cohesion: 0.19
Nodes (3): EmbeddingProvider, FastEmbedProvider, GeminiEmbeddingProvider

### Community 28 - "quote_repository.rs"
Cohesion: 0.25
Nodes (14): BatchQuoteOutcome, ChunkSearchRow, find_saved_quote_duplicate(), get_chapter_recap(), get_saved_quote_by_id(), get_tldr_cache(), insert_quote(), list_saved_quotes() (+6 more)

### Community 29 - "auth_test.rs"
Cohesion: 0.14
Nodes (13): login_req(), provision_verified_user(), refresh_req(), test_auth_concurrent_refresh_keeps_family_alive(), test_auth_email_aggregate_lockout_twenty_failures(), test_auth_pair_lockout_five_failures(), test_auth_password_change_kills_other_session_e2e(), test_auth_refresh_grace_window() (+5 more)

### Community 30 - "embedding.rs"
Cohesion: 0.16
Nodes (14): build_embedding_provider(), GeminiBatchEmbedRequest, GeminiBatchEmbedResponse, GeminiContent, GeminiEmbedding, GeminiEmbedRequest, GeminiEmbedResponse, GeminiPart (+6 more)

### Community 31 - "icons.rs"
Cohesion: 0.26
Nodes (20): arrow_right(), back(), cards(), check(), close(), download(), flame(), home() (+12 more)

### Community 32 - "home.rs"
Cohesion: 0.12
Nodes (13): HomePage(), LanguageFilter, latest_guest_resume(), PAGE_SIZE, PLATE_SRC, Quick, RAIL_SIZE, RailCard() (+5 more)

### Community 33 - "backfill_chunks.rs"
Cohesion: 0.13
Nodes (7): BATCH_SIZE, chunk_text(), CHUNK_WORDS, main(), MIN_CHARS, strip_html(), Model

### Community 34 - "post"
Cohesion: 0.20
Nodes (12): admin_replay(), authed(), authed_send(), heartbeat(), parse_envelope(), patch(), post(), put() (+4 more)

### Community 35 - "extract_text"
Cohesion: 0.12
Nodes (5): SanitizeTest, extract_text(), __init__(), sanitize_html(), _Sanitizer

### Community 36 - "reliability_test.rs"
Cohesion: 0.05
Nodes (3): BookCatalogPort, test_reliability_mock_repository_fault_injection(), test_signup_fails_closed_when_smtp_unreachable()

### Community 37 - "prelude"
Cohesion: 0.07
Nodes (6): Relation, TabBar(), current_url(), resume_chapter(), gallery_body(), GalleryPage()

### Community 38 - "security_owasp_test.rs"
Cohesion: 0.24
Nodes (10): test_owasp_login_brute_force_lockout(), test_owasp_otp_cooldown_and_email_bombing_prevention(), test_owasp_rate_limiting_headers_and_ip_extraction(), test_owasp_revoke_all_sessions(), test_owasp_security_headers_and_error_handling(), test_owasp_structured_validation_error_details(), test_owasp_timing_attack_mitigation_on_login(), test_owasp_token_revocation_on_logout() (+2 more)

### Community 39 - "routes/progress.rs"
Cohesion: 0.23
Nodes (4): get_active_progress(), merge_guest_progress(), progress_routes(), update_progress()

### Community 40 - "Fh"
Cohesion: 0.23
Nodes (16): Ah(), ch(), Dh(), Eh(), Fh(), gh(), jg(), lg() (+8 more)

### Community 41 - "Toasts"
Cohesion: 0.24
Nodes (3): Toast, Toasts, ToastStack()

### Community 43 - "HttpError"
Cohesion: 0.17
Nodes (7): HttpError, books_routes(), get_book(), get_chapter(), get_offline_bundle(), list_books(), search_books()

### Community 44 - "queue.rs"
Cohesion: 0.12
Nodes (12): dlq_entry(), get_job_status(), INGESTION_DLQ_STREAM, INGESTION_GROUP, INGESTION_STREAM, job_key(), JOB_KEY_PREFIX, JOB_TTL_SECS (+4 more)

### Community 45 - "b"
Cohesion: 0.24
Nodes (15): _0(), b(), bp(), Ci(), eb(), Fp(), h(), Ih() (+7 more)

### Community 46 - "test_worker_live.py"
Cohesion: 0.15
Nodes (4): DlqTest, main(), psql(), purge_seed_survivors()

### Community 48 - "admin_test.rs"
Cohesion: 0.16
Nodes (26): admin_get(), admin_patch(), AdminContext, BOUNDARY, multipart_body(), seed_funnel(), seed_status_book(), seed_users() (+18 more)

### Community 49 - "m"
Cohesion: 0.20
Nodes (14): ao(), co(), m(), io(), ji(), jm(), lo(), oo() (+6 more)

### Community 50 - "catalog_test.rs"
Cohesion: 0.30
Nodes (8): seed_test_catalog(), SeededCatalog, test_book_overview_chapter_and_offline_bundle(), test_catalog_listing_and_filtering(), test_catalog_typo_tolerant_fts_search(), test_gamification_heartbeat_and_badges(), test_heartbeat_credit_clamped_and_gate_per_user(), test_reading_progress_and_active_retrieval()

### Community 53 - "parse_epub"
Cohesion: 0.20
Nodes (6): make_epub(), ParseTest, parse_epub(), ParsedChapter, ParsedEpub, _safe_member()

### Community 54 - "book_card.rs"
Cohesion: 0.31
Nodes (5): BookCard(), Cover(), cover_tint(), COVER_TINTS, SkeletonCard()

### Community 55 - "test_sessions.py"
Cohesion: 0.30
Nodes (7): _count_local_quotes(), _mailpit_otp(), _post(), _put_local_quote(), test_authed_autosave_shelf_cleanup(), test_guest_profile_and_admin_guard(), test_guest_quote_merges_on_register()

### Community 57 - "test_worker.py"
Cohesion: 0.19
Nodes (5): ArchiveLimitsTest, ChunkTest, check_archive_limits(), chunk_words(), _opf_path()

### Community 58 - "components/progress.rs"
Cohesion: 0.38
Nodes (3): ProgressBar(), ProgressRing(), RING_RADIUS

### Community 59 - "components/auth.rs"
Cohesion: 0.28
Nodes (4): AuthSheet(), friendly_error(), merge_local_quotes(), Mode

### Community 62 - "test_axe.py"
Cohesion: 0.44
Nodes (8): _assert_clean(), _scan(), test_axe_admin(), test_axe_home(), test_axe_overview(), test_axe_profile_guest(), test_axe_quote_modal(), test_axe_reader()

### Community 63 - "playwright_sync_api"
Cohesion: 0.23
Nodes (6): assert_golden(), test_visual_admin(), test_visual_home(), test_visual_overview(), test_visual_profile_guest(), test_visual_reader()

### Community 64 - "semantic_ai_test.rs"
Cohesion: 0.14
Nodes (19): one_hot(), seed_reader(), seed_test_context(), SeededAiContext, SeededReader, test_ai_rate_limiter_burst_enforcement(), test_atomic_cards_scoped_to_path_book(), test_atomic_insight_cards_cache_hit_and_miss() (+11 more)

### Community 67 - "RetryAndRecapTests"
Cohesion: 0.16
Nodes (3): RetryAndRecapTests, backoff_seconds(), build_recap_input()

### Community 69 - "Model"
Cohesion: 0.18
Nodes (3): Entity, Model, Relation

### Community 70 - "assert_no_page_errors"
Cohesion: 0.30
Nodes (8): _story(), test_atomic_cards_404_tolerant(), test_auth_sheet_modes(), test_curator_desk_tabs_rows_replay_and_funnel(), test_quote_finder_empty_state(), test_quote_finder_error_branch(), test_site_header_toggle_no_reload(), assert_no_page_errors()

### Community 75 - "Project Baca"
Cohesion: 0.19
Nodes (12): Architecture Blueprint & Technical Design, Changelog, Distributed Processing & Ingestion Pipeline, Developer & Operations Guide, 1. What is Project Baca?, 2. Why Project Baca? (The Problem We Solve), 3. Reader Experience & Key Features, 4. Architectural Highlights & Technology (+4 more)

### Community 76 - "externalize_inline_scripts.py"
Cohesion: 0.24
Nodes (5): check(), externalize(), replace(), _is_inline(), main()

### Community 78 - "components/insights.rs"
Cohesion: 0.24
Nodes (9): AtomicCards(), CatchupRecap(), InsightList(), InsightText(), QuoteFinder(), share_quote(), Sheet(), string_field() (+1 more)

### Community 79 - "gi"
Cohesion: 0.28
Nodes (9): bi(), gi(), ii(), Ko(), li(), mi(), oi(), Qo() (+1 more)

### Community 83 - "sea_orm"
Cohesion: 0.15
Nodes (4): init_db_pool(), init_redis_client(), main(), shutdown_signal()

### Community 84 - "use_lang"
Cohesion: 0.11
Nodes (26): JobStatusDto, SiteHeader(), use_session(), ShellLayout(), use_toasts(), use_lang(), AdminPage(), CuratorDesk() (+18 more)

### Community 85 - "test_responsive.py"
Cohesion: 0.46
Nodes (4): _page_at(), test_home_and_book_fit_viewport(), test_reader_paginates_long_chapter(), _visible()

### Community 88 - "Ingestion Worker (`python_worker/`)"
Cohesion: 0.25
Nodes (7): Environment, Ingestion Worker (`python_worker/`), Job lifecycle, Notes & deviations, Run, Setup, Unit tests (no services required)

### Community 98 - "shared"
Cohesion: 0.36
Nodes (3): list_badges(), list_user_badges(), seed_default_badges_if_empty()

### Community 100 - "Vec"
Cohesion: 0.40
Nodes (5): GuestMergeRequest, SaveQuotesBatchRequest, SaveQuotesBatchResponseDto, merge_guest_progress(), save_quotes_batch()

### Community 101 - "ReadingProgressUpdateDto"
Cohesion: 0.50
Nodes (5): ReadingProgressUpdateDto, drain_pending(), PendingProgress, queue_pending_progress(), save_progress()

### Community 116 - "migrate.sh"
Cohesion: 0.76
Nodes (6): init_table(), migrate_down(), migrate_status(), migrate_up(), psql_cmd(), migrate.sh script

### Community 122 - "test_reader_flow.py"
Cohesion: 0.53
Nodes (3): _chapter_no(), test_offline_save_read_badge(), test_reader_body_and_type_sheet()

### Community 134 - "rate_limit.rs"
Cohesion: 0.16
Nodes (14): AI_POLICY, ai_rate_limit_middleware(), AUTH_POLICY, enforce(), extract_client_ip(), public_policy(), public_policy_takes_cap_from_config(), public_rate_limit_middleware() (+6 more)

### Community 138 - "goto"
Cohesion: 0.26
Nodes (7): goto(), test_hero_plate_is_static(), test_i18n_toggle_and_persist(), test_overview_and_sheets(), test_search_returns_cards(), test_home_brand_and_hero(), test_home_renders_catalog()

### Community 139 - "Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya."
Cohesion: 0.40
Nodes (4): Answer, Outcome, Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya., Source Nodes

### Community 144 - "shared"
Cohesion: 0.70
Nodes (5): domain, infra, server, shared, web

## Knowledge Gaps
- **118 isolated node(s):** `COMPLETION_MIN`, `COMPLETION_MAX`, `DAILY_THRESHOLD_SECONDS`, `BASE_HEARTBEAT_XP`, `STREAK_BONUS_XP` (+113 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 672 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **91 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppError` connect `AppError` to `shared/src/lib.rs`, `routes/admin.rs`, `quotes.rs`, `otp.rs`, `AppConfig`, `AppState`, `get_atomic_cards`, `Vec`, `quote_repository.rs`, `embedding.rs`, `reliability_test.rs`, `AuthUser`, `HttpError`, `queue.rs`, `AiConfig`, `email.rs`, `sea_orm`, `shared`, `MockEmbeddingProvider`?**
  _High betweenness centrality (0.118) - this node is a cross-community bridge._
- **Why does `AppState` connect `AppState` to `routes/admin.rs`, `rate_limit.rs`, `routes/progress.rs`, `quotes.rs`, `routes/auth.rs`, `AuthUser`, `HttpError`, `AppError`, `AppConfig`, `TestHarness`, `gamification.rs`, `get_atomic_cards`, `Vec`?**
  _High betweenness centrality (0.046) - this node is a cross-community bridge._
- **Why does `TestHarness` connect `TestHarness` to `semantic_ai_test.rs`, `admin_test.rs`, `AppState`, `catalog_test.rs`, `sea_orm`, `auth_test.rs`?**
  _High betweenness centrality (0.033) - this node is a cross-community bridge._
- **Are the 23 inferred relationships involving `n()` (e.g. with `axe.min.js` and `A()`) actually correct?**
  _`n()` has 23 INFERRED edges - model-reasoned connections that need verification._
- **Are the 2 inferred relationships involving `HttpError` (e.g. with `.from_request_parts()` and `enforce()`) actually correct?**
  _`HttpError` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `COMPLETION_MIN`, `COMPLETION_MAX`, `DAILY_THRESHOLD_SECONDS` to the rest of the system?**
  _118 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `shared/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.11649659863945579 - nodes in this community are weakly interconnected._