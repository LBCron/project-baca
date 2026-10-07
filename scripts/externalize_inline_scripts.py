#!/usr/bin/env python3
"""Move Trunk's inline <script> blocks into hashed external files.

Trunk injects an inline module bootstrap into index.html (and a live-reload
script under `trunk serve`). Serving those as `/boot-<sha256>.js` lets the
edge send a Content-Security-Policy whose script-src has no 'unsafe-inline',
so markup that slips past the worker sanitizer cannot run as script.

Runs as a Trunk `post_build` hook on the staging directory (TRUNK_STAGING_DIR)
or on the directory given as the first argument. The serve-mode live-reload
script keeps its request-time placeholders and is left alone (dev only). `--check <dir>` only
verifies that index.html carries no inline script and exits non-zero
otherwise. Standard library only, so the edge Docker build needs nothing
beyond python3.
"""
import hashlib
import os
import re
import sys
from pathlib import Path

SCRIPT_RE = re.compile(r"<script(?P<attrs>[^>]*)>(?P<body>.*?)</script>", re.DOTALL)


# `trunk serve` injects a live-reload script whose address placeholders are
# filled at request time; it never ships in a release build and stays inline.
SERVE_ONLY_MARKER = "{{__trunk_"


def _is_inline(match: re.Match) -> bool:
    body = match.group("body")
    return "src=" not in match.group("attrs") and bool(body.strip()) and SERVE_ONLY_MARKER not in body


def externalize(dist: Path, public_url: str) -> int:
    index = dist / "index.html"
    html = index.read_text(encoding="utf-8")
    moved = 0

    def replace(match: re.Match) -> str:
        nonlocal moved
        if not _is_inline(match):
            return match.group(0)
        body = match.group("body").strip() + "\n"
        name = f"boot-{hashlib.sha256(body.encode('utf-8')).hexdigest()[:16]}.js"
        (dist / name).write_text(body, encoding="utf-8")
        moved += 1
        kind = ' type="module"' if 'type="module"' in match.group("attrs") else ""
        return f'<script{kind} src="{public_url}{name}"></script>'

    index.write_text(SCRIPT_RE.sub(replace, html), encoding="utf-8")
    return moved


def check(dist: Path) -> int:
    html = (dist / "index.html").read_text(encoding="utf-8")
    inline = [m for m in SCRIPT_RE.finditer(html) if _is_inline(m)]
    if inline:
        print(f"{len(inline)} inline script(s) remain in {dist / 'index.html'}", file=sys.stderr)
        return 1
    print(f"{dist / 'index.html'}: no inline scripts")
    return 0


def main(argv: list[str]) -> int:
    if argv and argv[0] == "--check":
        return check(Path(argv[1] if len(argv) > 1 else os.environ.get("TRUNK_DIST_DIR", "dist")))
    target = argv[0] if argv else os.environ.get("TRUNK_STAGING_DIR")
    if not target:
        print("usage: externalize_inline_scripts.py [--check] <dist-dir>", file=sys.stderr)
        return 2
    public_url = os.environ.get("TRUNK_PUBLIC_URL", "/")
    if not public_url.endswith("/"):
        public_url += "/"
    moved = externalize(Path(target), public_url)
    print(f"externalized {moved} inline script(s) in {target}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
