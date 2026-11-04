"""Authed flow: OTP via Mailpit, automatic progress save, shelf, cleanup."""
import base64
import json
import re
import subprocess
import time
import urllib.error
import urllib.request

import pytest
from playwright.sync_api import Browser, Page, expect

from conftest import (
    API,
    WEB,
    api_get,
    api_post as _post,
    assert_no_page_errors,
    goto,
    install_fetch_shim,
    mailpit_otp as _mailpit_otp,
    register_reader,
)

pytestmark = pytest.mark.usefixtures("reset_signup_cap")



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
    started = _post("/auth/signup", {"display_name": "E2E", "email": email, "password": "E2EPassword123!"})
    otp = _mailpit_otp(email)
    tokens = _post("/auth/verify-otp", {"email": email, "registration_id": started["registration_id"], "otp": otp})
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


# Words of the seeded book's first chapter: the server keeps only quotes
# that are in their chapter.
LOCAL_QUOTE = "Ia menulis surat pertamanya di meja dapur"


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


def _chapter_phrases(book_id: str, count: int) -> list[str]:
    """``count`` distinct four-word phrases of the book's first chapter, each
    a quote the server accepts."""
    html = api_get(f"/books/{book_id}/chapters/1")["html_content"]
    words = re.sub(r"<[^>]+>", " ", html).split()
    phrases: list[str] = []
    for i in range(len(words) - 3):
        phrase = " ".join(words[i : i + 4])
        if phrase not in phrases:
            phrases.append(phrase)
        if len(phrases) == count:
            return phrases
    raise AssertionError(f"the first chapter holds fewer than {count} phrases")


def test_profile_quotes_open_a_page_at_a_time(clean_page: Page, book_with_chapters: str):
    """Sixty saved quotes open fifty at a time and the rest follow on request.
    Each batch shares one timestamp, so the pages also prove the id tiebreak:
    nothing repeats and nothing is skipped."""
    page = clean_page
    tokens = register_reader("pages")
    token = tokens["access_token"]
    chapter_id = api_get(f"/books/{book_with_chapters}")["chapters"][0]["id"]
    texts = _chapter_phrases(book_with_chapters, 60)
    for batch in (texts[:50], texts[50:]):
        items = [{"book_id": book_with_chapters, "chapter_id": chapter_id, "quote_text": t} for t in batch]
        _post("/quotes/save-batch", {"items": items}, token)
    try:
        goto(page, "/")
        page.evaluate(_sign_in_storage(tokens))
        goto(page, "/me")
        rows = page.locator(".quote-row blockquote")
        expect(rows).to_have_count(50)
        expect(page.get_by_text("All (50+)")).to_be_visible()
        more = page.locator("section[aria-labelledby='quotes-title'] .load-more button")
        more.click()
        expect(rows).to_have_count(60)
        expect(more).to_have_count(0)
        expect(page.get_by_text("All (60)")).to_be_visible()
        assert sorted(rows.all_inner_texts()) == sorted(texts)
        assert_no_page_errors(page)
    finally:
        req = urllib.request.Request(f"{API}/me", method="DELETE", headers={"Authorization": f"Bearer {token}"})
        with urllib.request.urlopen(req):
            pass


def _owner(tokens: dict) -> str:
    payload = tokens["access_token"].split(".")[1]
    return json.loads(base64.urlsafe_b64decode(payload + "=" * (-len(payload) % 4)))["sub"]


def _active_progress(tokens: dict):
    req = urllib.request.Request(f"{API}/progress/active", headers={"Authorization": f"Bearer {tokens['access_token']}"})
    with urllib.request.urlopen(req) as r:
        return json.load(r)["data"]


def _sign_in_storage(tokens: dict) -> str:
    return (
        f"localStorage.setItem('baca_access_token','{tokens['access_token']}');"
        f"localStorage.setItem('baca_refresh_token','{tokens['refresh_token']}');"
    )


def test_signup_past_the_mail_cap_stays_on_the_form(clean_page: Page):
    page = clean_page
    email = f"e2e-cooldown-{int(time.time() * 1000)}@example.com"
    for _ in range(3):
        _post("/auth/signup", {"display_name": "E2E", "email": email, "password": "E2EPassword123!"})
    goto(page, "/me")
    page.click(".guest-shelf .btn-primary")
    page.click(".sheet [role='tab']:has-text('Register')")
    page.fill(".sheet input[type='text']", "Second Try")
    page.fill(".sheet input[type='email']", email)
    page.fill(".sheet input[type='password']", "E2EPassword123!")
    page.click(".sheet button[type='submit']")
    page.wait_for_timeout(1200)
    expect(page.locator(".sheet .form-error")).to_be_visible()
    expect(page.locator("#auth-title")).to_have_text("Create an account")
    assert page.query_selector(".sheet .otp-input") is None, "no OTP step for a code never sent"
    assert_no_page_errors(page)


def test_garbage_tokens_show_the_guest_shelf(clean_page: Page):
    page = clean_page
    goto(page, "/")
    page.evaluate(_sign_in_storage({"access_token": "garbage.access.token", "refresh_token": "rt_garbage"}))
    goto(page, "/me")
    expect(page.locator(".guest-shelf .btn-primary")).to_be_visible()
    assert page.query_selector(".skeleton") is None, "no endless skeleton"
    assert page.evaluate("() => localStorage.getItem('baca_access_token')") is None
    assert_no_page_errors(page)


def test_offline_sign_out_sticks_after_reload(clean_page: Page):
    page = clean_page
    tokens = register_reader("offline-out")
    goto(page, "/")
    page.evaluate(_sign_in_storage(tokens))
    goto(page, "/me")
    sign_out = page.get_by_role("button", name="Sign out", exact=True).last
    expect(sign_out).to_be_visible()
    page.context.set_offline(True)
    try:
        sign_out.click()
        page.wait_for_timeout(500)
        assert page.evaluate("() => localStorage.getItem('baca_access_token')") is None
        assert page.evaluate("() => localStorage.getItem('baca_refresh_token')") is None
    finally:
        page.context.set_offline(False)
    goto(page, "/me")
    expect(page.locator(".guest-shelf .btn-primary")).to_be_visible()
    assert_no_page_errors(page)


PENDING_STORE = "pending_sync_queue"
QUEUE_WRITE = """([store, record]) => new Promise((resolve, reject) => {
  const open = indexedDB.open('project_baca_db');
  open.onsuccess = () => {
    const tx = open.result.transaction(store, 'readwrite');
    tx.objectStore(store).add(record);
    tx.oncomplete = () => resolve(true);
    tx.onerror = () => reject(tx.error);
  };
})"""
QUEUED_ANCHORS = """store => new Promise(resolve => {
  const open = indexedDB.open('project_baca_db');
  open.onsuccess = () => {
    const all = open.result.transaction(store, 'readonly').objectStore(store).getAll();
    all.onsuccess = () => resolve(all.result.map(op => op.update.last_anchor_cfi));
  };
})"""


def test_drain_keeps_writes_queued_meanwhile_and_never_doubles(browser: Browser, book_with_chapters: str):
    book_id = book_with_chapters
    chapter = api_get(f"/books/{book_id}")["chapters"][0]
    tokens = register_reader("drain")
    context = browser.new_context(color_scheme="dark", reduced_motion="reduce")
    install_fetch_shim(context, hold=[f"/progress/{book_id}"])
    context.add_init_script(_sign_in_storage(tokens))
    page = context.new_page()
    errors: list[str] = []
    page.on("pageerror", lambda e: errors.append(str(e)))
    sent: list[str] = []
    page.on(
        "request",
        lambda r: sent.append(json.loads(r.post_data or "{}").get("last_anchor_cfi"))
        if r.method == "PUT" and r.url.endswith(f"/progress/{book_id}")
        else None,
    )

    def op(anchor: str) -> dict:
        return {
            "book_id": book_id,
            "update": {"chapter_id": chapter["id"], "last_anchor_cfi": anchor, "completion_percentage": 5.0},
            "user_id": _owner(tokens),
        }

    try:
        page.goto(f"{WEB}/read/{book_id}?chapter={chapter['chapter_number']}", wait_until="domcontentloaded")
        page.wait_for_selector(".chapter-body")
        page.wait_for_timeout(600)
        page.evaluate(QUEUE_WRITE, [PENDING_STORE, op("p-901")])
        page.evaluate("() => window.dispatchEvent(new Event('online'))")
        page.wait_for_timeout(400)
        page.evaluate(QUEUE_WRITE, [PENDING_STORE, op("p-902")])
        page.evaluate("() => window.dispatchEvent(new Event('online'))")
        page.wait_for_timeout(400)
        page.evaluate("() => window.__release()")
        page.wait_for_timeout(1500)
        assert sent.count("p-901") == 1, f"replayed once, sent {sent}"
        assert "p-902" in page.evaluate(QUEUED_ANCHORS, PENDING_STORE), "a write queued during the drain stays queued"
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_queued_progress_never_reaches_the_next_account(browser: Browser, book_with_chapters: str):
    book_id = book_with_chapters
    chapter = api_get(f"/books/{book_id}")["chapters"][0]
    first, second = register_reader("queue-a"), register_reader("queue-b")
    context = browser.new_context(color_scheme="dark", reduced_motion="reduce")
    install_fetch_shim(context, chapters=1)
    page = context.new_page()
    errors: list[str] = []
    page.on("pageerror", lambda e: errors.append(str(e)))
    try:
        goto(page, "/")
        page.evaluate(_sign_in_storage(first))
        page.goto(f"{WEB}/read/{book_id}?chapter={chapter['chapter_number']}", wait_until="domcontentloaded")
        page.wait_for_selector(".chapter-body")
        page.wait_for_timeout(600)
        page.route(f"{API}/progress/**", lambda route: route.abort())
        page.keyboard.press("ArrowRight")
        page.wait_for_timeout(1700)
        assert page.evaluate(QUEUED_ANCHORS, PENDING_STORE), "the first account's page waits in the queue"
        page.unroute(f"{API}/progress/**")
        _put_local_quote(page, book_id, chapter["id"])

        goto(page, "/me")
        page.get_by_role("button", name="Sign out", exact=True).last.click()
        page.wait_for_timeout(800)
        assert page.evaluate(QUEUED_ANCHORS, PENDING_STORE) == [], "signing out drops the account's unsent writes"
        assert _count_local_quotes(page) == 1, "data kept for the guest stays"

        # A record from before writes carried their owner is never sent either.
        page.evaluate(QUEUE_WRITE, [PENDING_STORE, {"book_id": book_id, "update": {"chapter_id": chapter["id"], "last_anchor_cfi": "p-7", "completion_percentage": 40.0}}])
        page.evaluate(_sign_in_storage(second))
        goto(page, "/me")
        page.wait_for_timeout(1500)
        assert page.evaluate(QUEUED_ANCHORS, PENDING_STORE) == []
        assert _active_progress(second) is None, "nothing of the first account reaches the second"
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_wrong_current_password_stays_in_the_form(browser: Browser):
    tokens = register_reader("pwd-wrong")
    context = browser.new_context(color_scheme="dark", reduced_motion="reduce")
    page = context.new_page()
    errors: list[str] = []
    page.on("pageerror", lambda e: errors.append(str(e)))
    puts: list[str] = []
    page.on("request", lambda r: puts.append(r.url) if r.method == "PUT" and r.url.endswith("/me/password") else None)
    try:
        goto(page, "/")
        page.evaluate(_sign_in_storage(tokens))
        goto(page, "/me")
        page.fill("input[autocomplete='current-password']", "NotMyPassword123!")
        page.fill("input[autocomplete='new-password']", "BrandNewPass123!")
        page.click("#akun form button[type='submit']")
        expect(page.locator("#akun form .form-error")).to_contain_text("does not match")
        assert len(puts) == 1, puts
        assert page.evaluate("() => localStorage.getItem('baca_access_token')") == tokens["access_token"]
        assert errors == [], errors[:3]
    finally:
        context.close()


def test_refused_refresh_sends_no_more_bearer_requests(clean_page: Page):
    page = clean_page
    events: list[tuple[str, bool]] = []
    page.on("request", lambda r: events.append((r.url, "authorization" in r.headers)))
    page.on("response", lambda r: events.append(("refresh-answered", False)) if r.url.endswith("/auth/refresh") else None)
    goto(page, "/")
    page.evaluate(_sign_in_storage({"access_token": "garbage.access.token", "refresh_token": "rt_garbage"}))
    goto(page, "/me")
    page.wait_for_timeout(1500)
    answered = [i for i, (url, _) in enumerate(events) if url == "refresh-answered"]
    assert answered, "the refresh was tried"
    late = [url for url, bearer in events[answered[0] + 1:] if bearer]
    assert late == [], late
    assert_no_page_errors(page)


def test_fourth_code_in_an_hour_is_refused_and_not_mailed():
    email = f"e2e-cap-{int(time.time() * 1000)}@example.com"
    body = {"display_name": "E2E", "email": email, "password": "E2EPassword123!"}
    for _ in range(3):
        _post("/auth/signup", body)

    def mailed() -> int:
        with urllib.request.urlopen(f"http://localhost:8025/api/v1/search?query=to:{email}") as r:
            return json.load(r)["messages_count"]

    before = mailed()
    with pytest.raises(urllib.error.HTTPError) as refused:
        _post("/auth/signup", body)
    assert refused.value.code == 429
    assert refused.value.headers.get("Retry-After")
    time.sleep(1)
    assert mailed() == before == 3


def test_quote_dates_read_on_the_readers_clock(browser: Browser, book_with_chapters: str):
    """A quote saved at 06:56 WIB on 3 November is dated 3 November in
    Jakarta, though the server stored it at 23:56 UTC on the 2nd."""
    tokens = register_reader("wib")
    chapter_id = api_get(f"/books/{book_with_chapters}")["chapters"][0]["id"]
    sql = (
        "INSERT INTO saved_quotes (id, user_id, book_id, chapter_id, quote_text, created_at) "
        f"VALUES (gen_random_uuid(), '{_owner(tokens)}', '{book_with_chapters}', '{chapter_id}', "
        "'Pagi di Jakarta', '2026-11-02T23:56:00Z')"
    )
    subprocess.run(
        ["docker", "exec", "project_baca_db", "psql", "-U", "baca_user", "-d", "project_baca_db", "-c", sql],
        capture_output=True,
        check=True,
    )
    context = browser.new_context(base_url=WEB, timezone_id="Asia/Jakarta", locale="en-US")
    page = context.new_page()
    page.errors = []  # type: ignore[attr-defined]
    page.on("pageerror", lambda e: page.errors.append(str(e)))
    try:
        goto(page, "/")
        page.evaluate(_sign_in_storage(tokens))
        goto(page, "/me")
        row = page.locator(".quote-row", has_text="Pagi di Jakarta")
        expect(row.locator(".quote-meta")).to_contain_text("3 Nov 2026")
        assert_no_page_errors(page)
    finally:
        context.close()
        req = urllib.request.Request(f"{API}/me", method="DELETE", headers={"Authorization": f"Bearer {tokens['access_token']}"})
        with urllib.request.urlopen(req):
            pass


def _bearer(method: str, path: str, token: str, payload: dict | None = None):
    req = urllib.request.Request(
        f"{API}{path}",
        method=method,
        data=json.dumps(payload).encode() if payload is not None else None,
        headers={"Authorization": f"Bearer {token}", "Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req) as r:
        return json.load(r)["data"]


def _email(tokens: dict) -> str:
    payload = tokens["access_token"].split(".")[1]
    return json.loads(base64.urlsafe_b64decode(payload + "=" * (-len(payload) % 4)))["email"]


def _signed_in_page(browser: Browser, tokens: dict) -> Page:
    page = browser.new_context(base_url=WEB).new_page()
    page.errors = []  # type: ignore[attr-defined]
    page.on("pageerror", lambda e: page.errors.append(str(e)))
    goto(page, "/")
    page.evaluate(_sign_in_storage(tokens))
    goto(page, "/me")
    return page


def test_sign_out_everywhere_disarms_itself(browser: Browser):
    """The first tap only arms the button, and the arm lapses after four
    seconds or at a click anywhere else, so a stray tap later signs nothing
    out."""
    tokens = register_reader("arm")
    page = _signed_in_page(browser, tokens)
    try:
        button = page.locator("#akun > .sheet-actions button").first
        expect(button).to_have_text("Sign out everywhere")
        button.click()
        expect(button).to_have_text("Tap again to confirm")
        page.wait_for_timeout(4300)
        expect(button).to_have_text("Sign out everywhere")
        button.click()
        expect(button).to_have_text("Tap again to confirm")
        page.locator("#akun h2").click()
        expect(button).to_have_text("Sign out everywhere")
        assert page.evaluate("() => localStorage.getItem('baca_access_token')") == tokens["access_token"]
        assert_no_page_errors(page)
    finally:
        page.context.close()
        _bearer("DELETE", "/me", tokens["access_token"])


def test_password_change_refreshes_the_device_list(browser: Browser):
    tokens = register_reader("devices")
    for _ in range(2):
        _post("/auth/login", {"email": _email(tokens), "password": "E2EPassword123!"})
    page = _signed_in_page(browser, tokens)
    try:
        rows = page.locator(".device-row")
        expect(rows).to_have_count(3)
        page.fill("input[autocomplete='current-password']", "E2EPassword123!")
        page.fill("input[autocomplete='new-password']", "BrandNewPass123!")
        page.click("#akun form button[type='submit']")
        expect(page.get_by_text("Password updated.")).to_be_visible()
        expect(rows).to_have_count(1)
        expect(rows.first).to_have_class(re.compile(r"\bcurrent\b"))
        assert_no_page_errors(page)
    finally:
        page.context.close()


def test_shelf_shows_every_book_in_progress(browser: Browser):
    """Three books in progress all sit on the shelf, with no "still empty"
    note above them; a reader with only quotes gets a short line instead of
    the empty-shelf invitation."""
    tokens = register_reader("shelf")
    token = tokens["access_token"]
    books = [b["id"] for b in api_get("/books?limit=3")]
    for i, book in enumerate(books):
        chapter = api_get(f"/books/{book}")["chapters"][0]["id"]
        _bearer("PUT", f"/progress/{book}", token, {"chapter_id": chapter, "last_anchor_cfi": "p-1", "completion_percentage": 20.0 + i})
    page = _signed_in_page(browser, tokens)
    try:
        shelf = page.locator("section[aria-labelledby='shelf-title']")
        expect(shelf.locator(".shelf-item")).to_have_count(3)
        expect(shelf.locator(".empty-state")).to_have_count(0)
        assert_no_page_errors(page)
    finally:
        page.context.close()
        _bearer("DELETE", "/me", token)

    tokens = register_reader("quotes-only")
    token = tokens["access_token"]
    book = books[0]
    detail = api_get(f"/books/{book}")
    phrase = _chapter_phrases(book, 1)[0]
    _post("/quotes/save-batch", {"items": [{"book_id": book, "chapter_id": detail["chapters"][0]["id"], "quote_text": phrase}]}, token)
    page = _signed_in_page(browser, tokens)
    try:
        shelf = page.locator("section[aria-labelledby='shelf-title']")
        expect(shelf.get_by_text("No book in progress yet.")).to_be_visible()
        expect(shelf.locator(".empty-state")).to_have_count(0)
    finally:
        page.context.close()
        _bearer("DELETE", "/me", token)


def test_two_reader_tabs_beat_without_errors(browser: Browser, book_with_chapters: str):
    """The same book open in two tabs: both beat once a minute, the server
    credits one and answers the other `credited: false`, and nothing reaches
    the console as an error."""
    tokens = register_reader("twotabs")
    context = browser.new_context(base_url=WEB)
    beats: list[tuple[int, bool]] = []
    console_errors: list[str] = []

    def on_response(r):
        if r.request.method == "POST" and r.url.endswith("/activity/heartbeat"):
            beats.append((r.status, r.json()["data"]["credited"] if r.ok else None))

    try:
        pages = []
        for _ in range(2):
            page = context.new_page()
            page.on("response", on_response)
            page.on("console", lambda m: console_errors.append(m.text) if m.type == "error" else None)
            goto(page, "/")
            page.evaluate(_sign_in_storage(tokens))
            goto(page, f"/read/{book_with_chapters}")
            pages.append(page)
        pages[0].wait_for_timeout(66_000)
        assert len(beats) >= 2, beats
        assert all(status == 200 for status, _ in beats), beats
        assert [credited for _, credited in beats].count(True) >= 1, beats
        # A refused heartbeat would surface as a 429 resource error.
        assert not [e for e in console_errors if "429" in e or "Too Many" in e], console_errors[:3]
    finally:
        context.close()
        _bearer("DELETE", "/me", tokens["access_token"])
