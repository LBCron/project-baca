"""Public reads leave the token behind: signed in, the catalog and a chapter
go out without `Authorization`, so the dev stack (web :3000, API :8080, cross
origin like the apps) sends them with no CORS preflight, and the server does
not mark them `no-store`."""
import re
import urllib.request

import pytest
from playwright.sync_api import Browser

from conftest import API, api_get, assert_no_page_errors, goto, register_reader

pytestmark = pytest.mark.usefixtures("reset_signup_cap")

PUBLIC_READ = re.compile(r"/api/v1/(books(/|$)|badges$)")


def _path(url: str) -> str:
    return url.split("?")[0]


def test_signed_in_public_reads_carry_no_token_and_no_preflight(browser: Browser, book_with_chapters: str):
    tokens = register_reader("public-reads")
    book = book_with_chapters
    chapter = api_get(f"/books/{book}")["chapters"][0]["chapter_number"]
    context = browser.new_context(service_workers="block")
    page = context.new_page()
    page.errors = []  # type: ignore[attr-defined]
    page.on("pageerror", lambda e: page.errors.append(str(e)))

    reads: list[tuple[str, str | None, str]] = []
    preflights: list[str] = []

    def on_response(response):
        request = response.request
        if request.method == "GET" and PUBLIC_READ.search(_path(request.url)):
            reads.append((
                request.url,
                request.all_headers().get("authorization"),
                response.all_headers().get("cache-control", ""),
            ))

    # Playwright reports no preflight as a request of its own; DevTools does.
    cdp = context.new_cdp_session(page)
    cdp.send("Network.enable")
    cdp.on(
        "Network.requestWillBeSent",
        lambda e: preflights.append(e["request"]["url"]) if e["request"]["method"] == "OPTIONS" else None,
    )

    try:
        goto(page, "/")
        page.evaluate(
            f"localStorage.setItem('baca_access_token','{tokens['access_token']}');"
            f"localStorage.setItem('baca_refresh_token','{tokens['refresh_token']}');"
        )
        page.on("response", on_response)
        goto(page, "/")
        goto(page, f"/read/{book}?chapter={chapter}")

        catalog = [r for r in reads if _path(r[0]).endswith("/api/v1/books")]
        opened = [r for r in reads if _path(r[0]).endswith(f"/books/{book}/chapters/{chapter}")]
        assert catalog, "signed-in home never fetched the catalog"
        assert opened, "the reader never fetched the chapter"
        for url, authorization, cache_control in reads:
            assert authorization is None, f"public read sent a token: {url}"
            assert "no-store" not in cache_control, f"public read marked no-store: {url}"
        public_preflights = [u for u in preflights if PUBLIC_READ.search(_path(u))]
        assert public_preflights == [], f"preflight before public reads: {public_preflights[:3]}"

        # Control: a bearer GET from this origin does preflight, so the
        # observer above would have seen one for the public reads.
        page.evaluate(
            "t => fetch(t.api + '/me', {headers: {Authorization: 'Bearer ' + t.token}}).then(r => r.status)",
            {"api": API, "token": tokens["access_token"]},
        )
        page.wait_for_timeout(500)
        assert any(_path(u).endswith("/api/v1/me") for u in preflights), "DevTools reported no preflight at all"
        assert_no_page_errors(page)
    finally:
        context.close()
        req = urllib.request.Request(
            f"{API}/me", method="DELETE", headers={"Authorization": f"Bearer {tokens['access_token']}"}
        )
        with urllib.request.urlopen(req):
            pass
