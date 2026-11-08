#!/usr/bin/env python3
"""Smoke test of a deployed site: shell headers, cache rules, routing, the
www redirect, API health, then a reader in Chromium opening a chapter with no
CSP violation and no request to another origin.

Usage: prod_smoke.py https://rotaria.site

While Cloudflare Access guards the site, CF_ACCESS_CLIENT_ID and
CF_ACCESS_CLIENT_SECRET go on every request to the site and never to another
origin. SMOKE_EMAIL and SMOKE_PASSWORD sign the browser in first (the session
is revoked at the end). SMOKE_BROWSER=0 runs the HTTP checks only, for an
edge in front of a stub API. Exits 1 when any check fails.
"""
import json
import os
import re
import sys
import urllib.error
import urllib.request
from urllib.parse import urlsplit

READ_PATH = "/read/a0000000-0000-4000-8000-000000000001"
WASM = re.compile(r"/(web-[0-9a-f]{16}_bg\.wasm)")
LOCAL_HOSTS = {"localhost", "127.0.0.1", "::1"}

failures: list[str] = []


def check(label: str, ok: bool, detail: str = "") -> None:
    print(f"{'PASS' if ok else 'FAIL'} {label}" + (f": {detail}" if detail and not ok else ""))
    if not ok:
        failures.append(label)


def access_headers() -> dict[str, str]:
    client, secret = os.environ.get("CF_ACCESS_CLIENT_ID"), os.environ.get("CF_ACCESS_CLIENT_SECRET")
    return {"CF-Access-Client-Id": client, "CF-Access-Client-Secret": secret} if client and secret else {}


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


OPENER = urllib.request.build_opener(NoRedirect)


def fetch(url: str, headers: dict[str, str] | None = None, data: bytes | None = None):
    """Status, lower-cased headers, and body; redirects are answered, not followed."""
    req = urllib.request.Request(url, data=data, headers={**access_headers(), **(headers or {})})
    try:
        with OPENER.open(req, timeout=20) as r:
            return r.status, {k.lower(): v for k, v in r.headers.items()}, r.read()
    except urllib.error.HTTPError as e:
        return e.code, {k.lower(): v for k, v in e.headers.items()}, e.read()


def http_checks(base: str) -> None:
    status, headers, body = fetch(f"{base}/")
    check("shell answers 200", status == 200, str(status))
    csp = headers.get("content-security-policy", "")
    for directive in ("img-src 'self';", "connect-src 'self';", "base-uri 'none';", "script-src 'self' 'wasm-unsafe-eval';"):
        check(f"CSP has {directive}", directive in csp, csp)
    for name, want in (
        ("cross-origin-opener-policy", "same-origin"),
        ("cross-origin-resource-policy", "same-origin"),
        ("strict-transport-security", "max-age=31536000; includeSubDomains"),
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        ("referrer-policy", "strict-origin-when-cross-origin"),
        ("cache-control", "no-cache"),
    ):
        check(f"shell {name}: {want}", headers.get(name) == want, str(headers.get(name)))
    check("shell permissions-policy", "camera=()" in headers.get("permissions-policy", ""))
    check("shell has no server banner", "server" not in headers, str(headers.get("server")))

    wasm = WASM.search(body.decode("utf-8", "replace"))
    check("shell names the hashed wasm", wasm is not None)
    if wasm:
        status, headers, _ = fetch(f"{base}/{wasm.group(1)}", {"Accept-Encoding": "br"})
        check("wasm 200", status == 200, str(status))
        check("wasm content-type", headers.get("content-type") == "application/wasm", str(headers.get("content-type")))
        check("wasm content-encoding br", headers.get("content-encoding") == "br", str(headers.get("content-encoding")))
        check(
            "wasm immutable",
            headers.get("cache-control") == "public, max-age=31536000, immutable",
            str(headers.get("cache-control")),
        )

    status, headers, _ = fetch(f"{base}/sw.js")
    check("sw.js no-cache", status == 200 and headers.get("cache-control") == "no-cache", f"{status} {headers.get('cache-control')}")
    status, _, _ = fetch(f"{base}/missing.wasm")
    check("missing file is 404", status == 404, str(status))
    status, headers, _ = fetch(f"{base}{READ_PATH}")
    check("route falls back to the shell", status == 200 and headers.get("content-type", "").startswith("text/html"),
          f"{status} {headers.get('content-type')}")
    status, _, _ = fetch(f"{base}/api/v1/health")
    check("API health through the edge", status == 200, str(status))

    parts = urlsplit(base)
    if parts.hostname in LOCAL_HOSTS:
        status, headers, _ = fetch(f"{base}/read?x=1", {"Host": "www.rotaria.test"})
        want = "https://rotaria.test/read?x=1"
    else:
        status, headers, _ = fetch(f"{parts.scheme}://www.{parts.netloc}/read?x=1")
        want = f"https://{parts.netloc}/read?x=1"
    check("www redirects permanently to the apex", status in (301, 308) and headers.get("location") == want,
          f"{status} {headers.get('location')}")


def sign_in(base: str) -> dict | None:
    email, password = os.environ.get("SMOKE_EMAIL"), os.environ.get("SMOKE_PASSWORD")
    if not (email and password):
        print("SKIP sign-in: SMOKE_EMAIL/SMOKE_PASSWORD not set")
        return None
    status, _, body = fetch(f"{base}/api/v1/auth/login", {"Content-Type": "application/json"},
                            json.dumps({"email": email, "password": password}).encode())
    check("smoke account signs in", status == 200, str(status))
    return json.loads(body)["data"] if status == 200 else None


def sign_out(base: str, tokens: dict) -> None:
    fetch(f"{base}/api/v1/auth/logout",
          {"Content-Type": "application/json", "Authorization": f"Bearer {tokens['access_token']}"},
          json.dumps({"refresh_token": tokens["refresh_token"]}).encode())


def browser_checks(base: str) -> None:
    from playwright.sync_api import sync_playwright

    origin = "{0.scheme}://{0.netloc}".format(urlsplit(base))
    status, _, body = fetch(f"{base}/api/v1/books?limit=1")
    books = (json.loads(body).get("data") or []) if status == 200 else []
    check("catalog has a book", bool(books), str(status))
    if not books:
        return
    book = books[0]["id"]
    _, _, body = fetch(f"{base}/api/v1/books/{book}")
    chapter = json.loads(body)["data"]["chapters"][0]["chapter_number"]
    tokens = sign_in(base)

    foreign: list[str] = []
    violations: list[str] = []

    def only_this_origin(route):
        url = route.request.url
        if url.startswith(origin + "/"):
            route.continue_(headers={**route.request.headers, **access_headers()})
        else:
            # Aborted, so the Access token never travels to another origin.
            foreign.append(url)
            route.abort()

    with sync_playwright() as p:
        browser = p.chromium.launch()
        context = browser.new_context(service_workers="block")
        context.route("**/*", only_this_origin)
        context.add_init_script(
            "window.__csp = []; document.addEventListener('securitypolicyviolation',"
            " e => window.__csp.push(e.violatedDirective + ' ' + e.blockedURI));"
        )
        page = context.new_page()
        page.on("console", lambda m: violations.append(m.text) if "Content Security Policy" in m.text else None)
        try:
            page.goto(f"{base}/", wait_until="networkidle")
            if tokens:
                page.evaluate(
                    "t => { localStorage.setItem('baca_access_token', t.access_token);"
                    " localStorage.setItem('baca_refresh_token', t.refresh_token); }",
                    tokens,
                )
            with page.expect_response(lambda r: r.url.endswith(f"/books/{book}/chapters/{chapter}")) as opened:
                page.goto(f"{base}/read/{book}?chapter={chapter}", wait_until="networkidle")
            check("reader opens the chapter", opened.value.status == 200, str(opened.value.status))
            page.wait_for_timeout(1500)
            violations.extend(page.evaluate("window.__csp"))
        finally:
            browser.close()
            if tokens:
                sign_out(base, tokens)
    check("no CSP violation", violations == [], "; ".join(violations[:3]))
    check("no request to another origin", foreign == [], "; ".join(foreign[:3]))


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    base = sys.argv[1].rstrip("/")
    http_checks(base)
    if os.environ.get("SMOKE_BROWSER", "1") != "0":
        browser_checks(base)
    print(f"{len(failures)} failed" if failures else "all checks passed")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
