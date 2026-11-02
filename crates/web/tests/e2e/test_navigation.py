"""Navigation contract: in-page anchors land on their section after the
catalog loads, exactly one item carries the active marker, the search tab
really focuses the field, sheets keep keyboard focus, and the shell follows
the system scheme until the reader picks one."""
from playwright.sync_api import Browser, Page, expect

from conftest import WEB, assert_no_page_errors, goto

HEADER_CLEARANCE_PX = 140


def _section_top(page: Page, element_id: str):
    return page.evaluate(
        "id => { const el = document.getElementById(id); return el ? el.getBoundingClientRect().top : null; }",
        element_id,
    )


def test_how_it_works_scrolls_from_the_shelf(clean_page: Page):
    page = clean_page
    page.set_viewport_size({"width": 1280, "height": 800})
    goto(page, "/me")
    page.get_by_role("link", name="How it works").click()
    page.wait_for_timeout(2500)
    top = _section_top(page, "cara-kerja")
    # A short tail section cannot reach the header line: the page stops at
    # its bottom edge instead, which is the same intent for the reader.
    at_bottom = page.evaluate(
        "() => window.scrollY + window.innerHeight >= document.documentElement.scrollHeight - 2"
    )
    assert top is not None and top >= -2, top
    assert top <= HEADER_CLEARANCE_PX or at_bottom, (top, at_bottom)
    expect(page.get_by_role("link", name="How it works")).to_have_attribute("aria-current", "page")
    expect(page.locator(".nav-link[aria-current='page']")).to_have_count(1)
    assert_no_page_errors(page)


def test_header_marks_catalog_and_shelf(clean_page: Page):
    page = clean_page
    page.set_viewport_size({"width": 1280, "height": 800})
    goto(page, "/")
    expect(page.get_by_role("link", name="Catalog")).to_have_attribute("aria-current", "page")
    goto(page, "/me")
    expect(page.get_by_role("link", name="My shelf")).to_have_attribute("aria-current", "page")
    expect(page.locator(".nav-link[aria-current='page']")).to_have_count(1)
    assert_no_page_errors(page)


def test_tab_bar_marks_search_and_profile(clean_page: Page):
    page = clean_page
    page.set_viewport_size({"width": 390, "height": 844})
    goto(page, "/?search=1")
    tabs = page.locator(".tab")
    expect(tabs.nth(1)).to_have_attribute("aria-current", "page")
    expect(page.locator(".tab[aria-current='page']")).to_have_count(1)
    expect(page.locator("#catalog-search")).to_be_focused()
    goto(page, "/me#akun")
    expect(tabs.nth(3)).to_have_attribute("aria-current", "page")
    expect(page.locator(".tab[aria-current='page']")).to_have_count(1)
    assert_no_page_errors(page)


def test_cover_art_never_shows_a_broken_image(clean_page: Page):
    page = clean_page
    goto(page, "/")
    assert page.query_selector(".cover img") is None
    styles = page.evaluate(
        "() => [...document.querySelectorAll('.cover .cover-art')].map(el => el.style.backgroundImage)"
    )
    for style in styles:
        assert style.startswith("url("), style
    assert_no_page_errors(page)


def test_tab_stays_inside_an_open_sheet(clean_page: Page):
    page = clean_page
    page.set_viewport_size({"width": 1280, "height": 800})
    goto(page, "/")
    page.click(".header-signin")
    page.wait_for_selector(".sheet[role='dialog']")
    page.wait_for_timeout(300)
    inside = "() => !!document.activeElement.closest('.sheet[role=\\'dialog\\']')"
    for key in ["Tab"] * 12 + ["Shift+Tab"] * 12:
        page.keyboard.press(key)
        assert page.evaluate(inside), f"focus left the sheet after {key}"
    assert_no_page_errors(page)


def test_focused_search_field_clears_the_sticky_header(clean_page: Page):
    page = clean_page
    page.set_viewport_size({"width": 390, "height": 844})
    goto(page, "/?search=1")
    geometry = page.evaluate(
        """() => ({
            field: document.getElementById('catalog-search').getBoundingClientRect().top,
            header: document.querySelector('.site-header').getBoundingClientRect().bottom,
            focused: document.activeElement.id,
        })"""
    )
    assert geometry["focused"] == "catalog-search", geometry
    assert geometry["field"] >= geometry["header"], geometry
    assert_no_page_errors(page)


def _theme(page: Page) -> tuple[str | None, str]:
    return tuple(page.evaluate(
        """() => [document.documentElement.getAttribute('data-theme'),
                  document.querySelector('.site-header .btn-icon[aria-label*="theme"]').getAttribute('aria-label')]"""
    ))


def test_theme_toggle_names_its_target_and_follows_the_system(browser: Browser):
    context = browser.new_context(color_scheme="dark", viewport={"width": 1280, "height": 800})
    page = context.new_page()
    try:
        page.goto(f"{WEB}/", wait_until="networkidle")
        assert _theme(page) == (None, "Switch to the Paper theme")
        page.emulate_media(color_scheme="light")
        page.wait_for_timeout(300)
        assert _theme(page) == ("paper", "Switch to the Espresso theme"), "follows the system until chosen"
        page.click('.site-header .btn-icon[aria-label*="theme"]')
        page.wait_for_timeout(200)
        assert _theme(page) == (None, "Switch to the Paper theme")
        page.emulate_media(color_scheme="light")
        page.emulate_media(color_scheme="dark")
        page.emulate_media(color_scheme="light")
        page.wait_for_timeout(300)
        assert _theme(page) == (None, "Switch to the Paper theme"), "a chosen theme stays"
    finally:
        context.close()
