# Architecture

## 1. Crates and layering

```text
crates/domain    pure rules, no I/O: Percentage, BookStatus lifecycle, streak, badges, progress merge
crates/shared    DTOs, validation, error contract; compiled native and to WebAssembly
crates/infra     AppConfig from env, SeaORM pool and repositories, Redis, MinIO, SMTP (lettre), Gemini client
crates/server    Axum router, middleware, handlers, OpenAPI (utoipa)
crates/web       Leptos 0.7 client-side app, Trunk build, service worker
python_worker    ingestion consumer (section 6)
migrations       paired .up.sql / .down.sql (7 pairs, 13 tables)
```

Dependency direction: `server` -> `infra` -> `domain`; `shared` is used by all. Repositories call domain functions for rule evaluation; `server` never imports `domain`. Repository traits are not introduced until a second implementation exists.

## 2. Request path

Caddy terminates TLS and HTTP/3, serves the SPA from `dist/` with security headers and an immutable cache for hashed assets, proxies `/api/*` to Axum, and strips every client-address header except `X-Forwarded-For`, which it overwrites.

Axum layers, outermost first: tracing, CORS (`CORS_ALLOWED_ORIGINS`), server-minted `x-request-id`, security headers. Per-router limiters on one Redis fixed-window engine (`middleware/rate_limit.rs`; every counter, lockouts included, is `INCR` plus `EXPIRE NX` in one transaction, so no key outlives its window): auth 20/min per client; AI 10/min per user or guest client (a revoked token counts as a guest) and `AI_DAILY_BUDGET` per day across everyone, guests together at most a fifth of it; writes on `/me`, `/progress`, `/quotes` 120/min per account (`rl:user:{id}`; reads and requests without a live token pass); public reads `PUBLIC_RATE_LIMIT_PER_MINUTE` per client on books, covers, insights, and gamification when the value is above 0. A client is its IPv4 address, or the /64 of its IPv6 address (an IPv4-mapped address counts as IPv4), so rotating addresses inside one customer block buys nothing; the signup and login caps key on the same subject, while session records keep the raw address. The admin router raises the body limit to 50 MB plus 64 KiB; the upload handler enforces 50 MB while streaming. One reconnecting Redis handle (`ConnectionManager`, no internal retries, 1 s connect and 3 s response timeouts) serves every request: during an outage auth, AI, and bearer routes answer 503 (fail closed, so clients keep valid tokens), and the first request after recovery reconnects. S3 calls are capped at 15 s including SDK retries. Every API statement is capped by `DATABASE_STATEMENT_TIMEOUT_MS` (5,000 ms, a startup option of each pooled connection, above the 2 s lock wait of a status change), so one slow query cannot hold a connection; the worker's connections are not capped.

## 3. API surface

All paths below are under `/api/v1` unless noted. Responses use `{ "success": true, "data": ... }` or `{ "success": false, "error": { "code", "message", "details" } }`; codes: `BAD_REQUEST`, `VALIDATION_FAILED`, `UNAUTHORIZED`, `FORBIDDEN`, `NOT_FOUND`, `CONFLICT`, `RATE_LIMITED`, `AI_RATE_LIMITED`, `DATABASE_ERROR`, `INTERNAL_ERROR`, `EXTERNAL_SERVICE_ERROR` (502), `SERVICE_UNAVAILABLE` (503).

| Method and path | Auth | Notes |
|---|---|---|
| `GET /health` (root) | none | 503 `degraded` when the process has no database connection |
| `GET /health` | none | Postgres and Redis status; answers 200 even when `degraded` (issue #51) |
| `POST /auth/signup`, `/auth/verify-otp`, `/auth/login`, `/auth/refresh` | none | auth limiter; signup 502 when SMTP is down, 10/h per client, codes per address 3/h and 10/day, 503 past `OTP_HOURLY_BUDGET` across everyone; OTP: three wrong codes lock that registration for 5 min, 10 guesses per address per day (429, `Retry-After`) |
| `POST /auth/logout`, `/auth/revoke-all?keep_current=` | bearer | logout revokes every access token of the session; revoke-all drops other devices |
| `GET`, `PATCH`, `DELETE /me` | bearer | profile; delete revokes first, then cascades |
| `PUT /me/password` | bearer | revokes other sessions and returns a fresh token pair |
| `GET /me/sessions`, `DELETE /me/sessions/{id}` | bearer | per-device sessions, caller flagged |
| `GET /me/badges`, `GET /me/streak` | bearer | gamification standing |
| `GET /books`, `/books/search`, `/books/{id}`, `/books/{id}/chapters/{n}`, `/books/{id}/offline-bundle` | none | published books only; cursor paging; `q` capped at 200 chars, filters at column width |
| `GET /books/{id}/search?q=` | none | paragraphs of one published book containing `q` (2 to 200 chars, LIKE wildcards literal), in reading order, at most 50, each with chapter, `p-N` index, and a snippet built by `shared::text_hit_snippet` (the offline reader search uses the same function); reads `chapter_paragraphs` through a trigram index on its folded text (no-hit search on an 8 MB book: median 8 ms); `BOOK_SEARCH_RATE_LIMIT_PER_MINUTE` per client (production 30) |
| `GET /covers/{file}` | none | `<uuid>` plus webp, jpg, jpeg, or png from the covers bucket; the content type comes from that extension, never from the stored object; `Cache-Control: public, max-age=31536000, immutable` |
| `GET /books/{id}/chapters/{ref}/atomic-cards`, `.../recap` | none | `ref` is a chapter number or chapter UUID; 404 for unpublished books |
| `POST /books/{id}/quotes/search` | optional bearer | AI limiter; HNSW cosine search scoped `WHERE book_id = $1` |
| `POST /quotes/save`, `POST /quotes/save-batch`, `GET /quotes`, `GET /quotes/{id}/card` | bearer | the text must be in its chapter (folded, spaces ignored, so a selection across paragraphs and a quote-search passage both match), else 400 `VALIDATION_FAILED` or a `rejected` batch item; duplicate save returns the existing row with 200 `saved:false`; batch up to 50 with per-item outcome; at most 5,000 quotes per account, checked under the user's row lock (409 `QUOTA_EXCEEDED`, a batch past it saves nothing); `GET /quotes?cursor=&limit=` pages newest first by keyset on `created_at, id` (default 50, at most 100); card is owner-only SVG |
| `GET /progress/active`, `GET /progress/reading`, `PUT /progress/{book_id}`, `POST /progress/merge` | bearer | published books only (404 otherwise; active and reading skip books unpublished later); reading lists every unfinished book, newest first, at most 100; merge is transactional, capped at 100 records, `GREATEST` percentage |
| `POST /activity/heartbeat`, `GET /me/streak` | bearer | one rewarded heartbeat per user per 55 s (atomic `SET NX EX`; the reader beats every 60 s); another inside the window answers 200 `credited: false` with the standing unchanged; credit clamped to 90 s; days counted in the reader's `utc_offset_minutes` (-720 to 840, default UTC); a newly unlocked badge adds its `xp_reward` in the statement that inserts it |
| `GET /badges` | none | badge catalog |
| `GET /admin/books`, `POST /admin/books/upload`, `PATCH /admin/books/{id}`, `GET /admin/jobs/{id}`, `GET /admin/dlq`, `DELETE /admin/dlq/{id}`, `POST /admin/dlq/{id}/replay`, `GET /admin/analytics/drop-off` | bearer, role `admin` in the token claim and in the current user row (demotion is immediate, promotion needs a fresh token) | upload: `.epub`, 50 MB (the only route above axum's 2 MB body default; past it 413 `PAYLOAD_TOO_LARGE`), 202 with job id and a stream event, title and language left blank for the worker to read from the EPUB; status changes follow `BookStatus::can_transition_to` (409 otherwise, and 409 when the row stays locked past 2 s); an archived book is published again only if its ingestion had finished; a job carries `error` and `attempts`; the DLQ lists the book title and the stream total, and an entry can be dismissed |
| `/swagger-ui`, `/api-docs/openapi.json` (root) | none | not mounted when `APP_ENV=production`; no edge route |

Unknown paths answer 404 in the error envelope; the unversioned `/api/*` alias no longer exists. Rejections axum raises before a handler runs (malformed JSON, missing content type, unparsable path or query, a wrong method) are wrapped into the same envelope by one layer (`error::envelope_rejections`), keeping their status: 400 `BAD_REQUEST`, 415 `BAD_REQUEST`, 422 `VALIDATION_FAILED`, 405 `METHOD_NOT_ALLOWED`. Text with a NUL byte answers 400. The OpenAPI document adds the shared error answers to every operation in one modifier (`ErrorResponsesAddon`) and carries the validators' bounds in the schemas.

## 4. Security model

- Passwords: Argon2id (19 MiB, 2 iterations). Registration: each signup is its own record in Redis under a random `registration_id` (code SHA-256, name, password hash, 10 min TTL); `verify-otp` needs that id and writes the account row from that record only, so a second signup for an address neither blocks nor replaces the first. OTP attempts are counted with an atomic `INCR` before the comparison; three misses lock the code for 5 min.
- Signup: emails are trimmed and lowercased at deserialization, so case variants are one account and one lockout key; the account row is written with one `INSERT ... ON CONFLICT (email) DO UPDATE ... WHERE NOT is_active`, so two registrations of one address verified at once create one account and the other answers 409; guesses at a locked code spend nothing of the daily budget.
- Tokens: HS256 access token (1440 min in dev, 60 min in production compose); refresh token stored as a SHA-256 hash with a session record (device label, IP prefix, last access `jti`), rotated atomically (one Lua script consumes the old token and stores its successor with its session record, so a family revocation, also one script, cannot miss it and a concurrent tab always lands in the same session; 30 s grace for concurrent tabs, handing back the successor only while it is still live); a replayed token revokes the family including access tokens. Access tokens carry a session id (`sid`) that survives refreshes; logout and per-device revocation write `revoked_sid:{sid}` for as long as the session's access or refresh tokens could live; the bearer extractor and `/auth/refresh` both check it, so every token of that device dies.
- Lockouts: login attempts are counted with `INCR` before Argon2, 5 per (email, IP) and 20 per email per 15 min, and a failure that spends a budget locks it 15 min from then; Argon2 runs on every path (a dummy hash when there is none), so unknown, unverified and wrong-password answers look and time the same.
- Client address: with `TRUST_PROXY_HEADERS=true` only `TRUSTED_IP_HEADER` is read; `Caddyfile.prod` sets `X-Forwarded-For` and strips `X-Real-IP`, `CF-Connecting-IP`, `True-Client-IP`, `Forwarded`. The default JWT secret is refused in production or whenever proxy headers are trusted, and production refuses to boot without `TRUST_PROXY_HEADERS=true` and `TRUSTED_IP_HEADER` (every reader would share one bucket, and anyone could lock auth for everyone), with `OTP_HOURLY_BUDGET` or `AI_DAILY_BUDGET` at 0, and with `SMTP_SECURITY=none`.
- Headers: the API sets HSTS, `nosniff`, frame denial, referrer policy, COOP and CORP, a permissions policy, its own CSP, and `Cache-Control: no-store` on `/auth/*` and on every bearer response. Caddy sets the SPA headers, including a CSP with `script-src 'self' 'wasm-unsafe-eval'` and no inline scripts (a Trunk post-build hook moves the bootstrap into a hashed `boot-*.js`).
- Data exposure: draft and archived books are invisible on every public route, covers included, and answer the same 404 as an id that never existed (a takedown still leaves copies in browser and edge caches for up to the one-year cover lifetime); signup answers an address that already has an account like a new one and mails its owner a note instead of a code; validation errors name the field without echoing the value; logs mask connection strings, recipients, OTPs, and the Gemini key (sent in `x-goog-api-key`).
- Open work, security included, is tracked as GitHub issues on the Rotaria Roadmap project; vulnerabilities that are not yet public go to a private security advisory.

## 5. Web app (`crates/web`)

- Routes: `/`, `/book/:id`, `/me`, `/admin`, and `/__gallery` (component stories for tests) under `ShellLayout` (header, phone tab bar, auth sheet, toasts); `/read/:id` renders without chrome; unknown paths show the 404 page.
- Root contexts: language (`i18n::provide_lang`, table-driven ID/EN dictionary, browser default, persisted `rotaria_lang`), `Session` (tokens, profile, sheet state), `Toasts` (deduped, errors as `role=alert`), `ShellTheme` (follows the system scheme live until toggled, then `rotaria_theme`; the toggle's label names the theme it switches to; `theme-color` meta follows; `boot.js` applies it before first paint). Modal sheets keep Tab and Shift+Tab inside them; English counts take the singular for one (`Lang::text_n`).
- API client (`api/mod.rs`): gloo-net, 401 single-flight refresh (the rejected body is drained first so it cannot pin a connection; a refused refresh token, 401 or 400, clears the session, a network or server failure keeps it), offline detection, pending-write queue drained on `online` and login (single-flight, each replayed write deleted by its own key), signup errors surfaced instead of advancing to the code step, logout signs the device out before the best-effort server revoke, `asset_url` turns stored cover keys into `/api/v1/covers/{file}`.
- Storage: IndexedDB through rexie, `project_baca_db` v3 (guest progress, offline books and chapters, pending sync, local quotes). Reader and shelf fall back to it offline.
- Service worker (`sw.js`): `rotaria-shell-v3` holds one entry, `/`, refreshed by every navigation (network-first) and served for any route offline; `rotaria-assets-v3` is cache-first for content-hashed build output and the fonts (stale build files pruned after each shell fetch) and network-first for every other static file, so a deploy reaches returning readers on their next visit; `/api/*` and other origins are never cached. `Caddyfile.prod` marks only hashed files and `/fonts/*` immutable; the rest of the SPA revalidates (`no-cache`), and API responses keep the server's cache policy.
- Fonts: EB Garamond, Newsreader, and Plus Jakarta Sans are self-hosted from `crates/web/fonts` (variable woff2, latin and latin-ext subsets by `unicode-range`, `font-display: swap`, OFL texts beside them); the CSP has no third-party font or style source.
- Design: Espresso shell (`#1F1916`, clay `#CE734E`) with a Paper shell toggle; reading themes Paper, Sepia, Espresso; EB Garamond display, Newsreader body, Plus Jakarta Sans labels; copy in the literate "kamu" register; 44 px targets; motion disabled under `prefers-reduced-motion`.
- Illustrations (`assets/illustrations/`): PNG originals at 640 px, WebP copies (quality 82, `magick in.png -quality 82 -define webp:method=6 in.webp`) copied by `index.html`.

  | File | Used on |
  |---|---|
  | `library-bookshelf-ladder` | home hero plate (wide screens) |
  | `cozy-reader-armchair-owl` | guest shelf, reader finale |
  | `manuscript-inspection-clothesline` | curator desk empty lists |
  | `admin-sorting-pigeonholes` | non-curator admin, empty signed-in shelf |
  | `retro-rocket-discovery` | empty search, 404 |
  | `rotaria-mark.svg` | favicon and PWA tile; the header draws the same mark inline |
  | `rotaria-windmill.svg` | detailed brand plate for onboarding and email; too fine below 36 px |
- Responsive: under 768 px the tab bar, bottom sheets, and the book action dock; from 768 px header navigation and dialogs; catalog columns 2 / 3 / 4 / 5 at under 640 / 1024 / 1440 / wider; home shows two catalog rows until "see the whole catalog".

### Reader (`pages/reader/`)

- `layout.rs`: the chapter flows into CSS columns one page wide; the column gap equals twice the side padding, so one page stride is the viewport width. Two columns from a 1024 px window (the window, not the padded frame, so CSS and Rust share one breakpoint), and always on a window spanning two hinged screens (`horizontal-viewport-segments: 2`), where the spine sits on the hinge and the page gap is at least the hinge width; measure capped at 1440 px.
- `flip.rs`: a page turn rebuilds the leaving page as strips hinged on the spine (8 on a spread, 6 on a single page). Each strip holds a clone that reproduces exactly one column (spacer plus the elements intersecting that column), so a turn costs a page of layout, not a chapter. A damped spring drives the angle (drag k 900, zeta 1.0; release k 240, zeta 0.86); release past 90 degrees or a flick above 160 degrees per second completes, otherwise the page falls back; the last page resists at 22 degrees. Frames write only `transform` and `opacity`; no signals update per frame.
- `mod.rs`: pointer gestures (touch drags anywhere, mouse from the outer 22%, gestures starting on controls ignored), keys (arrows, Space, PageUp/Down, Home, End, Escape; Space on a focused control activates it, keys in fields stay in the field), center tap dims the bars, two fingers zoom (`touch-action: pinch-zoom`), position anchored to the first visible paragraph (`p-N`) and saved debounced (account via `PUT /progress/{book}`, guest via IndexedDB). Saving waits until the stored position is known and applied, a position that arrives after the chapter re-paginates, a chapter answer for a chapter already left is dropped (load generation), and a pending save is flushed on unmount and on `pagehide` (a `keepalive` request for accounts); heartbeat once a minute while visible, finale card after the last page. `tools.rs`: the chapter drawer, search inside the book (`GET /books/{id}/search`, or the IndexedDB copy offline, both through `shared::text_hit_snippet`; a hit opens its `p-N` page and the paragraph glows once), and the selection watch behind "Save quote" (account or device, as the quote finder). A chapter's closing card shows the first time it ends on a visit. Sheets close after the click that closes them (`close_after_event`). Reduced motion uses a 140 ms crossfade.

### App shell (`apps/tauri`)

- Tauri 2 around the same release bundle (`make build-web` into `target/app-web`, so an app build never overwrites the `crates/web/dist` that `trunk serve` is serving); a Cargo workspace of its own, so the root workspace and CI need no WebView toolchain.
- The window gets `window.__ROTARIA_API__` from `ROTARIA_API_BASE` at build time through an initialization script; `api_base()` prefers it, `boot.js` registers no service worker inside Tauri, and CORS allows `tauri://localhost` and `http://tauri.localhost`.
- `rotaria://book/<uuid>` opens the book (the id must parse as a UUID); a link that arrives before the first page load waits for it. Android declares the scheme in its manifest, desktop registers it at run time.
- Android (`gen/android`, identifier `io.github.aprxty3.rotaria`): `MainActivity` pads the page by the system bar insets, since Android 15 draws edge to edge and the WebView reports no safe-area insets; debug builds allow cleartext to the dev API at `10.0.2.2`. Sharing calls `navigator.share`; where it is missing the call throws and the clipboard fallback never runs (issue #14).

## 6. Ingestion and AI

**Flow.** Upload stores the EPUB at `raw-epubs/<book_id>.epub` and publishes to `stream:epub_ingestion`. Python workers in the consumer group `ingestion-workers` process it and report progress to the `job:{id}` hash read by the curator desk. Workers are stateless; add processes to scale.

**Message contract.** Stream fields `book_id`, `storage_path`, `job_id`, `timestamp`. Job hash `job:{id}`: `status` (`queued`, `parsing`, `chunking`, `embedding`, `summarizing`, `published`, `failed`), `progress` 0 to 100, `error`, `attempts`, timestamps; TTL 7 days. Dead letters go to `stream:epub_ingestion:dlq`; `POST /admin/dlq/{id}/replay` re-queues under the original job id. Both streams are capped with `XADD MAXLEN ~ 10000` (server and worker share the number), so Redis does not grow with every upload ever made.

**Pipeline per message.**
1. Download; the end-of-central-directory record `ZipFile` itself will use (`_EndRecData`) is read first, so an archive declaring more than 5,000 entries (or a directory too large for them) is refused before `ZipFile` builds one object per entry; then the zip guard (per-file size, compression ratio, ZipSlip paths).
2. Parse OPF and spine; a spine that names one document many times yields it once. Limits live in `EpubLimits` (2,000 chapters, 32 MB of text, 8,000 chunks; the largest real book in dev has 300, 6.3 MB, about 2,800). The cover decodes with Pillow 13 only as JPEG, PNG, WebP or GIF, at most 40 megapixels (a bomb is refused at its header), thumbnailed to 1,600 px and stored as `covers/<book_id>.webp`; any other cover is dropped and the book publishes on the typographic cover.
3. Sanitize XHTML: allowlisted tags, scripts, styles and handlers removed, decoded character references re-escaped; open elements are tracked, so an unclosed `iframe` or `object` ends with its parent and a void `embed` hides nothing; text outside any paragraph (a dropped `div`'s content) is wrapped in a `<p>`. A Project Gutenberg book loses everything before `*** START OF ... ***` (header, cover credits) and from `*** END OF ... ***` on (the license and any spine item after it), so neither becomes a chapter or costs Gemini calls. Each `<p>` is indexed in `chapter_paragraphs` as plain text (entities decoded, `<br>` a space, whitespace collapsed) and case-folded per character, numbered as a browser parses them, so row `n` is the reader's `p-n`; `make backfill-paragraphs` indexes books stored before the table.
4. Chunk about 400 words, tails under 50 merged. Each chunk stores in `cfi_range` the `p-N` anchor of the paragraph it starts in (the next paragraph when it starts on a heading; positions compared without whitespace), so a quote hit opens the reader on that paragraph (`/read/<id>?chapter=N&at=p-M`, lit as an in-book hit is). `make backfill-paragraphs` anchors stored chunks without new embeddings. A chapter with no `<p>` leaves it null and the hit opens the chapter.
5. Embed in batches, 768 dimensions, `EMBEDDING_MODEL_NAME` (`fastembed` is a stub; production needs `GEMINI_API_KEY`).
6. Atomic cards and the spoiler-free recap with `LLM_MODEL_NAME`; recap input is the key concepts of earlier chapters plus the current text within a 12k-character budget; rows upsert on `uq_tldr_cache`.
7. Insert chapters and chunks, set the book `published`, `XACK`. Metadata and cover commit before step 4 in their own transaction, so the book row is not held through hours of Gemini calls; `PATCH /admin/books/{id}` waits at most 2 s for the row (`lock_timeout`) and answers 409 past it. The job reads the status again before embedding, so a book archived meanwhile costs no further Gemini calls.

**Reliability.** At-least-once: `XACK` only after success; entries idle past `RECLAIM_IDLE_MS` (300000) are reclaimed with `XAUTOCLAIM` at the start of every loop turn, one at a time with a cursor that walks the pending list (a batch would leave the rest idle behind the first job for another replica to take), while a running job refreshes its claim (`XCLAIM ... JUSTID` from a heartbeat thread, at most every 60 s) so a long book is never taken by a second replica; every delivery increments `attempts` before the job runs, and a fourth delivery is dead-lettered without running, so a job that kills the process (OOM, SIGKILL) stops after three; a transient failure stays pending with its reason in the job hash, and the third is dead-lettered. Replay moves an entry with one Lua step (`XDEL`, then `XADD` only if it removed the entry), so concurrent replays re-queue once. SIGTERM or SIGINT interrupts even a blocking call: the job is handed back unacknowledged, its attempt uncounted, its status `queued`; compose runs the worker with `init: true`. A book archived before or during ingestion is never republished (`WHERE status <> 'archived'`). Title, author, language, and headings are clipped to their column widths. Gemini calls send the key in `x-goog-api-key`, retry transient failures (HTTP 429 and 5xx, connect and read timeouts) three times with quadratic backoff (1, 4, 9 s, jitter, `Retry-After` up to 60 s), and time out at 60 s (embeddings) or 120 s (generation). The Redis socket timeout (30 s) exceeds the `XREADGROUP` block window. A job no retry can fix raises `PermanentJobError` (`RejectedEpub` for a malformed upload or one past a limit, a missing object, a deleted book row) and is dead-lettered at once with its reason, which the job hash and the curator desk show; other errors reach the desk as their kind only, with the detail in the worker log. Every write to a job hash refreshes its 7-day TTL in the same `MULTI`. Production compose caps the worker at `mem_limit: 768m`.

**Search.** Quote search embeds the query through Gemini on the server and runs an HNSW cosine search scoped `WHERE book_id = $1`, so index memory follows the active book, not the corpus. Catalog search uses a partial GIN index (FTS and trigram) over published books; index and query both use the `simple` text config (`plainto_tsquery('simple', $1)`), so a word is matched as written, in the description too; there is no stemming.

## 7. Data

- Paired migrations in `migrations/`; schema and index matrix in `knowledge/erd.md`.
- Hot indexes: partial GIN FTS and trigram `WHERE status = 'published'`, HNSW cosine on `book_chunks.embedding`, foreign-key B-trees, catalog index on `(publication_year DESC NULLS LAST, id)`. The catalog query orders the same way (books without a year last), and its keyset cursor continues past them, so a page reads the index without a sort and every published book is reached once.
- Covers: the worker stores `covers/<book uuid>.<ext>` in the covers bucket and `books.cover_url` holds that key.
- `tldr_cache` has a unique key per (chapter, kind); replays upsert.

## 8. Deployment

- `docker-compose.prod.yml` with `.env.production.example`: postgres, redis, minio, server, worker, and edge (Caddy on 80 and 443 TCP and UDP), all on the default compose network; least-privilege roles, keys, and networks are issue #47. Targets: `make prod-build`, `prod-up`, `prod-down`, `prod-logs`.
- `make build` compiles release binaries and runs `make build-web`: the WASM bundle with `API_BASE_URL` baked in, built with the `wasm-release` Cargo profile (`opt-level = "z"`, LTO, one codegen unit) and `wasm-opt -Oz`: 1.57 MB raw and 608 KB gzip in 0.1.0 (1.47 MB and 573 KB before the reader tools; 2.83 MB and 909 KB with the default profile). `scripts/check_bundle_budget.py` fails the build above 1.8 MB raw or 700 KB gzip. Release builds minify HTML, CSS, and the glue. The edge image runs `make build-web-edge`, the same build plus brotli 11 and gzip copies (`scripts/precompress.py`) that Caddy sends as they are (`file_server { precompressed br gzip }`); the Tauri bundle leaves them out.
- Server and worker images run as uid 10001; the dev compose binds every port to `127.0.0.1`, and `Caddyfile.prod` drops the `Server` banner on every response, errors included.
- With `APP_ENV=production` the server exits non-zero when Postgres or Redis is unreachable at boot, and `restart: always` retries; development keeps running without them.
- The server healthcheck hits `/health`; `scripts/pg_backup.sh` dumps Postgres.
- Observability: tracing with `x-request-id` on every span; `LOG_FORMAT=json` for NDJSON.

## 9. Service levels

| Path | Target |
|---|---|
| REST API, cached or indexed routes | p95 < 50 ms |
| Catalog FTS and trigram search | < 5 ms |
| Scoped vector search | < 10 ms |
| Login, OTP verify (release build) | p95 < 150 ms, < 100 ms |
| Reader page turn | `transform` and `opacity` only; one column cloned per turn |

`make test-performance-release` checks the API targets.

Argon2 (19 MiB per hash or verify) runs behind a semaphore of `ARGON2_MAX_CONCURRENCY` slots (default: cores, at most 8). The slot moves into the blocking task, so a dropped request still holds it until its hash ends; a request that gets no slot within 2 s answers 503 with `Retry-After: 1`. Peak memory is about 24 MB plus 19 MiB per slot.

Memory, measured 2026-11-03 (release builds, dev data):

| Process | Idle | Under load |
|---|---|---|
| API server | 24 MB RSS | 61 MB at 256 concurrent mixed reads (`MALLOC_ARENA_MAX=2`, set in the image; 72 MB without); 189 MB at 100 concurrent logins with 8 Argon2 slots (`MALLOC_MMAP_THRESHOLD_=131072`, set in the image; 814 MB without it at 18 slots, 1.3 GB with no gate) |
| Ingestion worker | 35 MB RSS | 53 MB RSS, 19.5 MB Python heap, parsing a 300-chapter EPUB (6.3 MB of text) |
| Web app | 1.3 MB JS heap, 1.3 MB WASM memory on the catalog | 2.8 MB JS heap in the reader; DOM nodes and listeners stay flat over 60 page turns, 45 sheet cycles, and 36 chapter jumps |

Web first load, Lighthouse 12.6 mobile (simulated slow 4G, 4x CPU), release bundle behind Caddy with compression, dev data, median of 5 runs on the home page and 3 elsewhere, 2026-11-03:

| Route | Before (on-the-fly gzip, no minify) | After (precompressed brotli 11, minified HTML, CSS, and glue) |
|---|---|---|
| Home | score 52, LCP 5.73 s, FCP 5.28 s, CLS 0.291 | score 71, LCP 4.89 s, FCP 4.43 s, CLS 0.072 (every run) |
| Book | score 67, LCP 5.88 s | score 73, LCP 4.96 s |
| Reader | score 65, LCP 5.81 s | score 71, LCP 4.89 s |

The largest paint on every route is text the WASM app draws (the hero heading, the synopsis, a paragraph), so first paint waits on the 1.6 MB WASM: 473 KB with brotli, about 2.5 s of the simulated link. CLS came from the phone picks rail appearing above the catalog; it now holds its size while the books load. A further cut needs a static first paint before the WASM, which is a prerender.

## 10. Tests

Functional server suites are modules of one binary (`crates/server/tests/it`); `performance_test` and `load_stress_test` stay separate so parallel tests cannot skew latency assertions. Web tests are a Playwright pyramid in `crates/web/tests` (e2e, component, integration, a11y, visual). Targets and counts: [GUIDE.md](GUIDE.md) section 5.

## 11. Invariants

1. No `unwrap()` or `expect()` on production paths in any crate; errors flow through `Result<T, AppError>`.
2. Stateless API; scoped vector search; partial indexes on published books.
3. Clear crate boundaries, paired reversible migrations, Makefile as the single command surface.
4. One DTO and validation source (`crates/shared`); one i18n dictionary.
5. Postgres for relational, lexical, and vector data; Redis Streams instead of a broker.
6. MVP scope only; no abstraction before its second use.
7. No emoji in code, comments, commits, or docs. The fleuron `❖` is typography.
