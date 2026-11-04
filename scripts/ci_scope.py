#!/usr/bin/env python3
"""Decide which CI jobs a run needs, and write `code=` and `web=` to
$GITHUB_OUTPUT (or stdout).

A merge commit of a pull request was already tested in that pull request,
so its push to develop skips the heavy jobs; documentation alone never
needs them; the browser job runs only when the web bundle can change.
Pull requests to main, pushes to main, manual runs, and a nightly run on a
develop that moved keep the full suite.

    python3 scripts/ci_scope.py            # inside GitHub Actions
    python3 scripts/ci_scope.py --self-test
"""
import os
import re
import subprocess
import sys

DOCS = re.compile(r"(\.md$|^LICENSE$|^knowledge/)")
WEB = re.compile(
    r"^(crates/web/|crates/shared/|Cargo\.(toml|lock)$|\.github/|Makefile$|scripts/trunk\.sha256$)"
)
FULL = {"code": True, "web": True}
NONE = {"code": False, "web": False}


def classify(files):
    code = any(not DOCS.search(f) for f in files)
    return {"code": code, "web": code and any(WEB.search(f) for f in files)}


def git(*args):
    return subprocess.run(
        ["git", *args], check=True, capture_output=True, text=True
    ).stdout


def changed(rev_range):
    return [line for line in git("diff", "--name-only", rev_range).splitlines() if line]


def scope(env, changed=changed, git=git):
    event = env.get("EVENT", "")
    if event == "pull_request":
        base = env.get("BASE_REF", "")
        return FULL if base == "main" else classify(changed(f"origin/{base}...HEAD"))
    if event == "push":
        if env.get("REF") == "refs/heads/main":
            return FULL
        if env.get("HEAD_MESSAGE", "").startswith("Merge pull request #"):
            return NONE
        before = env.get("BEFORE", "")
        if not before or set(before) == {"0"}:
            return FULL
        return classify(changed(f"{before}..HEAD"))
    if event == "schedule":
        return FULL if git("log", "-1", "--since=25 hours ago", "--format=%H").strip() else NONE
    return FULL


def self_test():
    no_git = lambda *a: ""
    assert classify(["README.md", "docs/x.md"]) == NONE
    assert classify(["python_worker/worker.py"]) == {"code": True, "web": False}
    assert classify(["crates/server/src/lib.rs"]) == {"code": True, "web": False}
    assert classify(["crates/web/src/app.rs", "GUIDE.md"]) == FULL
    assert classify(["Cargo.lock"]) == FULL
    assert classify([]) == NONE

    files = lambda names: (lambda _range: names)
    pr = {"EVENT": "pull_request", "BASE_REF": "develop"}
    assert scope(pr, changed=files(["CHANGELOG.md"])) == NONE
    assert scope(pr, changed=files(["crates/shared/src/lib.rs"])) == FULL
    assert scope({"EVENT": "pull_request", "BASE_REF": "main"}, changed=files([])) == FULL

    push = {"EVENT": "push", "REF": "refs/heads/develop", "BEFORE": "abc"}
    merge = dict(push, HEAD_MESSAGE="Merge pull request #141 from aprxty3/m2/keputusan")
    assert scope(merge, changed=files(["crates/web/src/app.rs"])) == NONE
    assert scope(push, changed=files(["scripts/migrate.sh"])) == {"code": True, "web": False}
    assert scope(dict(push, BEFORE="0" * 40), changed=files([])) == FULL
    assert scope(dict(merge, REF="refs/heads/main"), changed=files([])) == FULL

    assert scope({"EVENT": "schedule"}, git=lambda *a: "abc\n") == FULL
    assert scope({"EVENT": "schedule"}, git=no_git) == NONE
    assert scope({"EVENT": "workflow_dispatch"}) == FULL
    print("ci_scope self-test passed")


def main():
    if sys.argv[1:] == ["--self-test"]:
        self_test()
        return
    result = scope(os.environ)
    lines = "".join(f"{k}={str(v).lower()}\n" for k, v in result.items())
    out = os.environ.get("GITHUB_OUTPUT")
    if out:
        with open(out, "a", encoding="utf-8") as fh:
            fh.write(lines)
    sys.stdout.write(lines)


if __name__ == "__main__":
    main()
