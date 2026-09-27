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
"""

from __future__ import annotations

import io
import json
import logging
import os
import socket
import sys
import urllib.request
import uuid
import zipfile
from dataclasses import dataclass, field
from datetime import datetime, timezone
from html.parser import HTMLParser
from xml.etree import ElementTree as ET

# Configuration

def env(name: str, default: str = "") -> str:
    return os.environ.get(name, default)


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
    group: str = "ingestion-workers"
    reclaim_idle_ms: int = field(
        default_factory=lambda: int(env("RECLAIM_IDLE_MS", "300000"))
    )


# Pure helpers (import-safe: stdlib only)

ALLOWED_TAGS = {"p", "em", "strong", "blockquote", "h1", "h2", "h3", "hr", "ul", "ol", "li", "br"}


class _Sanitizer(HTMLParser):
    """Allowlist sanitizer: keeps semantic tags, drops scripts/styles/attrs."""

    SKIP_TAGS = {"script", "style", "head", "title", "iframe", "object", "embed"}

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.parts: list[str] = []
        self.skip_depth = 0

    def handle_starttag(self, tag: str, attrs: list) -> None:
        tag = tag.lower()
        if tag in self.SKIP_TAGS:
            self.skip_depth += 1
            return
        if self.skip_depth:
            return
        if tag in ALLOWED_TAGS:
            self.parts.append(f"<{tag}>" if tag != "br" else "<br/>")

    def handle_endtag(self, tag: str) -> None:
        tag = tag.lower()
        if tag in self.SKIP_TAGS:
            self.skip_depth = max(0, self.skip_depth - 1)
            return
        if self.skip_depth:
            return
        if tag in ALLOWED_TAGS and tag not in {"br", "hr"}:
            self.parts.append(f"</{tag}>")

    def handle_data(self, data: str) -> None:
        if not self.skip_depth:
            self.parts.append(data)


def sanitize_html(html: str) -> str:
    """Returns cleaned HTML containing only allowlisted semantic tags."""
    parser = _Sanitizer()
    parser.feed(html)
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


@dataclass
class ParsedChapter:
    title: str
    html: str
    text: str
    word_count: int


@dataclass
class ParsedEpub:
    title: str
    author: str
    language: str
    cover_bytes: bytes | None
    cover_media_type: str
    chapters: list[ParsedChapter]


# Maximum total uncompressed bytes accepted from one archive.
MAX_ARCHIVE_BYTES = 200_000_000
# Maximum entries accepted from one archive.
MAX_ARCHIVE_FILES = 5_000


def check_archive_limits(infos: list) -> None:
    """Rejects zip bombs before any entry is extracted."""
    total = sum(info.file_size for info in infos)
    if len(infos) > MAX_ARCHIVE_FILES or total > MAX_ARCHIVE_BYTES:
        raise ValueError("Archive exceeds decompression limits")


def _opf_path(epub: zipfile.ZipFile) -> str:
    try:
        container = epub.read("META-INF/container.xml").decode("utf-8", "replace")
    except KeyError as exc:
        raise ValueError("EPUB missing META-INF/container.xml") from exc
    root = ET.fromstring(container)
    ns = {"c": "urn:oasis:names:tc:opendocument:xmlns:container"}
    node = root.find("c:rootfiles/c:rootfile", ns)
    if node is None or not node.get("full-path"):
        raise ValueError("EPUB container.xml has no rootfile")
    return node.get("full-path", "")


def parse_epub(data: bytes) -> ParsedEpub:
    """Parses EPUB bytes into metadata, optional cover, and spine-ordered chapters."""
    try:
        epub = zipfile.ZipFile(io.BytesIO(data))
    except zipfile.BadZipFile as exc:
        raise ValueError("Upload is not a valid ZIP/EPUB archive") from exc
    check_archive_limits(epub.infolist())

    opf_path = _opf_path(epub)
    base = opf_path.rpartition("/")[0]
    prefix = f"{base}/" if base else ""

    opf = ET.fromstring(epub.read(opf_path).decode("utf-8", "replace"))
    ns = {
        "opf": "http://www.idpf.org/2007/opf",
        "dc": "http://purl.org/dc/elements/1.1/",
    }

    def find_text(path: str) -> str:
        node = opf.find(path, ns)
        return node.text.strip() if node is not None and node.text else ""

    title = find_text("opf:metadata/dc:title")
    author = find_text("opf:metadata/dc:creator")
    language = find_text("opf:metadata/dc:language") or "en"

    manifest: dict[str, tuple[str, str]] = {}
    for item in opf.findall("opf:manifest/opf:item", ns):
        manifest[item.get("id", "")] = (
            item.get("href", ""),
            item.get("media-type", ""),
        )

    cover_bytes: bytes | None = None
    cover_media = ""
    for item_id, (href, media) in manifest.items():
        props = ""
        for item in opf.findall("opf:manifest/opf:item", ns):
            if item.get("id") == item_id:
                props = item.get("properties", "")
        if "cover-image" in props or item_id.lower() == "cover-image":
            cover_media = media
            try:
                cover_bytes = epub.read(prefix + href)
            except KeyError:
                cover_bytes = None
            break
    if cover_bytes is None:
        for item_id, (href, media) in manifest.items():
            if media.startswith("image/") and "cover" in (item_id + href).lower():
                cover_media = media
                try:
                    cover_bytes = epub.read(prefix + href)
                except KeyError:
                    cover_bytes = None
                break

    chapters: list[ParsedChapter] = []
    spine = opf.find("opf:spine", ns)
    if spine is None:
        raise ValueError("EPUB OPF has no spine")
    for itemref in spine.findall("opf:itemref", ns):
        ref = itemref.get("idref", "")
        if ref not in manifest:
            continue
        href, media = manifest[ref]
        if "html" not in media and "xhtml" not in media and "xml" not in media:
            continue
        try:
            raw = epub.read(prefix + href).decode("utf-8", "replace")
        except KeyError:
            continue
        cleaned = sanitize_html(raw)
        text = extract_text(cleaned)
        if not text:
            continue
        heading = ""
        for tag in ("h1", "h2", "h3"):
            start = cleaned.lower().find(f"<{tag}>")
            end = cleaned.lower().find(f"</{tag}>")
            if 0 <= start < end:
                heading = extract_text(cleaned[start : end + len(tag) + 3]).strip()
                break
        chapters.append(
            ParsedChapter(
                title=heading or f"Chapter {len(chapters) + 1}",
                html=cleaned,
                text=text,
                word_count=len(text.split()),
            )
        )

    if not chapters:
        raise ValueError("EPUB contains no readable spine chapters")
    return ParsedEpub(
        title=title,
        author=author,
        language=language,
        cover_bytes=cover_bytes,
        cover_media_type=cover_media,
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
    )


def make_db(settings: Settings):
    import pg8000.dbapi

    # pg8000.dbapi.connect accepts a full URL via the `host` parameter? No:
    # parse DATABASE_URL into parts explicitly.
    from urllib.parse import urlparse

    parts = urlparse(settings.database_url)
    return pg8000.dbapi.connect(
        user=parts.username or "baca_user",
        password=parts.password or "",
        host=parts.hostname or "localhost",
        port=parts.port or 5432,
        database=(parts.path or "/project_baca_db").lstrip("/"),
    )


# Gemini REST helpers (stdlib urllib, no extra dependency)

def _gemini_post(url: str, payload: dict, timeout: int = 60) -> dict:
    import urllib.error

    body = json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(url, data=body, headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", "replace")[:500]
        raise RuntimeError(f"Gemini API {exc.code}: {detail}") from exc


def embed_batch(texts: list[str], settings: Settings) -> list[list[float]]:
    """Returns one embedding vector per input text via batchEmbedContents."""
    if not settings.gemini_api_key:
        raise RuntimeError("GEMINI_API_KEY is not configured")
    url = (
        "https://generativelanguage.googleapis.com/v1beta/models/"
        f"{settings.embedding_model}:batchEmbedContents?key={settings.gemini_api_key}"
    )
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
        result = _gemini_post(url, payload)
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


def _strip_fences(text: str) -> str:
    text = text.strip()
    if text.startswith("```"):
        text = text.split("\n", 1)[1] if "\n" in text else ""
        if text.rstrip().endswith("```"):
            text = text.rstrip()[: -3]
    return text.strip()


def generate_json(prompt: str, settings: Settings) -> dict:
    """Calls the text LLM and parses the JSON object response."""
    if not settings.gemini_api_key:
        raise RuntimeError("GEMINI_API_KEY is not configured")
    url = (
        "https://generativelanguage.googleapis.com/v1beta/models/"
        f"{settings.llm_model}:generateContent?key={settings.gemini_api_key}"
    )
    payload = {"contents": [{"parts": [{"text": prompt}]}]}
    result = _gemini_post(url, payload, timeout=120)
    try:
        text = result["candidates"][0]["content"]["parts"][0]["text"]
    except (KeyError, IndexError, TypeError) as exc:
        raise RuntimeError(f"LLM response missing text: {result}") from exc
    return json.loads(_strip_fences(text))


# Pipeline

LOG = logging.getLogger("ingestion")


def set_job(redis_client, job_id: str, status: str, progress: int, error: str = "") -> None:
    fields = {
        "status": status,
        "progress": str(progress),
        "updated_at": datetime.now(timezone.utc).isoformat(),
    }
    if error:
        fields["error"] = error[:2000]
    redis_client.hset(f"job:{job_id}", mapping=fields)


def convert_cover_to_webp(cover_bytes: bytes) -> tuple[bytes, str]:
    from PIL import Image

    with Image.open(io.BytesIO(cover_bytes)) as image:
        canvas = image.convert("RGB")
        buffer = io.BytesIO()
        canvas.save(buffer, format="WEBP", quality=82)
        return buffer.getvalue(), "image/webp"


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
        response = minio_client.get_object(settings.bucket_epubs, storage_path)
        epub_bytes = response.read()
        response.close()
        response.release_conn()
        parsed = parse_epub(epub_bytes)

        # Metadata backfill: fill only blank admin-provided fields.
        cur.execute(
            "SELECT title, author, language FROM books WHERE id = %s",
            (book_id,),
        )
        row = cur.fetchone()
        if row is None:
            raise RuntimeError(f"Book row missing: {book_id}")
        updates: dict[str, str] = {}
        if (not row[0] or row[0] == "Unknown") and parsed.title:
            updates["title"] = parsed.title
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

        # Cover extraction (WebP when Pillow cooperates, original otherwise).
        if parsed.cover_bytes:
            try:
                cover_data, cover_type = convert_cover_to_webp(parsed.cover_bytes)
                cover_key = f"covers/{book_id}.webp"
            except Exception as exc:  # noqa: BLE001 - Pillow must never kill ingestion
                LOG.warning("cover conversion failed, storing original: %s", exc)
                ext = "png" if "png" in parsed.cover_media_type else "jpg"
                cover_data, cover_type = parsed.cover_bytes, parsed.cover_media_type or "image/jpeg"
                cover_key = f"covers/{book_id}.{ext}"
            minio_client.put_object(
                settings.bucket_covers,
                cover_key,
                io.BytesIO(cover_data),
                len(cover_data),
                content_type=cover_type,
            )
            cur.execute(
                "UPDATE books SET cover_url = %s, updated_at = NOW() WHERE id = %s",
                (cover_key, book_id),
            )

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

        # Scene chunking at ~400 words, then batched embeddings.
        pending: list[tuple[str, int, str]] = []
        for chapter_id, chapter in zip(chapter_ids, parsed.chapters):
            for index, piece in enumerate(chunk_words(chapter.text)):
                pending.append((chapter_id, index, piece))
        if not pending:
            raise RuntimeError("No embeddable chunks produced")

        set_job(redis_client, job_id, "embedding", 55)
        cur.execute("DELETE FROM book_chunks WHERE book_id = %s", (book_id,))
        for offset in range(0, len(pending), 20):
            batch = pending[offset : offset + 20]
            vectors = embed_batch([text for _, _, text in batch], settings)
            for (chapter_id, index, text), vector in zip(batch, vectors):
                literal = "[" + ",".join(repr(v) for v in vector) + "]"
                cur.execute(
                    """INSERT INTO book_chunks
                       (id, book_id, chapter_id, chunk_index, chunk_text, embedding)
                       VALUES (%s, %s, %s, %s, %s, %s::vector)""",
                    (
                        str(uuid.uuid4()),
                        book_id,
                        chapter_id,
                        index,
                        text,
                        literal,
                    ),
                )
            done = min(offset + 20, len(pending))
            set_job(redis_client, job_id, "embedding", 55 + int(20 * done / len(pending)))

        # Atomic cards + spoiler-free recaps via the text LLM.
        set_job(redis_client, job_id, "summarizing", 80)
        for number, (chapter_id, chapter) in enumerate(
            zip(chapter_ids, parsed.chapters), start=1
        ):
            cards = generate_json(ATOMIC_CARDS_PROMPT + chapter.text[:12000], settings)
            cur.execute(
                """INSERT INTO tldr_cache (id, book_id, chapter_id, recap_type, content_json, model_version)
                   VALUES (%s, %s, %s, 'chapter_atomic_cards', %s::jsonb, %s)
                   ON CONFLICT DO NOTHING""",
                (
                    str(uuid.uuid4()),
                    book_id,
                    chapter_id,
                    json.dumps(cards),
                    settings.llm_model,
                ),
            )
            if number > 1:  # recaps start at chapter 2
                recap = generate_json(RECAP_PROMPT + chapter.text[:12000], settings)
                cur.execute(
                    """INSERT INTO tldr_cache (id, book_id, chapter_id, recap_type, content_json, model_version)
                       VALUES (%s, %s, %s, 'chapter_recap', %s::jsonb, %s)
                       ON CONFLICT DO NOTHING""",
                    (
                        str(uuid.uuid4()),
                        book_id,
                        chapter_id,
                        json.dumps(recap),
                        settings.llm_model,
                    ),
                )
            set_job(
                redis_client,
                job_id,
                "summarizing",
                80 + int(15 * number / len(chapter_ids)),
            )

        # Publish the book.
        minutes = max(1, total_words // 200)
        cur.execute(
            """UPDATE books SET total_words = %s, estimated_reading_minutes = %s,
                      status = 'published', updated_at = NOW() WHERE id = %s""",
            (total_words, minutes, book_id),
        )
        conn.commit()
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


# Consumer loop

def ensure_group(redis_client, settings: Settings) -> None:
    try:
        redis_client.xgroup_create(
            settings.stream, settings.group, id="0", mkstream=True
        )
    except Exception as exc:  # noqa: BLE001 - BUSYGROUP means "already exists"
        if "BUSYGROUP" not in str(exc):
            raise


def run(once: bool = False) -> int:
    logging.basicConfig(
        level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s"
    )
    settings = Settings()
    redis_client = make_redis(settings)
    ensure_group(redis_client, settings)
    consumer = f"{socket.gethostname()}-{os.getpid()}"
    processed = 0

    while True:
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
            # Reclaim entries idle over 5 min (dead-worker recovery).
            try:
                _, entries = (
                    redis_client.xautoclaim(
                        settings.stream,
                        settings.group,
                        consumer,
                        min_idle_time=settings.reclaim_idle_ms,
                        start_id="0-0",
                        count=1,
                    )
                    or (None, [])
                )
            except Exception as exc:  # noqa: BLE001 - reclaim is best-effort
                LOG.warning("xautoclaim failed: %s", exc)
                entries = []
        if not entries:
            if once:
                break
            continue
        for msg_id, fields in entries:
                job_id = fields.get("job_id", msg_id)
                attempts_key = f"job:{job_id}"
                try:
                    attempts = int(redis_client.hget(attempts_key, "attempts") or 0)
                except (TypeError, ValueError):
                    attempts = 0
                try:
                    process_message(msg_id, fields, settings)
                    redis_client.xack(settings.stream, settings.group, msg_id)
                    processed += 1
                except Exception as exc:  # noqa: BLE001 - worker must survive bad jobs
                    LOG.exception("job %s failed: %s", job_id, exc)
                    redis_client.hincrby(attempts_key, "attempts", 1)
                    if attempts + 1 >= 3:
                        redis_client.xadd(
                            settings.dlq_stream,
                            {"job_id": job_id, "error": str(exc)[:2000]},
                        )
                        set_job(redis_client, job_id, "failed", 100, error=str(exc))
                        redis_client.xack(settings.stream, settings.group, msg_id)
                    # Otherwise leave unacknowledged for XPENDING reclaim.
                if once:
                    return 0
    return processed


def main() -> None:
    sys.exit(run(once="--once" in sys.argv[1:]))


if __name__ == "__main__":
    main()
