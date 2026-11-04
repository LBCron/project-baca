"""Shared pytest fixtures for the Rotaria web suite.

Requires: `make db-up`, `make seed-dev`, Axum on :8080 (`make dev-server`),
Trunk on :3000 (`make dev-web`).
"""
import json
import re
import subprocess
import time
import urllib.request

import pytest
from playwright.sync_api import Page, expect

WEB = "http://127.0.0.1:3000"
API = "http://localhost:8080/api/v1"
MAILPIT = "http://localhost:8025"


def api_get(path: str):
    with urllib.request.urlopen(f"{API}{path}") as r:
        return json.load(r)["data"]


SEED_BOOK_ID = "a0000000-0000-4000-8000-000000000001"


@pytest.fixture(scope="session")
def browser_context_args(browser_context_args):
    """The shell follows the system scheme on a first visit; goldens and
    structural checks are pinned to the Espresso surface, and the a11y suite
    scans the Paper surface explicitly."""
    return {**browser_context_args, "color_scheme": "dark"}


@pytest.fixture(scope="session")
def book_with_chapters() -> str:
    """ID of a book that has at least one chapter.

    Prefers the dev seed (`make seed-dev`, fixed id, deterministic content for
    visual goldens); falls back to any published book with chapters.
    """
    try:
        if api_get(f"/books/{SEED_BOOK_ID}").get("chapters"):
            return SEED_BOOK_ID
    except Exception:
        pass
    books = api_get("/books?limit=100")
    assert books, "catalog empty, run `make seed-dev` or ingest a book first"
    for book in books:
        try:
            detail = api_get(f"/books/{book['id']}")
        except Exception:
            continue
        if detail.get("chapters"):
            return book["id"]
    raise AssertionError("no book with chapters found; run `make seed-dev`")


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


def api_post(path: str, payload: dict, token: str | None = None):
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(f"{API}{path}", data=json.dumps(payload).encode(), headers=headers)
    with urllib.request.urlopen(req) as r:
        return json.load(r)["data"]


def mailpit_otp(email: str, timeout_s: int = 15) -> str:
    deadline = time.time() + timeout_s
    while time.time() < deadline:
        time.sleep(1)
        with urllib.request.urlopen(f"{MAILPIT}/api/v1/messages?limit=50") as r:
            msgs = json.load(r)["messages"]
        for m in msgs:
            if email in json.dumps(m):
                with urllib.request.urlopen(f"{MAILPIT}/api/v1/message/{m['ID']}") as r:
                    found = re.search(r"\b(\d{6})\b", json.dumps(json.load(r)))
                    if found:
                        return found.group(1)
    raise AssertionError("OTP email never arrived in Mailpit")


@pytest.fixture()
def reset_signup_cap():
    """Signup allows ten per hour per address, and every browser test signs
    up from 127.0.0.1; a full run plus a rerun would trip it. The dev Redis
    runs in the compose container, as `make seed-dev` assumes for Postgres."""
    subprocess.run(
        ["docker", "exec", "project_baca_redis", "redis-cli", "DEL", "otp:signup:ip:127.0.0.1"],
        capture_output=True,
        check=False,
    )


def register_reader(tag: str) -> dict:
    """Signs a fresh reader up through the real OTP mail; returns the token pair."""
    email = f"e2e-{tag}-{int(time.time() * 1000)}@example.com"
    started = api_post("/auth/signup", {"display_name": "E2E", "email": email, "password": "E2EPassword123!"})
    return api_post(
        "/auth/verify-otp",
        {"email": email, "registration_id": started["registration_id"], "otp": mailpit_otp(email)},
    )


def long_chapter_html(number: int, paragraphs: int = 40) -> str:
    """A chapter several pages long whose text names its chapter number."""
    return f"<h2>Bab {number}</h2>" + "".join(
        f"<p>Bab {number} paragraf {i + 1}. " + "Kalimat panjang yang mengisi halaman. " * 8 + "</p>"
        for i in range(paragraphs)
    )


# Wraps window.fetch inside the page (the WASM client calls it through
# web-sys), so a test can stretch chapters, delay or hold chosen responses,
# and keep its own timing independent of Playwright's dispatcher.
FETCH_SHIM = """
(() => {
  const cfg = %s;
  const original = window.fetch.bind(window);
  window.__held = [];
  window.__release = () => { window.__held.splice(0).forEach(go => go()); };
  const chapterOf = url => { const m = url.match(/\\/books\\/[^/]+\\/chapters\\/(\\d+)$/); return m ? Number(m[1]) : null; };
  window.fetch = (input, init) => {
    const url = (typeof input === 'string' ? input : input.url).split('?')[0];
    const number = chapterOf(url);
    let call = () => original(input, init);
    if (number !== null && cfg.longChapters) {
      call = () => original(input, init).then(r => r.json()).then(body => {
        body.data.html_content = cfg.longChapters[number - 1] || body.data.html_content;
        return new Response(JSON.stringify(body), { status: 200, headers: { 'Content-Type': 'application/json' } });
      });
    }
    if (cfg.hold.some(p => url.endsWith(p))) {
      return new Promise(resolve => window.__held.push(() => resolve(call())));
    }
    if (cfg.delay.some(p => url.endsWith(p))) {
      return new Promise(resolve => setTimeout(() => resolve(call()), cfg.delayMs));
    }
    return call();
  };
})();
"""


def install_fetch_shim(context, *, chapters: int = 0, delay=(), delay_ms: int = 1000, hold=()) -> None:
    config = {
        "longChapters": [long_chapter_html(n + 1) for n in range(chapters)] or None,
        "delay": list(delay),
        "delayMs": delay_ms,
        "hold": list(hold),
    }
    context.add_init_script(FETCH_SHIM % json.dumps(config))


GUEST_PROGRESS = """
() => new Promise(resolve => {
  const open = indexedDB.open('project_baca_db');
  open.onsuccess = () => {
    const db = open.result;
    if (!db.objectStoreNames.contains('guest_progress')) { resolve([]); return; }
    const all = db.transaction('guest_progress', 'readonly').objectStore('guest_progress').getAll();
    all.onsuccess = () => resolve(all.result);
  };
  open.onerror = () => resolve([]);
})
"""
