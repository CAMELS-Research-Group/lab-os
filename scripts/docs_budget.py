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
    .claude/CLAUDE.md     budget 12,288 B   (counted in ADDITION to a root
                                            CLAUDE.md, not instead of it)
    .claude/rules/*.md    budget  8,192 B each

Beyond the per-file budgets, the always-loaded surfaces — every
CLAUDE.md the scan finds (both locations sum when both exist) plus every
.claude/rules/*.md that is not path-scoped — are checked in aggregate
against 49,152 B. A rules file is path-scoped when its frontmatter holds
only a `paths:` block list of globs (see scoped_paths()); it loads only
when a session touches a matching path, so it keeps its per-file budget,
is listed as path-scoped, and stays out of the aggregate. Any other
frontmatter counts as always-loaded, which fails closed. Until
04-docs.md states this exclusion, its owner is the path-scoped tier
decision in Caravan's workflow-economy epic (CAMELS-Research-Group/Caravan#711);
the rule text lands with CAMELS-Research-Group/Caravan#714. The
rules scan is FLAT, matching the rule's own `*.md` glob:
`.claude/rules/` is not searched recursively, and a subdirectory under it
is recorded as an unmeasurable always-loaded surface rather than walked
or ignored. The aggregate is the invariant that actually protects session
context: per-file budgets alone permit unbounded growth, because nothing
caps how many rules files the always-loaded tier may hold. Same zone
semantics as a per-file budget.

A zone verdict for the aggregate is only reported when it can be
computed. Any surface that is present but unmeasurable (stat failure, a
non-regular file, a directory standing where a document belongs, or a
path resolving outside the repo root) is excluded from the sum. When that
surface is ALWAYS-LOADED the total would read low, so the run reports
PARTIAL instead of a zone and, under --enforce, fails closed. A per-file
skip loses one verdict, but a short aggregate is a wrong number presented
as authoritative.

The same fail-closed reading covers the degenerate case: a scan that
enumerates NO always-loaded surface at all has measured nothing, and
under --enforce that is an error rather than a pass. A green gate that
checked zero surfaces is the one output a budget check must never
produce.

Zone semantics (size measured in bytes on disk):

    size <= budget                 -> OK    (no output beyond the report line)
    budget < size <= 1.5 * budget  -> WARN  (GitHub ::warning annotation, exit 0)
    size > 1.5 * budget            -> FAIL  (exit 1 when --enforce; otherwise
                                             downgraded to a warning, exit 0)

Default mode is warn-only until a repo first passes green, per
`.claude/rules/04-docs.md` (section Tiers & budgets, the same file the
budgets come from): the script always exits 0 no matter what it finds in
that mode. Pass --enforce to exit 1 when any surface is in the FAIL
zone, when the always-loaded aggregate is PARTIAL, or when no
always-loaded surface was found to measure at all.

EOL drift: CRLF working copies (e.g. Windows checkouts with autocrlf)
measure larger than the LF checkouts CI sees; budgets are calibrated for
normalized (LF) checkouts and the 1.5x WARN band absorbs the drift.

Junction/symlink awareness: any scanned path (the .claude dir, the rules
dir, or an individual file) whose resolved location lies outside the
resolved repo root is not measured here — it is recorded as an "escaped"
skip. An escaping always-loaded surface therefore makes the aggregate
PARTIAL and, under --enforce, fails the run: the bytes still load into
this repo's sessions, so a total computed without them is short. A
.claude/rules symlinked or junctioned to a shared location (the
mission-control pattern) is a plausible vendoring shape that an
`enforce: true` repo will now go red on; that is deliberate, not an
oversight — a shared surface has to be counted where it loads, not
delegated to whichever repo happens to own the bytes. On Windows,
junctions may not register as symlinks via Path.is_symlink(), so the
check compares Path.resolve() results instead of testing link-ness.

Stdlib only; compatible with Python 3.11+.
"""

from __future__ import annotations

import argparse
import os
import re
import stat
import sys
from pathlib import Path
from typing import NamedTuple

BUDGET_CLAUDE_MD = 12_288
BUDGET_RULES_MD = 8_192

# Aggregate cap on everything that loads into EVERY session's context.
# The per-file numbers say how big any one surface may get; this says how
# much total context the always-loaded tier may claim. Whether it binds
# below the sum of the per-file budgets depends on how many rules files a
# repo carries, so the cap is stated as an absolute rather than as a
# fraction of that sum.
BUDGET_ALWAYS_LOADED_TOTAL = 49_152

# Surfaces that load unconditionally into every session (04-docs.md, AI tier).
ALWAYS_LOADED_FILES = ("CLAUDE.md", ".claude/CLAUDE.md")
ALWAYS_LOADED_RULES_PREFIX = ".claude/rules/"

# Skip kinds recorded for a present-but-unmeasurable surface.
SKIP_ESCAPED = "escaped"
SKIP_ERROR = "error"


class Skip(NamedTuple):
    """A surface that is present but could not be measured.

    A NamedTuple rather than a bare tuple because `always_loaded` is what
    the aggregate's fail-closed decision turns on, and it sits at index 3
    of a record whose annotation used to declare three fields. A reader
    trusting that annotation reaches for `sk[2]` — `detail`, non-empty at
    every construction site and so unconditionally truthy, which would
    fail every skip closed. Nothing type-checks this repo in CI, so the
    fix has to be one that indexing itself enforces: the arity is now
    declared, and `sk.always_loaded` names the field where correctness
    depends on it.
    """

    rel: str
    kind: str
    detail: str
    always_loaded: bool


class Finding(NamedTuple):
    """A measured surface. `always_loaded` and `path_scoped` are decided in
    scan(), where the file's bytes are read, and carried here so run() never
    re-derives either from the rel string alone (the same doctrine as
    `Skip.always_loaded`). `path_scoped` is True only for a rules file the
    slot would load whose own frontmatter takes it out of the total."""

    rel: str
    size: int
    budget: int
    zone: str
    always_loaded: bool
    path_scoped: bool


ZONE_OK = "OK"
ZONE_WARN = "WARN"
ZONE_FAIL = "FAIL"

# Deliberately NOT in the ZONE_* namespace: this is not a size zone but the
# statement that the aggregate could not be computed at all. Naming it as a
# zone invited reading it as a fourth verdict alongside OK/WARN/FAIL, which
# it never is — classify() can never return it and no per-file surface can
# carry it. The printed token stays "PARTIAL"; only the symbol moved.
AGGREGATE_INCOMPLETE = "PARTIAL"


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


# One `paths:` list item: a double- or single-quoted glob with no escapes,
# or a plain glob that cannot start a YAML indicator (`*` would be an
# alias, `[`/`{` a flow collection, `-`/`?`/`:` a nested node) and holds no
# whitespace, `#` or `:`. Anything else is not a form the scan can vouch
# for, so it never counts as path-scoped.
_PATHS_ITEM = re.compile(
    r"( +)- +(?:\"([^\"\\]+)\"|'([^']+)'|([^\s'\"\[\]{}&*!|>%@`#,?:-][^\s#:]*)) *"
)
# Common plain scalars that YAML 1.1 or 1.2 resolves to something other
# than a string (a conservative union, not an exhaustive list); a glob list
# holding one is not a list of glob strings.
_NON_STRING_PLAIN = re.compile(
    r"[-+]?(\.[0-9]+|[0-9][0-9_]*(\.[0-9_]*)?)([eE][-+]?[0-9]+)?"
    r"|0x[0-9a-fA-F_]+|0o[0-7_]+|~|null|true|false|yes|no|on|off|\.inf|\.nan",
    re.IGNORECASE,
)

# Frontmatter that opens with a fence and names `paths:`, whatever its form:
# when scoped_paths() rejects it, the author meant to scope the rule, so the
# fail-closed count is reported rather than left silent.
_LOOKS_SCOPED = re.compile(
    rb"(?:\xef\xbb\xbf)?\s*---\r?\n(?:(?!---\s*\n)[^\n]*\n)*?\s*paths\s*:"
)


def scoped_paths(raw: bytes) -> tuple[str, ...] | None:
    """The globs a rules file is scoped to, or None when it is not scoped.

    Accepts one form: YAML between `---` fences at byte 0 whose only key is
    `paths:`, holding a non-empty block list of glob strings, each item
    indented by the same one or more spaces and written double-quoted,
    single-quoted or plain. A block list was observed loading as path-scoped
    on Claude Code 2.1.308 (probe record: CAMELS-Research-Group/Caravan#858);
    the three quoting styles are treated as equivalent YAML strings. Blank
    lines inside the fences are allowed; nothing else is (a zero-indent list
    included). A flow list, a comma-separated string, another key, a BOM, an
    unclosed fence or undecodable bytes all return None, so the file counts
    as always-loaded. That direction is deliberate: Claude Code loads a
    rule whose frontmatter it cannot parse as if it had no `paths`, so a
    form this parser over-accepted would leave always-loaded bytes out of
    the aggregate, while one it under-accepts only overstates the total.
    """
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError:
        return None
    lines = text.replace("\r\n", "\n").split("\n")
    if lines[0] != "---":
        return None
    try:
        end = lines.index("---", 1)
    except ValueError:
        return None
    body = [line for line in lines[1:end] if line.strip()]
    if not body or body[0].rstrip(" ") != "paths:":
        return None
    globs: list[str] = []
    indent = None
    for line in body[1:]:
        match = _PATHS_ITEM.fullmatch(line)
        if match is None:
            return None
        if indent is None:
            indent = match.group(1)
        elif match.group(1) != indent:
            return None
        quoted = match.group(2) or match.group(3)
        plain = match.group(4)
        if plain is not None and _NON_STRING_PLAIN.fullmatch(plain):
            return None
        globs.append(quoted if quoted is not None else plain)
    return tuple(globs) or None


def is_always_loaded(rel: str, raw: bytes | None = None) -> bool:
    """True when a scanned surface loads into every session's context.

    CLAUDE.md — BOTH locations, cumulatively, when a repo carries both —
    and every .claude/rules/*.md file load unconditionally, because a
    session loads the directory (04-docs.md, section Tiers & budgets, owns
    the scope) — except a rules file whose bytes `raw` are path-scoped
    (scoped_paths()). Without `raw` the answer is the slot's: a rules file
    counts as always-loaded, which is the fail-closed reading for a surface
    whose bytes could not be read.

    Membership is FLAT and case-insensitive, matching the enumerator in
    collect_surfaces() byte for byte. Both halves of that sentence were
    once false and each was a silent short sum: a predicate that accepted
    a nested `.claude/rules/sub/02-b.md` while the enumerator listed only
    the directory's own `*.md` children let 90,000 B of always-loaded
    bytes leave the total with no skip and no annotation, and a
    case-sensitive suffix test dropped `01-R.MD` on the case-insensitive
    filesystems where `main`'s glob() had matched it. The predicate and
    the enumerator have to agree, or the aggregate reports a number for a
    set it did not measure. That agreement is the slot answer, without
    `raw`. Path scoping is the one deliberate exception, and it comes after
    enumeration: a path-scoped rule is still listed, measured and labelled,
    and only its bytes take it out of the total.
    """
    if rel in ALWAYS_LOADED_FILES:
        return True
    if not rel.startswith(ALWAYS_LOADED_RULES_PREFIX):
        return False
    tail = rel[len(ALWAYS_LOADED_RULES_PREFIX):]
    if "/" in tail or not tail.lower().endswith(".md"):
        return False
    return raw is None or scoped_paths(raw) is None


def escapes_root(path: Path, root: Path) -> bool:
    """True when `path` resolves to a location outside resolved `root`.

    Pure resolved-path containment comparison — deliberately NOT
    Path.is_symlink(), which reports False for Windows junctions even
    though Path.resolve() follows them out of the repo.

    A path that cannot be resolved at all (resolve() raises OSError) is
    reported as escaped: callers treat that as an unmeasurable surface,
    which is the fail-closed reading.
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
    list[tuple[Path, int]], list[Skip]
]:
    """((path, budget) pairs, skips) for the scanned surfaces under root.

    Missing surfaces are skipped silently — not every repo has every file.
    A surface that *is* present but cannot be measured is recorded as a
    `Skip` (rel, kind, detail, always_loaded), kind being 'escaped'
    (resolves outside the repo root — junction/symlink) or 'error' (stat
    failed, or the path is present but not the shape its slot expects).
    Skips are what let the aggregate declare itself incomplete instead of
    quietly reporting a short total as authoritative.

    `always_loaded` is decided here, where the slot is known, and carried
    on the record. It used to be re-derived downstream from the rel string
    alone, which forced a directory skip's rel to end in "/" purely so a
    prefix test would match — a display detail load-bearing for a
    correctness decision, and a booby trap for anyone who tidied it away.
    """
    surfaces: list[tuple[Path, int]] = []
    skips: list[Skip] = []

    def rel_of(path: Path) -> str:
        try:
            return path.relative_to(root).as_posix()
        except ValueError:
            return str(path)

    def consider(path: Path, budget: int, enumerated: bool = False) -> None:
        """Record `path` as a measurable surface, or as a skip.

        `enumerated` marks a path that a directory listing just reported as
        present. For those, "missing" is not the ordinary absence of an
        optional surface — it means the file vanished between the listing
        and the stat, which is an unmeasurable present surface.
        """
        rel = rel_of(path)
        always_loaded = is_always_loaded(rel)
        kind, detail = probe(path)
        if kind == "error":
            skips.append(Skip(rel, SKIP_ERROR, detail, always_loaded))
            return
        if kind == "dir":
            # A directory standing where a budgeted document belongs is
            # present and unmeasurable, not absent. Returning silently here
            # granted an exemption by surface shape that the aggregate's
            # doctrine explicitly denies: the bytes of that always-loaded
            # slot are unaccounted for either way.
            skips.append(Skip(
                rel, SKIP_ERROR, "directory standing in a file's place",
                always_loaded,
            ))
            return
        if kind == "missing":
            if enumerated:
                skips.append(Skip(
                    rel, SKIP_ERROR,
                    "vanished between the directory listing and stat",
                    always_loaded,
                ))
            return
        if escapes_root(path, root):
            skips.append(Skip(
                rel, SKIP_ESCAPED, "resolves outside the repo root",
                always_loaded,
            ))
            return
        surfaces.append((path, budget))

    consider(root / "CLAUDE.md", BUDGET_CLAUDE_MD)

    # The .claude directory is the parent of two always-loaded slots and was
    # the one link in the chain nothing classified. probe() on a path *under*
    # a non-directory is platform-dependent: POSIX reports ENOTDIR, which
    # reaches probe()'s OSError branch and fails closed, but Windows reports
    # ENOENT, and probe()'s FileNotFoundError branch only separates absence
    # from a dangling symlink — so a `.claude` that is a regular file made
    # BOTH children read as ordinary absence and the entire always-loaded
    # rules tier left the scan with no surface and no skip. Classifying the
    # parent once, here, makes that answer the same on every platform.
    claude_dir = root / ".claude"
    claude_rel = f"{rel_of(claude_dir)}/"
    claude_kind, claude_detail = probe(claude_dir)
    if claude_kind == "error":
        skips.append(Skip(claude_rel, SKIP_ERROR, claude_detail, True))
    elif claude_kind == "file":
        skips.append(Skip(
            claude_rel, SKIP_ERROR,
            "regular file standing in the .claude directory's place", True,
        ))
    if claude_kind != "dir":
        # Absent is the ordinary case for a repo with no .claude directory;
        # the two shapes above already recorded their skip. Either way there
        # is nothing beneath it to enumerate.
        return surfaces, skips

    consider(claude_dir / "CLAUDE.md", BUDGET_CLAUDE_MD)

    rules_dir = claude_dir / "rules"
    # The directory goes through the same classifier as the files. is_dir()
    # returns False on a self-referential symlink (ELOOP) and raises on an
    # unreadable parent (EACCES), so an inline is_dir() test dropped the
    # entire always-loaded rules tier with no skip recorded — the aggregate
    # then reported a short sum as authoritative. Trailing slash on the rel
    # is display only: it names a directory rather than a file, which is
    # also what keeps scan() from anchoring a GitHub annotation to it.
    # Nothing decides always-loaded-ness from that slash any more.
    rules_rel = f"{rel_of(rules_dir)}/"
    kind, detail = probe(rules_dir)
    if kind == "error":
        skips.append(Skip(rules_rel, SKIP_ERROR, detail, True))
    elif kind == "file":
        # A regular file where the rules DIRECTORY belongs. probe() reports
        # it truthfully and no branch below claims it, so the whole
        # always-loaded rules tier used to leave the scan with no surface
        # and no skip — the same silent-drop the directory rewrite exists
        # to close, reached from the opposite shape.
        skips.append(Skip(
            rules_rel, SKIP_ERROR,
            "regular file standing in the rules directory's place", True,
        ))
    elif kind == "dir":
        if escapes_root(rules_dir, root):
            skips.append(Skip(
                f"{rel_of(rules_dir)}/*.md", SKIP_ESCAPED,
                "rules directory resolves outside the repo root", True,
            ))
        else:
            try:
                # Case-insensitive to match is_always_loaded(), and to keep
                # main's glob("*.md") behaviour on case-insensitive
                # filesystems, where a case-sensitive endswith() silently
                # dropped `01-R.MD`.
                listed = sorted(os.scandir(rules_dir), key=lambda e: e.name)
                entries = [
                    Path(e.path) for e in listed
                    if e.name.lower().endswith(".md")
                ]
                # A subdirectory under .claude/rules/ is the shape the
                # enumerator and the membership predicate disagreed about:
                # the scan is flat (04-docs.md states the tier as a `*.md`
                # glob), but is_always_loaded() used to accept a nested
                # `sub/02-b.md`, so a nested tree contributed neither a
                # surface nor a skip and 90,000 B of always-loaded bytes
                # left the total with the aggregate still reading as
                # authoritative. The predicate is flat now; this records the
                # directory so the shape is never merely ignored. A `*.md`
                # DIRECTORY is not double-counted here — it is enumerated
                # above and consider() skips it by shape.
                nested = [
                    Path(e.path) for e in listed
                    if not e.name.lower().endswith(".md")
                    and e.is_dir(follow_symlinks=False)
                ]
            except OSError as exc:
                # glob() discards the scandir error, so an unlistable
                # directory contributed zero surfaces AND zero skips — a
                # present directory became indistinguishable from an empty
                # one. os.scandir surfaces the errno so it becomes a skip.
                skips.append(Skip(
                    rules_rel, SKIP_ERROR,
                    f"{exc.__class__.__name__}: {exc}", True,
                ))
            else:
                for path in nested:
                    skips.append(Skip(
                        f"{rel_of(path)}/", SKIP_ERROR,
                        "subdirectory under .claude/rules/; the rules scan "
                        "is flat, so any .md beneath it is unmeasured",
                        True,
                    ))
                for path in entries:
                    consider(path, BUDGET_RULES_MD, enumerated=True)

    return surfaces, skips


def scan(root: Path) -> tuple[list[Finding], list[str], list[Skip]]:
    """Scan surfaces under root.

    Returns (findings, warnings, skips): findings are `Finding` records
    per readable surface, each carrying whether it is always-loaded and
    whether it is path-scoped — scan() reads each rules file's bytes once,
    so path scoping is decided here; warnings
    are ready-to-print annotation lines; skips are `Skip`
    (rel, kind, detail, always_loaded) records for
    present-but-unmeasurable surfaces, carried so the aggregate can declare itself incomplete
    rather than report a short total as complete.
    """
    findings: list[Finding] = []
    warnings: list[str] = []
    surfaces, skips = collect_surfaces(root)
    for rel, kind, detail, _always_loaded in skips:
        suffix = f" ({detail})" if detail else ""
        # A skip whose rel names a directory (".claude/rules/") or a glob
        # (".claude/rules/*.md") is not a path GitHub can anchor an
        # annotation to, so those are emitted fileless — the same shape the
        # aggregate annotations already use.
        anchor = "" if rel.endswith("/") or "*" in rel else f" file={rel}"
        warnings.append(
            f"::warning{anchor}::{rel} is a scanned surface but could not "
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
            skips.append(Skip(rel, SKIP_ERROR, detail, is_always_loaded(rel)))
            continue
        raw = None
        if rel.startswith(ALWAYS_LOADED_RULES_PREFIX):
            try:
                raw = path.read_bytes()
            except OSError as exc:
                # Measured but unreadable: its scope is unknown, so it keeps
                # the slot's always-loaded answer and stays in the total.
                raw = None
                warnings.append(
                    f"::warning file={rel}::{rel} was measured but its bytes "
                    f"could not be read ({exc.__class__.__name__}: {exc}); "
                    f"its scope is unknown, so it counts as always-loaded."
                )
        always_loaded = is_always_loaded(rel, raw)
        path_scoped = is_always_loaded(rel) and not always_loaded
        if raw is not None and always_loaded and _LOOKS_SCOPED.match(raw):
            warnings.append(
                f"::warning file={rel}::{rel} has `paths:` frontmatter in a "
                f"form the scan does not recognise (see scoped_paths()); it "
                f"counts as always-loaded."
            )
        findings.append(Finding(
            rel, size, budget, classify(size, budget), always_loaded,
            path_scoped,
        ))
    return findings, warnings, skips


def run(root: Path, enforce: bool) -> tuple[int, list[str]]:
    """Scan `root` and build the report. Returns (exit_code, output_lines)."""
    lines: list[str] = []
    findings, unreadable, skips = scan(root)
    lines.extend(unreadable)
    failed = False

    for f in findings:
        rel, size, budget, zone = f.rel, f.size, f.budget, f.zone
        ratio = size / budget
        scope = (
            " — path-scoped, outside the always-loaded total"
            if f.path_scoped else ""
        )
        lines.append(
            f"[{zone:<4}] {rel} — {size:,} B / {budget:,} B budget "
            f"({ratio:.2f}x){scope}"
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
    always_loaded = [f for f in findings if f.always_loaded]
    # Doctrine, stated once here because this is where it bites: EVERY
    # unmeasurable always-loaded surface fails closed, with no exemption by
    # skip kind and none by surface shape. An "escaped" skip (resolves
    # outside the repo root) and an "error" skip (stat/scandir failed) are
    # the same class to this computation, and an unmeasurable rules
    # DIRECTORY counts exactly like an unmeasurable rules file — losing the
    # whole tier is the worse of the two, not the excusable one. Any of
    # them makes the aggregate PARTIAL and, under --enforce, red.
    #
    # One field, decided at construction where the slot is known. Deriving
    # it here from the rel string needed a second, prefix-based clause to
    # catch the rules-directory skip (whose rel is the bare directory and
    # so does not end in ".md") — which in turn made that rel's trailing
    # slash load-bearing for correctness.
    missed = [sk for sk in skips if sk.always_loaded]
    aggregate_zone = None
    if missed:
        measured = sum(f.size for f in always_loaded)
        cap = BUDGET_ALWAYS_LOADED_TOTAL
        aggregate_zone = AGGREGATE_INCOMPLETE
        named = ", ".join(sk.rel for sk in missed)
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
        total = sum(f.size for f in always_loaded)
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
    else:
        # Neither an unmeasurable surface nor a measured one: the scan
        # enumerated no always-loaded surface at all. A gate that measured
        # nothing must not report green — the states that reach here are a
        # repo with no CLAUDE.md and no .claude/rules/*.md, and a --root
        # pointed somewhere that is not the repo. Both are the gate failing
        # to find its subject, which is indistinguishable in the exit code
        # from a clean pass unless it is failed closed.
        detail = (
            "no always-loaded surface was found to measure — expected "
            "CLAUDE.md (or .claude/CLAUDE.md) and/or .claude/rules/*.md "
            "under the scanned root; the aggregate check ran against "
            "nothing"
        )
        if enforce:
            failed = True
            lines.append(f"::error::{detail}.")
        else:
            lines.append(
                f"::warning::{detail} — warn-only mode; this will fail once "
                f"enforcement is on."
            )

    # One summary, always. The "no budgeted surfaces found — nothing to
    # check" special case was removed rather than re-gated: every state that
    # could still reach it is a state the run has just annotated. An empty
    # root in warn-only mode emits "::warning::no always-loaded surface was
    # found to measure" and then closed with "nothing to check" — the last
    # line a caller reads stating the inverse of the line above it, and
    # warn-only is the documented default for every new repo. Its guard
    # tested `findings`, `skips` and `failed`, none of which is set on that
    # path. The counted summary is true of the empty case too: it reads
    # "0 surface(s) checked ... always-loaded total n/a".
    n_warn = sum(1 for f in findings if f.zone == ZONE_WARN)
    n_fail = sum(1 for f in findings if f.zone == ZONE_FAIL)
    mode = "enforce" if enforce else "warn-only"
    # Only stated when there is something to state, so a clean run's
    # output is unchanged.
    unmeasurable = f", {len(skips)} unmeasurable" if skips else ""
    n_scoped = sum(1 for f in findings if f.path_scoped)
    scoped = f", {n_scoped} path-scoped" if n_scoped else ""
    lines.append(
        f"docs-budget: {len(findings)} surface(s) checked, "
        f"{n_warn} warn-zone, {n_fail} fail-zone{unmeasurable}{scoped}; "
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
            "and the always-loaded surfaces in aggregate (CLAUDE.md + "
            "rules not path-scoped by `paths:` frontmatter; that exclusion is "
            "owned by CAMELS-Research-Group/Caravan#711 until 04-docs.md "
            "states it) 49,152 B. Zones: size <= budget -> OK; budget < size <= "
            "1.5x budget -> WARN (annotation, exit 0); size > 1.5x budget "
            "-> FAIL (exit 1 only with --enforce). An absent surface is "
            "skipped silently; an always-loaded surface that is present but "
            "unmeasurable — a stat failure, a non-regular file, a directory "
            "standing in a file's place, or a path resolving outside the "
            "repo root (junctions/symlinks) — makes the aggregate PARTIAL "
            "and fails under --enforce, as does finding no always-loaded "
            "surface at all."
        ),
    )
    parser.add_argument(
        "--root", type=Path, default=Path("."),
        help="repo root to scan (default: current directory)",
    )
    parser.add_argument(
        "--enforce", action="store_true",
        help="exit 1 when any surface is above 1.5x its budget, when the "
             "always-loaded aggregate cannot be computed (PARTIAL), or when "
             "no always-loaded surface was found to measure "
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
        try:
            from docs_budget_selftest import run_self_test
        except ImportError as exc:
            # The cause is reported rather than asserted: the self-test
            # module imports production symbols from this one, so a rename
            # raises ImportError from a file that is present, and a bare
            # "missing file" message sends the reader to the wrong place.
            print(
                "docs-budget: --self-test could not import "
                f"scripts/docs_budget_selftest.py ({exc}); it must sit "
                "alongside this script and import symbols this module still "
                "defines",
                file=sys.stderr,
            )
            return 2
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
