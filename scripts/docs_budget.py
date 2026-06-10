#!/usr/bin/env python3
"""docs-budget: byte-size budget check for always-loaded AI doc surfaces.

Spec: docs/superpowers/specs/2026-06-10-logging-and-docs-standard-design.md
(section 7.2 context budgets, section 9 Adherence Actions).

Scanned paths (relative to --root; each skipped silently when absent):

    CLAUDE.md             budget  8,192 B
    .claude/CLAUDE.md     budget  8,192 B   (some repos keep it here instead)
    .claude/rules/*.md    budget  5,120 B each
    project_log.md        budget 15,360 B

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
dir from lab-rules; that surface belongs to lab-rules' own budget run).

Stdlib only; compatible with Python 3.11+.
"""

from __future__ import annotations

import argparse
import sys
import tempfile
from pathlib import Path

BUDGET_CLAUDE_MD = 8_192
BUDGET_RULES_MD = 5_120
BUDGET_PROJECT_LOG = 15_360

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
    return findings, warnings


def run(root: Path, enforce: bool) -> tuple[int, list[str]]:
    """Scan `root` and build the report. Returns (exit_code, output_lines)."""
    lines: list[str] = []
    findings, unreadable = scan(root)
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

    print("docs_budget self-test")

    # --- 1. zone classification boundaries -------------------------------
    print("zone classification:")
    check("size == budget is OK", classify(8_192, BUDGET_CLAUDE_MD) == ZONE_OK)
    check("budget + 1 is WARN", classify(8_193, BUDGET_CLAUDE_MD) == ZONE_WARN)
    check("size == 1.5x budget is WARN", classify(12_288, BUDGET_CLAUDE_MD) == ZONE_WARN)
    check("1.5x budget + 1 is FAIL", classify(12_289, BUDGET_CLAUDE_MD) == ZONE_FAIL)
    check("rules boundary 7,680 is WARN", classify(7_680, BUDGET_RULES_MD) == ZONE_WARN)
    check("rules 7,681 is FAIL", classify(7_681, BUDGET_RULES_MD) == ZONE_FAIL)
    check("log boundary 23,040 is WARN", classify(23_040, BUDGET_PROJECT_LOG) == ZONE_WARN)
    check("log 23,041 is FAIL", classify(23_041, BUDGET_PROJECT_LOG) == ZONE_FAIL)

    # --- 2. static fixture: under-budget repo ----------------------------
    print("static under-budget fixture:")
    fixtures = Path(__file__).resolve().parent.parent / "tests" / "docs_budget"
    under = fixtures / "under_budget_repo"
    findings, _ = scan(under)
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
                "CLAUDE.md": 9_000,            # 8,192 < size <= 12,288
                ".claude/CLAUDE.md": 9_000,    # the alternate location is scanned
                ".claude/rules/01-r.md": 6_000,  # 5,120 < size <= 7,680
                "project_log.md": 20_000,      # 15,360 < size <= 23,040
            },
        )
        findings, _ = scan(warn_repo)
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

        # --- 4. generated fail-zone repo (> 1.5x) -------------------------
        print("generated fail-zone repo:")
        fail_repo = _build_repo(
            tmp / "fail_repo",
            {
                "CLAUDE.md": 12_289,
                ".claude/rules/01-r.md": 7_681,
                "project_log.md": 23_041,
            },
        )
        findings, _ = scan(fail_repo)
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
              any("[FAIL] CLAUDE.md — 12,289 B / 8,192 B budget" in l for l in lines_e))

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
            findings, _ = scan(repo)
            check("rules dir resolving outside repo is skipped",
                  [f[0] for f in findings] == ["CLAUDE.md"],
                  f"got {[f[0] for f in findings]}")
        else:
            print("  [SKIP] symlink creation unavailable on this platform; "
                  "pure-comparison checks above cover the logic")

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
            "budgets (spec section 7.2): CLAUDE.md and .claude/CLAUDE.md "
            "8,192 B; each .claude/rules/*.md 5,120 B; project_log.md "
            "15,360 B. Zones: size <= budget -> OK; budget < size <= "
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
