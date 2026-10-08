"""Paper page turn: strips appear while a page is dragged, a release past the
middle lands on the next page, an early release falls back, taps and keys
turn pages, and reduced motion skips the paper entirely."""
import pytest
from playwright.sync_api import Browser

from conftest import WEB, api_get

LONG_CHAPTER = "<h2>Probe</h2>" + "".join(
    f"<p>Paragraf percobaan nomor {i + 1}. " + "Kalimat panjang yang mengisi halaman. " * 8 + "</p>"
    for i in range(40)
)
STATUS = "() => document.querySelector('.reader-status span').textContent"
SCROLL_PAGE = "() => { const v = document.querySelector('.reader-viewport'); return Math.round(v.scrollLeft / v.clientWidth); }"
STRIPS = "() => document.querySelectorAll('.flip-layer .flip-strip').length"


def _open_long_chapter(browser: Browser, book_id: str, width: int, height: int, reduced: bool):
    context = browser.new_context(
        viewport={"width": width, "height": height},
        reduced_motion="reduce" if reduced else "no-preference",
        color_scheme="dark",
    )
    page = context.new_page()
    errors: list[str] = []
    page.on("pageerror", lambda e: errors.append(str(e)))
    chapter = api_get(f"/books/{book_id}")["chapters"][0]["chapter_number"]
    page.goto(f"{WEB}/read/{book_id}?chapter={chapter}", wait_until="networkidle")
    page.wait_for_selector(".chapter-body")
    page.evaluate(
        "html => { document.querySelector('.chapter-body').innerHTML = html; window.dispatchEvent(new Event('resize')); }",
        LONG_CHAPTER,
    )
    page.wait_for_timeout(700)
    book = page.evaluate("() => document.querySelector('.book').getBoundingClientRect().toJSON()")
    return context, page, errors, book


def _drag(page, book, from_frac: float, to_frac: float, steps: int = 10):
    y = book["y"] + book["height"] / 2
    x0 = book["x"] + book["width"] * from_frac
    x1 = book["x"] + book["width"] * to_frac
    page.mouse.move(x0, y)
    page.mouse.down()
    for i in range(1, steps + 1):
        page.mouse.move(x0 + (x1 - x0) * i / steps, y)
        page.wait_for_timeout(20)
    return y


@pytest.mark.parametrize("name,width,height,strips", [("desktop", 1280, 800, 8), ("phone", 390, 844, 6)])
def test_drag_turns_like_paper(browser: Browser, book_with_chapters: str, name, width, height, strips):
    context, page, errors, book = _open_long_chapter(browser, book_with_chapters, width, height, reduced=False)
    try:
        assert page.evaluate(SCROLL_PAGE) == 0
        y = _drag(page, book, 0.9, 0.6)
        page.wait_for_timeout(120)
        assert page.evaluate(STRIPS) == strips, f"{name}: strips while dragging"
        assert page.evaluate("() => document.querySelectorAll('.flip-layer .reader-viewport.clone').length") >= strips
        page.mouse.move(book["x"] + book["width"] * 0.2, y)
        page.mouse.up()
        page.wait_for_timeout(1000)
        assert page.evaluate(STRIPS) == 0, f"{name}: leaf removed after landing"
        assert page.evaluate(SCROLL_PAGE) == 1, f"{name}: landed on the next page"

        # an early release falls back to the same page
        y = _drag(page, book, 0.9, 0.84, steps=4)
        page.mouse.up()
        page.wait_for_timeout(800)
        assert page.evaluate(SCROLL_PAGE) == 1, f"{name}: short drag must fall back"
        assert page.evaluate(STRIPS) == 0

        # a tap on the right edge turns forward, on the left edge back
        page.mouse.click(book["x"] + book["width"] * 0.92, y)
        page.wait_for_timeout(1100)
        assert page.evaluate(SCROLL_PAGE) == 2, f"{name}: tap forward"
        page.mouse.click(book["x"] + book["width"] * 0.08, y)
        page.wait_for_timeout(1100)
        assert page.evaluate(SCROLL_PAGE) == 1, f"{name}: tap back"
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_keyboard_turns_and_dims(browser: Browser, book_with_chapters: str):
    context, page, errors, book = _open_long_chapter(browser, book_with_chapters, 1280, 800, reduced=False)
    try:
        page.keyboard.press("Space")
        page.wait_for_timeout(1100)
        assert page.evaluate(SCROLL_PAGE) == 1
        page.keyboard.press("Shift+Space")
        page.wait_for_timeout(1100)
        assert page.evaluate(SCROLL_PAGE) == 0
        page.keyboard.press("End")
        page.wait_for_timeout(300)
        assert page.evaluate(SCROLL_PAGE) > 1
        page.keyboard.press("Home")
        page.wait_for_timeout(300)
        assert page.evaluate(SCROLL_PAGE) == 0
        page.mouse.click(book["x"] + book["width"] * 0.5, book["y"] + book["height"] * 0.5)
        page.wait_for_timeout(200)
        assert page.evaluate("() => document.querySelector('.reader').classList.contains('dim')")
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_reduced_motion_turns_without_paper(browser: Browser, book_with_chapters: str):
    context, page, errors, book = _open_long_chapter(browser, book_with_chapters, 1280, 800, reduced=True)
    try:
        page.click(".reader-zone-next")
        page.wait_for_timeout(80)
        assert page.evaluate(STRIPS) == 0
        assert page.evaluate(SCROLL_PAGE) == 1
        assert errors == [], errors[:3]
    finally:
        context.close()
