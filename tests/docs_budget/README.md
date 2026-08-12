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
  - empty repo — missing-surface silence
  - symlink repo — `.claude/rules` linked outside the root, when the
    platform allows symlink creation

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
  vanished path): excluded from findings, named in a `::warning` line,
  exit 0 in both modes — unreadability never fails the run
- Junction/symlink escape: `escapes_root()` is a pure resolved-path
  containment comparison, always tested directly with plain paths
  (junctions cannot be created portably from stdlib Python, and Windows
  symlinks need privileges); when a symlink **can** be created, the
  scan-level skip is additionally tested end-to-end — otherwise that one
  check prints SKIP.

Run: `python scripts/docs_budget.py --self-test`
