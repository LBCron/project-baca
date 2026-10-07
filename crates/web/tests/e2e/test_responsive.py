"""Responsive contract across phone, foldable, tablet, and desktop viewports.

Layout, not pixels: no horizontal overflow, the right chrome per width
(tab bar vs header navigation, dock vs inline actions), tap targets at
least 24px, and a long chapter that really paginates everywhere.
"""
import pytest
from playwright.sync_api import Browser

from conftest import WEB, api_get

# (name, width, height): small phone, fold cover, phone, fold inner,
# short landscape phone, tablet portrait, desktop, wide desktop.
VIEWPORTS = [
    ("se", 320, 568),
    ("fold-cover", 344, 882),
    ("phone", 390, 844),
    ("fold-inner", 690, 829),
    ("phone-landscape", 740, 360),
    ("tablet", 768, 1024),
    ("desktop", 1280, 720),
    ("wide", 1920, 1080),
]

OVERFLOW_JS = "() => document.documentElement.scrollWidth - window.innerWidth"

SMALL_TARGETS_JS = """
() => Array.from(document.querySelectorAll('a, button, input, [role=button]'))
  .filter(el => {
    const cs = getComputedStyle(el);
    if (cs.display === 'none' || cs.visibility === 'hidden') return false;
    if (el.classList.contains('reader-zone') || el.classList.contains('visually-hidden')) return false;
    const r = el.getBoundingClientRect();
    return r.width > 0 && r.height > 0 && (r.width < 24 || r.height < 24);
  })
  .map(el => el.tagName + '.' + el.className)
"""

LONG_CHAPTER = "<h2>Probe</h2>" + "".join(
    f"<p>Paragraf percobaan nomor {i + 1}. " + "Kalimat panjang yang mengisi halaman. " * 8 + "</p>"
    for i in range(40)
)


def _page_at(browser: Browser, width: int, height: int):
    context = browser.new_context(
        viewport={"width": width, "height": height},
        is_mobile=width < 1024,
        has_touch=width < 1024,
        reduced_motion="reduce",
    )
    page = context.new_page()
    errors: list[str] = []
    page.on("pageerror", lambda e: errors.append(str(e)))
    page.errors = errors  # type: ignore[attr-defined]
    return context, page


def _visible(page, selector: str) -> bool:
    el = page.query_selector(selector)
    return bool(el and el.is_visible())


@pytest.mark.parametrize("name,width,height", VIEWPORTS, ids=[v[0] for v in VIEWPORTS])
def test_home_and_book_fit_viewport(browser: Browser, book_with_chapters: str, name, width, height):
    context, page = _page_at(browser, width, height)
    try:
        phone = width < 768
        page.goto(f"{WEB}/", wait_until="networkidle")
        page.wait_for_timeout(800)
        assert page.evaluate(OVERFLOW_JS) <= 0, f"{name}: home overflows horizontally"
        assert _visible(page, ".tabbar") == phone, f"{name}: tab bar visibility"
        assert _visible(page, ".nav-links") == (not phone), f"{name}: header nav visibility"
        assert _visible(page, ".hero-art") == (not phone), f"{name}: hero plate visibility"
        assert _visible(page, ".greeting") == phone, f"{name}: greeting visibility"
        assert page.evaluate(SMALL_TARGETS_JS) == [], f"{name}: tap targets under 24px"

        page.goto(f"{WEB}/book/{book_with_chapters}", wait_until="networkidle")
        page.wait_for_timeout(800)
        assert page.evaluate(OVERFLOW_JS) <= 0, f"{name}: book page overflows horizontally"
        assert not _visible(page, ".tabbar"), f"{name}: tab bar must step aside for the dock"
        dock = page.query_selector(".book-dock")
        assert dock is not None
        position = page.evaluate("el => getComputedStyle(el).position", dock)
        assert position == ("fixed" if phone else "static"), f"{name}: dock position {position}"
        assert page.evaluate(SMALL_TARGETS_JS) == [], f"{name}: tap targets under 24px"
        assert page.errors == [], page.errors[:3]  # type: ignore[attr-defined]
    finally:
        context.close()


@pytest.mark.parametrize("name,width,height", VIEWPORTS, ids=[v[0] for v in VIEWPORTS])
def test_reader_paginates_long_chapter(browser: Browser, book_with_chapters: str, name, width, height):
    detail = api_get(f"/books/{book_with_chapters}")
    ch_no = detail["chapters"][0]["chapter_number"]
    context, page = _page_at(browser, width, height)
    try:
        page.goto(f"{WEB}/read/{book_with_chapters}?chapter={ch_no}", wait_until="networkidle")
        page.wait_for_selector(".chapter-body")
        page.evaluate(
            "html => { document.querySelector('.chapter-body').innerHTML = html; window.dispatchEvent(new Event('resize')); }",
            LONG_CHAPTER,
        )
        page.wait_for_timeout(600)
        metrics = page.evaluate(
            """() => {
              const vp = document.querySelector('.reader-viewport');
              const reader = document.querySelector('.reader');
              return {
                pages: Math.ceil(vp.scrollWidth / vp.clientWidth),
                viewportHeight: vp.clientHeight,
                readerHeight: reader.getBoundingClientRect().height,
                columns: parseFloat(getComputedStyle(vp).columnWidth) < vp.clientWidth / 2 ? 2 : 1,
              };
            }"""
        )
        assert metrics["pages"] > 1, f"{name}: long chapter must paginate, got {metrics}"
        assert metrics["readerHeight"] <= height + 1, f"{name}: reader taller than the screen {metrics}"
        assert metrics["viewportHeight"] < height, f"{name}: viewport must leave room for bar and footer"
        assert metrics["columns"] == (2 if width >= 1024 else 1), f"{name}: column count {metrics}"
        status = page.inner_text(".reader-status")
        assert "1 " in status or "1 of" in status or "Halaman 1" in status, status
        page.click(".reader-zone-next")
        page.wait_for_timeout(400)
        assert page.evaluate("document.querySelector('.reader-viewport').scrollLeft") == page.evaluate(
            "document.querySelector('.reader-viewport').clientWidth"
        ), f"{name}: one turn must advance exactly one viewport width"
        assert page.errors == [], page.errors[:3]  # type: ignore[attr-defined]
    finally:
        context.close()
