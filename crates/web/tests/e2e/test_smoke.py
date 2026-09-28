"""Smoke: catalog renders, hero art, brand, zero page errors."""
from playwright.sync_api import Page, expect

from conftest import assert_no_page_errors, goto


def test_home_renders_catalog(clean_page: Page):
    page = clean_page
    goto(page, "/")
    page.wait_for_timeout(500)
    cards = page.query_selector_all(".book-card")
    assert len(cards) > 0, "catalog must render at least one card"
    assert_no_page_errors(page)


def test_home_brand_and_hero(clean_page: Page):
    page = clean_page
    goto(page, "/")
    page.wait_for_timeout(500)
    assert page.query_selector(".hero-art img") is not None
    assert page.query_selector(".search-bar") is not None
    brand = page.inner_text(".brand-title")
    assert "Rotaria" in brand, brand.strip()[:20]
    assert page.query_selector(".brand-mark") is not None
    assert_no_page_errors(page)
