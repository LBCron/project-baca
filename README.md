# Project Baca (Rotaria)

Open-access e-reader for public-domain literature: a paginated reader that turns pages like paper, spoiler-free recaps, atomic insight cards, semantic quote search, offline books, and a quiet reading streak. Indonesian and English interface.

## Stack

| Layer | Technology |
|---|---|
| API | Rust, Axum, SeaORM, utoipa (`crates/server`, `crates/infra`, `crates/domain`, `crates/shared`) |
| Web | Rust, Leptos 0.7 client-side WebAssembly built by Trunk; PWA with service worker and IndexedDB (`crates/web`) |
| Worker | Python 3.11 Redis Streams consumer: EPUB parsing, chunking, Gemini embeddings and summaries (`python_worker`) |
| Data | PostgreSQL 17 with pgvector and pg_trgm, Redis 7, MinIO or any S3 |
| Edge | Caddy: TLS, HTTP/3, static shell, security headers (`Caddyfile`, `Caddyfile.prod`) |

## Routes

- Web: `/` catalog, `/book/:id` overview, `/read/:id` reader, `/me` shelf and account, `/admin` curator desk.
- API: `/api/v1/*` only. Swagger UI at `/swagger-ui` outside production. Full table in [ARCHITECTURE.md](ARCHITECTURE.md).

## Quickstart

```bash
cp .env.example .env            # SMTP_SECURITY=none keeps Mailpit working
make db-up                      # Postgres, Redis, MinIO, Mailpit
make migrate-up seed-dev        # schema plus four CC0 books
make dev-server                 # API on :8080
make dev-web                    # PWA on :3000
make worker-install worker      # optional: ingestion worker for EPUB uploads
```

## Documentation

- [GUIDE.md](GUIDE.md): setup, configuration, tests, reader controls.
- [ARCHITECTURE.md](ARCHITECTURE.md): crates, request path, API surface, security model, web app and reader internals, deployment.
- [DISTRIBUTED.md](DISTRIBUTED.md): ingestion worker, message contract, retries, dead letters.
- [CHANGELOG.md](CHANGELOG.md), [NOTICE.md](NOTICE.md), [assets/README.md](assets/README.md).

## Content

Texts come from Standard Ebooks, Project Gutenberg, and Wikisource and are in the public domain. Licensing and attribution: [NOTICE.md](NOTICE.md).
