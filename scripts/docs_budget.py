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
that is present but unmeasurable (stat failure, a non-regular file, or a
path resolving outside the repo root) is excluded from the sum, so the
total would read low; the run then reports PARTIAL instead of a zone and, under
--enforce, fails closed. A per-file skip loses one verdict, but a
short aggregate is a wrong number presented as authoritative.

Zone semantics (size measured in bytes on disk):

    size <= budget                 -> OK    (no output beyond the report line)
    budget < size <= 1.5 * budget  -> WARN  (GitHub ::warning annotation, exit 0)
    size > 1.5 * budget            -> FAIL  (exit 1 when --enforce; otherwise
                                             downgraded to a warning, exit 0)

Default mode is warn-only until a repo first passes green, per
`.claude/rules/04-docs.md` (section Tiers & budgets, the same file the
budgets come from): the script always exits 0 no matter what it finds in
that mode. Pass --enforce
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
import os
import stat
import sys
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
    unconditionally — lab-owned `0x` rules and per-repo `10+` rules alike,
    because a session loads the directory, not a numbering range
    (04-docs.md, section Tiers & budgets, states the same scope).
    project_log.md does not — it is first-read tier (an agent reads its
    head, not the file) and carries its own overflow-to-archive path — so
    it is excluded from the aggregate.
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
    """Classify a surface path as 'file', 'dir', 'missing', or 'error'.

    Path.is_file()/is_dir() collapse these: they return False for a path
    that does not exist AND for one whose stat() raises (ELOOP on a
    self-referential symlink), while is_dir() *raises* on EACCES. That
    collapse is what let a present-but-unmeasurable surface leave the scan
    with no trace. Absent is normal; unmeasurable is an anomaly, and the
    caller must be able to tell them apart.

    This is the one classifier for both the file surfaces and the rules
    directory — the directory used to carry its own inline is_dir()/glob()
    re-implementation of the same decision, which is precisely where the
    unmeasurable case leaked back in.
    """
    try:
        st = path.stat()
    except FileNotFoundError:
        # A symlink whose target does not resolve is present but
        # unmeasurable, not absent: stat() follows the link and raises
        # ENOENT exactly as it does for a path that was never there.
        # islink() reads lstat and separates the two.
        if os.path.islink(path):
            return "error", "symlink target does not resolve"
        return "missing", ""
    except OSError as exc:
        return "error", f"{exc.__class__.__name__}: {exc}"
    if stat.S_ISREG(st.st_mode):
        return "file", ""
    if stat.S_ISDIR(st.st_mode):
        return "dir", ""
    # Present, stat-able, and neither a regular file nor a directory: a
    # FIFO, socket, or device node standing where a document belongs. It is
    # unmeasurable, not absent, and calling it "missing" dropped an
    # always-loaded surface with no skip recorded — exactly the short-sum-
    # as-authoritative failure the skip channel exists to prevent.
    return "error", "not a regular file"


def collect_surfaces(root: Path) -> tuple[
    list[tuple[Path, int]], list[tuple[str, str, str]]
]:
    """((path, budget) pairs, skips) for the scanned surfaces under root.

    Missing surfaces are skipped silently — not every repo has every file.
    A surface that *is* present but cannot be measured is recorded as a
    skip (rel, kind, detail), kind being 'escaped' (resolves outside the
    repo root — junction/symlink) or 'error' (stat failed, or the path
    is present but not a regular file). Skips are what
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
        if kind == "error":
            skips.append((rel_of(path), "error", detail))
            return
        if kind != "file":
            # Absent, or present but not a regular file (a directory in a
            # file's place). Neither is a measurable budgeted surface.
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
    # The directory goes through the same classifier as the files. is_dir()
    # returns False on a self-referential symlink (ELOOP) and raises on an
    # unreadable parent (EACCES), so an inline is_dir() test dropped the
    # entire always-loaded rules tier with no skip recorded — the aggregate
    # then reported a short sum as authoritative. Trailing slash on the rel:
    # it names the directory, and it is what makes the run()-side
    # ALWAYS_LOADED_RULES_PREFIX clause pick the skip up (it does not end
    # in .md, so is_always_loaded() alone would not).
    rules_rel = f"{rel_of(rules_dir)}/"
    kind, detail = probe(rules_dir)
    if kind == "error":
        skips.append((rules_rel, "error", detail))
    elif kind == "dir":
        if escapes_root(rules_dir, root):
            skips.append((
                f"{rel_of(rules_dir)}/*.md", "escaped",
                "rules directory resolves outside the repo root",
            ))
        else:
            try:
                entries = sorted(
                    Path(e.path) for e in os.scandir(rules_dir)
                    if e.name.endswith(".md")
                )
            except OSError as exc:
                # glob() discards the scandir error, so an unlistable
                # directory contributed zero surfaces AND zero skips — a
                # present directory became indistinguishable from an empty
                # one. os.scandir surfaces the errno so it becomes a skip.
                skips.append((
                    rules_rel, "error", f"{exc.__class__.__name__}: {exc}",
                ))
            else:
                for path in entries:
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
    # Doctrine, stated once here because this is where it bites: EVERY
    # unmeasurable always-loaded surface fails closed, with no exemption by
    # skip kind and none by surface shape. An "escaped" skip (resolves
    # outside the repo root) and an "error" skip (stat/scandir failed) are
    # the same class to this computation, and an unmeasurable rules
    # DIRECTORY counts exactly like an unmeasurable rules file — losing the
    # whole tier is the worse of the two, not the excusable one. Any of
    # them makes the aggregate PARTIAL and, under --enforce, red.
    #
    # The prefix clause is not redundant with is_always_loaded(): its one
    # input is the rules-directory skip, whose rel is the bare directory
    # (".claude/rules/") and so does not end in ".md". Every other skip
    # under the prefix names a file and is caught by the first clause.
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

    # Gated on skips too: a repo whose only always-loaded surface is
    # unmeasurable has no findings, and closing that run with "nothing to
    # check" states the inverse of its own PARTIAL line and exit code.
    if not findings and not skips:
        lines.append("docs-budget: no budgeted surfaces found — nothing to check.")
    else:
        n_warn = sum(1 for f in findings if f[3] == ZONE_WARN)
        n_fail = sum(1 for f in findings if f[3] == ZONE_FAIL)
        mode = "enforce" if enforce else "warn-only"
        # Only stated when there is something to state, so a clean run's
        # output is unchanged.
        unmeasurable = f", {len(skips)} unmeasurable" if skips else ""
        lines.append(
            f"docs-budget: {len(findings)} surface(s) checked, "
            f"{n_warn} warn-zone, {n_fail} fail-zone{unmeasurable}; "
            f"always-loaded total {aggregate_zone or 'n/a'} (mode: {mode})."
        )

    return (1 if failed else 0, lines)


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
            "-> FAIL (exit 1 only with --enforce). Missing surfaces are "
            "skipped silently; an always-loaded surface that is present but "
            "unmeasurable, or that resolves outside the repo root "
            "(junctions/symlinks), makes the aggregate PARTIAL and fails "
            "under --enforce."
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
        # The fixtures and assertions live in a sibling module so this file
        # stays under the Class-1 1,000-line budget
        # (reference/code-quality-taxonomy.md), the same split
        # backlog_lint.py already uses. `--self-test` stays the one entry
        # point and its CLI contract is unchanged.
        from docs_budget_selftest import run_self_test
        return run_self_test()

    if not args.root.is_dir():
        print(f"docs-budget: root is not a directory: {args.root}", file=sys.stderr)
        return 2

    code, lines = run(args.root, enforce=args.enforce)
    for line in lines:
        print(line)
    return code


if __name__ == "__main__":
    sys.exit(main())
