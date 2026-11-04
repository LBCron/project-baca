"""The static home shell in index.html paints the phone hero heading before the
WASM arrives, in the place, words, and faces the app then draws, and only on
the home page."""
import pytest
from playwright.sync_api import Browser

from conftest import GUEST_PROGRESS, WEB

VISIBLE_HEADING = """() => {
  const h = [...document.querySelectorAll('h1.hero-heading')]
    .find(e => e.offsetParent !== null && getComputedStyle(e).visibility === 'visible');
  if (!h) return null;
  const r = h.getBoundingClientRect();
  return {text: h.innerText.replace(/\\s+/g, ' ').trim(), x: r.x, y: r.y, w: r.width, h: r.height,
          inShell: !!h.closest('#boot-shell')};
}"""


def _heading(browser: Browser, path: str, width: int, height: int, wasm: bool,
             stored: dict[str, str], locale: str = "en-US"):
    context = browser.new_context(color_scheme="dark", locale=locale, viewport={"width": width, "height": height})
    for key, value in stored.items():
        context.add_init_script(f"localStorage.setItem('{key}', '{value}')")
    if not wasm:
        context.route("**/*.wasm", lambda route: route.abort())
    page = context.new_page()
    try:
        page.goto(f"{WEB}{path}", wait_until="domcontentloaded")
        if wasm:
            page.wait_for_function("() => !document.getElementById('boot-shell') && document.querySelector('.site-header .header-tools')")
            page.evaluate("() => document.fonts.ready")
        else:
            page.wait_for_function("() => { const h = document.documentElement; return !h.hasAttribute('data-boot') || h.hasAttribute('data-boot-fonts'); }")
        return page.evaluate(VISIBLE_HEADING)
    finally:
        context.close()


def _assert_same(static, app):
    assert static and static["inShell"], "no static heading before the WASM"
    assert app and not app["inShell"], "the app heading did not replace the shell"
    assert static["text"] == app["text"]
    for key in ("x", "y", "w", "h"):
        assert abs(static[key] - app[key]) <= 1, f"{key}: static {static} app {app}"


@pytest.mark.parametrize("width,height", [(375, 812), (700, 900)], ids=["phone", "wide-phone"])
@pytest.mark.parametrize("lang", ["ID", "EN"])
def test_static_heading_matches_the_app(browser: Browser, width, height, lang):
    stored = {"rotaria_lang": lang}
    _assert_same(_heading(browser, "/", width, height, False, stored),
                 _heading(browser, "/", width, height, True, stored))


@pytest.mark.parametrize("locale", ["id-ID", "en-GB"])
def test_static_heading_follows_the_browser_language(browser: Browser, locale):
    """Without a stored choice both sides fall back to the browser language."""
    _assert_same(_heading(browser, "/", 375, 812, False, {}, locale),
                 _heading(browser, "/", 375, 812, True, {}, locale))


def test_wide_screens_and_other_routes_show_no_static_heading(browser: Browser):
    assert _heading(browser, "/", 1280, 800, False, {"rotaria_lang": "ID"}) is None
    assert _heading(browser, "/me", 375, 812, False, {"rotaria_lang": "ID"}) is None


def test_a_reader_with_a_book_to_resume_gets_no_fresh_visitor_heading(browser: Browser, book_with_chapters: str):
    """The app greets a guest who has read a page with the book to resume, so
    on the next visit the fresh-visitor heading must not stand in for it."""
    context = browser.new_context(color_scheme="dark", viewport={"width": 375, "height": 812})
    try:
        page = context.new_page()
        page.goto(f"{WEB}/read/{book_with_chapters}?chapter=1", wait_until="domcontentloaded")
        page.wait_for_selector(".chapter-body")
        page.wait_for_timeout(2200)
        page.keyboard.press("ArrowRight")
        deadline = 20
        while not page.evaluate(GUEST_PROGRESS) and deadline:
            page.wait_for_timeout(500)
            deadline -= 1
        assert page.evaluate(GUEST_PROGRESS), "no guest position saved"
        page.goto(f"{WEB}/", wait_until="networkidle")
        page.wait_for_selector(".resume-card")
        assert page.evaluate("() => localStorage.getItem('rotaria_resume_hint')") == "1"
        context.route("**/*.wasm", lambda route: route.abort())
        page.goto(f"{WEB}/", wait_until="domcontentloaded")
        assert page.evaluate("() => document.documentElement.hasAttribute('data-boot')") is False
        assert page.evaluate(VISIBLE_HEADING) is None
    finally:
        context.close()
