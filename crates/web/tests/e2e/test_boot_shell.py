"""The static home shell in index.html paints the phone hero heading before the
WASM arrives, in the place, words, and faces the app then draws, and only on
the home page."""
import pytest
from playwright.sync_api import Browser

from conftest import WEB

VISIBLE_HEADING = """() => {
  const h = [...document.querySelectorAll('h1.hero-heading')]
    .find(e => e.offsetParent !== null && getComputedStyle(e).visibility === 'visible');
  if (!h) return null;
  const r = h.getBoundingClientRect();
  return {text: h.innerText.replace(/\\s+/g, ' ').trim(), x: r.x, y: r.y, w: r.width, h: r.height,
          inShell: !!h.closest('#boot-shell')};
}"""


def _heading(browser: Browser, path: str, width: int, height: int, lang: str, wasm: bool):
    context = browser.new_context(color_scheme="dark", viewport={"width": width, "height": height})
    context.add_init_script(f"localStorage.setItem('rotaria_lang', '{lang}')")
    if not wasm:
        context.route("**/*.wasm", lambda route: route.abort())
    page = context.new_page()
    try:
        page.goto(f"{WEB}{path}", wait_until="domcontentloaded")
        if wasm:
            page.wait_for_function("() => !document.getElementById('boot-shell') && document.querySelector('.site-header .header-tools')")
            page.evaluate("() => document.fonts.ready")
        else:
            page.wait_for_function("() => location.pathname !== '/' || document.documentElement.hasAttribute('data-boot-fonts')")
        return page.evaluate(VISIBLE_HEADING)
    finally:
        context.close()


@pytest.mark.parametrize("width,height", [(375, 812), (700, 900)], ids=["phone", "wide-phone"])
@pytest.mark.parametrize("lang", ["ID", "EN"])
def test_static_heading_matches_the_app(browser: Browser, width, height, lang):
    static = _heading(browser, "/", width, height, lang, wasm=False)
    app = _heading(browser, "/", width, height, lang, wasm=True)
    assert static and static["inShell"], "no static heading before the WASM"
    assert app and not app["inShell"], "the app heading did not replace the shell"
    assert static["text"] == app["text"]
    for key in ("x", "y", "w", "h"):
        assert abs(static[key] - app[key]) <= 1, f"{key}: static {static} app {app}"


def test_wide_screens_and_other_routes_show_no_static_heading(browser: Browser):
    assert _heading(browser, "/", 1280, 800, "ID", wasm=False) is None
    assert _heading(browser, "/me", 375, 812, "ID", wasm=False) is None
