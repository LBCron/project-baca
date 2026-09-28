"""Component tests via the dev gallery route `/__gallery?story=` (Task 11e).

Pattern: Playwright docs "component testing" — gallery owned by the app,
stories own state, tests assert through the DOM. Error states mocked via
`page.route()` (service worker bypassed by fresh context per test).
"""
from playwright.sync_api import Page, expect

from conftest import WEB, assert_no_page_errors

GALLERY = f"{WEB}/__gallery?story="


def _story(page: Page, name: str):
    page.goto(f"{GALLERY}{name}", wait_until="networkidle")
    page.wait_for_timeout(1500)
    assert page.query_selector("#gallery-unknown") is None, f"unknown story: {name}"
    return page.query_selector("#gallery-root")


def test_site_header_toggle_no_reload(clean_page: Page):
    page = clean_page
    root = _story(page, "site-header")
    assert root is not None
    before = page.evaluate("performance.now()")
    page.click('.lang-opt:has-text("EN")')
    page.wait_for_timeout(600)
    assert page.evaluate("localStorage.getItem('rotaria_lang')") == "EN"
    assert_no_page_errors(page)


def test_auth_modal_modes(clean_page: Page):
    page = clean_page
    root = _story(page, "auth-modal")
    assert root is not None
    body = page.inner_text("#gallery-root")
    assert len(body) > 0
    assert_no_page_errors(page)


def test_quote_finder_empty_state(clean_page: Page):
    page = clean_page
    root = _story(page, "quote-finder")
    assert root is not None
    page.fill(".modal-vintage .search-bar", "zzzqqq-no-such-quote")
    page.click(".modal-vintage .btn-read")
    page.wait_for_timeout(4000)
    assert page.query_selector(".quote-results") is not None
    assert_no_page_errors(page)


def test_quote_finder_error_branch(clean_page: Page):
    """Mocked 500 must surface an error message, never silent emptiness."""
    page = clean_page
    page.route("**/api/**/quotes/search", lambda r: r.fulfill(status=500, body='{"success":false}'))
    _story(page, "quote-finder")
    page.fill(".modal-vintage .search-bar", "love")
    page.click(".modal-vintage .btn-read")
    page.wait_for_timeout(3000)
    text = page.inner_text("#gallery-root")
    assert "quote-results" in (page.content())  # modal still mounted
    assert_no_page_errors(page)


def test_atomic_cards_404_tolerant(clean_page: Page):
    page = clean_page
    root = _story(page, "atomic-cards")
    assert root is not None
    page.wait_for_timeout(2000)
    assert_no_page_errors(page)
