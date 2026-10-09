# Graph Report - project-baca  (2026-10-09)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 3659 nodes · 7943 edges · 259 communities (132 shown, 127 thin omitted)
- Extraction: 88% EXTRACTED · 12% INFERRED · 0% AMBIGUOUS · INFERRED: 956 edges (avg confidence: 0.89)
- Token cost: 13,488 input · 2,999 output

## Graph Freshness
- Built from commit: `0803c871`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Project Documentation Index
- Security Audit Findings
- Job Queue Infrastructure
- Web Frontend Testing
- Reader Functional Requirements
- Security Audit Reports
- Database Schema Migrations
- Reader Animation Logic
- Engineering Standards
- AI Embedding Provider
- Offline Sync API
- Domain Status Rules
- API Boundary Tests
- Infrastructure Setup Guide
- User and Book Tables
- Shared Data Objects
- Content Ingestion ADRs
- Admin UI Components
- AI Agent Guidelines
- Frontend Route Constants
- JWT Session Management
- API Client Utilities
- Semantic Search Infrastructure
- Axum Auth Middleware
- System Architecture Overview
- Reader Layout Engine
- Project Milestones Roadmap
- Authentication Route Handlers
- Frontend Shell Layout
- Catalog API Endpoints
- Ingestion Pipeline Topology
- Architecture Decision Records
- Reader UI Enhancements
- Internationalization Middleware
- AI Insights API
- Admin Integration Tests
- Text Chunking Utility
- Home Page Components
- Knowledge Log History
- Auth Manual Testing
- Book Repository Access
- Reader Theme Preferences
- Worker Python Scripts
- Worker Error Handling
- API Contract Standards
- UI Audit Fixes
- Book Formatting Helpers
- Common Server Utilities
- IndexedDB Local Storage
- Server State Configuration
- AI Integration Tests
- QA Launch Checklist
- Design System Assets
- Quote Repository Access
- Quote API Handlers
- Production Deployment Config
- S3 Storage Service
- App Environment Config
- Auth Integration Tests
- SVG Icon Library
- UI Internationalization Module
- Knowledge Base Maintenance
- Badge Repository Access
- Pytest Browser Fixtures
- Reading Progress Sync
- Developer Environment Setup
- Auth UI Components
- Typed Configuration Models
- Account Management Security
- System Hardening ADRs
- Rate Limiting Middleware
- Admin Ingestion Features
- Production Launch Checklist
- Legal and Audit Trail
- Frontend Reader Engine
- SMTP Email Service
- Component Unit Tests
- HTML Sanitization Utility
- Test Data Seeders
- Book Cover API
- OTP Security Logic
- Insight UI Components
- EPUB Parsing Logic
- Database Entity Models
- Navigation Integration Tests
- UI Design Screenshots
- Book Route Handlers
- Data Philosophy Documentation
- User Repository Access
- Gamification API Handlers
- Insight Route Handlers
- OWASP Security Tests
- Accessibility Audit Tests
- Worker Unit Tests
- Password Hashing Logic
- Progress Repository Access
- Catalog Integration Tests
- Visual Regression Testing
- Build Script Utilities
- Retry Logic and Backoff
- Web Frontend Pages
- Toast Notification System
- Session and Auth Tests
- Manual Testing Documentation
- Gamification and Heartbeat
- Book Entity Models
- Rate Limit Middleware
- User Session Management
- API Data Transfer Objects
- Book Card Component
- Reader Interaction Tests
- TLDR Cache Model
- Reader Enhancement Tasks
- Saved Quotes Model
- Reading Progress Model
- Reading Streak Model
- User Entity Model
- Progress UI Components
- Reader Flow Tests
- Database Migration Scripts
- Chapter Entity Model
- Activity Log Model
- Tag Entity Model
- Entity Relation Definitions
- User Badge Model
- Security Middleware
- App State Configuration
- Guest Progress Merging
- Knowledge Base Index
- Tauri Shell Documentation
- Gemini API Integration
- Accessibility Regression Tests
- Graph Query Responses
- Launch and Trademark
- Rate Limit Follow-ups
- Responsive Design Follow-ups
- Redis Resilience Follow-ups
- Auth System Follow-ups
- Infrastructure Hygiene
- WASM Bundle Optimization
- Reader Position Integrity
- API Client Follow-ups
- Accessibility Polish
- Service Worker and Fonts
- Project Directory Structure
- Database Relations
- Database Relations
- Database Relations
- Database Relations
- Database Relations
- Database Relations
- Database Relations
- Database Relations
- Database Relations
- Database Relations
- Database Relations
- Offline Content DTOs
- Reading Streak DTO
- Avatar URL Validation
- Active Model Behavior
- Active Model Behavior
- Active Model Behavior
- Active Model Behavior
- Active Model Behavior
- Active Model Behavior
- Active Model Behavior
- Active Model Behavior
- Active Model Behavior
- Active Model Behavior
- Active Model Behavior
- Active Model Behavior
- Active Model Behavior
- Brain Router Skill
- Memory Care Skill
- Memory Recall Skill
- Database Backup Script
- Admin Sorting Visuals
- Admin Pigeonhole Visuals
- Reader View Visuals
- Reader Mascot Illustration
- Library Catalog Visuals
- Library Bookshelf Visuals
- Ingestion Monitor Visuals
- Manuscript Inspection Visuals
- AI Quote Visuals
- Discovery Rocket Visuals
- Brand Mark Assets
- Design Reference
- Brand Icon Assets
- App Icons
- PWA Assets
- PWA Assets
- PWA Assets
- Admin UI Snapshots
- Home UI Snapshots
- Book UI Snapshots
- Profile UI Snapshots
- Reader UI Snapshots
- Home UI Screenshots
- UI Audit Screenshots
- Book UI Screenshots
- Search UI Screenshots
- Reader UI Screenshots
- Reader UI Components
- Admin UI Screenshots
- Reader Views
- Auth UI Screenshots
- Reader UI Styling
- Project Management
- Technical Debt
- Rate Limiting Tasks

## God Nodes (most connected - your core abstractions)
1. `AppError` - 119 edges
2. `AppState` - 75 edges
3. `MEMORY.md: Architecture Decision Records` - 61 edges
4. `Project Baca Knowledge Log (audit trail)` - 54 edges
5. `Log — Project Baca Knowledge Bundle` - 51 edges
6. `Audit menyeluruh Rotaria, 8 Oktober 2026` - 44 edges
7. `Chronological Decision Records` - 37 edges
8. `Round-2 audit 2026-10-08` - 37 edges
9. `Project Baca PRD (MVP: E-Reader + Atomic AI Insights)` - 36 edges
10. `assert_no_page_errors()` - 34 edges

## Surprising Connections (you probably didn't know these)
- `8. Requirements Traceability Matrix (RTM)` --references--> `books()`  [INFERRED]
  knowledge/frd.md → crates/web/tests/component/test_components.py
- `Postgres for Everything (No Elasticsearch)` --semantically_similar_to--> `Postgres for Everything`  [INFERRED] [semantically similar]
  CLAUDE.md → knowledge/erd.md
- `2. Steps` --references--> `set_job()`  [INFERRED]
  knowledge/tasks/28_worker_followups.md → python_worker/worker.py
- `2026-09-28 M7 Sequential Execution: Hardening + BL-10 + BL-11 + BL-12` --references--> `_safe_member()`  [INFERRED]
  knowledge/log.md → python_worker/worker.py
- `1. Engineering Invariants & Data Philosophy` --references--> `books()`  [INFERRED]
  knowledge/erd.md → crates/web/tests/component/test_components.py

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Infrastructure Services** — redis_7, minio [EXTRACTED 1.00]
- **Book Overview to Atomic Cards Unavailable Flow** — knowledge_output_audit_2026_09_27_10_saved_offline_book_action_bar, knowledge_output_audit_2026_09_27_10_saved_offline_mono_ai_tool_buttons, knowledge_output_audit_2026_09_27_05_atomic_atomic_insights_modal, knowledge_output_audit_2026_09_27_05_atomic_atomic_cards_not_found_state [INFERRED 0.75]
- **Authentication and Security Hardening Lineage** — memory_adr_15_authentication, memory_owasp_asvs_hardening, memory_auth_latency_optimization, memory_adr_20_security_latency_hardening, memory_adr_21_full_system_audit, memory_adr_22_pre_m6_debt_paydown, memory_adr_27_m7_hardening_deploy_sessions_pwa, memory_adr_29_audit_hardening_two_surface, architecture_security_model [INFERRED 0.85]
- **EPUB ingestion lifecycle** — knowledge_manual_test_05_ingestion_and_admin_post_api_v1_admin_books_upload, knowledge_manual_test_05_ingestion_and_admin_draft_book_status, knowledge_manual_test_05_ingestion_and_admin_redis_job_hash_job_id, knowledge_manual_test_05_ingestion_and_admin_python_worker_pipeline, knowledge_manual_test_05_ingestion_and_admin_reclaim_idle_ms_xautoclaim_reclaim, knowledge_manual_test_05_ingestion_and_admin_stream_epub_ingestion_dlq_dead_letter_stream, knowledge_tasks_28_worker_followups_finding_28_2_xautoclaim_idle_time_never_refreshed_during_a_job, knowledge_tasks_28_worker_followups_finding_28_6_dlq_replay_xrange_xadd_xdel_race, knowledge_tasks_29_edge_and_compose_hygiene_finding_29_5_ingestion_streams_never_trimmed [INFERRED 0.85]
- **Quote Finder Search Flow** — knowledge_output_audit_2026_09_27_22_saved_ghost_secondary_buttons, knowledge_output_audit_2026_09_27_20_quote_empty_quote_finder_modal, knowledge_output_audit_2026_09_27_20_quote_empty_quote_search_input, knowledge_output_audit_2026_09_27_20_quote_empty_no_quotes_empty_state [INFERRED 0.85]
- **Offline-first web storage and sync** — knowledge_manual_test_06_web_reader_project_baca_db_indexeddb_rexie_db_version_2, knowledge_manual_test_06_web_reader_guest_progress_store, knowledge_manual_test_06_web_reader_offline_books_store, knowledge_manual_test_06_web_reader_offline_chapters_store, knowledge_manual_test_06_web_reader_pending_sync_queue_store, knowledge_manual_test_06_web_reader_save_offline, knowledge_manual_test_07_qa_sessions_pwa_launch_offline_reader_fallback_with_offline_copy_badge, knowledge_manual_test_07_qa_sessions_pwa_launch_pending_write_fifo_queue_with_footer_counter, knowledge_tasks_32_session_and_api_client_followups_finding_32_4_drain_pending_clears_the_store_wholesale_and_allows_concurrent_drains, knowledge_tasks_31_reader_position_integrity_finding_31_1_saved_anchor_race_with_chapter_layout [INFERRED 0.85]
- **Refresh token lifecycle hardening** — knowledge_manual_test_02_authentication_refresh_token_14d_single_use_rotation, knowledge_manual_test_02_authentication_refresh_token_family_revocation_on_replay, knowledge_manual_test_02_authentication_hashed_refresh_token_storage, knowledge_manual_test_07_qa_sessions_pwa_launch_401_single_flight_refresh_retry, knowledge_tasks_27_auth_followups_finding_27_3_grace_window_successor_returned_without_existence_check, knowledge_tasks_32_session_and_api_client_followups_finding_32_2_refresh_failure_leaves_tokens_stored_and_session_authed_true, knowledge_tasks_32_session_and_api_client_followups_refreshing_thread_local_single_flight_flag [INFERRED 0.85]
- **Admin EPUB ingestion pipeline** — knowledge_prd_us_12, knowledge_frd_fr_adm_01_epub_upload_validation, knowledge_frd_fr_adm_02_extraction_sanitization_scene_chunking, knowledge_frd_fr_adm_03_vector_embedding_ai_summary_caching, knowledge_srs_post_admin_books_upload, knowledge_srs_get_admin_jobs_id, knowledge_srs_stream_epub_ingestion, knowledge_ux_flow_screen_6_admin_epub_upload, knowledge_ux_flow_screen_7_ingestion_monitor, knowledge_prd_python_ingestion_worker [INFERRED 0.95]
- **EPUB Ingestion Flow** — distributed_stream_epub_ingestion, distributed_consumer_group_ingestion_workers, distributed_job_hash, distributed_dead_letter_stream, distributed_pipeline_per_message, python_worker_readme_ingestion_worker, docker_compose_prod_worker_service, architecture_admin_endpoints, architecture_ingestion_and_ai [INFERRED 0.95]
- **Guest-to-cloud auto-merge flow** — knowledge_prd_us_05, knowledge_frd_fr_usr_03_guest_to_cloud_auto_merge, knowledge_srs_post_progress_merge, knowledge_ux_flow_guest_to_account_reconciliation_auto_merge, knowledge_erd_user_reading_progress, knowledge_ux_flow_authsheet, knowledge_ux_flow_storage_rexie_db [INFERRED 0.95]
- **Scoped semantic quote search (WHERE book_id = $1)** — knowledge_prd_us_08, knowledge_frd_fr_ai_01_scoped_semantic_quote_finder, knowledge_srs_post_books_id_quotes_search, knowledge_erd_idx_book_chunks_hnsw_embedding, knowledge_erd_scoped_quote_search_query, knowledge_ux_flow_screen_4_scoped_quote_finder_modal, knowledge_ux_flow_scoped_semantic_search, knowledge_srs_ai_rate_limit [INFERRED 0.95]
- **Shared Engineering Invariants Across Agent Guides** — agents_zero_panics_in_rust, knowledge_erd_postgres_for_everything, agents_pragmatic_message_broker, agents_paired_sql_migrations, agents_centralized_makefile_automation, agents_edge_transport_termination, agents_zero_emoji_policy, claude_zero_panics_invariant, claude_postgres_for_everything, claude_redis_streams_for_async_tasks, claude_edge_transport_termination, gemini_rust_zero_panics, gemini_pragmatic_message_broker, gemini_seaorm_paired_migrations, architecture_invariants, memory_six_engineering_pillars_zero_emoji [INFERRED 0.95]

## Communities (259 total, 127 thin omitted)

### Community 0 - "Project Documentation Index"
Cohesion: 0.06
Nodes (71): Dokumen, Project Baca ERD & PostgreSQL 17 Database Schema, SeaORM (crates/infra), Project Baca FRD (Functional Requirements Document), 1. Directory map, 2. Documents, 3. Reading order, 4. Invariants (+63 more)

### Community 1 - "Security Audit Findings"
Cohesion: 0.06
Nodes (73): Audit menyeluruh Rotaria, 8 Oktober 2026, BE medium: draft/archived chapters and insights readable by UUID, BE low findings (swagger in prod, health lies, logout jti, no-store, saved_quotes uniqueness, root containers), BE high: worker sanitizer re-emits decoded entities as markup, BE medium: search term without maximum length, Bukti pengujian yang dijalankan, Design canvas Fase 2 (claude.ai artifact 32ipDoSuTqrREUBg2V5B8W), FE medium: admin job polling not cleaned up on leaving /admin (+65 more)

### Community 2 - "Job Queue Infrastructure"
Cohesion: 0.07
Nodes (26): dlq_entry(), get_job_status(), INGESTION_DLQ_STREAM, INGESTION_GROUP, INGESTION_STREAM, job_key(), JOB_KEY_PREFIX, JOB_TTL_SECS (+18 more)

### Community 3 - "Web Frontend Testing"
Cohesion: 0.05
Nodes (50): _page_at(), test_home_and_book_fit_viewport(), test_reader_paginates_long_chapter(), _visible(), Hero carousel, Leptos 0.7.8 WASM SPA, 4. Part C — M7 PWA Track (BL-12), crates/web/index.html (+42 more)

### Community 4 - "Reader Functional Requirements"
Cohesion: 0.06
Nodes (58): 2. Module 2: Reflowable Reader & Offline Engine, 3. Module 3: Authentication, Users & Reconciliation, FR-RDR-01: Multi-Column Reflowable Text Rendering, FR-RDR-02: Tap-to-Turn Navigation, FR-RDR-03: DOM CFI Position Locking, FR-RDR-04: Typography Controls, FR-RDR-05: Offline Storage via IndexedDB, FR-RDR-06: Connectivity Detection & Offline Fallback (+50 more)

### Community 5 - "Security Audit Reports"
Cohesion: 0.06
Nodes (57): BE medium: hand-rolled SMTP without STARTTLS and AUTH, BE high: client IP header spoofable in production (cf-connecting-ip, x-real-ip honoured as-is), BE verified good (Argon2id OWASP, jti blacklist, atomic refresh rotation, hashed OTP, user_id scoping, CSP, streaming upload, zip bomb guard), FE medium: service worker without shell fallback or cache pruning, Audit ronde 2, 2026-10-08: hasil test penuh dan celah tersisa, Audit ronde 2, 2026-10-08, Backend verified correct in round 2 (single trusted IP header, sanitizer escape, draft hiding, Gemini key in header, lettre SMTP, query caps, Swagger off, /health 503, jti blacklist, no-store, quote dedupe, covers route), Celah backend (task 26 sampai 29, tambahan task 21) (+49 more)

### Community 6 - "Database Schema Migrations"
Cohesion: 0.08
Nodes (58): Migration 02 create_users_and_roles, Migration 03 create_books_and_tags, Migration 06 create_progress_and_gamification, Migration 07 optimize_indexes_and_foreign_keys, table badges, table book_tags, table books, Catalog & Taxonomy Cluster (+50 more)

### Community 7 - "Reader Animation Logic"
Cohesion: 0.07
Nodes (28): angle_for_travel(), BEND_MAX, clear_children(), column_template(), Dir, DRAG_SPRING, DragStart, FLICK_VELOCITY (+20 more)

### Community 8 - "Engineering Standards"
Cohesion: 0.05
Nodes (33): 1. Single Source of Truth (SSOT), 2. Non-Negotiable Core Invariants, 3. Engineering Pillars, 4. Audit Trail Protocol, 5. Zero Emoji Policy & Link Integrity, AGENTS.md — AI Agent Guidelines & Governance, Graphify Knowledge Graph (graphify-out/), Minimalist Comments (no spec tracers in code) (+25 more)

### Community 9 - "AI Embedding Provider"
Cohesion: 0.08
Nodes (22): API_KEY_HEADER, build_embedding_provider(), EmbeddingProvider, FastEmbedProvider, GeminiBatchEmbedRequest, GeminiBatchEmbedResponse, GeminiContent, GeminiEmbedding (+14 more)

### Community 10 - "Offline Sync API"
Cohesion: 0.06
Nodes (49): Audit 2026-09-27 REPORT.md, BookDetailDto, GET /api/v1/books/{id}, GET /api/v1/books/{id}/offline-bundle, GET /api/v1/me/badges, Offline bundle for IndexedDB, Step 3.5: Book Overview & Chapter Summary, Step 3.7: Offline Bundle Synchronization (+41 more)

### Community 11 - "Domain Status Rules"
Cohesion: 0.06
Nodes (24): advance_streak(), BASE_HEARTBEAT_XP, BookStatus, COMPLETION_MAX, COMPLETION_MIN, DAILY_MILESTONE_BONUS_XP, DAILY_THRESHOLD_SECONDS, day() (+16 more)

### Community 12 - "API Boundary Tests"
Cohesion: 0.06
Nodes (6): get_cover(), test_cover_missing_and_invalid_names_are_404(), test_cover_roundtrip_through_storage(), BookCatalogPort, test_reliability_mock_repository_fault_injection(), test_signup_fails_closed_when_smtp_unreachable()

### Community 13 - "Infrastructure Setup Guide"
Cohesion: 0.07
Nodes (47): 1. Overview, 2. Prerequisites & Service Status Verification, 3. Server Health & OpenAPI Documentation, 4. Structured Log Inspection, Axum server runtime, LOG_FORMAT=json structured logging, Mailpit SMTP and webmail, make dev-server (+39 more)

### Community 14 - "User and Book Tables"
Cohesion: 0.07
Nodes (43): idx_users_admin_role, idx_users_email, users, book_tags, books, idx_book_tags_reverse, idx_books_catalog_filter, idx_books_published_author_trgm (+35 more)

### Community 15 - "Shared Data Objects"
Cohesion: 0.12
Nodes (40): ActiveProgressDto, AdminBookPatchRequest, AdminBookRowDto, ApiResponse, BadgeDto, BookCatalogQuery, BookDetailDto, BookSearchQuery (+32 more)

### Community 16 - "Content Ingestion ADRs"
Cohesion: 0.06
Nodes (40): book_chunks table, books table, Domain tables created by migrations, saved_quotes table, tldr_cache table, users table, ADR-20, Honest Expectations for Stub Content (+32 more)

### Community 17 - "Admin UI Components"
Cohesion: 0.09
Nodes (27): JobStatusDto, SiteHeader(), use_session(), ShellLayout(), use_toasts(), use_lang(), AdminPage(), CuratorDesk() (+19 more)

### Community 18 - "AI Agent Guidelines"
Cohesion: 0.06
Nodes (31): 1. Primary Commands (Makefile Orchestration), 2. Monorepo Architecture Overview, 3. Strict Coding Invariants, 4. Working rules, 5. Documentation & Audit Trail Workflow, CLAUDE.md — Claude Code Guidelines for Project Baca, 1. Navigation & Source of Truth (SSOT), 2. Non-Negotiable Engineering Invariants (+23 more)

### Community 19 - "Frontend Route Constants"
Cohesion: 0.14
Nodes (33): admin_books(), admin_dlq(), admin_dropoff(), admin_job(), admin_replay(), admin_set_status(), all_badges(), authed() (+25 more)

### Community 20 - "JWT Session Management"
Cohesion: 0.17
Nodes (36): blacklist_access_token(), Claims, generate_access_token(), generate_access_token_issued_at(), generate_refresh_token(), invalidate_user_tokens(), is_token_blacklisted(), is_user_token_revoked() (+28 more)

### Community 21 - "API Client Utilities"
Cohesion: 0.07
Nodes (28): ReadingProgressUpdateDto, active_progress(), ADMIN_PAGE_SIZE, as_html_element(), asset_url(), change_password(), drain_pending(), encode_param() (+20 more)

### Community 22 - "Semantic Search Infrastructure"
Cohesion: 0.09
Nodes (40): BE medium: Gemini API key can leak into logs via ?key= URL, Task 28 (medium): worker follow-ups (Gemini key in URL, reclaim after 5 min without unique chunk index, publish ignores archive, no SIGTERM, non-idempotent DLQ replay), Migration 01 init_extensions, Migration 04 create_chapters_and_chunks, Migration 05 create_tldr_cache, table book_chunks, table chapters, Chapters & Semantic Vectors Cluster (+32 more)

### Community 23 - "Axum Auth Middleware"
Cohesion: 0.08
Nodes (10): HttpError, AuthUser, get_active_progress(), merge_guest_progress(), progress_routes(), update_progress(), Finding 27.6: Progress endpoints accept draft and archived books, get_active_progress (+2 more)

### Community 24 - "System Architecture Overview"
Cohesion: 0.12
Nodes (36): Argon2id Password Hashing (19 MiB, 2 iterations), Auth Endpoints (/auth/signup, verify-otp, login, refresh, logout), Axum Middleware Layers (tracing, CORS, x-request-id, security headers), Catalog Endpoints (/books, /books/search, chapters, offline-bundle), Caddy Edge (TLS, HTTP/3, SPA from dist/, /api proxy), GET /covers/{file} (immutable cache), Cover Storage (covers/<book uuid>.<ext> in covers bucket), crates/domain (pure rules, no I/O) (+28 more)

### Community 25 - "Reader Layout Engine"
Cohesion: 0.08
Nodes (24): anchor_index(), columns_for(), css_px(), Paddings, page_of(), paragraphs(), TWO_COLUMN_MIN_WIDTH, ANCHOR_END (+16 more)

### Community 26 - "Project Milestones Roadmap"
Cohesion: 0.08
Nodes (19): [0.2.1] 2026-09-26, Account Abuse Controls, Argon2id Thread Offloading, 768-dim Dual-Mode Embedding Engine, Database Index Optimization (Migration 07), Milestone 01 Infrastructure and Entities, Milestone 02 Authentication and Guest Reconciliation, Milestone 03 Catalog Backend and Reader API (+11 more)

### Community 27 - "Authentication Route Handlers"
Cohesion: 0.19
Nodes (23): client_ip_from_headers(), auth_routes(), change_password(), delete_me(), get_me(), INVALID_CREDENTIALS, issue_token_pair(), list_my_sessions() (+15 more)

### Community 28 - "Frontend Shell Layout"
Cohesion: 0.07
Nodes (10): current(), HOW_IT_WORKS_ID, ACCOUNT_ID, current(), TabBar(), ERROR_MS, INFO_MS, gallery_body() (+2 more)

### Community 29 - "Catalog API Endpoints"
Cohesion: 0.07
Nodes (35): chapters table, 1. Overview, 2. Web Client Verification (M6, `http://127.0.0.1:3000`), 3. Catalog Discovery & Typo-Tolerant Search, 4. Book Overview, Reader Content & Offline Bundle, 5. Reading Progress, CFI Anchors & Active Position, 6. Gamification: Heartbeats, Streaks & Badges, Cursor-based catalog pagination (+27 more)

### Community 30 - "Ingestion Pipeline Topology"
Cohesion: 0.12
Nodes (28): Admin Endpoints (/admin/books, upload, jobs, dlq, analytics), Atomic Cards and Spoiler-Free Recap Generation (LLM_MODEL_NAME), Chunking (about 400 words, tails under 50 merged, CFI ranges), Consumer Group ingestion-workers, Cover Extraction to WebP (Pillow), stream:epub_ingestion:dlq, DISTRIBUTED.md: Ingestion Worker and Queue, Batch Embedding (768 dimensions, EMBEDDING_MODEL_NAME) (+20 more)

### Community 31 - "Architecture Decision Records"
Cohesion: 0.06
Nodes (36): 2026-09-25 — 6 Core Engineering Pillars & Zero Emoji Policy, 2026-09-25 — Data Access: SeaORM & Paired SQL Migrations, 2026-09-25 — Database Selection: "Postgres for Everything" vs Elasticsearch, 2026-09-25 — Eliminated Triton Server & Adopted 768-Dim Dual-Mode Embeddings, 2026-09-25 — OKF v0.2 Implementation & Obsidian Integration, 2026-09-25 — Queue Architecture: Redis Streams vs RedPanda / RabbitMQ, 2026-09-25 — Visual Identity: Vintage Literary (1900–1950), 2026-09-26 — Adoption of Caddy Reverse Proxy for HTTP/3 (QUIC) Edge Termination with Automatic HTTP/2 Fallback (ADR-17) (+28 more)

### Community 32 - "Reader UI Enhancements"
Cohesion: 0.09
Nodes (32): PUT /api/v1/progress/{book_id}, Reader tap zones, Recap entry point for chapters > 1, Search 250 ms debounce and Load More, Contents drawer from center tap menu, Task 14 guest local quote store, Task 23: Reader enhancements, columns_for (+24 more)

### Community 33 - "Internationalization Middleware"
Cohesion: 0.11
Nodes (11): apply_lang(), ENTRIES, Lang, LANG_KEY, lang_signal(), LangContext, provide_lang(), badge_description() (+3 more)

### Community 34 - "AI Insights API"
Cohesion: 0.09
Nodes (30): ADR-21, RFC rate limit headers, 1. Overview, 2. Scoped Quote Search (guest-open), 3. Saved Quotes & Vintage Cards (Authenticated, Owner-Scoped), 4. Atomic Cards & Spoiler-Free Recaps, 5. Honest Expectations for Stub Content, chapter_ref path parameter (+22 more)

### Community 35 - "Admin Integration Tests"
Cohesion: 0.15
Nodes (27): admin_get(), admin_patch(), AdminContext, BOUNDARY, multipart_body(), seed_funnel(), seed_status_book(), seed_users() (+19 more)

### Community 36 - "Text Chunking Utility"
Cohesion: 0.08
Nodes (10): BATCH_SIZE, chunk_text(), CHUNK_WORDS, main(), MIN_CHARS, strip_html(), init_db_pool(), init_redis_client() (+2 more)

### Community 37 - "Home Page Components"
Cohesion: 0.10
Nodes (21): prefers_reduced_motion(), reveal(), scroll_to_hash(), scroll_to_location_hash(), catalog_columns(), CATALOG_PREVIEW_ROWS, HomePage(), LanguageFilter (+13 more)

### Community 38 - "Knowledge Log History"
Cohesion: 0.06
Nodes (31): 2026-09-25 — Core Engineering Invariants & Zero Emoji Policy, 2026-09-25 — Dual-Mode Embeddings, Triton Elimination & Private Gitignore, 2026-09-25 — OKF v0.2 Knowledge Vault & Graphify Knowledge Graph, 2026-09-26 — Auth & User Management Performance Optimization, Query Architecture & Vulnerability Test Suite, 2026-09-26 — Cargo Workspace Initialization & 5-Crate Skeleton, 2026-09-26 — Database Performance Optimization: Index Pruning, Foreign Key Coverage & Catalog Zero-Sort, 2026-09-26 — Edge Transport Architecture: Caddy HTTP/3 (QUIC) Reverse Proxy & Auto-HTTPS, 2026-09-26 — Graphify Knowledge Graph & Obsidian Vault Synchronization (+23 more)

### Community 39 - "Auth Manual Testing"
Cohesion: 0.10
Nodes (28): 1. Overview, 2. Step-by-Step Test Procedure, 6-digit email OTP, Argon2id password hashing, GET /api/v1/me, JWT access token (24h), Manual Testing — Milestone 02: Authentication, Security & Guest Progress, PATCH /api/v1/me (+20 more)

### Community 40 - "Book Repository Access"
Cohesion: 0.20
Nodes (18): ADMIN_ROW_SELECT, AdminBookRow, AdminBookRowDto, chapter_dropoff(), ensure_book_published(), get_book_by_id(), get_book_for_admin(), get_book_status() (+10 more)

### Community 41 - "Reader Theme Preferences"
Cohesion: 0.09
Nodes (13): FONT_SIZE_MAX, FONT_SIZE_MIN, LEADING_PRESETS, LINE_HEIGHT_MAX, LINE_HEIGHT_MIN, prefers_light(), READER_KEY, ReaderPrefs (+5 more)

### Community 42 - "Worker Python Scripts"
Cohesion: 0.12
Nodes (15): 1. Findings (audit 2026-10-08, round 2), convert_cover_to_webp(), embed_batch(), ensure_group(), _gemini_post(), generate_json(), main(), make_db() (+7 more)

### Community 43 - "Worker Error Handling"
Cohesion: 0.10
Nodes (29): RECLAIM_IDLE_MS XAUTOCLAIM reclaim, docker-compose.prod.yml, Dockerfile.worker, Finding 28.2: XAUTOCLAIM idle time never refreshed during a job, Finding 28.4: EPUB-derived strings inserted untrimmed, Finding 28.5: Worker runs as PID 1 without SIGTERM handler, Finding 28.6: DLQ replay XRANGE/XADD/XDEL race, migrations/ (+21 more)

### Community 44 - "API Contract Standards"
Cohesion: 0.07
Nodes (29): 2026-09-26 — Comprehensive Testing Modules: Database Integrity, Load & Stress, and API Boundary Conformance, Legitimate contracts, not bugs (401 without token, 404 for ungenerated insights, empty quote search), 1. Security Architecture, 2. API Response & Error Standards, 3. REST API Endpoint Contracts, 4. Non-Functional Requirements (NFR), A. Health, B. Authentication and account (+21 more)

### Community 45 - "UI Audit Fixes"
Cohesion: 0.12
Nodes (29): 2026-09-27 — Pre-M7 Web Audit: 8 Findings Fixed, Save-Offline Root-Caused, Dimmed Modal Backdrop, No Quotes Empty State Message, Quote Finder Modal, Quote Search Input, Quote Finder Empty State Screenshot, Search and Close Buttons, Book Metadata Block (+21 more)

### Community 46 - "Book Formatting Helpers"
Cohesion: 0.07
Nodes (7): reading_minutes(), roman(), WORDS_PER_MINUTE, current_url(), resume_chapter(), scroll_to_location_hash, Step 31.5: Keydown guard, touch-action pinch-zoom, scroll after set_book

### Community 47 - "Common Server Utilities"
Cohesion: 0.11
Nodes (3): CLIENT_IP_HEADER, Seeded, TestHarness

### Community 48 - "IndexedDB Local Storage"
Cohesion: 0.12
Nodes (16): clear(), db(), DB_NAME, DB_VERSION, delete(), get_all(), local_quote_key(), LocalQuoteRecord (+8 more)

### Community 49 - "Server State Configuration"
Cohesion: 0.13
Nodes (9): api_health_check(), ApiDoc, AppState, create_app(), health_check(), not_found_handler(), REQUEST_ID_HEADER, SecurityAddon (+1 more)

### Community 50 - "AI Integration Tests"
Cohesion: 0.14
Nodes (20): one_hot(), seed_reader(), seed_test_context(), SeededAiContext, SeededReader, test_ai_rate_limiter_burst_enforcement(), test_atomic_cards_scoped_to_path_book(), test_atomic_insight_cards_cache_hit_and_miss() (+12 more)

### Community 51 - "QA Launch Checklist"
Cohesion: 0.11
Nodes (24): POST /api/v1/auth/revoke-all, Step 2.11: Revoke All Sessions, 1. Overview, 2. Part A — QA Gates, 3. Part B — M7 Session Track (BL-11), 5. Part D — Remaining Launch Checklist (Not Done, Do Not Fake), 6. Regression Checklist (Copy-Paste per Pass), BL-11 M7 session track (+16 more)

### Community 52 - "Design System Assets"
Cohesion: 0.12
Nodes (26): Design System (Espresso shell, Paper/Sepia/Espresso reading themes), Root Contexts (lang, Session, Toasts, ShellTheme), Service Worker (sw.js: shell and asset caches, /api never cached), admin-sorting-pigeonholes (admin and signed-in shelf empty states), cozy-reader-armchair-owl (guest shelf, reader finale), assets/README.md: Asset Catalog, library-bookshelf-ladder (home hero plate), manuscript-inspection-clothesline (curator desk empty states) (+18 more)

### Community 53 - "Quote Repository Access"
Cohesion: 0.25
Nodes (14): ChunkSearchRow, find_saved_quote_duplicate(), get_chapter_recap(), get_saved_quote_by_id(), get_tldr_cache(), insert_quote(), list_saved_quotes(), QuoteSaveOutcome (+6 more)

### Community 54 - "Quote API Handlers"
Cohesion: 0.21
Nodes (11): card_url_for(), get_quote_card(), handle_list_saved_quotes(), handle_save_quote(), handle_save_quotes_batch(), outcome(), quotes_routes(), render_vintage_quote_svg() (+3 more)

### Community 55 - "Production Deployment Config"
Cohesion: 0.14
Nodes (21): Deployment (docker-compose.prod.yml, make prod-*), Observability (x-request-id spans, LOG_FORMAT=json), Saved Quote Endpoints (/quotes, save-batch, card), Swagger UI and OpenAPI JSON (dev only), docker-compose.prod.yml (production compose), edge service (Dockerfile.edge, Caddy on 80 and 443 TCP and UDP), .env.production.example, minio service (cgr.dev/chainguard/minio, internal only) (+13 more)

### Community 56 - "S3 Storage Service"
Cohesion: 0.20
Nodes (4): StorageConfig, StorageService, StoredObject, AppError

### Community 57 - "App Environment Config"
Cohesion: 0.12
Nodes (15): SmtpSecurity, test_app_config_from_env_clean_execution(), test_config_accessors(), test_default_app_config(), AppConfig::from_env, Finding 21.2: TRUST_PROXY_HEADERS=false shares one guest bucket, Step 21.2: Production boot guard for proxy trust, Step 21.4: Tests for revoked bucket, config guard, counter TTL (+7 more)

### Community 58 - "Auth Integration Tests"
Cohesion: 0.14
Nodes (13): login_req(), provision_verified_user(), refresh_req(), test_auth_concurrent_refresh_keeps_family_alive(), test_auth_email_aggregate_lockout_twenty_failures(), test_auth_pair_lockout_five_failures(), test_auth_password_change_kills_other_session_e2e(), test_auth_refresh_grace_window() (+5 more)

### Community 59 - "SVG Icon Library"
Cohesion: 0.23
Nodes (22): arrow_down(), arrow_right(), back(), brand_mark(), cards(), check(), close(), download() (+14 more)

### Community 60 - "UI Internationalization Module"
Cohesion: 0.11
Nodes (20): FE medium: profile flashes empty copy while loading, 7. Module 7: User Interface Internationalization (i18n), FR-I18N-01: Reactive UI Language Switcher, FR-I18N-02: Manuscript Text Isolation, Module 7: UI Internationalization (i18n), 2026-09-27 — Rotaria P1–P7: Rebrand, i18n, Hero, Gamification, Social, Profile, Admin, F7: guest profile Your Shelf section and CTA, Rust Axum REST API (+12 more)

### Community 61 - "Knowledge Base Maintenance"
Cohesion: 0.12
Nodes (15): 2026-09-28 Twin-Doc Dedup: 5 Files Removed, 2026-09-29 — Knowledge Sync Repair: nested repo, twin dedup, full graph extract, Brain content, Provenance, Safety, Skills, Quad-Layer Agentic Dev Stack (Jev, Graphify, OKF Vault, GBrain), Brain content (+7 more)

### Community 62 - "Badge Repository Access"
Cohesion: 0.13
Nodes (4): redacted_endpoint(), list_badges(), list_user_badges(), seed_default_badges_if_empty()

### Community 63 - "Pytest Browser Fixtures"
Cohesion: 0.13
Nodes (10): api_get(), book_with_chapters(), browser_context_args(), clean_page(), stable_book_id(), call(), test_envelope_and_health(), test_merge_cap_boundary() (+2 more)

### Community 64 - "Reading Progress Sync"
Cohesion: 0.12
Nodes (19): POST /api/v1/auth/refresh, EPUB CFI reading anchor, GET /api/v1/progress/active, Step 3.8: Persist Reading Position, Step 3.9: Retrieve Active Reading Position, pending_sync_queue store, Step 7.12: Pending-Write Queue Drains on Reconnect, clear_token (+11 more)

### Community 65 - "Developer Environment Setup"
Cohesion: 0.12
Nodes (16): Test Layout (tests/it single binary, separate perf and load), playwright, pytest>=8, pytest-xdist>=3, Dev Services via make db-up (Postgres 5433, Redis 6380, MinIO 9005/9006, Mailpit 1025/8025), GUIDE.md: Developer Guide, Prerequisites (Rust wasm target, Trunk, Docker Compose, uv, Playwright venv), make purge-test-debris (janitor for crashed runs) (+8 more)

### Community 66 - "Auth UI Components"
Cohesion: 0.10
Nodes (6): AuthSheet(), friendly_error(), merge_local_quotes(), Mode, active_element(), manage_sheet_focus()

### Community 67 - "Typed Configuration Models"
Cohesion: 0.15
Nodes (6): AppConfig, AuthConfig, DatabaseConfig, EmailConfig, RedisConfig, ServerConfig

### Community 68 - "Account Management Security"
Cohesion: 0.11
Nodes (20): PUT /api/v1/me/password, Step 2.6: Change Password, 50 MB upload cap, change_password, DefaultBodyLimit, delete_me, ensure_book_published, Finding 27.3: Grace-window successor returned without existence check (+12 more)

### Community 69 - "System Hardening ADRs"
Cohesion: 0.13
Nodes (14): API Response Envelope (success/data, success/error), API Error Codes, Strict Script CSP (no inline scripts), Unversioned /api/* Alias Removed, Audit Hardening 2026-10-07 (50 MB uploads, atomic refresh, HSTS), Curator Desk (manuscripts, queue, retention), Earlier Hardening (ADR-20 to ADR-27), Guest Quotes Merged on Sign-In (+6 more)

### Community 70 - "Rate Limiting Middleware"
Cohesion: 0.15
Nodes (14): AI_POLICY, AUTH_POLICY, FALLBACK_CLIENT_IP, forwarded_for_takes_the_first_parseable_hop(), public_policy(), public_policy_takes_cap_from_config(), RateLimitPolicy, request_with_ip() (+6 more)

### Community 71 - "Admin Ingestion Features"
Cohesion: 0.16
Nodes (17): FR-ADM-01: EPUB Upload & Validation, FR-ADM-04: Ingestion Monitor & Retention Analytics, FR-USR-04 Role-Based Access Control, Module 6: Ingestion Pipeline & Admin, 2026-10-08 Tasks 15, 16, 17 Shipped, Redis 7 cache and Redis Streams, US-12 EPUB Upload & Taxonomy, GET /admin/dlq, POST /admin/dlq/{id}/replay (+9 more)

### Community 72 - "Production Launch Checklist"
Cohesion: 0.14
Nodes (18): BL-10 staging checklist (Task 09i), Launch item: caddy validate on both Caddyfiles, Launch item: k6/Criterion for p95 claims (ID-04), Launch item: prod up --build against a real domain, Launch item: R2/CDN cutover, Launch item: tag v0.3.0 (MVP), Part D — Remaining Launch Checklist, Caddyfile.prod (+10 more)

### Community 73 - "Legal and Audit Trail"
Cohesion: 0.15
Nodes (17): Audit Trail Protocol (doc routing by change type), [0.1.0] 2026-09-25, Keep a Changelog 1.0.0, Semantic Versioning 2.0.0, Documentation and Audit Trail Workflow, Audit Trail Protocol, Berne Convention, Disclaimer of Warranty (+9 more)

### Community 74 - "Frontend Reader Engine"
Cohesion: 0.12
Nodes (14): IndexedDB Storage (project_baca_db v3 via rexie), Progress Endpoints (/progress/active, PUT, merge), Reader flip.rs (spine-hinged strips, damped spring), Reader layout.rs (CSS columns one page wide), Reader mod.rs (gestures, keys, anchored position, heartbeat), Responsive Breakpoints (768 px chrome switch, 2/3/4/5 catalog columns), Home Catalog Preview (two rows), Reader as an Open Book (paper page turns) (+6 more)

### Community 75 - "SMTP Email Service"
Cohesion: 0.18
Nodes (9): build_transport(), mailpit_accepts_cleartext_delivery_when_running(), mask_email(), otp_message(), otp_message_addresses_sender_and_recipient(), send_otp_email(), smtp_error(), SMTP_TIMEOUT (+1 more)

### Community 76 - "Component Unit Tests"
Cohesion: 0.22
Nodes (10): _story(), test_atomic_cards_404_tolerant(), test_auth_sheet_modes(), test_curator_desk_tabs_rows_replay_and_funnel(), test_quote_finder_empty_state(), test_quote_finder_error_branch(), test_site_header_toggle_no_reload(), assert_no_page_errors() (+2 more)

### Community 77 - "HTML Sanitization Utility"
Cohesion: 0.12
Nodes (5): SanitizeTest, extract_text(), __init__(), sanitize_html(), _Sanitizer

### Community 78 - "Test Data Seeders"
Cohesion: 0.13
Nodes (5): 2. Quick Data Seeder for Manual Testing, DlqTest, main(), psql(), purge_seed_survivors()

### Community 79 - "Book Cover API"
Cohesion: 0.14
Nodes (5): COVER_CACHE_CONTROL, cover_content_type(), covers_routes(), get_cover(), is_cover_file_name()

### Community 80 - "OTP Security Logic"
Cohesion: 0.17
Nodes (9): generate_and_store_otp(), generate_numeric_otp(), hash_otp(), MAX_OTP_ATTEMPTS, OTP_LOCK_SECONDS, OTP_TTL_SECONDS, OtpRecord, test_hash_otp_consistency() (+1 more)

### Community 81 - "Insight UI Components"
Cohesion: 0.24
Nodes (9): AtomicCards(), CatchupRecap(), InsightList(), InsightText(), QuoteFinder(), share_quote(), Sheet(), string_field() (+1 more)

### Community 82 - "EPUB Parsing Logic"
Cohesion: 0.14
Nodes (9): 28 — Worker follow-ups, 2. Steps, 3. Verify, make_epub(), ParseTest, parse_epub(), ParsedChapter, ParsedEpub (+1 more)

### Community 83 - "Database Entity Models"
Cohesion: 0.12
Nodes (6): Model, Relation, Model, Relation, Model, Relation

### Community 84 - "Navigation Integration Tests"
Cohesion: 0.24
Nodes (11): goto(), test_hero_plate_is_static(), test_home_catalog_shows_two_rows_then_everything(), test_i18n_toggle_and_persist(), test_overview_and_sheets(), test_search_returns_cards(), _section_top(), test_cover_art_never_shows_a_broken_image() (+3 more)

### Community 85 - "UI Design Screenshots"
Cohesion: 0.18
Nodes (17): Atomic Cards NOT_FOUND Unavailable State, Atomic Insights Ch. 1 Modal, Funnel Novel Test Book Overview (behind modal), Dimmed Modal Backdrop Overlay, Monospace Close Button, Legacy Project Baca Header, Audit Screenshot 05: Atomic Insights Modal, Empty Guest Profile State (+9 more)

### Community 86 - "Book Route Handlers"
Cohesion: 0.36
Nodes (6): books_routes(), get_book(), get_chapter(), get_offline_bundle(), list_books(), search_books()

### Community 87 - "Data Philosophy Documentation"
Cohesion: 0.15
Nodes (16): books(), 1. Engineering Invariants & Data Philosophy, 2. Entity Relationship Diagram, 3. Data Dictionary & Table Specifications, 4. Index Matrix & Query Patterns, 5. Migration History (`migrations/`), A. Identity & Auth Cluster, B. Catalog & Taxonomy Cluster (+8 more)

### Community 88 - "User Repository Access"
Cohesion: 0.37
Nodes (8): activate_user_by_email(), create_inactive_user(), delete_user_by_id(), find_user_by_email(), find_user_by_id(), update_inactive_credentials(), update_user_password(), update_user_profile()

### Community 89 - "Gamification API Handlers"
Cohesion: 0.34
Nodes (6): gamification_routes(), get_my_streak(), HEARTBEAT_MIN_INTERVAL_SECS, list_badges(), list_user_badges(), record_heartbeat()

### Community 90 - "Insight Route Handlers"
Cohesion: 0.29
Nodes (4): get_atomic_cards(), get_chapter_recap(), insights_routes(), resolve_chapter()

### Community 91 - "OWASP Security Tests"
Cohesion: 0.23
Nodes (11): test_owasp_login_brute_force_lockout(), test_owasp_otp_cooldown_and_email_bombing_prevention(), test_owasp_rate_limiting_headers_and_ip_extraction(), test_owasp_revoke_all_sessions(), test_owasp_security_headers_and_error_handling(), test_owasp_structured_validation_error_details(), test_owasp_timing_attack_mitigation_on_login(), test_owasp_token_revocation_on_logout() (+3 more)

### Community 92 - "Accessibility Audit Tests"
Cohesion: 0.28
Nodes (9): _assert_clean(), _scan(), test_axe_admin(), test_axe_home(), test_axe_overview(), test_axe_paper_surface(), test_axe_profile_guest(), test_axe_quote_modal() (+1 more)

### Community 93 - "Worker Unit Tests"
Cohesion: 0.16
Nodes (5): ArchiveLimitsTest, ChunkTest, check_archive_limits(), chunk_words(), _opf_path()

### Community 94 - "Password Hashing Logic"
Cohesion: 0.31
Nodes (10): DUMMY_ARGON2_HASH, hash_password(), hash_password_async(), test_async_hash_and_verify(), test_dummy_argon2_hash_validity(), test_hash_and_verify_password_success(), test_verify_password_invalid_hash(), test_verify_password_wrong_password() (+2 more)

### Community 96 - "Progress Repository Access"
Cohesion: 0.35
Nodes (7): check_and_award_badges(), get_active_progress(), get_streak(), record_heartbeat(), seconds_read_on(), update_progress(), ReadingHeartbeatResponse

### Community 97 - "Catalog Integration Tests"
Cohesion: 0.26
Nodes (10): seed_book_with_chapter(), seed_test_catalog(), SeededCatalog, test_book_overview_chapter_and_offline_bundle(), test_catalog_listing_and_filtering(), test_catalog_typo_tolerant_fts_search(), test_gamification_heartbeat_and_badges(), test_heartbeat_credit_clamped_and_gate_per_user() (+2 more)

### Community 98 - "Visual Regression Testing"
Cohesion: 0.25
Nodes (6): assert_golden(), test_visual_admin(), test_visual_home(), test_visual_overview(), test_visual_profile_guest(), test_visual_reader()

### Community 99 - "Build Script Utilities"
Cohesion: 0.20
Nodes (6): 2026-09-27 — Milestone 05 Complete: Ingestion Pipeline & Admin (ADR-19), check(), externalize(), replace(), _is_inline(), main()

### Community 100 - "Retry Logic and Backoff"
Cohesion: 0.16
Nodes (3): RetryAndRecapTests, backoff_seconds(), build_recap_input()

### Community 102 - "Toast Notification System"
Cohesion: 0.24
Nodes (3): Toast, Toasts, ToastStack()

### Community 103 - "Session and Auth Tests"
Cohesion: 0.27
Nodes (7): _count_local_quotes(), _mailpit_otp(), _post(), _put_local_quote(), test_authed_autosave_shelf_cleanup(), test_guest_profile_and_admin_guard(), test_guest_quote_merges_on_register()

### Community 104 - "Manual Testing Documentation"
Cohesion: 0.18
Nodes (4): 1. Directory Structure & Milestone Modules, 2. Server & Service URLs Quick Reference, 3. General Testing Workflow, Project Baca — Manual Testing Documentation

### Community 105 - "Gamification and Heartbeat"
Cohesion: 0.18
Nodes (8): ADR-22, Step 2.8b: Verify Refresh Tokens Are Stored Hashed, POST /api/v1/activity/heartbeat, Step 3.10: Send Reading Heartbeat, Step 3.12: View User Unlocked Badges, Step 6.10: Heartbeat + Streak Toast, Step 21.3: Pipeline SET NX EX then INCR, Step 27.2: OTP attempts as a separate INCR/EXPIRE key

### Community 106 - "Book Entity Models"
Cohesion: 0.18
Nodes (3): Entity, Model, Relation

### Community 107 - "Rate Limit Middleware"
Cohesion: 0.45
Nodes (5): ai_rate_limit_middleware(), enforce(), extract_client_ip(), public_rate_limit_middleware(), rate_limit_middleware()

### Community 110 - "API Data Transfer Objects"
Cohesion: 0.22
Nodes (5): ApiResponse<T>, AtomicCardsDto, ChapterRecapDto, atomic_cards(), chapter_recap()

### Community 111 - "Book Card Component"
Cohesion: 0.31
Nodes (5): BookCard(), Cover(), cover_tint(), COVER_TINTS, SkeletonCard()

### Community 112 - "Reader Interaction Tests"
Cohesion: 0.42
Nodes (5): _drag(), _open_long_chapter(), test_drag_turns_like_paper(), test_keyboard_turns_and_dims(), test_reduced_motion_turns_without_paper()

### Community 114 - "Reader Enhancement Tasks"
Cohesion: 0.25
Nodes (6): 1. Goal, 23 — Reader Enhancements, 2. Verify, Board, Order, Task Board

### Community 119 - "Progress UI Components"
Cohesion: 0.38
Nodes (3): ProgressBar(), ProgressRing(), RING_RADIUS

### Community 120 - "Reader Flow Tests"
Cohesion: 0.52
Nodes (4): _chapter_no(), test_end_of_book_notice_is_visible(), test_offline_save_read_badge(), test_reader_body_and_type_sheet()

### Community 121 - "Database Migration Scripts"
Cohesion: 0.76
Nodes (6): init_table(), migrate_down(), migrate_status(), migrate_up(), psql_cmd(), migrate.sh script

### Community 128 - "App State Configuration"
Cohesion: 0.60
Nodes (6): AppConfig, AppState, AppState as central Axum dependency-injection bridge, get_book, Graphify query: trace data path and dependencies across architectural boundaries, TestHarness

### Community 129 - "Guest Progress Merging"
Cohesion: 0.33
Nodes (3): user_reading_progress table, POST /api/v1/progress/merge, Step 2.9: Reconcile Guest Progress

### Community 130 - "Knowledge Base Index"
Cohesion: 0.33
Nodes (5): 1. Directory Role & Responsibility Map, 2. Core Specification Documents, 3. Progressive Disclosure Hierarchy, 4. Engineering Invariants, Knowledge Catalog Index & Directory Architecture Map (OKF v0.2)

### Community 131 - "Tauri Shell Documentation"
Cohesion: 0.33
Nodes (5): 1. Goal, 22 — Tauri Shell, 2. Preconditions from the web build, 3. Steps, 4. Verify

### Community 132 - "Gemini API Integration"
Cohesion: 0.40
Nodes (6): embed_batch, Finding 28.1: Gemini key sent in the URL, Finding 28.7: Read-phase TimeoutError skips retry loop, _gemini_post, generate_json, Step 28.7: Catch TimeoutError in the retry loop

### Community 134 - "Graph Query Responses"
Cohesion: 0.40
Nodes (4): Answer, Outcome, Q: nah pertanyaanmu terkait graf seriusan ga paham, apalagi across architetureal boundaries gitu. tolong deh telusuri jalur data dan dependencynya., Source Nodes

### Community 135 - "Launch and Trademark"
Cohesion: 0.40
Nodes (4): 1. Goal, 20 — Launch Remaining and Trademark, 2. Steps, 3. Verify

### Community 136 - "Rate Limit Follow-ups"
Cohesion: 0.40
Nodes (4): 1. Findings (audit 2026-10-07), 21 — Rate-Limit Follow-ups, 2. Steps, 3. Verify

### Community 137 - "Responsive Design Follow-ups"
Cohesion: 0.40
Nodes (4): 1. Findings (viewport matrix, 2026-10-08), 24 — Responsive Follow-ups, 2. Steps, 3. Verify

### Community 138 - "Redis Resilience Follow-ups"
Cohesion: 0.40
Nodes (4): 1. Findings (audit 2026-10-08, round 2), 26 — Redis reconnect, boot fail-fast, outbound timeouts, 2. Steps, 3. Verify

### Community 139 - "Auth System Follow-ups"
Cohesion: 0.40
Nodes (4): 1. Findings (audit 2026-10-08, round 2), 27 — Auth follow-ups, 2. Steps, 3. Verify

### Community 140 - "Infrastructure Hygiene"
Cohesion: 0.40
Nodes (4): 1. Findings, 29 — Edge and compose hygiene, 2. Steps, 3. Verify

### Community 141 - "WASM Bundle Optimization"
Cohesion: 0.40
Nodes (4): 1. Measurement (2026-10-08, `trunk build --release`), 2. Steps, 30 — Slim the WASM bundle, 3. Verify

### Community 142 - "Reader Position Integrity"
Cohesion: 0.40
Nodes (4): 1. Findings (audit 2026-10-08, round 2), 2. Steps, 31 — Reader position integrity, 3. Verify

### Community 143 - "API Client Follow-ups"
Cohesion: 0.40
Nodes (4): 1. Findings (audit 2026-10-08, round 2), 2. Steps, 32 — Session and API client follow-ups, 3. Verify

### Community 144 - "Accessibility Polish"
Cohesion: 0.40
Nodes (4): 1. Findings (audit 2026-10-08, round 2), 2. Steps, 33 — Shell and accessibility polish, 3. Verify

### Community 145 - "Service Worker and Fonts"
Cohesion: 0.40
Nodes (4): 1. Findings (audit 2026-10-08, round 2), 2. Steps, 34 — Service worker and fonts, 3. Verify

### Community 146 - "Project Directory Structure"
Cohesion: 0.70
Nodes (5): domain, infra, server, shared, web

### Community 158 - "Offline Content DTOs"
Cohesion: 0.50
Nodes (4): ChapterDetailDto, OfflineBundleDto, chapter(), offline_bundle()

## Knowledge Gaps
- **484 isolated node(s):** `Relation`, `Relation`, `Relation`, `Relation`, `Relation` (+479 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 1040 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **127 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `2026-09-28 M7 Sequential Execution: Hardening + BL-10 + BL-11 + BL-12` connect `Security Audit Reports` to `Project Documentation Index`, `Security Audit Findings`, `EPUB Parsing Logic`, `Reader Functional Requirements`?**
  _High betweenness centrality (0.119) - this node is a cross-community bridge._
- **Why does `GET /api/v1/health` connect `Security Audit Reports` to `Production Launch Checklist`, `QA Launch Checklist`, `Infrastructure Setup Guide`?**
  _High betweenness centrality (0.107) - this node is a cross-community bridge._
- **Why does `Task 23: Reader enhancements` connect `Reader UI Enhancements` to `Web Frontend Testing`, `Offline Sync API`, `Insight UI Components`, `Reader Flow Tests`, `Reader Layout Engine`, `Catalog API Endpoints`, `App Environment Config`?**
  _High betweenness centrality (0.094) - this node is a cross-community bridge._
- **What connects `Relation`, `Relation`, `Relation` to the rest of the system?**
  _484 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Project Documentation Index` be split into smaller, more focused modules?**
  _Cohesion score 0.05741626794258373 - nodes in this community are weakly interconnected._
- **Should `Security Audit Findings` be split into smaller, more focused modules?**
  _Cohesion score 0.055135135135135134 - nodes in this community are weakly interconnected._
- **Should `Job Queue Infrastructure` be split into smaller, more focused modules?**
  _Cohesion score 0.07163461538461538 - nodes in this community are weakly interconnected._