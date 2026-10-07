"""Reader flows: body render, type sheet, recap chip, offline save + read."""
from playwright.sync_api import Page, expect

from conftest import WEB, api_get, assert_no_page_errors, goto


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
    assert "Page 1 of" in page.inner_text(".reader-status")
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
    # Visit the reader ONLINE first: sw.js caches navigations per-URL, so the
    # offline fallback only works for URLs already in SHELL_CACHE.
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
