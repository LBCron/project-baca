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

Axum layers, outermost first: tracing, CORS (`CORS_ALLOWED_ORIGINS`), server-minted `x-request-id`, security headers. Per-router limiters on one Redis fixed-window engine (`middleware/rate_limit.rs`): auth 20/min per IP; AI 10/min per user or guest IP; public reads `PUBLIC_RATE_LIMIT_PER_MINUTE` per IP on books, covers, insights, and gamification when the value is above 0. The admin router raises the body limit to 50 MB plus 64 KiB; the upload handler enforces 50 MB while streaming. Redis outage on auth or AI paths answers 503 (fail closed).

## 3. API surface

All paths below are under `/api/v1` unless noted. Responses use `{ "success": true, "data": ... }` or `{ "success": false, "error": { "code", "message", "details" } }`; codes: `BAD_REQUEST`, `VALIDATION_FAILED`, `UNAUTHORIZED`, `FORBIDDEN`, `NOT_FOUND`, `CONFLICT`, `RATE_LIMITED`, `AI_RATE_LIMITED`, `DATABASE_ERROR`, `INTERNAL_ERROR`, `EXTERNAL_SERVICE_ERROR` (502), `SERVICE_UNAVAILABLE` (503).

| Method and path | Auth | Notes |
|---|---|---|
| `GET /health` (root) | none | 503 `degraded` when the process has no database connection |
| `GET /health` | none | Postgres and Redis status |
| `POST /auth/signup`, `/auth/verify-otp`, `/auth/login`, `/auth/refresh` | none | auth limiter; signup 502 when SMTP is down, 60 s cooldown per email, 10/h per IP; OTP: three wrong codes lock for 5 min (429, `Retry-After`) |
| `POST /auth/logout`, `/auth/revoke-all?keep_current=` | bearer | logout blacklists the session's last access token; revoke-all drops other devices |
| `GET`, `PATCH`, `DELETE /me` | bearer | profile; delete revokes first, then cascades |
| `PUT /me/password` | bearer | revokes other sessions and returns a fresh token pair |
| `GET /me/sessions`, `DELETE /me/sessions/{id}` | bearer | per-device sessions, caller flagged |
| `GET /me/badges`, `GET /me/streak` | bearer | gamification standing |
| `GET /books`, `/books/search`, `/books/{id}`, `/books/{id}/chapters/{n}`, `/books/{id}/offline-bundle` | none | published books only; cursor paging; `q` capped at 200 chars, filters at column width |
| `GET /covers/{file}` | none | `<uuid>` plus webp, jpg, jpeg, or png from the covers bucket, `Cache-Control: public, max-age=31536000, immutable` |
| `GET /books/{id}/chapters/{ref}/atomic-cards`, `.../recap` | none | `ref` is a chapter number or chapter UUID; 404 for unpublished books |
| `POST /books/{id}/quotes/search` | optional bearer | AI limiter; HNSW cosine search scoped `WHERE book_id = $1` |
| `POST /quotes/save`, `POST /quotes/save-batch`, `GET /quotes`, `GET /quotes/{id}/card` | bearer | duplicate save returns the existing row with 200 `saved:false`; batch up to 50 with per-item outcome; card is owner-only SVG |
| `GET /progress/active`, `PUT /progress/{book_id}`, `POST /progress/merge` | bearer | merge is transactional, capped at 100 records, `GREATEST` percentage |
| `POST /activity/heartbeat` | bearer | one rewarded heartbeat per user per 60 s (atomic `SET NX EX`), credit clamped to 90 s |
| `GET /badges` | none | badge catalog |
| `GET /admin/books`, `POST /admin/books/upload`, `PATCH /admin/books/{id}`, `GET /admin/jobs/{id}`, `GET /admin/dlq`, `POST /admin/dlq/{id}/replay`, `GET /admin/analytics/drop-off` | bearer, role `admin` re-read from the user row | upload: `.epub`, 50 MB, 202 with job id and a stream event; status changes follow `BookStatus::can_transition_to` (409 otherwise) |
| `/swagger-ui`, `/api-docs/openapi.json` (root) | none | not mounted when `APP_ENV=production`; no edge route |

Unknown paths answer 404 in the error envelope; the unversioned `/api/*` alias no longer exists.

## 4. Security model

- Passwords: Argon2id (19 MiB, 2 iterations). OTP: six digits, SHA-256 in Redis, 10 min TTL, three attempts then a 5 min lock.
- Tokens: HS256 access token (1440 min in dev, 60 min in production compose); refresh token stored as a SHA-256 hash with a session record (device label, IP prefix, last access `jti`), rotated atomically (`GETDEL`, 30 s grace for concurrent tabs); a replayed token revokes the family including access tokens. Logout and per-device revocation blacklist the last access `jti`.
- Lockouts: 5 failures per (email, IP) and 20 per email lock login for 15 min; unverified accounts behave like a wrong password.
- Client address: with `TRUST_PROXY_HEADERS=true` only `TRUSTED_IP_HEADER` is read; `Caddyfile.prod` sets `X-Forwarded-For` and strips `X-Real-IP`, `CF-Connecting-IP`, `True-Client-IP`, `Forwarded`. The default JWT secret is refused in production or whenever proxy headers are trusted.
- Headers: the API sets HSTS, `nosniff`, frame denial, referrer policy, COOP and CORP, a permissions policy, its own CSP, and `Cache-Control: no-store` on `/auth/*` and on every bearer response. Caddy sets the SPA headers, including a CSP with `script-src 'self' 'wasm-unsafe-eval'` and no inline scripts (a Trunk post-build hook moves the bootstrap into a hashed `boot-*.js`).
- Data exposure: draft and archived books are invisible on every public route; validation errors name the field without echoing the value; logs mask connection strings, recipients, OTPs, and the Gemini key (sent in `x-goog-api-key`).
- Open security work is tracked as tickets in `knowledge/tickets/` (private repo).

## 5. Web app (`crates/web`)

- Routes: `/`, `/book/:id`, `/me`, `/admin`, and `/__gallery` (component stories for tests) under `ShellLayout` (header, phone tab bar, auth sheet, toasts); `/read/:id` renders without chrome; unknown paths show the 404 page.
- Root contexts: language (`i18n::provide_lang`, table-driven ID/EN dictionary, browser default, persisted `rotaria_lang`), `Session` (tokens, profile, sheet state), `Toasts` (deduped, errors as `role=alert`), `ShellTheme` (system scheme by default, `rotaria_theme` once toggled, `theme-color` meta follows; `boot.js` applies it before first paint).
- API client (`api/mod.rs`): gloo-net, 401 single-flight refresh, offline detection, pending-write queue drained on `online` and login, `asset_url` turns stored cover keys into `/api/v1/covers/{file}`.
- Storage: IndexedDB through rexie, `project_baca_db` v3 (guest progress, offline books and chapters, pending sync, local quotes). Reader and shelf fall back to it offline.
- Service worker (`sw.js`): `rotaria-shell-v2` precaches `/` and serves it for unvisited deep links offline; `rotaria-assets-v2` caches hashed assets and prunes stale ones; `/api/*` is never cached.
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

- `layout.rs`: the chapter flows into CSS columns one page wide; the column gap equals twice the side padding, so one page stride is the viewport width. Two columns from 1024 px, measure capped at 1440 px.
- `flip.rs`: a page turn rebuilds the leaving page as strips hinged on the spine (8 on a spread, 6 on a single page). Each strip holds a clone that reproduces exactly one column (spacer plus the elements intersecting that column), so a turn costs a page of layout, not a chapter. A damped spring drives the angle (drag k 900, zeta 1.0; release k 240, zeta 0.86); release past 90 degrees or a flick above 160 degrees per second completes, otherwise the page falls back; the last page resists at 22 degrees. Frames write only `transform` and `opacity`; no signals update per frame.
- `mod.rs`: pointer gestures (touch drags anywhere, mouse from the outer 22%, gestures starting on controls ignored), keys (arrows, Space, PageUp/Down, Home, End, Escape), center tap dims the bars, position anchored to the first visible paragraph (`p-N`) and saved debounced (account via `PUT /progress/{book}`, guest via IndexedDB), heartbeat once a minute while visible, finale card after the last page. Reduced motion uses a 140 ms crossfade.

## 6. Ingestion and AI

**Flow.** Upload stores the EPUB at `raw-epubs/<book_id>.epub` and publishes to `stream:epub_ingestion`. Python workers in the consumer group `ingestion-workers` process it and report progress to the `job:{id}` hash read by the curator desk. Workers are stateless; add processes to scale.

**Message contract.** Stream fields `book_id`, `storage_path`, `job_id`, `timestamp`. Job hash `job:{id}`: `status` (`queued`, `parsing`, `chunking`, `embedding`, `summarizing`, `published`, `failed`), `progress` 0 to 100, `error`, `attempts`, timestamps; TTL 7 days. Dead letters go to `stream:epub_ingestion:dlq`; `POST /admin/dlq/{id}/replay` re-queues under the original job id.

**Pipeline per message.**
1. Download; zip guard (per-file size, compression ratio, ZipSlip paths).
2. Parse OPF and spine; cover to WebP with Pillow as `covers/<book_id>.<ext>` (original bytes on failure; a cover never fails a job).
3. Sanitize XHTML: allowlisted tags, scripts, styles and handlers removed, decoded character references re-escaped.
4. Chunk about 400 words, tails under 50 merged, with CFI ranges.
5. Embed in batches, 768 dimensions, `EMBEDDING_MODEL_NAME` (`fastembed` is a stub; production needs `GEMINI_API_KEY`).
6. Atomic cards and the spoiler-free recap with `LLM_MODEL_NAME`; recap input is the key concepts of earlier chapters plus the current text within a 12k-character budget; rows upsert on `uq_tldr_cache`.
7. Insert chapters and chunks, set the book `published`, `XACK`.

**Reliability.** At-least-once: `XACK` only after success; entries idle past `RECLAIM_IDLE_MS` (300000) are reclaimed with `XAUTOCLAIM`; three attempts, then the dead-letter stream. Gemini calls retry transient failures three times with quadratic backoff (1, 4, 9 s, jitter, `Retry-After` up to 60 s) and time out at 60 s (embeddings) or 120 s (generation). The Redis socket timeout (30 s) exceeds the `XREADGROUP` block window.

**Search.** Quote search embeds the query through Gemini on the server and runs an HNSW cosine search scoped `WHERE book_id = $1`, so index memory follows the active book, not the corpus. Catalog search uses a partial GIN index (FTS and trigram) over published books.

## 7. Data

- Paired migrations in `migrations/`; schema and index matrix in `knowledge/erd.md`.
- Hot indexes: partial GIN FTS and trigram `WHERE status = 'published'`, HNSW cosine on `book_chunks.embedding`, foreign-key B-trees, zero-sort catalog index on `(publication_year DESC NULLS LAST, id)`.
- Covers: the worker stores `covers/<book uuid>.<ext>` in the covers bucket and `books.cover_url` holds that key.
- `tldr_cache` has a unique key per (chapter, kind); replays upsert.

## 8. Deployment

- `docker-compose.prod.yml` with `.env.production.example`: postgres, redis, minio (internal network only), server, worker, edge (Caddy on 80 and 443 TCP and UDP). Targets: `make prod-build`, `prod-up`, `prod-down`, `prod-logs`.
- `make build` compiles release binaries and the WASM bundle with `API_BASE_URL` baked in.
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
