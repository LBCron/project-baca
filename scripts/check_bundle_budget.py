#!/usr/bin/env python3
"""Fail when the release WebAssembly bundle outgrows its size budget.

Usage: check_bundle_budget.py <dist dir>

The budget sits above the measured bundle (1.57 MB raw, 608 KB gzip in
0.1.0 with the `wasm-release` profile and `wasm-opt -Oz`), so a feature can
land, but an accidental dependency or a lost optimisation pass fails the
build. Standard library only.
"""
import gzip
import sys
from pathlib import Path

RAW_LIMIT = 1_800_000
GZIP_LIMIT = 700_000


def main(dist: Path) -> int:
    bundles = sorted(dist.glob("*_bg.wasm"))
    if len(bundles) != 1:
        print(f"expected one *_bg.wasm in {dist}, found {len(bundles)}", file=sys.stderr)
        return 2
    data = bundles[0].read_bytes()
    raw, packed = len(data), len(gzip.compress(data, compresslevel=9))
    print(f"{bundles[0].name}: {raw:,} B raw (budget {RAW_LIMIT:,}), {packed:,} B gzip (budget {GZIP_LIMIT:,})")
    if raw > RAW_LIMIT or packed > GZIP_LIMIT:
        print("WebAssembly bundle is over budget", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    sys.exit(main(Path(sys.argv[1])))
