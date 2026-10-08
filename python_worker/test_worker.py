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
            result = worker._gemini_post("https://x/v1beta/models/m:generateContent?key=k", {}, job_id="j1")
        self.assertEqual(result, {"ok": True})
        self.assertEqual(len(calls), 3)
        self.assertEqual([round(c.args[0]) for c in sleep.call_args_list], [2, 2], "Retry-After drives the wait")

    def test_gemini_post_gives_up_after_max_attempts_and_skips_client_errors(self):
        def always_busy(request, timeout):
            raise urllib.error.HTTPError(request.full_url, 503, "busy", {}, None)

        with mock.patch.object(worker.urllib.request, "urlopen", always_busy), \
             mock.patch.object(worker.time, "sleep") as sleep, \
             self.assertRaises(RuntimeError):
            worker._gemini_post("https://x/v1beta/models/m:generateContent?key=k", {})
        self.assertEqual(sleep.call_count, worker.LLM_MAX_ATTEMPTS - 1)

        def forbidden(request, timeout):
            raise urllib.error.HTTPError(request.full_url, 403, "no", {}, None)

        with mock.patch.object(worker.urllib.request, "urlopen", forbidden), \
             mock.patch.object(worker.time, "sleep") as sleep, \
             self.assertRaises(RuntimeError):
            worker._gemini_post("https://x/v1beta/models/m:generateContent?key=k", {})
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
