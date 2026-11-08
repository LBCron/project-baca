"""Unit tests for the ingestion worker's pure helpers (stdlib only).

Run: `.venv/bin/python -m unittest test_worker -v`
No Redis, MinIO, Postgres, or network access required.
"""

import io
import json
import unittest
import urllib.error
import urllib.parse
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
        self.assertEqual(sanitize_html("<embed src=x>tail text"), "<p>tail text</p>")

    def test_loose_text_is_wrapped_in_a_paragraph(self):
        self.assertEqual(sanitize_html("<div>teks lepas</div>"), "<p>teks lepas</p>")
        self.assertEqual(sanitize_html("<div>a</div><div>b <em>c</em></div>"), "<p>a</p><p>b <em>c</em></p>")
        self.assertEqual(
            sanitize_html("<p>in p</p>loose<br/>line<h2>H</h2>tail"),
            "<p>in p</p><p>loose<br/>line</p><h2>H</h2><p>tail</p>",
        )
        self.assertEqual(sanitize_html("<blockquote>q</blockquote>"), "<blockquote><p>q</p></blockquote>")
        self.assertEqual(sanitize_html("<ul><li>item</li></ul>\n"), "<ul><li>item</li></ul>\n")
        self.assertEqual(worker.paragraph_texts(sanitize_html("<div>teks lepas</div>")), ["teks lepas"])

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

    def test_default_chunks_fit_the_model_window(self):
        words = [f"w{i}" for i in range(700)]
        self.assertEqual([len(c.split()) for c in chunk_words(" ".join(words))], [300, 300, 100])

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
            "before = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss\n"
            "try:\n"
            "    worker.parse_epub(data)\n"
            "    print('parsed')\n"
            "except worker.RejectedEpub as exc:\n"
            "    print('rejected')\n"
            "print(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss - before)\n"
        )
        try:
            out = subprocess.run(
                [sys.executable, "-I", "-c", probe], capture_output=True, text=True, check=True
            ).stdout.split()
        finally:
            os.unlink(bomb.name)
        self.assertEqual(out[0], "rejected")
        # Measured past the imports and the file read, whose size differs
        # between machines (162 MB on a CI runner, under 150 MB here).
        grown = int(out[1]) // 1024
        self.assertLess(grown, 50, f"refusing grew peak RSS by {grown} MB")

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
        client, dead, _ = run_once_with(
            attempts=3, side_effect=worker.UpstreamError("Embedding service unavailable (HTTP 503)")
        )
        self.assertEqual(dead.call_args.args[4], "Embedding service unavailable (HTTP 503)")
        client.xack.assert_called_once()

    def test_a_deleted_pending_entry_is_acknowledged_and_skipped(self):
        client = mock.MagicMock()
        client.xautoclaim.return_value = ["0-0", [("5-0", None)], []]
        with mock.patch.object(worker, "make_redis", return_value=client), \
             mock.patch.object(worker, "process_message") as process:
            worker.run(once=True, settings=worker.Settings())
        process.assert_not_called()
        client.xack.assert_called_once_with(worker.Settings().stream, worker.Settings().group, "5-0")

    def test_a_failed_book_goes_back_to_draft_without_its_chunks(self):
        conn = mock.Mock()
        conn.cursor.return_value.rowcount = 1
        with mock.patch.object(worker, "make_db", return_value=conn), \
             mock.patch.object(worker, "dead_letter"):
            worker.fail_job(mock.MagicMock(), worker.Settings(), "1-0", {"job_id": "j", "book_id": "b"}, "why")
        (update, update_params), (delete, delete_params) = [
            c.args for c in conn.cursor.return_value.execute.call_args_list
        ]
        self.assertIn("status = 'draft'", update)
        self.assertIn("status = 'processing'", update)
        self.assertEqual(update_params, ("b",))
        self.assertEqual((delete, delete_params), ("DELETE FROM book_chunks WHERE book_id = %s", ("b",)))
        conn.commit.assert_called_once()

    def test_a_book_no_longer_processing_keeps_its_chunks(self):
        conn = mock.Mock()
        conn.cursor.return_value.rowcount = 0  # archived or published meanwhile
        with mock.patch.object(worker, "make_db", return_value=conn):
            worker.return_to_draft(worker.Settings(), "b")
        self.assertEqual(conn.cursor.return_value.execute.call_count, 1, "only the status update")

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
             mock.patch.object(worker, "summarize_chapters") as summarize:
            worker.process_message("1-0", {"book_id": "b", "storage_path": "p", "job_id": "j"}, worker.Settings())
        embed.assert_not_called()
        summarize.assert_not_called()
        status = redis_client.pipeline.return_value.hset.call_args.kwargs["mapping"]
        self.assertEqual(status["error"], worker.ARCHIVED_ERROR)
        self.assertTrue(
            any("SET status = 'processing'" in sql for sql in cursor.statements),
            "a draft shows as processing once its job starts",
        )


GUTENBERG_HEADER = (
    "The Project Gutenberg eBook of Kisah Lama. This eBook is for the use of anyone anywhere."
    "<p>*** START OF THE PROJECT GUTENBERG EBOOK KISAH LAMA ***</p>"
)
GUTENBERG_LICENSE = (
    "<p>*** END OF THE PROJECT GUTENBERG EBOOK KISAH LAMA ***</p>"
    "<p>Updated editions will replace the previous one. Section 1. General Terms of Use.</p>"
)


class GutenbergTest(unittest.TestCase):
    def test_header_and_license_are_not_part_of_the_book(self):
        parsed = worker.parse_epub(make_epub([
            ("Cover", "A Gutenberg cover page with credits."),
            ("Kisah Lama", GUTENBERG_HEADER + "<p>Pada suatu pagi kabut turun.</p>"),
            ("Bab Dua", "Laras menulis surat."),
            ("Akhir", "<p>Kapal tiba.</p>" + GUTENBERG_LICENSE),
            ("THE FULL PROJECT GUTENBERG LICENSE", "Section 2. Information about the Mission."),
        ]))
        texts = [c.text for c in parsed.chapters]
        self.assertEqual(len(parsed.chapters), 3, texts)
        self.assertTrue(texts[0].endswith("Pada suatu pagi kabut turun."), texts[0])
        self.assertTrue(texts[-1].endswith("Kapal tiba."), texts[-1])
        self.assertFalse(any("Gutenberg" in t or "Section" in t for t in texts), texts)

    def test_a_book_without_markers_is_untouched(self):
        parsed = worker.parse_epub(make_epub([("One", "Satu."), ("Two", "Dua.")]))
        self.assertEqual([c.text for c in parsed.chapters], ["One Satu.", "Two Dua."])


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

    def test_each_chunk_anchors_on_the_paragraph_it_starts_in(self):
        paragraphs = ["", "alpha beta gamma", "delta epsilon", "zeta eta theta"]
        # The heading sits before any paragraph; a chunk starting there opens
        # the first paragraph after it, one starting mid-paragraph opens that one.
        chunks = ["Heading alpha", "beta gamma delta", "epsilon zeta eta", "theta"]
        self.assertEqual(worker.chunk_anchors(chunks, paragraphs), ["p-2", "p-2", "p-3", "p-4"])

    def test_a_chunk_past_every_paragraph_anchors_on_the_last(self):
        self.assertEqual(worker.chunk_anchors(["one", "tail"], ["one"]), ["p-1", "p-1"])
        self.assertEqual(worker.chunk_anchors(["loose"], []), [None])

    def test_ingestion_writes_the_paragraph_index(self):
        cursor = FakeCursor("draft")
        cursor.fetchone = lambda: ("chapter-id",) if cursor.statements[-1].lstrip().startswith("INSERT INTO chapters") else ("Title", "Author", "en", "draft")
        params = []
        record = cursor.execute
        cursor.execute = lambda sql, p=(): (record(sql, p), params.append((sql, p)))
        conn = mock.Mock()
        conn.cursor.return_value = cursor
        minio = mock.Mock()
        minio.get_object.return_value = io.BytesIO(make_epub([("One", "Some text worth reading.")]))
        minio.get_object.return_value.release_conn = lambda: None
        with mock.patch.object(worker, "make_redis", return_value=mock.MagicMock()), \
             mock.patch.object(worker, "make_minio", return_value=minio), \
             mock.patch.object(worker, "make_db", return_value=conn), \
             mock.patch.object(worker, "count_tokens", side_effect=lambda texts, *_: [10] * len(texts)), \
             mock.patch.object(worker, "embed_batch", side_effect=lambda texts, *_: [[0.0]] * len(texts)), \
             mock.patch.object(worker, "make_llm"), \
             mock.patch.object(worker, "summarize_chapters", side_effect=RuntimeError("stop at publish")):
            with self.assertRaisesRegex(RuntimeError, "stop at publish"):
                worker.process_message("1-0", {"book_id": "b", "storage_path": "p", "job_id": "j"}, worker.Settings())
        inserts = [p for sql, p in params if sql.startswith("INSERT INTO chapter_paragraphs")]
        self.assertEqual(inserts, [["b", 1, 1, "Some text worth reading.", "some text worth reading."]])
        chunk = next(p for sql, p in params if "INSERT INTO book_chunks" in sql)
        self.assertEqual(chunk[5], "p-1")


class ParagraphBackfillTest(unittest.TestCase):
    def test_each_book_is_indexed_and_committed_on_its_own(self):
        events: list[str] = []

        class Cursor:
            last = ""

            def execute(self, sql, params=()):
                self.last = sql
                if sql.startswith("SELECT id, chapter_number"):
                    events.append(f"read {params[0]}")
                if sql.startswith("UPDATE book_chunks"):
                    events.append(f"anchor {params[1]} {params[0]}")

            def fetchall(self):
                if "DISTINCT" in self.last:
                    return [("b1",), ("b2",)]
                if "FROM chapters" in self.last:
                    return [("c1", 1, "<h2>Bab</h2><p>alpha beta</p>")]
                if "FROM book_chunks" in self.last:
                    return [("k1", "Bab alpha beta")]
                return []

        conn = mock.Mock()
        conn.cursor.return_value = Cursor()
        conn.commit.side_effect = lambda: events.append("commit")
        with mock.patch.object(worker, "make_db", return_value=conn):
            self.assertEqual(worker.backfill_paragraphs(worker.Settings()), 2)
        self.assertEqual(
            events,
            ["read b1", "anchor k1 p-1", "commit", "read b2", "anchor k1 p-1", "commit"],
        )
        conn.close.assert_called_once()


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
             mock.patch.object(worker, "count_tokens", side_effect=lambda texts, *_: [10] * len(texts)), \
             mock.patch.object(worker, "embed_batch", side_effect=lambda texts, *_: [[0.0]] * len(texts)), \
             mock.patch.object(worker, "make_llm"), \
             mock.patch.object(worker, "summarize_chapters", side_effect=lambda c, r, j, texts, s: ([{}] * len(texts), {})):
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


class StorageClientTest(unittest.TestCase):
    def test_the_client_signs_for_the_configured_region(self):
        settings = worker.Settings(
            s3_endpoint="https://acct.r2.cloudflarestorage.com", s3_region="auto"
        )
        with mock.patch("minio.Minio") as minio_cls:
            worker.make_minio(settings)
        args, kwargs = minio_cls.call_args
        self.assertEqual(args, ("acct.r2.cloudflarestorage.com",))
        self.assertEqual(kwargs["region"], "auto")
        self.assertTrue(kwargs["secure"])


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

    def test_post_json_retries_transient_errors_then_succeeds(self):
        calls = []

        def fake_urlopen(request, timeout):
            calls.append(request.full_url)
            if len(calls) < 3:
                raise urllib.error.HTTPError(request.full_url, 503, "busy", {"Retry-After": "2"}, None)
            return io.BytesIO(b'{"ok": true}')

        with mock.patch.object(worker.urllib.request, "urlopen", fake_urlopen), \
             mock.patch.object(worker.time, "sleep") as sleep:
            result = worker._post_json("http://x/embed", {}, "Svc", job_id="j1")
        self.assertEqual(result, {"ok": True})
        self.assertEqual(len(calls), 3)
        self.assertEqual([round(c.args[0]) for c in sleep.call_args_list], [2, 2], "Retry-After drives the wait")

    def test_post_json_gives_up_after_max_attempts_and_skips_client_errors(self):
        def always_busy(request, timeout):
            raise urllib.error.HTTPError(request.full_url, 503, "busy", {}, None)

        with mock.patch.object(worker.urllib.request, "urlopen", always_busy), \
             mock.patch.object(worker.time, "sleep") as sleep, \
             self.assertRaises(worker.UpstreamError):
            worker._post_json("http://x/embed", {}, "Svc")
        self.assertEqual(sleep.call_count, worker.LLM_MAX_ATTEMPTS - 1)

        def forbidden(request, timeout):
            raise urllib.error.HTTPError(request.full_url, 403, "no", {}, None)

        with mock.patch.object(worker.urllib.request, "urlopen", forbidden), \
             mock.patch.object(worker.time, "sleep") as sleep, \
             self.assertRaises(worker.UpstreamError):
            worker._post_json("http://x/embed", {}, "Svc")
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


class EmbeddingTransportTest(unittest.TestCase):
    def fake_service(self, dim=2):
        seen = []

        def fake_urlopen(request, timeout):
            seen.append(request)
            inputs = json.loads(request.data)["inputs"]
            return io.BytesIO(json.dumps([[0.5] * dim for _ in inputs]).encode())

        return seen, fake_urlopen

    def test_passages_go_to_the_self_hosted_service_with_their_prefix(self):
        settings = worker.Settings(embedding_url="http://embedder:80/", embedding_dim=2)
        seen, fake = self.fake_service()
        with mock.patch.object(worker.urllib.request, "urlopen", fake):
            vectors = worker.embed_batch(["satu", "dua"], settings)
        self.assertEqual(vectors, [[0.5, 0.5], [0.5, 0.5]])
        self.assertEqual(len(seen), 2, "one passage per request, so queries never wait behind a batch")
        self.assertEqual({r.full_url for r in seen}, {"http://embedder:80/embed"})
        self.assertEqual(
            [json.loads(r.data) for r in seen],
            [
                {"inputs": ["passage: satu"], "normalize": True, "truncate": True},
                {"inputs": ["passage: dua"], "normalize": True, "truncate": True},
            ],
        )

    def test_a_wrong_dimension_fails_the_batch(self):
        seen, fake = self.fake_service(dim=3)
        with mock.patch.object(worker.urllib.request, "urlopen", fake), \
             self.assertRaisesRegex(worker.UpstreamError, "3 dimensions"):
            worker.embed_batch(["one"], worker.Settings(embedding_dim=2))

    def test_the_curator_reads_which_service_failed(self):
        self.assertEqual(
            worker.curator_reason(worker.UpstreamError("Embedding service unreachable")),
            "Embedding service unreachable",
        )

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
            self.assertEqual(worker._post_json("http://x/embed", {}, "Svc"), {"ok": True})
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
             mock.patch.object(worker, "summarize_chapters") as summarize:
            worker.process_message("1-0", {"book_id": "b", "storage_path": "p", "job_id": "j"}, worker.Settings())
        embed.assert_not_called()
        summarize.assert_not_called()
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


class LivenessTest(unittest.TestCase):
    def setUp(self):
        import tempfile
        from pathlib import Path

        self.dir = tempfile.TemporaryDirectory()
        self.addCleanup(self.dir.cleanup)
        patcher = mock.patch.object(worker, "HEARTBEAT_FILE", Path(self.dir.name) / "alive")
        patcher.start()
        self.addCleanup(patcher.stop)

    def test_no_heartbeat_is_unhealthy(self):
        self.assertFalse(worker.is_alive())

    def test_fresh_heartbeat_is_healthy_and_a_stale_one_is_not(self):
        worker.mark_alive()
        self.assertTrue(worker.is_alive())
        stale = worker.time.time() - worker.HEARTBEAT_MAX_AGE_S - 1
        worker.os.utime(worker.HEARTBEAT_FILE, (stale, stale))
        self.assertFalse(worker.is_alive())

    def test_an_idle_loop_beats(self):
        client = mock.Mock()
        client.xautoclaim.return_value = ["0-0", [], []]
        client.xreadgroup.return_value = []
        with mock.patch.object(worker, "make_redis", return_value=client), \
             mock.patch.object(worker, "ensure_group"):
            worker.run(once=True, settings=worker.Settings())
        self.assertTrue(worker.is_alive())

    def test_a_long_job_keeps_beating(self):
        settings = worker.Settings(reclaim_idle_ms=300)
        with worker.keep_claimed(mock.Mock(), settings, "c1", "5-0"):
            worker.time.sleep(0.4)
        self.assertTrue(worker.is_alive())

    def test_unwritable_heartbeat_does_not_stop_the_worker(self):
        with mock.patch.object(worker, "HEARTBEAT_FILE", worker.Path("/nonexistent-dir/alive")):
            worker.mark_alive()
            self.assertFalse(worker.is_alive())


class DatabaseUrlTest(unittest.TestCase):
    def test_percent_encoded_credentials_are_decoded(self):
        dbapi = mock.Mock()
        pg8000 = mock.Mock(dbapi=dbapi)
        settings = worker.Settings(database_url="postgres://rotaria_app:p%40ss%2Fw@db:5432/rotaria")
        with mock.patch.dict("sys.modules", {"pg8000": pg8000, "pg8000.dbapi": dbapi}):
            worker.make_db(settings)
        dbapi.connect.assert_called_once_with(
            user="rotaria_app", password="p@ss/w", host="db", port=5432, database="rotaria"
        )


class FakeRedis:
    """The few hash commands the job code uses, kept in a dict."""

    def __init__(self):
        self.hashes: dict[str, dict] = {}

    def hget(self, key, field):
        return self.hashes.get(key, {}).get(field)

    def pipeline(self):
        store, ops = self.hashes, []

        class Pipe:
            def hset(self, key, field=None, value=None, mapping=None):
                ops.append(lambda: store.setdefault(key, {}).update(mapping or {field: value}))

            def hdel(self, key, field):
                ops.append(lambda: store.get(key, {}).pop(field, None))

            def hincrby(self, key, field, amount):
                ops.append(lambda: store.setdefault(key, {}).update({field: int(store.get(key, {}).get(field, 0)) + amount}))

            def expire(self, key, seconds):
                ops.append(lambda: True)

            def execute(self):
                return [op() for op in ops]

        return Pipe()


def ns(**fields):
    return mock.Mock(spec=list(fields), **fields)


def succeeded(payload, stop_reason="end_turn"):
    # A thinking block comes first, as adaptive thinking returns it.
    content = [ns(type="thinking"), ns(type="text", text=json.dumps(payload))]
    return ns(type="succeeded", message=ns(stop_reason=stop_reason, content=content))


class FakeBatches:
    """Message Batches as the SDK exposes them: counts every call and answers
    each request through ``answer(custom_id, params)``."""

    def __init__(self, answer, polls_until_ended=1, retrieve_error=None):
        self.answer = answer
        self.polls_until_ended = polls_until_ended
        self.retrieve_error = retrieve_error
        self.created: list[list[dict]] = []
        self.requests_by_id: dict[str, list[dict]] = {}
        self.retrieves = 0
        self.cancelled: list[str] = []

    def cancel(self, batch_id):
        self.cancelled.append(batch_id)

    def create(self, requests):
        self.created.append(list(requests))
        batch_id = f"msgbatch_{len(self.created)}"
        self.requests_by_id[batch_id] = list(requests)
        return ns(id=batch_id)

    def retrieve(self, batch_id):
        self.retrieves += 1
        if self.retrieve_error is not None:
            error, self.retrieve_error = self.retrieve_error, None
            raise error
        done = self.retrieves >= self.polls_until_ended
        return ns(processing_status="ended" if done else "in_progress")

    def results(self, batch_id):
        return iter(
            ns(custom_id=r["custom_id"], result=self.answer(r["custom_id"], r["params"]))
            for r in self.requests_by_id[batch_id]
        )


def fake_client(batches):
    client = mock.Mock()
    client.messages.batches = batches
    return client


def card_or_recap(custom_id, params):
    n = custom_id.split("-")[1]
    if params["output_config"]["format"]["schema"] is worker.CARDS_SCHEMA:
        return succeeded({"key_concepts": [f"concept {n}"], "notable_quotes": [], "historical_context": "c"})
    return succeeded({"summary": f"so far {n}", "key_characters": [], "spoiler_free_note": "n"})


class ClaudeBatchTest(unittest.TestCase):
    def settings(self):
        return worker.Settings(llm_model="claude-sonnet-5-5", batch_poll_seconds=0)

    def test_cards_then_recaps_in_two_batches_with_the_decided_request_shape(self):
        batches = FakeBatches(card_or_recap, polls_until_ended=3)
        redis_client = FakeRedis()
        with mock.patch.object(worker.time, "sleep") as sleep:
            cards, recaps = worker.summarize_chapters(
                fake_client(batches), redis_client, "j", ["one", "two", "three"], self.settings()
            )
        self.assertEqual(len(batches.created), 2, "one batch of cards, one of recaps")
        self.assertEqual([r["custom_id"] for r in batches.created[0]], ["chapter-1", "chapter-2", "chapter-3"])
        self.assertEqual([r["custom_id"] for r in batches.created[1]], ["chapter-2", "chapter-3"])
        for request in batches.created[0] + batches.created[1]:
            params = request["params"]
            self.assertEqual(params["model"], "claude-sonnet-5-5")
            self.assertEqual(params["output_config"]["effort"], "medium")
            self.assertEqual(params["output_config"]["format"]["type"], "json_schema")
            self.assertNotIn("thinking", params, "adaptive thinking is the model's default")
            self.assertIn("in the language the", params["messages"][0]["content"], "answers follow the book's language")
        recap_three = batches.created[1][1]["params"]["messages"][0]["content"]
        self.assertIn("Chapter 1: concept 1\nChapter 2: concept 2", recap_three, "recaps read earlier cards")
        self.assertEqual([c["key_concepts"] for c in cards], [["concept 1"], ["concept 2"], ["concept 3"]])
        self.assertEqual(sorted(recaps), [2, 3])
        self.assertEqual(recaps[3]["summary"], "so far 3")
        self.assertGreater(sleep.call_count, 0, "an unfinished batch is polled")

    def test_a_one_chapter_book_needs_no_recap_batch(self):
        batches = FakeBatches(card_or_recap)
        cards, recaps = worker.summarize_chapters(fake_client(batches), FakeRedis(), "j", ["one"], self.settings())
        self.assertEqual(len(batches.created), 1)
        self.assertEqual((len(cards), recaps), (1, {}))

    def test_a_job_released_mid_poll_resumes_its_batch_instead_of_paying_twice(self):
        redis_client = FakeRedis()
        batches = FakeBatches(card_or_recap, retrieve_error=worker.WorkerShutdown("SIGTERM"))
        with self.assertRaises(worker.WorkerShutdown):
            worker.summarize_chapters(fake_client(batches), redis_client, "j", ["one", "two"], self.settings())
        self.assertIn("llm_batch_cards", redis_client.hashes["job:j"], "the batch id survives the release")

        worker.summarize_chapters(fake_client(batches), redis_client, "j", ["one", "two"], self.settings())
        self.assertEqual(len(batches.created), 2, "cards once, recaps once: the cards batch was not resubmitted")

    def test_a_changed_book_never_reuses_an_old_batch(self):
        redis_client = FakeRedis()
        batches = FakeBatches(card_or_recap)
        worker.summarize_chapters(fake_client(batches), redis_client, "j", ["one"], self.settings())
        worker.summarize_chapters(fake_client(batches), redis_client, "j", ["one, revised"], self.settings())
        self.assertEqual(len(batches.created), 2)

    def test_a_changed_book_cancels_the_batch_it_no_longer_needs(self):
        redis_client = FakeRedis()
        batches = FakeBatches(card_or_recap, retrieve_error=worker.WorkerShutdown("SIGTERM"))
        with self.assertRaises(worker.WorkerShutdown):
            worker.summarize_chapters(fake_client(batches), redis_client, "j", ["one"], self.settings())
        worker.summarize_chapters(fake_client(batches), redis_client, "j", ["one, revised"], self.settings())
        self.assertEqual(batches.cancelled, ["msgbatch_1"])

    def test_a_cancel_that_fails_does_not_stop_the_new_batch(self):
        redis_client = FakeRedis()
        batches = FakeBatches(card_or_recap, retrieve_error=worker.WorkerShutdown("SIGTERM"))
        with self.assertRaises(worker.WorkerShutdown):
            worker.summarize_chapters(fake_client(batches), redis_client, "j", ["one"], self.settings())
        batches.cancel = mock.Mock(side_effect=RuntimeError("already ended"))
        cards, _ = worker.summarize_chapters(fake_client(batches), redis_client, "j", ["two"], self.settings())
        self.assertEqual(cards, [{"key_concepts": ["concept 1"], "notable_quotes": [], "historical_context": "c"}])

    def test_a_batch_the_api_forgot_is_dropped_so_the_retry_submits_anew(self):
        import anthropic
        import httpx2

        response = httpx2.Response(404, request=httpx2.Request("GET", "https://api.anthropic.com"))
        gone = anthropic.NotFoundError("not found", response=response, body=None)
        redis_client = FakeRedis()
        batches = FakeBatches(card_or_recap, retrieve_error=gone)
        with self.assertRaisesRegex(worker.UpstreamError, "no longer has the cards batch"):
            worker.summarize_chapters(fake_client(batches), redis_client, "j", ["one"], self.settings())
        self.assertNotIn("llm_batch_cards", redis_client.hashes["job:j"])
        worker.summarize_chapters(fake_client(batches), redis_client, "j", ["one"], self.settings())
        self.assertEqual(len(batches.created), 2, "a fresh batch, not the forgotten one")
        self.assertEqual(batches.cancelled, [], "nothing to cancel once it is dropped")

    def test_a_refused_chapter_goes_without_cards_and_later_recaps_still_come(self):
        def refuse_two(custom_id, params):
            if custom_id == "chapter-2" and params["output_config"]["format"]["schema"] is worker.CARDS_SCHEMA:
                return succeeded({}, stop_reason="refusal")
            return card_or_recap(custom_id, params)

        redis_client = FakeRedis()
        batches = FakeBatches(refuse_two)
        with self.assertLogs("ingestion", "WARNING") as logs:
            cards, recaps = worker.summarize_chapters(
                fake_client(batches), redis_client, "j", ["one", "secret words", "three"], self.settings()
            )
        self.assertIsNone(cards[1])
        self.assertEqual(sorted(recaps), [2, 3], "a chapter without cards still gets its recap")
        recap_three = batches.created[1][1]["params"]["messages"][0]["content"]
        self.assertIn("Chapter 1: concept 1\nCURRENT CHAPTER", recap_three, "the declined chapter is left out")
        (line,) = logs.output
        self.assertIn("job j: chapter 2 cards declined by the model", line)
        self.assertNotIn("secret", line, "no book text in the log")

        worker.summarize_chapters(fake_client(batches), redis_client, "j", ["one", "secret words", "three"], self.settings())
        self.assertEqual(len(batches.created), 2, "a refusal is final: a rerun asks for nothing again")

    def test_an_errored_request_is_retried_alone(self):
        answers = {"chapter-2": ns(type="errored", error=ns(error=ns(type="overloaded_error")))}
        batches = FakeBatches(lambda cid, p: answers.pop(cid, None) or card_or_recap(cid, p))
        redis_client = FakeRedis()
        with self.assertRaises(worker.UpstreamError) as caught:
            worker.summarize_chapters(fake_client(batches), redis_client, "j", ["a", "b", "c"], self.settings())
        reason = worker.curator_reason(caught.exception)
        self.assertEqual(reason, "Claude API failed 1 of 3 cards (chapter 2 cards errored (overloaded_error))")
        self.assertEqual(worker.short_reason(reason), reason, "error tracking keeps the whole reason")

        cards, _ = worker.summarize_chapters(fake_client(batches), redis_client, "j", ["a", "b", "c"], self.settings())
        self.assertEqual([[r["custom_id"] for r in b] for b in batches.created[:2]], [
            ["chapter-1", "chapter-2", "chapter-3"],
            ["chapter-2"],
        ], "the answered chapters are not paid for again")
        self.assertEqual([c["key_concepts"] for c in cards], [["concept 1"], ["concept 2"], ["concept 3"]])

    def test_a_cut_off_answer_gets_one_more_try_then_its_chapter_goes_without(self):
        batches = FakeBatches(
            lambda cid, p: succeeded({}, stop_reason="max_tokens") if cid == "chapter-1" else card_or_recap(cid, p)
        )
        redis_client = FakeRedis()
        with self.assertRaisesRegex(worker.UpstreamError, r"chapter 1 cards cut off at the token limit"):
            worker.summarize_chapters(fake_client(batches), redis_client, "j", ["a", "b"], self.settings())
        cards, recaps = worker.summarize_chapters(fake_client(batches), redis_client, "j", ["a", "b"], self.settings())
        self.assertEqual([[r["custom_id"] for r in b] for b in batches.created[:2]], [["chapter-1", "chapter-2"], ["chapter-1"]])
        self.assertIsNone(cards[0])
        self.assertEqual(sorted(recaps), [2])

    def test_expired_and_errored_requests_fail_the_phase(self):
        answers = {
            "chapter-1": ns(type="expired"),
            "chapter-2": ns(type="errored", error=ns(error=ns(type="overloaded_error"))),
            "chapter-3": ns(type="canceled"),
        }
        with self.assertRaises(worker.UpstreamError) as caught:
            worker.summarize_chapters(
                fake_client(FakeBatches(lambda cid, _p: answers[cid])), FakeRedis(), "j", ["a", "b", "c"], self.settings()
            )
        self.assertIn("failed 3 of 3 cards (chapter 1 cards expired)", str(caught.exception))

    def test_a_missing_key_fails_for_good_before_any_call(self):
        with mock.patch.dict("os.environ", {"ANTHROPIC_API_KEY": ""}):
            with self.assertRaisesRegex(worker.PermanentJobError, "ANTHROPIC_API_KEY is not set"):
                worker.make_llm()

    def test_a_dropped_connection_keeps_the_batch_for_the_retry(self):
        import anthropic
        import httpx2

        redis_client = FakeRedis()
        error = anthropic.APIConnectionError(request=httpx2.Request("GET", "https://api.anthropic.com"))
        with self.assertRaisesRegex(worker.UpstreamError, "^Claude API unreachable$"):
            worker.summarize_chapters(
                fake_client(FakeBatches(card_or_recap, retrieve_error=error)), redis_client, "j", ["one"], self.settings()
            )
        self.assertIn("llm_batch_cards", redis_client.hashes["job:j"], "the paid batch is still polled next run")

    def test_ingestion_writes_claude_cards_and_recaps_and_never_calls_google(self):
        cursor = FakeCursor("draft")
        params = []
        record = cursor.execute
        cursor.execute = lambda sql, p=(): (record(sql, p), params.append((sql, p)))
        cursor.fetchone = lambda: ("chapter-id",) if cursor.statements[-1].lstrip().startswith(
            "INSERT INTO chapters"
        ) else ("Title", "Author", "en", "draft")
        cursor.rowcount = 1
        conn = mock.Mock()
        conn.cursor.return_value = cursor
        minio = mock.Mock()
        minio.get_object.return_value = io.BytesIO(make_epub([("One", "First words."), ("Two", "Second words.")]))
        minio.get_object.return_value.release_conn = lambda: None
        hosts = []

        def embedder(request, timeout):
            hosts.append(urllib.parse.urlsplit(request.full_url).hostname)
            if request.full_url.endswith("/tokenize"):
                inputs = json.loads(request.data)["inputs"]
                return io.BytesIO(json.dumps([[{"id": 0}] * 10 for _ in inputs]).encode())
            return io.BytesIO(json.dumps([[0.5] * 768]).encode())

        batches = FakeBatches(card_or_recap)
        settings = worker.Settings(embedding_url="http://embedder:80", batch_poll_seconds=0)
        with mock.patch.object(worker, "make_redis", return_value=FakeRedis()), \
             mock.patch.object(worker, "make_minio", return_value=minio), \
             mock.patch.object(worker, "make_db", return_value=conn), \
             mock.patch.object(worker, "make_llm", return_value=fake_client(batches)), \
             mock.patch.object(worker.urllib.request, "urlopen", embedder):
            worker.process_message("1-0", {"book_id": "b", "storage_path": "p", "job_id": "j"}, settings)

        self.assertEqual(len(batches.created), 2)
        self.assertEqual(set(hosts), {"embedder"}, "embeddings stay on the host; nothing reaches Google")
        rows = [p for sql, p in params if "INSERT INTO tldr_cache" in sql]
        self.assertEqual([(r[3], r[5]) for r in rows], [
            ("chapter_atomic_cards", "claude-sonnet-5-5"),
            ("chapter_atomic_cards", "claude-sonnet-5-5"),
            ("chapter_recap", "claude-sonnet-5-5"),
        ])
        self.assertTrue(any("status = 'published'" in sql for sql, _ in params))


class TokenFitTest(unittest.TestCase):
    def test_every_piece_fits_and_no_word_is_lost(self):
        def count(texts):  # three tokens a word plus the prefix and <s></s>
            return [3 * len(t.split()) + 4 for t in texts]

        chunks = [" ".join(f"a{i}" for i in range(300)), "short one", " ".join(f"b{i}" for i in range(171))]
        pieces = worker.fit_chunks(chunks, count, limit=512)
        self.assertTrue(all(n <= 512 for n in count(pieces)), count(pieces))
        self.assertEqual(" ".join(pieces).split(), " ".join(chunks).split(), "nothing dropped, order kept")
        self.assertEqual(pieces[2], "short one", "a chunk that fits is left alone")

    def test_counts_come_from_the_service_tokenizer_with_the_prefix(self):
        seen = []

        def fake(request, timeout):
            seen.append(request)
            inputs = json.loads(request.data)["inputs"]
            return io.BytesIO(json.dumps([[{"id": 0}] * len(t.split()) for t in inputs]).encode())

        texts = [f"word {i}" for i in range(40)]
        with mock.patch.object(worker.urllib.request, "urlopen", fake):
            counts = worker.count_tokens(texts, worker.Settings(embedding_url="http://embedder:80"))
        self.assertEqual(counts, [3] * 40, "passage: word N")
        self.assertEqual([len(json.loads(r.data)["inputs"]) for r in seen], [32, 8])
        self.assertEqual(seen[0].full_url, "http://embedder:80/tokenize")
        self.assertTrue(json.loads(seen[0].data)["add_special_tokens"])
        self.assertEqual(json.loads(seen[0].data)["inputs"][0], "passage: word 0")


class RecordingDb:
    """A connection and cursor in one that logs statements and commits in
    order, so a test can see which work ran inside which transaction."""

    def __init__(self, status="draft", chunks=0):
        self.status = status
        self.chunks = chunks
        self.events: list[tuple] = []
        self.rowcount = 1
        self.chapters = 0

    def cursor(self):
        return self

    def execute(self, sql, params=()):
        self.events.append(("sql", " ".join(sql.split()), params))

    def fetchone(self):
        last = self.events[-1][1]
        if last.startswith("INSERT INTO chapters"):
            self.chapters += 1
            return (f"chapter-{self.chapters}",)
        if last.startswith("SELECT COUNT(*) FROM book_chunks"):
            return (self.chunks,)
        return ("Title", "Author", "en", self.status)

    def commit(self):
        self.events.append(("commit",))

    def rollback(self):
        self.events.append(("rollback",))

    def close(self):
        self.events.append(("close",))

    def statements(self) -> list[tuple]:
        return [e for e in self.events if e[0] == "sql"]

    def uncommitted(self) -> list[str]:
        """Statements since the last commit, rollback, or close."""
        open_ = []
        for event in self.events:
            open_ = [] if event[0] != "sql" else open_ + [event[1]]
        return open_

    def committed(self, prefix: str) -> list[tuple]:
        done, pending = [], []
        for event in self.events:
            if event[0] == "commit":
                done, pending = done + pending, []
            elif event[0] in ("rollback", "close"):
                pending = []
            else:
                pending.append(event)
        return [e for e in done if e[1].startswith(prefix)]


class IngestionTransactionTest(unittest.TestCase):
    def run_job(self, db, batches, redis_client, embed=None, llm=None):
        minio = mock.Mock()
        minio.get_object.return_value = io.BytesIO(make_epub([("One", "First words."), ("Two", "Second words.")]))
        minio.get_object.return_value.release_conn = lambda: None
        embed = embed or (lambda texts, *_: [[0.5] * 768] * len(texts))
        llm = llm or mock.Mock(return_value=fake_client(batches))
        with mock.patch.object(worker, "make_redis", return_value=redis_client), \
             mock.patch.object(worker, "make_minio", return_value=minio), \
             mock.patch.object(worker, "make_db", return_value=db), \
             mock.patch.object(worker, "count_tokens", side_effect=lambda texts, *_: [10] * len(texts)), \
             mock.patch.object(worker, "embed_batch", side_effect=embed), \
             mock.patch.object(worker, "make_llm", llm):
            worker.process_message(
                "1-0", {"book_id": "b", "storage_path": "p", "job_id": "j"},
                worker.Settings(batch_poll_seconds=0),
            )

    def test_no_connection_is_held_while_the_model_or_the_batches_work(self):
        db = RecordingDb()
        open_during = {}

        def embed(texts, *_):
            open_during.setdefault("embedding", db.uncommitted())
            return [[0.5] * 768] * len(texts)

        def answer(custom_id, params):
            open_during.setdefault("batch", db.events[-1])
            return card_or_recap(custom_id, params)

        self.run_job(db, FakeBatches(answer), FakeRedis(), embed)
        self.assertEqual(open_during, {"embedding": [], "batch": ("close",)})
        # Cards, recaps, word count, and publish land in one last transaction.
        last = db.statements()[-5:]
        self.assertEqual([e[1].split(" (")[0] for e in last[:3]], ["INSERT INTO tldr_cache"] * 3)
        self.assertIn("status = 'published'", last[-1][1])
        self.assertEqual(db.uncommitted(), [])

    def test_the_publish_runs_on_a_connection_opened_after_the_batches(self):
        before, after = RecordingDb(), RecordingDb()
        with mock.patch.object(worker, "make_db", side_effect=[before, after]):
            minio = mock.Mock()
            minio.get_object.return_value = io.BytesIO(make_epub([("One", "First words.")]))
            minio.get_object.return_value.release_conn = lambda: None
            with mock.patch.object(worker, "make_redis", return_value=FakeRedis()), \
                 mock.patch.object(worker, "make_minio", return_value=minio), \
                 mock.patch.object(worker, "count_tokens", side_effect=lambda texts, *_: [10] * len(texts)), \
                 mock.patch.object(worker, "embed_batch", side_effect=lambda texts, *_: [[0.5] * 768] * len(texts)), \
                 mock.patch.object(worker, "make_llm", return_value=fake_client(FakeBatches(card_or_recap))):
                worker.process_message("1-0", {"book_id": "b", "storage_path": "p", "job_id": "j"},
                                       worker.Settings(batch_poll_seconds=0))
        self.assertIn(("close",), before.events)
        self.assertEqual(before.committed("INSERT INTO tldr_cache"), [])
        self.assertEqual(len(after.committed("INSERT INTO tldr_cache")), 1)
        self.assertTrue(any("status = 'published'" in e[1] for e in after.committed("UPDATE books")))

    def test_a_release_mid_batch_leaves_the_book_hidden_and_the_retry_resumes_it(self):
        redis_client = FakeRedis()
        batches = FakeBatches(card_or_recap, retrieve_error=worker.WorkerShutdown("SIGTERM"))
        embedded = []

        def embed(texts, *_):
            embedded.extend(texts)
            return [[0.5] * 768] * len(texts)

        first = RecordingDb()
        with self.assertRaises(worker.WorkerShutdown):
            self.run_job(first, batches, redis_client, embed)
        self.assertEqual(len(first.committed("INSERT INTO book_chunks")), 2, "chunks are kept")
        self.assertEqual(first.committed("INSERT INTO tldr_cache"), [])
        self.assertFalse(any("status = 'published'" in e[1] for e in first.statements()))
        self.assertEqual(first.uncommitted(), [], "nothing was left half written")
        self.assertEqual(len(embedded), 2)

        second = RecordingDb(status="processing", chunks=2)
        self.run_job(second, batches, redis_client, embed)
        self.assertEqual(len(embedded), 2, "the committed chunks were not embedded again")
        self.assertEqual(second.committed("DELETE FROM book_chunks"), [], "nor deleted")
        self.assertEqual(len(batches.created), 2, "the cards batch was polled again, not paid twice")
        upserts = second.committed("INSERT INTO tldr_cache")
        self.assertEqual(len(upserts), 3)
        self.assertTrue(all("ON CONFLICT ON CONSTRAINT uq_tldr_cache DO UPDATE" in e[1] for e in upserts))
        self.assertTrue(any("status = 'published'" in e[1] for e in second.committed("UPDATE books")))

    def test_a_resume_with_chunks_missing_embeds_the_book_again(self):
        redis_client = FakeRedis()
        calls = []
        embed = lambda texts, *_: calls.append(len(texts)) or [[0.5] * 768] * len(texts)  # noqa: E731
        batches = FakeBatches(card_or_recap, retrieve_error=worker.WorkerShutdown("x"))
        with self.assertRaises(worker.WorkerShutdown):
            self.run_job(RecordingDb(), batches, redis_client, embed)
        second = RecordingDb(status="processing", chunks=1)
        self.run_job(second, batches, redis_client, embed)
        self.assertEqual(sum(calls), 4, "both runs embedded both chunks")
        self.assertEqual(len(second.committed("DELETE FROM book_chunks")), 1)

    def test_a_failed_batch_leaves_a_consistent_unpublished_book(self):
        def errored(custom_id, params):
            return ns(type="errored", error=ns(error=ns(type="overloaded_error")))

        db = RecordingDb()
        with self.assertRaises(worker.UpstreamError):
            self.run_job(db, FakeBatches(errored), FakeRedis())
        self.assertEqual(db.uncommitted(), [])
        self.assertEqual(db.committed("INSERT INTO tldr_cache"), [])
        self.assertFalse(any("status = 'published'" in e[1] for e in db.statements()))
        self.assertEqual(db.committed("UPDATE books SET total_words"), [], "not marked as fully ingested")

    def test_a_refused_chapter_publishes_the_book_without_its_cards(self):
        def refuse_one(custom_id, params):
            if custom_id == "chapter-1" and params["output_config"]["format"]["schema"] is worker.CARDS_SCHEMA:
                return succeeded({}, stop_reason="refusal")
            return card_or_recap(custom_id, params)

        db = RecordingDb()
        self.run_job(db, FakeBatches(refuse_one), FakeRedis())
        rows = [(e[2][2], e[2][3]) for e in db.committed("INSERT INTO tldr_cache")]
        self.assertEqual(rows, [("chapter-2", "chapter_atomic_cards"), ("chapter-2", "chapter_recap")])
        self.assertTrue(any("status = 'published'" in e[1] for e in db.committed("UPDATE books")))

    def test_a_missing_key_fails_before_any_embedding(self):
        embed = mock.Mock()
        llm = mock.Mock(side_effect=worker.PermanentJobError("ANTHROPIC_API_KEY is not set"))
        db = RecordingDb()
        with self.assertRaises(worker.PermanentJobError):
            self.run_job(db, FakeBatches(card_or_recap), FakeRedis(), embed, llm)
        embed.assert_not_called()
        self.assertEqual(db.committed("INSERT INTO book_chunks"), [])

    def test_a_book_archived_while_embedding_never_reaches_the_batches(self):
        db = RecordingDb()
        batches = FakeBatches(card_or_recap)

        def embed(texts, *_):
            db.status = "archived"
            return [[0.5] * 768] * len(texts)

        redis_client = FakeRedis()
        self.run_job(db, batches, redis_client, embed)
        self.assertEqual(batches.created, [])
        self.assertEqual(redis_client.hashes["job:j"]["error"], worker.ARCHIVED_ERROR)
        self.assertFalse(any("status = 'published'" in e[1] for e in db.statements()))

    def test_a_redelivered_job_of_a_published_book_touches_nothing(self):
        db = RecordingDb(status="published")
        batches = FakeBatches(card_or_recap)
        redis_client = FakeRedis()
        self.run_job(db, batches, redis_client)
        self.assertEqual([e[1].split(" FROM")[0] for e in db.events if e[0] == "sql"], ["SELECT title, author, language, status"])
        self.assertEqual(batches.created, [])
        self.assertEqual(redis_client.hashes["job:j"]["status"], "published")


class ErrorTrackingTest(unittest.TestCase):
    def tearDown(self):
        import sentry_sdk

        sentry_sdk.init()
        worker._error_tracking = False

    def test_without_a_dsn_tracking_stays_off(self):
        with mock.patch.dict("os.environ", {"SENTRY_DSN": ""}):
            self.assertFalse(worker.init_error_tracking())
        worker.report_job_failure("job-1", RuntimeError("nothing to send"))

    def test_a_failed_job_carries_its_id_and_no_book_text(self):
        import json
        import logging
        import uuid

        import sentry_sdk
        from sentry_sdk.transport import Transport

        class Captured(Transport):
            def __init__(self):
                super().__init__()
                self.events = []

            def capture_envelope(self, envelope):
                event = envelope.get_event()
                if event is not None:
                    self.events.append(event)

        transport = Captured()
        with mock.patch.dict("os.environ", {"SENTRY_DSN": "https://key@sentry.invalid/1", "APP_ENV": "production"}):
            self.assertTrue(worker.init_error_tracking(transport=transport))

        def embed(chapter_text):
            request_body = {"contents": chapter_text}  # noqa: F841 - a local Sentry must not ship
            raise RuntimeError(f"LLM response missing text: {{'candidates': '{chapter_text}'}}")

        # Built at run time, so the source lines Sentry sends cannot hold it.
        book_text = "chapter-" + uuid.uuid4().hex
        worker.LOG.error("chapter %s", book_text)
        logging.getLogger("urllib3").warning(book_text)
        try:
            embed(book_text)
        except RuntimeError as exc:
            worker.report_job_failure("job-42", exc)
        worker.report_job_failure("job-43", message="The worker stopped 3 times while ingesting this book")
        sentry_sdk.flush()

        self.assertEqual(len(transport.events), 2, transport.events)
        failed, stopped = transport.events
        self.assertEqual(failed["tags"]["job_id"], "job-42")
        self.assertEqual(failed["environment"], "production")
        self.assertEqual(failed["exception"]["values"][-1]["value"], "LLM response missing text")
        self.assertEqual(stopped["tags"]["job_id"], "job-43")
        for event in transport.events:
            dumped = json.dumps(event, default=str)
            self.assertNotIn(book_text, dumped)
            self.assertNotIn("request", event)
            self.assertFalse(event.get("breadcrumbs", {}).get("values"))

    def test_scrubbed_reasons_are_short_and_carry_no_detail(self):
        event = {
            "message": "x" * 500,
            "exception": {"values": [
                {"value": "Claude API unavailable (HTTP 529)"},
                {"value": "y" * 500},
                {"value": "LLM response missing text: model reply"},
            ]},
            "tags": {"job_id": "job-7"},
        }
        scrubbed = worker.scrub_event(event, {})
        values = [v["value"] for v in scrubbed["exception"]["values"]]
        self.assertEqual(values, ["Claude API unavailable (HTTP 529)", "y" * 120, "LLM response missing text"])
        self.assertEqual(len(scrubbed["message"]), 120)
        self.assertEqual(scrubbed["tags"], {"job_id": "job-7"})

    def test_job_failures_are_reported(self):
        with mock.patch.object(worker, "report_job_failure") as report:
            run_once_with(attempts=1, side_effect=worker.UpstreamError("Claude API unavailable (HTTP 529)"))
        self.assertEqual(report.call_args.args[0], "j")

    def test_a_failed_claude_batch_reaches_sentry_with_its_reason_and_no_book_text(self):
        import uuid

        import sentry_sdk
        from sentry_sdk.transport import Transport

        class Captured(Transport):
            def __init__(self):
                super().__init__()
                self.events = []

            def capture_envelope(self, envelope):
                event = envelope.get_event()
                if event is not None:
                    self.events.append(event)

        transport = Captured()
        with mock.patch.dict("os.environ", {"SENTRY_DSN": "https://key@sentry.invalid/1"}):
            self.assertTrue(worker.init_error_tracking(transport=transport))
        book_text = "bab-" + uuid.uuid4().hex
        errored = FakeBatches(lambda cid, _p: ns(type="errored", error=ns(error=ns(type="overloaded_error"))))
        try:
            worker.summarize_chapters(
                fake_client(errored), FakeRedis(), "job-9", [book_text], worker.Settings(batch_poll_seconds=0)
            )
        except worker.UpstreamError as exc:
            worker.report_job_failure("job-9", exc)
        sentry_sdk.flush()

        (event,) = transport.events
        self.assertEqual(event["tags"]["job_id"], "job-9")
        self.assertEqual(
            event["exception"]["values"][-1]["value"],
            "Claude API failed 1 of 1 cards (chapter 1 cards errored (overloaded_error))",
        )
        self.assertNotIn(book_text, json.dumps(event, default=str))
