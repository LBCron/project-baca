"""Catalog flows: carousel, i18n toggle+ persist, search, overview, drawers."""
from playwright.sync_api import Page, expect

from conftest import assert_no_page_errors, goto


def test_hero_carousel_rotates(clean_page: Page):
    page = clean_page
    goto(page, "/")
    slide1 = page.get_attribute(".hero-slide", "src")
    page.wait_for_timeout(8000)
    slide2 = page.get_attribute(".hero-slide", "src")
    assert slide1 != slide2, "carousel must rotate within ~7s"
    assert len(page.query_selector_all(".hero-dot")) == 5
    assert_no_page_errors(page)


def test_i18n_toggle_and_persist(clean_page: Page):
    page = clean_page
    goto(page, "/")
    page.click('.lang-opt:has-text("EN")')
    page.wait_for_timeout(800)
    heading_en = page.inner_text(".hero-heading")
    assert "Sirkulasi" not in heading_en, heading_en[:50]
    assert page.evaluate("localStorage.getItem('rotaria_lang')") == "EN"
    page.click('.lang-opt:has-text("ID")')
    page.wait_for_timeout(800)
    assert_no_page_errors(page)


def test_search_returns_cards(clean_page: Page):
    page = clean_page
    goto(page, "/")
    page.fill(".hero-copy .search-bar", "a")
    page.wait_for_timeout(1200)
    searched = page.query_selector_all(".book-card")
    assert len(searched) > 0, "search must return cards"
    assert_no_page_errors(page)


def test_overview_and_drawers(clean_page: Page, book_with_chapters: str):
    page = clean_page
    goto(page, f"/book/{book_with_chapters}")
    page.wait_for_timeout(500)
    assert page.query_selector(".book-overview") is not None
    assert len(page.query_selector_all(".chapter-row")) > 0
    assert page.query_selector("text=Quote Finder") is not None
    assert page.query_selector("text=Atomic Cards") is not None

    page.click("text=Quote Finder")
    page.wait_for_timeout(500)
    page.fill(".modal-vintage .search-bar", "love and sacrifice")
    page.click(".modal-vintage .btn-read")
    page.wait_for_timeout(4000)
    assert page.query_selector(".quote-results") is not None
    page.keyboard.press("Escape")
    assert_no_page_errors(page)
