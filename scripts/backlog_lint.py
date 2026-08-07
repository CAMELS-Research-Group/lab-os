#!/usr/bin/env python3
"""backlog-lint: hygiene check for the lab-wide BACKLOG.md.

Spec: docs/prds/backlog-lint.md (B5). Enforces the backlog-item convention
(templates/backlog-item.template.md) so the Index stays a trustworthy
"what's ready" surface — the readiness bar becomes an invariant CI guarantees
instead of a rule a groomer must remember.

Checks (errors → exit 1 under --enforce; warn-only otherwise, matching the
docs-budget warn-until-first-green posture — EXCEPT tooling defects: an
unusable required-field schema or an unreadable BACKLOG.md exits 1 regardless
of --enforce, since green there would mean a check dimension silently
vanished):

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
    "gated dataset" and Unix home paths (`/Users/...`, `/home/...`) — clear
    leaks only. They do NOT catch private github.com URLs, real gated-dataset
    prefixes, or Windows-style paths; the scan is a tripwire, not a
    guarantee (the self-test pins those three misses as characterization
    cases so the boundary stays visible).

Warnings (never fail): a `Done when` with no concrete artifact reference
(a command / file / observable behaviour) — structural checks stay hard,
concreteness is only nudged, so a legitimate item is never red over phrasing.

Importable API, designed for reuse by the follow-on view/digest tools
(PR #68) so there is one parser and one dependency grammar, not three:
parse_backlog, parse_deps, required_fields, render_index, and the Backlog
dataclass. On Backlog, `items` is the source of truth; `index` /
`index_table` are the COMMITTED Index rows exactly as parsed — exposed so
lint() can byte-compare them against render_index(items), and unverified
until it does. Consumers who need trustworthy rows should derive them from
`items`, not trust `index`.

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

# Semantic field labels the rules below key on, named once. _load_schema
# asserts every one of them is present in the template-derived schema, so a
# label rename in the template fails closed instead of silently disarming the
# Status/size/deps/Done-when rules while the required-field check stays green.
F_OWNER = "Owner"
F_SIZE = "Rough size"
F_DONE = "Done when"
F_DEPS = "Depends on"
F_STATUS = "Status"
_SEMANTIC_FIELDS = (F_OWNER, F_SIZE, F_DONE, F_DEPS, F_STATUS)

# `Depends on` sentinels meaning "no dependencies" (shared by parse_deps and
# the unparseable-value check in lint).
_DEP_NONE = ("—", "-", "", "none")

# Public-tier spot check: obvious private surfaces that must not appear in a
# public backlog. Intentionally conservative — flags clear leaks only (the
# literal phrases and Unix home paths; see the docstring for what it misses).
_PRIVATE_PATTERNS = (
    re.compile(r"\bprivate[-_ ]repo\b", re.I),
    re.compile(r"/Users/[^/\s]+/"),          # absolute home paths (macOS)
    re.compile(r"/home/[^/\s]+/"),           # absolute home paths (Linux, incl. CI runners)
    re.compile(r"\bgated[-_ ]dataset\b", re.I),
)


@dataclass
class Item:
    id: str
    title: str                    # from the heading — a field line cannot forge it
    fields: dict[str, str]        # values with continuation lines joined by " "
    raw_fields: dict[str, str]    # values with continuation lines preserved ("\n"-joined)
    line: int                     # 1-based line of the "## <id> —" heading


@dataclass
class Backlog:
    # `items` is the source of truth; `index`/`index_table` are the COMMITTED
    # Index exactly as parsed — unverified until lint() byte-compares the
    # table against render_index(items). Derive trustworthy rows from `items`.
    raw: str                      # the full source text, verbatim — a LIVE field:
                                  # the view/digest tools (PR #68) read `raw.strip()`
                                  # to tell a truly-empty backlog from one that
                                  # merely parsed to zero Item blocks (truncated /
                                  # destroyed source), a distinction `items`/`index`
                                  # alone cannot make. (Removed as dead in the #67
                                  # review round; re-added here with that real
                                  # consumer, so it is no longer a dead public field.)
    index: list[dict[str, str]]   # committed rows: id, title, owner, size, status
    index_table: list[str]        # the committed Index table lines, verbatim
    items: list[Item]
    inbox: list[tuple[int, str]]  # (1-based line, text) for every Inbox content line
    parse_errors: list[str] = field(default_factory=list)         # structural defects (hard errors)
    orphans: list[tuple[int, str]] = field(default_factory=list)  # unattached Items-section lines


@dataclass
class Findings:
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)


# --- parsing -----------------------------------------------------------------

_ITEM_HEAD = re.compile(r"^##\s+(B\d+)\s+—\s+(.*\S)\s*$")
_FIELD = re.compile(r"^-\s+\*\*(?P<key>[^:*]+):\*\*\s*(?P<val>.*)$")
_LIST_MARKER = re.compile(r"^\s*([-*+]|\d+[.)])\s+")


def required_fields(template_text: str) -> list[str]:
    """The required field labels, parsed from the item template (single source).

    HTML comments are stripped first, so an example `- **Field:**` line in
    the template's guidance comment cannot silently grow the schema.
    Callers must treat an empty result as an error (fail closed): a template
    that parses to zero fields means the schema source is broken, not that
    no fields are required.
    """
    fields: list[str] = []
    for line in re.sub(r"<!--.*?-->", "", template_text, flags=re.S).splitlines():
        m = _FIELD.match(line)
        if m:
            fields.append(m.group("key").strip())
    return fields


def parse_deps(raw: str) -> list[str]:
    """Parse a `Depends on` value into item ids.

    The dependency grammar, designed for reuse by the follow-on view/digest
    tools (PR #68) so there is one copy, not three: "—" / "-" / "" / "none"
    mean no dependencies; otherwise every `B<number>` token is a dependency.
    A value that is neither a sentinel nor contains `B<number>` tokens also
    returns [] — lint() flags that case as an error, so a typo'd id cannot
    silently read as "unblocked". Callers that skip lint() inherit the gap.
    """
    return [] if raw.strip() in _DEP_NONE else re.findall(r"B\d+", raw)


def _uncell(v: str) -> str:
    """Inverse of _cell: a table cell's `\\|` renders as a literal `|`."""
    return v.replace("\\|", "|")


def _parse_index(lines: list[str]) -> tuple[list[dict[str, str]], list[str]]:
    """Parse the Index section: (rows, verbatim table lines).

    Cells split on UNESCAPED `|` only (and are then unescaped), mirroring
    render_index/_cell which escape a literal `|` in a value as `\\|`. Splitting
    on every raw `|` would over-split an escaped-pipe title into extra cells and
    silently drop the row — the lint↔view disagreement PR #68's review flagged
    (a title like `Ready \\| thing` lints clean but vanishes from `index`).
    """
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
        # Split on `|` not preceded by a backslash, then unescape each cell, so
        # an escaped-pipe value stays one cell and reads back as its literal.
        cells = [_uncell(c.strip()) for c in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
        if len(cells) != 5 or cells[0] in ("id", "---") or set(cells[0]) == {"-"}:
            continue
        rows.append(dict(zip(("id", "title", "owner", "size", "status"), cells)))
    return rows, table


def parse_backlog(text: str) -> Backlog:
    """Parse BACKLOG.md into its Index rows, Inbox lines, and Item blocks.

    Field values consume continuation lines (until the next `- **` field, the
    next `## ` heading, or a blank line): `fields` holds them joined with a
    space; `raw_fields` keeps the original multi-line text so structural rules
    (single-condition `Done when`) can see line breaks. Structural defects
    land in `parse_errors` (hard errors in lint()) with the offending lines
    in `orphans` (leak-scanned); a Backlog whose `parse_errors` is non-empty
    did NOT fully parse and must not be treated as complete. Importable.
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
            # An item heading OUTSIDE the Items section is an unambiguous
            # structural defect: the Items section is renamed/misspelled or
            # missing, so its item blocks parse to zero and their committed
            # Index rows would be silently dropped by --write-index. Record a
            # hard error so --check never goes green after such a wipe, however
            # the section went missing. (In a well-formed file every item
            # heading sits under `## Items`, so this never fires on a valid
            # backlog — the guard above already consumed those.)
            if _ITEM_HEAD.match(line):
                parse_errors.append(
                    f"item heading outside Items section at line {i}: {stripped}")
            continue

        head = _ITEM_HEAD.match(line)
        if head:
            cur = Item(id=head.group(1), title=head.group(2), fields={},
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
            if key in cur.fields:
                # A second `- **Key:**` line in one item: last-write-wins would
                # let lint and render_index see a different value than a
                # top-down human reader sees (the first). Fail closed — matching
                # the module's "ambiguous input errors" posture — and keep the
                # first value so the render stays deterministic under the error.
                parse_errors.append(
                    f"duplicate field '{key}' under item {cur.id} at line {i}")
                cur_key = key
                continue
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
    return Backlog(raw=text, index=index, index_table=index_table, items=items,
                   inbox=inbox, parse_errors=parse_errors, orphans=orphans)


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
        size = it.fields.get(F_SIZE, "").split()
        out.append("| {} | {} | {} | {} | {} |".format(
            it.id,
            _cell(it.title),
            _cell(it.fields.get(F_OWNER, "")),
            _cell(size[0] if size else ""),
            _cell(it.fields.get(F_STATUS, "")),
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
        f.errors.append(e)

    for it in backlog.items:
        where = f"{it.id} (line {it.line})"
        if it.id in ids_seen:
            f.errors.append(f"{where}: duplicate item id (also at line {ids_seen[it.id]})")
        ids_seen[it.id] = it.line

        for key in req_fields:
            if key not in it.fields:
                f.errors.append(f"{where}: missing required field '{key}'")
            elif key != F_DONE and _is_placeholder(it.fields[key]):
                # `Done when` has its own dedicated check below; every other
                # required field must also be filled — an empty value or a
                # verbatim `<placeholder>` is not a filled field.
                f.errors.append(f"{where}: required field '{key}' is empty or a placeholder")

        dw = it.fields.get(F_DONE, "")
        dw_raw = it.raw_fields.get(F_DONE, dw)
        if _is_placeholder(dw):
            f.errors.append(f"{where}: 'Done when' is empty or a placeholder")
        elif _is_multi_condition(dw_raw):
            f.errors.append(f"{where}: 'Done when' must be a single condition "
                            "(wrapped prose is fine; a nested list is not)")
        elif not _has_artifact_ref(dw):
            f.warnings.append(f"{where}: 'Done when' has no concrete artifact reference (command / file / behaviour)")

        st = it.fields.get(F_STATUS, "")
        if st and st not in STATUSES:
            f.errors.append(f"{where}: Status '{st}' not one of {STATUSES}")

        size = it.fields.get(F_SIZE, "")
        size_tok = size.split()[0] if size else ""
        if size_tok and size_tok not in SIZES:
            f.errors.append(f"{where}: Rough size '{size_tok}' not one of {SIZES}")
        if size_tok == "L" and st == "ready":
            f.errors.append(f"{where}: size L must be split before it can be 'ready'")

        blob = " ".join((it.title, *it.fields.values()))
        for pat in _PRIVATE_PATTERNS:
            if pat.search(blob):
                f.errors.append(f"{where}: item embeds a non-public reference (matched /{pat.pattern}/)")

    # Inbox is the one surface anyone may write to — leak-scan it too.
    for lineno, text in backlog.inbox:
        for pat in _PRIVATE_PATTERNS:
            if pat.search(text):
                f.errors.append(f"Inbox (line {lineno}): embeds a non-public reference "
                                f"(matched /{pat.pattern}/)")

    # Unattached Items-section text is already a hard parse error above; scan
    # it for leaks too, so a non-public reference in a severed line is named
    # as such rather than hiding behind the structural error.
    for lineno, text in backlog.orphans:
        for pat in _PRIVATE_PATTERNS:
            if pat.search(text):
                f.errors.append(f"Items (line {lineno}): unattached text embeds a "
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
        f.errors.append(f"Index is stale ({detail}): it is generated from the Item "
                        "blocks — edit the blocks, then run "
                        "`python3 scripts/backlog_lint.py --write-index`")

    # Depends on: value understood + referential integrity + acyclicity
    item_ids = set(ids_seen)
    deps: dict[str, list[str]] = {}
    for it in backlog.items:
        raw_dep = it.fields.get(F_DEPS, "—")
        refs = parse_deps(raw_dep)
        deps[it.id] = refs
        # parse_deps returns [] both for the no-deps sentinels and for a
        # value it could not read (`TBD`, a lowercase `b12`) — the latter
        # must be an error, or a typo reads as "unblocked" here and in
        # every tool that reuses the grammar.
        leftover = re.sub(r"B\d+", "", raw_dep).strip(" \t,;")
        if raw_dep.strip() not in _DEP_NONE and leftover:
            f.errors.append(f"{it.id}: 'Depends on' value {raw_dep.strip()!r} is neither "
                            "a no-deps sentinel (— / none) nor `B<n>` ids")
        for r in refs:
            if r not in item_ids:
                f.errors.append(f"{it.id}: Depends on '{r}' which is not a backlog item")
    cyc = _first_cycle(deps)
    if cyc:
        f.errors.append(f"Depends on: dependency cycle {' -> '.join(cyc)}")

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


def _load_schema(template_path: Path) -> tuple[list[str] | None, str]:
    """Required-field schema from the template: (fields, "") or (None, why).

    None = unusable (fail closed). The cause is returned rather than
    discarded, so the caller's one-line error names WHICH failure happened
    (unreadable / no field lines / missing semantic label) instead of making
    the operator guess among them."""
    try:
        text = template_path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as e:
        return None, f"cannot be read ({e})"
    req = required_fields(text)
    if not req:
        return None, "defines no `- **Field:**` lines"
    missing = [x for x in _SEMANTIC_FIELDS if x not in req]
    if missing:
        # A renamed label would keep the required-field check green while
        # every rule keyed on the old name silently stopped firing.
        return None, ("is missing semantic label(s) the lint rules key on: "
                      + ", ".join(missing))
    return req, ""


def _annotation(kind: str, msg: str, path: Path) -> str:
    """A GitHub annotation anchored to the backlog file (and line, when the
    finding names one), so it reaches the PR's Files-changed view — in the
    warn-only posture the annotation IS the signal."""
    # Anchor to a FILE line only. The stale-Index error says "first difference
    # at table line N" — that N indexes the Index table, not the file, so a
    # bare `line (\d+)` would anchor the annotation to the wrong file line.
    # Exclude any "line N" immediately preceded by "table ".
    m = re.search(r"(?<!table )line (\d+)", msg)
    where = f"file={path}" + (f",line={m.group(1)}" if m else "")
    return f"::{kind} {where}::backlog-lint: {msg}"


def _run(backlog_path: Path, template_path: Path, enforce: bool) -> int:
    if not backlog_path.exists():
        print(f"backlog-lint: {backlog_path} not found; nothing to check.")
        return 0
    req, why = _load_schema(template_path)
    if req is None:
        # Fail closed regardless of --enforce: an unusable schema source is
        # a code/infra defect, not a content finding — green here would mean
        # the whole required-field dimension silently vanished.
        return _fail(f"template {template_path} {why} — required-field "
                     "schema unavailable")
    try:
        text = backlog_path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as e:
        # Fail closed, one clean line: an unreadable backlog is a tooling
        # defect, not "no findings" (UnicodeDecodeError is not an OSError).
        return _fail(f"cannot read {backlog_path}: {e}")
    findings = lint(parse_backlog(text), req)
    for w in findings.warnings:
        print(_annotation("warning", w, backlog_path) if _in_ci() else f"WARN  {w}")
    for e in findings.errors:
        print(_annotation("error", e, backlog_path) if _in_ci() else f"ERROR {e}")
    if not findings.errors and not findings.warnings:
        print("backlog-lint: OK")
    if findings.errors and enforce:
        return 1
    return 0


def _write_index(backlog_path: Path) -> int:
    if not backlog_path.exists():
        return _fail(f"{backlog_path} not found")
    try:
        text = backlog_path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as e:
        return _fail(f"cannot read {backlog_path}: {e}")
    parsed = parse_backlog(text)
    if parsed.parse_errors:
        # Refuse to regenerate from a source that did not fully parse: the
        # render sees only the blocks the parser owned, so writing it would
        # silently delete the committed Index row of every block behind a
        # structural error — and report success. Fix the structure first.
        listing = "; ".join(parsed.parse_errors)
        return _fail(f"refusing to regenerate: {len(parsed.parse_errors)} "
                     f"structural error(s) must be fixed first: {listing}")
    if not parsed.items and parsed.index:
        # Belt to Fix B's suspenders: the source parsed to ZERO items but the
        # committed Index still carries data rows. Regenerating here would
        # render an empty table and wipe every committed row while reporting
        # success — the exact zero-parsed-items destruction path. The most
        # likely cause is a renamed/missing `## Items` heading (which Fix B
        # also flags via parse_errors above), but a genuinely-empty-yet-
        # committed file reaches here with no parse_errors, so guard it too.
        return _fail(f"refusing to regenerate: source parsed zero items but "
                     f"the committed Index has {len(parsed.index)} row(s) — "
                     f"the Items section is likely renamed or missing")
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
    backlog_path.write_text(new, encoding="utf-8")
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

    if prev_ga is None:
        os.environ.pop("GITHUB_ACTIONS", None)
    else:
        os.environ["GITHUB_ACTIONS"] = prev_ga

    print("backlog-lint self-test:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
