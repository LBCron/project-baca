"""Unit tests for the ingestion worker's pure helpers (stdlib only).

Run: `.venv/bin/python -m unittest test_worker -v`
No Redis, MinIO, Postgres, or network access required.
"""

import io
import unittest
import urllib.error
import zipfile
from unittest import mock

import worker
from worker import (
    backoff_seconds,
    build_recap_input,
    check_archive_limits,
    chunk_words,
    extract_text,
    parse_epub,
    sanitize_html,
)


def make_epub(
    chapters: list[tuple[str, str]],
    title: str = "Test Book",
    author: str = "Test Author",
    language: str = "en",
    with_cover: bool = False,
) -> bytes:
    """Builds a minimal valid EPUB in memory."""
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("mimetype", "application/epub+zip")
        archive.writestr(
            "META-INF/container.xml",
            """<?xml version="1.0"?>
            <container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
              <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
            </container>""",
        )
        manifest_items = []
        spine_items = []
        for index, (heading, body) in enumerate(chapters):
            item_id = f"chap{index}"
            manifest_items.append(
                f'<item id="{item_id}" href="chap{index}.xhtml" media-type="application/xhtml+xml"/>'
            )
            spine_items.append(f'<itemref idref="{item_id}"/>')
            archive.writestr(
                f"OEBPS/chap{index}.xhtml",
                f"""<?xml version="1.0" encoding="utf-8"?>
                <html xmlns="http://www.w3.org/1999/xhtml"><head><title>{heading}</title></head>
                <body><h1>{heading}</h1><p>{body}</p></body></html>""",
            )
        cover_item = ""
        if with_cover:
            from PIL import Image

            image = Image.new("RGB", (8, 8), color="red")
            cover_buffer = io.BytesIO()
            image.save(cover_buffer, format="PNG")
            archive.writestr("OEBPS/cover.png", cover_buffer.getvalue())
            cover_item = '<item id="cover-image" href="cover.png" media-type="image/png" properties="cover-image"/>'
            manifest_items.append(cover_item)
        archive.writestr(
            "OEBPS/content.opf",
            f"""<?xml version="1.0" encoding="utf-8"?>
            <package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid">
              <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
                <dc:title>{title}</dc:title>
                <dc:creator>{author}</dc:creator>
                <dc:language>{language}</dc:language>
              </metadata>
              <manifest>{''.join(manifest_items)}</manifest>
              <spine>{''.join(spine_items)}</spine>
            </package>""",
        )
    return buffer.getvalue()


class SanitizeTest(unittest.TestCase):
    def test_strips_scripts_and_handlers_but_keeps_semantics(self):
        dirty = (
            '<p onclick="evil()">Hello <em>world</em></p>'
            '<script>alert(1)</script><style>.x{}</style>'
            '<iframe src="x"></iframe><div>plain</div>'
        )
        clean = sanitize_html(dirty)
        self.assertIn("<p>Hello <em>world</em></p>", clean)
        self.assertNotIn("<script", clean)
        self.assertNotIn("onclick", clean)
        self.assertNotIn("<iframe", clean)
        self.assertNotIn("<div", clean)
        self.assertIn("plain", clean)

    def test_entity_encoded_markup_stays_inert_text(self):
        dirty = "<p>&lt;img src=x onerror=alert(1)&gt; &amp; Tom &amp;amp; Jerry</p>"
        clean = sanitize_html(dirty)
        self.assertNotIn("<img", clean)
        self.assertIn("&lt;img src=x onerror=alert(1)&gt;", clean)
        self.assertIn("&amp; Tom &amp;amp; Jerry", clean)
        self.assertEqual(extract_text(clean), "<img src=x onerror=alert(1)> & Tom &amp; Jerry")

    def test_a_void_embed_does_not_hide_what_follows(self):
        self.assertEqual(sanitize_html("<p>a</p><embed src=x><p>b</p>"), "<p>a</p><p>b</p>")
        self.assertEqual(sanitize_html("<embed src=x>tail text"), "tail text")

    def test_an_unclosed_skipped_element_ends_with_its_parent(self):
        dirty = "<div><p>a</p><iframe src=x><p>inside</p></div><p>b</p><object><p>c</p></object><p>d</p>"
        self.assertEqual(sanitize_html(dirty), "<p>a</p><p>b</p><p>d</p>")
        nested = "<section><object><iframe>x</iframe><p>still hidden</p></object><p>e</p></section>"
        self.assertEqual(sanitize_html(nested), "<p>e</p>")
        self.assertEqual(sanitize_html('<div><iframe src="x"/><p>f</p></div>'), "<p>f</p>")
        self.assertEqual(
            sanitize_html("<html><head><title>T</title></head><body><p>g</p></body></html>"),
            "<p>g</p>",
        )

    def test_no_markup_outside_the_allowlist_survives_random_html(self):
        """Whatever nesting, unclosed or stray tags arrive, the output holds
        only bare allowlisted tags; every other ``<`` and ``>`` is text."""
        import random
        import re

        allowed = re.compile(r"</?(p|em|strong|blockquote|h1|h2|h3|ul|ol|li)>|<br/>|<hr>")
        tags = ["p", "em", "iframe", "object", "embed", "script", "style", "title", "head", "div",
                "img", "br", "hr", "noscript", "xmp", "textarea", "svg", "math", "a", "plaintext"]
        texts = ["text", "&lt;b&gt;", "a<b", "x>y", "&amp;", "<", ">", "<!--", "-->", "<![CDATA[", "<?x ?>"]
        rng = random.Random(11)

        def fragment(depth=0):
            parts = []
            for _ in range(rng.randint(1, 6)):
                roll, tag = rng.random(), rng.choice(tags)
                if roll < 0.3:
                    parts.append(f"<{tag} onerror=alert(1) src=x>")
                elif roll < 0.5:
                    parts.append(f"</{tag}>")
                elif roll < 0.6:
                    parts.append(f"<{tag}/>")
                elif roll < 0.75 and depth < 3:
                    parts.append(f"<{tag}>{fragment(depth + 1)}</{tag}>")
                else:
                    parts.append(rng.choice(texts))
            return "".join(parts)

        for _ in range(3_000):
            dirty = fragment()
            leftover = allowed.sub("", sanitize_html(dirty))
            self.assertNotRegex(leftover, r"[<>]", dirty)

    def test_stray_end_tags_cost_linear_time(self):
        """Forty thousand unmatched end tags took seconds when each one
        scanned the open-element stack."""
        import time

        dirty = "<b>" * 40_000 + "</i>" * 40_000
        started = time.monotonic()
        sanitize_html(dirty)
        self.assertLess(time.monotonic() - started, 1.0)

    def test_extract_text_resolves_entities(self):
        self.assertEqual(
            extract_text("<p>A &amp; B</p><script>hidden</script>"),
            "A & B",
        )


class ChunkTest(unittest.TestCase):
    def test_splits_at_target_and_merges_tail(self):
        words = [f"w{i}" for i in range(830)]
        chunks = chunk_words(" ".join(words), target=400, minimum=50)
        self.assertEqual(len(chunks), 2)
        self.assertEqual(len(chunks[0].split()), 400)
        self.assertEqual(len(chunks[1].split()), 430)

    def test_short_text_single_chunk(self):
        self.assertEqual(chunk_words("hello world"), ["hello world"])
        self.assertEqual(chunk_words(""), [])


class ParseTest(unittest.TestCase):
    def test_parses_metadata_and_spine_order(self):
        data = make_epub([("Alpha", "first body"), ("Beta", "second body")])
        parsed = parse_epub(data)
        self.assertEqual(parsed.title, "Test Book")
        self.assertEqual(parsed.author, "Test Author")
        self.assertEqual(parsed.language, "en")
        self.assertEqual([c.title for c in parsed.chapters], ["Alpha", "Beta"])
        self.assertIn("first body", parsed.chapters[0].text)
        self.assertEqual(parsed.chapters[0].word_count, 3)  # Alpha + first + body

    def test_extracts_cover_image(self):
        data = make_epub([("Only", "body here")], with_cover=True)
        parsed = parse_epub(data)
        self.assertIsNotNone(parsed.cover_bytes)
        self.assertTrue(parsed.cover_bytes.startswith(b"\x89PNG"))

    def test_rejects_non_zip(self):
        with self.assertRaises(ValueError):
            parse_epub(b"definitely not a zip file")

    def test_rejects_empty_spine(self):
        data = make_epub([])
        with self.assertRaises(ValueError):
            parse_epub(data)


def repeated_spine_epub(refs: int) -> bytes:
    """An EPUB whose spine names its one chapter ``refs`` times. The OPF is
    stored, not deflated: deflated it would trip the compression ratio check
    first and never reach the spine."""
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", zipfile.ZIP_DEFLATED) as archive:
        archive.writestr(
            "META-INF/container.xml",
            '<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container">'
            '<rootfiles><rootfile full-path="c.opf"/></rootfiles></container>',
        )
        words = " ".join(f"w{i}" for i in range(500))
        archive.writestr("one.xhtml", f"<html><body><h1>One</h1><p>{words}</p></body></html>")
        archive.writestr(
            "c.opf",
            '<package xmlns="http://www.idpf.org/2007/opf"><metadata/>'
            '<manifest><item id="c" href="one.xhtml" media-type="application/xhtml+xml"/></manifest>'
            "<spine>" + '<itemref idref="c"/>' * refs + "</spine></package>",
            compress_type=zipfile.ZIP_STORED,
        )
    return buffer.getvalue()


def many_entries_zip(entries: int) -> bytes:
    """A stored zip of ``entries`` empty files written record by record, so a
    400,000-entry archive costs its bytes here and no ZipInfo per entry."""
    import struct

    local, central = io.BytesIO(), io.BytesIO()
    for i in range(entries):
        name = f"{i:06d}".encode()
        offset = local.tell()
        local.write(struct.pack("<IHHHHHIIIHH", 0x04034B50, 20, 0, 0, 0, 0, 0, 0, 0, len(name), 0) + name)
        central.write(
            struct.pack("<IHHHHHHIIIHHHHHII", 0x02014B50, 20, 20, 0, 0, 0, 0, 0, 0, 0, len(name), 0, 0, 0, 0, 0, offset)
            + name
        )
    directory = central.getvalue()
    start = local.tell()
    count = min(entries, 0xFFFF)
    end = struct.pack("<IHHHHIIH", 0x06054B50, 0, 0, count, count, len(directory), start, 0)
    return local.getvalue() + directory + end


class LimitsTest(unittest.TestCase):
    def test_a_spine_naming_one_entry_many_times_is_one_chapter(self):
        for refs in (2_000, 10_000):
            self.assertEqual(len(parse_epub(repeated_spine_epub(refs)).chapters), 1, refs)

    def test_past_the_chapter_limit_is_rejected_with_its_reason(self):
        data = make_epub([(f"C{i}", "body words") for i in range(3)])
        with self.assertRaisesRegex(worker.RejectedEpub, "more than 2 chapters"):
            parse_epub(data, worker.EpubLimits(chapters=2))
        self.assertEqual(len(parse_epub(data, worker.EpubLimits(chapters=3)).chapters), 3)

    def test_past_the_text_limit_is_rejected(self):
        data = make_epub([("Long", "word " * 2_000)])
        with self.assertRaisesRegex(worker.RejectedEpub, "text exceeds"):
            parse_epub(data, worker.EpubLimits(text_bytes=5_000))

    def test_declared_entries_past_the_limit_are_refused_before_zipfile(self):
        data = many_entries_zip(10)
        with self.assertRaisesRegex(worker.RejectedEpub, "more than 5 files"):
            worker.check_central_directory(data, worker.EpubLimits(archive_files=5))
        worker.check_central_directory(data, worker.EpubLimits(archive_files=10))
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            self.assertEqual(len(archive.infolist()), 10, "the fixture is a real zip")

    def test_an_end_record_signature_in_the_offset_field_is_no_way_past(self):
        """``rfind`` on the signature used to land inside the real record
        when its offset field spelled ``PK\\x05\\x06`` and skip the check."""
        data = bytearray(many_entries_zip(6_000))
        data[-6:-2] = b"PK\x05\x06"
        with self.assertRaisesRegex(worker.RejectedEpub, "more than 5000 files"):
            worker.check_central_directory(bytes(data), worker.EpubLimits())

    def test_a_zip64_archive_behind_a_stub_with_false_counts_is_refused(self):
        """A prepended stub moves the ZIP64 record away from where its
        locator points; ZipFile still finds it, so the check must too."""
        import struct

        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w", zipfile.ZIP_STORED) as archive:
            for i in range(66_000):
                archive.writestr(str(i), b"")
        data = bytearray(b"MZ" + b"\0" * 4094 + buffer.getvalue())
        data[-14:-10] = struct.pack("<HH", 1, 1)
        data[-10:-6] = struct.pack("<I", 46)
        with self.assertRaisesRegex(worker.RejectedEpub, "more than 5000 files"):
            worker.check_central_directory(bytes(data), worker.EpubLimits())

    def test_xml_that_does_not_parse_is_rejected_with_its_reason(self):
        source = zipfile.ZipFile(io.BytesIO(make_epub([("One", "body words")])))
        out = io.BytesIO()
        with zipfile.ZipFile(out, "w") as archive:
            for info in source.infolist():
                data = source.read(info.filename)
                archive.writestr(info, b"<package><unclosed>" if info.filename.endswith(".opf") else data)
        with self.assertRaisesRegex(worker.RejectedEpub, "not well-formed XML"):
            parse_epub(out.getvalue())

    def test_a_400000_entry_archive_is_refused_with_little_memory(self):
        """Building every ZipInfo used to cost 469 MB just to refuse it."""
        import os
        import subprocess
        import sys
        import tempfile

        with tempfile.NamedTemporaryFile(suffix=".epub", delete=False) as bomb:
            bomb.write(many_entries_zip(400_000))
        here = os.path.dirname(os.path.abspath(__file__))
        probe = (
            "import resource, sys\n"
            f"sys.path.insert(0, {here!r})\n"
            "import worker\n"
            f"data = open({bomb.name!r}, 'rb').read()\n"
            "try:\n"
            "    worker.parse_epub(data)\n"
            "    print('parsed')\n"
            "except worker.RejectedEpub as exc:\n"
            "    print('rejected')\n"
            "print(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)\n"
        )
        try:
            out = subprocess.run(
                [sys.executable, "-I", "-c", probe], capture_output=True, text=True, check=True
            ).stdout.split()
        finally:
            os.unlink(bomb.name)
        self.assertEqual(out[0], "rejected")
        self.assertLess(int(out[1]) // 1024, 150, f"peak RSS {int(out[1]) // 1024} MB")

    def test_past_the_chunk_limit_fails_before_any_embedding(self):
        cursor = FakeCursor("draft")
        conn = mock.Mock()
        conn.cursor.return_value = cursor
        minio = mock.Mock()
        minio.get_object.return_value = io.BytesIO(make_epub([("Long", "word " * 1_000)]))
        minio.get_object.return_value.release_conn = lambda: None
        redis_client = mock.MagicMock()
        settings = worker.Settings(limits=worker.EpubLimits(chunks=1))
        with mock.patch.object(worker, "make_redis", return_value=redis_client), \
             mock.patch.object(worker, "make_minio", return_value=minio), \
             mock.patch.object(worker, "make_db", return_value=conn), \
             mock.patch.object(worker, "embed_batch") as embed:
            with self.assertRaisesRegex(worker.RejectedEpub, "more than 1 passages"):
                worker.process_message("1-0", {"book_id": "b", "storage_path": "p", "job_id": "j"}, settings)
        embed.assert_not_called()

    def test_a_rejected_upload_fails_at_once_without_retries(self):
        client, dead, _ = run_once_with(side_effect=worker.RejectedEpub("EPUB has more than 2000 chapters"))
        dead.assert_called_once()
        self.assertEqual(dead.call_args.args[3:], ("b", "EPUB has more than 2000 chapters"))
        client.xack.assert_called_once()
        status = client.pipeline.return_value.hset.call_args.kwargs["mapping"]
        self.assertEqual((status["status"], status["error"]), ("failed", "EPUB has more than 2000 chapters"))


def run_once_with(attempts: int = 1, **process):
    """One worker turn over one new entry, with the job's attempt counter at
    ``attempts`` after this delivery; returns the Redis mock and the
    dead_letter and process_message mocks."""
    client = mock.MagicMock()
    client.pipeline.return_value.execute.return_value = [attempts, True]
    client.xreadgroup.return_value = [("s", [("9-0", {"job_id": "j", "book_id": "b", "storage_path": "p"})])]
    with mock.patch.object(worker, "make_redis", return_value=client), \
         mock.patch.object(worker, "keep_claimed"), \
         mock.patch.object(worker, "process_message", **process) as run_job, \
         mock.patch.object(worker, "return_to_draft"), \
         mock.patch.object(worker, "dead_letter") as dead:
        worker.run(once=True, settings=worker.Settings())
    return client, dead, run_job


class FailureHandlingTest(unittest.TestCase):
    def test_a_missing_object_is_permanent(self):
        from minio.error import S3Error

        minio = mock.Mock()
        minio.get_object.side_effect = S3Error(None, "NoSuchKey", "gone", "r", "rid", "hid")
        with self.assertRaisesRegex(worker.PermanentJobError, "gone from storage"):
            worker.read_upload(minio, "bucket", "raw-epubs/x.epub")

    def test_a_job_past_its_attempts_is_dead_lettered_without_running(self):
        client, dead, run_job = run_once_with(attempts=worker.MAX_ATTEMPTS + 1)
        run_job.assert_not_called()
        dead.assert_called_once()
        self.assertIn("stopped 3 times", dead.call_args.args[4])
        client.xack.assert_called_once()

    def test_a_transient_failure_stays_pending_and_says_so(self):
        client, dead, _ = run_once_with(attempts=1, side_effect=ConnectionResetError("peer reset at 10.0.0.7:9000"))
        dead.assert_not_called()
        client.xack.assert_not_called()
        status = client.pipeline.return_value.hset.call_args.kwargs["mapping"]
        self.assertEqual(status["status"], "queued")
        self.assertIn("Unexpected ConnectionResetError", status["error"])
        self.assertNotIn("10.0.0.7", status["error"], "internal detail stays in the log")

    def test_the_last_transient_failure_is_dead_lettered(self):
        client, dead, _ = run_once_with(attempts=3, side_effect=RuntimeError("Gemini API unavailable (HTTP 503)"))
        self.assertEqual(dead.call_args.args[4], "Gemini API unavailable (HTTP 503)")
        client.xack.assert_called_once()

    def test_a_deleted_pending_entry_is_acknowledged_and_skipped(self):
        client = mock.MagicMock()
        client.xautoclaim.return_value = ["0-0", [("5-0", None)], []]
        with mock.patch.object(worker, "make_redis", return_value=client), \
             mock.patch.object(worker, "process_message") as process:
            worker.run(once=True, settings=worker.Settings())
        process.assert_not_called()
        client.xack.assert_called_once_with(worker.Settings().stream, worker.Settings().group, "5-0")

    def test_a_failed_book_goes_back_to_draft(self):
        conn = mock.Mock()
        with mock.patch.object(worker, "make_db", return_value=conn), \
             mock.patch.object(worker, "dead_letter"):
            worker.fail_job(mock.MagicMock(), worker.Settings(), "1-0", {"job_id": "j", "book_id": "b"}, "why")
        sql, params = conn.cursor.return_value.execute.call_args.args
        self.assertIn("status = 'draft'", sql)
        self.assertIn("status = 'processing'", sql)
        self.assertEqual(params, ("b",))
        conn.commit.assert_called_once()

    def test_reclaim_carries_its_cursor(self):
        client = mock.MagicMock()
        client.xautoclaim.return_value = ["7-0", [("3-0", {"job_id": "j"})], []]
        entries, cursor = worker.reclaim_idle(client, worker.Settings(), "c", "2-0")
        self.assertEqual((entries, cursor), ([("3-0", {"job_id": "j"})], "7-0"))
        self.assertEqual(client.xautoclaim.call_args.kwargs["start_id"], "2-0")


class ArchivedBeforeEmbeddingTest(unittest.TestCase):
    def test_a_book_archived_after_the_first_commit_spends_nothing_more(self):
        class ArchivedMidway(FakeCursor):
            def fetchone(self):
                archived = any("SELECT status FROM books" in sql for sql in self.statements)
                return ("Title", "Author", "en", "archived" if archived else "draft")

        cursor = ArchivedMidway("draft")
        conn = mock.Mock()
        conn.cursor.return_value = cursor
        minio = mock.Mock()
        minio.get_object.return_value = io.BytesIO(make_epub([("One", "Some text worth reading.")]))
        minio.get_object.return_value.release_conn = lambda: None
        redis_client = mock.MagicMock()
        with mock.patch.object(worker, "make_redis", return_value=redis_client), \
             mock.patch.object(worker, "make_minio", return_value=minio), \
             mock.patch.object(worker, "make_db", return_value=conn), \
             mock.patch.object(worker, "embed_batch") as embed, \
             mock.patch.object(worker, "generate_json") as generate:
            worker.process_message("1-0", {"book_id": "b", "storage_path": "p", "job_id": "j"}, worker.Settings())
        embed.assert_not_called()
        generate.assert_not_called()
        status = redis_client.pipeline.return_value.hset.call_args.kwargs["mapping"]
        self.assertEqual(status["error"], worker.ARCHIVED_ERROR)
        self.assertTrue(
            any("SET status = 'processing'" in sql for sql in cursor.statements),
            "a draft shows as processing once its job starts",
        )


class ParagraphIndexTest(unittest.TestCase):
    def test_paragraphs_are_numbered_as_a_browser_parses_them(self):
        html = (
            "<h2>Opening</h2><p>The secure\n   ancestral hall</p>"
            "<p>Tom &amp; Jerry<br/>again</p>"
            "<p>open<blockquote>quoted</blockquote></p>"
            "<ul><li>item</li></ul><p><em>last</em> one</p>"
        )
        # A blockquote closes the open <p>; its stray </p> opens an empty one.
        self.assertEqual(
            worker.paragraph_texts(html),
            ["The secure ancestral hall", "Tom & Jerry again", "open", "", "last one"],
        )

    def test_folding_matches_the_server(self):
        # The same literal is asserted for shared::fold_for_search.
        self.assertEqual(
            worker.fold_for_search("\u0130STANBUL \u039f\u0394\u03a5\u03a3\u03a3\u0395\u03a5\u03a3 \u03c2"),
            "istanbul \u03bf\u03b4\u03c5\u03c3\u03c3\u03b5\u03c5\u03c3 \u03c3",
        )

    def test_ingestion_writes_the_paragraph_index(self):
        cursor = FakeCursor("draft")
        cursor.fetchone = lambda: ("chapter-id",) if cursor.statements[-1].lstrip().startswith("INSERT INTO chapters") else ("Title", "Author", "en", "draft")
        params = []
        record = cursor.execute
        cursor.execute = lambda sql, p=(): (record(sql, p), params.append((sql, p)))
        conn = mock.Mock()
        conn.cursor.return_value = cursor
        conn.commit.side_effect = [None, RuntimeError("stop at publish")]
        minio = mock.Mock()
        minio.get_object.return_value = io.BytesIO(make_epub([("One", "Some text worth reading.")]))
        minio.get_object.return_value.release_conn = lambda: None
        with mock.patch.object(worker, "make_redis", return_value=mock.MagicMock()), \
             mock.patch.object(worker, "make_minio", return_value=minio), \
             mock.patch.object(worker, "make_db", return_value=conn), \
             mock.patch.object(worker, "embed_batch", side_effect=lambda texts, *_: [[0.0]] * len(texts)), \
             mock.patch.object(worker, "generate_json", return_value={}):
            with self.assertRaisesRegex(RuntimeError, "stop at publish"):
                worker.process_message("1-0", {"book_id": "b", "storage_path": "p", "job_id": "j"}, worker.Settings())
        inserts = [p for sql, p in params if sql.startswith("INSERT INTO chapter_paragraphs")]
        self.assertEqual(inserts, [["b", 1, 1, "Some text worth reading.", "some text worth reading."]])


class CorruptMemberTest(unittest.TestCase):
    def test_a_corrupt_chapter_rejects_the_upload(self):
        data = bytearray(make_epub([("One", "word " * 2_000)]))
        name = b"OEBPS/chap0.xhtml"
        # Overwrite deflated bytes just past the member's local header name.
        start = data.find(name) + len(name) + 40
        data[start : start + 40] = bytes(range(40))
        with self.assertRaisesRegex(worker.RejectedEpub, "chap0.xhtml' is corrupt"):
            worker.parse_epub(bytes(data))

    def test_a_job_failed_before_is_not_dead_lettered_twice(self):
        client = mock.MagicMock()
        client.hget.return_value = "failed"
        with mock.patch.object(worker, "dead_letter") as dead, \
             mock.patch.object(worker, "return_to_draft"):
            worker.fail_job(client, worker.Settings(), "1-0", {"job_id": "j", "book_id": "b"}, "why")
        dead.assert_not_called()
        client.xack.assert_called_once()


class MetadataBackfillTest(unittest.TestCase):
    def backfill(self, row: tuple, epub: bytes) -> dict:
        """Runs the pipeline to its first commit on a book row ``row`` and
        returns the columns the backfill UPDATE wrote."""

        class Recording(FakeCursor):
            def __init__(self):
                super().__init__("draft")
                self.calls = []

            def execute(self, sql, params=()):
                super().execute(sql, params)
                self.calls.append((sql, params))

            def fetchone(self):
                return row

        cursor = Recording()
        conn = mock.Mock()
        conn.cursor.return_value = cursor
        conn.commit.side_effect = RuntimeError("stop after the first commit")
        minio = mock.Mock()
        minio.get_object.return_value = io.BytesIO(epub)
        minio.get_object.return_value.release_conn = lambda: None
        with mock.patch.object(worker, "make_redis", return_value=mock.MagicMock()), \
             mock.patch.object(worker, "make_minio", return_value=minio), \
             mock.patch.object(worker, "make_db", return_value=conn):
            with self.assertRaisesRegex(RuntimeError, "first commit"):
                worker.process_message("1-0", {"book_id": "b", "storage_path": "p", "job_id": "j"}, worker.Settings())
        sql, params = next(c for c in cursor.calls if c[0].startswith("UPDATE books SET title"))
        columns = [part.split(" = ")[0].strip() for part in sql.split("SET", 1)[1].split(", updated_at")[0].split(",")]
        return dict(zip(columns, params))

    def test_a_blank_upload_takes_title_and_language_from_the_epub(self):
        written = self.backfill(("", "Unknown", "", "draft"), make_epub([("One", "Text")], title="Kisah Lama", language="id"))
        self.assertEqual((written["title"], written["language"]), ("Kisah Lama", "id"))

    def test_an_epub_without_a_title_gets_a_placeholder(self):
        written = self.backfill(("", "Unknown", "", "draft"), make_epub([("One", "Text")], title=""))
        self.assertEqual(written["title"], worker.UNTITLED)

    def test_curator_fields_are_kept(self):
        cursor_row = ("Judul Kurator", "Penulis", "en", "draft")
        with self.assertRaises(StopIteration):
            self.backfill(cursor_row, make_epub([("One", "Text")], title="Other", language="id"))


class ArchiveLimitsTest(unittest.TestCase):
    def test_rejects_oversized_archives(self):
        from types import SimpleNamespace

        huge = [SimpleNamespace(file_size=200_000_001)]
        with self.assertRaises(ValueError):
            check_archive_limits(huge)
        many = [SimpleNamespace(file_size=10)] * 5_001
        with self.assertRaises(ValueError):
            check_archive_limits(many)
        fine = [SimpleNamespace(file_size=1_000_000)] * 10
        check_archive_limits(fine)  # must not raise


def png_of(width: int, height: int) -> bytes:
    """A valid all-black grayscale PNG built row by row, so a 13000x13000
    image costs a few hundred KiB and never 169 MB of pixels here."""
    import struct
    import zlib

    def chunk(kind: bytes, data: bytes) -> bytes:
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

    squeeze = zlib.compressobj(9)
    row = b"\x00" * (width + 1)
    idat = b"".join(squeeze.compress(row) for _ in range(height)) + squeeze.flush()
    header = struct.pack(">IIBBBBB", width, height, 8, 0, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", idat) + chunk(b"IEND", b"")


def image_bytes(fmt: str, size=(8, 8)) -> bytes:
    from PIL import Image

    buffer = io.BytesIO()
    Image.new("RGB", size, color="red").save(buffer, format=fmt)
    return buffer.getvalue()


class CoverTest(unittest.TestCase):
    def test_allowed_formats_become_webp_within_the_edge(self):
        for fmt in ("PNG", "JPEG", "GIF", "WEBP"):
            webp = worker.convert_cover_to_webp(image_bytes(fmt))
            self.assertEqual((webp[:4], webp[8:12]), (b"RIFF", b"WEBP"), fmt)
        from PIL import Image

        wide = worker.convert_cover_to_webp(image_bytes("PNG", (4000, 100)))
        with Image.open(io.BytesIO(wide)) as shown:
            self.assertEqual(max(shown.size), worker.COVER_MAX_EDGE)

    def test_other_formats_never_reach_a_decoder(self):
        refused = {
            "svg": b'<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8"/>',
            "tiff": image_bytes("TIFF"),
            "bmp": image_bytes("BMP"),
            "jp2": b"\x00\x00\x00\x0cjP  \r\n\x87\n" + b"\x00" * 64,
        }
        for name, data in refused.items():
            with self.assertRaises(Exception, msg=name):
                worker.convert_cover_to_webp(data)

    def test_a_decompression_bomb_is_refused_with_little_memory(self):
        """13000x13000 in 160 KiB used to cost 1.77 GB of RSS; it must be
        refused at the header, measured in a fresh interpreter."""
        import os
        import subprocess
        import sys
        import tempfile

        with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as bomb:
            bomb.write(png_of(13000, 13000))
        here = os.path.dirname(os.path.abspath(__file__))
        probe = (
            "import resource, sys\n"
            f"sys.path.insert(0, {here!r})\n"
            "import worker\n"
            f"data = open({bomb.name!r}, 'rb').read()\n"
            "try:\n"
            "    worker.convert_cover_to_webp(data)\n"
            "    print('decoded')\n"
            "except Exception as exc:\n"
            "    print(type(exc).__name__)\n"
            "print(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)\n"
        )
        try:
            out = subprocess.run(
                [sys.executable, "-I", "-c", probe], capture_output=True, text=True, check=True
            ).stdout.split()
        finally:
            os.unlink(bomb.name)
        self.assertEqual(out[0], "DecompressionBombError")
        self.assertLess(int(out[1]) // 1024, 200, f"peak RSS {int(out[1]) // 1024} MB")

    def test_a_refused_cover_publishes_the_book_without_one(self):
        cursor = FakeCursor("draft")
        cursor.rowcount = 1
        conn = mock.Mock()
        conn.cursor.return_value = cursor
        minio = mock.Mock()
        svg = b'<svg xmlns="http://www.w3.org/2000/svg"/>'
        epub = epub_with_cover(make_epub([("One", "Some text worth reading.")]), "cover.svg", "image/svg+xml", svg)
        minio.get_object.return_value = io.BytesIO(epub)
        minio.get_object.return_value.release_conn = lambda: None
        with mock.patch.object(worker, "make_redis", return_value=mock.MagicMock()), \
             mock.patch.object(worker, "make_minio", return_value=minio), \
             mock.patch.object(worker, "make_db", return_value=conn), \
             mock.patch.object(worker, "embed_batch", side_effect=lambda texts, *_: [[0.0]] * len(texts)), \
             mock.patch.object(worker, "generate_json", return_value={}):
            worker.process_message("1-0", {"book_id": "b", "storage_path": "p", "job_id": "j"}, worker.Settings())
        minio.put_object.assert_not_called()
        self.assertFalse(any("cover_url" in sql for sql in cursor.statements))
        self.assertTrue(any("status = 'published'" in sql for sql in cursor.statements))


def epub_with_cover(epub: bytes, href: str, media_type: str, cover: bytes) -> bytes:
    """Copies ``epub`` with ``data`` added at ``href`` and named the cover."""
    source = zipfile.ZipFile(io.BytesIO(epub))
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as archive:
        for info in source.infolist():
            data = source.read(info.filename)
            if info.filename == "OEBPS/content.opf":
                item = f'<item id="cover-image" href="{href}" media-type="{media_type}" properties="cover-image"/>'
                data = data.decode().replace("</manifest>", item + "</manifest>").encode()
            archive.writestr(info, data)
        archive.writestr(f"OEBPS/{href}", cover)
    return out.getvalue()


if __name__ == "__main__":
    unittest.main()


class RetryAndRecapTests(unittest.TestCase):
    def test_backoff_schedule_is_quadratic_with_bounded_jitter(self):
        self.assertEqual([backoff_seconds(n) for n in (1, 2, 3)], [1.0, 4.0, 9.0])
        self.assertAlmostEqual(backoff_seconds(2, jitter=0.2), 4.8)
        self.assertAlmostEqual(backoff_seconds(2, jitter=0.9), 4.8, msg="jitter is capped at 20%")

    def test_backoff_honours_retry_after_within_cap(self):
        self.assertEqual(backoff_seconds(1, retry_after="7"), 7.0)
        self.assertEqual(backoff_seconds(1, retry_after="900"), 60.0)
        self.assertEqual(backoff_seconds(3, retry_after="soon"), 9.0, msg="unparseable header falls back")

    def test_gemini_post_retries_transient_errors_then_succeeds(self):
        calls = []

        def fake_urlopen(request, timeout):
            calls.append(request.full_url)
            if len(calls) < 3:
                raise urllib.error.HTTPError(request.full_url, 503, "busy", {"Retry-After": "2"}, None)
            return io.BytesIO(b'{"ok": true}')

        with mock.patch.object(worker.urllib.request, "urlopen", fake_urlopen), \
             mock.patch.object(worker.time, "sleep") as sleep:
            result = worker._gemini_post("https://x/v1beta/models/m:generateContent", {}, "k", job_id="j1")
        self.assertEqual(result, {"ok": True})
        self.assertEqual(len(calls), 3)
        self.assertEqual([round(c.args[0]) for c in sleep.call_args_list], [2, 2], "Retry-After drives the wait")

    def test_gemini_post_gives_up_after_max_attempts_and_skips_client_errors(self):
        def always_busy(request, timeout):
            raise urllib.error.HTTPError(request.full_url, 503, "busy", {}, None)

        with mock.patch.object(worker.urllib.request, "urlopen", always_busy), \
             mock.patch.object(worker.time, "sleep") as sleep, \
             self.assertRaises(RuntimeError):
            worker._gemini_post("https://x/v1beta/models/m:generateContent", {}, "k")
        self.assertEqual(sleep.call_count, worker.LLM_MAX_ATTEMPTS - 1)

        def forbidden(request, timeout):
            raise urllib.error.HTTPError(request.full_url, 403, "no", {}, None)

        with mock.patch.object(worker.urllib.request, "urlopen", forbidden), \
             mock.patch.object(worker.time, "sleep") as sleep, \
             self.assertRaises(RuntimeError):
            worker._gemini_post("https://x/v1beta/models/m:generateContent", {}, "k")
        self.assertEqual(sleep.call_count, 0, "client errors are not retried")

    def test_recap_input_lists_earlier_chapters_then_current_text(self):
        previous = [
            {"key_concepts": ["a letter arrives", "the harbour"]},
            {"key_concepts": ["a debt is paid"]},
        ]
        text = build_recap_input(previous, "Now the storm.")
        self.assertTrue(text.startswith("Chapter 1: a letter arrives; the harbour\nChapter 2: a debt is paid"))
        self.assertTrue(text.endswith("CURRENT CHAPTER:\nNow the storm."))

    def test_recap_input_respects_budget_by_trimming_current_then_oldest(self):
        previous = [{"key_concepts": ["x" * 3000]}, {"key_concepts": ["y" * 3000]}]
        current = "c" * 20000
        text = build_recap_input(previous, current, budget=12000)
        self.assertLessEqual(len(text), 12000)
        self.assertIn("Chapter 2: " + "y" * 3000, text)
        self.assertIn("c" * worker.CURRENT_CHAPTER_MIN_CHARS, text)
        crowded = build_recap_input([{"key_concepts": ["z" * 9000]}, {"key_concepts": ["w" * 9000]}], current, budget=12000)
        self.assertNotIn("z" * 9000, crowded, "the oldest summary goes first when room runs out")
        self.assertLessEqual(len(crowded), 12000)


class StreamCapTest(unittest.TestCase):
    def test_dead_letters_are_capped_like_the_ingestion_stream(self):
        client = mock.Mock()
        worker.dead_letter(client, worker.Settings(), "job-1", "book-1", "boom")
        args, kwargs = client.xadd.call_args
        self.assertEqual(args[0], "stream:epub_ingestion:dlq")
        self.assertEqual(args[1], {"job_id": "job-1", "book_id": "book-1", "error": "boom"})
        self.assertEqual(kwargs, {"maxlen": 10_000, "approximate": True})


class GeminiTransportTest(unittest.TestCase):
    def capture(self):
        seen = []

        def fake_urlopen(request, timeout):
            seen.append(request)
            return io.BytesIO(b'{"embeddings": [{"values": [0.5, 0.5]}], "candidates": [{"content": {"parts": [{"text": "{}"}]}}]}')

        return seen, fake_urlopen

    def test_key_travels_in_a_header_never_the_url(self):
        settings = worker.Settings(gemini_api_key="secret-key", embedding_dim=2)
        seen, fake = self.capture()
        with mock.patch.object(worker.urllib.request, "urlopen", fake):
            worker.embed_batch(["one"], settings)
            worker.generate_json("prompt", settings)
        self.assertEqual(len(seen), 2)
        for request in seen:
            self.assertNotIn("key=", request.full_url)
            self.assertNotIn("secret-key", request.full_url)
            self.assertEqual(request.get_header("X-goog-api-key"), "secret-key")

    def test_read_phase_timeouts_are_retried(self):
        calls = []

        def slow_then_ok(request, timeout):
            calls.append(1)
            if len(calls) == 1:
                raise TimeoutError("The read operation timed out")
            if len(calls) == 2:
                raise ConnectionResetError("peer reset")
            return io.BytesIO(b'{"ok": true}')

        with mock.patch.object(worker.urllib.request, "urlopen", slow_then_ok), \
             mock.patch.object(worker.time, "sleep"):
            self.assertEqual(worker._gemini_post("https://x/models/m:generateContent", {}, "k"), {"ok": True})
        self.assertEqual(len(calls), 3)


class ColumnWidthTest(unittest.TestCase):
    def test_oversized_metadata_is_clipped_to_column_widths(self):
        long_title = "T" * 400
        data = make_epub([("H" * 300, "Body words here.")], title=long_title, language="x" * 30)
        parsed = parse_epub(data)
        self.assertEqual(len(parsed.title), worker.TITLE_MAX_CHARS)
        self.assertEqual(len(parsed.language), worker.LANGUAGE_MAX_CHARS)
        self.assertLessEqual(len(parsed.chapters[0].title), worker.TITLE_MAX_CHARS)


class FakeCursor:
    """Answers the pipeline's first SELECT with a fixed book row."""

    def __init__(self, status):
        self.status = status
        self.statements = []
        self.rowcount = 0

    def execute(self, sql, params=()):
        self.statements.append(sql)

    def fetchone(self):
        return ("Title", "Author", "en", self.status)

    def close(self):
        pass


class ArchivedBookTest(unittest.TestCase):
    def test_archived_book_is_skipped_before_any_paid_call(self):
        cursor = FakeCursor("archived")
        conn = mock.Mock()
        conn.cursor.return_value = cursor
        redis_client = mock.Mock()
        redis_client.pipeline.return_value.execute.return_value = [1, True]
        minio = mock.Mock()
        minio.get_object.return_value = io.BytesIO(make_epub([("One", "Some text.")]))
        minio.get_object.return_value.release_conn = lambda: None
        with mock.patch.object(worker, "make_redis", return_value=redis_client), \
             mock.patch.object(worker, "make_minio", return_value=minio), \
             mock.patch.object(worker, "make_db", return_value=conn), \
             mock.patch.object(worker, "embed_batch") as embed, \
             mock.patch.object(worker, "generate_json") as generate:
            worker.process_message("1-0", {"book_id": "b", "storage_path": "p", "job_id": "j"}, worker.Settings())
        embed.assert_not_called()
        generate.assert_not_called()
        self.assertFalse(any("status = 'published'" in sql for sql in cursor.statements))
        last_status = redis_client.pipeline.return_value.hset.call_args.kwargs["mapping"]
        self.assertEqual(last_status["status"], "failed")
        self.assertEqual(last_status["error"], worker.ARCHIVED_ERROR)


class ClaimAndShutdownTest(unittest.TestCase):
    def test_heartbeat_stays_inside_the_reclaim_window(self):
        self.assertEqual(worker.claim_heartbeat_seconds(300_000), 60.0)
        self.assertLess(worker.claim_heartbeat_seconds(2_000), 2.0)
        self.assertGreater(worker.claim_heartbeat_seconds(0), 0.0, "no busy loop at idle 0")

    def test_keep_claimed_refreshes_the_entry_while_the_job_runs(self):
        client = mock.Mock()
        settings = worker.Settings(reclaim_idle_ms=900)
        with worker.keep_claimed(client, settings, "c1", "5-0"):
            worker.time.sleep(1.0)
        self.assertGreaterEqual(client.xclaim.call_count, 2)
        args, kwargs = client.xclaim.call_args
        self.assertEqual(args, (settings.stream, settings.group, "c1", 0, ["5-0"]))
        self.assertEqual(kwargs, {"justid": True})
        calls = client.xclaim.call_count
        worker.time.sleep(0.5)
        self.assertEqual(client.xclaim.call_count, calls, "heartbeat stops with the job")

    def test_shutdown_is_not_swallowed_by_pipeline_guards(self):
        self.assertFalse(issubclass(worker.WorkerShutdown, Exception))

    def test_released_job_does_not_burn_an_attempt(self):
        client = mock.Mock()
        pipe = client.pipeline.return_value
        pipe.execute.return_value = [0, True]
        worker.release_job(client, "j9")
        pipe.hincrby.assert_called_once_with("job:j9", "attempts", -1)
        self.assertEqual(pipe.hset.call_args.kwargs["mapping"]["status"], "queued")

    def test_every_job_write_refreshes_the_hash_ttl(self):
        client = mock.Mock()
        pipe = client.pipeline.return_value
        pipe.execute.return_value = [1, True]
        worker.set_job(client, "j1", "parsing", 5)
        worker.release_job(client, "j1")
        writes = len(pipe.hset.call_args_list) + len(pipe.hincrby.call_args_list)
        self.assertEqual(writes, 3)
        self.assertEqual(
            [c.args for c in pipe.expire.call_args_list],
            [("job:j1", worker.JOB_TTL_SECONDS)] * writes,
        )


class ReclaimTest(unittest.TestCase):
    def test_reclaim_reads_entries_from_redis7_and_redis62_replies(self):
        entry = ("7-0", {"job_id": "j"})
        for reply in (["0-0", [entry], []], ["0-0", [entry]]):
            client = mock.Mock()
            client.xautoclaim.return_value = reply
            self.assertEqual(worker.reclaim_idle(client, worker.Settings(), "c"), ([entry], "0-0"))

    def test_reclaim_failure_is_not_fatal(self):
        client = mock.Mock()
        client.xautoclaim.side_effect = RuntimeError("NOGROUP")
        self.assertEqual(worker.reclaim_idle(client, worker.Settings(), "c"), ([], "0-0"))
