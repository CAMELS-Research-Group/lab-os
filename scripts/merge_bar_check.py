#!/usr/bin/env python3
"""merge-bar-check: PR template completeness.

Spec: docs/superpowers/specs/2026-06-10-logging-and-docs-standard-design.md, section 9.
Stdlib only; compatible with Python 3.11+.

What it checks
--------------
Section presence (always enforced): every required ``## <heading>`` from the
PR template must appear as a heading line in the PR body. GitHub keeps the
template's headings when the author fills it in, so a missing heading means
a deleted/unfilled section. Headings are read dynamically from the template
file passed via ``--template`` -- the template is authoritative. If that
path does not exist (e.g. a caller repo without a template), the built-in
fallback list below is used; it mirrors the lab template in
``.github/pull_request_template.md`` and must be kept in sync with it.

CLI
---
Check a PR::

    python scripts/merge_bar_check.py \
        --body-file pr_body.md \
        [--template .github/pull_request_template.md]

Output: one human-readable line per violation, each naming the rule
(``missing-section``). Exit 0 = pass, 1 = violations, 2 = usage/setup error.

Self-test (runs every fixture under ``tests/merge_bar_check/``)::

    python scripts/merge_bar_check.py --self-test

Parsing notes
-------------
Lines inside fenced code blocks (``` or ~~~) are ignored in both the template
and the PR body, so a heading quoted inside a code fence does not satisfy the
check. HTML-comment spans (``<!-- ... -->``, including multi-line comments)
are likewise stripped before matching: a heading that appears only inside a
comment counts for nothing, while text outside the span on the same line
still counts. Inside a fence, ``<!--`` is literal code, not a comment. Files
are read as UTF-8 (BOM tolerated); CRLF and LF both accepted.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# Fallback only -- the template file passed via --template is authoritative.
# Keep in sync with .github/pull_request_template.md.
FALLBACK_SECTIONS = (
    "Summary",
    "Type of change",
    "Changes",
    "Motivation / context",
    "Verification",
    "Checklist",
    "Related",
)

_HEADING_RE = re.compile(r"^##\s+(.+?)\s*$")
_FENCE_RE = re.compile(r"^\s*(```|~~~)")


def _norm(text: str) -> str:
    """Collapse internal whitespace runs; comparison stays case-sensitive."""
    return " ".join(text.split())


def _strip_comment_spans(line: str, in_comment: bool) -> tuple[str, bool]:
    """Remove HTML-comment spans (``<!-- ... -->``) from one line.

    ``in_comment`` carries multi-line comment state across lines. Text outside
    a comment span on the same line is kept, so ``foo <!-- bar --> baz``
    yields ``foo  baz``.
    """
    out: list[str] = []
    pos = 0
    while True:
        if in_comment:
            close = line.find("-->", pos)
            if close == -1:
                break
            pos = close + 3
            in_comment = False
        else:
            start = line.find("<!--", pos)
            if start == -1:
                out.append(line[pos:])
                break
            out.append(line[pos:start])
            pos = start + 4
            in_comment = True
    return "".join(out), in_comment


def _unfenced_lines(text: str):
    """Yield lines outside fenced code blocks (``` / ~~~), with HTML-comment
    spans stripped. Inside a fence, ``<!--`` is literal code, not a comment;
    inside a comment, a fence marker is comment text, not a fence."""
    fence = None
    in_comment = False
    for line in text.splitlines():
        if fence is not None:
            m = _FENCE_RE.match(line)
            if m and m.group(1) == fence:
                fence = None
            continue
        line, in_comment = _strip_comment_spans(line, in_comment)
        m = _FENCE_RE.match(line)
        if m:
            fence = m.group(1)
            continue
        yield line


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8-sig")


def template_sections(template_path: Path | None) -> tuple[list[str], str]:
    """Return (required headings, source description). Template file wins."""
    if template_path is not None and template_path.is_file():
        headings = [
            m.group(1)
            for line in _unfenced_lines(_read_text(template_path))
            if (m := _HEADING_RE.match(line))
        ]
        if headings:
            return headings, str(template_path)
    return list(FALLBACK_SECTIONS), "built-in fallback list"


def run_checks(body: str, sections: list[str]) -> list[str]:
    """Return one human-readable violation line per missing template section."""
    violations: list[str] = []

    body_headings = {
        _norm(m.group(1))
        for line in _unfenced_lines(body)
        if (m := _HEADING_RE.match(line))
    }
    for section in sections:
        if _norm(section) not in body_headings:
            violations.append(
                f'missing-section: required template heading "## {section}" '
                "not found in PR body"
            )

    return violations


# --- self-test ---------------------------------------------------------------

# case dir name -> (expected exit code, substrings each required in some violation)
SELF_TEST_EXPECTATIONS = {
    "pass_full_compliance": (0, ()),
    "fail_missing_section": (1, ("missing-section",)),
    # HTML-comment handling: a heading commented out must not satisfy the check.
    "fail_heading_in_comment": (1, ("missing-section",)),
    # Fence handling: a heading inside a fence must not satisfy the check either.
    "fail_fenced_content": (1, ("missing-section",)),
}


def self_test() -> int:
    repo_root = Path(__file__).resolve().parent.parent
    fixtures_dir = repo_root / "tests" / "merge_bar_check"
    template_path = repo_root / ".github" / "pull_request_template.md"

    sections, source = template_sections(template_path)
    if source == "built-in fallback list":
        print("self-test: ERROR: lab PR template not found at "
              f"{template_path}; self-test requires the real template")
        return 2

    # Drift guard: the built-in fallback must mirror the real template.
    drift_failed = tuple(sections) != FALLBACK_SECTIONS
    print(f"self-test: {'FAIL' if drift_failed else 'PASS'}  fallback-drift  "
          f"(FALLBACK_SECTIONS must equal template_sections() from {template_path})")
    if drift_failed:
        print(f"           template: {tuple(sections)!r}")
        print(f"           fallback: {FALLBACK_SECTIONS!r}")

    case_dirs = sorted(d for d in fixtures_dir.iterdir() if d.is_dir())
    found = {d.name for d in case_dirs}
    expected_names = set(SELF_TEST_EXPECTATIONS)
    if found != expected_names:
        print(f"self-test: ERROR: fixture set mismatch under {fixtures_dir}")
        for name in sorted(expected_names - found):
            print(f"  missing fixture: {name}")
        for name in sorted(found - expected_names):
            print(f"  unexpected fixture: {name}")
        return 2

    failures = 1 if drift_failed else 0
    for case in case_dirs:
        body = _read_text(case / "body.md")
        violations = run_checks(body, sections)
        exit_code = 1 if violations else 0

        want_code, want_substrs = SELF_TEST_EXPECTATIONS[case.name]
        ok = exit_code == want_code and all(
            any(s in v for v in violations) for s in want_substrs
        )
        status = "PASS" if ok else "FAIL"
        print(f"self-test: {status}  {case.name}  "
              f"(exit {exit_code}, want {want_code}"
              + "".join(f', violation containing "{s}"' for s in want_substrs)
              + ")")
        if not ok:
            failures += 1
            for v in violations:
                print(f"           got: {v}")

    total = len(case_dirs) + 1  # +1 for the fallback-drift guard
    print(f"self-test: {total - failures}/{total} cases passed "
          f"(template: {source})")
    return 1 if failures else 0


# --- entry point -------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="merge_bar_check.py",
        description="Verify PR body against the lab merge bar "
                    "(template sections). See module docstring for full "
                    "semantics.",
    )
    parser.add_argument("--body-file", help="file containing the PR body (markdown)")
    parser.add_argument("--template",
                        help="PR template to read required ## headings from "
                             "(authoritative; falls back to the built-in lab "
                             "list when the path does not exist)")
    parser.add_argument("--self-test", action="store_true",
                        help="run all fixtures under tests/merge_bar_check/")
    args = parser.parse_args(argv)

    if args.self_test:
        return self_test()

    if not args.body_file:
        parser.error("--body-file is required (or use --self-test)")

    body_path = Path(args.body_file)
    if not body_path.is_file():
        print(f"merge-bar-check: ERROR: file not found: {body_path}")
        return 2

    sections, source = template_sections(
        Path(args.template) if args.template else None
    )
    if args.template and source == "built-in fallback list":
        print(f"merge-bar-check: note: template {args.template!r} not found; "
              "using built-in fallback section list", file=sys.stderr)

    violations = run_checks(_read_text(body_path), sections)
    for v in violations:
        print(v)
    if violations:
        print(f"merge-bar-check: FAIL ({len(violations)} violation(s); "
              f"required sections from: {source})")
        return 1
    print(f"merge-bar-check: OK (required sections from: {source})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
