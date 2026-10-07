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
]
CURATOR_DLQ = [
    {"id": "1700000000000-0", "job_id": "deadbeef-0000-4000-8000-000000000000",
     "error": "Gemini API unavailable (HTTP 503)", "failed_at": "2026-10-03T08:00:00Z"},
]
CURATOR_FUNNEL = [
    {"chapter_number": 1, "chapter_title": "Surat Pertama", "readers_reached": 10, "drop_off_pct": 0.0},
    {"chapter_number": 2, "chapter_title": "Balasan", "readers_reached": 6, "drop_off_pct": 40.0},
]


def test_curator_desk_tabs_rows_replay_and_funnel(clean_page: Page):
    """The desk renders every status with counts, archives through the API,
    replays a dead letter, and draws the funnel, all against mocked admin
    endpoints."""
    page = clean_page
    archived = []
    replayed = []

    def books(route):
        if route.request.method == "PATCH":
            archived.append(json.loads(route.request.post_data)["status"])
            row = dict(CURATOR_BOOKS[0], status="archived")
            route.fulfill(status=200, content_type="application/json", body=json.dumps({"success": True, "data": row}))
            return
        route.fulfill(status=200, content_type="application/json", body=json.dumps({"success": True, "data": CURATOR_BOOKS}))

    def dlq(route):
        if route.request.method == "POST":
            replayed.append(route.request.url)
            route.fulfill(status=200, content_type="application/json",
                          body=json.dumps({"success": True, "data": {"job_id": CURATOR_DLQ[0]["job_id"], "replayed": True}}))
            return
        route.fulfill(status=200, content_type="application/json", body=json.dumps({"success": True, "data": CURATOR_DLQ}))

    page.route("**/api/**/admin/books**", books)
    page.route("**/api/**/admin/dlq**", dlq)
    page.route("**/api/**/admin/analytics/drop-off*", lambda r: r.fulfill(
        status=200, content_type="application/json", body=json.dumps({"success": True, "data": CURATOR_FUNNEL})))
    _story(page, "curator-desk")

    rows = page.query_selector_all(".manuscript-row")
    assert len(rows) == 2, "both statuses listed"
    body = page.inner_text(".manuscript-list")
    assert "Naskah Terbit" in body and "3 chapters" in body and "12 chunks" in body
    page.click('.manuscript-row:has-text("Naskah Terbit") button:has-text("Archive")')
    page.click('.manuscript-row:has-text("Naskah Terbit") button:has-text("Tap again")')
    page.wait_for_timeout(600)
    assert archived == ["archived"], "archive goes through PATCH after the second tap"
    assert "Archived" in page.inner_text('.manuscript-row:has-text("Naskah Terbit")')

    page.click('.desk-tabs button:has-text("Queue")')
    page.wait_for_timeout(800)
    assert "Gemini API unavailable" in page.inner_text(".manuscript-list")
    page.click('.dlq-row button:has-text("Replay")')
    page.wait_for_timeout(600)
    assert len(replayed) == 1 and replayed[0].endswith("/replay")
    assert page.query_selector(".dlq-row") is None, "replayed entry leaves the list"

    page.click('.desk-tabs button:has-text("Retention")')
    page.wait_for_timeout(800)
    bars = page.query_selector_all(".funnel-row")
    assert len(bars) == 2
    widths = [float(page.evaluate("el => el.style.width", b.query_selector(".funnel-bar > span")).rstrip("%")) for b in bars]
    assert widths == [100.0, 60.0], widths
    assert "40% drop" in page.inner_text(".funnel")
    assert_no_page_errors(page)
