"""API contract tests: hit endpoints directly.

Locks the wire contract so docs (10f) cannot go stale silently:
envelope shape, 401/404/429 paths, merge-cap boundary.
"""
import json
import urllib.error
import urllib.request

import pytest

from conftest import API


def call(method: str, path: str, data=None, token: str | None = None):
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(
        f"{API}{path}",
        data=json.dumps(data).encode() if data is not None else None,
        headers=headers,
        method=method,
    )
    try:
        with urllib.request.urlopen(req) as r:
            return r.status, json.load(r)
    except urllib.error.HTTPError as e:
        return e.code, json.loads(e.read() or b"{}")


def test_envelope_and_health():
    status, body = call("GET", "/books?limit=1")
    assert status == 200
    assert body["success"] is True
    assert isinstance(body["data"], list)


def test_unknown_book_search_404():
    status, body = call(
        "POST", "/books/00000000-0000-0000-0000-000000000099/quotes/search",
        {"query": "anything at all", "limit": 5},
    )
    assert status == 404
    assert body["success"] is False


def test_save_quote_requires_auth():
    status, _ = call("POST", "/quotes/save", {"book_id": "x", "chapter_id": "y", "quote_text": "z"})
    assert status in (400, 401)


def test_merge_cap_boundary(book_with_chapters: str):
    books = call("GET", "/books?limit=1")[1]["data"]
    assert books, "catalog must be non-empty"
