"""Visual regression: golden screenshots per route.

First run generates goldens into `*-snapshots/` (committed to git).
Intentional UI change: `pytest --update-snapshots`, then REVIEW the image
diff before committing (never blind-accept). Chromium canonical only —
fonts/render differ per browser.
"""
from playwright.sync_api import Page
from visual.golden import assert_golden

from conftest import assert_no_page_errors, goto


def test_visual_home(clean_page: Page):
    page = clean_page
    goto(page, "/")
    page.wait_for_timeout(1000)
    assert_golden(page, "home.png")
    assert_no_page_errors(page)


def test_visual_overview(clean_page: Page, stable_book_id: str):
    page = clean_page
    goto(page, f"/book/{stable_book_id}")
    page.wait_for_timeout(1000)
    assert_golden(page, "overview.png")
    assert_no_page_errors(page)


def test_visual_reader(clean_page: Page, stable_book_id: str):
    import urllib.request
    import json

    page = clean_page
    with urllib.request.urlopen(
        f"http://localhost:8080/api/v1/books/{stable_book_id}"
    ) as r:
        detail = json.load(r)["data"]
    ch_no = detail["chapters"][0]["chapter_number"]
    goto(page, f"/read/{stable_book_id}?chapter={ch_no}")
    page.wait_for_timeout(1000)
    assert_golden(page, "reader.png")
    assert_no_page_errors(page)


def test_visual_profile_guest(clean_page: Page):
    page = clean_page
    goto(page, "/me")
    assert_golden(page, "profile-guest.png")
    assert_no_page_errors(page)


def test_visual_admin(clean_page: Page):
    page = clean_page
    goto(page, "/admin")
    page.wait_for_timeout(500)
    assert_golden(page, "admin.png")
    assert_no_page_errors(page)
