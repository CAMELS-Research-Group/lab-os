#!/usr/bin/env python3
"""log-lint: mechanical adherence checks for lab project logs.

Implements the log-lint Action from the logging & docs standard
(docs/superpowers/specs/2026-06-10-logging-and-docs-standard-design.md §9,
§4.3-4.6). Parses the normative template structure
(templates/project_log.template.md) and checks a changed project_log.md
against the target-branch baseline.

CLI
---
    python3 scripts/log_lint.py --changed project_log.md \
        [--baseline <path>] [--archive <path>]
    python3 scripts/log_lint.py --self-test

Baseline semantics (spec §9): the baseline is the PR target branch's current
HEAD version of the log — NOT the merge-base (three-dot baselines misread the
mandated rebase/conflict-resolution flow as foreign additions). A missing,
empty, or omitted baseline file means "no baseline": the repo is adopting the
standard, every entry is new, immutability checks are vacuous, and format
checks still apply.

EOL handling: every input file is normalized (CRLF/CR -> LF) on read, before
any comparison or byte count. "Byte-identical modulo EOL normalization" for
archive reappearance is therefore implemented as equality of normalized entry
blocks. This also makes the committed fixtures immune to git EOL translation
on any platform (no .gitattributes needed): a CRLF checkout produces the same
verdicts, and the EOL-sensitive archive case is exercised by self-test cases
that generate CRLF variants programmatically at runtime.

Per-entry budget: an entry block is measured as the UTF-8 byte length of the
normalized block — header line plus body, LF line endings, trailing blank and
"---" separator lines stripped. The budget applies to NEW entries only:
pre-existing entries are immutable (§4.4), so flagging them would create an
unfixable permanent failure.

Violations (one line each on stdout: "<file>: <violation>: <detail>")
---------------------------------------------------------------------
  malformed-structure         not exactly one "## Standing Decisions" followed
                              by exactly one "## Entries"
  file-missing                the --changed path does not exist
  file-not-utf8               an input file is not valid UTF-8
  baseline-unparseable        baseline structure could not be located;
                              immutability checks were skipped
  malformed-entry-header      a "## " line in the entries region that does not
                              match "## YYYY-MM-DD HH:MM <em-dash> <subject>"
                              (or has an impossible timestamp)
  duplicate-entry-header      two entries share the same header key
  entry-separator-missing     entry header not preceded by "---" on its own
                              line (nearest non-blank line above)
  new-entries-out-of-order    the new head block is not non-strict descending
                              by header timestamp
  entry-inserted-mid-history  a new entry appears below a pre-existing entry
  preexisting-entry-modified  an entry present in the baseline has a different
                              body in the changed file
  entry-removed-not-archived  an entry left the log without reappearing in the
                              archive byte-identical modulo EOL normalization
  entry-over-budget           a new entry block exceeds 1,500 bytes
  index-line-malformed        an index line starting "- <timestamp>" that does
                              not match the index grammar
                              "- YYYY-MM-DD HH:MM <em-dash> <subject> <middle-dot> <ref>"
  index-key-unmatched         an index line whose "date+time <em-dash> subject"
                              key matches no entry header in the hot window or
                              the archive

The Standing Decisions index region is exempt from immutability checks by
design (supersession edits it). The whole-file 15 KB cap is docs-budget's
job, not log-lint's.

Override convention (enforced by .github/workflows/log-lint.yml, not by this
script): a PR carrying the `log-lint:override` label skips lint enforcement,
but the PR body must contain a line starting `log-lint:override:` followed by
a non-empty reason (e.g. "log-lint:override: migrating legacy log"), or the
workflow job fails.

Exit codes: 0 clean / self-test green; 1 violations / self-test failures;
2 usage error. Python 3.11 stdlib only.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tempfile
from collections import Counter
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import List, Optional, Tuple

EM_DASH = "—"     # —
MIDDLE_DOT = "·"  # ·
ENTRY_BUDGET_BYTES = 1500
SD_HEADING = "## Standing Decisions"
ENTRIES_HEADING = "## Entries"

# HARD CONSTRAINT: all grammar regexes are digit-anchored. The normative
# template's HTML comments contain literal "YYYY-MM-DD HH:MM" grammar
# examples; digit anchoring makes those lines inert without parsing or
# stripping comments.
TIMESTAMP_PAT = r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}"
ENTRY_HEADER_RE = re.compile(rf"^## ({TIMESTAMP_PAT}) {EM_DASH} (.+)$")
# Detection: a line that *claims* to be an index line (leading "- " plus a
# digit-anchored timestamp). Claimed-but-ungrammatical lines are violations;
# anything else in the index region (comments, blanks, prose) is ignored.
INDEX_DETECT_RE = re.compile(rf"^- {TIMESTAMP_PAT}")
# Greedy subject + final non-empty ref field: the subject/ref split happens at
# the LAST " <middle-dot> ", so subjects containing the separator still parse.
# The ref is only required to be non-empty (template grammar shows
# "#<PR-or-archive-link>", but archive re-pointing legitimately produces refs
# that are not bare PR numbers, so the leading "#" is not enforced).
INDEX_LINE_RE = re.compile(
    rf"^- ({TIMESTAMP_PAT}) {EM_DASH} (.+) {MIDDLE_DOT} (\S.*)$"
)


@dataclass
class Entry:
    key: str       # "YYYY-MM-DD HH:MM — <subject>" (verbatim index key)
    ts: str        # "YYYY-MM-DD HH:MM" (validated; lexicographic == chronological)
    subject: str
    block: str     # normalized block: header + body, trailing blanks/--- stripped
    line_no: int   # 1-based header line number in the source file


Violation = Tuple[str, str, str]  # (file label, violation name, detail)


def _valid_ts(ts: str) -> bool:
    try:
        datetime.strptime(ts, "%Y-%m-%d %H:%M")
        return True
    except ValueError:
        return False


def read_normalized(path: Path) -> str:
    """Read a file as UTF-8 and normalize all line endings to LF.

    Decodes with utf-8-sig so a leading UTF-8 BOM (common from Windows
    editors) is stripped instead of corrupting the first line into a
    malformed-structure false positive.
    """
    return path.read_bytes().decode("utf-8-sig").replace("\r\n", "\n").replace("\r", "\n")


def find_structure(lines: List[str]) -> Optional[Tuple[int, int]]:
    """Locate the two load-bearing H2 anchors. Exactly one of each, SD first."""
    sd = [i for i, line in enumerate(lines) if line == SD_HEADING]
    en = [i for i, line in enumerate(lines) if line == ENTRIES_HEADING]
    if len(sd) != 1 or len(en) != 1 or sd[0] > en[0]:
        return None
    return sd[0], en[0]


def _separator_above(lines: List[str], header_idx: int) -> bool:
    """True if the nearest non-blank line above the header is '---'."""
    for j in range(header_idx - 1, -1, -1):
        stripped = lines[j].strip()
        if stripped:
            return stripped == "---"
    return False


def parse_entries(
    lines: List[str],
    start: int,
    label: str,
    violations: List[Violation],
    strict: bool,
) -> List[Entry]:
    """Parse an entries region (or a whole archive file when start == 0).

    strict=True (the changed log) emits format violations; strict=False
    (baseline, archive) parses leniently — those files are not the artifact
    under review.

    Every line starting "## " is a block boundary, so content under a
    malformed header never bleeds into the previous entry's block (which
    would false-positive preexisting-entry-modified).
    """
    boundaries: List[Tuple[int, Optional[str], Optional[str]]] = []
    for i in range(start, len(lines)):
        line = lines[i]
        if line.startswith("## "):
            m = ENTRY_HEADER_RE.match(line)
            if m and _valid_ts(m.group(1)):
                boundaries.append((i, m.group(1), m.group(2)))
            else:
                boundaries.append((i, None, None))
                if strict:
                    violations.append((label, "malformed-entry-header", line))

    entries: List[Entry] = []
    seen = set()
    for n, (i, ts, subject) in enumerate(boundaries):
        if ts is None:
            continue
        end = boundaries[n + 1][0] if n + 1 < len(boundaries) else len(lines)
        block_lines = list(lines[i:end])
        # The trailing "---" belongs to the NEXT entry's separator; strip it
        # and surrounding blanks off this block's tail.
        while block_lines and block_lines[-1].strip() in ("", "---"):
            block_lines.pop()
        key = f"{ts} {EM_DASH} {subject}"
        if key in seen:
            if strict:
                violations.append((label, "duplicate-entry-header", key))
            continue
        seen.add(key)
        if strict and not _separator_above(lines, i):
            violations.append((label, "entry-separator-missing", key))
        entries.append(Entry(key, ts, subject, "\n".join(block_lines), i + 1))
    return entries


def parse_index(
    lines: List[str],
    start: int,
    end: int,
    label: str,
    violations: List[Violation],
) -> List[str]:
    """Return the index keys of well-formed index lines; flag malformed ones."""
    keys: List[str] = []
    for i in range(start, end):
        line = lines[i]
        if not INDEX_DETECT_RE.match(line):
            continue
        m = INDEX_LINE_RE.match(line)
        if not m or not _valid_ts(m.group(1)):
            violations.append((label, "index-line-malformed", line))
            continue
        keys.append(f"{m.group(1)} {EM_DASH} {m.group(2)}")
    return keys


def lint(
    changed_path: Path,
    baseline_path: Optional[Path],
    archive_path: Optional[Path],
) -> List[Violation]:
    violations: List[Violation] = []
    clabel = str(changed_path)

    if not changed_path.exists():
        violations.append((clabel, "file-missing", "changed log file does not exist"))
        return violations
    try:
        ctext = read_normalized(changed_path)
    except UnicodeDecodeError as exc:
        violations.append((clabel, "file-not-utf8", str(exc)))
        return violations

    clines = ctext.split("\n")
    structure = find_structure(clines)
    if structure is None:
        violations.append((
            clabel,
            "malformed-structure",
            f"expected exactly one '{SD_HEADING}' followed by exactly one '{ENTRIES_HEADING}'",
        ))
        return violations
    sd_idx, en_idx = structure

    index_keys = parse_index(clines, sd_idx + 1, en_idx, clabel, violations)
    centries = parse_entries(clines, en_idx + 1, clabel, violations, strict=True)

    # --- baseline (target-branch HEAD). Missing/empty => no baseline:
    # every entry is new; immutability checks are vacuous.
    bentries: List[Entry] = []
    if baseline_path is not None and baseline_path.exists():
        try:
            btext = read_normalized(baseline_path)
        except UnicodeDecodeError as exc:
            violations.append((str(baseline_path), "file-not-utf8", str(exc)))
            btext = ""
        if btext.strip():
            blines = btext.split("\n")
            bstructure = find_structure(blines)
            if bstructure is None:
                violations.append((
                    str(baseline_path),
                    "baseline-unparseable",
                    "baseline structure not found; immutability checks skipped",
                ))
            else:
                bentries = parse_entries(
                    blines, bstructure[1] + 1, str(baseline_path), violations, strict=False
                )

    # --- archive (lenient parse, no structural requirements: grep-only tier).
    aentries: List[Entry] = []
    if archive_path is not None and archive_path.exists():
        try:
            atext = read_normalized(archive_path)
            aentries = parse_entries(
                atext.split("\n"), 0, str(archive_path), violations, strict=False
            )
        except UnicodeDecodeError as exc:
            violations.append((str(archive_path), "file-not-utf8", str(exc)))

    bmap = {e.key: e for e in bentries}
    amap = {e.key: e for e in aentries}
    ckeys = {e.key for e in centries}

    # --- placement: a PR's new entries form one contiguous block at the head
    # of the entries region (spec §4.5). Any new entry below the first
    # pre-existing entry is mid-history.
    first_old = next(
        (pos for pos, e in enumerate(centries) if e.key in bmap), None
    )
    head_block = centries if first_old is None else centries[:first_old]
    if first_old is not None:
        for e in centries[first_old:]:
            if e.key not in bmap:
                violations.append((clabel, "entry-inserted-mid-history", e.key))

    # --- order: the new head block must be non-strict descending by header
    # timestamp (ties allowed). The boundary against the pre-existing head is
    # deliberately unchecked: §4.5 requires the new block to be *internally*
    # date-ordered.
    for above, below in zip(head_block, head_block[1:]):
        if above.ts < below.ts:
            violations.append((
                clabel,
                "new-entries-out-of-order",
                f"'{above.key}' precedes newer '{below.key}'",
            ))

    # --- immutability: pre-existing entries survive unchanged.
    for e in centries:
        old = bmap.get(e.key)
        if old is not None and old.block != e.block:
            violations.append((clabel, "preexisting-entry-modified", e.key))

    # --- removal: an entry deleted from the log must reappear in the archive
    # byte-identical modulo EOL normalization (both blocks are normalized on
    # read, so plain equality implements "modulo EOL").
    for old in bentries:
        if old.key in ckeys:
            continue
        archived = amap.get(old.key)
        if archived is None or archived.block != old.block:
            violations.append((clabel, "entry-removed-not-archived", old.key))

    # --- per-entry byte budget, new entries only (pre-existing entries are
    # immutable, so an old over-budget entry must not permanently fail CI).
    for e in centries:
        if e.key in bmap:
            continue
        size = len(e.block.encode("utf-8"))
        if size > ENTRY_BUDGET_BYTES:
            violations.append((
                clabel,
                "entry-over-budget",
                f"{e.key} ({size} bytes > {ENTRY_BUDGET_BYTES})",
            ))

    # --- index <-> entry key match: every index line must key to an entry
    # header in the hot window or the archive. (The index region is exempt
    # from immutability by design — supersession edits it.)
    known_keys = ckeys | set(amap)
    for key in index_keys:
        if key not in known_keys:
            violations.append((clabel, "index-key-unmatched", key))

    return violations


# ---------------------------------------------------------------------------
# self-test: runs every fixture pair under tests/log_lint/ (manifest-driven)
# plus programmatic EOL-variant cases, and asserts the expected verdicts.
# CI and humans share this single entry point.
# ---------------------------------------------------------------------------

def _fmt_counts(counts: Counter) -> str:
    """Render a violation-name multiset readably: 'name x2, other x1'."""
    return ", ".join(f"{k} x{n}" for k, n in sorted(counts.items())) or "(none)"


def _run_case(
    name: str,
    changed: Path,
    baseline: Optional[Path],
    archive: Optional[Path],
    expect: List[str],
) -> bool:
    # Compare the full MULTISET of violation names (not a deduplicated set):
    # a fixture that must emit the same violation twice fails if it emits it
    # once, and vice versa.
    got = Counter(v[1] for v in lint(changed, baseline, archive))
    want = Counter(expect)
    if got == want:
        print(f"ok   {name}")
        return True
    print(f"FAIL {name}: expected [{_fmt_counts(want)}], got [{_fmt_counts(got)}]")
    return False


def self_test() -> int:
    fixtures = Path(__file__).resolve().parent.parent / "tests" / "log_lint"
    manifest = json.loads((fixtures / "manifest.json").read_text(encoding="utf-8"))

    total = 0
    failures = 0
    for case in manifest["cases"]:
        total += 1
        baseline = fixtures / case["baseline"] if case.get("baseline") else None
        archive = fixtures / case["archive"] if case.get("archive") else None
        if not _run_case(
            case["name"], fixtures / case["changed"], baseline, archive, case["expect"]
        ):
            failures += 1

    # EOL-variant cases are generated at runtime rather than committed:
    # this repo has no .gitattributes, so committed CRLF bytes would not be
    # stable across clones/platforms. Generating them here keeps the
    # byte-level "modulo EOL" behavior tested deterministically everywhere.
    with tempfile.TemporaryDirectory() as td:
        tdir = Path(td)

        # 1) archive content CRLF, removed entry LF: must still count as
        #    byte-identical modulo EOL normalization.
        src = read_normalized(fixtures / "pass_archive_move" / "archive.md")
        crlf_archive = tdir / "archive_crlf.md"
        crlf_archive.write_bytes(src.replace("\n", "\r\n").encode("utf-8"))
        total += 1
        if not _run_case(
            "pass_archive_move_crlf_archive",
            fixtures / "pass_archive_move" / "changed.md",
            fixtures / "_shared" / "baseline.md",
            crlf_archive,
            [],
        ):
            failures += 1

        # 2) changed log itself in CRLF: verdicts must be EOL-invariant.
        src = read_normalized(fixtures / "pass_new_entries_head" / "changed.md")
        crlf_changed = tdir / "changed_crlf.md"
        crlf_changed.write_bytes(src.replace("\n", "\r\n").encode("utf-8"))
        total += 1
        if not _run_case(
            "pass_new_entries_head_crlf_changed",
            crlf_changed,
            fixtures / "_shared" / "baseline.md",
            None,
            [],
        ):
            failures += 1

        # 3) changed log with a UTF-8 BOM (Windows editors): read_normalized
        #    decodes utf-8-sig, so the BOM must not corrupt the title line
        #    into a malformed-structure false positive.
        src = read_normalized(fixtures / "pass_new_entries_head" / "changed.md")
        bom_changed = tdir / "changed_bom.md"
        bom_changed.write_bytes(b"\xef\xbb\xbf" + src.encode("utf-8"))
        total += 1
        if not _run_case(
            "pass_new_entries_head_bom_changed",
            bom_changed,
            fixtures / "_shared" / "baseline.md",
            None,
            [],
        ):
            failures += 1

        # 4) baseline path that does not exist (new-adoption case as the
        #    workflow invokes it): all entries new, format checks still apply.
        total += 1
        if not _run_case(
            "pass_missing_baseline_path",
            fixtures / "pass_new_file" / "changed.md",
            tdir / "no-such-baseline.md",
            None,
            [],
        ):
            failures += 1

    if failures:
        print(f"self-test: {failures}/{total} cases FAILED")
        return 1
    print(f"self-test: {total} cases, all passed")
    return 0


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(
        prog="log_lint.py",
        description="Mechanical lint for lab project logs (spec §9).",
        epilog=(
            "Override convention (enforced by the workflow, not this script): "
            "a PR labeled 'log-lint:override' skips lint enforcement, but its "
            "body must contain a line starting 'log-lint:override:' followed "
            "by a non-empty reason, or the workflow job fails."
        ),
    )
    parser.add_argument("--changed", type=Path, help="changed project_log.md (the PR version)")
    parser.add_argument(
        "--baseline",
        type=Path,
        default=None,
        help="target-branch HEAD version of the log; missing/empty file = no "
        "baseline (all entries treated as new)",
    )
    parser.add_argument(
        "--archive",
        type=Path,
        default=None,
        help="project_log_archive.md; missing file = empty archive",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="run every fixture pair under tests/log_lint/ and assert verdicts",
    )
    args = parser.parse_args(argv)

    # Violation details contain em-dashes; never let a non-UTF-8 console
    # (Windows redirection) crash the report itself.
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(errors="replace")

    if args.self_test:
        return self_test()
    if args.changed is None:
        parser.error("--changed is required (or use --self-test)")

    violations = lint(args.changed, args.baseline, args.archive)
    for label, name, detail in violations:
        print(f"{label}: {name}: {detail}")
    if violations:
        return 1
    print(f"{args.changed}: log-lint clean")
    return 0


if __name__ == "__main__":
    sys.exit(main())
