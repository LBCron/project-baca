"""Catalog flows: hero plate, i18n toggle + persist, search, overview, sheets."""
from playwright.sync_api import Page, expect

from conftest import assert_no_page_errors, goto


def test_hero_plate_is_static(clean_page: Page):
    page = clean_page
    goto(page, "/")
    src = page.get_attribute(".hero-slide", "src")
    assert src and src.endswith("library-bookshelf-ladder.webp"), src
    assert page.query_selector(".hero-dot") is None, "no carousel controls"
    assert page.query_selector(".plate-caption") is not None
    assert_no_page_errors(page)


def test_i18n_toggle_and_persist(clean_page: Page):
    page = clean_page
    goto(page, "/")
    page.click('.lang-opt:has-text("ID")')
    page.wait_for_timeout(800)
    heading_id = page.inner_text(".hero-heading")
    assert "Buku" in heading_id, heading_id[:50]
    assert page.evaluate("localStorage.getItem('rotaria_lang')") == "ID"
    page.click('.lang-opt:has-text("EN")')
    page.wait_for_timeout(800)
    heading_en = page.inner_text(".hero-heading")
    assert "Buku" not in heading_en, heading_en[:50]
    assert page.evaluate("localStorage.getItem('rotaria_lang')") == "EN"
    assert_no_page_errors(page)


def test_search_returns_cards(clean_page: Page):
    page = clean_page
    goto(page, "/")
    page.fill(".hero-copy .search-bar input", "a")
    page.wait_for_timeout(1500)
    searched = page.query_selector_all(".book-card")
    assert len(searched) > 0, "search must return cards"
    assert_no_page_errors(page)


def test_overview_and_sheets(clean_page: Page, book_with_chapters: str):
    page = clean_page
    goto(page, f"/book/{book_with_chapters}")
    page.wait_for_timeout(500)
    assert page.query_selector(".book-overview") is not None
    assert len(page.query_selector_all(".chapter-row")) > 0
    assert page.query_selector("text=Find a quote") is not None
    assert page.query_selector("text=Insight cards") is not None

    page.click("text=Find a quote")
    page.wait_for_timeout(500)
    page.fill(".sheet .search-bar input", "love and sacrifice")
    page.click(".sheet .search-bar .btn-primary")
    page.wait_for_timeout(4000)
    assert page.query_selector(".quote-results") is not None
    page.keyboard.press("Escape")
    assert_no_page_errors(page)
