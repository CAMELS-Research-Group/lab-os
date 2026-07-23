# Documentation

All lab repos. Rationale: `PR-LIFECYCLE.md`. Project-log mechanics: `03-logging.md`.

## Single source

Every fact has one owning doc; others link to or visibly derive from it, naming it ("source of truth: `spec.md` §3"). Public-tier docs never depend on private-source access — restate generalized, robust to private-doc changes. Derived public-tier docs: re-verify against owning source before stakeholder-facing events (check-ins, releases, outreach). Review check: restated fact owned elsewhere? → link or sourced derivation.

## Tiers & budgets

| Tier | Reader | Surfaces | Standard |
|---|---|---|---|
| AI | Agents | Always-loaded: `CLAUDE.md`, `.claude/rules/`. First-read: log head, `GLOSSARY.md` (domain vocab, if present). Grep-only: archive, `TROUBLESHOOTING.md`, `reference/code-quality-taxonomy.md` (finding taxonomy; single-source for rules and skills) | Dense, deterministic, count-free; grep-only unbudgeted |
| ENG | Lab members | PRDs, specs, plans, TRD/ADD, runbooks | Skimmable; stable anchors; explicit contracts; code-free plans |
| Public | Stakeholders | Roadmaps, overviews, outreach | Jargon-free; no codenames; overclaim-scrubbed; single-sourced |

Budgets (bytes): per-repo `CLAUDE.md` 8 KB · `.claude/rules/*.md` 5 KB each · `project_log.md` 15 KB. Warn 1.0×, fail 1.5×; `docs-budget` warn-only per repo until first green.

## ENG document standards

- **PRD** — living doc, stable path, amended never archived (only its lifecycle `Status:` advances — §Bundle lifecycle). Required: Problem · Success criteria (measurable) · Scope (in/explicitly out) · Constraints · Plan (phased) · Open questions. Open questions owns known gaps. No embedded decision log — `project_log.md` owns decisions.
- **Plan** — code-free: per task, Files/Depends on/Spec link/Architectural constraints/Acceptance/Verification/Agent-suitable/Commit; only code blocks: shell in Verification lines. On code-touching plans, run `standards-aware-planning` (derives from the taxonomy; expresses constraints in `codebase-design` terms) to populate Architectural constraints; `none triggered` when empty; doc-only plans omit the element. Its execution counterpart: while implementing code-touching tasks, apply `execution-writing-disposition` for classes at altitude `execution`. No standalone per-bundle Design doc (cross-cutting shared specs under `_specs/_design/` remain a valid doc type) and no plan-carried Execution Log: decisions-with-rationale and rejected alternatives live in the bundle's `log.md`; known gaps live in the PRD's Open questions. The plan is one file in the planning bundle (source of truth: `03-logging.md` spec-log altitude).

## Bundle lifecycle

Each planning bundle carries one **lifecycle `Status:`** in its PRD header — the bundle's single state marker (source of truth for "is this the active plan"). Enumerated, non-terminal → terminal: `draft` · `active` · `paused` — then terminal `complete` · `superseded` · `abandoned`. Format: `**Status:** <state> — <optional free-text qualifier>` (e.g. `active — in review, PR #91`).

- `draft` shaping/pre-signoff · `active` in execution · `paused` on hold (mirrors the `03-logging.md` pause banner).
- `complete` slice shipped/merged · `superseded` replaced by another bundle · `abandoned` deliberately not built. **Terminal states freeze the body** — amend only for accuracy, never new scope.
- **Supersession is a forward pointer, not an edit:** the new bundle's PRD carries `Supersedes: <handle>` (`03-logging.md` convention); the old flips to `Status: superseded — by <handle>`. Never delete the old.
- **Retain in place, never move.** "Bundle archival" (merge bar) = flip `Status: complete` in place; bundles stay dated under `_specs/`. The PRD stays a living doc — only its Status advances (§ENG document standards, PRD).

## Rules numbering

lab-os owns `0x-*`; per-repo rules use `10+`. Base `0x` rules and the doc assets listed in the manifest of `scripts/rules_sync.py` **in the lab-os workspace fork** (github.com/WatsonWBlair/Agentic_Workspace) go into member repos as verbatim copies under a one-line sync header: edits land in that fork — the byte source — and re-sync from there; the convention itself is owned upstream. That script is also the drift check, and it runs in the fork, not in member repos.
