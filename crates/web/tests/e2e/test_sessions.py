"""Authed flow: OTP via Mailpit, automatic progress save, shelf, cleanup."""
import json
import re
import time
import urllib.request

from playwright.sync_api import Page, expect

from conftest import API, MAILPIT, api_get, assert_no_page_errors, goto


def _post(path: str, payload: dict, token: str | None = None):
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(
        f"{API}{path}", data=json.dumps(payload).encode(), headers=headers
    )
    with urllib.request.urlopen(req) as r:
        return json.load(r)["data"]


def _mailpit_otp(email: str, timeout_s: int = 15) -> str:
    deadline = time.time() + timeout_s
    while time.time() < deadline:
        time.sleep(1)
        with urllib.request.urlopen(f"{MAILPIT}/api/v1/messages?limit=50") as r:
            msgs = json.load(r)["messages"]
        for m in msgs:
            if email in json.dumps(m):
                with urllib.request.urlopen(f"{MAILPIT}/api/v1/message/{m['ID']}") as r:
                    found = re.search(r"\b(\d{6})\b", json.dumps(json.load(r)))
                    if found:
                        return found.group(1)
    raise AssertionError("OTP email never arrived in Mailpit")


def test_authed_autosave_shelf_cleanup(clean_page: Page, book_with_chapters: str):
    page = clean_page
    book_id = book_with_chapters
    detail = api_get(f"/books/{book_id}")
    ch_no = detail["chapters"][0]["chapter_number"]

    goto(page, "/")
    page.click("text=Sign In")
    page.wait_for_timeout(500)
    assert page.query_selector(".sheet[role='dialog']") is not None
    page.keyboard.press("Escape")

    email = f"e2e-{int(time.time())}@example.com"
    _post("/auth/signup", {"display_name": "E2E", "email": email, "password": "E2EPassword123!"})
    otp = _mailpit_otp(email)
    tokens = _post("/auth/verify-otp", {"email": email, "otp": otp})
    assert tokens.get("access_token"), "OTP verify must issue tokens"

    page.evaluate(
        f"localStorage.setItem('baca_access_token','{tokens['access_token']}');"
        f"localStorage.setItem('baca_refresh_token','{tokens['refresh_token']}');"
    )
    goto(page, "/me")
    body_text = page.inner_text("body")
    assert "E2E" in body_text
    assert "Sessions" in body_text

    # Opening a chapter saves the position by itself (debounced), no tap needed.
    goto(page, f"/read/{book_id}?chapter={ch_no}")
    page.wait_for_timeout(3000)
    req = urllib.request.Request(
        f"{API}/progress/active",
        headers={"Authorization": f"Bearer {tokens['access_token']}"},
    )
    with urllib.request.urlopen(req) as r:
        act = json.load(r)["data"]
    assert isinstance(act, dict) and act.get("book_id") == book_id
    assert act.get("completion_percentage", 0) > 0

    req = urllib.request.Request(
        f"{API}/me", method="DELETE",
        headers={"Authorization": f"Bearer {tokens['access_token']}"},
    )
    with urllib.request.urlopen(req):
        pass
    page.evaluate("localStorage.clear()")
    goto(page, "/me")
    assert "Sign in" in page.inner_text("body")
    assert_no_page_errors(page)


def test_guest_profile_and_admin_guard(clean_page: Page):
    page = clean_page
    goto(page, "/me")
    assert "Sign in" in page.inner_text("body")
    goto(page, "/admin")
    page.wait_for_timeout(500)
    assert page.query_selector('input[type="file"]') is None, "guests never see the upload form"
    assert "curators" in page.inner_text("body")
    assert_no_page_errors(page)


LOCAL_QUOTE = "Kata-kata lebih setia daripada manusia."


def _put_local_quote(page: Page, book_id: str, chapter_id: str) -> None:
    page.evaluate(
        """([bookId, chapterId, text]) => new Promise((resolve, reject) => {
            const open = indexedDB.open('project_baca_db');
            open.onerror = () => reject(open.error);
            open.onsuccess = () => {
                const tx = open.result.transaction('local_quotes', 'readwrite');
                tx.objectStore('local_quotes').put({
                    key: bookId + ':' + chapterId + ':probe',
                    book_id: bookId,
                    chapter_id: chapterId,
                    quote_text: text,
                    saved_at: new Date().toISOString(),
                });
                tx.oncomplete = () => resolve(true);
                tx.onerror = () => reject(tx.error);
            };
        })""",
        [book_id, chapter_id, LOCAL_QUOTE],
    )


def _count_local_quotes(page: Page) -> int:
    return page.evaluate(
        """() => new Promise((resolve) => {
            const open = indexedDB.open('project_baca_db');
            open.onsuccess = () => {
                const req = open.result.transaction('local_quotes').objectStore('local_quotes').count();
                req.onsuccess = () => resolve(req.result);
            };
        })"""
    )


def test_guest_quote_merges_on_register(clean_page: Page, book_with_chapters: str):
    """A quote kept as a guest reaches the account once the sign-in sheet
    finishes registration, then leaves the device store."""
    page = clean_page
    chapter_id = api_get(f"/books/{book_with_chapters}")["chapters"][0]["id"]
    goto(page, "/me")
    _put_local_quote(page, book_with_chapters, chapter_id)
    goto(page, "/me")
    assert LOCAL_QUOTE in page.inner_text("body"), "guest shelf lists the local quote"

    email = f"merge-{int(time.time())}@example.com"
    page.click("text=Sign In")
    page.click('.segmented button:has-text("Register")')
    page.fill('input[autocomplete="nickname"]', "Merge Probe")
    page.fill('input[type="email"]', email)
    page.fill('input[type="password"]', "MergeProbe123!")
    page.click(".sheet button[type=submit]")
    otp = _mailpit_otp(email)
    page.fill(".otp-input", otp)
    page.click(".sheet button[type=submit]")
    page.wait_for_timeout(3000)
    token = page.evaluate("localStorage.getItem('baca_access_token')")
    assert token, "registration must sign the reader in"

    req = urllib.request.Request(f"{API}/quotes", headers={"Authorization": f"Bearer {token}"})
    with urllib.request.urlopen(req) as r:
        quotes = json.load(r)["data"]
    assert [q["quote_text"] for q in quotes] == [LOCAL_QUOTE]
    assert _count_local_quotes(page) == 0, "merged quotes leave the device store"

    goto(page, "/me")
    assert LOCAL_QUOTE in page.inner_text("body")

    req = urllib.request.Request(
        f"{API}/me", method="DELETE", headers={"Authorization": f"Bearer {token}"}
    )
    with urllib.request.urlopen(req):
        pass
    assert_no_page_errors(page)
