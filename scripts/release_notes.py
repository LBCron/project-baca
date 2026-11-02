#!/usr/bin/env python3
"""Release version and notes from one source: the workspace version in
Cargo.toml and its section in CHANGELOG.md.

Usage:
  release_notes.py --check-version   the workspace version has a CHANGELOG section
  release_notes.py vX.Y.Z            the tag names the workspace version; prints its notes

Standard library only (tomllib, Python 3.11+).
"""
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def workspace_version() -> str:
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    return cargo["workspace"]["package"]["version"]


def changelog_section(version: str) -> str | None:
    text = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    heading = rf"^## \[{re.escape(version)}\][^\n]*\n"
    found = re.search(heading + r"(.*?)(?=^## \[|\Z)", text, re.DOTALL | re.MULTILINE)
    return found.group(1).strip() if found else None


def main(argv: list[str]) -> int:
    if len(argv) != 1:
        print(__doc__, file=sys.stderr)
        return 2
    version = workspace_version()
    notes = changelog_section(version)
    if notes is None:
        print(f"CHANGELOG.md has no '## [{version}]' section for the workspace version", file=sys.stderr)
        return 1
    if argv[0] == "--check-version":
        print(f"workspace version {version} has release notes")
        return 0
    if argv[0] != f"v{version}":
        print(f"tag {argv[0]} does not name the workspace version v{version}", file=sys.stderr)
        return 1
    print(notes)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
