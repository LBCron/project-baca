"""ARIA snapshots: structural accessibility-tree regression (Task 11f).

Cheap + stable: roles/hierarchy asserted, dynamic content via regex/partial.
If 10g changes semantics intentionally, update the inline template here.
"""
from playwright.sync_api import Page, expect

from conftest import assert_no_page_errors, goto


def test_aria_home_structure(clean_page: Page):
    page = clean_page
    goto(page, "/")
    # Real header tree (banner with sign-in + language group); heading text
    # is dynamic (i18n), so assert roles/structure, not copy.
    expect(page.locator("header")).to_match_aria_snapshot("""
      - banner:
        - link /Rotaria/:
          - /url: /
        - navigation:
          - link "Catalog":
            - /url: /
          - button "Sign In"
          - group "Language":
            - button "ID"
            - button "EN"
    """)
    assert_no_page_errors(page)


def test_aria_nav_landmarks(clean_page: Page):
    page = clean_page
    goto(page, "/")
    snapshot = page.aria_snapshot()
    assert "banner" in snapshot or "navigation" in snapshot, snapshot[:300]
    assert_no_page_errors(page)
