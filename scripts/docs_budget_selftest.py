#!/usr/bin/env python3
"""docs-budget self-test: fixtures + assertions that prove each rule bites.

Split out of docs_budget.py so the production module stays under the
Class-1 1,000-line budget (reference/code-quality-taxonomy.md), the same
split backlog_lint.py / backlog_lint_selftest.py already uses. The single
entry point is still `python3 scripts/docs_budget.py --self-test`, which
imports run_self_test() from here — the CLI contract is unchanged. Every
symbol under test is imported from the production module (one classifier,
one scanner), so a mutation there is caught here.

Stdlib only; Python 3.11+.
"""
from __future__ import annotations

import contextlib
import io
import os
import tempfile
from pathlib import Path

import docs_budget
from docs_budget import (
    BUDGET_ALWAYS_LOADED_TOTAL,
    BUDGET_CLAUDE_MD,
    BUDGET_PROJECT_LOG,
    BUDGET_RULES_MD,
    ZONE_FAIL,
    ZONE_OK,
    ZONE_WARN,
    classify,
    escapes_root,
    is_always_loaded,
    main,
    probe,
    run,
    scan,
)


def _write_sized(path: Path, size: int) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"x" * size)


def _build_repo(base: Path, spec: dict[str, int]) -> Path:
    """Create a fixture repo at `base` with files of exact byte sizes."""
    for rel, size in spec.items():
        _write_sized(base / Path(rel), size)
    return base


def run_self_test() -> int:
    failures: list[str] = []

    def check(name: str, ok: bool, detail: str = "") -> None:
        status = "PASS" if ok else "FAIL"
        print(f"  [{status}] {name}" + (f" — {detail}" if detail and not ok else ""))
        if not ok:
            failures.append(name)

    print("docs_budget self-test")

    # --- 1. zone classification boundaries -------------------------------
    print("zone classification:")
    check("size == budget is OK", classify(12_288, BUDGET_CLAUDE_MD) == ZONE_OK)
    check("budget + 1 is WARN", classify(12_289, BUDGET_CLAUDE_MD) == ZONE_WARN)
    check("size == 1.5x budget is WARN", classify(18_432, BUDGET_CLAUDE_MD) == ZONE_WARN)
    check("1.5x budget + 1 is FAIL", classify(18_433, BUDGET_CLAUDE_MD) == ZONE_FAIL)
    check("rules boundary 12,288 is WARN", classify(12_288, BUDGET_RULES_MD) == ZONE_WARN)
    check("rules 12,289 is FAIL", classify(12_289, BUDGET_RULES_MD) == ZONE_FAIL)
    check("log boundary 23,040 is WARN", classify(23_040, BUDGET_PROJECT_LOG) == ZONE_WARN)
    check("log 23,041 is FAIL", classify(23_041, BUDGET_PROJECT_LOG) == ZONE_FAIL)
    # Pin the aggregate cap to the exact byte, the same way the per-file
    # constants above are pinned: the OK/WARN pair fixes it at 49,152 (any
    # smaller value reds the first check, any larger value reds the second).
    check("aggregate boundary 49,152 is OK",
          classify(49_152, BUDGET_ALWAYS_LOADED_TOTAL) == ZONE_OK)
    check("aggregate 49,153 is WARN",
          classify(49_153, BUDGET_ALWAYS_LOADED_TOTAL) == ZONE_WARN)
    check("aggregate boundary 73,728 is WARN",
          classify(73_728, BUDGET_ALWAYS_LOADED_TOTAL) == ZONE_WARN)
    check("aggregate 73,729 is FAIL",
          classify(73_729, BUDGET_ALWAYS_LOADED_TOTAL) == ZONE_FAIL)

    print("always-loaded membership:")
    check("root CLAUDE.md is always-loaded", is_always_loaded("CLAUDE.md"))
    check("nested .claude/CLAUDE.md is always-loaded",
          is_always_loaded(".claude/CLAUDE.md"))
    check("a rules file is always-loaded",
          is_always_loaded(".claude/rules/01-workflow.md"))
    check("project_log.md is NOT always-loaded",
          not is_always_loaded("project_log.md"))
    check("a non-md file under rules/ is NOT always-loaded",
          not is_always_loaded(".claude/rules/notes.txt"))

    # --- 2. static fixture: under-budget repo ----------------------------
    print("static under-budget fixture:")
    fixtures = Path(__file__).resolve().parent.parent / "tests" / "docs_budget"
    under = fixtures / "under_budget_repo"
    findings, _, _ = scan(under)
    check(
        "scans 3 surfaces (CLAUDE.md, one rules file, project_log.md)",
        len(findings) == 3,
        f"got {len(findings)}: {[f[0] for f in findings]}",
    )
    check("all surfaces in OK zone", all(f[3] == ZONE_OK for f in findings))
    code_warn, _ = run(under, enforce=False)
    code_enf, _ = run(under, enforce=True)
    check("exit 0 in warn-only mode", code_warn == 0)
    check("exit 0 in enforce mode", code_enf == 0)

    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)

        # --- 3. generated warn-zone repo (1.0x–1.5x) ----------------------
        print("generated warn-zone repo:")
        warn_repo = _build_repo(
            tmp / "warn_repo",
            {
                "CLAUDE.md": 14_000,           # 12,288 < size <= 18,432
                ".claude/CLAUDE.md": 14_000,   # the alternate location is scanned
                ".claude/rules/01-r.md": 10_000,  # 8,192 < size <= 12,288
                "project_log.md": 20_000,      # 15,360 < size <= 23,040
            },
        )
        findings, _, _ = scan(warn_repo)
        check("scans 4 surfaces incl. .claude/CLAUDE.md", len(findings) == 4,
              f"got {[f[0] for f in findings]}")
        check("all surfaces in WARN zone", all(f[3] == ZONE_WARN for f in findings))
        code_w, lines_w = run(warn_repo, enforce=False)
        code_e, lines_e = run(warn_repo, enforce=True)
        check("warn zone exits 0 in warn-only mode", code_w == 0)
        check("warn zone exits 0 in enforce mode", code_e == 0)
        check(
            "one ::warning annotation per surface",
            sum(1 for l in lines_w if l.startswith("::warning")) == 4
            and sum(1 for l in lines_e if l.startswith("::warning")) == 4,
        )
        check("no ::error annotations for warn zone",
              not any(l.startswith("::error") for l in lines_w + lines_e))
        # 38,000 B of always-loaded surfaces here — deliberately under the
        # 49,152 B aggregate cap, so the per-surface counts above are exact.
        check("aggregate stays OK while every file is warn-zone",
              any("[OK  ] always-loaded total" in l for l in lines_w))

        # --- 4. generated fail-zone repo (> 1.5x) -------------------------
        print("generated fail-zone repo:")
        fail_repo = _build_repo(
            tmp / "fail_repo",
            {
                "CLAUDE.md": 18_433,
                ".claude/rules/01-r.md": 12_289,
                "project_log.md": 23_041,
            },
        )
        findings, _, _ = scan(fail_repo)
        check("all surfaces in FAIL zone",
              len(findings) == 3 and all(f[3] == ZONE_FAIL for f in findings))
        code_w, lines_w = run(fail_repo, enforce=False)
        code_e, lines_e = run(fail_repo, enforce=True)
        check("fail zone exits 0 in warn-only mode", code_w == 0)
        check("fail zone exits 1 in enforce mode", code_e == 1)
        check("warn-only mode downgrades fails to ::warning",
              sum(1 for l in lines_w if l.startswith("::warning")) == 3
              and not any(l.startswith("::error") for l in lines_w))
        check("enforce mode emits ::error per fail-zone surface",
              sum(1 for l in lines_e if l.startswith("::error")) == 3)
        check("report lines name file, size, budget, zone",
              any("[FAIL] CLAUDE.md — 18,433 B / 12,288 B budget" in l for l in lines_e))

        # --- 4b. aggregate always-loaded cap ------------------------------
        # The behaviour per-file budgets cannot express: every file is
        # individually within budget, but the always-loaded tier as a whole
        # claims too much of every session's context.
        print("aggregate always-loaded cap (every file individually OK):")
        agg_warn_repo = _build_repo(
            tmp / "agg_warn_repo",
            # 7 x 8,000 B = 56,000 B -> 1.14x of 49,152 B
            {f".claude/rules/{i:02d}-r.md": 8_000 for i in range(1, 8)},
        )
        findings, _, _ = scan(agg_warn_repo)
        check("every per-file zone is OK", all(f[3] == ZONE_OK for f in findings))
        code_w, lines_w = run(agg_warn_repo, enforce=False)
        code_e, lines_e = run(agg_warn_repo, enforce=True)
        check("aggregate reported in the WARN zone",
              any("[WARN] always-loaded total (7 surface(s)) — 56,000 B "
                  "/ 49,152 B budget" in l
                  for l in lines_e),
              f"got {[l for l in lines_e if 'always-loaded' in l]}")
        check("aggregate warn emits a fileless ::warning",
              any(l.startswith("::warning::always-loaded") for l in lines_e))
        check("aggregate warn exits 0 in both modes", code_w == 0 and code_e == 0)

        agg_fail_repo = _build_repo(
            tmp / "agg_fail_repo",
            # 10 x 8,000 B = 80,000 B -> 1.63x, past the 73,728 B fail line
            {f".claude/rules/{i:02d}-r.md": 8_000 for i in range(1, 11)},
        )
        findings, _, _ = scan(agg_fail_repo)
        check("every per-file zone is still OK",
              all(f[3] == ZONE_OK for f in findings))
        code_w, lines_w = run(agg_fail_repo, enforce=False)
        code_e, lines_e = run(agg_fail_repo, enforce=True)
        check("aggregate fail exits 1 in enforce mode", code_e == 1)
        check("aggregate fail exits 0 in warn-only mode", code_w == 0)
        check("aggregate fail emits a fileless ::error in enforce mode",
              any(l.startswith("::error::always-loaded") for l in lines_e))
        check("warn-only downgrades the aggregate fail to ::warning",
              any(l.startswith("::warning::always-loaded") for l in lines_w)
              and not any(l.startswith("::error") for l in lines_w))

        # project_log.md is first-read tier, not always-loaded: it must not
        # count toward the aggregate even though it is a budgeted surface.
        print("aggregate excludes project_log.md:")
        agg_log_repo = _build_repo(
            tmp / "agg_log_repo",
            {"CLAUDE.md": 1_000, "project_log.md": 15_000},
        )
        _, lines_l = run(agg_log_repo, enforce=True)
        check("project_log.md bytes excluded from the aggregate total",
              any("always-loaded total (1 surface(s)) — 1,000 B" in l
                  for l in lines_l),
              f"got {[l for l in lines_l if 'always-loaded' in l]}")

        # The summary line's aggregate clause, and its "n/a" false branch,
        # were both unasserted: a log-only repo has no always-loaded surface
        # at all, so it must read "n/a" and stay exit 0 — distinct from a
        # repo that was checked and passed.
        check("summary line names the aggregate zone",
              any("always-loaded total FAIL (mode: enforce)." in l
                  for l in lines_e),
              f"got {[l for l in lines_e if l.startswith('docs-budget:')]}")
        log_only_repo = _build_repo(
            tmp / "log_only_repo", {"project_log.md": 1_000},
        )
        code_n, lines_n = run(log_only_repo, enforce=True)
        check("log-only repo reports the aggregate as n/a",
              any("always-loaded total n/a (mode: enforce)." in l
                  for l in lines_n),
              f"got {[l for l in lines_n if l.startswith('docs-budget:')]}")
        check("log-only repo emits no aggregate report line",
              not any("always-loaded total (" in l for l in lines_n))
        check("log-only repo exits 0 in enforce mode", code_n == 0)

        # Both CLAUDE.md locations may coexist; is_always_loaded's docstring
        # calls the nested one an alternate, so pin that the aggregate sums
        # both rather than picking one.
        both_claude_repo = _build_repo(
            tmp / "both_claude_repo",
            {"CLAUDE.md": 1_000, ".claude/CLAUDE.md": 2_000},
        )
        _, lines_b = run(both_claude_repo, enforce=True)
        check("both CLAUDE.md locations sum into the aggregate",
              any("always-loaded total (2 surface(s)) — 3,000 B" in l
                  for l in lines_b),
              f"got {[l for l in lines_b if 'always-loaded' in l]}")

        # --- 5. missing surfaces skipped silently -------------------------
        print("missing surfaces:")
        empty_repo = tmp / "empty_repo"
        empty_repo.mkdir()
        code_w, _ = run(empty_repo, enforce=False)
        code_e, _ = run(empty_repo, enforce=True)
        check("empty repo finds nothing and exits 0 in both modes",
              scan(empty_repo) == ([], [], []) and code_w == 0 and code_e == 0)

        # --- 5b. unreadable surface (stat() raises) ------------------------
        # Cross-platform simulation: monkeypatch collect_surfaces to hand
        # scan() a surface that vanished between collection and stat()
        # (FileNotFoundError is an OSError, same handling as permission
        # denied). Restored in `finally`.
        print("unreadable surface:")
        ghost_repo = tmp / "ghost_repo"
        _write_sized(ghost_repo / "CLAUDE.md", 100)
        _orig_collect = docs_budget.collect_surfaces
        def _collect_with_ghost(root: Path) -> tuple[
            list[tuple[Path, int]], list[tuple[str, str, str]]
        ]:
            surfaces, skips = _orig_collect(root)
            return surfaces + [(root / "project_log.md", BUDGET_PROJECT_LOG)], skips
        try:
            docs_budget.collect_surfaces = _collect_with_ghost
            findings, unreadable, _ = scan(ghost_repo)
            check("unreadable surface excluded from findings",
                  [f[0] for f in findings] == ["CLAUDE.md"],
                  f"got {[f[0] for f in findings]}")
            check("::warning emitted naming the unreadable file",
                  len(unreadable) == 1
                  and unreadable[0].startswith("::warning")
                  and "project_log.md" in unreadable[0],
                  f"got {unreadable}")
            code_w, lines_w = run(ghost_repo, enforce=False)
            code_e, lines_e = run(ghost_repo, enforce=True)
            check("unreadable surface exits 0 in warn-only mode", code_w == 0)
            check("unreadable surface exits 0 even in enforce mode", code_e == 0)
            check("run output carries the unreadable-file warning, no ::error",
                  any("project_log.md" in l and l.startswith("::warning")
                      for l in lines_w + lines_e)
                  and not any(l.startswith("::error") for l in lines_w + lines_e))
        finally:
            docs_budget.collect_surfaces = _orig_collect

        # --- 6. junction/symlink escape ------------------------------------
        # escapes_root() is a pure resolved-path containment comparison, so
        # the core logic is testable without creating a real link (Windows
        # junctions can't be created portably from stdlib Python; symlinks
        # need privileges). We always test the pure comparison; when the
        # platform lets us create a symlink we additionally test the real
        # scan-level skip.
        print("junction/symlink escape:")
        outside = tmp / "outside_rules"
        _write_sized(outside / "99-big.md", 9_000)
        repo = tmp / "link_repo"
        _write_sized(repo / "CLAUDE.md", 100)
        check("pure comparison: outside dir escapes root",
              escapes_root(outside, repo))
        check("pure comparison: in-root path does not escape",
              not escapes_root(repo / "CLAUDE.md", repo))
        check("pure comparison: root does not escape itself",
              not escapes_root(repo, repo))
        try:
            (repo / ".claude").mkdir()
            (repo / ".claude" / "rules").symlink_to(outside, target_is_directory=True)
            link_made = True
        except OSError:
            link_made = False
        if link_made:
            findings, _, skips = scan(repo)
            check("rules dir resolving outside repo is skipped",
                  [f[0] for f in findings] == ["CLAUDE.md"],
                  f"got {[f[0] for f in findings]}")
            check("escaped rules dir is recorded as a skip, not dropped",
                  any(sk[1] == "escaped" for sk in skips),
                  f"got {skips}")
            code_esc, lines_esc = run(repo, enforce=True)
            check("escaped rules dir makes the aggregate PARTIAL and fails closed",
                  code_esc == 1
                  and any("[PARTIAL] always-loaded total" in l for l in lines_esc),
                  f"got exit {code_esc}, "
                  f"{[l for l in lines_esc if 'always-loaded' in l]}")
        else:
            print("  [SKIP] symlink creation unavailable on this platform; "
                  "pure-comparison checks above cover the logic")

        # --- 7. aggregate completeness -------------------------------------
        # The failure this guards: an always-loaded surface that is present
        # but unmeasurable leaves the sum, and a genuine over-cap total reads
        # as comfortably under. Reproduced with the fail-zone fixture — 10 x
        # 8,000 B = 80,000 B is past the 73,728 B fail line, but drop one
        # file and the naive sum is 72,000 B, a green WARN.
        print("aggregate completeness (an incomplete scan must not read green):")
        agg_drop_repo = _build_repo(
            tmp / "agg_drop_repo",
            {f".claude/rules/{i:02d}-r.md": 8_000 for i in range(1, 11)},
        )
        _orig_collect_2 = docs_budget.collect_surfaces
        dropped_rel = ".claude/rules/01-r.md"

        def _collect_dropping_one(root: Path) -> tuple[
            list[tuple[Path, int]], list[tuple[str, str, str]]
        ]:
            surfaces, skips = _orig_collect_2(root)
            kept = [(pth, bud) for pth, bud in surfaces
                    if pth.relative_to(root).as_posix() != dropped_rel]
            return kept, skips + [(dropped_rel, "error", "OSError: simulated")]

        try:
            docs_budget.collect_surfaces = _collect_dropping_one
            code_w, lines_w = run(agg_drop_repo, enforce=False)
            code_e, lines_e = run(agg_drop_repo, enforce=True)
            check("an unmeasurable surface makes the aggregate PARTIAL",
                  any("[PARTIAL] always-loaded total" in l for l in lines_e),
                  f"got {[l for l in lines_e if 'always-loaded' in l]}")
            check("the partial line reports the counted bytes as a floor",
                  any("72,000 B counted" in l and "total is incomplete" in l
                      for l in lines_e),
                  f"got {[l for l in lines_e if 'always-loaded' in l]}")
            check("no zone/ratio verdict is printed for a partial aggregate",
                  not any("[OK  ] always-loaded" in l or "[WARN] always-loaded" in l
                          or "[FAIL] always-loaded" in l for l in lines_e))
            check("partial aggregate fails closed in enforce mode", code_e == 1)
            check("partial aggregate emits a fileless ::error naming the surface",
                  any(l.startswith("::error::") and dropped_rel in l
                      for l in lines_e),
                  f"got {[l for l in lines_e if l.startswith('::error')]}")
            check("partial aggregate exits 0 in warn-only mode", code_w == 0)
            check("warn-only downgrades the partial ::error to ::warning",
                  any(l.startswith("::warning::") and dropped_rel in l
                      for l in lines_w)
                  and not any(l.startswith("::error") for l in lines_w))
            check("summary line names the aggregate as PARTIAL",
                  any("always-loaded total PARTIAL (mode: enforce)." in l
                      for l in lines_e),
                  f"got {[l for l in lines_e if l.startswith('docs-budget:')]}")
            check("the skipped surface is also warned about by file",
                  any(l.startswith(f"::warning file={dropped_rel}::")
                      for l in lines_e))
        finally:
            docs_budget.collect_surfaces = _orig_collect_2

        # project_log.md is not always-loaded, so losing it must NOT make the
        # aggregate partial — the fail-closed rule is scoped to the tier the
        # aggregate actually measures.
        check("an unmeasurable non-always-loaded surface leaves the "
              "aggregate authoritative",
              not any("PARTIAL" in l for l in run(ghost_repo, enforce=True)[1]),
              f"got {[l for l in run(ghost_repo, enforce=True)[1] if 'always-loaded' in l]}")

        # --- 8. real unmeasurable triggers (no monkeypatch) ----------------
        # Sections 5b and 7 inject skip tuples, so they never reach probe()
        # itself. These use real filesystem triggers, and the directory ones
        # are the Blocker: an inline is_dir()/glob() pair dropped the whole
        # always-loaded rules tier with no skip, and a genuine over-cap total
        # read as a comfortable OK.
        print("real unmeasurable triggers:")

        def partial_closed(repo_root: Path, label: str) -> None:
            code_e2, lines_e2 = run(repo_root, enforce=True)
            code_w2, _ = run(repo_root, enforce=False)
            check(f"{label}: aggregate reads PARTIAL, never a zone",
                  any("[PARTIAL] always-loaded total" in l for l in lines_e2)
                  and not any("] always-loaded total (" in l and "PARTIAL" not in l
                              for l in lines_e2),
                  f"got {[l for l in lines_e2 if 'always-loaded' in l]}")
            check(f"{label}: fails closed under --enforce", code_e2 == 1)
            check(f"{label}: exits 0 in warn-only mode", code_w2 == 0)
            check(f"{label}: summary counts the unmeasurable surfaces",
                  any("unmeasurable; always-loaded total PARTIAL" in l
                      for l in lines_e2),
                  f"got {[l for l in lines_e2 if l.startswith('docs-budget:')]}")

        loop_repo = _build_repo(tmp / "loop_rules_repo", {"CLAUDE.md": 10_000})
        (loop_repo / ".claude").mkdir(parents=True, exist_ok=True)
        loop_link = loop_repo / ".claude" / "rules"
        try:
            loop_link.symlink_to(loop_link, target_is_directory=True)
            links_ok = True
        except OSError:
            links_ok = False
        if links_ok:
            _, skips_loop = docs_budget.collect_surfaces(loop_repo)
            check("looping rules dir is a recorded skip, not a silent absence",
                  [(sk[0], sk[1]) for sk in skips_loop] == [(".claude/rules/", "error")],
                  f"got {skips_loop}")
            partial_closed(loop_repo, "looping rules dir")

            # probe()'s OSError branch through a real ELOOP on a rules
            # file, in a repo with no measurable surface at all: that is the
            # skips-only summary path, which used to close a failing run
            # with "no budgeted surfaces found — nothing to check".
            eloop_repo = tmp / "eloop_file_repo"
            (eloop_repo / ".claude" / "rules").mkdir(parents=True)
            eloop_file = eloop_repo / ".claude" / "rules" / "01-r.md"
            eloop_file.symlink_to(eloop_file)
            check("probe() reports a real ELOOP as 'error', not 'missing'",
                  probe(eloop_file)[0] == "error", f"got {probe(eloop_file)}")
            partial_closed(eloop_repo, "looping rules file")

            # The per-file escaped skip through its own trigger: section 6
            # covers only the escaped directory.
            outside_file = tmp / "outside_claude" / "CLAUDE.md"
            _write_sized(outside_file, 40_000)
            esc_repo = _build_repo(tmp / "escaped_file_repo",
                                   {".claude/rules/01-r.md": 40_000})
            (esc_repo / "CLAUDE.md").symlink_to(outside_file)
            _, skips_esc = docs_budget.collect_surfaces(esc_repo)
            check("escaped per-file surface is a recorded skip",
                  ("CLAUDE.md", "escaped") in [(sk[0], sk[1]) for sk in skips_esc],
                  f"got {skips_esc}")
            partial_closed(esc_repo, "escaped CLAUDE.md")

            # A symlink whose target does not resolve: present as a link,
            # unmeasurable through it, and indistinguishable from an absent
            # directory until lstat separates them.
            dangle_repo = _build_repo(tmp / "dangle_repo", {"CLAUDE.md": 10_000})
            (dangle_repo / ".claude").mkdir()
            (dangle_repo / ".claude" / "rules").symlink_to(
                tmp / "no-such-target", target_is_directory=True)
            check("probe() reports a dangling symlink as 'error', not 'missing'",
                  probe(dangle_repo / ".claude" / "rules")[0] == "error",
                  f"got {probe(dangle_repo / '.claude' / 'rules')}")
            partial_closed(dangle_repo, "dangling rules symlink")
        else:
            print("  [SKIP] symlink creation unavailable on this platform")

        # chmod triggers: an unlistable rules dir (EACCES out of scandir,
        # which glob() discarded) and an unreadable .claude parent (EACCES out
        # of stat, which is_dir() raised as a traceback — exit 1 even
        # warn-only).
        perm_repo = _build_repo(
            tmp / "perm_rules_repo",
            {"CLAUDE.md": 10_000,
             **{f".claude/rules/{i:02d}-r.md": 8_000 for i in range(1, 11)}},
        )
        code_pre, lines_pre = run(perm_repo, enforce=True)
        check("baseline: the readable fixture is a genuine aggregate FAIL",
              code_pre == 1
              and any("[FAIL] always-loaded total" in l for l in lines_pre),
              f"got exit {code_pre}")
        perm_rules = perm_repo / ".claude" / "rules"
        try:
            perm_rules.chmod(0o000)
            perm_ok = not os.access(perm_rules, os.R_OK)
        except OSError:
            perm_ok = False
        if perm_ok:
            try:
                partial_closed(perm_repo, "unlistable rules dir")
            finally:
                perm_rules.chmod(0o755)
            try:
                (perm_repo / ".claude").chmod(0o000)
                partial_closed(perm_repo, "unreadable .claude parent")
            finally:
                (perm_repo / ".claude").chmod(0o755)
        else:
            print("  [SKIP] chmod does not restrict access here (running as root?)")

        # --- 9. rules scope and non-regular surfaces -----------------------
        print("rules scope:")
        txt_repo = _build_repo(
            tmp / "txt_rules_repo",
            {".claude/rules/01-r.md": 1_000, ".claude/rules/notes.txt": 30_000},
        )
        findings_t, _, _ = scan(txt_repo)
        check("a non-.md file under .claude/rules is not budgeted",
              [f[0] for f in findings_t] == [".claude/rules/01-r.md"],
              f"got {[f[0] for f in findings_t]}")
        _, lines_t = run(txt_repo, enforce=True)
        check("a non-.md file under .claude/rules is not counted in the aggregate",
              any("always-loaded total (1 surface(s)) — 1,000 B" in l for l in lines_t),
              f"got {[l for l in lines_t if 'always-loaded' in l]}")

        dir_md_repo = _build_repo(tmp / "dir_md_repo", {"CLAUDE.md": 1_000})
        (dir_md_repo / ".claude" / "rules" / "01-r.md").mkdir(parents=True)
        findings_d, _, skips_d = scan(dir_md_repo)
        check("a directory named *.md is not budgeted on its inode size",
              [f[0] for f in findings_d] == ["CLAUDE.md"],
              f"got {[f[0] for f in findings_d]}")
        check("a directory named *.md is not an unmeasurable skip either",
              skips_d == [], f"got {skips_d}")

        class _UnresolvablePath:
            """Stand-in: a resolve() that raises is not portably reproducible
            with a real path on every platform."""

            def resolve(self) -> Path:
                raise OSError("simulated resolution failure")

        check("escapes_root fails closed when resolution raises",
              escapes_root(_UnresolvablePath(), tmp))

        # --- 10. exit codes through main() ---------------------------------
        # Every assertion above reads run()'s code; CI reads main()'s. Pattern
        # copied from scripts/backlog_lint_selftest.py.
        print("exit codes through main():")

        def run_main(args: list[str]) -> tuple[int, str]:
            buf = io.StringIO()
            # stderr too: the non-directory-root case prints a deliberate
            # error that must not read as a real failure in a CI log.
            with contextlib.redirect_stdout(buf), contextlib.redirect_stderr(buf):
                rc = main(args)
            return rc, buf.getvalue()

        rc_m, out_m = run_main(["--root", str(under)])
        check("main: under-budget repo exits 0", rc_m == 0, out_m)
        rc_m, out_m = run_main(["--root", str(agg_fail_repo)])
        check("main: aggregate fail exits 0 in warn-only mode", rc_m == 0, out_m)
        rc_m, out_m = run_main(["--root", str(agg_fail_repo), "--enforce"])
        check("main: aggregate fail exits 1 under --enforce", rc_m == 1, out_m)
        rc_m, out_m = run_main(["--root", str(under / "CLAUDE.md")])
        check("main: a non-directory root exits 2", rc_m == 2, out_m)

    print(f"self-test: {'FAIL — ' + str(len(failures)) + ' failure(s)' if failures else 'all checks passed'}")
    return 1 if failures else 0


