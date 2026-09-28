"""Shared pytest fixtures for the Rotaria web suite.

Requires: `make db-up`, Axum on :8080 (`make dev-server`),
Trunk on :3000 (`make dev-web`).
"""
import json
import urllib.request

import pytest
from playwright.sync_api import Page, expect

WEB = "http://127.0.0.1:3000"
API = "http://localhost:8080/api/v1"
MAILPIT = "http://localhost:8025"


def api_get(path: str):
    with urllib.request.urlopen(f"{API}{path}") as r:
        return json.load(r)["data"]


@pytest.fixture(scope="session")
def book_with_chapters() -> str:
    """ID of a book that has at least one chapter (never debris content)."""
    books = api_get("/books?limit=100")
    assert books, "catalog empty, seed or ingest a book first"
    # Prefer the stable seed book (deterministic content for visual goldens);
    # fall back to any debris book with chapters for flow tests.
    fallback = None
    for book in books:
        try:
            detail = api_get(f"/books/{book['id']}")
        except Exception:
            continue
        if detail.get("chapters"):
            if "Sitti Nurbaya" in (book.get("title") or "") and fallback is None:
                return book["id"]
            if fallback is None:
                fallback = book["id"]
    if fallback is not None:
        return fallback
    raise AssertionError("no book with chapters found; ingest a sample EPUB first")


@pytest.fixture(scope="session")
def stable_book_id(book_with_chapters: str) -> str:
    """Alias kept explicit for visual tests: they MUST use the pinned book."""
    return book_with_chapters


@pytest.fixture()
def clean_page(page: Page):
    """Fresh page with a pageerror collector attached."""
    errors: list[str] = []
    page.on("pageerror", lambda e: errors.append(str(e)))
    page.errors = errors  # type: ignore[attr-defined]
    yield page


def assert_no_page_errors(page: Page) -> None:
    assert page.errors == [], f"page errors: {page.errors[:3]}"  # type: ignore[attr-defined]


def goto(page: Page, path: str) -> None:
    page.goto(f"{WEB}{path}", wait_until="networkidle")
    page.wait_for_timeout(2000)
