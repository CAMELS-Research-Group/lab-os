#!/usr/bin/env python3
"""docs-budget: byte-size budget check for always-loaded AI doc surfaces.

Budgets are owned by .claude/rules/04-docs.md section "Tiers & budgets" and
mirrored here; the self-test pins them so this script cannot drift from the
rule it enforces.

Scanned paths (relative to --root; each skipped silently when absent):

    CLAUDE.md             budget 12,288 B
    .claude/CLAUDE.md     budget 12,288 B   (some repos keep it here instead)
    .claude/rules/*.md    budget  8,192 B each
    project_log.md        budget 15,360 B

Plus one synthetic surface, the reason this script exists in its current form:

    <always-loaded aggregate>  budget 49,152 B

The aggregate sums CLAUDE.md and every .claude/rules/*.md — what the AI tier
costs every session and every subagent — and deliberately excludes
project_log.md, which is first-read rather than always-loaded. Per-file checks
alone cannot see the failure mode where every surface sits inside its own
budget while the tier as a whole is over; that shape is what motivated the
aggregate check. Its annotations carry no `file=`, because it names no one
file.

Zone semantics (size measured in bytes on disk):

    size <= budget                 -> OK    (no output beyond the report line)
    budget < size <= 1.5 * budget  -> WARN  (GitHub ::warning annotation, exit 0)
    size > 1.5 * budget            -> FAIL  (exit 1 when --enforce; otherwise
                                             downgraded to a warning, exit 0)

Default mode is warn-only (the warn-only-until-first-green posture): the script always exits 0 no matter what it finds. Pass --enforce
to exit 1 when any surface is in the FAIL zone.

EOL drift: CRLF working copies (e.g. Windows checkouts with autocrlf)
measure larger than the LF checkouts CI sees; budgets are calibrated for
normalized (LF) checkouts and the 1.5x WARN band absorbs the drift.

Junction/symlink awareness: any scanned path (the .claude dir, the rules
dir, or an individual file) whose resolved location lies outside the
resolved repo root is skipped. On Windows, junctions may not register as
symlinks via Path.is_symlink(), so the check compares Path.resolve()
results instead of testing link-ness (mission-control junctions its rules
dir from lab-os; that surface belongs to lab-os' own budget run).

Stdlib only; compatible with Python 3.11+.
"""

from __future__ import annotations

import argparse
import sys
import tempfile
from pathlib import Path

BUDGET_CLAUDE_MD = 12_288
BUDGET_RULES_MD = 8_192
BUDGET_PROJECT_LOG = 15_360
BUDGET_AGGREGATE = 49_152

# The aggregate is reported as a synthetic surface: it names no single file, so
# its annotations carry no `file=` and cannot be anchored in a diff.
AGGREGATE_LABEL = "<always-loaded aggregate>"

ZONE_OK = "OK"
ZONE_WARN = "WARN"
ZONE_FAIL = "FAIL"


def fail_threshold(budget: int) -> int:
    """Largest size still in the WARN zone: 1.5 * budget, exact in ints."""
    return (budget * 3) // 2


def classify(size: int, budget: int) -> str:
    """Zone for a file of `size` bytes against `budget` bytes.

    size <= budget -> OK; budget < size <= 1.5*budget -> WARN; else FAIL.
    """
    if size <= budget:
        return ZONE_OK
    if size <= fail_threshold(budget):
        return ZONE_WARN
    return ZONE_FAIL


def escapes_root(path: Path, root: Path) -> bool:
    """True when `path` resolves to a location outside resolved `root`.

    Pure resolved-path containment comparison — deliberately NOT
    Path.is_symlink(), which reports False for Windows junctions even
    though Path.resolve() follows them out of the repo.
    """
    try:
        resolved = path.resolve()
        resolved_root = root.resolve()
    except OSError:
        return True
    return not resolved.is_relative_to(resolved_root)


def collect_surfaces(root: Path) -> list[tuple[Path, int]]:
    """(path, budget) pairs for every scanned surface present under root.

    Missing surfaces are skipped silently; surfaces resolving outside the
    repo root (junction/symlink escape) are skipped silently too.
    """
    surfaces: list[tuple[Path, int]] = []

    fixed = [
        (root / "CLAUDE.md", BUDGET_CLAUDE_MD),
        (root / ".claude" / "CLAUDE.md", BUDGET_CLAUDE_MD),
        (root / "project_log.md", BUDGET_PROJECT_LOG),
    ]
    for path, budget in fixed:
        if path.is_file() and not escapes_root(path, root):
            surfaces.append((path, budget))

    rules_dir = root / ".claude" / "rules"
    if rules_dir.is_dir() and not escapes_root(rules_dir, root):
        for path in sorted(rules_dir.glob("*.md")):
            if path.is_file() and not escapes_root(path, root):
                surfaces.append((path, BUDGET_RULES_MD))

    return surfaces


def collect_aggregate_members(root: Path) -> list[Path]:
    """Always-loaded surfaces that count toward the aggregate budget.

    CLAUDE.md (either location) plus every .claude/rules/*.md. project_log.md
    is deliberately excluded: it is first-read, not always-loaded, and
    04-docs.md scopes the aggregate to "CLAUDE.md + every rules file,
    excluding project_log.md".
    """
    members: list[Path] = []
    for path, budget in collect_surfaces(root):
        if path.name == "project_log.md":
            continue
        members.append(path)
    return members


def aggregate_finding(root: Path) -> tuple[str, int, int, str] | None:
    """(label, total bytes, budget, zone) for the always-loaded tier.

    None when the repo has no always-loaded surface at all — there is no tier
    to report on, and a 0 B aggregate line would be noise.

    Unreadable members are skipped rather than failing the run, matching
    scan()'s per-file posture; the total is then an undercount, which is the
    safe direction for a budget check.
    """
    members = collect_aggregate_members(root)
    if not members:
        return None
    total = 0
    for path in members:
        try:
            total += path.stat().st_size
        except OSError:
            continue
    return (AGGREGATE_LABEL, total, BUDGET_AGGREGATE,
            classify(total, BUDGET_AGGREGATE))


def scan(root: Path) -> tuple[list[tuple[str, int, int, str]], list[str]]:
    """Scan surfaces under root.

    Returns (findings, warnings): findings are (relative posix path,
    size bytes, budget bytes, zone) per readable surface; warnings are
    ready-to-print ::warning lines for surfaces that could not be stat'd.
    """
    findings: list[tuple[str, int, int, str]] = []
    warnings: list[str] = []
    for path, budget in collect_surfaces(root):
        rel = path.relative_to(root).as_posix()
        try:
            size = path.stat().st_size
        except OSError as exc:
            # Unreadable surfaces (permission denied, TOCTOU-vanished file)
            # are warned about and skipped — never a failure, even in
            # --enforce mode: a budget check must not turn a filesystem
            # hiccup into a red CI run.
            warnings.append(
                f"::warning file={rel}::{rel} could not be read "
                f"({exc.__class__.__name__}: {exc}); skipping its budget check."
            )
            continue
        findings.append((rel, size, budget, classify(size, budget)))

    aggregate = aggregate_finding(root)
    if aggregate is not None:
        findings.append(aggregate)

    return findings, warnings


def run(root: Path, enforce: bool) -> tuple[int, list[str]]:
    """Scan `root` and build the report. Returns (exit_code, output_lines)."""
    lines: list[str] = []
    findings, unreadable = scan(root)
    lines.extend(unreadable)
    failed = False

    for rel, size, budget, zone in findings:
        ratio = size / budget
        is_aggregate = rel == AGGREGATE_LABEL
        # GitHub anchors an annotation to a file; the aggregate has none, so it
        # is emitted bare rather than mis-anchored to an arbitrary member.
        anchor = "" if is_aggregate else f" file={rel}"
        subject = (
            "the always-loaded tier (CLAUDE.md + .claude/rules/*.md)"
            if is_aggregate else rel
        )
        remedy = (
            "Demote a surface to grep-only (04-docs.md: over aggregate -> demote, "
            "not raise the cap)."
            if is_aggregate else
            "Compress, or move detail to the ENG tier."
        )
        lines.append(
            f"[{zone:<4}] {rel} — {size:,} B / {budget:,} B budget ({ratio:.2f}x)"
        )
        if zone == ZONE_WARN:
            lines.append(
                f"::warning{anchor}::{subject} is over its context budget: "
                f"{size:,} B vs {budget:,} B ({ratio:.2f}x). {remedy} "
                f"(fails above 1.5x)."
            )
        elif zone == ZONE_FAIL:
            if enforce:
                failed = True
                lines.append(
                    f"::error{anchor}::{subject} exceeds 1.5x its context budget: "
                    f"{size:,} B vs {budget:,} B ({ratio:.2f}x). {remedy}"
                )
            else:
                lines.append(
                    f"::warning{anchor}::{subject} exceeds 1.5x its context "
                    f"budget: {size:,} B vs {budget:,} B ({ratio:.2f}x). "
                    f"Warn-only mode — this will fail once enforcement is on."
                )

    if not findings:
        lines.append("docs-budget: no budgeted surfaces found — nothing to check.")
    else:
        n_warn = sum(1 for f in findings if f[3] == ZONE_WARN)
        n_fail = sum(1 for f in findings if f[3] == ZONE_FAIL)
        mode = "enforce" if enforce else "warn-only"
        lines.append(
            f"docs-budget: {len(findings)} surface(s) checked, "
            f"{n_warn} warn-zone, {n_fail} fail-zone (mode: {mode})."
        )

    return (1 if failed else 0, lines)


# --------------------------------------------------------------------------
# Self-test
# --------------------------------------------------------------------------

def _write_sized(path: Path, size: int) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"x" * size)


def _build_repo(base: Path, spec: dict[str, int]) -> Path:
    """Create a fixture repo at `base` with files of exact byte sizes."""
    for rel, size in spec.items():
        _write_sized(base / Path(rel), size)
    return base


def self_test() -> int:
    failures: list[str] = []

    def check(name: str, ok: bool, detail: str = "") -> None:
        status = "PASS" if ok else "FAIL"
        print(f"  [{status}] {name}" + (f" — {detail}" if detail and not ok else ""))
        if not ok:
            failures.append(name)

    def per_file(findings: list) -> list:
        """Findings minus the synthetic aggregate row.

        Every assertion about per-file surfaces filters it out; the aggregate
        has its own section.
        """
        return [f for f in findings if f[0] != AGGREGATE_LABEL]

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

    # --- 2. static fixture: under-budget repo ----------------------------
    print("static under-budget fixture:")
    fixtures = Path(__file__).resolve().parent.parent / "tests" / "docs_budget"
    under = fixtures / "under_budget_repo"
    findings, _ = scan(under)
    check(
        "scans 3 surfaces (CLAUDE.md, one rules file, project_log.md)",
        len(per_file(findings)) == 3,
        f"got {len(per_file(findings))}: {[f[0] for f in per_file(findings)]}",
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
                "CLAUDE.md": 13_000,           # 12,288 < size <= 18,432
                ".claude/CLAUDE.md": 13_000,   # the alternate location is scanned
                ".claude/rules/01-r.md": 9_000,  # 8,192 < size <= 12,288
                "project_log.md": 20_000,      # 15,360 < size <= 23,040
            },
        )
        findings, _ = scan(warn_repo)
        check("scans 4 surfaces incl. .claude/CLAUDE.md",
              len(per_file(findings)) == 4,
              f"got {[f[0] for f in per_file(findings)]}")
        check("all surfaces in WARN zone",
              all(f[3] == ZONE_WARN for f in per_file(findings)))
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
        findings, _ = scan(fail_repo)
        check("all surfaces in FAIL zone",
              len(per_file(findings)) == 3
              and all(f[3] == ZONE_FAIL for f in per_file(findings)))
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

        # --- 5. missing surfaces skipped silently -------------------------
        print("missing surfaces:")
        empty_repo = tmp / "empty_repo"
        empty_repo.mkdir()
        code_w, _ = run(empty_repo, enforce=False)
        code_e, _ = run(empty_repo, enforce=True)
        check("empty repo finds nothing and exits 0 in both modes",
              scan(empty_repo) == ([], []) and code_w == 0 and code_e == 0)

        # --- 5b. unreadable surface (stat() raises) ------------------------
        # Cross-platform simulation: monkeypatch collect_surfaces to hand
        # scan() a surface that vanished between collection and stat()
        # (FileNotFoundError is an OSError, same handling as permission
        # denied). Restored in `finally`.
        print("unreadable surface:")
        ghost_repo = tmp / "ghost_repo"
        _write_sized(ghost_repo / "CLAUDE.md", 100)
        _orig_collect = collect_surfaces
        def _collect_with_ghost(root: Path) -> list[tuple[Path, int]]:
            return _orig_collect(root) + [
                (root / "project_log.md", BUDGET_PROJECT_LOG)
            ]
        try:
            globals()["collect_surfaces"] = _collect_with_ghost
            findings, unreadable = scan(ghost_repo)
            check("unreadable surface excluded from findings",
                  [f[0] for f in per_file(findings)] == ["CLAUDE.md"],
                  f"got {[f[0] for f in per_file(findings)]}")
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
            globals()["collect_surfaces"] = _orig_collect

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
            findings, _ = scan(repo)
            check("rules dir resolving outside repo is skipped",
                  [f[0] for f in per_file(findings)] == ["CLAUDE.md"],
                  f"got {[f[0] for f in per_file(findings)]}")
        else:
            print("  [SKIP] symlink creation unavailable on this platform; "
                  "pure-comparison checks above cover the logic")

        # --- 7. budgets track .claude/rules/04-docs.md, not the retired §7.2 ---
        # These are pinned so the gate cannot silently drift from the rule it
        # enforces again: 04-docs.md §Tiers & budgets states CLAUDE.md 12 KB,
        # each .claude/rules/*.md 8 KB, always-loaded aggregate 48 KB.
        print("budgets match 04-docs.md:")
        check("rules budget is 8 KB", BUDGET_RULES_MD == 8_192,
              f"got {BUDGET_RULES_MD}")
        check("CLAUDE.md budget is 12 KB", BUDGET_CLAUDE_MD == 12_288,
              f"got {BUDGET_CLAUDE_MD}")
        check("aggregate budget is 48 KB", BUDGET_AGGREGATE == 49_152,
              f"got {BUDGET_AGGREGATE}")
        check("rules 8,192 is OK", classify(8_192, BUDGET_RULES_MD) == ZONE_OK)
        check("rules 8,193 is WARN", classify(8_193, BUDGET_RULES_MD) == ZONE_WARN)
        check("rules 12,288 is WARN", classify(12_288, BUDGET_RULES_MD) == ZONE_WARN)
        check("rules 12,289 is FAIL", classify(12_289, BUDGET_RULES_MD) == ZONE_FAIL)
        check("CLAUDE.md 12,288 is OK", classify(12_288, BUDGET_CLAUDE_MD) == ZONE_OK)
        check("CLAUDE.md 18,433 is FAIL", classify(18_433, BUDGET_CLAUDE_MD) == ZONE_FAIL)

        # --- 8. the always-loaded aggregate ---------------------------------
        # The defect this check exists for: every individual surface inside its
        # own budget while the tier as a whole is over. Per-file checks alone
        # report green on exactly this shape.
        print("aggregate — every file OK, tier over:")
        agg_repo = _build_repo(
            tmp / "agg_repo",
            {
                ".claude/CLAUDE.md": 10_000,        # OK against 12,288
                ".claude/rules/01-a.md": 7_000,     # OK against 8,192
                ".claude/rules/02-b.md": 7_000,
                ".claude/rules/03-c.md": 7_000,
                ".claude/rules/04-d.md": 7_000,
                ".claude/rules/05-e.md": 7_000,
                ".claude/rules/06-f.md": 7_000,
            },
        )
        findings, _ = scan(agg_repo)
        per_file = [f for f in findings if f[0] != AGGREGATE_LABEL]
        agg = [f for f in findings if f[0] == AGGREGATE_LABEL]
        check("every per-file surface is OK",
              all(f[3] == ZONE_OK for f in per_file),
              f"got {[(f[0], f[3]) for f in per_file if f[3] != ZONE_OK]}")
        check("an aggregate finding is reported", len(agg) == 1,
              f"got {len(agg)}")
        check("aggregate sums CLAUDE.md + every rules file",
              bool(agg) and agg[0][1] == 52_000, f"got {agg[0][1] if agg else None}")
        check("aggregate over budget is WARN",
              bool(agg) and agg[0][3] == ZONE_WARN,
              f"got {agg[0][3] if agg else None}")
        code_w, lines_w = run(agg_repo, enforce=False)
        code_e, lines_e = run(agg_repo, enforce=True)
        check("aggregate warn zone exits 0 in both modes",
              code_w == 0 and code_e == 0)
        check("aggregate annotation carries no file= (it names no one file)",
              any(l.startswith("::warning::") and "aggregate" in l for l in lines_e),
              f"got {[l for l in lines_e if l.startswith('::warning')]}")

        # project_log.md is NOT always-loaded and must not count toward the tier
        print("aggregate excludes project_log.md:")
        _write_sized(agg_repo / "project_log.md", 20_000)
        findings, _ = scan(agg_repo)
        agg = [f for f in findings if f[0] == AGGREGATE_LABEL]
        check("aggregate unchanged by project_log.md",
              bool(agg) and agg[0][1] == 52_000,
              f"got {agg[0][1] if agg else None}")

        print("aggregate above 1.5x fails under --enforce:")
        big_repo = _build_repo(
            tmp / "agg_fail_repo",
            {f".claude/rules/{i:02d}-r.md": 8_192 for i in range(1, 10)},
        )
        findings, _ = scan(big_repo)
        agg = [f for f in findings if f[0] == AGGREGATE_LABEL]
        check("9 x 8,192 = 73,728 aggregate is still WARN at exactly 1.5x",
              bool(agg) and agg[0][1] == 73_728 and agg[0][3] == ZONE_WARN,
              f"got {agg[0] if agg else None}")
        _write_sized(big_repo / ".claude" / "rules" / "10-r.md", 1)
        findings, _ = scan(big_repo)
        agg = [f for f in findings if f[0] == AGGREGATE_LABEL]
        check("73,729 crosses into FAIL",
              bool(agg) and agg[0][3] == ZONE_FAIL,
              f"got {agg[0] if agg else None}")
        code_w, lines_w = run(big_repo, enforce=False)
        code_e, lines_e = run(big_repo, enforce=True)
        check("aggregate fail exits 0 in warn-only mode", code_w == 0)
        check("aggregate fail exits 1 in enforce mode", code_e == 1)
        check("enforce emits an ::error naming the aggregate",
              any(l.startswith("::error::") and "aggregate" in l for l in lines_e),
              f"got {[l for l in lines_e if l.startswith('::error')]}")

        print("aggregate absent when there are no always-loaded surfaces:")
        log_only = _build_repo(tmp / "log_only_repo", {"project_log.md": 100})
        findings, _ = scan(log_only)
        check("no aggregate line when nothing is always-loaded",
              not any(f[0] == AGGREGATE_LABEL for f in findings),
              f"got {[f[0] for f in findings]}")

    print(f"self-test: {'FAIL — ' + str(len(failures)) + ' failure(s)' if failures else 'all checks passed'}")
    return 1 if failures else 0


# --------------------------------------------------------------------------
# CLI
# --------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="docs_budget.py",
        description=(
            "Check always-loaded AI doc surfaces against the lab context "
            "budgets (04-docs.md, Tiers & budgets): CLAUDE.md and "
            ".claude/CLAUDE.md 12,288 B; each .claude/rules/*.md 8,192 B; "
            "project_log.md 15,360 B; and the always-loaded aggregate "
            "(CLAUDE.md + every rules file, excluding project_log.md) "
            "49,152 B. Zones: size <= budget -> OK; budget < size <= "
            "1.5x budget -> WARN (annotation, exit 0); size > 1.5x budget "
            "-> FAIL (exit 1 only with --enforce). Missing surfaces and "
            "surfaces that resolve outside the repo root (junctions/"
            "symlinks) are skipped."
        ),
    )
    parser.add_argument(
        "--root", type=Path, default=Path("."),
        help="repo root to scan (default: current directory)",
    )
    parser.add_argument(
        "--enforce", action="store_true",
        help="exit 1 when any surface is above 1.5x its budget "
             "(default: warn-only — always exit 0)",
    )
    parser.add_argument(
        "--self-test", action="store_true",
        help="run the fixture-backed self-test and exit",
    )
    args = parser.parse_args(argv)

    if args.self_test:
        return self_test()

    if not args.root.is_dir():
        print(f"docs-budget: root is not a directory: {args.root}", file=sys.stderr)
        return 2

    code, lines = run(args.root, enforce=args.enforce)
    for line in lines:
        print(line)
    return code


if __name__ == "__main__":
    sys.exit(main())
