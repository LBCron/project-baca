# Graph Report - project-baca-local-github-f0cac0  (2026-10-08)

## Corpus Check
- 157 files · ~323,299 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 17 file(s) not represented in the graph (top: (none) 5, .example 2, .prod 1)

## Summary
- 2387 nodes · 5538 edges · 170 communities (81 shown, 89 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 367 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `80aa09ee`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- shared/src/lib.rs
- axe.min.js
- n
- Lang
- domain/src/lib.rs
- AuthUser
- quotes.rs
- e
- books
- routes/auth.rs
- otp.rs
- database_test.rs
- AppError
- config.rs
- T
- reader/mod.rs
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
- HttpError
- queue.rs
- b
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
- assert_no_page_errors
- Project Baca
- externalize_inline_scripts.py
- components/insights.rs
- gi
- Asset Catalog
- main
- use_lang
- test_responsive.py
- Ingestion Worker (`python_worker/`)
- progress_repository.rs
- manage_sheet_focus
- request_id_middleware
- test_reader_flip.py
- test_smoke.py
- test_aria.py
- validate_avatar_url
- SmtpSecurity
- migrate.sh
- Entity
- Model
- user_badges.rs
- test_reader_flow.py
- enforce
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
1. `AppError` - 119 edges
2. `AppState` - 75 edges
3. `n()` - 62 edges
4. `HttpError` - 50 edges
5. `assert_no_page_errors()` - 34 edges
6. `AuthUser` - 33 edges
7. `use_lang()` - 33 edges
8. `e()` - 31 edges
9. `A()` - 31 edges
10. `TestHarness` - 29 edges

## Surprising Connections (you probably didn't know these)
- `Web Entry Point` --references--> `Rotaria Windmill Icon`  [EXTRACTED]
  crates/web/index.html → assets/illustrations/rotaria-windmill.svg
- `test_signup_fails_closed_when_smtp_unreachable()` --calls--> `signup()`  [INFERRED]
  crates/server/tests/it/reliability_test.rs → crates/server/src/routes/auth.rs
- `ReaderPage()` --calls--> `reading_minutes()`  [INFERRED]
  crates/web/src/pages/reader/mod.rs → crates/web/src/format.rs
- `Changelog` --references--> `Architecture Blueprint & Technical Design`  [EXTRACTED]
  CHANGELOG.md → ARCHITECTURE.md
- `Changelog` --references--> `Distributed Processing & Ingestion Pipeline`  [EXTRACTED]
  CHANGELOG.md → DISTRIBUTED.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Infrastructure Services** — redis_7, minio [EXTRACTED 1.00]

## Communities (170 total, 89 thin omitted)

### Community 0 - "shared/src/lib.rs"
Cohesion: 0.12
Nodes (42): ActiveProgressDto, AdminBookPatchRequest, AdminBookRowDto, ApiResponse, BadgeDto, BookCatalogQuery, BookDetailDto, BookSearchQuery (+34 more)

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

### Community 5 - "AuthUser"
Cohesion: 0.12
Nodes (15): AuthUser, forbidden(), require_admin(), admin_routes(), AdminCatalogQuery, DlqQuery, dropoff_analytics(), DropoffQuery (+7 more)

### Community 6 - "quotes.rs"
Cohesion: 0.22
Nodes (11): card_url_for(), get_quote_card(), handle_list_saved_quotes(), handle_save_quote(), handle_save_quotes_batch(), outcome(), quotes_routes(), render_vintage_quote_svg() (+3 more)

### Community 7 - "e"
Cohesion: 0.13
Nodes (35): bh(), bl(), cf(), cp(), D(), dl(), dp(), e() (+27 more)

### Community 8 - "books"
Cohesion: 0.07
Nodes (43): idx_users_admin_role, idx_users_email, users, book_tags, books, idx_book_tags_reverse, idx_books_catalog_filter, idx_books_published_author_trgm (+35 more)

### Community 9 - "routes/auth.rs"
Cohesion: 0.18
Nodes (22): auth_routes(), change_password(), delete_me(), get_me(), INVALID_CREDENTIALS, issue_token_pair(), list_my_sessions(), login() (+14 more)

### Community 10 - "otp.rs"
Cohesion: 0.05
Nodes (27): generate_and_store_otp(), generate_numeric_otp(), hash_otp(), MAX_OTP_ATTEMPTS, OTP_LOCK_SECONDS, OTP_TTL_SECONDS, OtpRecord, test_hash_otp_consistency() (+19 more)

### Community 11 - "database_test.rs"
Cohesion: 0.10
Nodes (4): redacted_endpoint(), list_badges(), list_user_badges(), seed_default_badges_if_empty()

### Community 12 - "AppError"
Cohesion: 0.06
Nodes (65): ADMIN_ROW_SELECT, AdminBookRow, AdminBookRowDto, chapter_dropoff(), ensure_book_published(), get_book_by_id(), get_book_for_admin(), get_book_status() (+57 more)

### Community 13 - "config.rs"
Cohesion: 0.12
Nodes (9): AppConfig, AuthConfig, DatabaseConfig, RedisConfig, ServerConfig, StorageConfig, test_app_config_from_env_clean_execution(), test_config_accessors() (+1 more)

### Community 14 - "T"
Cohesion: 0.11
Nodes (31): a1(), af(), bc(), cc(), dc(), fc(), fe(), g1() (+23 more)

### Community 15 - "reader/mod.rs"
Cohesion: 0.08
Nodes (22): anchor_index(), columns_for(), css_px(), Paddings, page_of(), paragraphs(), TWO_COLUMN_MIN_WIDTH, ANCHOR_END (+14 more)

### Community 16 - "te"
Cohesion: 0.09
Nodes (29): ae(), Ap(), bf(), r(), Ef(), ep(), f0(), ge() (+21 more)

### Community 17 - "AppState"
Cohesion: 0.12
Nodes (9): api_health_check(), ApiDoc, AppState, create_app(), health_check(), not_found_handler(), REQUEST_ID_HEADER, SecurityAddon (+1 more)

### Community 18 - "api/mod.rs"
Cohesion: 0.09
Nodes (19): ADMIN_PAGE_SIZE, as_html_element(), asset_url(), change_password(), clear_token(), encode_param(), is_authed(), logout() (+11 more)

### Community 19 - "serde"
Cohesion: 0.12
Nodes (6): Model, Relation, Model, Relation, Model, Relation

### Community 20 - "theme.rs"
Cohesion: 0.09
Nodes (13): FONT_SIZE_MAX, FONT_SIZE_MIN, LEADING_PRESETS, LINE_HEIGHT_MAX, LINE_HEIGHT_MIN, prefers_light(), READER_KEY, ReaderPrefs (+5 more)

### Community 21 - "gamification.rs"
Cohesion: 0.37
Nodes (6): gamification_routes(), get_my_streak(), HEARTBEAT_MIN_INTERVAL_SECS, list_badges(), list_user_badges(), record_heartbeat()

### Community 22 - "conftest.py"
Cohesion: 0.21
Nodes (5): api_get(), book_with_chapters(), browser_context_args(), clean_page(), stable_book_id()

### Community 23 - "worker.py"
Cohesion: 0.13
Nodes (14): convert_cover_to_webp(), embed_batch(), ensure_group(), _gemini_post(), generate_json(), main(), make_db(), make_minio() (+6 more)

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
Cohesion: 0.08
Nodes (22): API_KEY_HEADER, build_embedding_provider(), EmbeddingProvider, FastEmbedProvider, GeminiBatchEmbedRequest, GeminiBatchEmbedResponse, GeminiContent, GeminiEmbedding (+14 more)

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
Cohesion: 0.14
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
Cohesion: 0.08
Nodes (13): CLIENT_IP_HEADER, Seeded, TestHarness, seed_book_with_chapter(), seed_test_catalog(), SeededCatalog, test_book_overview_chapter_and_offline_bundle(), test_catalog_listing_and_filtering() (+5 more)

### Community 53 - "parse_epub"
Cohesion: 0.20
Nodes (6): make_epub(), ParseTest, parse_epub(), ParsedChapter, ParsedEpub, _safe_member()

### Community 54 - "book_card.rs"
Cohesion: 0.31
Nodes (5): BookCard(), Cover(), cover_tint(), COVER_TINTS, SkeletonCard()

### Community 55 - "test_sessions.py"
Cohesion: 0.16
Nodes (12): _count_local_quotes(), _mailpit_otp(), _post(), _put_local_quote(), test_authed_autosave_shelf_cleanup(), test_guest_profile_and_admin_guard(), test_guest_quote_merges_on_register(), call() (+4 more)

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
Cohesion: 0.25
Nodes (6): assert_golden(), test_visual_admin(), test_visual_home(), test_visual_overview(), test_visual_profile_guest(), test_visual_reader()

### Community 64 - "semantic_ai_test.rs"
Cohesion: 0.14
Nodes (20): one_hot(), seed_reader(), seed_test_context(), SeededAiContext, SeededReader, test_ai_rate_limiter_burst_enforcement(), test_atomic_cards_scoped_to_path_book(), test_atomic_insight_cards_cache_hit_and_miss() (+12 more)

### Community 65 - "rate_limit.rs"
Cohesion: 0.20
Nodes (11): AI_POLICY, AUTH_POLICY, FALLBACK_CLIENT_IP, forwarded_for_takes_the_first_parseable_hop(), public_policy(), public_policy_takes_cap_from_config(), RateLimitPolicy, request_with_ip() (+3 more)

### Community 67 - "RetryAndRecapTests"
Cohesion: 0.16
Nodes (3): RetryAndRecapTests, backoff_seconds(), build_recap_input()

### Community 68 - "email.rs"
Cohesion: 0.17
Nodes (10): EmailConfig, build_transport(), mailpit_accepts_cleartext_delivery_when_running(), mask_email(), otp_message(), otp_message_addresses_sender_and_recipient(), send_otp_email(), smtp_error() (+2 more)

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
Cohesion: 0.22
Nodes (5): check(), externalize(), replace(), _is_inline(), main()

### Community 78 - "components/insights.rs"
Cohesion: 0.24
Nodes (9): AtomicCards(), CatchupRecap(), InsightList(), InsightText(), QuoteFinder(), share_quote(), Sheet(), string_field() (+1 more)

### Community 79 - "gi"
Cohesion: 0.28
Nodes (9): bi(), gi(), ii(), Ko(), li(), mi(), oi(), Qo() (+1 more)

### Community 83 - "main"
Cohesion: 0.18
Nodes (4): init_db_pool(), init_redis_client(), main(), shutdown_signal()

### Community 84 - "use_lang"
Cohesion: 0.09
Nodes (27): JobStatusDto, SiteHeader(), use_session(), ShellLayout(), use_toasts(), use_lang(), AdminPage(), CuratorDesk() (+19 more)

### Community 85 - "test_responsive.py"
Cohesion: 0.46
Nodes (4): _page_at(), test_home_and_book_fit_viewport(), test_reader_paginates_long_chapter(), _visible()

### Community 88 - "Ingestion Worker (`python_worker/`)"
Cohesion: 0.25
Nodes (7): Environment, Ingestion Worker (`python_worker/`), Job lifecycle, Notes & deviations, Run, Setup, Unit tests (no services required)

### Community 96 - "progress_repository.rs"
Cohesion: 0.19
Nodes (9): Model, Relation, check_and_award_badges(), get_active_progress(), get_streak(), record_heartbeat(), seconds_read_on(), update_progress() (+1 more)

### Community 100 - "test_reader_flip.py"
Cohesion: 0.36
Nodes (5): _drag(), _open_long_chapter(), test_drag_turns_like_paper(), test_keyboard_turns_and_dims(), test_reduced_motion_turns_without_paper()

### Community 116 - "migrate.sh"
Cohesion: 0.76
Nodes (6): init_table(), migrate_down(), migrate_status(), migrate_up(), psql_cmd(), migrate.sh script

### Community 122 - "test_reader_flow.py"
Cohesion: 0.52
Nodes (4): _chapter_no(), test_end_of_book_notice_is_visible(), test_offline_save_read_badge(), test_reader_body_and_type_sheet()

### Community 134 - "enforce"
Cohesion: 0.33
Nodes (7): ai_rate_limit_middleware(), client_ip_from_headers(), enforce(), extract_client_ip(), public_rate_limit_middleware(), rate_limit_middleware(), set_limit_headers()

### Community 138 - "goto"
Cohesion: 0.24
Nodes (11): goto(), test_hero_plate_is_static(), test_home_catalog_shows_two_rows_then_everything(), test_i18n_toggle_and_persist(), test_overview_and_sheets(), test_search_returns_cards(), _section_top(), test_cover_art_never_shows_a_broken_image() (+3 more)

### Community 139 - "Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya."
Cohesion: 0.40
Nodes (4): Answer, Outcome, Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya., Source Nodes

### Community 144 - "shared"
Cohesion: 0.70
Nodes (5): domain, infra, server, shared, web

## Knowledge Gaps
- **142 isolated node(s):** `COMPLETION_MIN`, `COMPLETION_MAX`, `DAILY_THRESHOLD_SECONDS`, `BASE_HEARTBEAT_XP`, `STREAK_BONUS_XP` (+137 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 746 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **89 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppError` connect `AppError` to `progress_repository.rs`, `shared/src/lib.rs`, `email.rs`, `AuthUser`, `quotes.rs`, `SmtpSecurity`, `otp.rs`, `database_test.rs`, `queue.rs`, `config.rs`, `HttpError`, `AppState`, `reliability_test.rs`, `main`, `Value`, `get_atomic_cards`, `embedding.rs`, `quote_repository.rs`?**
  _High betweenness centrality (0.075) - this node is a cross-community bridge._
- **Why does `AppState` connect `AppState` to `AuthUser`, `enforce`, `routes/progress.rs`, `quotes.rs`, `routes/auth.rs`, `.from_request_parts`, `HttpError`, `AppError`, `config.rs`, `otp.rs`, `TestHarness`, `gamification.rs`, `get_atomic_cards`, `embedding.rs`?**
  _High betweenness centrality (0.053) - this node is a cross-community bridge._
- **Why does `TestHarness` connect `TestHarness` to `semantic_ai_test.rs`, `admin_test.rs`, `AppState`, `main`, `auth_test.rs`?**
  _High betweenness centrality (0.038) - this node is a cross-community bridge._
- **Are the 23 inferred relationships involving `n()` (e.g. with `axe.min.js` and `A()`) actually correct?**
  _`n()` has 23 INFERRED edges - model-reasoned connections that need verification._
- **Are the 2 inferred relationships involving `HttpError` (e.g. with `.from_request_parts()` and `enforce()`) actually correct?**
  _`HttpError` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `COMPLETION_MIN`, `COMPLETION_MAX`, `DAILY_THRESHOLD_SECONDS` to the rest of the system?**
  _142 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `shared/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.11529411764705882 - nodes in this community are weakly interconnected._