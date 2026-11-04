#!/usr/bin/env python3
"""Writes `.br` (when the brotli CLI is installed) and `.gz` beside each
text asset of a web bundle, at the highest levels, for a server that sends
precompressed files (Caddy `file_server { precompressed }`). On a phone the
WASM download is the first paint: brotli 11 ships it about 28 percent
smaller than on-the-fly gzip.

Usage: precompress.py <dist-dir>
"""
import gzip
import pathlib
import shutil
import subprocess
import sys

TEXT_SUFFIXES = {".wasm", ".js", ".css", ".html", ".svg", ".webmanifest", ".txt", ".json"}


def main(dist: pathlib.Path) -> int:
    brotli = shutil.which("brotli")
    if brotli is None:
        print("precompress: brotli not installed, writing gzip only", file=sys.stderr)
    count = 0
    for path in sorted(dist.rglob("*")):
        if not path.is_file() or path.suffix not in TEXT_SUFFIXES:
            continue
        data = path.read_bytes()
        path.with_name(path.name + ".gz").write_bytes(gzip.compress(data, compresslevel=9, mtime=0))
        if brotli:
            subprocess.run([brotli, "-q", "11", "-f", "-o", f"{path}.br", str(path)], check=True)
        count += 1
    print(f"precompress: {count} files in {dist}")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2 or not pathlib.Path(sys.argv[1]).is_dir():
        sys.exit(__doc__)
    sys.exit(main(pathlib.Path(sys.argv[1])))
