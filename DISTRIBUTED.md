---
type: Technical Architecture Specification
title: Ingestion Worker and Queue
tags:
  - distributed
  - embedding
  - fastembed
  - gemini
  - ingestion
  - okf
  - redis-streams
  - scalability
  - worker
---

# Ingestion Worker and Queue

## 1. Topology

Axum publishes one message per EPUB upload. Python workers consume it through the consumer group `ingestion-workers` on `stream:epub_ingestion`, write to PostgreSQL and the covers bucket, and report progress to a Redis hash the curator desk polls. Workers are stateless; run more processes to scale.

## 2. Message contract

- Stream fields: `book_id`, `storage_path` (`raw-epubs/<book_id>.epub`), `job_id`, `timestamp`.
- Job hash `job:{id}`: `status` (`queued`, `parsing`, `chunking`, `embedding`, `summarizing`, `published`, or `failed`), `progress` 0 to 100, `book_id`, `storage_path`, `created_at`, `updated_at`, optional `error` and `attempts`. TTL 7 days. Read by `GET /api/v1/admin/jobs/{id}`.
- Dead letters: `stream:epub_ingestion:dlq`, listed by `GET /api/v1/admin/dlq` and replayed by `POST /api/v1/admin/dlq/{id}/replay` under the original job id.

## 3. Pipeline per message

1. Download the EPUB; zip guard (per-file size, compression ratio, ZipSlip paths).
2. Parse OPF and spine; extract the cover and convert it to WebP with Pillow (original bytes kept on failure) as `covers/<book_id>.<ext>`.
3. Sanitize XHTML: allowlisted tags, scripts, styles, and handlers removed, decoded character references re-escaped on output; detect scene breaks; count words.
4. Chunk about 400 words per piece, tails under 50 merged, with CFI ranges.
5. Embed in batches (768 dimensions) with `EMBEDDING_MODEL_NAME`. The `fastembed` provider is a stub that errors descriptively; production needs `GEMINI_API_KEY`.
6. Generate atomic cards and the spoiler-free recap per chapter with `LLM_MODEL_NAME` (default `gemini-flash-latest`). Recap input is the key concepts of chapters 1 to n-1 plus the current chapter text inside a 12k-character budget (current chapter keeps at least 4k, oldest summaries drop first). Rows upsert on `uq_tldr_cache`.
7. Insert chapters and chunks, set the book `published`, `XACK`.

## 4. Reliability

- At-least-once delivery: `XACK` only after success; entries idle longer than `RECLAIM_IDLE_MS` (default 300000) are reclaimed with `XAUTOCLAIM`.
- Three attempts per message; afterwards the entry moves to the dead-letter stream with its error and is acknowledged, so one corrupt file never stalls the queue. Replays are idempotent thanks to the `tldr_cache` upsert.
- Gemini calls retry transient failures up to three times with quadratic backoff (1 s, 4 s, 9 s, up to 20% jitter), honour `Retry-After` (capped at 60 s), and fail at once on 4xx other than 429. Timeouts: 60 s for embeddings, 120 s for generation. Logs carry the job id and model, never the key, which travels in `x-goog-api-key`.
- The Redis socket timeout (30 s) exceeds the `XREADGROUP` block window, so long polls return empty instead of raising.
- A cover failure never fails the job.

## 5. Operations

```bash
make worker-install    # uv venv + requirements
make worker            # long-running consumer
make worker-once       # one message, then exit
make worker-test       # 16 unit tests, no services
make worker-test-live  # dead-letter end-to-end, needs make db-up
```

Environment: the same names as `.env` plus `LLM_MODEL_NAME` and `RECLAIM_IDLE_MS`.

## 6. Scaling notes

- Vector search is scoped per book (`WHERE book_id = $1` over the HNSW index), so index memory follows the active book rather than the corpus.
- Add consumers to add throughput; autoscaling can key on `XLEN stream:epub_ingestion`.
- Not done yet: stream trimming (`XTRIM`) and read replicas for catalog queries. Tracked in `knowledge/tasks`.
