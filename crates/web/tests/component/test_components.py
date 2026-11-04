"""Component tests via the dev gallery route `/__gallery?story=`.

Pattern: Playwright docs "component testing" — gallery owned by the app,
stories own state, tests assert through the DOM. Error states mocked via
`page.route()` (service worker bypassed by fresh context per test).
"""
import json

from playwright.sync_api import Page, expect

from conftest import WEB, assert_no_page_errors

GALLERY = f"{WEB}/__gallery?story="


def _story(page: Page, name: str):
    page.goto(f"{GALLERY}{name}", wait_until="networkidle")
    page.wait_for_timeout(1500)
    assert page.query_selector("#gallery-unknown") is None, f"unknown story: {name}"
    return page.query_selector("#gallery-root")


def test_site_header_toggle_no_reload(clean_page: Page):
    page = clean_page
    root = _story(page, "site-header")
    assert root is not None
    page.click('.lang-opt:has-text("EN")')
    page.wait_for_timeout(600)
    assert page.evaluate("localStorage.getItem('rotaria_lang')") == "EN"
    assert_no_page_errors(page)


def test_auth_sheet_modes(clean_page: Page):
    page = clean_page
    root = _story(page, "auth-modal")
    assert root is not None
    assert page.query_selector(".sheet[role='dialog']") is not None
    page.click('.segmented button:has-text("Register")')
    page.wait_for_timeout(300)
    assert page.query_selector('input[autocomplete="nickname"]') is not None
    assert_no_page_errors(page)


def test_quote_finder_empty_state(clean_page: Page):
    page = clean_page
    root = _story(page, "quote-finder")
    assert root is not None
    page.fill(".sheet .search-bar input", "zzzqqq-no-such-quote")
    page.click(".sheet .search-bar .btn-primary")
    page.wait_for_timeout(4000)
    assert page.query_selector(".quote-results") is not None
    assert_no_page_errors(page)


def test_quote_finder_error_branch(clean_page: Page):
    """Mocked 500 must surface an error message, never silent emptiness."""
    page = clean_page
    page.route("**/api/**/quotes/search", lambda r: r.fulfill(status=500, body='{"success":false}'))
    _story(page, "quote-finder")
    page.fill(".sheet .search-bar input", "love")
    page.click(".sheet .search-bar .btn-primary")
    page.wait_for_timeout(3000)
    assert page.query_selector(".sheet .form-error") is not None, "error must be visible"
    assert "quote-results" in page.content()
    assert_no_page_errors(page)


def test_atomic_cards_404_tolerant(clean_page: Page):
    page = clean_page
    root = _story(page, "atomic-cards")
    assert root is not None
    page.wait_for_timeout(2000)
    assert "Not available" in page.inner_text("#gallery-root")
    assert_no_page_errors(page)


CURATOR_BOOKS = [
    {"id": "11111111-1111-4111-8111-111111111111", "title": "Naskah Terbit", "author": "A. Penulis", "language": "id",
     "status": "published", "chapter_count": 3, "chunk_count": 12,
     "created_at": "2026-10-01T10:00:00Z", "updated_at": "2026-10-01T10:00:00Z"},
    {"id": "22222222-2222-4222-8222-222222222222", "title": "Naskah Draf", "author": "B. Penulis", "language": "en",
     "status": "draft", "chapter_count": 0, "chunk_count": 0,
     "created_at": "2026-10-02T10:00:00Z", "updated_at": "2026-10-02T10:00:00Z"},
    {"id": "44444444-4444-4444-8444-444444444444", "title": "Naskah Arsip", "author": "C. Penulis", "language": "id",
     "status": "archived", "chapter_count": 2, "chunk_count": 5,
     "created_at": "2026-09-30T10:00:00Z", "updated_at": "2026-09-30T10:00:00Z"},
    {"id": "66666666-6666-4666-8666-666666666666", "title": "Arsip Setengah Jalan", "author": "D. Penulis", "language": "id",
     "status": "archived", "chapter_count": 2, "chunk_count": 0,
     "created_at": "2026-09-29T10:00:00Z", "updated_at": "2026-09-29T10:00:00Z"},
]
CURATOR_DLQ = [
    {"id": "1700000000000-0", "job_id": "deadbeef-0000-4000-8000-000000000000",
     "book_id": "22222222-2222-4222-8222-222222222222", "book_title": "Naskah Draf",
     "error": "Gemini API unavailable (HTTP 503)", "failed_at": "2026-10-03T08:00:00Z"},
    {"id": "1700000000001-0", "job_id": "cafef00d-0000-4000-8000-000000000000",
     "error": "The uploaded EPUB is gone from storage; upload it again", "failed_at": "2026-10-03T09:00:00Z"},
]
# The stream holds more dead letters than one page shows.
CURATOR_DLQ_TOTAL = 7
CURATOR_FUNNEL = [
    {"chapter_number": 1, "chapter_title": "Surat Pertama", "readers_reached": 10, "drop_off_pct": 0.0},
    {"chapter_number": 2, "chapter_title": "Balasan", "readers_reached": 6, "drop_off_pct": 40.0},
]


def test_curator_desk_tabs_rows_replay_and_funnel(clean_page: Page):
    """The desk renders every status with counts, archives and restores
    through the API, lists dead letters by book with the stream total,
    replays and dismisses them, and draws the funnel, all against mocked
    admin endpoints."""
    page = clean_page
    archived = []
    replayed = []
    dismissed = []

    def books(route):
        if route.request.method == "PATCH":
            status = json.loads(route.request.post_data)["status"]
            archived.append(status)
            book_id = route.request.url.rsplit("/", 1)[1]
            row = dict(next(b for b in CURATOR_BOOKS if b["id"] == book_id), status=status)
            route.fulfill(status=200, content_type="application/json", body=json.dumps({"success": True, "data": row}))
            return
        route.fulfill(status=200, content_type="application/json", body=json.dumps({"success": True, "data": CURATOR_BOOKS}))

    def dlq(route):
        if route.request.method == "POST":
            replayed.append(route.request.url)
            route.fulfill(status=200, content_type="application/json",
                          body=json.dumps({"success": True, "data": {"job_id": CURATOR_DLQ[0]["job_id"], "replayed": True}}))
            return
        if route.request.method == "DELETE":
            dismissed.append(route.request.url)
            route.fulfill(status=200, content_type="application/json",
                          body=json.dumps({"success": True, "data": {"dismissed": True}}))
            return
        route.fulfill(status=200, content_type="application/json", body=json.dumps(
            {"success": True, "data": {"total": CURATOR_DLQ_TOTAL, "entries": CURATOR_DLQ}}))

    page.route("**/api/**/admin/books**", books)
    page.route("**/api/**/admin/dlq**", dlq)
    page.route("**/api/**/admin/analytics/drop-off*", lambda r: r.fulfill(
        status=200, content_type="application/json", body=json.dumps({"success": True, "data": CURATOR_FUNNEL})))
    _story(page, "curator-desk")

    rows = page.query_selector_all(".manuscript-row")
    assert len(rows) == 4, "every status listed"
    assert page.query_selector('.manuscript-row:has-text("Arsip Setengah Jalan") button') is None, \
        "an archive that never finished ingesting offers no restore"
    body = page.inner_text(".manuscript-list")
    assert "Naskah Terbit" in body and "3 chapters" in body and "12 chunks" in body
    page.click('.manuscript-row:has-text("Naskah Terbit") button:has-text("Archive")')
    page.click('.manuscript-row:has-text("Naskah Terbit") button:has-text("Tap again")')
    page.wait_for_timeout(600)
    assert archived == ["archived"], "archive goes through PATCH after the second tap"
    assert "Archived" in page.inner_text('.manuscript-row:has-text("Naskah Terbit")')
    page.click('.manuscript-row:has-text("Naskah Arsip") button:has-text("Publish again")')
    page.wait_for_timeout(600)
    assert archived == ["archived", "published"], "restore publishes in one tap"
    assert "Published" in page.inner_text('.manuscript-row:has-text("Naskah Arsip")')

    page.click('.desk-tabs button:has-text("Queue")')
    page.wait_for_timeout(800)
    queue = page.inner_text(".manuscript-list")
    assert "Gemini API unavailable" in queue
    assert "Naskah Draf" in queue and "The book is gone" in queue, "each dead letter names its book"
    counter = '#dlq-title + .form-hint'
    assert page.inner_text(counter) == "7", "the counter is the stream total, not the page"
    page.click('.dlq-row:has-text("Naskah Draf") button:has-text("Replay")')
    page.wait_for_timeout(600)
    assert len(replayed) == 1 and replayed[0].endswith("/replay")
    page.click('.dlq-row:has-text("The book is gone") button:has-text("Dismiss")')
    page.wait_for_timeout(600)
    assert len(dismissed) == 1 and dismissed[0].endswith("/admin/dlq/1700000000001-0")
    assert page.query_selector(".dlq-row") is None, "replayed and dismissed entries leave the list"
    assert page.inner_text(counter) == "5"

    page.click('.desk-tabs button:has-text("Retention")')
    page.wait_for_timeout(800)
    bars = page.query_selector_all(".funnel-row")
    assert len(bars) == 2
    widths = [float(page.evaluate("el => el.style.width", b.query_selector(".funnel-bar > span")).rstrip("%")) for b in bars]
    assert widths == [100.0, 60.0], widths
    assert "40% drop" in page.inner_text(".funnel")
    assert_no_page_errors(page)


def test_curator_desk_shows_why_a_job_failed(clean_page: Page, tmp_path):
    """An upload the worker refuses shows the reason under the failure, so
    the curator knows what to fix instead of only that it failed."""
    page = clean_page
    job_id = "33333333-3333-4333-8333-333333333333"
    reason = "EPUB holds more than 5000 files"
    listings = []

    def books(route):
        listings.append(route.request.url)
        route.fulfill(status=200, content_type="application/json", body=json.dumps({"success": True, "data": CURATOR_BOOKS}))

    page.route("**/api/**/admin/books**", books)
    page.route("**/api/**/admin/books/upload", lambda r: r.fulfill(
        status=202, content_type="application/json", body=json.dumps({"success": True, "data": {
            "job_id": job_id, "book_id": CURATOR_BOOKS[1]["id"], "status": "queued"}})))
    page.route(f"**/api/**/admin/jobs/{job_id}", lambda r: r.fulfill(
        status=200, content_type="application/json", body=json.dumps({"success": True, "data": {
            "job_id": job_id, "book_id": CURATOR_BOOKS[1]["id"], "status": "failed", "progress": 100,
            "error": reason}})))
    _story(page, "curator-desk")
    epub = tmp_path / "refused.epub"
    epub.write_bytes(b"PK\x03\x04 not really an epub")
    page.set_input_files(".upload-card input[type=file]", str(epub))
    page.click(".upload-card button[type=submit]")
    card = page.locator(".job-card")
    expect(card.locator(".form-error")).to_be_visible()
    expect(card.locator(".job-error")).to_have_text(reason)
    assert page.input_value(".upload-card input[type=file]") == "", "the form is cleared after a queued upload"
    assert len(listings) >= 2, "the manuscript list reloads after a queued upload"
    assert_no_page_errors(page)


def test_curator_upload_survives_an_expired_session_and_refuses_oversize(clean_page: Page, tmp_path):
    """An upload sent with an expired access token refreshes once and goes
    out again with the file; a file over 50 MB never leaves the browser."""
    page = clean_page
    job_id = "55555555-5555-4555-8555-555555555555"
    uploads = []

    def upload(route):
        uploads.append((route.request.headers.get("authorization"), len(route.request.post_data_buffer or b"")))
        if len(uploads) == 1:
            route.fulfill(status=401, content_type="application/json", body=json.dumps(
                {"success": False, "error": {"code": "UNAUTHORIZED", "message": "Invalid or expired access token"}}))
            return
        route.fulfill(status=202, content_type="application/json", body=json.dumps({"success": True, "data": {
            "job_id": job_id, "book_id": CURATOR_BOOKS[1]["id"], "status": "queued"}}))

    page.route("**/api/**/admin/books**", lambda r: r.fulfill(
        status=200, content_type="application/json", body=json.dumps({"success": True, "data": CURATOR_BOOKS})))
    page.route("**/api/**/admin/books/upload", upload)
    page.route("**/api/**/auth/refresh", lambda r: r.fulfill(
        status=200, content_type="application/json", body=json.dumps({"success": True, "data": {
            "access_token": "fresh-access", "refresh_token": "fresh-refresh", "token_type": "Bearer", "expires_in": 900}})))
    page.route(f"**/api/**/admin/jobs/{job_id}", lambda r: r.fulfill(
        status=200, content_type="application/json", body=json.dumps({"success": True, "data": {
            "job_id": job_id, "book_id": CURATOR_BOOKS[1]["id"], "status": "queued", "progress": 0}})))
    _story(page, "curator-desk")
    page.evaluate("""() => {
        localStorage.setItem('baca_access_token', 'stale-access');
        localStorage.setItem('baca_refresh_token', 'live-refresh');
    }""")

    epub = tmp_path / "small.epub"
    epub.write_bytes(b"PK\x03\x04 small but fine")
    page.set_input_files(".upload-card input[type=file]", str(epub))
    page.click(".upload-card button[type=submit]")
    expect(page.locator(".job-card")).to_be_visible()
    assert [auth for auth, _ in uploads] == ["Bearer stale-access", "Bearer fresh-access"], uploads
    assert uploads[1][1] > 0, "the retried upload carries the file again"
    assert "expired" not in page.inner_text("body"), "no raw token error reaches the curator"

    big = tmp_path / "huge.epub"
    with open(big, "wb") as f:
        f.truncate(50 * 1024 * 1024 + 1)
    page.set_input_files(".upload-card input[type=file]", str(big))
    page.click(".upload-card button[type=submit]")
    expect(page.locator("body")).to_contain_text("over 50 MB")
    assert len(uploads) == 2, "an oversize file is refused before it is sent"
    assert_no_page_errors(page)


def test_curator_errors_read_in_the_readers_language(clean_page: Page, tmp_path):
    """In Indonesian the server's English never reaches the screen: a refused
    file says why in "kamu" copy, a rate limit says how long to wait."""
    page = clean_page
    page.add_init_script("localStorage.setItem('rotaria_lang', 'ID')")
    answers = [
        (400, {"code": "BAD_REQUEST", "message": "Uploaded file is not a ZIP-based EPUB (missing PK signature)"}),
        (400, {"code": "VALIDATION_FAILED", "message": "Field 'title' exceeds 255 characters", "details": {"title": [{"code": "length", "message": "Field 'title' exceeds 255 characters"}]}}),
        (429, {"code": "RATE_LIMITED", "message": "Rate limit exceeded: at most 5 uploads per 15 minutes. Try again in 857 seconds"}),
    ]
    page.route("**/api/**/admin/books**", lambda r: r.fulfill(
        status=200, content_type="application/json", body=json.dumps({"success": True, "data": CURATOR_BOOKS})))
    page.route("**/api/**/admin/books/upload", lambda r: r.fulfill(
        status=answers[0][0], content_type="application/json",
        body=json.dumps({"success": False, "error": answers.pop(0)[1]})))
    _story(page, "curator-desk")
    epub = tmp_path / "notes.epub"
    epub.write_bytes(b"not a zip at all")
    toasts = page.locator(".toast")
    for expected in ["Berkas itu bukan EPUB yang bisa kami baca", "Judulnya terlalu panjang", "Coba lagi dalam 15 menit"]:
        page.set_input_files(".upload-card input[type=file]", str(epub))
        page.click(".upload-card button[type=submit]")
        expect(toasts.filter(has_text=expected)).to_be_visible()
    body = page.inner_text("body")
    assert "ZIP-based" not in body and "Rate limit" not in body, body[-400:]
    assert_no_page_errors(page)
