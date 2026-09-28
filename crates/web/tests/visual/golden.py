"""Golden image helpers (Task 11f).

Python Playwright here has no `to_have_screenshot`; compare manually with
Pillow: first run writes goldens, later runs diff with tolerance. Update
goldens deliberately by deleting the file, then REVIEW the new image.
"""
from pathlib import Path

from PIL import Image, ImageChops
from playwright.sync_api import Page

SNAP_DIR = Path(__file__).parent / "test_visual.py-snapshots"
SNAP_DIR.mkdir(exist_ok=True)

# Max differing pixels before failing (font-smoothing tolerance).
MAX_DIFF_PIXELS = 100


def assert_golden(page: Page, name: str) -> None:
    # Stabilize font rendering: golden flakiness comes from screenshots
    # taken mid font-swap (same pixels, different subpixel coverage).
    page.evaluate("document.fonts.ready.then(() => true)")
    page.wait_for_timeout(400)
    golden = SNAP_DIR / name
    if not golden.exists():
        page.screenshot(path=str(golden))
        return
    actual = SNAP_DIR / f".{name}.actual.png"
    page.screenshot(path=str(actual))
    a = Image.open(golden).convert("RGB")
    b = Image.open(actual).convert("RGB")
    if a.size != b.size:
        raise AssertionError(f"viewport size changed: {a.size} vs {b.size}")
    diff = ImageChops.difference(a, b)
    n = sum(1 for px in diff.getdata() if px != (0, 0, 0))
    if n <= MAX_DIFF_PIXELS:
        actual.unlink(missing_ok=True)
        return
    raise AssertionError(f"{name}: {n} differing pixels (review .{name}.actual.png)")
