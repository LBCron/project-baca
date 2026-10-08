"""Axe-core accessibility scans per route.

axe-core 4.10.2 vendored locally (`axe.min.js`, no CDN/network dependency).
Tags: WCAG 2.0/2.1 A+AA. Known issues recorded as fingerprints, never blanket
excludes (Playwright a11y docs pattern).
"""
import json
from pathlib import Path

import pytest
from playwright.sync_api import Browser, Page

from conftest import WEB, assert_no_page_errors, goto

AXE_JS = Path(__file__).parent / "axe.min.js"
TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"]


def _scan(page: Page) -> list[dict]:
    page.add_script_tag(path=str(AXE_JS))
    result = page.evaluate(
        """async (tags) => {
            const r = await axe.run(document, { runOnly: { type: 'tag', values: tags } });
            return r.violations.map(v => ({
                rule: v.id,
                impact: v.impact,
                targets: v.nodes.map(n => n.target),
            }));
        }""",
        TAGS,
    )
    return result


def _assert_clean(page: Page, route: str) -> None:
    goto(page, route)
    page.wait_for_timeout(1000)
    violations = _scan(page)
    assert violations == [], (
        f"{route} a11y violations: {json.dumps(violations, indent=1)[:1500]}"
    )
    assert_no_page_errors(page)


def test_axe_home(clean_page: Page):
    _assert_clean(clean_page, "/")


def test_axe_overview(clean_page: Page, book_with_chapters: str):
    _assert_clean(clean_page, f"/book/{book_with_chapters}")


def test_axe_reader(clean_page: Page, book_with_chapters: str):
    import urllib.request

    with urllib.request.urlopen(
        f"http://localhost:8080/api/v1/books/{book_with_chapters}"
    ) as r:
        detail = json.load(r)["data"]
    ch_no = detail["chapters"][0]["chapter_number"]
    _assert_clean(clean_page, f"/read/{book_with_chapters}?chapter={ch_no}")


def test_axe_profile_guest(clean_page: Page):
    _assert_clean(clean_page, "/me")


def test_axe_admin(clean_page: Page):
    _assert_clean(clean_page, "/admin")


def test_axe_quote_modal(clean_page: Page, book_with_chapters: str):
    page = clean_page
    goto(page, f"/book/{book_with_chapters}")
    page.click("text=Find a quote")
    page.wait_for_timeout(800)
    violations = _scan(page)
    assert violations == [], (
        f"quote modal violations: {json.dumps(violations, indent=1)[:1500]}"
    )
    assert_no_page_errors(page)


@pytest.mark.parametrize("route", ["/", "/book/{book}", "/me"])
def test_axe_paper_surface(browser: Browser, book_with_chapters: str, route: str):
    """A first visit on a light system scheme opens the Paper shell."""
    context = browser.new_context(viewport={"width": 1280, "height": 800}, color_scheme="light")
    page = context.new_page()
    try:
        page.goto(f"{WEB}{route.format(book=book_with_chapters)}", wait_until="networkidle")
        page.wait_for_timeout(1500)
        assert page.get_attribute("html", "data-theme") == "paper"
        violations = _scan(page)
        assert violations == [], (
            f"paper {route} a11y violations: {json.dumps(violations, indent=1)[:1500]}"
        )
    finally:
        context.close()
