# Changelog — Project Baca

All notable changes are documented chronologically following [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Reader: a chapter that fails to load no longer moves the chapter number, the URL or the saved position; the reader stays on the chapter on screen and says "Bab ini belum bisa dimuat", and a book that never loaded cannot reach the finale. Progress queued while the API was unreachable goes out when the app starts signed in.
- Queued progress belongs to the account that wrote it: signing out (or a refused session) drops that account's unsent writes, and a drain sends only the signed-in account's writes, so a shared device never moves one reader's position into another's shelf.
- Reader: resizing the window or rotating the phone keeps the paragraph being read on screen; the position is held as a paragraph and the share of it already read, applied once the new column width is laid out, instead of drifting back about a page per change.

## [0.1.0] — 2026-11-03

First release: the reader, the curator desk, the ingestion worker, and the deployment stack as they stand on this date. A 0.y version is initial development under semantic versioning; the public launch waits on the launch gate (staging domain, live email, trademark search).

### Added
- Tauri 2 app shell (`apps/tauri`) for Android and desktop around the same web bundle: the API base is injected at run time, no service worker inside the app, `rotaria://book/<id>` links open the book (also on a cold start), and Android keeps the page between the system bars. Verified on an Android 16 emulator: catalog from the host API, a book opened by deep link, page turns by swipe, and reading a saved book in airplane mode. `make app-dev`, `app-build`, `app-test`, `app-android-dev`, `app-android-build`.
- Reader tools that never leave the book: a chapter drawer from the bar (where you stand, jump without reload), a closing card the first time each chapter ends (insight cards or the next chapter), "Save quote" for selected text (account or device), and search inside the book that opens the page holding the hit, online through `GET /books/{id}/search` and offline from the saved copy. The last page of a chapter says so instead of "~0 minutes left".
- Reader as an open book: paper frame with running heads and folios; pages turn as spine-hinged strips driven by a damped spring from a drag, tap, or key; one-column clones keep a turn cheap; the last page resists and a finale card closes the book; center tap dims the bars; haptic on landing; reduced motion keeps a crossfade; the chapter title links to the chapter list.
- Home catalog preview: two rows (2x2 to 2x5 by width) and a "see the whole catalog" button; search and filters show everything with paging.
- Idle illustrations placed: clothesline (empty curator lists), pigeonholes (empty signed-in shelf), rocket (404 and empty search), owl (finale).
- Navigation markers: `aria-current` from hash and query, header dot, "Cara kerja" arrow, "Rak saya", a seven-stroke brand mark in the header and as favicon, system color scheme on first visit.
- Cover delivery: `GET /api/v1/covers/{file}` streams from the covers bucket with an immutable cache; covers render as CSS backgrounds with a typographic fallback.
- Curator desk: manuscripts (all statuses, counts, filters, archive, upload with a live job card), queue (dead-letter replay), retention (drop-off funnel). Endpoints `GET /admin/books`, `PATCH /admin/books/{id}`, `GET /admin/dlq`, `POST /admin/dlq/{id}/replay`.
- Worker resilience: Gemini retries with quadratic backoff and `Retry-After`; recap built from prior key concepts within the prompt budget; `tldr_cache` upsert on replay.
- Per-device sessions: `GET /me/sessions`, `DELETE /me/sessions/{id}`, `POST /auth/revoke-all?keep_current=true`; the shelf lists devices with per-row sign-out.
- Guest quotes kept on the device (IndexedDB v3) and merged on sign-in through `POST /quotes/save-batch` (up to 50, per-item outcome).
- Test isolation: `TestHarness` tracks seeded rows and `cleanup()` restores counts; `make seed-dev` installs four CC0 works with fixed ids; the janitor sweeps survivors.
- Rotaria redesign and responsive pass (2026-10-07 and 08): two-surface design system, phone tab bar, bottom sheets, action dock, paginated reader with type sheet, shelf with streak and sessions, table-driven ID/EN dictionary, WebP illustrations, safe-area insets, five-column catalog from 1440 px.
- `GET /api/v1/me/streak`; `PUBLIC_RATE_LIMIT_PER_MINUTE` public read cap on the shared limiter engine; functional server suites consolidated in `crates/server/tests/it` with lean debug profiles.
- Milestones 04 to 07 (2026-09-27 and 28): dual-mode embeddings and scoped HNSW quote search; atomic cards and recaps from `tldr_cache`; saved quotes with SVG cards; MinIO storage, admin upload (202), Python worker with DLQ and reclaim; job monitor and drop-off funnel; Leptos web reader with five routes; 401 auto-refresh; PWA manifest, service worker, offline reader, pending-write queue; production images, `docker-compose.prod.yml`, `Caddyfile.prod`, `.env.production.example`; `crates/domain` with DB-free tests; `make purge-test-debris`; `scripts/pg_backup.sh`.

### Changed
- WebAssembly bundle down from 2.83 MB to 1.47 MB raw (909 KB to 573 KB gzip; 1.57 MB and 608 KB in this release with the reader tools): a `wasm-release` Cargo profile and `wasm-opt -Oz` in `make build-web`, which the edge image also runs; the build fails above 1.8 MB raw or 700 KB gzip. Unused `web-sys` features dropped.

### Removed
- Documentation consolidated (2026-10-09): `DISTRIBUTED.md`, `assets/README.md`, and `python_worker/README.md` folded into `ARCHITECTURE.md`; one agent rule file (`AGENTS.md`).
- The unversioned `/api/*` alias; `/api/v1/*` is the only API surface.

### Fixed
- Phone home: the "picks" rail started flush with the screen edge instead of the content edge (scroll snapping ignored the rail's padding), and cards with long titles grew wider than the rest; the search tab no longer leaves the focused field under the sticky header (the scroll was aimed before the page settled).
- Sheets: closing one by a click no longer throws "closure invoked after being dropped" twice (the click still bubbled to listeners the close had freed; the close now waits for the dispatch to end); sheets opened over the reader keep the shell's ink instead of the page's; search forms inside sheets keep their one-line pill; ghost buttons on the reader's finale card take the page's ink.
- Dual-screen devices: a window spanning two hinged screens reads as a spread with the spine on the hinge; below 1024 px it used to run one column straight through the hinge. The responsive matrix now also runs in Indonesian (the suite browser is English, so ID copy had never been checked) and emulates hinges through Chromium display features.
- Shell accessibility: Tab and Shift+Tab stay inside open sheets; the focused search field clears the sticky header on phones; the error toast passes AA on Paper (it was 2.85:1); English counts read "1 chapter", "1 day in a row", "1 reader"; the theme toggle names the theme it switches to and the shell follows a system scheme change until a theme is chosen; running heads and tab labels grow to 0.7 and 0.75 rem; the empty shelf promises the first badge after the first minute, as the badge rule grants it.
- Deploys reach returning readers: the service worker serves unhashed files (boot.js, icons, illustrations, manifest) network-first and keeps one shell entry instead of one per visited URL; `Caddyfile.prod` stops marking unhashed files immutable and no longer touches the cache policy of API responses. The three house fonts are self-hosted, so the app boots and keeps its typography without Google Fonts, offline included; the CSP drops both Google origins.
- Session client: a signup refused by the cooldown or an outage stays on the form with the message; a refused refresh token returns the UI to the guest view instead of an avatar over an endless skeleton; signing out offline sticks after a reload; the pending-write drain runs once at a time and keeps writes queued during it; a rejected 401 body is drained so repeated expiries cannot exhaust the browser's connections to the API.
- Reader position: reopening a book lands on the saved page even when the position arrives after the chapter, and page one no longer overwrites it; a late answer for a chapter already left is dropped; leaving or closing the tab inside the save debounce keeps the page just turned; the spread starts at a 1024 px window; Space on a focused control activates it; pinch zoom works; the chapter title link scrolls to the chapter list.
- Ingestion worker: dead-worker recovery works on Redis 7 (`XAUTOCLAIM` answers with three values and was never unpacked); a running job keeps its claim, so a long book is not ingested twice; SIGTERM releases the active job within the stop grace; replaying a dead letter is atomic; archived books are never republished; oversized EPUB metadata is clipped instead of failing the job; read timeouts are retried; the Gemini key moved from the URL to `x-goog-api-key`.
- Redis restarts no longer break login and bearer routes until an API restart: one reconnecting handle serves every request, an outage answers 503 instead of 401, and the next request after recovery reconnects. In production the server exits when Postgres or Redis is unreachable at boot; S3 calls stop after 15 s; request-id spans no longer leak across awaits.
- Anchors scroll after content loads and stop under the sticky header; the search tab focuses the field; sheets close on Escape and return focus; the upload input is keyboard-reachable; remaining aria labels translated.
- Signals read after awaits use `try_get`; toasts dedupe with `role=alert` for errors; catalog, book, and reader show a network error with retry; the admin poll stops on unmount; shelf skeletons; service worker shell fallback and asset pruning; Paper contrast passes AA.
- Reader root used `min-height` and never paginated long chapters; it is a fixed-height flex column; insight sheets fetch on open; measure capped at 1440 px.
- Quote API contracts: unknown book 404, invalid (book, chapter) pair 400, saving and cards require the owner.
- Save Offline key mismatch (`book_id` vs `id`, IndexedDB v2) and typed chapter records; shared header and button styles; hero dot labels and faint contrasts.
- Docs and comments: spec tracers removed from code, SSOT corrections in SRS and ERD, history re-sorted newest first.

### Security
- Deploy surface trimmed: dev compose ports bind to `127.0.0.1`; `Caddyfile.prod` drops the `Server` banner on API and error responses; production refuses `SMTP_SECURITY=none`; server and worker images run as uid 10001 (the server image gains the `wget` its healthcheck calls); ingestion and dead-letter streams are capped at about 10000 entries.
- Rate limits cannot be gamed or stuck: every counter is `INCR` plus `EXPIRE NX` in one transaction (auth, AI, public, signup, lockouts, OTP); a revoked token counts in the guest AI bucket; production refuses to boot with a public cap but no proxy trust, and configuration errors exit with a message instead of a panic.
- Account takeover and abuse paths closed: a signup with a different password cannot replace the password of a registration whose code is pending (429 until it expires); OTP attempts are counted atomically, so parallel guesses cannot pass the three-attempt limit; the refresh grace window refuses a successor killed by revoke-all; password change revokes sessions before writing the new hash; emails are trimmed and lowercased, and a concurrent duplicate signup answers 409; progress writes and the active shelf ignore draft and archived books; only the EPUB upload route accepts bodies above 2 MB.
- Client address comes from one header: with `TRUST_PROXY_HEADERS=true` only `TRUSTED_IP_HEADER` (default `x-forwarded-for`) is read; `Caddyfile.prod` strips the other address headers. The default JWT secret is refused whenever proxy headers are trusted.
- Worker sanitizer re-escapes decoded character references, so markup hidden in entities stays text.
- Draft and archived books answer 404 on chapter, insight, and offline-bundle routes.
- SMTP through `lettre` with STARTTLS (`SMTP_SECURITY=starttls|none`); Gemini key sent in `x-goog-api-key`; connection strings, recipients, and OTPs masked in logs.
- Swagger and OpenAPI unmounted in production; `/health` answers 503 without a database; `Cache-Control: no-store` on auth and bearer responses; validation errors stop echoing values; search queries capped at 200 characters.
- Logout blacklists the session's last access token; heartbeat gate is one atomic `SET NX EX`; a duplicate quote save returns the existing row; `PATCH /admin/books/{id}` to `processing` is 409.
- Strict script CSP: a Trunk post-build hook externalizes the inline bootstrap, `script-src` drops `unsafe-inline`, `make test-web-full` fails on any inline script.
- OTP lock answers 429 with `Retry-After` after three wrong codes.
- Audit hardening (2026-10-07): 50 MB EPUB uploads past axum's default body limit; atomic refresh rotation (`GETDEL`, 30 s grace); unverified accounts behave like a wrong password; signup fails closed on SMTP (502); password change returns a fresh pair; heartbeat credit clamped to 90 s per user; admin guard re-reads the user row; HSTS on the API; SPA CSP in both Caddyfiles.
- Earlier hardening (ADR-20 to ADR-27): SHA-256 refresh storage with family revocation on replay; two-layer login lockout; signup SMTP cap; OTP lock without deletion; fail-closed 503 on Redis outage for auth and AI paths; EPUB magic check and worker zip guard; CORS allowlist; server-minted request ids; avatar URL allowlist; guest merge transactional with a 100-record cap.

### Foundations (2026-09-25 to 2026-09-28, before any release)
These milestones were once headed 0.1.0, 0.2.0, and 0.2.1 but never shipped; they are folded in here.
- Workspace of five crates (`domain`, `shared`, `infra`, `server`, `web`), typed configuration from the environment, SeaORM entities and seven paired migrations, PostgreSQL 17 with pgvector and `pg_trgm`, Redis 7, MinIO, and Mailpit in the dev compose.
- Authentication: signup with a six-digit OTP stored as SHA-256 in Redis, Argon2id passwords off the async runtime, JWT access tokens with rotating refresh tokens, logout and revoke-all, guest progress merge.
- Catalog with cursor paging, trigram and FTS search, chapter delivery, offline bundles, reading progress anchors, streak heartbeats, and badges.
- Caddy as the TLS and HTTP/3 edge in front of a plain TCP Axum service; OpenAPI through utoipa; OWASP-oriented headers, rate-limit headers, and login lockouts.
- Corrections to what those headings claimed: email is sent directly over SMTP (no Redis Streams email queue), and there are no CSRF cookies, since the API takes bearer tokens only.
