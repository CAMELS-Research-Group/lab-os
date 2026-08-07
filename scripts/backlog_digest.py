#!/usr/bin/env python3
"""backlog-digest: the sprint-boundary grooming digest from BACKLOG.md.

Monday-style notification, git-native: emits a markdown digest of what the team
should look at when it grooms the backlog —

  - Ready, needs an owner   (status ready, owner blank)
  - In progress, re-check "Done when"   (status in-progress)
  - Size L, split before ready   (size L)
  - Ready & unblocked   (status ready, every dependency done)

Status / owner / size / title come from the **Index table**, dependencies from
the Item blocks' `Depends on` fields. backlog-lint's byte-exact Index check
runs warn-only until its enforce flip, so the mix is NOT assumed safe: before
rendering, the source-integrity guard shared with backlog_view.py refuses —
exit 1 — any backlog with parse errors, zero Item blocks despite non-empty
text, or Index rows drifting from the Item blocks. A digest that reads
"nothing to groom" because the source was wreckage is the failure mode the
guard exists to prevent. A missing backlog file is a clean one-line error,
exit 1 (the scheduled workflow does no exit-code branching; any non-zero
fails it).

The scheduled workflow writes this to the Actions run summary every sprint
boundary, and — only when explicitly armed — posts/updates a GitHub issue. This
script just renders the text; the surface is the workflow's job.

Reuses parse_backlog from backlog_lint (one parser, one dependency grammar —
not copies) and ready_unblocked + source_integrity_errors from backlog_view
(one predicate, one guard — their long-term home is backlog_lint beside the
parser, kept out of it here so this branch does not fork PR #67's file).
Stdlib only; Python 3.11+.
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from backlog_lint import Backlog, parse_backlog  # noqa: E402
from backlog_view import _in_ci, ready_unblocked, source_integrity_errors  # noqa: E402


def _blank_owner(o: str) -> bool:
    return o.strip() in ("", "—", "-", "TBD", "tbd")


def _size_token(r: dict[str, str]) -> str:
    """First whitespace token of the size cell — backlog_lint's convention
    (`split()[0]` checked against SIZES), not a prefix match: "Large-ish"
    must not count as L."""
    parts = r["size"].split()
    return parts[0] if parts else ""


def render_digest(backlog: Backlog) -> str:
    def line(r: dict[str, str]) -> str:
        return f"- {r['id']} — {r['title']} ({r['owner']}, {r['size']})"

    ready_no_owner = [r for r in backlog.index if r["status"] == "ready" and _blank_owner(r["owner"])]
    in_progress = [r for r in backlog.index if r["status"] == "in-progress"]
    size_l = [r for r in backlog.index if _size_token(r) == "L"]
    unblocked = ready_unblocked(backlog)

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
    path = Path(args.backlog)
    if not path.exists():
        # Clean one-line error, matching backlog_lint's missing-file report —
        # not a raw FileNotFoundError traceback.
        msg = f"{path} not found"
        print(f"::error::backlog-digest: {msg}" if _in_ci() else f"ERROR {msg}")
        return 1
    backlog = parse_backlog(path.read_text(encoding="utf-8"))
    errs = source_integrity_errors(backlog)
    if errs:
        # A broken source must not render as an empty "nothing to groom"
        # digest (run summary / standing issue). Guard shared with
        # backlog_view — see its docstring for the three refusal classes.
        for e in errs:
            print(f"::error::backlog-digest: {path}: {e}" if _in_ci()
                  else f"ERROR {path}: {e}")
        msg = f"{path} failed source integrity — refusing to render a digest"
        print(f"::error::backlog-digest: {msg}" if _in_ci() else f"ERROR {msg}")
        return 1
    print(render_digest(backlog))
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
    return _FIXTURE.read_text(encoding="utf-8")


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
    errs = lint(parse_backlog(fix), required_fields(template.read_text(encoding="utf-8"))).errors
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

    # Every documented blank-owner sentinel arm, not just the "—" the fixture
    # happens to use.
    for v in ("", "—", "-", "TBD", "tbd"):
        expect(f"_blank_owner({v!r}) is blank", _blank_owner(v))
    expect("_blank_owner('Kiara') is not blank", not _blank_owner("Kiara"))

    # Size is backlog_lint's token convention, not a prefix match.
    largish = fix.replace("| Arya | L |", "| Arya | Large-ish |") \
                 .replace("**Rough size:** L\n", "**Rough size:** Large-ish\n")
    expect("size filter is token-exact, not startswith('L')",
           "B4" not in render_digest(parse_backlog(largish))
           .split("## Size L")[1].split("## ")[0])

    # The CLI entry point — the argparse wiring and stdout path the scheduled
    # workflow runs, plus the fail-closed lanes main() owns.
    import contextlib
    import io
    import tempfile

    def run_main(path: Path) -> tuple[int, str]:
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            rc = main(["--backlog", str(path)])
        return rc, buf.getvalue()

    with tempfile.TemporaryDirectory() as td:
        src = Path(td) / "BACKLOG.md"
        src.write_text(fix, encoding="utf-8")
        rc, out_text = run_main(src)
        expect("CLI renders a valid backlog to stdout (exit 0)",
               rc == 0 and "# Backlog grooming digest" in out_text)
        rc, out_text = run_main(Path(td) / "absent.md")
        expect("missing backlog: clean one-line error, exit 1",
               rc == 1 and "not found" in out_text)
        # Destroyed source: must refuse, never print "nothing to groom".
        src.write_text("<<<<<<< HEAD\ntotal wreckage\n=======\nother side\n"
                       ">>>>>>> theirs\n", encoding="utf-8")
        rc, out_text = run_main(src)
        expect("destroyed source refuses a digest (exit 1)",
               rc == 1 and "refusing" in out_text
               and "grooming digest" not in out_text)
        # Index row with no Item block (drop B6's block, keep its row): the
        # old predicate advertised B6 as ready & unblocked despite
        # `Depends on: B3` (B3 is in-progress, not done).
        src.write_text(fix.split("## B6 —")[0], encoding="utf-8")
        rc, out_text = run_main(src)
        expect("Index/Item drift refuses a digest, names the id",
               rc == 1 and "B6" in out_text)
        # Blocker B (adversarial self-review): CELL-content drift through the
        # SHARED source_integrity_errors. B5's block says done; the committed
        # Index row still says ready; ids match. The old guard passed and the
        # digest read `ready` from the Index. The fixed shared guard refuses.
        cell_drift = fix.replace("- **Depends on:** B1\n- **Status:** ready",
                                 "- **Depends on:** B1\n- **Status:** done")
        src.write_text(cell_drift, encoding="utf-8")
        rc, out_text = run_main(src)
        expect("status cell drift refuses a digest (exit 1, shared guard)",
               rc == 1 and "refusing" in out_text
               and "grooming digest" not in out_text)
        drifted_bl = parse_backlog(cell_drift)
        expect("digest cell drift caught though id-sets MATCH (Blocker B)",
               source_integrity_errors(drifted_bl) != []
               and not ({r["id"] for r in drifted_bl.index}
                        ^ {it.id for it in drifted_bl.items}))
        # `Depends on` fail-open through the SHARED guard (Watson's #68
        # blockers): B6 genuinely depends on B3 (in-progress, not done). A
        # malformed value made parse_deps read [] and the digest advertise B6
        # under "Ready & unblocked — pick these up". The shared
        # source_integrity_errors now refuses. B6's line is unique (B3 uses
        # `Depends on: —`). Mutation-check: drop the dep loop in
        # source_integrity_errors and this goes green (a digest renders B6).
        prose_dep = fix.replace("- **Depends on:** B3\n- **Status:** ready",
                                "- **Depends on:** blocked on Jean\n- **Status:** ready")
        src.write_text(prose_dep, encoding="utf-8")
        rc, out_text = run_main(src)
        expect("malformed Depends-on refuses a digest (exit 1, shared guard)",
               rc == 1 and "refusing" in out_text
               and "grooming digest" not in out_text and "B6" in out_text)
        # No false positive: the clean fixture (all deps well-formed) still
        # renders, and B5 (well-formed `Depends on: B1`, B1 done) is unblocked.
        expect("well-formed deps: clean fixture still passes the shared guard",
               source_integrity_errors(parse_backlog(fix)) == [])

    print("backlog-digest self-test:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
