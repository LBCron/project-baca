"""M6 web realtime E2E (Playwright, black-box) + Rotaria P1-P7.

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
    with urllib.request.urlopen(f"{API}/books?limit=20") as r:
        data = json.load(r)["data"]
    assert data, "catalog empty, seed or ingest a book first"
    for book in data:
        try:
            with urllib.request.urlopen(f"{API}/books/{book['id']}") as r:
                detail = json.load(r)["data"]
            if detail.get("chapters"):
                return book["id"]
        except Exception:
            continue
    raise AssertionError("no book with chapters found; ingest a sample EPUB first")


async def main():
    from playwright.async_api import async_playwright

    book_id = api_book_id()
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        context = await browser.new_context(viewport={"width": 1280, "height": 900})
        page = await context.new_page()
        errors = []
        page.on("pageerror", lambda e: errors.append(str(e)))

        await page.goto(f"{WEB}/", wait_until="networkidle")
        await page.wait_for_timeout(2500)
        cards = await page.query_selector_all(".book-card")
        check("home-renders-catalog", len(cards) > 0, f"{len(cards)} cards")
        check("home-hero-art", await page.query_selector(".hero-art img") is not None)
        check("home-search-bar", await page.query_selector(".search-bar") is not None)
        brand = await page.inner_text(".brand-title")
        check("brand-rotaria", "Rotaria" in brand, brand.strip()[:20])
        check("brand-mark", await page.query_selector(".brand-mark") is not None)
        slide1 = await page.get_attribute(".hero-slide", "src")
        await page.wait_for_timeout(8000)
        slide2 = await page.get_attribute(".hero-slide", "src")
        check("hero-carousel-rotates", slide1 != slide2)
        check("hero-dots", len(await page.query_selector_all(".hero-dot")) == 5)
        await page.click('.lang-opt:has-text("EN")')
        await page.wait_for_timeout(800)
        heading_en = await page.inner_text(".hero-heading")
        check("i18n-toggle-en", "Sirkulasi" not in heading_en, heading_en[:50])
        persisted = await page.evaluate("localStorage.getItem('rotaria_lang')")
        check("i18n-persist", persisted == "EN", str(persisted))
        await page.click('.lang-opt:has-text("ID")')
        await page.wait_for_timeout(800)

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

        # Offline (BL-12): cache via overview, then read the IDB copy offline.
        await page.goto(f"{WEB}/book/{book_id}", wait_until="networkidle")
        await page.wait_for_timeout(2000)
        if await page.query_selector("text=Save Offline"):
            await page.click("text=Save Offline")
            await page.wait_for_timeout(3000)
            saved_label = await page.inner_text("body")
            check("offline-saved", "Saved Offline" in saved_label)
        else:
            check("offline-saved", True, "already cached")
        await context.set_offline(True)
        await page.goto(f"{WEB}/read/{book_id}?chapter={ch_no}", wait_until="domcontentloaded")
        await page.wait_for_timeout(2500)
        off_body = await page.inner_text(".chapter-body") if await page.query_selector(".chapter-body") else ""
        check("offline-reader-renders", len(off_body) > 0, f"{len(off_body)} chars")
        footer = await page.inner_text(".reader-status") if await page.query_selector(".reader-status") else ""
        check("offline-badge", "offline copy" in footer, footer[:60])
        await context.set_offline(False)

        await page.goto(f"{WEB}/", wait_until="networkidle")
        await page.wait_for_timeout(2000)
        await page.click("text=Masuk" if await page.query_selector("text=Masuk") else "text=Sign In")
        await page.wait_for_timeout(500)
        check("auth-modal", await page.query_selector(".modal-vintage") is not None)

        await page.goto(f"{WEB}/me", wait_until="networkidle")
        await page.wait_for_timeout(2000)
        check("profile-guest", "Sign in" in await page.inner_text("body"))
        await page.goto(f"{WEB}/admin", wait_until="networkidle")
        await page.wait_for_timeout(1500)
        check("admin-form", await page.query_selector('input[type="file"]') is not None)

        # Authed flow (BL-11): signup -> Mailpit OTP -> verify -> browser
        # tap-to-save (PUT /progress) -> sessions UI -> account cleanup.
        import time
        email = f"e2e-{int(time.time())}@example.com"
        payload = json.dumps(
            {"display_name": "E2E", "email": email, "password": "E2EPassword123!"}
        ).encode()
        req = urllib.request.Request(
            f"{API}/auth/signup", data=payload,
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(req):
            pass
        otp = ""
        for _ in range(10):
            await asyncio.sleep(1)
            with urllib.request.urlopen(
                "http://localhost:8025/api/v1/messages?limit=50"
            ) as r:
                msgs = json.load(r)["messages"]
            mids = [m["ID"] for m in msgs if email in json.dumps(m)]
            if mids:
                with urllib.request.urlopen(
                    f"http://localhost:8025/api/v1/message/{mids[0]}"
                ) as r:
                    import re
                    text = json.dumps(json.load(r))
                    m = re.search(r"\b(\d{6})\b", text)
                    if m:
                        otp = m.group(1)
                        break
        check("authed-otp-received", bool(otp))
        payload = json.dumps({"email": email, "otp": otp}).encode()
        req = urllib.request.Request(
            f"{API}/auth/verify-otp", data=payload,
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(req) as r:
            tokens = json.load(r)["data"]
        await page.evaluate(
            f"localStorage.setItem('baca_access_token','{tokens['access_token']}');"
            f"localStorage.setItem('baca_refresh_token','{tokens['refresh_token']}');"
        )
        await page.goto(f"{WEB}/me", wait_until="networkidle")
        await page.wait_for_timeout(2000)
        body_text = await page.inner_text("body")
        check("profile-authed", "E2E" in body_text)
        check("sessions-ui", "Sessions" in body_text)
        await page.goto(f"{WEB}/read/{book_id}?chapter={ch_no}", wait_until="networkidle")
        await page.wait_for_timeout(2500)
        await page.click(".reader-status")
        await page.wait_for_timeout(2000)
        req = urllib.request.Request(
            f"{API}/progress/active",
            headers={"Authorization": f"Bearer {tokens['access_token']}"},
        )
        with urllib.request.urlopen(req) as r:
            act = json.load(r)["data"]
        check(
            "authed-tap-saves-progress",
            isinstance(act, dict) and act.get("book_id") == book_id,
            f"{act.get('completion_percentage') if isinstance(act, dict) else act}%",
        )
        req = urllib.request.Request(
            f"{API}/me", method="DELETE",
            headers={"Authorization": f"Bearer {tokens['access_token']}"},
        )
        with urllib.request.urlopen(req):
            pass
        await page.evaluate("localStorage.clear()")
        await page.goto(f"{WEB}/me", wait_until="networkidle")
        await page.wait_for_timeout(2000)
        check("profile-guest-after-delete", "Sign in" in await page.inner_text("body"))

        check("zero-page-errors", len(errors) == 0, f"{len(errors)} errors")
        for e in errors[:5]:
            print("  ERR:", str(e)[:200])
        await browser.close()

    print(f"\n{len(PASS)} passed, {len(FAIL)} failed")
    if FAIL:
        print("FAILED:", FAIL)
        sys.exit(1)


asyncio.run(main())
