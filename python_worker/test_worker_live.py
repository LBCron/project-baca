"""Live worker tests: dead letters, claim refresh, and graceful shutdown.

Needs Redis, Postgres, and MinIO running (skipped without Redis). Every test
works on its own probe stream and group, so the shared dev backlog is never
consumed. Run: `.venv/bin/python -m unittest test_worker_live -v`
"""

import io
import os
import signal
import socket
import threading
import time
import unittest
import uuid
from unittest import mock

import redis

import worker
from test_worker import make_epub

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


def database_up() -> bool:
    try:
        worker.make_db(worker.Settings()).close()
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
            reclaimed, _ = worker.reclaim_idle(self.client, settings, "replica-b")
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


@unittest.skipUnless(services_up() and database_up(), "live Redis and Postgres required")
class ArchiveMidIngestionLiveTest(unittest.TestCase):
    """A curator archives a book while its ingestion is summarizing (Gemini
    slowed to seconds per call): the archive answers at once, and the job
    leaves the book archived instead of publishing it."""

    def test_archiving_mid_ingestion_answers_within_three_seconds(self):
        settings = probe_settings()
        client = redis.Redis.from_url(settings.redis_url, decode_responses=True)
        book_id, job_id = str(uuid.uuid4()), f"archive-probe-{uuid.uuid4()}"
        conn = worker.make_db(settings)
        cur = conn.cursor()
        # "Unknown" makes the job write the title back, as a real upload
        # does; that write used to hold the row until the job committed.
        cur.execute(
            """INSERT INTO books (id, title, author, language, primary_theme, description,
                                  cover_url, epub_storage_path, source_name, status)
               VALUES (%s, 'Unknown', 'Unknown', 'en', 'Probe', 'probe', '', 'p', 'probe', 'draft')""",
            (book_id,),
        )
        conn.commit()
        summarizing = threading.Event()

        def slow_summary(*_args, **_kwargs):
            summarizing.set()
            time.sleep(4)
            return {}

        minio = mock.Mock()
        minio.get_object.return_value = io.BytesIO(make_epub([("One", "Some words."), ("Two", "More words.")]))
        minio.get_object.return_value.release_conn = lambda: None
        vector = [1.0] + [0.0] * (settings.embedding_dim - 1)
        try:
            with mock.patch.object(worker, "make_minio", return_value=minio), \
                 mock.patch.object(worker, "embed_batch", side_effect=lambda texts, *_: [vector] * len(texts)), \
                 mock.patch.object(worker, "generate_json", side_effect=slow_summary):
                job = threading.Thread(
                    target=worker.process_message,
                    args=("1-0", {"book_id": book_id, "storage_path": "p", "job_id": job_id}, settings),
                )
                job.start()
                self.assertTrue(summarizing.wait(30), "the job reaches its summaries")
                admin = worker.make_db(settings)
                started = time.monotonic()
                admin_cur = admin.cursor()
                admin_cur.execute("SET lock_timeout = '2s'")
                admin_cur.execute(
                    "UPDATE books SET status = 'archived', updated_at = NOW() WHERE id = %s", (book_id,)
                )
                admin.commit()
                waited = time.monotonic() - started
                admin.close()
                job.join(60)
            self.assertLess(waited, 3, f"archiving waited {waited:.1f} s on the job")
            cur.execute("SELECT status, title FROM books WHERE id = %s", (book_id,))
            self.assertEqual(cur.fetchone(), ["archived", "Test Book"])
            conn.commit()
            self.assertEqual(client.hget(f"job:{job_id}", "status"), "failed")
            self.assertEqual(client.hget(f"job:{job_id}", "error"), worker.ARCHIVED_ERROR)
        finally:
            cur.execute("DELETE FROM books WHERE id = %s", (book_id,))
            conn.commit()
            conn.close()
            client.delete(f"job:{job_id}")


@unittest.skipUnless(services_up(), "live Redis required")
class JobTtlLiveTest(unittest.TestCase):
    def test_every_job_hash_the_worker_writes_expires(self):
        client = redis.Redis.from_url(
            os.environ.get("REDIS_URL", "redis://localhost:6380/0"), decode_responses=True
        )
        job_id = f"ttl-probe-{uuid.uuid4()}"
        key = f"job:{job_id}"
        try:
            # A hash the server let expire comes back with a TTL from every path.
            for write in (
                lambda: worker.set_job(client, job_id, "parsing", 5),
                lambda: worker.release_job(client, job_id),
                lambda: worker.write_job(client, job_id, lambda pipe, k: pipe.hincrby(k, "attempts", 1)),
            ):
                client.delete(key)
                write()
                self.assertGreater(client.ttl(key), worker.JOB_TTL_SECONDS - 60)
        finally:
            client.delete(key)


@unittest.skipUnless(services_up() and database_up(), "live Redis and Postgres required")
class FailureRecoveryLiveTest(unittest.TestCase):
    def setUp(self):
        self.client = redis.Redis.from_url(
            os.environ.get("REDIS_URL", "redis://localhost:6380/0"), decode_responses=True
        )
        self.settings = probe_settings(reclaim_idle_ms=500)
        worker.ensure_group(self.client, self.settings)
        self.jobs: list[str] = []

    def tearDown(self):
        self.client.delete(
            self.settings.stream, self.settings.dlq_stream, *[f"job:{j}" for j in self.jobs]
        )

    def pending_under_a_dead_replica(self, count: int) -> list[str]:
        """Entries delivered to a consumer that then died, as after a crash."""
        jobs = [f"probe-{uuid.uuid4()}" for _ in range(count)]
        for job_id in jobs:
            self.client.xadd(
                self.settings.stream,
                {"book_id": str(uuid.uuid4()), "storage_path": f"raw-epubs/{job_id}.epub", "job_id": job_id},
            )
        self.client.xreadgroup(self.settings.group, "dead-replica", {self.settings.stream: ">"}, count=count)
        self.jobs += jobs
        return jobs

    def test_a_released_job_is_not_starved_by_an_old_backlog(self):
        self.pending_under_a_dead_replica(100)
        (released,) = self.pending_under_a_dead_replica(1)
        time.sleep(self.settings.reclaim_idle_ms / 1000)
        started = time.monotonic()
        while self.client.hget(f"job:{released}", "status") != "failed" and time.monotonic() - started < 15:
            worker.run(once=True, settings=self.settings)
        self.assertLess(time.monotonic() - started, 10)
        self.assertIn("gone from storage", self.client.hget(f"job:{released}", "error"))
        self.assertEqual(self.client.xlen(self.settings.dlq_stream), 101, "each missing upload fails at once")
        self.assertEqual(self.client.xpending(self.settings.stream, self.settings.group)["pending"], 0)

    def test_a_job_that_killed_the_worker_three_times_is_not_run_again(self):
        (job_id,) = self.pending_under_a_dead_replica(1)
        self.client.hset(f"job:{job_id}", mapping={"attempts": str(worker.MAX_ATTEMPTS)})
        time.sleep(self.settings.reclaim_idle_ms / 1000)
        with mock.patch.object(worker, "process_message") as run_job:
            worker.run(once=True, settings=self.settings)
        run_job.assert_not_called()
        (_, letter), = self.client.xrange(self.settings.dlq_stream)
        self.assertEqual(letter["job_id"], job_id)
        self.assertIn("stopped 3 times", letter["error"])
        self.assertEqual(self.client.hget(f"job:{job_id}", "status"), "failed")
        self.assertEqual(self.client.xpending(self.settings.stream, self.settings.group)["pending"], 0)


if __name__ == "__main__":
    unittest.main()
