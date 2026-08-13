#!/usr/bin/env python3
"""docs-budget: byte-size budget check for always-loaded AI doc surfaces.

Budgets are owned by `.claude/rules/04-docs.md` (section Tiers & budgets);
this script is their enforcement. Original design:
docs/superpowers/specs/2026-06-10-logging-and-docs-standard-design.md
(section 7.2 context budgets, section 9 Adherence Actions) — that doc's
8/5 KB figures were a stated first guess (its section 13) and were revised
upward on 2026-08-12; the rule file, not the dated design doc, is current.

Scanned paths (relative to --root; each skipped silently when absent):

    CLAUDE.md             budget 12,288 B
    .claude/CLAUDE.md     budget 12,288 B   (some repos keep it here instead)
    .claude/rules/*.md    budget  8,192 B each
    project_log.md        budget 15,360 B

Beyond the per-file budgets, the always-loaded surfaces — CLAUDE.md plus
every .claude/rules/*.md, but NOT project_log.md (first-read tier, and it
has its own archive-overflow path) — are checked in aggregate against
49,152 B. The aggregate is the invariant that actually protects session
context: per-file budgets alone permit unbounded growth by adding rules
files, and every rules file sitting at its full budget would blow the
total on its own. Same zone semantics as a per-file budget.

The aggregate is only reported when it can be computed. A surface
that is present but unmeasurable (stat failure, or a path resolving
outside the repo root) is excluded from the sum, so the total would
read low; the run then reports PARTIAL instead of a zone and, under
--enforce, fails closed. A per-file skip loses one verdict, but a
short aggregate is a wrong number presented as authoritative.

Zone semantics (size measured in bytes on disk):

    size <= budget                 -> OK    (no output beyond the report line)
    budget < size <= 1.5 * budget  -> WARN  (GitHub ::warning annotation, exit 0)
    size > 1.5 * budget            -> FAIL  (exit 1 when --enforce; otherwise
                                             downgraded to a warning, exit 0)

Default mode is warn-only (the section-7.2 warn-only-until-first-green
posture): the script always exits 0 no matter what it finds. Pass --enforce
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
import stat
import sys
import tempfile
from pathlib import Path

BUDGET_CLAUDE_MD = 12_288
BUDGET_RULES_MD = 8_192
BUDGET_PROJECT_LOG = 15_360

# Aggregate cap on everything that loads into EVERY session's context.
# Deliberately smaller than the sum of the per-file budgets: the per-file
# numbers say how big any one surface may get, this says how much total
# context the always-loaded tier may claim.
BUDGET_ALWAYS_LOADED_TOTAL = 49_152

# Surfaces that load unconditionally into every session (04-docs.md, AI tier).
ALWAYS_LOADED_FILES = ("CLAUDE.md", ".claude/CLAUDE.md")
ALWAYS_LOADED_RULES_PREFIX = ".claude/rules/"

ZONE_OK = "OK"
ZONE_WARN = "WARN"
ZONE_FAIL = "FAIL"
# Not a size zone: the aggregate could not be computed at all.
ZONE_PARTIAL = "PARTIAL"


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


def is_always_loaded(rel: str) -> bool:
    """True when a scanned surface loads into every session's context.

    CLAUDE.md (either location) and every .claude/rules/*.md file load
    unconditionally. project_log.md does not — it is first-read tier (an
    agent reads its head, not the file) and carries its own overflow-to-
    archive path — so it is excluded from the aggregate.
    """
    if rel in ALWAYS_LOADED_FILES:
        return True
    return rel.startswith(ALWAYS_LOADED_RULES_PREFIX) and rel.endswith(".md")


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


def probe(path: Path) -> tuple[str, str]:
    """Classify a surface path as 'file', 'missing', or 'error'.

    Path.is_file() collapses these three: it returns False for a path that
    does not exist AND for one whose stat() raises (ELOOP on a
    self-referential symlink, EACCES on an unreadable parent). That
    collapse is what let a present-but-unmeasurable surface leave the scan
    with no trace. Absent is normal; unmeasurable is an anomaly, and the
    caller must be able to tell them apart.
    """
    try:
        st = path.stat()
    except FileNotFoundError:
        return "missing", ""
    except OSError as exc:
        return "error", f"{exc.__class__.__name__}: {exc}"
    return ("file" if stat.S_ISREG(st.st_mode) else "missing"), ""


def collect_surfaces(root: Path) -> tuple[
    list[tuple[Path, int]], list[tuple[str, str, str]]
]:
    """((path, budget) pairs, skips) for the scanned surfaces under root.

    Missing surfaces are skipped silently — not every repo has every file.
    A surface that *is* present but cannot be measured is recorded as a
    skip (rel, kind, detail), kind being 'escaped' (resolves outside the
    repo root — junction/symlink) or 'error' (stat failed). Skips are what
    let the aggregate declare itself partial instead of quietly reporting
    a short total as authoritative.
    """
    surfaces: list[tuple[Path, int]] = []
    skips: list[tuple[str, str, str]] = []

    def rel_of(path: Path) -> str:
        try:
            return path.relative_to(root).as_posix()
        except ValueError:
            return str(path)

    def consider(path: Path, budget: int) -> None:
        kind, detail = probe(path)
        if kind == "missing":
            return
        if kind == "error":
            skips.append((rel_of(path), "error", detail))
            return
        if escapes_root(path, root):
            skips.append((rel_of(path), "escaped", "resolves outside the repo root"))
            return
        surfaces.append((path, budget))

    for path, budget in (
        (root / "CLAUDE.md", BUDGET_CLAUDE_MD),
        (root / ".claude" / "CLAUDE.md", BUDGET_CLAUDE_MD),
        (root / "project_log.md", BUDGET_PROJECT_LOG),
    ):
        consider(path, budget)

    rules_dir = root / ".claude" / "rules"
    if rules_dir.is_dir():
        if escapes_root(rules_dir, root):
            skips.append((
                f"{rel_of(rules_dir)}/*.md", "escaped",
                "rules directory resolves outside the repo root",
            ))
        else:
            for path in sorted(rules_dir.glob("*.md")):
                consider(path, BUDGET_RULES_MD)

    return surfaces, skips


def scan(root: Path) -> tuple[
    list[tuple[str, int, int, str]], list[str], list[tuple[str, str, str]]
]:
    """Scan surfaces under root.

    Returns (findings, warnings, skips): findings are (relative posix
    path, size bytes, budget bytes, zone) per readable surface; warnings
    are ready-to-print annotation lines; skips are (rel, kind, detail)
    for present-but-unmeasurable surfaces, carried so the aggregate can
    declare itself partial rather than report a short total as complete.
    """
    findings: list[tuple[str, int, int, str]] = []
    warnings: list[str] = []
    surfaces, skips = collect_surfaces(root)
    for rel, kind, detail in skips:
        suffix = f" ({detail})" if detail else ""
        warnings.append(
            f"::warning file={rel}::{rel} is a scanned surface but could not "
            f"be measured{suffix}; it is excluded from its per-file check and "
            f"from the always-loaded aggregate."
        )
    for path, budget in surfaces:
        rel = path.relative_to(root).as_posix()
        try:
            size = path.stat().st_size
        except OSError as exc:
            # Unreadable surfaces (permission denied, TOCTOU-vanished file)
            # are warned about and skipped rather than crashing the run. The
            # skip is recorded, not swallowed: a per-file check can lose one
            # verdict harmlessly, but the aggregate would otherwise report a
            # short total as authoritative.
            detail = f"{exc.__class__.__name__}: {exc}"
            warnings.append(
                f"::warning file={rel}::{rel} could not be read "
                f"({detail}); skipping its budget check."
            )
            skips.append((rel, "error", detail))
            continue
        findings.append((rel, size, budget, classify(size, budget)))
    return findings, warnings, skips


def run(root: Path, enforce: bool) -> tuple[int, list[str]]:
    """Scan `root` and build the report. Returns (exit_code, output_lines)."""
    lines: list[str] = []
    findings, unreadable, skips = scan(root)
    lines.extend(unreadable)
    failed = False

    for rel, size, budget, zone in findings:
        ratio = size / budget
        lines.append(
            f"[{zone:<4}] {rel} — {size:,} B / {budget:,} B budget ({ratio:.2f}x)"
        )
        if zone == ZONE_WARN:
            lines.append(
                f"::warning file={rel}::{rel} is over its context budget: "
                f"{size:,} B vs {budget:,} B ({ratio:.2f}x). Compress, or move "
                f"detail to the ENG tier (fails above 1.5x)."
            )
        elif zone == ZONE_FAIL:
            if enforce:
                failed = True
                lines.append(
                    f"::error file={rel}::{rel} exceeds 1.5x its context budget: "
                    f"{size:,} B vs {budget:,} B ({ratio:.2f}x). Compress, or move "
                    f"detail to the ENG tier."
                )
            else:
                lines.append(
                    f"::warning file={rel}::{rel} exceeds 1.5x its context budget: "
                    f"{size:,} B vs {budget:,} B ({ratio:.2f}x). Warn-only mode — "
                    f"this will fail once enforcement is on."
                )

    # Aggregate always-loaded budget. A surface that could not be measured
    # is excluded from the sum, so the total would otherwise read low — and
    # a low total is not a lost verdict like a per-file skip, it is a wrong
    # number presented as authoritative, able to turn a real FAIL into a
    # green WARN. So an incomplete always-loaded scan makes the aggregate
    # PARTIAL, and under --enforce it fails closed: better a red run naming
    # the unmeasurable file than a green one computed from a short sum.
    always_loaded = [f for f in findings if is_always_loaded(f[0])]
    missed = [sk for sk in skips if is_always_loaded(sk[0])
              or sk[0].startswith(ALWAYS_LOADED_RULES_PREFIX)]
    aggregate_zone = None
    if missed:
        measured = sum(f[1] for f in always_loaded)
        cap = BUDGET_ALWAYS_LOADED_TOTAL
        aggregate_zone = ZONE_PARTIAL
        named = ", ".join(sk[0] for sk in missed)
        lines.append(
            f"[{aggregate_zone:<4}] always-loaded total "
            f"({len(always_loaded)} surface(s) measured, {len(missed)} "
            f"unmeasurable) — {measured:,} B counted / {cap:,} B budget; "
            f"total is incomplete."
        )
        detail = (
            f"the always-loaded aggregate could not be computed: "
            f"{len(missed)} surface(s) present but unmeasurable ({named}). "
            f"The {measured:,} B counted is a floor, not the total"
        )
        if enforce:
            failed = True
            lines.append(f"::error::{detail}.")
        else:
            lines.append(
                f"::warning::{detail} — warn-only mode; this will fail once "
                f"enforcement is on."
            )
    elif always_loaded:
        total = sum(f[1] for f in always_loaded)
        cap = BUDGET_ALWAYS_LOADED_TOTAL
        aggregate_zone = classify(total, cap)
        ratio = total / cap
        lines.append(
            f"[{aggregate_zone:<4}] always-loaded total "
            f"({len(always_loaded)} surface(s)) — {total:,} B / {cap:,} B "
            f"budget ({ratio:.2f}x)"
        )
        remedy = (
            "Demote a surface to the grep-only tier (04-docs.md, AI tier), "
            "or compress"
        )
        if aggregate_zone == ZONE_WARN:
            lines.append(
                f"::warning::always-loaded context is over its aggregate "
                f"budget: {total:,} B vs {cap:,} B ({ratio:.2f}x). {remedy} "
                f"(fails above 1.5x)."
            )
        elif aggregate_zone == ZONE_FAIL:
            if enforce:
                failed = True
                lines.append(
                    f"::error::always-loaded context exceeds 1.5x its "
                    f"aggregate budget: {total:,} B vs {cap:,} B "
                    f"({ratio:.2f}x). {remedy}."
                )
            else:
                lines.append(
                    f"::warning::always-loaded context exceeds 1.5x its "
                    f"aggregate budget: {total:,} B vs {cap:,} B "
                    f"({ratio:.2f}x). Warn-only mode — this will fail once "
                    f"enforcement is on."
                )

    if not findings:
        lines.append("docs-budget: no budgeted surfaces found — nothing to check.")
    else:
        n_warn = sum(1 for f in findings if f[3] == ZONE_WARN)
        n_fail = sum(1 for f in findings if f[3] == ZONE_FAIL)
        mode = "enforce" if enforce else "warn-only"
        lines.append(
            f"docs-budget: {len(findings)} surface(s) checked, "
            f"{n_warn} warn-zone, {n_fail} fail-zone; always-loaded total "
            f"{aggregate_zone or 'n/a'} (mode: {mode})."
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
        _orig_collect = collect_surfaces
        def _collect_with_ghost(root: Path) -> tuple[
            list[tuple[Path, int]], list[tuple[str, str, str]]
        ]:
            surfaces, skips = _orig_collect(root)
            return surfaces + [(root / "project_log.md", BUDGET_PROJECT_LOG)], skips
        try:
            globals()["collect_surfaces"] = _collect_with_ghost
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
        _orig_collect_2 = collect_surfaces
        dropped_rel = ".claude/rules/01-r.md"

        def _collect_dropping_one(root: Path) -> tuple[
            list[tuple[Path, int]], list[tuple[str, str, str]]
        ]:
            surfaces, skips = _orig_collect_2(root)
            kept = [(pth, bud) for pth, bud in surfaces
                    if pth.relative_to(root).as_posix() != dropped_rel]
            return kept, skips + [(dropped_rel, "error", "OSError: simulated")]

        try:
            globals()["collect_surfaces"] = _collect_dropping_one
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
            globals()["collect_surfaces"] = _orig_collect_2

        # project_log.md is not always-loaded, so losing it must NOT make the
        # aggregate partial — the fail-closed rule is scoped to the tier the
        # aggregate actually measures.
        check("an unmeasurable non-always-loaded surface leaves the "
              "aggregate authoritative",
              not any("PARTIAL" in l for l in run(ghost_repo, enforce=True)[1]),
              f"got {[l for l in run(ghost_repo, enforce=True)[1] if 'always-loaded' in l]}")

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
            "budgets (.claude/rules/04-docs.md): CLAUDE.md and "
            ".claude/CLAUDE.md 12,288 B; each .claude/rules/*.md 8,192 B; "
            "project_log.md 15,360 B; and the always-loaded surfaces in "
            "aggregate (CLAUDE.md + rules, excluding project_log.md) "
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
