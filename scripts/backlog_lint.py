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
    old orphan-row / missing-row / duplicate-Index-id checks.)
  - `Depends on` referential integrity (ids exist) + acyclicity (no cycle)
  - public-tier spot check over Items (titles + full field values, including
    continuation lines) AND the Inbox section. Intentionally conservative:
    the patterns match only the literal phrases "private repo" /
    "gated dataset" and POSIX home paths (`/Users/...`) — clear leaks only.
    They do NOT catch private github.com URLs, real gated-dataset prefixes,
    or Windows-style paths; the scan is a tripwire, not a guarantee.

Warnings (never fail): a `Done when` with no concrete artifact reference
(a command / file / observable behaviour) — structural checks stay hard,
concreteness is only nudged, so a legitimate item is never red over phrasing.

Importable API (the backlog view + digest tools reuse it — one parser,
one dependency grammar, not three): parse_backlog, parse_deps,
required_fields, render_index.

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

    The single dependency grammar for all three backlog tools (lint, view,
    digest import this — one copy, not three): "—" / "-" / "" / "none" mean
    no dependencies; otherwise every `B<number>` token is a dependency.
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
        if cur is None:
            continue
        fm = _FIELD.match(line)
        if fm:
            key = fm.group("key").strip()
            cur.fields[key] = fm.group("val").strip()
            cur.raw_fields[key] = fm.group("val").strip()
            cur_key = key
            continue
        if not stripped or line.startswith("## "):
            _finish_field()
            continue
        if cur_key is not None:
            cur.fields[cur_key] = (cur.fields[cur_key] + " " + stripped).strip()
            cur.raw_fields[cur_key] = cur.raw_fields[cur_key] + "\n" + line
    return Backlog(index=index, index_table=index_table, items=items,
                   inbox=inbox, raw=text)


# --- derived Index -----------------------------------------------------------

def render_index(items: list[Item]) -> str:
    """Render the Index table from the Item blocks (the Index is derived).

    Single source: the Item blocks. `title` comes from the item heading,
    `owner` / `size` / `status` from the Owner / Rough size / Status fields.
    """
    out = ["| id | title | owner | size | status |", "|---|---|---|---|---|"]
    for it in items:
        size = it.fields.get("Rough size", "").split()
        out.append("| {} | {} | {} | {} | {} |".format(
            it.id,
            it.fields.get("__title__", ""),
            it.fields.get("Owner", ""),
            size[0] if size else "",
            it.fields.get("Status", ""),
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

    for it in backlog.items:
        where = f"{it.id} (line {it.line})"
        if it.id in ids_seen:
            f.err(f"{where}: duplicate item id (also at line {ids_seen[it.id]})")
        ids_seen[it.id] = it.line

        for key in req_fields:
            if key not in it.fields:
                f.err(f"{where}: missing required field '{key}'")

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

    # The Index is derived from the Item blocks: the committed table must
    # byte-match the render (stale, hand-edited, orphan, or missing rows all
    # surface as a diff).
    committed = "\n".join(backlog.index_table)
    rendered = render_index(backlog.items)
    if committed != rendered:
        f.err("Index is stale: it is generated from the Item blocks — edit the "
              "blocks, then run `python3 scripts/backlog_lint.py --write-index`")

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
    """Required-field schema from the template. None = unusable (fail closed)."""
    if not template_path.exists():
        return None
    req = required_fields(template_path.read_text())
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
        return _fail(f"template {template_path} is missing or defines no "
                     "`- **Field:**` lines — required-field schema unavailable")
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
    rendered = render_index(parse_backlog(text).items) + "\n"
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

    # CLI exit-code contract through main()/_run (the Phase-2 flip surface).
    import tempfile
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

    print("backlog-lint self-test:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
