"""Live worker tests: poison jobs reach the DLQ after 3 attempts.

Needs Redis + Postgres running (skipped otherwise). Never touches MinIO:
the poison references a nonexistent book, so the pipeline fails before any
download. Run: `RECLAIM_IDLE_MS=0 .venv/bin/python -m unittest test_worker_live -v`
"""

import os
import subprocess
import sys
import unittest
import uuid

import redis

STREAM = "stream:epub_ingestion"
DLQ = "stream:epub_ingestion:dlq"


def services_up() -> bool:
    try:
        redis.Redis.from_url(
            os.environ.get("REDIS_URL", "redis://localhost:6380/0"),
            socket_timeout=2,
        ).ping()
        return True
    except Exception:  # noqa: BLE001 - absence means "skip", not failure
        return False


@unittest.skipUnless(services_up(), "live Redis required")
class DlqTest(unittest.TestCase):
    def test_poison_job_reaches_dlq(self):
        client = redis.Redis.from_url(
            os.environ.get("REDIS_URL", "redis://localhost:6380/0"),
            decode_responses=True,
        )
        job_id = f"dlq-probe-{uuid.uuid4()}"
        book_id = str(uuid.uuid4())
        try:
            client.xadd(
                STREAM,
                {
                    "book_id": book_id,
                    "storage_path": "raw-epubs/does-not-exist.epub",
                    "job_id": job_id,
                    "timestamp": "probe",
                },
            )
            # Two virtual attempts already recorded: the next failure is the
            # third, so one worker run must move the job to the DLQ.
            client.hset(f"job:{job_id}", mapping={"attempts": "2"})
            worker = os.path.join(os.path.dirname(__file__), "worker.py")
            env = dict(os.environ, RECLAIM_IDLE_MS="0")
            subprocess.run(
                [sys.executable, worker, "--once"],
                env=env,
                capture_output=True,
                timeout=120,
                check=False,
            )
            dlq_entries = client.xrange(DLQ, "-", "+", count=100)
            self.assertTrue(
                any(
                    fields.get("job_id") == job_id
                    for _, fields in dlq_entries
                ),
                "poison job must land in the DLQ",
            )
            self.assertEqual(client.hget(f"job:{job_id}", "status"), "failed")
        finally:
            for key in (f"job:{job_id}",):
                client.delete(key)
            for msg_id, fields in client.xrange(DLQ, "-", "+", count=100):
                if fields.get("job_id") == job_id:
                    client.xdel(DLQ, msg_id)


if __name__ == "__main__":
    unittest.main()
