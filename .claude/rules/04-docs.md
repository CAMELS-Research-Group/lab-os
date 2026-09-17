# Documentation

All lab repos. Rationale: `PR-LIFECYCLE.md`.

## Single source

Every fact has one owning doc; others link to or visibly derive from it, naming it ("source of truth: `spec.md` §3"). Public-tier docs never depend on private-source access — restate generalized, robust to private-doc changes; re-verify derived ones against the owning source before stakeholder-facing events. Review check: restated fact owned elsewhere? → link or sourced derivation.

## Tiers & budgets

| Tier | Reader | Surfaces | Standard |
|---|---|---|---|
| AI | Agents | Always-loaded: `CLAUDE.md`, `.claude/rules/`. First-read: `GLOSSARY.md` (domain vocab, if present). Grep-only: `TROUBLESHOOTING.md`, `reference/code-quality-taxonomy.md` (single-sources rules and skills), `.claude/skills/**` (loaded on invocation) | Dense, deterministic, count-free; grep-only unbudgeted |
| ENG | Lab members | PRDs, specs, plans, TRD/ADD, runbooks | Skimmable; stable anchors; explicit contracts; code-free plans |
| Public | Stakeholders | Roadmaps, overviews, outreach | Jargon-free; no codenames; overclaim-scrubbed; single-sourced |

Budgets (bytes): per-repo `CLAUDE.md` 12 KB · `.claude/rules/*.md` 8 KB each · **always-loaded aggregate** (`CLAUDE.md` + every rules file) 48 KB. Warn 1.0×, fail 1.5×; `docs-budget` warn-only per repo until first green. The aggregate binds: per-file budgets cap any one surface, the aggregate caps what the always-loaded tier costs every session and every subagent. Over aggregate → demote a surface to grep-only, not raise the cap; a raise is a decision recorded in the scope's main-bundle `spec.md`.

## ENG document standards

- **PRD** — living doc while the bundle is open (its `Status:` advances — §Bundle lifecycle & the main bundle). Required: Problem · Success criteria (measurable) · Scope (in/explicitly out) · Constraints · Plan (phased) · Open questions. Open questions owns known gaps. No decision bodies — link to the bundle's `spec.md`.
- **Spec** — the bundle's `spec.md`: what was decided; current design authority. Decision summary table (question → resolution → status) · per-decision sections (contract impact) · finalized contracts; legend `DECIDED / RECOMMENDED / PARKED`. Single-source at bundle altitude: a decision lives here once, with its rationale and the alternatives rejected; `prd.md`/`plan.md` link, never restate. Chore/docs-only bundles omit it; three-file bundles are grandfathered. Cross-cutting shared specs stay under `_specs/shared-design/`.
- **Design** — the bundle's `design.md`: the technical shape a code-touching slice commits to — architecture, contracts/schemas, data flow, failure modes. **Required when the slice meaningfully touches code** (new module, contract/schema change, cross-component behavior); docs-only/chore bundles omit it. `spec.md` owns what was decided, `design.md` the shape it produces; interface sketches live here, never in the plan.
- **Plan** — code-free: per task, Files/Depends on/Requires/Spec link/Architectural constraints/Acceptance/Verification/Agent-suitable/Commit; only code blocks: shell in Verification lines. On code-touching plans, run `standards-aware-planning` (if present) to populate Architectural constraints; `none triggered` when empty; doc-only plans omit it. While implementing, apply `execution-writing-disposition` (if present) for classes at altitude `execution`. No plan-carried Execution Log: deviations and gate evidence live in the PR body's Verification section; resolved decisions live once in `spec.md`; known gaps live in the PRD's Open questions. The plan is one file in the planning bundle.

## Bundle lifecycle & the main bundle

Each scope keeps one **main bundle** — `_specs/main/` (workspace roots: `_specs/<scope>/main/`), sibling to its dated bundles, never dated, never deleted, **exempt from byte budgets**: the same file set as a slice, each stated current-state — the **single source of truth for what is implemented**. `prd.md` standing purpose/scope · `spec.md` decisions document (the fold target) · `design.md` implemented shape · `plan.md` execution map. Templates: `templates/docs/main-bundle/`.

Each bundle carries one **lifecycle `Status:`** in its PRD header: `draft` · `active` · `paused`, then terminal `complete` · `superseded` · `abandoned`. Format: `**Status:** <state> — <optional qualifier>`.

- `draft` shaping/pre-signoff · `active` in execution · `paused` on hold.
- **Terminal status ⇒ fold, then delete.** In the PR that flips it (or the next chore PR), fold file-to-file into the main bundle — decisions → its `spec.md`; shape → its `design.md`; scope/success changes → its `prd.md` — then **delete the bundle directory**. Git history is the archival record (`git log --all -- '_specs/<scope>/<bundle>/'`).
- `superseded`/`abandoned` fold at most a one-line note into the main bundle's `spec.md`; the superseding bundle's PRD carries `Supersedes: <handle>`.

Review check: PRD `Status:` off-enum, or terminal with the bundle directory still present past its closing PR? → correct the enum / fold and delete.

## Rules numbering

lab-os owns `0x-*`; per-repo rules use `10+`. Base `0x` rules and the doc assets in the `scripts/rules_sync.py` manifest go into member repos as verbatim copies under a one-line sync header. A member repo carrying per-repo rules on `0x` slots is renumbered to `10+` before the first vendored sync. **Upstream lab-os (github.com/CAMELS-Research-Group/lab-os) owns both the convention and the canonical bytes.** Caravan (github.com/CAMELS-Research-Group/Caravan) is where edits are staged before syncing upstream; that script — also the drift check — runs there, not in member repos.
