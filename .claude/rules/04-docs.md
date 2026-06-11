# Documentation

All lab repos. Rationale: `PR-LIFECYCLE.md`. Project-log mechanics: `03-logging.md`.

## Single source

Every fact has one owning doc; others link to or visibly derive from it, naming it ("source of truth: `spec.md` §3"). Public-tier docs never depend on private-source access — restate generalized, robust to private-doc changes. Derived public-tier docs: re-verify against owning source before stakeholder-facing events (check-ins, releases, outreach). Review check: restated fact owned elsewhere? → link or sourced derivation.

## Tiers & budgets

| Tier | Reader | Surfaces | Standard |
|---|---|---|---|
| AI | Agents | Always-loaded: `CLAUDE.md`, `.claude/rules/`. First-read: log head. Grep-only: archive, `TROUBLESHOOTING.md` | Dense, deterministic, count-free; grep-only unbudgeted |
| ENG | Lab members | PRDs, specs, plans, TRD/ADD, runbooks | Skimmable; stable anchors; explicit contracts; code-free plans |
| Public | Stakeholders | Roadmaps, overviews, outreach | Jargon-free; no codenames; overclaim-scrubbed; single-sourced |

Budgets (bytes): per-repo `CLAUDE.md` 8 KB · `.claude/rules/*.md` 5 KB each · `project_log.md` 15 KB. Warn 1.0×, fail 1.5×; `docs-budget` warn-only per repo until first green.

## ENG document standards

- **PRD** — living doc, stable path, amended never archived. Required: Problem · Success criteria (measurable) · Scope (in/explicitly out) · Constraints · Plan (phased) · Open questions. No embedded decision log — `project_log.md` owns decisions.
- **Design doc** — one slice: problem, decisions with rationale + rejected alternatives, known gaps stated honestly. Top status line (draft/reviewed/superseded-by).
- **Plan** — code-free: per task, Files/Depends on/Spec link/Acceptance/Verification/Commit; only code blocks: shell in Verification lines. Carries `## Execution Log`.

## Rules numbering

lab-rules owns `0x-*`; per-repo rules use `10+`.
