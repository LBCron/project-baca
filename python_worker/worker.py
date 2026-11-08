"""Project Baca EPUB ingestion worker.

Consumes ``stream:epub_ingestion`` via the ``ingestion-workers`` consumer group
and runs the 5-phase pipeline: download -> parse/sanitize -> chunk -> embed ->
summarize/publish. Job progress is reported to the ``job:{id}`` Redis hash
consumed by ``GET /api/v1/admin/jobs/{id}``.

Pure, unit-testable helpers (no I/O): :func:`sanitize_html`,
:func:`extract_text`, :func:`chunk_words`, :func:`parse_epub`.
Service clients are constructed lazily inside :func:`main` so unit tests import
this module without Redis/MinIO/Postgres running.

Usage:
    .venv/bin/python worker.py [--once]   # --once: process a single message then exit
    .venv/bin/python worker.py --healthcheck   # exit 0 while the loop is alive
"""

from __future__ import annotations

import html
import io
import json
import logging
import os
import random
import re
import signal
import socket
import sys
import tempfile
import threading
import time
import urllib.request
import uuid
import zipfile
import zlib
from collections import Counter
from contextlib import contextmanager
from dataclasses import dataclass, field
from datetime import datetime, timezone
from html.parser import HTMLParser
from pathlib import Path
from xml.etree import ElementTree as ET

# Configuration

def env(name: str, default: str = "") -> str:
    return os.environ.get(name, default)


@dataclass(frozen=True)
class EpubLimits:
    """What one upload may cost. Loose next to the largest real book in dev
    (300 chapters, 6.3 MB of text, about 2,800 chunks), tight next to an
    amplification: a 1 KiB file naming one entry ten thousand times."""

    archive_bytes: int = 200_000_000
    archive_files: int = 5_000
    entry_bytes: int = 10_000_000
    compression_ratio: int = 100
    chapters: int = 2_000
    text_bytes: int = 32_000_000
    chunks: int = 8_000


class PermanentJobError(Exception):
    """A job no retry can fix: a broken upload, a missing object or row. It
    goes to the dead-letter queue at once, its message shown to the curator,
    so it must name the problem without internal detail."""


class RejectedEpub(PermanentJobError, ValueError):
    """An upload that can never ingest: malformed or past a limit."""


@dataclass
class Settings:
    redis_url: str = field(default_factory=lambda: env("REDIS_URL", "redis://localhost:6380/0"))
    database_url: str = field(
        default_factory=lambda: env(
            "DATABASE_URL", "postgres://baca_user:baca_password@localhost:5433/project_baca_db"
        )
    )
    s3_endpoint: str = field(default_factory=lambda: env("S3_ENDPOINT", "http://localhost:9005"))
    s3_region: str = field(default_factory=lambda: env("S3_REGION", "auto"))
    s3_access_key: str = field(default_factory=lambda: env("S3_ACCESS_KEY_ID", "minioadmin"))
    s3_secret_key: str = field(default_factory=lambda: env("S3_SECRET_ACCESS_KEY", "minioadmin"))
    bucket_epubs: str = field(default_factory=lambda: env("S3_BUCKET_EPUBS", "baca-epubs"))
    bucket_covers: str = field(default_factory=lambda: env("S3_BUCKET_COVERS", "baca-covers"))
    gemini_api_key: str = field(
        default_factory=lambda: env("GEMINI_API_KEY", env("AI_API_KEY", ""))
    )
    embedding_model: str = field(
        default_factory=lambda: env("EMBEDDING_MODEL_NAME", env("AI_MODEL_NAME", "gemini-embedding-2"))
    )
    embedding_dim: int = field(
        default_factory=lambda: int(env("EMBEDDING_DIMENSION", env("AI_DIMENSION", "768")))
    )
    llm_model: str = field(default_factory=lambda: env("LLM_MODEL_NAME", "gemini-flash-latest"))
    stream: str = "stream:epub_ingestion"
    dlq_stream: str = "stream:epub_ingestion:dlq"
    # Mirrors STREAM_MAXLEN in crates/infra/src/queue.rs: an approximate cap
    # so neither stream grows with every upload ever made.
    stream_maxlen: int = 10_000
    group: str = "ingestion-workers"
    reclaim_idle_ms: int = field(
        default_factory=lambda: int(env("RECLAIM_IDLE_MS", "300000"))
    )
    limits: EpubLimits = field(default_factory=EpubLimits)


# Pure helpers (import-safe: stdlib only)

ALLOWED_TAGS = {"p", "em", "strong", "blockquote", "h1", "h2", "h3", "hr", "ul", "ol", "li", "br"}


# Elements that never take an end tag; they cannot open a skipped region.
VOID_TAGS = {
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "param", "source", "track", "wbr",
}


# Allowed elements that hold running text themselves; text outside all of
# them is wrapped in a <p> so it gets a p-N anchor and is searchable.
TEXT_CONTAINERS = {"p", "li", "h1", "h2", "h3"}
# Allowed elements that end a run of loose text.
BLOCK_TAGS = {"p", "blockquote", "h1", "h2", "h3", "hr", "ul", "ol", "li"}
# Dropped elements that still separate paragraphs in the source layout.
LAYOUT_BLOCKS = {
    "div", "section", "article", "header", "footer", "main", "aside", "nav", "body",
    "figure", "figcaption", "table", "tr", "td", "th", "pre", "dl", "dt", "dd", "center",
}


class _Sanitizer(HTMLParser):
    """Allowlist sanitizer: keeps semantic tags, drops scripts/styles/attrs.

    Open elements are tracked as a stack so a skipped element that is never
    closed (common for ``iframe`` and ``object`` in real XHTML) ends with its
    parent instead of swallowing the rest of the chapter. Text that sits in
    no paragraph (a dropped ``div``'s content) is wrapped in one."""

    SKIP_TAGS = {"script", "style", "head", "title", "iframe", "object"}
    # Chapters are XHTML: ``<iframe src="x"/>`` is a complete element there,
    # but the HTML raw-text rule would read the rest of the chapter as the
    # iframe's text. Its content is skipped either way.
    CDATA_CONTENT_ELEMENTS = ("script", "style")

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.parts: list[str] = []
        self.open_tags: list[str] = []
        # Counts of open_tags, so a stray end tag is O(1) to dismiss; a
        # scan of the stack made ten thousand of them quadratic.
        self.open_counts: Counter[str] = Counter()
        self.skip_from: int | None = None
        self.containers = 0
        self.loose_p = False

    def _end_loose(self) -> None:
        if self.loose_p:
            self.parts.append("</p>")
            self.loose_p = False

    def _start_loose(self) -> None:
        if not self.containers and not self.loose_p:
            self.parts.append("<p>")
            self.loose_p = True

    def handle_starttag(self, tag: str, attrs: list) -> None:
        tag = tag.lower()
        if tag not in VOID_TAGS:
            if self.skip_from is None and tag in self.SKIP_TAGS:
                self.skip_from = len(self.open_tags)
            self.open_tags.append(tag)
            self.open_counts[tag] += 1
        if self.skip_from is not None:
            return
        if tag in BLOCK_TAGS or tag in LAYOUT_BLOCKS:
            self._end_loose()
        elif tag in ALLOWED_TAGS and tag != "br":
            self._start_loose()
        if tag in ALLOWED_TAGS:
            self.parts.append(f"<{tag}>" if tag != "br" else "<br/>")
            if tag in TEXT_CONTAINERS:
                self.containers += 1

    def handle_endtag(self, tag: str) -> None:
        tag = tag.lower()
        if self.open_counts[tag]:
            while True:
                closed = self.open_tags.pop()
                self.open_counts[closed] -= 1
                if closed == tag:
                    break
        if self.skip_from is not None:
            if len(self.open_tags) <= self.skip_from:
                self.skip_from = None
            return
        if tag in BLOCK_TAGS or tag in LAYOUT_BLOCKS:
            self._end_loose()
        if tag in ALLOWED_TAGS and tag not in {"br", "hr"}:
            self.parts.append(f"</{tag}>")
            if tag in TEXT_CONTAINERS and self.containers:
                self.containers -= 1

    def handle_data(self, data: str) -> None:
        # convert_charrefs decoded entities into text, so an encoded
        # ``&lt;img onerror=...&gt;`` would otherwise be re-emitted as live markup.
        if self.skip_from is None:
            if data.strip():
                self._start_loose()
            self.parts.append(html.escape(data, quote=False))


def sanitize_html(html: str) -> str:
    """Returns cleaned HTML containing only allowlisted semantic tags."""
    parser = _Sanitizer()
    parser.feed(html)
    parser.close()
    parser._end_loose()
    return "".join(parser.parts)


def extract_text(html: str) -> str:
    """Returns plain whitespace-normalized text (tags and entities resolved)."""

    class _Text(HTMLParser):
        SKIP = {"script", "style", "head", "title"}

        def __init__(self) -> None:
            super().__init__(convert_charrefs=True)
            self.parts: list[str] = []
            self.skip_depth = 0

        def handle_starttag(self, tag: str, attrs: list) -> None:
            if tag.lower() in self.SKIP:
                self.skip_depth += 1
            elif not self.skip_depth:
                self.parts.append(" ")

        def handle_endtag(self, tag: str) -> None:
            if tag.lower() in self.SKIP:
                self.skip_depth = max(0, self.skip_depth - 1)
            elif not self.skip_depth:
                self.parts.append(" ")

        def handle_data(self, data: str) -> None:
            if not self.skip_depth:
                self.parts.append(data)

    parser = _Text()
    parser.feed(html)
    return " ".join("".join(parser.parts).split())


# Start tags that close an open <p> under the HTML parsing rules, among the
# tags the sanitizer keeps; "</p>" with no <p> open creates an empty one.
P_CLOSERS = {"p", "blockquote", "h1", "h2", "h3", "hr", "ul", "ol", "li"}


def fold_for_search(text: str) -> str:
    """Case-folds one character at a time, final sigma as sigma; mirrors
    ``shared::fold_for_search`` so the stored text and a query fold alike."""
    return "".join((ch.lower()[:1] or ch) for ch in text).replace("\u03c2", "\u03c3")


def paragraph_texts(html: str) -> list[str]:
    """Plain text of each <p> element of sanitized chapter HTML, in the
    order and count a browser parses them, so entry ``N-1`` is the reader's
    ``p-N``. Entities are decoded, a ``<br>`` reads as a space (a verse line
    must not run into the next), and whitespace is collapsed, exactly as
    offline search reads the stored HTML."""

    class _Paragraphs(HTMLParser):
        def __init__(self) -> None:
            super().__init__(convert_charrefs=True)
            self.found: list[list[str]] = []
            self.open = False

        def handle_starttag(self, tag: str, attrs: list) -> None:
            tag = tag.lower()
            if tag in P_CLOSERS:
                self.open = False
            if tag == "br" and self.open:
                self.found[-1].append(" ")
            if tag == "p":
                self.found.append([])
                self.open = True

        def handle_endtag(self, tag: str) -> None:
            if tag.lower() != "p":
                if tag.lower() in P_CLOSERS:
                    self.open = False
                return
            if not self.open:
                self.found.append([])
            self.open = False

        def handle_data(self, data: str) -> None:
            if self.open:
                self.found[-1].append(data)

    parser = _Paragraphs()
    parser.feed(html)
    parser.close()
    return [" ".join("".join(parts).split()) for parts in parser.found]


def chunk_words(text: str, target: int = 400, minimum: int = 50) -> list[str]:
    """Splits text into ~target-word chunks at whitespace boundaries.

    A trailing fragment shorter than ``minimum`` words merges into the previous
    chunk instead of producing a degenerate tail.
    """
    words = text.split()
    if not words:
        return []
    chunks: list[str] = []
    current: list[str] = []
    for word in words:
        current.append(word)
        if len(current) >= target:
            chunks.append(" ".join(current))
            current = []
    if current:
        if chunks and len(current) < minimum:
            chunks[-1] = chunks[-1] + " " + " ".join(current)
        else:
            chunks.append(" ".join(current))
    return chunks


def chunk_anchors(chunks: list[str], paragraphs: list[str]) -> list[str | None]:
    """The reader anchor (``p-N``) each chunk starts in, or the first
    paragraph after its start when that is a heading. Positions are compared
    with all whitespace removed, because chunks and paragraphs space the same
    text differently (tag boundaries, ``<br>``). A paragraph the chapter text
    does not contain is skipped rather than guessed."""
    text = "".join("".join(chunks).split())
    spans: list[tuple[int, int]] = []
    cursor = 0
    for n, paragraph in enumerate(paragraphs, start=1):
        squeezed = "".join(paragraph.split())
        at = text.find(squeezed, cursor) if squeezed else -1
        if at >= 0:
            spans.append((n, at + len(squeezed)))
            cursor = at + len(squeezed)
    anchors: list[str | None] = []
    offset = 0
    for chunk in chunks:
        n = next((n for n, end in spans if end > offset), spans[-1][0] if spans else None)
        anchors.append(f"p-{n}" if n is not None else None)
        offset += len("".join(chunk.split()))
    return anchors


# Widths of the VARCHAR columns the parsed metadata lands in; an EPUB that
# overflows them must not fail the job after its embeddings were paid for.
TITLE_MAX_CHARS = 255
AUTHOR_MAX_CHARS = 255
LANGUAGE_MAX_CHARS = 10


def clip(value: str, limit: int) -> str:
    return value.strip()[:limit].rstrip()


@dataclass
class ParsedChapter:
    title: str
    html: str
    text: str
    word_count: int
    chunks: list[str]
    paragraphs: list[str] = field(default_factory=list)


@dataclass
class ParsedEpub:
    title: str
    author: str
    language: str
    cover_bytes: bytes | None
    chapters: list[ParsedChapter]


# Room per central directory record for its name and extra fields.
CENTRAL_RECORD_ALLOWANCE = 512


def check_central_directory(data: bytes, limits: EpubLimits) -> None:
    """Bounds the central directory before ``ZipFile`` builds one object per
    entry: 400,000 entries in 33 MiB used to cost 469 MB just to be refused.
    The end record is the one ``ZipFile`` itself will use (``_EndRecData``,
    ZIP64 and prepended stubs included); a second parser could pick another
    record and wave a crafted archive through. ``ZipFile`` walks the
    directory by its size, never its declared count, so the size is bounded
    as well as the count."""
    try:
        record = zipfile._EndRecData(io.BytesIO(data))
    except OSError:
        record = None
    if record is None:
        raise RejectedEpub("Upload is not a valid ZIP/EPUB archive")
    entries, size = record[zipfile._ECD_ENTRIES_TOTAL], record[zipfile._ECD_SIZE]
    if entries > limits.archive_files or size > limits.archive_files * CENTRAL_RECORD_ALLOWANCE:
        raise RejectedEpub(f"EPUB holds more than {limits.archive_files} files")


def check_archive_limits(infos: list, limits: EpubLimits = EpubLimits()) -> None:
    """Rejects zip bombs before any entry is extracted."""
    total = 0
    for info in infos:
        size = info.file_size
        if size > limits.entry_bytes:
            raise RejectedEpub("Archive entry exceeds per-file limit")
        squeezed = getattr(info, "compress_size", 0) or 0
        if squeezed > 0 and size // squeezed > limits.compression_ratio:
            raise RejectedEpub("Archive entry exceeds compression ratio")
        total += size
    if len(infos) > limits.archive_files or total > limits.archive_bytes:
        raise RejectedEpub("Archive exceeds decompression limits")


def _safe_member(prefix: str, href: str) -> str:
    """Joins an OPF-relative href without letting `..`/absolute paths escape."""
    import posixpath

    if href.startswith("/") or href.startswith("\\"):
        raise RejectedEpub(f"EPUB entry escapes package: {href!r}")
    joined = posixpath.normpath(prefix + href)
    if joined.startswith("..") or posixpath.isabs(joined):
        raise RejectedEpub(f"EPUB entry escapes package: {href!r}")
    return joined


# What zipfile raises for a member it cannot inflate: a corrupt deflate
# stream, a bad CRC, a cut-off entry, an unknown method, or encryption.
CORRUPT_MEMBER_ERRORS = (zipfile.BadZipFile, zlib.error, EOFError, NotImplementedError, RuntimeError)


def _read_member(epub: zipfile.ZipFile, name: str) -> bytes:
    """Reads one archive member; a missing one stays a ``KeyError`` for the
    caller, a corrupt one rejects the upload."""
    try:
        return epub.read(name)
    except CORRUPT_MEMBER_ERRORS as exc:
        raise RejectedEpub(f"EPUB entry {name!r} is corrupt") from exc


def _xml(text: str, name: str) -> ET.Element:
    try:
        return ET.fromstring(text)
    except ET.ParseError as exc:
        raise RejectedEpub(f"EPUB {name} is not well-formed XML") from exc


def _opf_path(epub: zipfile.ZipFile) -> str:
    try:
        container = _read_member(epub, "META-INF/container.xml").decode("utf-8", "replace")
    except KeyError as exc:
        raise RejectedEpub("EPUB missing META-INF/container.xml") from exc
    root = _xml(container, "container.xml")
    ns = {"c": "urn:oasis:names:tc:opendocument:xmlns:container"}
    node = root.find("c:rootfiles/c:rootfile", ns)
    if node is None or not node.get("full-path"):
        raise RejectedEpub("EPUB container.xml has no rootfile")
    return node.get("full-path", "")


# Project Gutenberg wraps every book in a header and a license, both inside
# the spine. The book's own text lies between these two marker lines.
GUTENBERG_START = re.compile(r"\*\*\*\s*START OF (?:THE|THIS) PROJECT GUTENBERG[^*]*\*\*\*", re.I)
GUTENBERG_END = re.compile(r"\*\*\*\s*END OF (?:THE|THIS) PROJECT GUTENBERG[^*]*\*\*\*", re.I)
# Tags a cut leaves dangling: closers after the start, openers before the end.
LEADING_CLOSERS = re.compile(r"^(?:\s*</[a-z0-9]+>)+")
TRAILING_OPENERS = re.compile(r"(?:<[a-z0-9]+>\s*)+$")


def parse_epub(data: bytes, limits: EpubLimits = EpubLimits()) -> ParsedEpub:
    """Parses EPUB bytes into metadata, optional cover, and spine-ordered
    chapters, refusing anything past ``limits`` with :class:`RejectedEpub`."""
    check_central_directory(data, limits)
    try:
        epub = zipfile.ZipFile(io.BytesIO(data))
    except zipfile.BadZipFile as exc:
        raise RejectedEpub("Upload is not a valid ZIP/EPUB archive") from exc
    check_archive_limits(epub.infolist(), limits)

    opf_path = _opf_path(epub)
    base = opf_path.rpartition("/")[0]
    prefix = f"{base}/" if base else ""

    opf = _xml(_read_member(epub, _safe_member("", opf_path)).decode("utf-8", "replace"), "package document")
    ns = {
        "opf": "http://www.idpf.org/2007/opf",
        "dc": "http://purl.org/dc/elements/1.1/",
    }

    def find_text(path: str) -> str:
        node = opf.find(path, ns)
        return node.text.strip() if node is not None and node.text else ""

    title = clip(find_text("opf:metadata/dc:title"), TITLE_MAX_CHARS)
    author = clip(find_text("opf:metadata/dc:creator"), AUTHOR_MAX_CHARS)
    language = clip(find_text("opf:metadata/dc:language"), LANGUAGE_MAX_CHARS) or "en"

    manifest: dict[str, tuple[str, str]] = {}
    properties: dict[str, str] = {}
    for item in opf.findall("opf:manifest/opf:item", ns):
        item_id = item.get("id", "")
        manifest[item_id] = (item.get("href", ""), item.get("media-type", ""))
        properties[item_id] = item.get("properties", "")

    cover_bytes: bytes | None = None
    for item_id, (href, media) in manifest.items():
        if "cover-image" in properties[item_id] or item_id.lower() == "cover-image":
            try:
                cover_bytes = _read_member(epub, _safe_member(prefix, href))
            except (KeyError, ValueError):
                cover_bytes = None
            break
    if cover_bytes is None:
        for item_id, (href, media) in manifest.items():
            if media.startswith("image/") and "cover" in (item_id + href).lower():
                try:
                    cover_bytes = _read_member(epub, _safe_member(prefix, href))
                except (KeyError, ValueError):
                    cover_bytes = None
                break

    chapters: list[ParsedChapter] = []
    spine = opf.find("opf:spine", ns)
    if spine is None:
        raise RejectedEpub("EPUB OPF has no spine")
    # A spine may name one document many times; it is one chapter.
    read: set[str] = set()
    text_bytes = 0
    chunk_count = 0
    gutenberg_ended = False
    for itemref in spine.findall("opf:itemref", ns):
        if gutenberg_ended:
            break
        ref = itemref.get("idref", "")
        if ref not in manifest:
            continue
        href, media = manifest[ref]
        if "html" not in media and "xhtml" not in media and "xml" not in media:
            continue
        try:
            member = _safe_member(prefix, href)
        except ValueError:
            continue
        if member in read:
            continue
        read.add(member)
        try:
            raw = _read_member(epub, member).decode("utf-8", "replace")
        except KeyError:
            continue
        cleaned = sanitize_html(raw)
        start = GUTENBERG_START.search(cleaned)
        if start:
            # Everything before the marker was the header: spend nothing on it.
            chapters.clear()
            text_bytes = chunk_count = 0
            cleaned = LEADING_CLOSERS.sub("", cleaned[start.end() :])
        end = GUTENBERG_END.search(cleaned)
        if end:
            gutenberg_ended = True
            cleaned = TRAILING_OPENERS.sub("", cleaned[: end.start()].rstrip())
        text = extract_text(cleaned)
        if not text:
            continue
        if len(chapters) == limits.chapters:
            raise RejectedEpub(f"EPUB has more than {limits.chapters} chapters")
        text_bytes += len(text.encode("utf-8"))
        if text_bytes > limits.text_bytes:
            raise RejectedEpub(f"EPUB text exceeds {limits.text_bytes // 1_000_000} MB")
        # Counted here, before anything is written or paid for.
        chunks = chunk_words(text)
        chunk_count += len(chunks)
        if chunk_count > limits.chunks:
            raise RejectedEpub(
                f"EPUB yields more than {limits.chunks} passages, the most one book may embed"
            )
        heading = ""
        for tag in ("h1", "h2", "h3"):
            start = cleaned.lower().find(f"<{tag}>")
            end = cleaned.lower().find(f"</{tag}>")
            if 0 <= start < end:
                heading = extract_text(cleaned[start : end + len(tag) + 3]).strip()
                break
        chapters.append(
            ParsedChapter(
                title=clip(heading, TITLE_MAX_CHARS) or f"Chapter {len(chapters) + 1}",
                html=cleaned,
                text=text,
                word_count=len(text.split()),
                chunks=chunks,
                paragraphs=paragraph_texts(cleaned),
            )
        )

    if not chapters:
        raise RejectedEpub("EPUB contains no readable spine chapters")
    return ParsedEpub(
        title=title,
        author=author,
        language=language,
        cover_bytes=cover_bytes,
        chapters=chapters,
    )


# Service clients (lazy imports keep unit tests dependency-free)

def make_redis(settings: Settings):
    import redis

    # socket_timeout must exceed the XREADGROUP BLOCK window, or long polls
    # raise TimeoutError instead of returning empty.
    return redis.Redis.from_url(
        settings.redis_url, decode_responses=True, socket_timeout=30
    )


def make_minio(settings: Settings):
    from minio import Minio

    endpoint = settings.s3_endpoint.replace("http://", "").replace("https://", "")
    secure = settings.s3_endpoint.startswith("https://")
    return Minio(
        endpoint,
        access_key=settings.s3_access_key,
        secret_key=settings.s3_secret_key,
        secure=secure,
        # Without a region the client asks the store for one first; R2
        # expects requests signed for `auto`.
        region=settings.s3_region,
    )


def make_db(settings: Settings):
    import pg8000.dbapi

    # pg8000.dbapi.connect accepts a full URL via the `host` parameter? No:
    # parse DATABASE_URL into parts explicitly.
    from urllib.parse import unquote, urlparse

    parts = urlparse(settings.database_url)
    return pg8000.dbapi.connect(
        user=unquote(parts.username or "baca_user"),
        password=unquote(parts.password or ""),
        host=parts.hostname or "localhost",
        port=parts.port or 5432,
        database=(parts.path or "/project_baca_db").lstrip("/"),
    )


# Gemini REST helpers (stdlib urllib, no extra dependency)

LLM_MAX_ATTEMPTS = 4
RETRYABLE_HTTP = {429, 500, 502, 503, 504}
RETRY_AFTER_CAP_S = 60.0


def backoff_seconds(retry: int, retry_after: str | None = None, jitter: float = 0.0) -> float:
    """Delay before the n-th retry: 1s, 4s, 9s with up to 20% jitter, or the
    server's Retry-After (capped) when it sent one."""
    if retry_after:
        try:
            return min(RETRY_AFTER_CAP_S, max(0.0, float(retry_after)))
        except ValueError:
            pass
    return float(retry * retry) * (1.0 + max(0.0, min(jitter, 0.2)))


GEMINI_BASE_URL = "https://generativelanguage.googleapis.com/v1beta/models/"


def _gemini_post(
    url: str, payload: dict, api_key: str, timeout: int = 60, job_id: str = ""
) -> dict:
    """POSTs to Gemini, retrying transient failures with quadratic backoff.

    The key travels in `x-goog-api-key`, never in the URL, because
    `HTTPError` keeps the URL and logs or proxies could then leak it."""
    import urllib.error

    body = json.dumps(payload).encode("utf-8")
    model = url.rsplit("/models/", 1)[-1].split(":", 1)[0]
    last_error = "Gemini API unreachable"
    for attempt in range(1, LLM_MAX_ATTEMPTS + 1):
        request = urllib.request.Request(
            url,
            data=body,
            headers={"Content-Type": "application/json", "x-goog-api-key": api_key},
        )
        retry_after = None
        try:
            with urllib.request.urlopen(request, timeout=timeout) as response:
                return json.loads(response.read().decode("utf-8"))
        except urllib.error.HTTPError as exc:
            if exc.code not in RETRYABLE_HTTP:
                raise RuntimeError(f"Gemini API rejected the request (HTTP {exc.code})") from exc
            last_error = f"Gemini API unavailable (HTTP {exc.code})"
            retry_after = exc.headers.get("Retry-After") if exc.headers else None
        except (urllib.error.URLError, TimeoutError, ConnectionError):
            # A timeout while reading the body is a TimeoutError, not a
            # URLError, and deserves the same retry as a failed connect.
            last_error = "Gemini API unreachable"
        if attempt == LLM_MAX_ATTEMPTS:
            break
        delay = backoff_seconds(attempt, retry_after, jitter=random.random() * 0.2)
        LOG.warning(
            "job %s: %s on %s (attempt %d/%d); retrying in %.1fs",
            job_id or "-", last_error, model, attempt, LLM_MAX_ATTEMPTS, delay,
        )
        time.sleep(delay)
    raise RuntimeError(last_error)


def embed_batch(texts: list[str], settings: Settings, job_id: str = "") -> list[list[float]]:
    """Returns one embedding vector per input text via batchEmbedContents."""
    if not settings.gemini_api_key:
        raise RuntimeError("GEMINI_API_KEY is not configured")
    url = f"{GEMINI_BASE_URL}{settings.embedding_model}:batchEmbedContents"
    all_vectors: list[list[float]] = []
    for offset in range(0, len(texts), 20):
        batch = texts[offset : offset + 20]
        payload = {
            "requests": [
                {
                    "model": f"models/{settings.embedding_model}",
                    "content": {"parts": [{"text": text}]},
                    "outputDimensionality": settings.embedding_dim,
                }
                for text in batch
            ]
        }
        result = _gemini_post(url, payload, settings.gemini_api_key, job_id=job_id)
        embeddings = result.get("embeddings", [])
        if len(embeddings) != len(batch):
            raise RuntimeError(f"Embedding batch returned {len(embeddings)} for {len(batch)} texts")
        for embedding in embeddings:
            values = embedding.get("values", [])
            if len(values) != settings.embedding_dim:
                raise RuntimeError(
                    f"Embedding dimension {len(values)} != {settings.embedding_dim}"
                )
            all_vectors.append([float(v) for v in values])
    return all_vectors


ATOMIC_CARDS_PROMPT = """Analyze this book chapter and respond with a single JSON object (no markdown fences, no commentary) with exactly these keys:
{"key_concepts": ["3-5 short noun phrases"], "notable_quotes": ["1-2 verbatim quotes from the chapter"], "historical_context": "2-3 sentences"}.
Do not invent quotes; use only text from the chapter.

CHAPTER:
"""

RECAP_PROMPT = """Summarize ONLY what happens up to and including this chapter of an ongoing book. Respond with a single JSON object (no markdown fences, no commentary) with exactly these keys:
{"summary": "events so far in 4-6 sentences", "key_characters": ["names"], "spoiler_free_note": "one sentence"}.
Never reveal or speculate about later chapters.

CHAPTERS SO FAR (last one is the current chapter):
"""


PROMPT_BUDGET_CHARS = 12000
CURRENT_CHAPTER_MIN_CHARS = 4000


def build_recap_input(previous_cards: list[dict], current_text: str, budget: int = PROMPT_BUDGET_CHARS) -> str:
    """Assembles the recap prompt body: one line of key concepts per earlier
    chapter (oldest first), then the current chapter text, trimmed to fit the
    budget. The current chapter keeps at least a few thousand characters; the
    oldest summaries go first when room runs out."""
    lines = []
    for number, cards in enumerate(previous_cards, start=1):
        concepts = [c for c in (cards or {}).get("key_concepts", []) if isinstance(c, str)]
        if concepts:
            lines.append(f"Chapter {number}: " + "; ".join(concepts))
    marker = "\nCURRENT CHAPTER:\n"
    while True:
        head = "\n".join(lines)
        room = budget - len(head) - len(marker)
        if room >= CURRENT_CHAPTER_MIN_CHARS or not lines:
            return head + marker + current_text[: max(0, room)]
        lines.pop(0)


def _strip_fences(text: str) -> str:
    text = text.strip()
    if text.startswith("```"):
        text = text.split("\n", 1)[1] if "\n" in text else ""
        if text.rstrip().endswith("```"):
            text = text.rstrip()[: -3]
    return text.strip()


def generate_json(prompt: str, settings: Settings, job_id: str = "") -> dict:
    """Calls the text LLM and parses the JSON object response."""
    if not settings.gemini_api_key:
        raise RuntimeError("GEMINI_API_KEY is not configured")
    url = f"{GEMINI_BASE_URL}{settings.llm_model}:generateContent"
    payload = {"contents": [{"parts": [{"text": prompt}]}]}
    result = _gemini_post(url, payload, settings.gemini_api_key, timeout=120, job_id=job_id)
    try:
        text = result["candidates"][0]["content"]["parts"][0]["text"]
    except (KeyError, IndexError, TypeError) as exc:
        raise RuntimeError(f"LLM response missing text: {str(result)[:200]}") from exc
    return json.loads(_strip_fences(text))


# Pipeline

LOG = logging.getLogger("ingestion")

# Mirrors JOB_TTL_SECS in crates/infra/src/queue.rs.
JOB_TTL_SECONDS = 7 * 24 * 3600


def write_job(redis_client, job_id: str, write):
    """Runs ``write(pipe, key)`` with the hash TTL refreshed in the same
    MULTI: a job hash the server let expire is never recreated without one.
    Returns the reply of the write."""
    key = f"job:{job_id}"
    pipe = redis_client.pipeline()
    write(pipe, key)
    pipe.expire(key, JOB_TTL_SECONDS)
    return pipe.execute()[0]


def set_job(redis_client, job_id: str, status: str, progress: int, error: str = "") -> None:
    fields = {
        "status": status,
        "progress": str(progress),
        "updated_at": datetime.now(timezone.utc).isoformat(),
    }
    fields["error"] = error[:2000]
    write_job(redis_client, job_id, lambda pipe, key: pipe.hset(key, mapping=fields))


# The cover formats an EPUB may carry; SVG, TIFF, JPEG 2000, BMP and the
# rest never reach a decoder.
COVER_FORMATS = ["JPEG", "PNG", "WEBP", "GIF"]
# Pixels decoded for one cover. Past this Pillow refuses at the header,
# before allocating anything, so a small file cannot claim gigabytes.
MAX_COVER_PIXELS = 40_000_000
# Longest edge of a stored cover; JPEGs decode straight to this scale.
COVER_MAX_EDGE = 1600


def convert_cover_to_webp(cover_bytes: bytes) -> bytes:
    """Decodes an untrusted cover and re-encodes it as WebP. Raises on any
    format outside :data:`COVER_FORMATS` or any decompression bomb."""
    import warnings

    from PIL import Image

    Image.MAX_IMAGE_PIXELS = MAX_COVER_PIXELS
    with warnings.catch_warnings():
        warnings.simplefilter("error", Image.DecompressionBombWarning)
        with Image.open(io.BytesIO(cover_bytes), formats=COVER_FORMATS) as image:
            image.thumbnail((COVER_MAX_EDGE, COVER_MAX_EDGE))
            canvas = image.convert("RGB")
            buffer = io.BytesIO()
            canvas.save(buffer, format="WEBP", quality=82)
            return buffer.getvalue()


ARCHIVED_ERROR = "Book is archived; it was not republished"
UNTITLED = "Untitled"
MISSING_OBJECT_CODES = {"NoSuchKey", "NoSuchBucket"}


def read_upload(minio_client, bucket: str, path: str) -> bytes:
    from minio.error import S3Error

    try:
        response = minio_client.get_object(bucket, path)
    except S3Error as exc:
        if exc.code in MISSING_OBJECT_CODES:
            raise PermanentJobError("The uploaded EPUB is gone from storage; upload it again") from exc
        raise
    try:
        return response.read()
    finally:
        response.close()
        response.release_conn()


def curator_reason(exc: BaseException) -> str:
    """What the curator desk shows. Only permanent errors are worded for it;
    anything else names its kind, and the worker log keeps the detail."""
    if isinstance(exc, PermanentJobError):
        return str(exc)
    if isinstance(exc, RuntimeError) and str(exc).startswith("Gemini API"):
        return str(exc)
    return f"Unexpected {type(exc).__name__}; the worker log has the details"


def process_message(msg_id: str, fields: dict, settings: Settings) -> None:
    """Runs the full 5-phase pipeline for one stream message (raises on failure)."""
    import pg8000.dbapi  # noqa: F401  (ensures driver import errors surface here)

    book_id = fields["book_id"]
    storage_path = fields["storage_path"]
    job_id = fields.get("job_id", msg_id)

    redis_client = make_redis(settings)
    minio_client = make_minio(settings)
    conn = make_db(settings)
    cur = conn.cursor()

    try:
        set_job(redis_client, job_id, "parsing", 5)
        epub_bytes = read_upload(minio_client, settings.bucket_epubs, storage_path)
        parsed = parse_epub(epub_bytes, settings.limits)

        # Metadata backfill: fill only blank admin-provided fields.
        cur.execute(
            "SELECT title, author, language, status FROM books WHERE id = %s",
            (book_id,),
        )
        row = cur.fetchone()
        if row is None:
            raise PermanentJobError("The book was deleted before ingestion")
        if row[3] == "archived":
            # A curator archived it; a replay must neither republish it nor
            # spend embeddings on it.
            set_job(redis_client, job_id, "failed", 100, error=ARCHIVED_ERROR)
            LOG.info("book %s is archived; ingestion skipped", book_id)
            return
        # The catalog shows the book as processing until it publishes; a
        # failure for good hands it back to draft (see fail_job).
        cur.execute(
            "UPDATE books SET status = 'processing', updated_at = NOW() WHERE id = %s AND status = 'draft'",
            (book_id,),
        )
        updates: dict[str, str] = {}
        # A curator who left the title blank gets the EPUB's, or a
        # placeholder rather than an empty title in the catalog.
        if not row[0] or row[0] == "Unknown":
            updates["title"] = parsed.title or UNTITLED
        if (not row[1] or row[1] == "Unknown") and parsed.author:
            updates["author"] = parsed.author
        if not row[2] and parsed.language:
            updates["language"] = parsed.language
        if updates:
            assignments = ", ".join(f"{col} = %s" for col in updates)
            cur.execute(
                f"UPDATE books SET {assignments}, updated_at = NOW() WHERE id = %s",
                (*updates.values(), book_id),
            )

        # Only a cover that decodes and re-encodes is stored; anything else
        # leaves the book on the typographic cover the web draws.
        cover_data = None
        if parsed.cover_bytes:
            try:
                cover_data = convert_cover_to_webp(parsed.cover_bytes)
            except Exception as exc:  # noqa: BLE001 - a bad cover must never kill ingestion
                LOG.warning("cover of %s refused, publishing without one: %s", book_id, exc)
        if cover_data:
            cover_key = f"covers/{book_id}.webp"
            minio_client.put_object(
                settings.bucket_covers,
                cover_key,
                io.BytesIO(cover_data),
                len(cover_data),
                content_type="image/webp",
            )
            cur.execute(
                "UPDATE books SET cover_url = %s, updated_at = NOW() WHERE id = %s",
                (cover_key, book_id),
            )
        # The book row is written only above and at publish. Committing now
        # frees it for the hours of embedding and summaries below: a curator
        # archiving the book meanwhile is not left waiting on this job, and
        # the publish step still leaves an archived book archived.
        conn.commit()

        # Chapters upsert (idempotent across re-runs).
        set_job(redis_client, job_id, "chunking", 30)
        chapter_ids: list[str] = []
        total_words = 0
        for number, chapter in enumerate(parsed.chapters, start=1):
            total_words += chapter.word_count
            cur.execute(
                """INSERT INTO chapters (id, book_id, chapter_number, title, word_count, html_content)
                   VALUES (%s, %s, %s, %s, %s, %s)
                   ON CONFLICT (book_id, chapter_number) DO UPDATE SET
                     title = EXCLUDED.title, word_count = EXCLUDED.word_count,
                     html_content = EXCLUDED.html_content
                   RETURNING id""",
                (
                    str(uuid.uuid4()),
                    book_id,
                    number,
                    chapter.title,
                    chapter.word_count,
                    chapter.html,
                ),
            )
            chapter_ids.append(str(cur.fetchone()[0]))
            write_paragraphs(cur, book_id, number, chapter.paragraphs)

        # Scene chunking at ~400 words, then batched embeddings.
        pending: list[tuple[str, int, str, str | None]] = []
        for chapter_id, chapter in zip(chapter_ids, parsed.chapters):
            anchors = chunk_anchors(chapter.chunks, chapter.paragraphs)
            for index, (piece, anchor) in enumerate(zip(chapter.chunks, anchors)):
                pending.append((chapter_id, index, piece, anchor))
        if not pending:
            raise PermanentJobError("EPUB yields no passages to embed")
        # The book row is free since the first commit, so a curator may have
        # archived it already; stop before the embeddings and summaries are paid.
        cur.execute("SELECT status FROM books WHERE id = %s", (book_id,))
        current = cur.fetchone()
        if current is not None and current[-1] == "archived":
            conn.rollback()
            set_job(redis_client, job_id, "failed", 100, error=ARCHIVED_ERROR)
            LOG.info("book %s was archived before embedding; ingestion stopped", book_id)
            return

        set_job(redis_client, job_id, "embedding", 55)
        cur.execute("DELETE FROM book_chunks WHERE book_id = %s", (book_id,))
        for offset in range(0, len(pending), 20):
            batch = pending[offset : offset + 20]
            vectors = embed_batch([text for _, _, text, _ in batch], settings, job_id)
            for (chapter_id, index, text, anchor), vector in zip(batch, vectors):
                literal = "[" + ",".join(repr(v) for v in vector) + "]"
                cur.execute(
                    """INSERT INTO book_chunks
                       (id, book_id, chapter_id, chunk_index, chunk_text, cfi_range, embedding)
                       VALUES (%s, %s, %s, %s, %s, %s, %s::vector)""",
                    (
                        str(uuid.uuid4()),
                        book_id,
                        chapter_id,
                        index,
                        text,
                        anchor,
                        literal,
                    ),
                )
            done = min(offset + 20, len(pending))
            set_job(redis_client, job_id, "embedding", 55 + int(20 * done / len(pending)))

        # Atomic cards + spoiler-free recaps via the text LLM.
        set_job(redis_client, job_id, "summarizing", 80)
        previous_cards: list[dict] = []
        for number, (chapter_id, chapter) in enumerate(
            zip(chapter_ids, parsed.chapters), start=1
        ):
            cards = generate_json(
                ATOMIC_CARDS_PROMPT + chapter.text[:PROMPT_BUDGET_CHARS], settings, job_id
            )
            cur.execute(
                """INSERT INTO tldr_cache (id, book_id, chapter_id, recap_type, content_json, model_version)
                   VALUES (%s, %s, %s, 'chapter_atomic_cards', %s::jsonb, %s)
                   ON CONFLICT ON CONSTRAINT uq_tldr_cache DO UPDATE
                       SET content_json = EXCLUDED.content_json,
                           model_version = EXCLUDED.model_version,
                           created_at = NOW()""",
                (
                    str(uuid.uuid4()),
                    book_id,
                    chapter_id,
                    json.dumps(cards),
                    settings.llm_model,
                ),
            )
            if number > 1:  # recaps start at chapter 2
                recap = generate_json(
                    RECAP_PROMPT + build_recap_input(previous_cards, chapter.text), settings, job_id
                )
                cur.execute(
                    """INSERT INTO tldr_cache (id, book_id, chapter_id, recap_type, content_json, model_version)
                       VALUES (%s, %s, %s, 'chapter_recap', %s::jsonb, %s)
                       ON CONFLICT ON CONSTRAINT uq_tldr_cache DO UPDATE
                       SET content_json = EXCLUDED.content_json,
                           model_version = EXCLUDED.model_version,
                           created_at = NOW()""",
                    (
                        str(uuid.uuid4()),
                        book_id,
                        chapter_id,
                        json.dumps(recap),
                        settings.llm_model,
                    ),
                )
            previous_cards.append(cards)
            set_job(
                redis_client,
                job_id,
                "summarizing",
                80 + int(15 * number / len(chapter_ids)),
            )

        # The word count marks a finished ingestion even for a book archived
        # meanwhile, so a curator can restore it; only the status waits on
        # the archive.
        minutes = max(1, total_words // 200)
        cur.execute(
            """UPDATE books SET total_words = %s, estimated_reading_minutes = %s, updated_at = NOW()
               WHERE id = %s""",
            (total_words, minutes, book_id),
        )
        cur.execute(
            "UPDATE books SET status = 'published' WHERE id = %s AND status <> 'archived'",
            (book_id,),
        )
        published = cur.rowcount > 0
        conn.commit()
        if not published:
            set_job(redis_client, job_id, "failed", 100, error=ARCHIVED_ERROR)
            LOG.info("book %s was archived during ingestion; left archived", book_id)
            return
        set_job(redis_client, job_id, "published", 100)
        LOG.info("book %s published (%d words)", book_id, total_words)
    finally:
        try:
            cur.close()
        except Exception:  # noqa: BLE001 - cleanup must not mask pipeline errors
            pass
        try:
            conn.close()
        except Exception:  # noqa: BLE001
            pass


# Rows per INSERT when writing paragraphs: one statement per paragraph
# would cost a round trip each for books of tens of thousands.
PARAGRAPH_BATCH = 500


def write_paragraphs(cur, book_id: str, chapter_number: int, paragraphs: list[str]) -> None:
    """Replaces the indexed paragraphs of one chapter (re-ingestion and the
    backfill both rewrite them whole)."""
    cur.execute(
        "DELETE FROM chapter_paragraphs WHERE book_id = %s AND chapter_number = %s",
        (book_id, chapter_number),
    )
    rows = [(n, text) for n, text in enumerate(paragraphs, start=1)]
    for offset in range(0, len(rows), PARAGRAPH_BATCH):
        batch = rows[offset : offset + PARAGRAPH_BATCH]
        values = ", ".join(["(%s, %s, %s, %s, %s)"] * len(batch))
        params = [
            v for n, text in batch for v in (book_id, chapter_number, n, text, fold_for_search(text))
        ]
        cur.execute(
            f"INSERT INTO chapter_paragraphs (book_id, chapter_number, n, text, folded) VALUES {values}",
            params,
        )


def backfill_paragraphs(settings: Settings) -> int:
    """Indexes the paragraphs of every chapter from its stored HTML and
    anchors its stored chunks on them, for books ingested before either
    existed; the embeddings are kept, nothing is paid again. Returns
    chapters done."""
    conn = make_db(settings)
    try:
        cur = conn.cursor()
        cur.execute("SELECT DISTINCT book_id FROM chapters")
        books = [str(row[0]) for row in cur.fetchall()]
        done = 0
        # One book's HTML in memory and one transaction per book: a large
        # library neither loads whole nor holds its locks until the end.
        for book_id in books:
            done += backfill_book(cur, book_id)
            conn.commit()
        return done
    finally:
        conn.close()


def backfill_book(cur, book_id: str) -> int:
    """Indexes one book's paragraphs and anchors its chunks. Returns chapters done."""
    cur.execute(
        "SELECT id, chapter_number, html_content FROM chapters WHERE book_id = %s ORDER BY chapter_number",
        (book_id,),
    )
    chapters = cur.fetchall()
    for chapter_id, chapter_number, html_content in chapters:
        paragraphs = paragraph_texts(html_content or "")
        write_paragraphs(cur, book_id, chapter_number, paragraphs)
        cur.execute(
            "SELECT id, chunk_text FROM book_chunks WHERE chapter_id = %s ORDER BY chunk_index",
            (str(chapter_id),),
        )
        chunks = cur.fetchall()
        anchors = chunk_anchors([text for _, text in chunks], paragraphs)
        for (chunk_id, _), anchor in zip(chunks, anchors):
            cur.execute("UPDATE book_chunks SET cfi_range = %s WHERE id = %s", (anchor, str(chunk_id)))
    return len(chapters)


# Consumer loop

# Runs of one job before it is dead-lettered, crashes included.
MAX_ATTEMPTS = 3


class WorkerShutdown(BaseException):
    """Raised from the SIGTERM/SIGINT handler. A BaseException so the
    pipeline's `except Exception` guards cannot swallow it."""


def _raise_shutdown(signum, _frame) -> None:
    raise WorkerShutdown(signal.Signals(signum).name)


# The container healthcheck (`worker.py --healthcheck`) reads this file's age:
# the idle loop touches it every few seconds, a running job's claim refresh
# at least once a minute.
HEARTBEAT_FILE = Path(tempfile.gettempdir()) / "rotaria-worker.alive"
HEARTBEAT_MAX_AGE_S = 120.0


def mark_alive() -> None:
    try:
        HEARTBEAT_FILE.touch()
    except OSError:
        pass  # Ingestion goes on; the healthcheck reports the worker unhealthy.


def is_alive() -> bool:
    try:
        age = time.time() - HEARTBEAT_FILE.stat().st_mtime
    except OSError:
        return False
    return age <= HEARTBEAT_MAX_AGE_S


def claim_heartbeat_seconds(reclaim_idle_ms: int) -> float:
    """Refresh often enough that an entry never looks idle to `XAUTOCLAIM`."""
    return max(0.2, min(60.0, reclaim_idle_ms / 3000))


@contextmanager
def keep_claimed(redis_client, settings: Settings, consumer: str, msg_id: str):
    """Keeps a stream entry claimed while its job runs.

    `XAUTOCLAIM` measures idle time from the last delivery, and a long book
    outlives `RECLAIM_IDLE_MS`; without a refresh another replica would take
    the entry mid-job and ingest it twice. `XCLAIM ... JUSTID` resets the
    idle time without counting a delivery."""
    stop = threading.Event()
    interval = claim_heartbeat_seconds(settings.reclaim_idle_ms)

    def beat() -> None:
        while not stop.wait(interval):
            mark_alive()
            try:
                redis_client.xclaim(
                    settings.stream, settings.group, consumer, 0, [msg_id], justid=True
                )
            except Exception as exc:  # noqa: BLE001 - a missed beat is retried
                LOG.warning("claim refresh for %s failed: %s", msg_id, exc)

    thread = threading.Thread(target=beat, name=f"claim-{msg_id}", daemon=True)
    thread.start()
    try:
        yield
    finally:
        stop.set()
        thread.join(timeout=5)


def reclaim_idle(
    redis_client, settings: Settings, consumer: str, cursor: str = "0-0"
) -> tuple[list, str]:
    """Takes over the next entry idle past `RECLAIM_IDLE_MS` (dead-worker
    recovery), scanning the pending list from ``cursor``; returns the entries
    and the cursor for the next call, which wraps to ``0-0`` after a pass.

    One entry at a time on purpose: a batch would leave the rest idle behind
    the first job, and another replica would take and ingest them twice.
    Redis 7 answers with three values (cursor, entries, deleted ids)."""
    try:
        reply = redis_client.xautoclaim(
            settings.stream,
            settings.group,
            consumer,
            min_idle_time=settings.reclaim_idle_ms,
            start_id=cursor,
            count=1,
        )
    except Exception as exc:  # noqa: BLE001 - reclaim is best-effort
        LOG.warning("xautoclaim failed: %s", exc)
        return [], "0-0"
    if not reply or len(reply) < 2:
        return [], "0-0"
    return list(reply[1]), str(reply[0])


def release_job(redis_client, job_id: str) -> None:
    """Hands an interrupted job back: the attempt does not count, the entry
    stays pending for another worker, and the curator sees it queued."""
    write_job(redis_client, job_id, lambda pipe, key: pipe.hincrby(key, "attempts", -1))
    set_job(redis_client, job_id, "queued", 0)

def ensure_group(redis_client, settings: Settings) -> None:
    try:
        redis_client.xgroup_create(
            settings.stream, settings.group, id="0", mkstream=True
        )
    except Exception as exc:  # noqa: BLE001 - BUSYGROUP means "already exists"
        if "BUSYGROUP" not in str(exc):
            raise


def run(once: bool = False, settings: Settings | None = None) -> int:
    logging.basicConfig(
        level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s"
    )
    settings = settings or Settings()
    redis_client = make_redis(settings)
    ensure_group(redis_client, settings)
    consumer = f"{socket.gethostname()}-{os.getpid()}"
    processed = 0
    # `docker stop` sends SIGTERM and kills ten seconds later; raising here
    # interrupts even a blocking Gemini call, so the job is released in time.
    signal.signal(signal.SIGTERM, _raise_shutdown)
    signal.signal(signal.SIGINT, _raise_shutdown)

    # Reclaim runs every turn, before new work, and its cursor walks the
    # pending list: one entry failing for good cannot hold the rest back.
    cursor = "0-0"
    while True:
        mark_alive()
        entries, cursor = reclaim_idle(redis_client, settings, consumer, cursor)
        if not entries:
            try:
                messages = redis_client.xreadgroup(
                    settings.group, consumer, {settings.stream: ">"}, count=1, block=5000
                )
            except Exception as exc:  # noqa: BLE001 - stalled/blocked read: retry loop
                LOG.warning("stream read stalled (%s); retrying", exc)
                if once:
                    break
                continue
            entries = [entry for _, batch in (messages or []) for entry in batch]
        if not entries:
            if once:
                break
            continue
        for msg_id, fields in entries:
            if handle_entry(redis_client, settings, consumer, msg_id, fields):
                processed += 1
            if once:
                return 0
    return processed


def handle_entry(redis_client, settings: Settings, consumer: str, msg_id: str, fields) -> bool:
    """Runs one stream entry to an end: published, retried later, or failed
    for good. Returns whether it published."""
    if not fields:
        # The entry's payload was deleted (XDEL) while it sat pending.
        redis_client.xack(settings.stream, settings.group, msg_id)
        return False
    job_id = fields.get("job_id", msg_id)
    try:
        attempts = int(
            write_job(redis_client, job_id, lambda pipe, key: pipe.hincrby(key, "attempts", 1))
        )
    except (TypeError, ValueError):
        attempts = 0
    if attempts > MAX_ATTEMPTS:
        # Earlier attempts never reached an except clause: the process was
        # killed (OOM, SIGKILL) mid-job. A fourth run would only die again.
        reason = f"The worker stopped {MAX_ATTEMPTS} times while ingesting this book"
        report_job_failure(job_id, message=reason)
        fail_job(redis_client, settings, msg_id, fields, reason)
        return False
    try:
        with keep_claimed(redis_client, settings, consumer, msg_id):
            process_message(msg_id, fields, settings)
    except WorkerShutdown:
        release_job(redis_client, job_id)
        LOG.info("shutdown: job %s released unacknowledged", job_id)
        raise
    except PermanentJobError as exc:
        LOG.warning("job %s failed for good: %s", job_id, exc)
        report_job_failure(job_id, exc)
        fail_job(redis_client, settings, msg_id, fields, curator_reason(exc))
        return False
    except Exception as exc:  # noqa: BLE001 - worker must survive bad jobs
        LOG.exception("job %s failed: %s", job_id, exc)
        report_job_failure(job_id, exc)
        if attempts >= MAX_ATTEMPTS:
            fail_job(redis_client, settings, msg_id, fields, curator_reason(exc))
        else:
            # Left unacknowledged: reclaim retries it once it is idle.
            set_job(
                redis_client, job_id, "queued", 0,
                error=curator_reason(exc),
            )
        return False
    redis_client.xack(settings.stream, settings.group, msg_id)
    return True


def fail_job(redis_client, settings: Settings, msg_id: str, fields: dict, reason: str) -> None:
    """Ends a job for good: dead letter first, so a crash before the ack
    leaves it pending rather than lost."""
    job_id = fields.get("job_id", msg_id)
    # A crash between this dead letter and the ack below redelivers the
    # entry; the job already failed, so a second letter would only double a
    # replay.
    if redis_client.hget(f"job:{job_id}", "status") != "failed":
        dead_letter(redis_client, settings, job_id, fields.get("book_id", ""), reason)
    set_job(redis_client, job_id, "failed", 100, error=reason)
    return_to_draft(settings, fields.get("book_id", ""))
    redis_client.xack(settings.stream, settings.group, msg_id)


def return_to_draft(settings: Settings, book_id: str) -> None:
    """A book whose job failed for good is no longer processing. Best effort:
    the dead letter already holds the failure, and a replay moves it again."""
    if not book_id:
        return
    try:
        conn = make_db(settings)
        try:
            conn.cursor().execute(
                "UPDATE books SET status = 'draft', updated_at = NOW() WHERE id = %s AND status = 'processing'",
                (book_id,),
            )
            conn.commit()
        finally:
            conn.close()
    except Exception as exc:  # noqa: BLE001 - must not undo the dead letter
        LOG.warning("book %s left processing: %s", book_id, exc)


def dead_letter(redis_client, settings: Settings, job_id: str, book_id: str, reason: str) -> None:
    redis_client.xadd(
        settings.dlq_stream,
        {"job_id": job_id, "book_id": book_id, "error": reason[:2000]},
        maxlen=settings.stream_maxlen,
        approximate=True,
    )


# Error tracking: Sentry when SENTRY_DSN is set, off otherwise.

_error_tracking = False


SENTRY_REASON_CHARS = 120


def short_reason(text: str) -> str:
    """The head of an error message: up to the first ": ", where messages
    put their detail (a model reply, a response body), and at most
    SENTRY_REASON_CHARS characters."""
    return text.split(": ", 1)[0][:SENTRY_REASON_CHARS]


def scrub_event(event: dict, _hint: dict) -> dict:
    """Keeps an event to the exception kind, a short reason, and its job id:
    no request, log breadcrumbs, extra data, user, frame locals, or message
    detail, any of which could hold book text or a model request or reply."""
    for key in ("request", "breadcrumbs", "extra", "user"):
        event.pop(key, None)
    if isinstance(event.get("message"), str):
        event["message"] = short_reason(event["message"])
    for exception in (event.get("exception") or {}).get("values") or []:
        if isinstance(exception.get("value"), str):
            exception["value"] = short_reason(exception["value"])
        for frame in (exception.get("stacktrace") or {}).get("frames") or []:
            frame.pop("vars", None)
    return event


def init_error_tracking(transport=None) -> bool:
    """Starts Sentry when SENTRY_DSN is set. Only exceptions reported by
    report_job_failure leave the process: log records raise no events and
    leave no breadcrumbs."""
    global _error_tracking
    dsn = env("SENTRY_DSN")
    if not dsn:
        return False
    import sentry_sdk
    from sentry_sdk.integrations.logging import LoggingIntegration

    sentry_sdk.init(
        dsn=dsn,
        environment=env("APP_ENV", "development"),
        send_default_pii=False,
        include_local_variables=False,
        max_request_body_size="never",
        integrations=[LoggingIntegration(level=None, event_level=None)],
        before_breadcrumb=lambda _crumb, _hint: None,
        before_send=scrub_event,
        transport=transport,
    )
    _error_tracking = True
    return True


def report_job_failure(job_id: str, exc: BaseException | None = None, message: str = "") -> None:
    """Sends a failed job to Sentry tagged with its job id."""
    if not _error_tracking:
        return
    import sentry_sdk

    with sentry_sdk.new_scope() as scope:
        scope.set_tag("job_id", job_id)
        if exc is not None:
            sentry_sdk.capture_exception(exc)
        else:
            sentry_sdk.capture_message(message, level="error")


def main() -> None:
    if "--healthcheck" in sys.argv[1:]:
        sys.exit(0 if is_alive() else 1)
    init_error_tracking()
    if "--backfill-paragraphs" in sys.argv[1:]:
        logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
        LOG.info("indexed the paragraphs of %d chapters", backfill_paragraphs(Settings()))
        return
    try:
        sys.exit(run(once="--once" in sys.argv[1:]))
    except WorkerShutdown as reason:
        LOG.info("worker stopped by %s", reason)
        sys.exit(0)


if __name__ == "__main__":
    main()
