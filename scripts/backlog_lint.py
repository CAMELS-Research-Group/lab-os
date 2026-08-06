#!/usr/bin/env python3
"""backlog-lint: hygiene check for the lab-wide BACKLOG.md.

Spec: docs/prds/backlog-lint.md (B5). Enforces the backlog-item convention
(templates/backlog-item.template.md) so the Index stays a trustworthy
"what's ready" surface — the readiness bar becomes an invariant CI guarantees
instead of a rule a groomer must remember.

Checks (errors → exit 1 under --enforce; warn-only otherwise, matching the
docs-budget warn-until-first-green posture):

  - every Item block carries the required fields (parsed from the item
    template, the single source — a missing or empty template fails the run
    outright rather than silently disabling the check)
  - `Done when` is present, not a `<placeholder>`, and a single condition:
    wrapped continuation lines are joined into one value; a nested list
    inside the field (a second bulleted/numbered condition) is an error
  - `Status` is one of: inbox | ready | in-progress | done
  - `Rough size` is one of: S | M | L, and an `L` item is never `ready`
    (split it first)
  - the Index is a GENERATED projection of the Item blocks: the committed
    Index table must byte-match render_index(items). Edit the Item blocks,
    then run `--write-index`; never hand-edit the Index. (This subsumes the
    old orphan-row / missing-row / duplicate-Index-id checks.) `--write-index`
    refuses to run while the file has structural parse errors — regenerating
    from a partial parse would silently drop the unparsed blocks' rows.
  - `Depends on` referential integrity (ids exist) + acyclicity (no cycle)
  - structural integrity of the Items section: a `## ` heading that is not a
    well-formed item heading, and non-blank text no item or field owns (a
    severed continuation, prose between a heading and the first field, text
    between `## Items` and the first item heading, or any line inside a
    block whose heading failed to parse), are hard errors — nothing silently
    attaches to the wrong item or escapes the checks below
  - public-tier spot check over Items (titles + full field values, including
    continuation lines and unattached text) AND the Inbox section.
    Intentionally conservative:
    the patterns match only the literal phrases "private repo" /
    "gated dataset" and POSIX home paths (`/Users/...`) — clear leaks only.
    They do NOT catch private github.com URLs, real gated-dataset prefixes,
    or Windows-style paths; the scan is a tripwire, not a guarantee.

Warnings (never fail): a `Done when` with no concrete artifact reference
(a command / file / observable behaviour) — structural checks stay hard,
concreteness is only nudged, so a legitimate item is never red over phrasing.

Importable API, designed for reuse by the follow-on view/digest tools
(PR #68) so there is one parser and one dependency grammar, not three:
parse_backlog, parse_deps, required_fields, render_index.

Stdlib only; Python 3.11+.
"""
from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

STATUSES = ("inbox", "ready", "in-progress", "done")
SIZES = ("S", "M", "L")
_DEFAULT_BACKLOG = "BACKLOG.md"
_DEFAULT_TEMPLATE = "templates/backlog-item.template.md"

# Public-tier spot check: obvious private surfaces that must not appear in a
# public backlog. Intentionally conservative — flags clear leaks only (the
# literal phrases and POSIX home paths; see the docstring for what it misses).
_PRIVATE_PATTERNS = (
    re.compile(r"\bprivate[-_ ]repo\b", re.I),
    re.compile(r"/Users/[^/\s]+/"),          # absolute home paths
    re.compile(r"\bgated[-_ ]dataset\b", re.I),
)


@dataclass
class Item:
    id: str
    fields: dict[str, str]        # values with continuation lines joined by " "
    raw_fields: dict[str, str]    # values with continuation lines preserved ("\n"-joined)
    line: int                     # 1-based line of the "## <id> —" heading


@dataclass
class Backlog:
    index: list[dict[str, str]]   # rows: id, title, owner, size, status
    index_table: list[str]        # the committed Index table lines, verbatim
    items: list[Item]
    inbox: list[tuple[int, str]]  # (1-based line, text) for every Inbox content line
    raw: str
    parse_errors: list[str] = field(default_factory=list)         # structural defects (hard errors)
    orphans: list[tuple[int, str]] = field(default_factory=list)  # unattached Items-section lines


@dataclass
class Findings:
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)

    def err(self, msg: str) -> None:
        self.errors.append(msg)

    def warn(self, msg: str) -> None:
        self.warnings.append(msg)


# --- parsing -----------------------------------------------------------------

_ITEM_HEAD = re.compile(r"^##\s+(B\d+)\s+—\s+(.*\S)\s*$")
_FIELD = re.compile(r"^-\s+\*\*(?P<key>[^:*]+):\*\*\s*(?P<val>.*)$")
_LIST_MARKER = re.compile(r"^\s*([-*+]|\d+[.)])\s+")


def required_fields(template_text: str) -> list[str]:
    """The required field labels, parsed from the item template (single source).

    Callers must treat an empty result as an error (fail closed): a template
    that parses to zero fields means the schema source is broken, not that
    no fields are required.
    """
    fields: list[str] = []
    for line in template_text.splitlines():
        m = _FIELD.match(line)
        if m:
            fields.append(m.group("key").strip())
    return fields


def parse_deps(raw: str) -> list[str]:
    """Parse a `Depends on` value into item ids.

    The dependency grammar, designed for reuse by the follow-on view/digest
    tools (PR #68) so there is one copy, not three: "—" / "-" / "" / "none"
    mean no dependencies; otherwise every `B<number>` token is a dependency.
    """
    return [] if raw.strip() in ("—", "-", "", "none") else re.findall(r"B\d+", raw)


def _parse_index(lines: list[str]) -> tuple[list[dict[str, str]], list[str]]:
    """Parse the Index section: (rows, verbatim table lines)."""
    rows: list[dict[str, str]] = []
    table: list[str] = []
    in_index = False
    for line in lines:
        if line.strip() == "## Index":
            in_index = True
            continue
        if in_index and line.startswith("## "):
            break
        if not in_index or not line.strip().startswith("|"):
            continue
        table.append(line)
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) != 5 or cells[0] in ("id", "---") or set(cells[0]) == {"-"}:
            continue
        rows.append(dict(zip(("id", "title", "owner", "size", "status"), cells)))
    return rows, table


def parse_backlog(text: str) -> Backlog:
    """Parse BACKLOG.md into its Index rows, Inbox lines, and Item blocks.

    Field values consume continuation lines (until the next `- **` field, the
    next `## ` heading, or a blank line): `fields` holds them joined with a
    space; `raw_fields` keeps the original multi-line text so structural rules
    (single-condition `Done when`) can see line breaks. Importable.
    """
    lines = text.splitlines()
    index, index_table = _parse_index(lines)

    items: list[Item] = []
    inbox: list[tuple[int, str]] = []
    parse_errors: list[str] = []
    orphans: list[tuple[int, str]] = []
    section = ""
    cur: Item | None = None
    cur_key: str | None = None

    def _finish_field() -> None:
        nonlocal cur_key
        cur_key = None

    for i, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped in ("## Index", "## Inbox", "## Items"):
            section = stripped
            cur, cur_key = None, None
            continue

        if section == "## Inbox":
            if line.startswith("## "):
                section = ""
                continue
            if stripped:
                inbox.append((i, line))
            continue

        if section != "## Items":
            continue

        head = _ITEM_HEAD.match(line)
        if head:
            cur = Item(id=head.group(1), fields={"__title__": head.group(2)},
                       raw_fields={}, line=i)
            items.append(cur)
            _finish_field()
            continue
        if line.startswith("## "):
            # An Items-section heading that is not a well-formed item heading.
            # Sever attachment AND record a hard error: otherwise the fields
            # that follow would silently attach to the previous item, or the
            # whole block would vanish without a trace.
            parse_errors.append(f"unrecognized item heading at line {i}: {stripped}")
            cur, cur_key = None, None
            continue
        if not stripped:
            _finish_field()
            continue
        if cur is None:
            # Non-blank Items-section text no item owns: prose between
            # `## Items` and the first item heading, or any line inside a
            # block whose heading failed to parse. Hard error + orphan
            # record — same treatment as owned-line errors below, so the
            # leak scan still covers this text instead of dropping it.
            parse_errors.append(f"unattached text in Items section at line {i}")
            orphans.append((i, line))
            continue
        fm = _FIELD.match(line)
        if fm:
            key = fm.group("key").strip()
            cur.fields[key] = fm.group("val").strip()
            cur.raw_fields[key] = fm.group("val").strip()
            cur_key = key
            continue
        if cur_key is not None:
            cur.fields[cur_key] = (cur.fields[cur_key] + " " + stripped).strip()
            cur.raw_fields[cur_key] = cur.raw_fields[cur_key] + "\n" + line
        else:
            # Non-blank text no field owns (a blank line severed a
            # continuation, or prose sits between the heading and the first
            # field): hard error — this text would otherwise escape every
            # field-level check, including the leak scan.
            parse_errors.append(f"unattached text under item {cur.id} at line {i}")
            orphans.append((i, line))
    return Backlog(index=index, index_table=index_table, items=items,
                   inbox=inbox, raw=text, parse_errors=parse_errors,
                   orphans=orphans)


# --- derived Index -----------------------------------------------------------

def _cell(v: str) -> str:
    """Escape a value for a Markdown table cell: a literal `|` would split
    the row into extra cells, so it is escaped as `\\|` (renders as `|`)."""
    return v.replace("|", "\\|")


def render_index(items: list[Item]) -> str:
    """Render the Index table from the Item blocks (the Index is derived).

    Single source: the Item blocks. `title` comes from the item heading,
    `owner` / `size` / `status` from the Owner / Rough size / Status fields.
    Literal `|` in any value is escaped so the table shape survives.
    """
    out = ["| id | title | owner | size | status |", "|---|---|---|---|---|"]
    for it in items:
        size = it.fields.get("Rough size", "").split()
        out.append("| {} | {} | {} | {} | {} |".format(
            it.id,
            _cell(it.fields.get("__title__", "")),
            _cell(it.fields.get("Owner", "")),
            _cell(size[0] if size else ""),
            _cell(it.fields.get("Status", "")),
        ))
    return "\n".join(out)


# --- linting -----------------------------------------------------------------

def _is_placeholder(v: str) -> bool:
    v = v.strip()
    return not v or (v.startswith("<") and v.endswith(">"))


def _is_multi_condition(raw: str) -> bool:
    """True when a field's raw value contains a nested list — i.e. more than
    one condition. Plain wrapped prose (indented continuation lines) is fine."""
    return any(_LIST_MARKER.match(ln) for ln in raw.splitlines()[1:])


def _has_artifact_ref(v: str) -> bool:
    # A concrete check: backticked token, a path, a URL, or a filename-ish word.
    return bool(re.search(r"`[^`]+`|https?://|[\w./-]+\.\w{2,4}\b|\b\w+\.(py|md|yml|sh)\b", v))


def lint(backlog: Backlog, req_fields: list[str]) -> Findings:
    f = Findings()
    ids_seen: dict[str, int] = {}

    # Structural parse defects (unrecognized headings, unattached text) are
    # hard errors — a file that cannot be attributed line-by-line must not
    # pass as merely "fields look fine".
    for e in backlog.parse_errors:
        f.err(e)

    for it in backlog.items:
        where = f"{it.id} (line {it.line})"
        if it.id in ids_seen:
            f.err(f"{where}: duplicate item id (also at line {ids_seen[it.id]})")
        ids_seen[it.id] = it.line

        for key in req_fields:
            if key not in it.fields:
                f.err(f"{where}: missing required field '{key}'")
            elif key != "Done when" and _is_placeholder(it.fields[key]):
                # `Done when` has its own dedicated check below; every other
                # required field must also be filled — an empty value or a
                # verbatim `<placeholder>` is not a filled field.
                f.err(f"{where}: required field '{key}' is empty or a placeholder")

        dw = it.fields.get("Done when", "")
        dw_raw = it.raw_fields.get("Done when", dw)
        if _is_placeholder(dw):
            f.err(f"{where}: 'Done when' is empty or a placeholder")
        elif _is_multi_condition(dw_raw):
            f.err(f"{where}: 'Done when' must be a single condition "
                  "(wrapped prose is fine; a nested list is not)")
        elif not _has_artifact_ref(dw):
            f.warn(f"{where}: 'Done when' has no concrete artifact reference (command / file / behaviour)")

        st = it.fields.get("Status", "")
        if st and st not in STATUSES:
            f.err(f"{where}: Status '{st}' not one of {STATUSES}")

        size = it.fields.get("Rough size", "")
        size_tok = size.split()[0] if size else ""
        if size_tok and size_tok not in SIZES:
            f.err(f"{where}: Rough size '{size_tok}' not one of {SIZES}")
        if size_tok == "L" and st == "ready":
            f.err(f"{where}: size L must be split before it can be 'ready'")

        blob = " ".join(it.fields.values())
        for pat in _PRIVATE_PATTERNS:
            if pat.search(blob):
                f.err(f"{where}: item embeds a non-public reference (matched /{pat.pattern}/)")

    # Inbox is the one surface anyone may write to — leak-scan it too.
    for lineno, text in backlog.inbox:
        for pat in _PRIVATE_PATTERNS:
            if pat.search(text):
                f.err(f"Inbox (line {lineno}): embeds a non-public reference "
                      f"(matched /{pat.pattern}/)")

    # Unattached Items-section text is already a hard parse error above; scan
    # it for leaks too, so a non-public reference in a severed line is named
    # as such rather than hiding behind the structural error.
    for lineno, text in backlog.orphans:
        for pat in _PRIVATE_PATTERNS:
            if pat.search(text):
                f.err(f"Items (line {lineno}): unattached text embeds a "
                      f"non-public reference (matched /{pat.pattern}/)")

    # The Index is derived from the Item blocks: the committed table must
    # byte-match the render (stale, hand-edited, orphan, or missing rows all
    # surface as a diff).
    committed = backlog.index_table
    rendered = render_index(backlog.items).splitlines()
    if committed != rendered:
        for n, (c, r) in enumerate(zip(committed, rendered), start=1):
            if c != r:
                detail = (f"first difference at table line {n}: "
                          f"committed {c!r} vs rendered {r!r}")
                break
        else:
            n = min(len(committed), len(rendered))
            if len(committed) > len(rendered):
                detail = f"first extra committed line: {committed[n]!r}"
            else:
                detail = f"first missing line (from the render): {rendered[n]!r}"
        f.err(f"Index is stale ({detail}): it is generated from the Item "
              "blocks — edit the blocks, then run "
              "`python3 scripts/backlog_lint.py --write-index`")

    # Depends on: referential integrity + acyclicity
    item_ids = set(ids_seen)
    deps: dict[str, list[str]] = {}
    for it in backlog.items:
        refs = parse_deps(it.fields.get("Depends on", "—"))
        deps[it.id] = refs
        for r in refs:
            if r not in item_ids:
                f.err(f"{it.id}: Depends on '{r}' which is not a backlog item")
    cyc = _first_cycle(deps)
    if cyc:
        f.err(f"Depends on: dependency cycle {' -> '.join(cyc)}")

    return f


def _first_cycle(graph: dict[str, list[str]]) -> list[str] | None:
    WHITE, GREY, BLACK = 0, 1, 2
    color = {n: WHITE for n in graph}
    stack: list[str] = []

    def dfs(n: str) -> list[str] | None:
        color[n] = GREY
        stack.append(n)
        for m in graph.get(n, []):
            if m not in color:
                continue
            if color[m] == GREY:
                return stack[stack.index(m):] + [m]
            if color[m] == WHITE:
                r = dfs(m)
                if r:
                    return r
        stack.pop()
        color[n] = BLACK
        return None

    for n in graph:
        if color[n] == WHITE:
            r = dfs(n)
            if r:
                return r
    return None


# --- CLI ---------------------------------------------------------------------

def _fail(msg: str) -> int:
    print(f"::error::backlog-lint: {msg}" if _in_ci() else f"ERROR {msg}")
    return 1


def _load_schema(template_path: Path) -> list[str] | None:
    """Required-field schema from the template. None = unusable (fail closed).

    Any read failure (missing file, permission denied) is "unusable" — the
    caller reports one clean error line instead of a traceback."""
    try:
        text = template_path.read_text()
    except OSError:
        return None
    req = required_fields(text)
    return req or None


def _run(backlog_path: Path, template_path: Path, enforce: bool) -> int:
    if not backlog_path.exists():
        print(f"backlog-lint: {backlog_path} not found; nothing to check.")
        return 0
    req = _load_schema(template_path)
    if req is None:
        # Fail closed regardless of --enforce: a missing/empty schema source is
        # a code/infra defect, not a content finding — green here would mean
        # the whole required-field dimension silently vanished.
        return _fail(f"template {template_path} is missing, unreadable, or "
                     "defines no `- **Field:**` lines — required-field "
                     "schema unavailable")
    findings = lint(parse_backlog(backlog_path.read_text()), req)
    for w in findings.warnings:
        print(f"::warning::backlog-lint: {w}" if _in_ci() else f"WARN  {w}")
    for e in findings.errors:
        print(f"::error::backlog-lint: {e}" if _in_ci() else f"ERROR {e}")
    if not findings.errors and not findings.warnings:
        print("backlog-lint: OK")
    if findings.errors and enforce:
        return 1
    return 0


def _write_index(backlog_path: Path) -> int:
    if not backlog_path.exists():
        return _fail(f"{backlog_path} not found")
    text = backlog_path.read_text()
    parsed = parse_backlog(text)
    if parsed.parse_errors:
        # Refuse to regenerate from a source that did not fully parse: the
        # render sees only the blocks the parser owned, so writing it would
        # silently delete the committed Index row of every block behind a
        # structural error — and report success. Fix the structure first.
        listing = "; ".join(parsed.parse_errors)
        return _fail(f"refusing to regenerate: {len(parsed.parse_errors)} "
                     f"structural error(s) must be fixed first: {listing}")
    lines = text.splitlines(keepends=True)
    start = end = None
    in_index = False
    for i, line in enumerate(lines):
        if line.strip() == "## Index":
            in_index = True
            continue
        if in_index and line.startswith("## "):
            break
        if in_index and line.strip().startswith("|"):
            if start is None:
                start = i
            end = i + 1
    if start is None:
        return _fail("no Index table found under `## Index` — add the header "
                     "row by hand once, then --write-index maintains it")
    rendered = render_index(parsed.items) + "\n"
    new = "".join(lines[:start]) + rendered + "".join(lines[end:])
    backlog_path.write_text(new)
    print(f"backlog-lint: wrote derived Index to {backlog_path}")
    return 0


def _in_ci() -> bool:
    import os
    return bool(os.environ.get("GITHUB_ACTIONS"))


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="Lint the lab-wide BACKLOG.md.")
    ap.add_argument("--check", default=_DEFAULT_BACKLOG, help="path to BACKLOG.md")
    ap.add_argument("--template", default=_DEFAULT_TEMPLATE, help="item template (field source)")
    ap.add_argument("--enforce", action="store_true", help="exit 1 on errors (default: warn-only)")
    ap.add_argument("--write-index", action="store_true",
                    help="regenerate the (derived) Index table from the Item blocks")
    ap.add_argument("--self-test", action="store_true", help="run built-in fixtures and exit")
    args = ap.parse_args(argv)
    if args.self_test:
        return _self_test()
    if args.write_index:
        return _write_index(Path(args.check))
    return _run(Path(args.check), Path(args.template), args.enforce)


# --- self test (fixtures prove each rule bites) ------------------------------

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


def _self_test() -> int:  # noqa: C901 - a linear fixture list reads best flat
    ok = True

    # The schema comes from the real template — no hardcoded fallback list, so
    # a template edit that breaks the field regex fails HERE, loudly.
    req = required_fields(_repo_template().read_text())

    def expect(name: str, cond: bool, detail: str = "") -> None:
        nonlocal ok
        if not cond:
            ok = False
        print(f"  [{'ok' if cond else 'FAIL'}] {name}" + (f" -> {detail}" if not cond else ""))

    def check(name: str, text: str, want_err_substr: str | None) -> None:
        errs = lint(parse_backlog(text), req).errors
        hit = any(want_err_substr in e for e in errs) if want_err_substr else not errs
        expect(name, hit, str(errs))

    expect("template schema is non-empty (fail-closed source)",
           bool(req) and "Done when" in req and "Owner" in req, str(req))

    good = lint(parse_backlog(_GOOD), req)
    expect("good backlog is clean (no errors)", not good.errors, str(good.errors))
    expect("wrapped Done-when joins fully (no false concreteness warning)",
           not good.warnings, str(good.warnings))
    b2 = [i for i in parse_backlog(_GOOD).items if i.id == "B2"][0]
    expect("continuation line captured in parsed value",
           b2.fields["Done when"].endswith("`docs/y.md` with the new section"),
           b2.fields["Done when"])

    check("missing field", _GOOD.replace("- **Value:** worth it\n", ""), "missing required field 'Value'")
    check("placeholder Done when", _GOOD.replace("`scripts/x.py --self-test` passes", "<the check>"), "placeholder")
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
    check("dangling depends", _GOOD.replace("- **Depends on:** B1", "- **Depends on:** B7"), "not a backlog item")
    check("dependency cycle", _GOOD.replace("- **Depends on:** —", "- **Depends on:** B2"), "cycle")
    check("private path leak", _GOOD.replace("something is missing", "see /Users/someone/secret/x"), "non-public")
    check("private-repo phrase leak", _GOOD.replace("another gap", "tracked in the private repo"), "non-public")
    check("gated-dataset phrase leak", _GOOD.replace("worth it", "unblocks the gated dataset work"), "non-public")
    check("leak on a continuation line is caught",
          _GOOD.replace("  `docs/y.md` with the new section", "  `docs/y.md` under /Users/someone/private/"),
          "non-public")
    check("Inbox leak is caught",
          _GOOD.replace("- a raw idea waiting to be shaped", "- idea parked at /Users/someone/notes/idea.md"),
          "Inbox")

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

    with tempfile.TemporaryDirectory() as td:
        tdp = Path(td)
        good_b, bad_b = tdp / "GOOD.md", tdp / "BAD.md"
        good_b.write_text(_GOOD)
        bad_b.write_text(_GOOD.replace("`scripts/x.py --self-test` passes", "<the check>"))
        tpl = str(_repo_template())
        expect("main: clean backlog exits 0",
               main(["--check", str(good_b), "--template", tpl]) == 0)
        expect("main: errors without --enforce exit 0 (warn-only)",
               main(["--check", str(bad_b), "--template", tpl]) == 0)
        expect("main: errors with --enforce exit 1",
               main(["--check", str(bad_b), "--template", tpl, "--enforce"]) == 1)
        expect("main: missing template exits 1 (fail closed)",
               main(["--check", str(good_b), "--template", str(tdp / "no-such.md")]) == 1)
        empty_tpl = tdp / "empty-template.md"
        empty_tpl.write_text("# a template with no field lines\n")
        expect("main: template with zero fields exits 1 (fail closed)",
               main(["--check", str(good_b), "--template", str(empty_tpl)]) == 1)
        # --write-index round-trip: hand-broken Index regenerates to clean.
        stale = tdp / "STALE.md"
        stale.write_text(_GOOD.replace("| B1 | First | Kiara | S | ready |",
                                       "| B1 | WRONG | Nobody | L | done |"))
        expect("--write-index repairs a stale Index",
               _write_index(stale) == 0
               and not lint(parse_backlog(stale.read_text()), req).errors)
        piped_b = tdp / "PIPED.md"
        piped_b.write_text(_GOOD.replace("## B1 — First", "## B1 — First | piped"))
        expect("--write-index handles | in a title; result lints clean",
               _write_index(piped_b) == 0
               and not lint(parse_backlog(piped_b.read_text()), req).errors)
        # Blocker 2 (PR #67 review): --write-index must refuse on structural
        # parse errors instead of silently deleting the unparsed blocks' rows.
        mal = tdp / "MALFORMED.md"
        mal_text = _GOOD.replace("## B2 — Second", "## B2 - Second")
        mal.write_text(mal_text)
        rc, out = run_main(["--check", str(mal), "--write-index"])
        expect("--write-index on a structurally broken backlog exits non-zero",
               rc == 1, out)
        expect("--write-index refusal names the structural error",
               "refusing to regenerate" in out
               and "unrecognized item heading" in out, out)
        expect("--write-index leaves the broken file byte-identical",
               mal.read_text() == mal_text)
        unread_tpl = tdp / "unreadable-template.md"
        unread_tpl.write_text("- **Field:** x\n")
        unread_tpl.chmod(0)
        expect("main: unreadable template exits 1 (one-line failure, no traceback)",
               main(["--check", str(good_b), "--template", str(unread_tpl)]) == 1)
        unread_tpl.chmod(0o644)

    print("backlog-lint self-test:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
