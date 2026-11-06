# Project Baca (Rotaria)

Open-access e-reader for public-domain literature: a paginated reader that turns pages like paper, spoiler-free recaps, atomic insight cards, semantic quote search, offline books, and a quiet reading streak. Indonesian and English interface.

## Stack

| Layer | Technology |
|---|---|
| API | Rust, Axum, SeaORM, utoipa (`crates/server`, `crates/infra`, `crates/domain`, `crates/shared`) |
| Web | Rust, Leptos 0.7 client-side WebAssembly built by Trunk; PWA with service worker and IndexedDB (`crates/web`) |
| Worker | Python 3.11 Redis Streams consumer: EPUB parsing, chunking, Gemini embeddings and summaries (`python_worker`) |
| Data | PostgreSQL 17 with pgvector and pg_trgm, Redis 7, MinIO or any S3 |
| Edge | Caddy: static shell, security headers, API proxy (`Caddyfile`, `Caddyfile.prod`); TLS and HTTP/3 locally, Cloudflare Tunnel in front in production |

## Routes

- Web: `/` catalog, `/book/:id` overview, `/read/:id` reader, `/me` shelf and account, `/admin` curator desk.
- API: `/api/v1/*` only. Swagger UI at `/swagger-ui` outside production. Full table in [ARCHITECTURE.md](ARCHITECTURE.md).

## Quickstart

```bash
cp .env.example .env            # SMTP_SECURITY=none keeps Mailpit working
make db-up                      # Postgres, Redis, MinIO, Mailpit
make migrate-up seed-dev        # schema plus four CC0 books
make dev backend                # API on :8080
make dev frontend               # PWA on :3000
make worker-install worker      # optional: ingestion worker for EPUB uploads
```

## Documentation

- [GUIDE.md](GUIDE.md): setup, configuration, tests, reader controls.
- [ARCHITECTURE.md](ARCHITECTURE.md): crates, request path, API, security, web app and reader, ingestion worker, data, deployment, service levels.
- [CHANGELOG.md](CHANGELOG.md) and [NOTICE.md](NOTICE.md); releases and container images are built by GitHub Actions (GUIDE section 8).

## Content and license

Texts come from Standard Ebooks, Project Gutenberg, and Wikisource and are in the public domain; fonts are under the SIL Open Font License. The source code is proprietary, all rights reserved ([LICENSE](LICENSE)); versions 0.1.1 and earlier stay under MIT OR Apache-2.0. Attribution and third-party licenses: [NOTICE.md](NOTICE.md).
