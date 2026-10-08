# Ingestion Worker (`python_worker/`)

Consumes `stream:epub_ingestion` through the `ingestion-workers` consumer group: download, parse and sanitize, chunk, embed, summarize, publish. Progress goes to the `job:{id}` Redis hash read by `GET /api/v1/admin/jobs/{id}`. Full contract and reliability rules: [DISTRIBUTED.md](../DISTRIBUTED.md).

## Setup and run

```bash
cd python_worker
uv venv .venv && uv pip install -r requirements.txt
set -a && . ../.env && set +a
.venv/bin/python worker.py          # long-running consumer
.venv/bin/python worker.py --once   # one message, then exit
```

From the repo root: `make worker-install`, `make worker`, `make worker-once`.

## Tests

```bash
.venv/bin/python -m unittest test_worker -v          # 16 unit tests, no services
RECLAIM_IDLE_MS=0 .venv/bin/python -m unittest test_worker_live -v   # needs Redis and Postgres
```

## Environment

Same names as the repo `.env`: `REDIS_URL`, `DATABASE_URL`, `S3_ENDPOINT`, `S3_REGION`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`, `S3_BUCKET_EPUBS`, `S3_BUCKET_COVERS`, `GEMINI_API_KEY`, `EMBEDDING_MODEL_NAME`, `EMBEDDING_DIMENSION`; worker only: `LLM_MODEL_NAME` (default `gemini-flash-latest`), `RECLAIM_IDLE_MS` (default 300000).

## Notes

- Covers convert to WebP with Pillow; on failure the original bytes are stored. A cover never fails a job.
- Gemini is called over stdlib `urllib` with retries (1 s, 4 s, 9 s, jitter, `Retry-After`); error bodies are sanitized before logging and the key is never logged.
- Three failed attempts move a message to `stream:epub_ingestion:dlq`; curators replay it from the desk.
