"""Service worker and house fonts: deploys reach returning readers, the shell
cache stays one entry, nothing loads from a third party, and the fonts work
offline."""
from urllib.parse import urlparse

from playwright.sync_api import Browser, Page

from conftest import WEB, api_get

SHELL_CACHE = "rotaria-shell-v3"
ASSET_CACHE = "rotaria-assets-v3"
OWN_HOSTS = {"127.0.0.1:3000", "localhost:8080"}
HOUSE_FONTS = {"EB Garamond", "Newsreader", "Plus Jakarta Sans"}
LOADED_FONTS = """async () => {
  await document.fonts.ready;
  return [...document.fonts].filter(f => f.status === 'loaded').map(f => f.family.replace(/"/g, ''));
}"""


def _controlled(page: Page, path: str = "/") -> None:
    """Opens `path` with the service worker installed and in control."""
    page.goto(f"{WEB}{path}", wait_until="networkidle")
    page.evaluate("() => navigator.serviceWorker.ready")
    if not page.evaluate("() => !!navigator.serviceWorker.controller"):
        page.reload(wait_until="networkidle")
    assert page.evaluate("() => !!navigator.serviceWorker.controller")


def _new_page(browser: Browser):
    context = browser.new_context(color_scheme="dark")
    page = context.new_page()
    errors: list[str] = []
    page.on("pageerror", lambda e: errors.append(str(e)))
    return context, page, errors


def test_home_and_reader_load_nothing_from_third_parties(browser: Browser, book_with_chapters: str):
    context, page, errors = _new_page(browser)
    hosts: set[str] = set()
    page.on("request", lambda r: hosts.add(urlparse(r.url).netloc))
    try:
        page.goto(f"{WEB}/", wait_until="networkidle")
        page.goto(f"{WEB}/read/{book_with_chapters}?chapter=1", wait_until="networkidle")
        page.wait_for_selector(".chapter-body")
        assert hosts <= OWN_HOSTS, f"third-party hosts: {hosts - OWN_HOSTS}"
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_shell_cache_keeps_one_navigation_entry(browser: Browser, book_with_chapters: str):
    context, page, errors = _new_page(browser)
    book = book_with_chapters
    routes = [
        "/", "/me", f"/book/{book}", f"/book/{book}#daftar-bab", f"/read/{book}?chapter=1",
        f"/read/{book}?chapter=2", "/admin", "/?search=1", "/no-such-page", "/me?tab=quotes",
    ]
    try:
        _controlled(page)
        for route in routes:
            page.goto(f"{WEB}{route}", wait_until="domcontentloaded")
            page.wait_for_timeout(300)
        entries = page.evaluate(
            "name => caches.open(name).then(c => c.keys()).then(ks => ks.map(r => new URL(r.url).pathname + new URL(r.url).search))",
            SHELL_CACHE,
        )
        assert entries == ["/"], entries
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_unhashed_files_are_fresh_on_the_next_visit(browser: Browser):
    context, page, errors = _new_page(browser)
    try:
        _controlled(page)
        page.evaluate(
            """name => caches.open(name).then(c => c.put('/boot.js',
                 new Response('/* stale */', { headers: { 'Content-Type': 'text/javascript' } })))""",
            ASSET_CACHE,
        )
        page.reload(wait_until="networkidle")
        served = page.evaluate("() => fetch('/boot.js').then(r => r.text())")
        assert "stale" not in served and "rotaria_theme" in served, served[:80]
        cached = page.evaluate(
            "name => caches.open(name).then(c => c.match('/boot.js')).then(r => r && r.text())", ASSET_CACHE
        )
        assert cached and "stale" not in cached, "the cache follows the network copy"
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_house_fonts_offline_on_home_and_reader(browser: Browser, book_with_chapters: str):
    context, page, errors = _new_page(browser)
    book = book_with_chapters
    chapter = api_get(f"/books/{book}")["chapters"][0]["chapter_number"]
    try:
        _controlled(page, f"/book/{book}")
        save = '[aria-label="Save for offline reading"]'
        if page.query_selector(save):
            page.click(save)
            page.wait_for_selector('[aria-label="Saved on this device"]', timeout=10000)
        page.goto(f"{WEB}/read/{book}?chapter={chapter}", wait_until="networkidle")
        page.wait_for_selector(".chapter-body")
        page.goto(f"{WEB}/", wait_until="networkidle")

        context.set_offline(True)
        page.goto(f"{WEB}/", wait_until="domcontentloaded")
        page.wait_for_selector(".brand-title")
        home_fonts = set(page.evaluate(LOADED_FONTS))
        assert {"EB Garamond", "Plus Jakarta Sans"} <= home_fonts, home_fonts
        page.goto(f"{WEB}/read/{book}?chapter={chapter}", wait_until="domcontentloaded")
        page.wait_for_selector(".chapter-body")
        reader_fonts = set(page.evaluate(LOADED_FONTS))
        assert HOUSE_FONTS <= reader_fonts, reader_fonts
        assert errors == [], errors[:3]
    finally:
        context.set_offline(False)
        context.close()


def test_native_shell_sets_the_api_base_and_skips_the_worker(browser: Browser):
    """The Tauri app injects the API base and keeps no service worker; one
    bundle serves both."""
    context = browser.new_context(color_scheme="dark")
    context.add_init_script(
        "window.__TAURI_INTERNALS__ = {}; window.__ROTARIA_API__ = 'http://localhost:8080/api/v1/';"
    )
    page = context.new_page()
    api_calls: list[str] = []
    page.on("request", lambda r: api_calls.append(r.url) if "/api/v1/" in r.url else None)
    errors: list[str] = []
    page.on("pageerror", lambda e: errors.append(str(e)))
    try:
        page.goto(f"{WEB}/", wait_until="networkidle")
        page.wait_for_selector(".book-card")
        page.wait_for_timeout(500)
        assert page.evaluate("() => navigator.serviceWorker.getRegistrations().then(r => r.length)") == 0
        assert api_calls and all("/api/v1//" not in url for url in api_calls), api_calls[:3]
        assert errors == [], errors[:3]
    finally:
        context.close()
