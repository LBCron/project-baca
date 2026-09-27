"""Unit tests for the ingestion worker's pure helpers (stdlib only).

Run: `.venv/bin/python -m unittest test_worker -v`
No Redis, MinIO, Postgres, or network access required.
"""

import io
import unittest
import zipfile

from worker import check_archive_limits, chunk_words, extract_text, parse_epub, sanitize_html


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
