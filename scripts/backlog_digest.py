#!/usr/bin/env python3
"""backlog-digest: the sprint-boundary grooming digest from BACKLOG.md.

Monday-style notification, git-native: emits a markdown digest of what the team
should look at when it grooms the backlog —

  - Ready, needs an owner   (status ready, owner blank)
  - In progress, re-check "Done when"   (status in-progress)
  - Size L, split before ready   (size L)
  - Ready & unblocked   (status ready, every dependency done)

Status / owner / size / title come from the **Index table**, dependencies from
the Item blocks' `Depends on` fields — safe to mix because backlog-lint
enforces the Index as a byte-exact render of the Item blocks.

The scheduled workflow writes this to the Actions run summary every sprint
boundary, and — only when explicitly armed — posts/updates a GitHub issue. This
script just renders the text; the surface is the workflow's job.

Reuses parse_backlog + parse_deps from backlog_lint (one parser, one
dependency grammar — not copies). Stdlib only; Python 3.11+.
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from backlog_lint import Backlog, parse_backlog, parse_deps  # noqa: E402


def _blank_owner(o: str) -> bool:
    return o.strip() in ("", "—", "-", "TBD", "tbd")


def render_digest(backlog: Backlog) -> str:
    status = {r["id"]: r["status"] for r in backlog.index}
    deps = {it.id: parse_deps(it.fields.get("Depends on", "—")) for it in backlog.items}

    def line(r: dict[str, str]) -> str:
        return f"- {r['id']} — {r['title']} ({r['owner']}, {r['size']})"

    ready_no_owner = [r for r in backlog.index if r["status"] == "ready" and _blank_owner(r["owner"])]
    in_progress = [r for r in backlog.index if r["status"] == "in-progress"]
    size_l = [r for r in backlog.index if r["size"].upper().startswith("L")]
    unblocked = [r for r in backlog.index
                 if r["status"] == "ready"
                 and all(status.get(d) == "done" for d in deps.get(r["id"], []))]

    out = ["# Backlog grooming digest", "",
           "_Sprint-boundary grooming surface, generated from `BACKLOG.md`._", ""]

    def section(title: str, rows: list[dict[str, str]], empty: str) -> None:
        out.append(f"## {title}")
        if rows:
            out.extend(line(r) for r in rows)
        else:
            out.append(f"_{empty}_")
        out.append("")

    section("Ready — needs an owner", ready_no_owner, "none")
    section("In progress — re-check “Done when”", in_progress, "none")
    section("Size L — split before it can be ready", size_l, "none")
    section("Ready & unblocked — pick these up", unblocked, "none right now")
    return "\n".join(out).rstrip() + "\n"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="Render the backlog grooming digest.")
    ap.add_argument("--backlog", default="BACKLOG.md")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args(argv)
    if args.self_test:
        return _self_test()
    print(render_digest(parse_backlog(Path(args.backlog).read_text())))
    return 0


# The self-test fixture lives at tests/backlog_digest/BACKLOG.md — the house
# pattern (cf. tests/log_lint/, tests/merge_bar_check/, tests/docs_budget/):
# a reviewable, diffable file rather than an in-module literal. It is
# lint-valid (zero errors — asserted in the self-test): every Item block
# carries the full template schema, and the Index is the byte-exact render of
# the blocks. B5/B6 differ only in whether their dependency is done, which is
# what exercises the unblocked predicate.
_FIXTURE = Path(__file__).resolve().parent.parent / "tests/backlog_digest/BACKLOG.md"


def _fixture() -> str:
    return _FIXTURE.read_text()


def _self_test() -> int:
    from backlog_lint import lint, required_fields

    ok = True

    def expect(name: str, cond: bool) -> None:
        nonlocal ok
        print(f"  [{'ok' if cond else 'FAIL'}] {name}")
        ok = ok and cond

    fix = _fixture()

    # The fixture must model a real input: backlog_lint accepts it with zero
    # errors (full schema, Index == render of the blocks).
    template = Path(__file__).resolve().parent.parent / "templates/backlog-item.template.md"
    errs = lint(parse_backlog(fix), required_fields(template.read_text())).errors
    expect("fixture is lint-valid (zero errors)", not errs)

    d = render_digest(parse_backlog(fix))

    def sec(title: str) -> str:
        return d.split(f"## {title}")[1].split("## ")[0]

    needs_owner = sec("Ready — needs an owner")
    in_progress = sec("In progress")
    size_l = sec("Size L")
    unblocked = sec("Ready & unblocked")

    expect("orphan-ready B2 flagged needs-owner", "B2" in needs_owner)
    expect("owned-ready B5 not in needs-owner", "B5" not in needs_owner)
    expect("in-progress B3 flagged", "B3" in in_progress)
    expect("size-L B4 flagged", "B4" in size_l)
    # Dependency resolution: done dep -> unblocked; not-done dep -> excluded.
    expect("B5 unblocked (dep B1 is done)", "B5" in unblocked)
    expect("no-dep ready B2 unblocked", "B2" in unblocked)
    expect("B6 NOT unblocked (dep B3 not done)", "B6" not in unblocked)
    expect("non-ready B3/B4 not in unblocked", "B3" not in unblocked and "B4" not in unblocked)

    print("backlog-digest self-test:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
