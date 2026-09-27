# Ingestion Worker (`python_worker/`)

background pipeline: consumes `stream:epub_ingestion` via the
`ingestion-workers` consumer group — download → parse/sanitize → chunk →
embed → summarize/publish. Progress is reported to the `job:{id}` Redis hash
read by `GET /api/v1/admin/jobs/{id}`.

## Setup

```bash
cd python_worker
uv venv .venv
uv pip install -r requirements.txt
```

## Run

```bash
# Long-running consumer (production-like)
set -a && . ../.env && set +a
.venv/bin/python worker.py

# Single message then exit (tests, backfill, debugging)
.venv/bin/python worker.py --once
```

## Unit tests (no services required)

```bash
.venv/bin/python -m unittest test_worker -v
```

## Environment

Same names as the repo `.env`: `REDIS_URL`, `DATABASE_URL`, `S3_ENDPOINT`,
`S3_REGION`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`, `S3_BUCKET_EPUBS`,
`S3_BUCKET_COVERS`, `GEMINI_API_KEY`, `EMBEDDING_MODEL_NAME`,
`EMBEDDING_DIMENSION`, plus `LLM_MODEL_NAME` (default `gemini-flash-latest`).

## Job lifecycle

Stream message fields: `book_id`, `storage_path`, `job_id`, `timestamp`.
Job hash `job:{id}` fields: `status` (`queued → parsing → chunking →
embedding → summarizing → published`, or `failed`), `progress` (0–100),
`book_id`, `storage_path`, `created_at`, `updated_at`, optional `error`,
`attempts`. Hashes expire after 7 days. Jobs failing 3 attempts move to
`stream:epub_ingestion:dlq` and are acknowledged. Idle pending entries are
reclaimed via `XAUTOCLAIM` after 5 minutes (dead-worker recovery).

## Notes & deviations

* Covers are converted to WebP via Pillow; on conversion failure the original
  bytes are stored instead (ingestion never dies on a cover).
* Embeddings and summaries call the Gemini REST API directly over stdlib
  `urllib` (no extra dependency). Error bodies are surfaced verbatim.
* The Rust one-shot `crates/infra/examples/backfill_chunks.rs` covers the
  pre-Task-05 stub catalog; this worker is the streaming replacement.
