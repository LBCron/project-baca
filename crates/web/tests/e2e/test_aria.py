"""ARIA snapshots: structural accessibility-tree regression.

Roles and hierarchy are asserted, copy is dynamic (i18n) so only the
English defaults appear here. Update the template on intentional changes.
"""
from playwright.sync_api import Page, expect

from conftest import assert_no_page_errors, goto


def test_aria_home_structure(clean_page: Page):
    page = clean_page
    goto(page, "/")
    expect(page.locator("header.site-header")).to_match_aria_snapshot("""
      - banner:
        - link "Rotaria":
          - /url: /
        - navigation "Main":
          - link "Catalog":
            - /url: /
          - link "My shelf":
            - /url: /me
          - link "How it works":
            - /url: /#cara-kerja
        - group "Language":
          - button "ID"
          - button "EN"
        - button "Switch to the Paper theme"
        - button "Sign In"
    """)
    assert_no_page_errors(page)


def test_aria_nav_landmarks(clean_page: Page):
    page = clean_page
    goto(page, "/")
    snapshot = page.aria_snapshot()
    assert "banner" in snapshot and "navigation" in snapshot, snapshot[:300]
    assert "main" in snapshot, snapshot[:300]
    assert_no_page_errors(page)


def _title(page: Page, path: str, wait_for: str) -> str:
    goto(page, path)
    page.wait_for_selector(wait_for)
    page.wait_for_timeout(600)
    return page.title()


def test_every_route_has_its_own_title_in_both_languages(clean_page: Page, book_with_chapters: str):
    """Tabs, history, and screen readers tell routes apart (WCAG 2.4.2)."""
    page = clean_page
    for lang in ["EN", "ID"]:
        page.add_init_script(f"localStorage.setItem('rotaria_lang', '{lang}')")
        titles = [
            _title(page, "/", ".book-grid"),
            _title(page, f"/book/{book_with_chapters}", ".book-overview h1"),
            _title(page, f"/read/{book_with_chapters}?chapter=1", ".chapter-body"),
            _title(page, "/me", "main"),
            _title(page, "/admin", "main"),
            _title(page, "/no-such-page", "main"),
        ]
        assert len(set(titles)) == len(titles), titles
        assert all("Rotaria" in t for t in titles), titles
    assert_no_page_errors(page)


def test_book_text_carries_the_books_language(clean_page: Page, book_with_chapters: str):
    """An Indonesian book read in the English interface is announced and
    hyphenated as Indonesian (WCAG 3.1.2)."""
    page = clean_page
    page.add_init_script("localStorage.setItem('rotaria_lang', 'EN')")
    goto(page, f"/read/{book_with_chapters}?chapter=1")
    expect(page.locator(".chapter-body")).to_have_attribute("lang", "id")
    name = page.get_attribute(".reader-title", "aria-label") or ""
    visible = " ".join(page.inner_text(".reader-title").split())
    assert name.lower().startswith(visible.lower()), (name, visible)
    goto(page, f"/book/{book_with_chapters}")
    expect(page.locator(".book-overview .prose").first).to_have_attribute("lang", "id")
    assert_no_page_errors(page)


def test_type_panel_takes_focus_and_gives_it_back(clean_page: Page, book_with_chapters: str):
    page = clean_page
    goto(page, f"/read/{book_with_chapters}?chapter=1")
    page.wait_for_selector(".chapter-body")
    page.focus(".btn-type")
    page.keyboard.press("Enter")
    page.wait_for_timeout(300)
    assert page.evaluate("() => !!document.activeElement.closest('.type-sheet')"), "focus moves into the panel"
    page.keyboard.press("Escape")
    page.wait_for_timeout(300)
    assert page.evaluate("() => document.activeElement.classList.contains('btn-type')"), "focus returns to Aa"
    assert_no_page_errors(page)


def test_a_missing_page_keeps_the_shell(clean_page: Page):
    page = clean_page
    goto(page, "/no-such-page")
    expect(page.locator("header.site-header")).to_be_visible()
    expect(page.locator("main")).to_contain_text("not been written")
    expect(page.get_by_role("navigation").first).to_be_visible()
    expect(page.locator("main a[href='/']")).to_be_visible()
    assert_no_page_errors(page)
