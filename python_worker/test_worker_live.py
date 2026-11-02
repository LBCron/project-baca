"""Live worker tests: dead letters, claim refresh, and graceful shutdown.

Needs Redis, Postgres, and MinIO running (skipped without Redis). Every test
works on its own probe stream and group, so the shared dev backlog is never
consumed. Run: `.venv/bin/python -m unittest test_worker_live -v`
"""

import os
import signal
import socket
import threading
import time
import unittest
import uuid

import redis

import worker

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
        settings = probe_settings(reclaim_idle_ms=0)
        job_id = f"dlq-probe-{uuid.uuid4()}"
        try:
            worker.ensure_group(client, settings)
            client.xadd(
                settings.stream,
                {
                    "book_id": str(uuid.uuid4()),
                    "storage_path": "raw-epubs/does-not-exist.epub",
                    "job_id": job_id,
                    "timestamp": "probe",
                },
            )
            # Two virtual attempts already recorded: the next failure is the
            # third, so one worker run must move the job to the DLQ.
            client.hset(f"job:{job_id}", mapping={"attempts": "2"})
            worker.run(once=True, settings=settings)
            dlq_entries = client.xrange(settings.dlq_stream, "-", "+", count=100)
            self.assertTrue(
                any(fields.get("job_id") == job_id for _, fields in dlq_entries),
                "poison job must land in the DLQ",
            )
            self.assertEqual(client.hget(f"job:{job_id}", "status"), "failed")
            self.assertEqual(client.xpending(settings.stream, settings.group)["pending"], 0)
        finally:
            client.delete(f"job:{job_id}", settings.stream, settings.dlq_stream)


def probe_settings(**overrides) -> worker.Settings:
    """Settings on a throwaway stream and group, so a probe never consumes
    the shared dev backlog or leaves entries in the real streams."""
    tag = uuid.uuid4().hex[:8]
    values = dict(
        stream=f"stream:probe:{tag}",
        dlq_stream=f"stream:probe:{tag}:dlq",
        group=f"probe-{tag}",
    )
    values.update(overrides)
    return worker.Settings(**values)


def silent_listener() -> tuple[int, socket.socket]:
    """Accepts connections and never answers, like a wedged object store."""
    server = socket.socket()
    server.bind(("127.0.0.1", 0))
    server.listen()
    held = []

    def accept_forever():
        while True:
            try:
                held.append(server.accept()[0])
            except OSError:
                return

    threading.Thread(target=accept_forever, daemon=True).start()
    return server.getsockname()[1], server


@unittest.skipUnless(services_up(), "live Redis required")
class ClaimAndShutdownLiveTest(unittest.TestCase):
    def setUp(self):
        self.client = redis.Redis.from_url(
            os.environ.get("REDIS_URL", "redis://localhost:6380/0"),
            decode_responses=True,
        )

    def cleanup(self, settings, *keys):
        self.client.delete(settings.stream, settings.dlq_stream, *keys)

    def test_a_job_longer_than_the_reclaim_window_is_not_taken(self):
        settings = probe_settings(reclaim_idle_ms=1500)
        try:
            worker.ensure_group(self.client, settings)
            msg_id = self.client.xadd(settings.stream, {"job_id": "probe"})
            self.client.xreadgroup(settings.group, "replica-a", {settings.stream: ">"}, count=1)
            with worker.keep_claimed(self.client, settings, "replica-a", msg_id):
                time.sleep(3.5)
                _, stolen, *_ = self.client.xautoclaim(
                    settings.stream, settings.group, "replica-b", min_idle_time=1500, start_id="0-0"
                )
            self.assertEqual(stolen, [], "a live job must stay with its worker")
            time.sleep(1.8)
            reclaimed = worker.reclaim_idle(self.client, settings, "replica-b")
            self.assertEqual([entry[0] for entry in reclaimed], [msg_id], "a dead worker's job is reclaimed")
        finally:
            self.cleanup(settings)

    def test_sigterm_mid_job_releases_it_within_ten_seconds(self):
        port, listener = silent_listener()
        settings = probe_settings(s3_endpoint=f"http://127.0.0.1:{port}")
        job_id = f"shutdown-probe-{uuid.uuid4()}"
        previous = {sig: signal.getsignal(sig) for sig in (signal.SIGTERM, signal.SIGINT)}
        try:
            worker.ensure_group(self.client, settings)
            self.client.xadd(
                settings.stream,
                {"book_id": str(uuid.uuid4()), "storage_path": "raw-epubs/x.epub", "job_id": job_id},
            )
            threading.Timer(1.5, os.kill, (os.getpid(), signal.SIGTERM)).start()
            started = time.monotonic()
            with self.assertRaises(worker.WorkerShutdown):
                worker.run(once=True, settings=settings)
            self.assertLess(time.monotonic() - started, 10)
            job = self.client.hgetall(f"job:{job_id}")
            self.assertEqual(job["status"], "queued")
            self.assertEqual(job["attempts"], "0", "an interrupted run is not an attempt")
            pending = self.client.xpending(settings.stream, settings.group)
            self.assertEqual(pending["pending"], 1, "the entry stays for another worker")
        finally:
            for sig, handler in previous.items():
                signal.signal(sig, handler)
            listener.close()
            self.cleanup(settings, f"job:{job_id}")


if __name__ == "__main__":
    unittest.main()
