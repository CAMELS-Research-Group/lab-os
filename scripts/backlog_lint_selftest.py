#!/usr/bin/env python3
"""backlog-lint self-test: fixtures + assertions that prove each rule bites.

Split out of backlog_lint.py so the production module stays under the
Class-1 1,000-line budget (reference/code-quality-taxonomy.md). The single
entry point is still `python3 scripts/backlog_lint.py --self-test`, which
imports run_self_test() from here — the CLI contract is unchanged. Every
symbol under test is imported from the production module (one parser, one
grammar), so a mutation there is caught here.

Stdlib only; Python 3.11+.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

from backlog_lint import (
    _DEFAULT_TEMPLATE,
    _write_index,
    lint,
    main,
    parse_backlog,
    render_index,
    required_fields,
)


_GOOD = """# Fixture backlog

## Index

| id | title | owner | size | status |
|---|---|---|---|---|
| B1 | First | Kiara | S | ready |
| B2 | Second | Watson | M | in-progress |

## Inbox

- a raw idea waiting to be shaped

## Items

## B1 — First

- **Problem:** something is missing
- **Who it helps:** the team
- **Value:** worth it
- **Owner:** Kiara
- **Rough size:** S
- **Done when:** `scripts/x.py --self-test` passes
- **Depends on:** —
- **Status:** ready

## B2 — Second

- **Problem:** another gap
- **Who it helps:** groomers
- **Value:** compounding
- **Owner:** Watson
- **Rough size:** M
- **Done when:** the wrapped dashboard render lands in
  `docs/y.md` with the new section
- **Depends on:** B1
- **Status:** in-progress
"""


def _repo_template() -> Path:
    return Path(__file__).resolve().parent.parent / _DEFAULT_TEMPLATE


def run_self_test() -> int:  # noqa: C901 - a linear fixture list reads best flat
    ok = True

    # The schema comes from the real template — no hardcoded fallback list, so
    # a template edit that breaks the field regex fails HERE, loudly.
    req = required_fields(_repo_template().read_text(encoding="utf-8"))

    def expect(name: str, cond: bool, detail: str = "") -> None:
        nonlocal ok
        if not cond:
            ok = False
        print(f"  [{'ok' if cond else 'FAIL'}] {name}" + (f" -> {detail}" if not cond else ""))

    def check(name: str, text: str, want_err_substr: str | None,
              want_n: int | None = None) -> None:
        """want_err_substr=None asserts NO errors; want_n pins the exact error
        count where it is stable, so 'the rule bit' is distinguishable from
        'the rule bit plus collateral noise'."""
        errs = lint(parse_backlog(text), req).errors
        hit = any(want_err_substr in e for e in errs) if want_err_substr else not errs
        if want_n is not None:
            hit = hit and len(errs) == want_n
        expect(name, hit, str(errs))

    expect("template schema is non-empty (fail-closed source)",
           bool(req) and "Done when" in req and "Owner" in req, str(req))
    expect("required_fields ignores field lines inside HTML comments",
           required_fields("<!--\n- **Fake:** guidance\n-->\n- **Owner:** x\n") == ["Owner"],
           str(required_fields("<!--\n- **Fake:** guidance\n-->\n- **Owner:** x\n")))

    good = lint(parse_backlog(_GOOD), req)
    expect("good backlog is clean (no errors)", not good.errors, str(good.errors))
    check("clean fixture through check() (exercises its assert-clean arm)", _GOOD, None)
    expect("wrapped Done-when joins fully (no false concreteness warning)",
           not good.warnings, str(good.warnings))
    b2 = [i for i in parse_backlog(_GOOD).items if i.id == "B2"][0]
    expect("continuation line captured in parsed value",
           b2.fields["Done when"].endswith("`docs/y.md` with the new section"),
           b2.fields["Done when"])

    check("missing field", _GOOD.replace("- **Value:** worth it\n", ""), "missing required field 'Value'", want_n=1)
    check("placeholder Done when", _GOOD.replace("`scripts/x.py --self-test` passes", "<the check>"), "placeholder", want_n=1)
    check("multi-condition Done when (nested list)",
          _GOOD.replace("- **Done when:** `scripts/x.py --self-test` passes",
                        "- **Done when:** `scripts/x.py --self-test` passes\n  - and a second condition holds"),
          "single condition")
    check("bad status", _GOOD.replace("- **Status:** ready", "- **Status:** blocked"), "not one of")
    check("bad size token", _GOOD.replace("- **Rough size:** M", "- **Rough size:** XL"), "not one of")
    check("L marked ready", _GOOD.replace("- **Rough size:** S", "- **Rough size:** L"), "must be split")
    check("duplicate item id", _GOOD.replace("## B2 — Second", "## B1 — Second"), "duplicate item id")
    check("hand-edited Index row (derived-index diff)",
          _GOOD.replace("| B1 | First | Kiara | S | ready |", "| B1 | INDEX TITLE | Kiara | S | ready |"),
          "Index is stale")
    check("Item without an Index row (derived-index diff)",
          _GOOD.replace("| B2 | Second | Watson | M | in-progress |\n", ""), "Index is stale")
    check("orphan Index row (derived-index diff)",
          _GOOD.replace("| B2 | Second | Watson | M | in-progress |\n",
                        "| B2 | Second | Watson | M | in-progress |\n| B9 | Ghost | Kiara | S | ready |\n"),
          "Index is stale")
    check("dangling depends", _GOOD.replace("- **Depends on:** B1", "- **Depends on:** B7"), "not a backlog item", want_n=1)
    check("'Depends on: TBD' is an error, not silently no-deps",
          _GOOD.replace("- **Depends on:** B1", "- **Depends on:** TBD"),
          "neither a no-deps sentinel", want_n=1)
    check("lowercase dep id ('b1') is an error, not silently no-deps",
          _GOOD.replace("- **Depends on:** B1", "- **Depends on:** b1"),
          "neither a no-deps sentinel", want_n=1)
    check("dependency cycle", _GOOD.replace("- **Depends on:** —", "- **Depends on:** B2"), "cycle", want_n=1)
    check("private path leak", _GOOD.replace("something is missing", "see /Users/someone/secret/x"), "non-public", want_n=1)
    check("Linux home-path leak (/home/... — the CI runner's own prefix)",
          _GOOD.replace("something is missing", "see /home/runner/work/x"), "non-public", want_n=1)
    check("leak in an item heading title is caught",
          _GOOD.replace("## B1 — First", "## B1 — First (in /Users/kiara/notes/)"),
          "non-public")
    # Characterization of the tripwire's documented misses (docstring): these
    # three MUST NOT match, so the scan's boundary is visible from the suite.
    for miss_name, miss_text in (
        ("private github.com URL is (documented) not caught",
         "see https://github.com/example-org/example-notes-repo/issues/9"),
        ("gated-dataset prefix is (documented) not caught",
         "needs mimic-iv-2.2/hosp access first"),
        ("Windows home path is (documented) not caught",
         r"see C:\Users\watson\notes"),
    ):
        miss_errs = lint(parse_backlog(_GOOD.replace("something is missing", miss_text)), req).errors
        expect(miss_name, not any("non-public" in e for e in miss_errs), str(miss_errs))
    check("private-repo phrase leak", _GOOD.replace("another gap", "tracked in the private repo"), "non-public")
    check("gated-dataset phrase leak", _GOOD.replace("worth it", "unblocks the gated dataset work"), "non-public")
    check("leak on a continuation line is caught",
          _GOOD.replace("  `docs/y.md` with the new section", "  `docs/y.md` under /Users/someone/private/"),
          "non-public")
    check("Inbox leak is caught",
          _GOOD.replace("- a raw idea waiting to be shaped", "- idea parked at /Users/someone/notes/idea.md"),
          "Inbox")
    # Blocker A (PR #67 adversarial self-review): a stray `## …` heading inside
    # Inbox must NOT silently exit the section and drop every line below it out
    # of BOTH the Inbox and orphan leak scans. Test PAIR — (1) a private
    # reference one line below a stray `## Notes` heading in Inbox still ERRORS;
    # (2) the same leak with no stray heading above it (control) also errors, so
    # the fix is not smuggling the signal in via the heading. Mutation-check:
    # revert the Inbox branch to `section = ""` and (1) goes green (no error).
    check("leak below a stray `## ` heading in Inbox is still caught (Blocker A)",
          _GOOD.replace("- a raw idea waiting to be shaped",
                        "- a raw idea waiting to be shaped\n\n## Notes\n\n"
                        "- see /Users/watson/private/keys.txt"),
          "non-public")
    check("Inbox leak with no stray heading above it (Blocker A control)",
          _GOOD.replace("- a raw idea waiting to be shaped",
                        "- see /Users/watson/private/keys.txt"),
          "non-public")
    # The fix must not false-positive: a stray `## ` heading in Inbox with clean
    # content below stays green, and the legit Inbox→Items transition survives.
    check("stray `## ` heading in Inbox with clean content is not a false positive",
          _GOOD.replace("- a raw idea waiting to be shaped",
                        "- a raw idea waiting to be shaped\n\n## Notes\n\n"
                        "- a second clean idea"),
          None)

    # Structural windows (adversarial self-review, PR #67): malformed headings
    # and unattached text fail loudly — nothing mis-attaches or vanishes.
    bad_head = _GOOD.replace("## B2 — Second", "## B2 - Second")
    check("malformed item heading (hyphen, not em-dash) is a hard error",
          bad_head, "unrecognized item heading")
    parsed_bad_head = parse_backlog(bad_head)
    expect("fields after a malformed heading do not attach to the previous item",
           [i.id for i in parsed_bad_head.items] == ["B1"]
           and parsed_bad_head.items[0].fields["Problem"] == "something is missing",
           str([(i.id, i.fields) for i in parsed_bad_head.items]))
    check("verbatim template heading (## <id> — ...) is an unrecognized heading",
          _GOOD + "\n## <id> — <one-line title>\n\n- **Problem:** <x>\n",
          "unrecognized item heading")
    check("severed continuation (blank line) is unattached text",
          _GOOD.replace("- **Done when:** the wrapped dashboard render lands in\n  `docs/y.md` with the new section",
                        "- **Done when:** the wrapped dashboard render lands in `docs/y.md`\n\n  and the new section renders"),
          "unattached text under item B2")
    check("prose between heading and first field is unattached text",
          _GOOD.replace("## B1 — First\n\n- **Problem:**",
                        "## B1 — First\n\nstray prose no field owns\n\n- **Problem:**"),
          "unattached text under item B1")
    check("leak scan reaches unattached text",
          _GOOD.replace("- **Done when:** the wrapped dashboard render lands in\n  `docs/y.md` with the new section",
                        "- **Done when:** the wrapped dashboard render lands in `docs/y.md`\n\n  parked at /Users/someone/private/notes.md"),
          "non-public")
    # Blocker 1 (PR #67 review): the two `cur is None` windows. The four cases
    # above all run with an item OPEN; these two run with no item open, which
    # is exactly where the old `continue` silently dropped text.
    check("text between ## Items and the first item heading is a hard error",
          _GOOD.replace("## Items\n", "## Items\n\nstray note above the first item\n"),
          "unattached text in Items section")
    check("leak scan reaches text before the first item heading",
          _GOOD.replace("## Items\n",
                        "## Items\n\nstray note referencing /Users/watson/private/x.md\n"),
          "non-public")
    check("field lines under a malformed heading are recorded, not dropped",
          _GOOD.replace("## B2 — Second", "## B2 - Second"),
          "unattached text in Items section")
    check("leak scan reaches field lines under a malformed heading",
          _GOOD.replace("## B2 — Second", "## B2 - Second")
               .replace("another gap", "see /Users/someone/private/notes.md"),
          "non-public")

    # Whole-file leak scan (PR #67 re-review, Watson Imp 2): the scan must
    # reach EVERY otherwise-unowned line, not just Items + Inbox. Before the
    # fix, prose in the preamble, a cell in the `## Index` region, and text
    # under a stray heading escaped every scan, so a private/gated reference
    # there went GREEN under --check --enforce. Each region now errors.
    # Mutation-check: drop the `orphans.append` in parse_backlog's
    # section != "## Items" fall-through and all three go green.
    check("leak in the preamble (before ## Index) is caught",
          "# Backlog\n\nparked at /Users/someone/private/notes.md\n" + _GOOD,
          "non-public")
    check("leak in the ## Index region is caught",
          _GOOD.replace("| B1 | First | Kiara | S | ready |",
                        "| B1 | First (see /Users/someone/private/x) | Kiara | S | ready |"),
          "non-public")
    check("leak under a stray heading outside Items/Inbox is caught",
          # A `## Notes` heading + leak in the ## Index region (section is
          # "## Index" here, so this is the new fall-through, not the existing
          # Items-section orphan path).
          _GOOD.replace("## Inbox",
                        "## Notes\n\nsee /home/runner/private/keys.txt\n\n## Inbox"),
          "non-public")
    # False-positive guard: the clean fixture's preamble/Index lines are now
    # leak-scanned too, and must stay green.
    check("whole-file scan adds no false positive on the clean fixture", _GOOD, None)

    # Placeholder / empty checks cover EVERY required field, not just Done when.
    check("placeholder in a non-Done-when required field",
          _GOOD.replace("- **Owner:** Kiara", "- **Owner:** <who is driving it>"),
          "required field 'Owner' is empty or a placeholder")
    check("empty required field value",
          _GOOD.replace("- **Value:** worth it", "- **Value:**"),
          "required field 'Value' is empty or a placeholder")
    verbatim = _GOOD + """
## B3 — Copied template, never filled

- **Problem:** <what is broken or missing, and for whom>
- **Who it helps:** <the person or workflow that benefits>
- **Value:** <why it is worth doing now rather than later>
- **Owner:** <who is driving it — required before an item leaves Inbox>
- **Rough size:** <S = one sitting · M = a few · L = a project — split anything L before it is ready>
- **Done when:** <the single observable check that proves it shipped — a command, a behaviour, a file>
- **Depends on:** <other item ids, or —>
- **Status:** <inbox = raw idea · ready = shaped and unblocked · in-progress · done>
"""
    tpl_errs = lint(parse_backlog(verbatim), req).errors
    expect("verbatim-template item: every placeholder field is named",
           all(any(f"'{k}'" in e and "placeholder" in e for e in tpl_errs) for k in req),
           str(tpl_errs))

    stale_errs = lint(parse_backlog(_GOOD.replace(
        "| B1 | First | Kiara | S | ready |",
        "| B1 | INDEX TITLE | Kiara | S | ready |")), req).errors
    expect("stale-index error names the first differing row",
           any("Index is stale" in e and "INDEX TITLE" in e for e in stale_errs),
           str(stale_errs))

    piped = parse_backlog(_GOOD.replace("## B1 — First", "## B1 — First | piped"))
    rendered_rows = render_index(piped.items).splitlines()
    expect("render_index escapes | in titles (row shape survives)",
           "First \\| piped" in rendered_rows[2]
           and all(len(re.split(r"(?<!\\)\|", row)) == 7 for row in rendered_rows),
           str(rendered_rows))

    # An escaped-pipe title round-trips through render → parse (PR #68 review):
    # `--write-index` escapes `|` as `\|` in the Index cell, and _parse_index
    # must split on unescaped pipes and unescape the cell — otherwise the row
    # over-splits and drops out of `index`, and the view/digest tools (which
    # read status/owner/title FROM `index`) disagree with lint on the same file.
    piped_written = _GOOD.replace("## B1 — First", "## B1 — First | piped") \
                         .replace("| B1 | First | Kiara | S | ready |",
                                  "| B1 | First \\| piped | Kiara | S | ready |")
    piped_bl = parse_backlog(piped_written)
    expect("escaped-pipe title: lint clean (render byte-matches committed row)",
           not lint(piped_bl, req).errors, str(lint(piped_bl, req).errors))
    expect("escaped-pipe Index row survives and unescapes (lint↔view agree)",
           any(r["id"] == "B1" and r["title"] == "First | piped"
               for r in piped_bl.index), str(piped_bl.index))

    # A second `- **Key:**` line in one item fails closed, not last-write-wins.
    check("duplicate field label under one item is a hard error",
          _GOOD.replace("- **Owner:** Kiara",
                        "- **Owner:** Kiara\n- **Owner:** Someone Else"),
          "duplicate field 'Owner'")

    # `raw` is a LIVE field: the full source text, verbatim (PR #68's
    # source-integrity guard reads `raw.strip()` to tell a truly-empty backlog
    # from one that parsed to zero Item blocks — truncated/destroyed source).
    expect("Backlog.raw carries the full verbatim source",
           parse_backlog(_GOOD).raw == _GOOD, "raw diverged from input")
    expect("raw.strip() distinguishes destroyed source (non-empty, zero items)",
           (lambda b: bool(b.raw.strip()) and not b.items)(
               parse_backlog("<<<<<<< HEAD\nwreck\n=======\nother\n>>>>>>> x\n")),
           "destroyed-source signal not detectable via raw")

    hijacked = parse_backlog(_GOOD.replace(
        "- **Problem:** something is missing",
        "- **__title__:** HIJACKED\n- **Problem:** something is missing"))
    expect("a '__title__' field line cannot forge the rendered title",
           hijacked.items[0].title == "First"
           and "HIJACKED" not in render_index(hijacked.items),
           render_index(hijacked.items))

    # CLI exit-code contract through main()/_run (the Phase-2 flip surface).
    # Fixture main() calls run with stdout captured so their deliberate
    # findings do not land as stray ::error annotations on a real CI job.
    import contextlib
    import io
    import tempfile

    def run_main(args: list[str]) -> tuple[int, str]:
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            rc = main(args)
        return rc, buf.getvalue()

    # Pin GITHUB_ACTIONS so the annotation contract is exercised regardless of
    # where the self-test runs: in the shipped warn-only posture the
    # ::error / ::warning prefixes are the ONLY signal that reaches a PR.
    import os
    prev_ga = os.environ.get("GITHUB_ACTIONS")
    os.environ["GITHUB_ACTIONS"] = "true"

    with tempfile.TemporaryDirectory() as td:
        tdp = Path(td)
        good_b, bad_b = tdp / "GOOD.md", tdp / "BAD.md"
        good_b.write_text(_GOOD, encoding="utf-8")
        bad_b.write_text(_GOOD.replace("`scripts/x.py --self-test` passes", "<the check>"), encoding="utf-8")
        tpl = str(_repo_template())
        rc, out = run_main(["--check", str(good_b), "--template", tpl])
        expect("main: clean backlog exits 0", rc == 0, out)
        rc, out = run_main(["--check", str(bad_b), "--template", tpl])
        expect("main: errors without --enforce exit 0 (warn-only)", rc == 0, out)
        expect("CI errors annotate as ::error anchored to file and line",
               f"::error file={bad_b},line=" in out, out)
        expect("main: errors with --enforce exit 1",
               run_main(["--check", str(bad_b), "--template", tpl, "--enforce"])[0] == 1)
        # Imp D (PR #67 review): --require-backlog makes a MISSING BACKLOG a
        # hard error under --enforce. lab-os ships one and passes the flag;
        # member repos leave it off and no-op green. The flag is off by default.
        missing_b = tdp / "NO-SUCH-BACKLOG.md"
        expect("missing backlog, flag absent: no-op green (member-repo path intact)",
               run_main(["--check", str(missing_b), "--template", tpl])[0] == 0)
        expect("missing backlog, --require-backlog without --enforce: warn-only green",
               run_main(["--check", str(missing_b), "--template", tpl,
                         "--require-backlog"])[0] == 0)
        rc, out = run_main(["--check", str(missing_b), "--template", tpl,
                            "--require-backlog", "--enforce"])
        expect("missing backlog, --require-backlog --enforce: hard error exit 1",
               rc == 1 and "--require-backlog" in out, out)
        # A PRESENT-but-broken backlog still reports structurally under
        # --require-backlog (the flag fires only on a truly-absent file, so it
        # never masks a present file's structural findings).
        rc, out = run_main(["--check", str(bad_b), "--template", tpl,
                            "--require-backlog", "--enforce"])
        expect("present-but-broken backlog under --require-backlog reports structurally",
               rc == 1 and "placeholder" in out, out)
        warn_b = tdp / "WARNONLY.md"
        warn_b.write_text(_GOOD.replace("`scripts/x.py --self-test` passes",
                                        "the team agrees it works"), encoding="utf-8")
        rc, out = run_main(["--check", str(warn_b), "--template", tpl])
        expect("CI warnings annotate as ::warning (never ::error), exit 0",
               rc == 0 and f"::warning file={warn_b}" in out and "::error" not in out,
               out)
        expect("main: missing template exits 1 (fail closed)",
               run_main(["--check", str(good_b), "--template", str(tdp / "no-such.md")])[0] == 1)
        empty_tpl = tdp / "empty-template.md"
        empty_tpl.write_text("# a template with no field lines\n", encoding="utf-8")
        expect("main: template with zero fields exits 1 (fail closed)",
               run_main(["--check", str(good_b), "--template", str(empty_tpl)])[0] == 1)
        renamed_tpl = tdp / "renamed-label-template.md"
        renamed_tpl.write_text(_repo_template().read_text(encoding="utf-8").replace("- **Status:**", "- **State:**"), encoding="utf-8")
        rc, out = run_main(["--check", str(good_b), "--template", str(renamed_tpl)])
        expect("main: template with a renamed semantic label exits 1, names it",
               rc == 1 and "Status" in out, out)
        # --write-index round-trip: hand-broken Index regenerates to clean.
        stale = tdp / "STALE.md"
        stale.write_text(_GOOD.replace("| B1 | First | Kiara | S | ready |",
                                       "| B1 | WRONG | Nobody | L | done |"), encoding="utf-8")
        expect("--write-index repairs a stale Index",
               _write_index(stale) == 0
               and not lint(parse_backlog(stale.read_text(encoding="utf-8")), req).errors)
        piped_b = tdp / "PIPED.md"
        piped_b.write_text(_GOOD.replace("## B1 — First", "## B1 — First | piped"), encoding="utf-8")
        expect("--write-index handles | in a title; result lints clean",
               _write_index(piped_b) == 0
               and not lint(parse_backlog(piped_b.read_text(encoding="utf-8")), req).errors)
        # Blocker 2 (PR #67 review): --write-index must refuse on structural
        # parse errors instead of silently deleting the unparsed blocks' rows.
        mal = tdp / "MALFORMED.md"
        mal_text = _GOOD.replace("## B2 — Second", "## B2 - Second")
        mal.write_text(mal_text, encoding="utf-8")
        rc, out = run_main(["--check", str(mal), "--write-index"])
        expect("--write-index on a structurally broken backlog exits non-zero",
               rc == 1, out)
        expect("--write-index refusal names the structural error",
               "refusing to regenerate" in out
               and "unrecognized item heading" in out, out)
        expect("--write-index leaves the broken file byte-identical",
               mal.read_text(encoding="utf-8") == mal_text)
        # Blocker 1 (this fix, PR #67 review — zero-parsed-items --write-index
        # destruction): when `## Items` is renamed/misspelled/absent the source
        # parses to zero items. Pre-fix that produced ZERO parse errors, so
        # --write-index rendered an empty table, wiped every committed Index row
        # and reported success, and --check then went GREEN on the header-only
        # file. Fix B (item heading outside Items → parse_error) makes --check
        # ERROR and, via the existing parse_errors refusal, makes --write-index
        # refuse; Fix A (zero items + committed rows → refuse) is the belt for
        # the genuinely-empty-Items case that carries no parse error.
        renamed = tdp / "RENAMED-ITEMS.md"
        renamed_text = _GOOD.replace("## Items", "## Backlog items")
        renamed.write_text(renamed_text, encoding="utf-8")
        rc, out = run_main(["--check", str(renamed), "--write-index"])
        expect("renamed ## Items: --write-index refuses (non-zero)", rc == 1, out)
        expect("renamed ## Items: --write-index leaves the file byte-identical",
               renamed.read_text(encoding="utf-8") == renamed_text)
        rc, out = run_main(["--check", str(renamed), "--template", tpl, "--enforce"])
        expect("renamed ## Items: --check --enforce ERRORS, never green",
               rc == 1 and "item heading outside Items section" in out, out)
        # A genuinely-empty Items section (heading present, zero item blocks)
        # with a committed Index carrying rows: no parse error, so Fix A alone
        # must stop --write-index from wiping the committed rows.
        emptied = tdp / "EMPTY-ITEMS.md"
        emptied_text = _GOOD.split("## Items")[0] + "## Items\n"
        emptied.write_text(emptied_text, encoding="utf-8")
        expect("empty Items + committed rows: zero items, zero parse errors (Fix-A precondition)",
               (lambda b: not b.items and not b.parse_errors and len(b.index) == 2)(
                   parse_backlog(emptied_text)), str(parse_backlog(emptied_text).parse_errors))
        rc, out = run_main(["--check", str(emptied), "--write-index"])
        expect("empty Items + committed rows: --write-index refuses (Fix A)",
               rc == 1 and "parsed zero items" in out, out)
        expect("empty Items: --write-index leaves the file byte-identical",
               emptied.read_text(encoding="utf-8") == emptied_text)
        # Unreadable-template fixture: a DIRECTORY, not a chmod(0) file —
        # chmod(0) does not deny reads to root, so on a root-executing runner
        # (container jobs, some self-hosted docker runners) that variant would
        # false-red every PR. read_text() on a directory raises
        # IsADirectoryError (an OSError subclass), exercising the same
        # _load_schema branch with no privilege dependence.
        unread_tpl = tdp / "unreadable-template-dir"
        unread_tpl.mkdir()
        expect("main: unreadable template exits 1 (one-line failure, no traceback)",
               run_main(["--check", str(good_b), "--template", str(unread_tpl)])[0] == 1)
        # Encoding is pinned to UTF-8 on every read/write (PR #67 review
        # Blocker 1): the item-heading grammar REQUIRES an em-dash, so an
        # unpinned read under a locale codepage (cp1252 on Windows, ascii under
        # LC_ALL=C) mis-decodes `—` and every item vanishes / the file fails to
        # decode. This round-trip writes an em-dash file and reads it back
        # through the parser; under a non-UTF-8 locale it fails unless encoding
        # is pinned. Run the whole suite under `LC_ALL=C PYTHONUTF8=0` to prove
        # it end to end.
        emdash = tdp / "EMDASH.md"
        emdash.write_text(_GOOD, encoding="utf-8")
        round_tripped = parse_backlog(emdash.read_text(encoding="utf-8"))
        expect("em-dash headings survive a write→read round-trip (encoding pinned)",
               [it.id for it in round_tripped.items] == ["B1", "B2"]
               and round_tripped.items[0].title == "First",
               str([it.title for it in round_tripped.items]))

        # Blocker A (PR #67 re-review): the round-trip above reads through the
        # TEST's own `encoding="utf-8"`, so it exercises NOTHING about the
        # production script's output encoding. Under `LC_ALL=C PYTHONUTF8=0`
        # stdout defaults to ASCII, and every printed em-dash (findings, the
        # `--require-backlog` warning, argparse help, the fail-closed one-liner)
        # raised UnicodeEncodeError mid-run — inverting warn-only into a
        # false-red traceback. Prove closure end-to-end by running the REAL
        # script in a subprocess under that locale and asserting no output path
        # crashes. Mutation-check: remove _harden_stdout() (any em-dash print
        # crashes → cases 2/3 fail), or delete `encoding=` from a production
        # read (the em-dash file mis-decodes under C → case 1 loses "OK").
        import subprocess
        script = Path(__file__).resolve().parent / "backlog_lint.py"
        c_env = {k: v for k, v in os.environ.items()
                 if k not in ("PYTHONIOENCODING", "GITHUB_ACTIONS")}
        c_env.update(LC_ALL="C", LANG="C", PYTHONUTF8="0")

        def run_c(args: list[str]) -> tuple[int, str]:
            # Decode the child's output as UTF-8 explicitly: the child now
            # emits UTF-8 em-dashes, and this parent may itself be running
            # under LC_ALL=C (the whole suite is run that way to prove closure),
            # where text=True would otherwise decode the child with ASCII and
            # choke on its own success output.
            r = subprocess.run([sys.executable, str(script), *args],
                               capture_output=True, text=True,
                               encoding="utf-8", errors="replace", env=c_env)
            return r.returncode, r.stdout + r.stderr

        dep_bad = tdp / "DEPBAD.md"
        dep_bad.write_text(_GOOD.replace("- **Depends on:** B1",
                                         "- **Depends on:** TBD"), encoding="utf-8")

        def _clean(out: str) -> bool:
            return "Traceback" not in out and "UnicodeEncodeError" not in out

        # 1. A clean em-dash backlog decodes and prints under C locale: exit 0,
        #    "OK" — proves the PRODUCTION read decoded the em-dash headings
        #    (encoding pinned) and stdout printed without crashing.
        rc, out = run_c(["--check", str(emdash), "--template", tpl])
        expect("real script: clean em-dash backlog under LC_ALL=C exits 0, OK, no crash",
               rc == 0 and "OK" in out and _clean(out), out)
        # 2. An em-dash-bearing WARNING path (the --require-backlog message
        #    embeds `— this repo must ship`) prints and exits 0 under C locale.
        rc, out = run_c(["--check", str(tdp / "NO-SUCH.md"), "--require-backlog"])
        expect("real script: em-dash WARN under LC_ALL=C exits 0 clean (no traceback)",
               rc == 0 and "--require-backlog" in out and _clean(out), out)
        # 3. An em-dash-bearing ERROR finding under --enforce (`Depends on: TBD`
        #    → "(— / none)") prints and exits 1 under C locale — the warn→red
        #    finding path no longer crashes into a false-red traceback.
        rc, out = run_c(["--check", str(dep_bad), "--template", tpl, "--enforce"])
        expect("real script: em-dash ERROR under LC_ALL=C --enforce exits 1 clean",
               rc == 1 and "Depends on" in out and _clean(out), out)

    if prev_ga is None:
        os.environ.pop("GITHUB_ACTIONS", None)
    else:
        os.environ["GITHUB_ACTIONS"] = prev_ga

    print("backlog-lint self-test:", "PASS" if ok else "FAIL")
    return 0 if ok else 1

