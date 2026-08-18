# docs-budget fixtures

Fixtures for `scripts/docs_budget.py --self-test`.

## Strategy: one static repo, the rest generated

Budgets are fixed (CLAUDE.md 12,288 B; each `.claude/rules/*.md` 8,192 B;
`project_log.md` 15,360 B; always-loaded aggregate 49,152 B — see
`.claude/rules/04-docs.md`), so exercising the warn (>1.0x) and fail
(>1.5x) zones requires files of 10–24 KB. Committing that much filler
adds no review value, so:

- **Committed here:** `under_budget_repo/` — a tiny static fixture repo
  whose three surfaces all sit in the OK zone.
- **Generated at self-test runtime** (in a `tempfile.TemporaryDirectory`,
  exact byte sizes, deleted afterward):
  - warn-zone repo — `CLAUDE.md` 14,000 B, `.claude/CLAUDE.md` 14,000 B
    (proves the alternate location is scanned), `.claude/rules/01-r.md`
    10,000 B, `project_log.md` 20,000 B — 38,000 B of always-loaded
    surfaces, deliberately under the aggregate cap so the per-surface
    annotation counts stay exact
  - fail-zone repo — `CLAUDE.md` 18,433 B, `.claude/rules/01-r.md` 12,289 B,
    `project_log.md` 23,041 B (each exactly one byte past its 1.5x line)
  - aggregate repos — 7 x 8,000 B rules files (56,000 B, warn) and 10 x
    8,000 B (80,000 B, fail): every file individually inside its budget,
    the always-loaded tier as a whole over the cap
  - aggregate-exclusion repo — `CLAUDE.md` 1,000 B + `project_log.md`
    15,000 B, proving the log is not counted in the aggregate
  - log-only repo — `project_log.md` 1,000 B alone, so the aggregate has
    nothing to measure and must report `n/a` rather than a passing zone
  - both-CLAUDE.md repo — `CLAUDE.md` 1,000 B + `.claude/CLAUDE.md`
    2,000 B, pinning that the aggregate sums both locations (3,000 B)
  - completeness repo — the 10 x 8,000 B fail-zone aggregate with one
    rules file made unmeasurable, so the naive sum (72,000 B) would fall
    back under the fail line
  - empty repo — missing-surface silence
  - symlink repo — `.claude/rules` linked outside the root, when the
    platform allows symlink creation
  - real-trigger repos — the unmeasurable cases built on a real filesystem
    rather than by monkeypatching `collect_surfaces`: `.claude/rules` as a
    symlink loop, as a symlink whose target does not resolve, and (via
    `chmod 000`) as an unlistable directory and under an unreadable
    `.claude`; a rules *file* that is a symlink loop; and a `CLAUDE.md`
    that resolves outside the root. Each is guarded — the checks print
    SKIP where the platform refuses symlinks, or where `chmod` does not
    restrict access
  - rules-scope repos — a `.txt` beside a rules file (neither budgeted nor
    counted) and a *directory* named `01-r.md` (not budgeted on its inode
    size)

## What the self-test covers

- Zone boundary classification: size == budget → OK; budget+1 → WARN;
  size == 1.5x → WARN; 1.5x+1 → FAIL (all three budgets)
- Both modes on every zone: warn-only always exits 0; enforce exits 1 only
  on a fail-zone surface
- Annotation output: `::warning` for warn zone (and for fail zone in
  warn-only mode), `::error` for fail zone under `--enforce`
- Missing surfaces skipped silently
- Unreadable surfaces (permission denied / TOCTOU-vanished; simulated
  cross-platform by monkeypatching `collect_surfaces` to hand `scan()` a
  vanished path): excluded from findings, named in a `::warning` line.
  A surface outside the always-loaded tier — `project_log.md` — still
  exits 0 in both modes, because losing one per-file verdict is harmless
- Aggregate completeness: when the unmeasurable surface **is** always-
  loaded, the total would read low, so the run reports `PARTIAL` with the
  counted bytes as a floor instead of a zone, names every unmeasurable
  surface in a fileless annotation, and fails closed under `--enforce`
  (warn-only still exits 0). Covers the three drop paths — a `stat()`
  failure on a file, a path resolving outside the repo root, and an
  unmeasurable or unlistable `.claude/rules` **directory**, which is
  classified by the same `probe()` the files go through
- Exit codes through `main()`, not only `run()`: CI reads `main()`'s code,
  so the fail-closed assertions are pinned at the seam CI actually uses
- Fail-closed guards that are invisible in a passing run: `escapes_root()`
  returning `True` when resolution raises, and a skips-only scan reporting
  its unmeasurable count instead of "nothing to check"
- The aggregate cap constant pinned to the byte by an OK/WARN boundary
  pair, and the summary line's aggregate clause including its `n/a`
  branch
- Junction/symlink escape: `escapes_root()` is a pure resolved-path
  containment comparison, always tested directly with plain paths
  (junctions cannot be created portably from stdlib Python, and Windows
  symlinks need privileges); when a symlink **can** be created, the
  scan-level skip is additionally tested end-to-end — otherwise that one
  check prints SKIP.

The fixtures and assertions live in `scripts/docs_budget_selftest.py` so
`scripts/docs_budget.py` stays under the Class-1 1,000-line budget
(`reference/code-quality-taxonomy.md`) — the same split
`backlog_lint.py` / `backlog_lint_selftest.py` uses. The entry point is
unchanged.

Run: `python scripts/docs_budget.py --self-test`
