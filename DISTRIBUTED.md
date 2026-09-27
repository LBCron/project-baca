---
type: Technical Architecture Specification
title: "Project Baca — Distributed Worker, Ingestion & Scaling Architecture"
description: "Distributed compute architecture for EPUB ingestion, Redis Streams queues, Dual-Mode Embeddings, and horizontal scaling."
tags: [distributed, worker, ingestion, redis-streams, embedding, gemini, fastembed, scalability, okf]
---

# DISTRIBUTED.md — Distributed Processing & Ingestion Pipeline

Distributed architecture, asynchronous queue management, and horizontal scaling strategy for **Project Baca**.

## 1. Processing Topology

Clear separation between stateless HTTP API nodes and background compute workers:

```text
+-------------------+       +-------------------+       +-------------------+       +-----------------------+
|  Web/PWA Client   | ----> |  Caddy Edge Gate  | ----> |  Axum API Server  | ----> |  Redis 7 Streams      |
|  (Leptos WASM)    | <==== |  HTTP/3 QUIC Edge | <---- |  (Stateless Node) |       |  Stream: stream:epub_ingestion |
+-------------------+       +-------------------+       +-------------------+       +-----------------------+

                                                                    |
                                                                    v
+-----------------------+       +-------------------+       +-----------------------+
|  Dual-Mode Embedding  | <==== |  Python Ingestion | <==== |  Consumer Group:      |
|  - Gemini API (Cloud) |       |  Worker Instances |       |  ingestion-workers    |
|  - FastEmbed CPU (ARM)| ====> |  (Horizontal Pods)|       |  - Auto-claim pending |
+-----------------------+       +-------------------+       +-----------------------+
                                          |
                                          v
                                +-------------------+
                                |  PostgreSQL 17    |
                                |  + pgvector HNSW  |
                                |  + MinIO / S3 R2  |
                                +-------------------+
```

## 2. Asynchronous EPUB Ingestion Pipeline

Public domain manuscript ingestion executes in 5 isolated phases:

1. **Upload & Object Storage:**
   - Admin uploads `.epub` file via `/api/admin/books/upload`.
   - Axum validates file magic bytes, saves raw EPUB to `baca-epubs/raw/<book_id>.epub` in S3/MinIO, and pushes event to Redis Stream `stream:epub_ingestion`.
2. **Parsing & Sanitization (Python Worker):**
   - Worker consumes task from `ingestion-workers` consumer group via `XREADGROUP`.
   - Validates OPF metadata, extracts cover image to `baca-covers/`, and cleans HTML (stripping malicious tags, inline scripts, and styling).
3. **Scene-based & Semantic Chunking:**
   - Splits chapter text into semantic chunks of 300–500 words at natural paragraph/scene boundaries.
   - Computes DOM CFI for precise reader deep-linking.
4. **Batch Vectorization via Dual-Mode Embeddings (768-dim):**
   - **Cloud Mode (Default):** Google GenAI API (`text-embedding-004`). Generates 768-dimensional embeddings without local GPU requirements.
   - **Local / Offline Mode:** CPU-optimized ONNX runtime via `fastembed` (supports ARM64 NEON / Apple Silicon). Low RAM usage (<150 MB) and small footprint (~60 MB).
   - Standard 768 dimensions cut PostgreSQL HNSW index memory consumption in half compared to 1536-dimensional models.
5. **Bulk Insert & Indexing:**
   - Bulk inserts chunk records and vectors into `book_chunks`.
   - Updates book status to `published` in `books`.

## 3. Scoped Vector Search Strategy

Vector search memory scales with corpus size. Project Baca uses a scoped search pattern:

* **Scoped Search Pattern (`WHERE book_id = $1`):**
  - Rather than searching the entire multi-book corpus, semantic quote searches are scoped to the active book:
    ```sql
    SELECT id, chapter_id, content, cfi_range, 1 - (embedding <=> $2) AS similarity
    FROM book_chunks
    WHERE book_id = $1
    ORDER BY embedding <=> $2
    LIMIT 5;
    ```
  - Isolating searches keeps HNSW memory overhead minimal while scaling to hundreds of thousands of chunks on disk.

## 4. Fault Tolerance & Queue Reliability

1. **Message Acknowledgment:** Tasks are only removed from the pending list after worker calls `XACK`.
2. **Dead Worker Recovery:** Supervisor monitors `XPENDING` every 60 seconds; uncompleted tasks past 5 minutes are reclaimed via `XCLAIM`.
3. **Dead-Letter Queue (DLQ):** Corrupt files failing after 3 retries move to `stream:epub_ingestion:dlq` for curator inspection without stalling the main queue.

## 5. Horizontal Scaling Roadmap (Phase 2)

* **Worker Autoscaling:** Scale Python worker pods based on queue length (`XLEN stream:epub_ingestion`) via Kubernetes KEDA.
* **Read Replicas:** Route FTS catalog queries to PostgreSQL read replicas, preserving the primary database for write transactions and reading progress updates.
* **Edge Caching:** Cache sanitized HTML chapters at Cloudflare edge locations (7-day TTL) for sub-20ms reader access globally.
