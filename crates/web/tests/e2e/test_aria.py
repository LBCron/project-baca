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
        - button "Switch theme"
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
