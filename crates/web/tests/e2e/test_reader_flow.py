"""Reader flows: body render, type sheet, recap chip, offline save + read,
and the integrity of the saved reading position."""
import json
import urllib.request

import pytest
from playwright.sync_api import Page, expect

from conftest import (
    API,
    GUEST_PROGRESS,
    WEB,
    api_get,
    assert_no_page_errors,
    goto,
    install_fetch_shim,
    register_reader,
)

pytestmark = pytest.mark.usefixtures("reset_signup_cap")



def _chapter_no(book_id: str, index: int = 0) -> tuple[int, int]:
    detail = api_get(f"/books/{book_id}")
    chapters = detail["chapters"]
    return chapters[index]["chapter_number"], len(chapters)


def test_reader_body_and_type_sheet(clean_page: Page, book_with_chapters: str):
    page = clean_page
    ch_no, n_chapters = _chapter_no(book_with_chapters)
    goto(page, f"/read/{book_with_chapters}?chapter={ch_no}")
    page.wait_for_timeout(500)
    body = page.inner_text(".chapter-body") if page.query_selector(".chapter-body") else ""
    assert len(body) > 0, "chapter body must render"
    assert page.inner_text(".reader-status").startswith(("Page 1 of", "Pages 1"))
    page.click(".reader-tools .btn-type")
    page.wait_for_timeout(400)
    assert page.query_selector(".type-sheet") is not None
    page.click('.type-sheet .swatch:has-text("Sepia")')
    page.wait_for_timeout(300)
    assert page.get_attribute(".reader", "data-reading") == "sepia"
    if n_chapters > 1:
        ch2, _ = _chapter_no(book_with_chapters, 1)
        goto(page, f"/read/{book_with_chapters}?chapter={ch2}")
        assert page.query_selector(".recap-chip") is not None
    assert_no_page_errors(page)


def test_offline_save_read_badge(clean_page: Page, book_with_chapters: str):
    page = clean_page
    book_id = book_with_chapters
    ch_no, _ = _chapter_no(book_id)
    goto(page, f"/book/{book_id}")
    save = '[aria-label="Save for offline reading"]'
    if page.query_selector(save):
        page.click(save)
        page.wait_for_timeout(3000)
        assert page.query_selector('[aria-label="Saved on this device"]') is not None
    # One online visit installs the service worker, whose shell entry then
    # answers this route offline.
    goto(page, f"/read/{book_id}?chapter={ch_no}")
    page.wait_for_timeout(500)
    page.context.set_offline(True)
    try:
        page.goto(f"{WEB}/read/{book_id}?chapter={ch_no}", wait_until="commit")
        page.wait_for_timeout(2500)
        off_body = page.inner_text(".chapter-body") if page.query_selector(".chapter-body") else ""
        assert len(off_body) > 0, "offline copy must render from IndexedDB"
        footer = page.inner_text(".reader-status") if page.query_selector(".reader-status") else ""
        assert "offline copy" in footer, footer[:60]
    finally:
        page.context.set_offline(False)
    assert_no_page_errors(page)


def test_end_of_book_notice_is_visible(clean_page: Page, book_with_chapters: str):
    page = clean_page
    _, n_chapters = _chapter_no(book_with_chapters)
    last, _ = _chapter_no(book_with_chapters, n_chapters - 1)
    goto(page, f"/read/{book_with_chapters}?chapter={last}")
    page.wait_for_timeout(500)
    for _ in range(12):
        page.keyboard.press("ArrowRight")
        page.wait_for_timeout(120)
    toasts = page.locator(".reader .toast")
    expect(toasts).to_have_count(1)
    expect(toasts.first).to_be_visible()
    assert_no_page_errors(page)


# Position integrity. Every test stretches the chapters (several pages each)
# and turns with reduced motion, so a page lands the moment it is asked for.

SCROLL_PAGE = "() => { const v = document.querySelector('.reader-viewport'); return Math.round(v.scrollLeft / v.clientWidth); }"
FIRST_PARAGRAPH_ON_PAGE = """n => {
  const v = document.querySelector('.reader-viewport');
  const ps = [...v.querySelectorAll('.chapter-body p')];
  return 'p-' + (ps.findIndex(p => Math.floor(p.offsetLeft / v.clientWidth) >= n) + 1);
}"""


def _reader_context(new_context, **shim):
    context = new_context(viewport={"width": 900, "height": 800}, reduced_motion="reduce")
    install_fetch_shim(context, chapters=3, **shim)
    page = context.new_page()
    errors: list[str] = []
    page.on("pageerror", lambda e: errors.append(str(e)))
    return context, page, errors


def _open(page: Page, book_id: str, chapter: int = 1, settle_ms: int = 2200) -> None:
    page.goto(f"{WEB}/read/{book_id}?chapter={chapter}", wait_until="domcontentloaded")
    page.wait_for_selector(".chapter-body")
    page.wait_for_timeout(settle_ms)


def _turn(page: Page, key: str, times: int = 1) -> None:
    for _ in range(times):
        page.keyboard.press(key)
        page.wait_for_timeout(250)


def _guest_record(page: Page, book_id: str) -> dict | None:
    return next((r for r in page.evaluate(GUEST_PROGRESS) if r["book_id"] == book_id), None)


def _account_anchor(token: str, book_id: str) -> str | None:
    req = urllib.request.Request(f"{API}/progress/active", headers={"Authorization": f"Bearer {token}"})
    with urllib.request.urlopen(req) as r:
        active = json.load(r)["data"]
    return active["last_anchor_cfi"] if active and active["book_id"] == book_id else None


def _reopens_saved_page(page: Page, book_id: str, saved_anchor) -> None:
    _open(page, book_id)
    _turn(page, "ArrowRight", 2)
    expected = page.evaluate(FIRST_PARAGRAPH_ON_PAGE, 2)
    page.wait_for_timeout(1700)
    assert saved_anchor() == expected

    # The position answer is held back a second; the chapter wins the race.
    page.reload(wait_until="domcontentloaded")
    page.wait_for_selector(".chapter-body")
    page.wait_for_timeout(2600)
    assert page.evaluate(SCROLL_PAGE) == 2, "reopens on the saved page"
    page.wait_for_timeout(1500)
    assert saved_anchor() == expected, "the saved page is not overwritten by page one"


def test_guest_reopens_saved_page_when_position_is_slow(new_context, book_with_chapters: str):
    book_id = book_with_chapters
    context, page, errors = _reader_context(new_context, delay=[f"/books/{book_id}"])
    try:
        _reopens_saved_page(page, book_id, lambda: (_guest_record(page, book_id) or {}).get("last_anchor_cfi"))
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_account_reopens_saved_page_when_position_is_slow(new_context, book_with_chapters: str):
    book_id = book_with_chapters
    tokens = register_reader("position")
    context, page, errors = _reader_context(new_context, delay=[f"/books/{book_id}", "/progress/active"])
    context.add_init_script(
        f"localStorage.setItem('baca_access_token','{tokens['access_token']}');"
        f"localStorage.setItem('baca_refresh_token','{tokens['refresh_token']}');"
    )
    try:
        _reopens_saved_page(page, book_id, lambda: _account_anchor(tokens["access_token"], book_id))
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_stale_chapter_answer_is_dropped(new_context, book_with_chapters: str):
    book_id = book_with_chapters
    chapters = api_get(f"/books/{book_id}")["chapters"]
    context, page, errors = _reader_context(new_context, hold=[f"/books/{book_id}/chapters/2"])
    try:
        _open(page, book_id)
        page.keyboard.press("End")
        page.wait_for_timeout(250)
        _turn(page, "ArrowRight")
        _turn(page, "ArrowLeft")
        page.wait_for_timeout(600)
        page.evaluate("() => window.__release()")
        page.wait_for_timeout(1900)
        body = page.inner_text(".chapter-body")
        assert "Bab 1 paragraf" in body and "Bab 2" not in body
        assert "chapter=1" in page.url
        record = _guest_record(page, book_id)
        assert record and record["last_chapter_id"] == chapters[0]["id"]
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_leaving_inside_the_debounce_saves_the_new_page(new_context, book_with_chapters: str):
    book_id = book_with_chapters
    context, page, errors = _reader_context(new_context)
    try:
        _open(page, book_id)
        page.keyboard.press("ArrowRight")
        page.wait_for_timeout(60)
        expected = page.evaluate(FIRST_PARAGRAPH_ON_PAGE, 1)
        page.click(".reader-bar a.btn-icon")
        page.wait_for_timeout(1000)
        assert "/book/" in page.url
        assert (_guest_record(page, book_id) or {}).get("last_anchor_cfi") == expected
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_chapter_title_link_scrolls_to_the_list(clean_page: Page, book_with_chapters: str):
    page = clean_page
    goto(page, f"/book/{book_with_chapters}#daftar-bab")
    geometry = page.evaluate(
        """() => ({
            top: document.getElementById('daftar-bab').getBoundingClientRect().top,
            y: scrollY,
            end: document.documentElement.scrollHeight - innerHeight,
            height: innerHeight,
        })"""
    )
    assert geometry["y"] > 0, geometry
    # Under the header, or as far as a short page can scroll with it in view.
    assert 0 <= geometry["top"] < 240 or (geometry["y"] >= geometry["end"] - 1 and geometry["top"] < geometry["height"]), geometry
    assert_no_page_errors(page)


# Reader tools: chapter drawer, chapter-end card, quotes from a selection,
# and search inside the book.

PHONE = {"width": 390, "height": 844}
FLASHED_ON_SHOWN_PAGE = """() => {
  const v = document.querySelector('.reader-viewport');
  const hit = document.querySelector('.chapter-body p.search-flash');
  if (!hit) return null;
  return Math.floor(hit.offsetLeft / v.clientWidth) === Math.round(v.scrollLeft / v.clientWidth);
}"""
SELECT_PARAGRAPH = """n => {
  const p = document.querySelectorAll('.chapter-body p')[n];
  const range = document.createRange();
  range.selectNodeContents(p);
  const selection = getSelection();
  selection.removeAllRanges();
  selection.addRange(range);
  return p.textContent.trim().replace(/\\s+/g, ' ');
}"""
LOCAL_QUOTES = """() => new Promise(resolve => {
  const open = indexedDB.open('project_baca_db');
  open.onsuccess = () => {
    const all = open.result.transaction('local_quotes', 'readonly').objectStore('local_quotes').getAll();
    all.onsuccess = () => resolve(all.result.map(q => q.quote_text));
  };
})"""


def _context(new_context, viewport=PHONE, tokens: dict | None = None):
    context = new_context(viewport=viewport, reduced_motion="reduce")
    if tokens:
        context.add_init_script(
            f"localStorage.setItem('baca_access_token','{tokens['access_token']}');"
            f"localStorage.setItem('baca_refresh_token','{tokens['refresh_token']}');"
        )
    page = context.new_page()
    errors: list[str] = []
    page.on("pageerror", lambda e: errors.append(str(e)))
    return context, page, errors


def test_chapter_drawer_jumps_and_saves_the_position(new_context, book_with_chapters: str):
    book_id = book_with_chapters
    chapters = api_get(f"/books/{book_id}")["chapters"]
    context, page, errors = _context(new_context)
    try:
        _open(page, book_id, settle_ms=1500)
        page.click(".reader-title")
        drawer = page.locator(".chapter-drawer")
        expect(drawer.locator(".chapter-row.here")).to_have_count(1)
        drawer.locator(".chapter-row button").nth(2).click()
        page.wait_for_timeout(1500)
        assert page.query_selector(".chapter-drawer") is None, "the drawer closes"
        assert "chapter=3" in page.url
        expect(page.locator(".reader-title .label")).to_contain_text(chapters[2]["title"])
        page.wait_for_timeout(1500)
        record = _guest_record(page, book_id)
        assert record and record["last_chapter_id"] == chapters[2]["id"]
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_chapter_end_card_shows_once_per_chapter(new_context, book_with_chapters: str):
    context, page, errors = _context(new_context)
    try:
        _open(page, book_with_chapters, settle_ms=1500)
        page.keyboard.press("End")
        _turn(page, "ArrowRight")
        card = page.locator(".chapter-end")
        expect(card).to_be_visible()
        assert "chapter=1" in page.url, "the card comes before the next chapter"
        card.locator("button.btn-ghost").click()
        expect(page.locator(".sheet[role='dialog']")).to_be_visible()
        page.keyboard.press("Escape")
        _turn(page, "ArrowRight")
        page.wait_for_timeout(1200)
        assert "chapter=2" in page.url and page.query_selector(".chapter-end") is None
        _turn(page, "ArrowLeft")
        page.wait_for_timeout(1200)
        assert "chapter=1" in page.url
        _turn(page, "ArrowRight")
        page.wait_for_timeout(1200)
        assert "chapter=2" in page.url and page.query_selector(".chapter-end") is None, "once per chapter"
        assert errors == [], errors[:3]
    finally:
        context.close()


def _save_selection(page: Page, index: int = 1) -> str:
    text = page.evaluate(SELECT_PARAGRAPH, index)
    chip = page.locator(".quote-chip")
    expect(chip).to_be_visible()
    chip.click()
    page.wait_for_timeout(1200)
    expect(page.locator(".quote-chip")).to_have_count(0)
    return text


def test_selected_text_becomes_a_guest_quote(new_context, book_with_chapters: str):
    context, page, errors = _context(new_context)
    try:
        _open(page, book_with_chapters, settle_ms=1500)
        text = _save_selection(page)
        assert text in page.evaluate(LOCAL_QUOTES)
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_selected_text_becomes_an_account_quote(new_context, book_with_chapters: str):
    tokens = register_reader("quote")
    context, page, errors = _context(new_context, tokens=tokens)
    try:
        _open(page, book_with_chapters, settle_ms=1500)
        text = _save_selection(page)
        req = urllib.request.Request(f"{API}/quotes", headers={"Authorization": f"Bearer {tokens['access_token']}"})
        with urllib.request.urlopen(req) as r:
            saved = [q["quote_text"] for q in json.load(r)["data"]]
        assert text in saved
        assert errors == [], errors[:3]
    finally:
        context.close()


def _search(page: Page, query: str):
    page.click(".btn-search")
    page.fill("#book-search", query)
    page.keyboard.press("Enter")
    hits = page.locator(".search-hit")
    expect(hits.first).to_be_visible()
    return hits


def test_book_search_opens_the_page_that_holds_the_hit(new_context, book_with_chapters: str):
    context, page, errors = _context(new_context)
    try:
        _open(page, book_with_chapters, settle_ms=1500)
        hits = _search(page, "surat")
        assert hits.count() >= 3
        expect(hits.first.locator("mark")).to_have_text("surat")
        hits.nth(hits.count() - 1).click()
        page.wait_for_timeout(1600)
        assert page.query_selector(".sheet[role='dialog']") is None
        assert "chapter=1" not in page.url, "the last hit is in a later chapter"
        assert page.evaluate(FLASHED_ON_SHOWN_PAGE) is True, "the hit's paragraph is on the page shown"
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_book_search_works_offline_from_the_saved_copy(new_context, book_with_chapters: str):
    book_id = book_with_chapters
    context, page, errors = _context(new_context)
    try:
        page.goto(f"{WEB}/book/{book_id}", wait_until="networkidle")
        save = '[aria-label="Save for offline reading"]'
        if page.query_selector(save):
            page.click(save)
            page.wait_for_selector('[aria-label="Saved on this device"]', timeout=10000)
        _open(page, book_id, settle_ms=1500)
        context.set_offline(True)
        hits = _search(page, "surat")
        expect(page.locator(".sheet .form-hint")).to_have_text("Searched in the copy on your device.")
        hits.nth(1).click()
        page.wait_for_timeout(1600)
        assert page.evaluate(FLASHED_ON_SHOWN_PAGE) is True
        assert errors == [], errors[:3]
    finally:
        context.set_offline(False)
        context.close()
