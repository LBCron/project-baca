#!/usr/bin/env python3
"""Adds `<link rel="preconnect">` for the API origin to index.html when the
bundle is built against an API on another origin (API_BASE_URL absolute),
so the browser opens that connection while the WASM downloads instead of
after it. A relative base (the edge image, same origin) needs none.

Runs as a Trunk `post_build` hook on TRUNK_STAGING_DIR, or on the directory
given as the first argument. Standard library only.
"""
import html
import os
import sys
from pathlib import Path
from urllib.parse import urlsplit


def origin_of(base: str) -> str | None:
    parts = urlsplit(base)
    if parts.scheme in ("http", "https") and parts.netloc:
        return f"{parts.scheme}://{parts.netloc}"
    return None


def add_preconnect(dist: Path, base: str) -> bool:
    origin = origin_of(base)
    index = dist / "index.html"
    page = index.read_text(encoding="utf-8")
    if origin is None or f'rel="preconnect" href="{origin}"' in page:
        return False
    link = f'<link rel="preconnect" href="{html.escape(origin)}" crossorigin>'
    index.write_text(page.replace("</head>", link + "</head>", 1), encoding="utf-8")
    return True


if __name__ == "__main__":
    target = sys.argv[1] if len(sys.argv) > 1 else os.environ.get("TRUNK_STAGING_DIR")
    if not target:
        sys.exit("usage: preconnect_api.py <dist-dir>")
    added = add_preconnect(Path(target), os.environ.get("API_BASE_URL", ""))
    print("preconnect to the API origin added" if added else "preconnect: nothing to add")
