#!/usr/bin/env python3
"""Purge test debris from shared dev services (dev-only, never production).

Removes, in order:
  1. Pending (unacked) stream entries whose storage object is a PKfake
     (11/24-byte probe bodies) — acked, deleted, job hash removed.
  2. `draft` book rows titled 'Sherlock Test' / 'scan' (admin_test probes).
  3. PKfake objects (11/24 bytes) under raw-epubs/ in the EPUB bucket.
  4. Rows left by test seeds that panicked before `TestHarness::cleanup`:
     books whose cover points at a test host or whose source is a test
     marker, and users with test-only email domains. Cascades take their
     chapters, chunks, progress, quotes, and cache rows.

Leaves untouched: real published books (worker-ingested covers live in the
covers bucket), the dev seed catalog (`make seed-dev`), the DLQ stream, and
job hashes of surviving jobs.

Usage: make purge-test-debris   (loads .env via the Makefile)
"""

import os
import subprocess
import sys

import redis
from minio import Minio

STREAM = "stream:epub_ingestion"
GROUP = "ingestion-workers"
FAKE_SIZES = (11, 24)
FAKE_TITLES = ("Sherlock Test", "scan")
TEST_COVER_HOSTS = ("https://assets.baca.local/%", "https://example.com/%")
TEST_COVER_PATHS = ("/covers/test.jpg",)
TEST_SOURCE_NAMES = ("Test", "Test Source", "Draft")
TEST_EMAIL_DOMAINS = ("baca.local", "example.com", "test.local")


def psql(sql: str) -> str:
    out = subprocess.run(
        ["docker", "exec", "project_baca_db", "psql", "-U", "baca_user",
         "-d", "project_baca_db", "-tAc", sql],
        capture_output=True, text=True, check=True,
    )
    return out.stdout.strip()


def purge_seed_survivors() -> tuple[str, str]:
    """Deletes books and users that only test seeds create; returns counts."""
    hosts = " OR ".join(f"cover_url LIKE '{h}'" for h in TEST_COVER_HOSTS)
    paths = ",".join(f"'{p}'" for p in TEST_COVER_PATHS)
    sources = ",".join(f"'{s}'" for s in TEST_SOURCE_NAMES)
    books = psql(
        "WITH gone AS (DELETE FROM books "
        f"WHERE {hosts} OR cover_url IN ({paths}) OR source_name IN ({sources}) "
        "RETURNING 1) SELECT count(*) FROM gone"
    )
    domains = " OR ".join(f"email LIKE '%@{d}'" for d in TEST_EMAIL_DOMAINS)
    users = psql(
        f"WITH gone AS (DELETE FROM users WHERE {domains} RETURNING 1) SELECT count(*) FROM gone"
    )
    # Tags only exist alongside books; one without any book is seed debris.
    psql(
        "DELETE FROM tags WHERE NOT EXISTS (SELECT 1 FROM book_tags bt WHERE bt.tag_id = tags.id)"
    )
    return books, users


def main() -> int:
    red = redis.Redis.from_url(os.environ.get("REDIS_URL", "redis://localhost:6380/0"), decode_responses=True)
    minio_client = Minio(
        os.environ.get("S3_ENDPOINT", "http://localhost:9005").replace("http://", "").replace("https://", ""),
        access_key=os.environ.get("S3_ACCESS_KEY_ID", "minioadmin"),
        secret_key=os.environ.get("S3_SECRET_ACCESS_KEY", "minioadmin"),
        secure=os.environ.get("S3_ENDPOINT", "").startswith("https://"),
    )
    bucket = os.environ.get("S3_BUCKET_EPUBS", "baca-epubs")

    # Fake objects first (their names identify the orphan stream entries).
    fake_objs = {
        o.object_name
        for o in minio_client.list_objects(bucket, prefix="raw-epubs/", recursive=True)
        if o.size in FAKE_SIZES
    }
    for name in fake_objs:
        minio_client.remove_object(bucket, name)

    # Pending entries pointing at fake objects.
    purged_pending = 0
    for entry in red.xpending_range(STREAM, GROUP, "-", "+", 500):
        mid = entry["message_id"]
        for _, fields in red.xrange(STREAM, mid, mid):
            if fields.get("storage_path") in fake_objs:
                try:
                    red.xack(STREAM, GROUP, mid)
                except Exception:  # noqa: BLE001 - ack best-effort
                    pass
                red.xdel(STREAM, mid)
                if fields.get("job_id"):
                    red.delete(f"job:{fields['job_id']}")
                purged_pending += 1

    # Draft probe rows.
    titles = ",".join(f"'{t}'" for t in FAKE_TITLES)
    out = subprocess.run(
        ["docker", "exec", "project_baca_db", "psql", "-U", "baca_user",
         "-d", "project_baca_db", "-tAc",
         f"DELETE FROM books WHERE status='draft' AND title IN ({titles})"],
        capture_output=True, text=True,
    )
    seed_books, seed_users = purge_seed_survivors()
    print(f"fake objects deleted: {len(fake_objs)}")
    print(f"pending entries purged: {purged_pending}")
    print(f"draft rows deleted: {out.stdout.strip()}")
    print(f"test-seed books deleted: {seed_books}")
    print(f"test-seed users deleted: {seed_users}")
    print(f"pending now: {red.xpending(STREAM, GROUP)['pending']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
