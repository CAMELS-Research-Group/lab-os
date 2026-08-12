# Documentation

All lab repos. Rationale: `PR-LIFECYCLE.md`. Project-log mechanics: `03-logging.md`.

## Single source

Every fact has one owning doc; others link to or visibly derive from it, naming it ("source of truth: `spec.md` §3"). Public-tier docs never depend on private-source access — restate generalized, robust to private-doc changes; re-verify derived ones against the owning source before stakeholder-facing events. Review check: restated fact owned elsewhere? → link or sourced derivation.

## Tiers & budgets

| Tier | Reader | Surfaces | Standard |
|---|---|---|---|
| AI | Agents | Always-loaded: `CLAUDE.md`, `.claude/rules/`. First-read: log head, `GLOSSARY.md` (domain vocab, if present). Grep-only: archive, `TROUBLESHOOTING.md`, `reference/code-quality-taxonomy.md` (single-sources rules and skills), `.claude/skills/**` (loaded on invocation) | Dense, deterministic, count-free; grep-only unbudgeted |
| ENG | Lab members | PRDs, specs, plans, TRD/ADD, runbooks | Skimmable; stable anchors; explicit contracts; code-free plans |
| Public | Stakeholders | Roadmaps, overviews, outreach | Jargon-free; no codenames; overclaim-scrubbed; single-sourced |

Budgets (bytes): per-repo `CLAUDE.md` 12 KB · `.claude/rules/*.md` 8 KB each · **always-loaded aggregate** (`CLAUDE.md` + every rules file, excluding `project_log.md`) 48 KB · `project_log.md` 15 KB at project altitude, 40 KB at lab (altitudes: `03-logging.md`). Warn 1.0×, fail 1.5×; `docs-budget` warn-only per repo until first green. The aggregate binds: per-file budgets cap any one surface, the aggregate caps what the always-loaded tier costs every session and every subagent. Over aggregate → demote a surface to grep-only, not raise the cap; a raise is a logged lab-level decision.

## ENG document standards

- **PRD** — living doc, stable path, amended never archived (only its `Status:` advances — §Bundle lifecycle). Required: Problem · Success criteria (measurable) · Scope (in/explicitly out) · Constraints · Plan (phased) · Open questions. Open questions owns known gaps. No decision bodies — link to the bundle's `spec.md`; decisions outliving the slice route to `project_log.md`.
- **Spec** — the bundle's `spec.md`: what was decided; current design authority. Decision summary table (question → resolution → status) · per-decision sections (contract impact) · finalized contracts; legend `DECIDED / RECOMMENDED / PARKED`. Single-source at bundle altitude: a decision lives here once — `log.md` records how it got there; `prd.md`/`plan.md` link, never restate. Chore/docs-only bundles (log-overflow archives, doc syncs) omit it; three-file bundles are grandfathered. Cross-cutting shared specs stay under `_specs/shared-design/`.
- **Plan** — code-free: per task, Files/Depends on/Requires/Spec link/Architectural constraints/Acceptance/Verification/Agent-suitable/Commit; only code blocks: shell in Verification lines. On code-touching plans, run `standards-aware-planning` (if present) to populate Architectural constraints; `none triggered` when empty; doc-only plans omit it. Its execution counterpart: while implementing those tasks, apply `execution-writing-disposition` (if present) for classes at altitude `execution`. No plan-carried Execution Log: deviations and gate evidence live in the bundle's `log.md`; resolved decisions live once in `spec.md` (rationale/alternatives history in `log.md`); known gaps live in the PRD's Open questions. The plan is one file in the planning bundle (`03-logging.md` spec-log altitude).

## Bundle lifecycle

Each planning bundle carries one **lifecycle `Status:`** in its PRD header — its single state marker. Enumerated, non-terminal → terminal: `draft` · `active` · `paused` — then terminal `complete` · `superseded` · `abandoned`. Format: `**Status:** <state> — <optional free-text qualifier>` (e.g. `active — in review, PR #91`).

- `draft` shaping/pre-signoff · `active` in execution · `paused` on hold (mirrors the `03-logging.md` pause banner).
- `complete` slice shipped/merged · `superseded` replaced by another bundle · `abandoned` deliberately not built. **Terminal states freeze the body** — amend only for accuracy, never new scope.
- **Supersession is a forward pointer, not an edit:** the new bundle's PRD carries `Supersedes: <handle>` (`03-logging.md` convention); the old flips to `Status: superseded — by <handle>`. Never delete the old.
- **Retain in place, never move.** "Bundle archival" (merge bar) = flip `Status: complete`; bundles stay dated under `_specs/`.

Review check: PRD `Status:` off-enum or stale? → correct to an enumerated value.

## Rules numbering

lab-os owns `0x-*`; per-repo rules use `10+`. Base `0x` rules and the doc assets in the `scripts/rules_sync.py` manifest go into member repos as verbatim copies under a one-line sync header. A member repo carrying per-repo rules on `0x` slots is renumbered to `10+` before the first vendored sync. **Upstream lab-os (github.com/CAMELS-Research-Group/lab-os) owns both the convention and the canonical bytes.** The lab-os workspace fork (github.com/WatsonWBlair/Agentic_Workspace) is where edits are staged before syncing upstream; that script — also the drift check — runs there, not in member repos.
