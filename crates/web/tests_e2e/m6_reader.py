"""M6 web realtime E2E (Playwright, black-box).

Requires: `make db-up`, Axum on :8080 (`make dev-server`),
Trunk on :3000 (`make dev-web`).

Run: ~/.venvs/webapp-testing/bin/python crates/web/tests_e2e/m6_reader.py
Asserts structural flows, never debris content (titles/counts vary).
Exit non-zero on any failure.
"""
import asyncio
import sys
import urllib.request
import json

WEB = "http://127.0.0.1:3000"
API = "http://localhost:8080/api/v1"

PASS = []
FAIL = []


def check(name, cond, detail=""):
    (PASS if cond else FAIL).append(name)
    print(f"[{'PASS' if cond else 'FAIL'}] {name} {detail}")


def api_book_id():
    with urllib.request.urlopen(f"{API}/books?limit=1") as r:
        data = json.load(r)["data"]
    assert data, "catalog empty, seed or ingest a book first"
    return data[0]["id"]


async def main():
    from playwright.async_api import async_playwright

    book_id = api_book_id()
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page(viewport={"width": 1280, "height": 900})
        errors = []
        page.on("pageerror", lambda e: errors.append(str(e)))

        await page.goto(f"{WEB}/", wait_until="networkidle")
        await page.wait_for_timeout(2500)
        cards = await page.query_selector_all(".book-card")
        check("home-renders-catalog", len(cards) > 0, f"{len(cards)} cards")
        check("home-hero-art", await page.query_selector(".hero-art img") is not None)
        check("home-search-bar", await page.query_selector(".search-bar") is not None)

        await page.fill(".hero-copy .search-bar", "a")
        await page.wait_for_timeout(1200)
        searched = await page.query_selector_all(".book-card")
        check("search-returns-cards", len(searched) > 0, f"{len(searched)} cards")
        await page.goto(f"{WEB}/", wait_until="networkidle")
        await page.wait_for_timeout(2000)

        await page.goto(f"{WEB}/book/{book_id}", wait_until="networkidle")
        await page.wait_for_timeout(2500)
        check("overview-renders", await page.query_selector(".book-overview") is not None)
        chapters = await page.query_selector_all(".chapter-row")
        check("overview-chapters", len(chapters) > 0, f"{len(chapters)} rows")
        check("qf-button", await page.query_selector("text=Quote Finder") is not None)
        check("atomic-button", await page.query_selector("text=Atomic Cards") is not None)

        await page.click("text=Quote Finder")
        await page.wait_for_timeout(500)
        await page.fill(".modal-vintage .search-bar", "love and sacrifice")
        await page.click(".modal-vintage .btn-read")
        await page.wait_for_timeout(4000)
        check("qf-modal-no-crash", await page.query_selector(".quote-results") is not None)
        await page.keyboard.press("Escape")
        await page.goto(f"{WEB}/book/{book_id}", wait_until="networkidle")
        await page.wait_for_timeout(2000)

        with urllib.request.urlopen(f"{API}/books/{book_id}") as r:
            detail = json.load(r)["data"]
        ch_no = detail["chapters"][0]["chapter_number"] if detail["chapters"] else 1
        n_chapters = len(detail["chapters"])
        await page.goto(f"{WEB}/read/{book_id}?chapter={ch_no}", wait_until="networkidle")
        await page.wait_for_timeout(2500)
        body = await page.inner_text(".chapter-body") if await page.query_selector(".chapter-body") else ""
        check("reader-renders-body", len(body) > 0, f"{len(body)} chars")
        await page.click(".reader-tools .lang-switch")
        await page.wait_for_timeout(400)
        check("type-drawer", await page.query_selector(".type-drawer") is not None)
        if n_chapters > 1:
            ch2 = detail["chapters"][1]["chapter_number"]
            await page.goto(f"{WEB}/read/{book_id}?chapter={ch2}", wait_until="networkidle")
            await page.wait_for_timeout(2000)
            check("recap-button-ch2", await page.query_selector("text=Recap") is not None)
        else:
            check("recap-button-ch2", True, "skipped, single chapter")

        await page.goto(f"{WEB}/", wait_until="networkidle")
        await page.wait_for_timeout(2000)
        await page.click("text=Sign In")
        await page.wait_for_timeout(500)
        check("auth-modal", await page.query_selector(".modal-vintage") is not None)

        check("zero-page-errors", len(errors) == 0, f"{len(errors)} errors")
        for e in errors[:5]:
            print("  ERR:", str(e)[:200])
        await browser.close()

    print(f"\n{len(PASS)} passed, {len(FAIL)} failed")
    if FAIL:
        print("FAILED:", FAIL)
        sys.exit(1)


asyncio.run(main())
