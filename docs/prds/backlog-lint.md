# Backlog-lint — PRD

**Status:** draft <!-- draft | active | paused | complete -->
**Date:** 2026-07-22 · **Repo:** lab-os

> **Decisions live in `project_log.md`, not here.** When a decision is reached while executing
> this PRD, log it there (`.claude/rules/03-logging.md` entry triggers). This PRD stays decision-free.

---

## Problem

The lab backlog's conventions live in `BACKLOG.md` and `templates/backlog-item.template.md`, but nothing enforces them. The **readiness bar** — a single checkable "Done when" — plus the Index↔Items correspondence, the status ladder, size discipline, and dependency references are all things a person must remember to check at grooming time. Nothing catches a malformed item, an Index row with no matching block, a `Depends on` that points at an id that doesn't exist, or a size-`L` item marked `ready` before it merges. As more members — and agent sessions — add items, that drift is silent and cumulative: the Index stops being a trustworthy "what's ready" surface.

The lab already enforces its other text conventions by CI (`log-lint`, `docs-budget`, `merge-bar-check`). The backlog has no equivalent, so its readiness bar is a rule people remember rather than an invariant the repo guarantees.

---

## Success criteria

<!-- Observable, falsifiable. -->

- A PR that adds or edits `BACKLOG.md` **fails CI** when an item is missing a required field, its `Done when` is empty / a placeholder / more than one condition (wrapped continuation lines join into one value; a nested list inside the field fails), its `Status` is off the `inbox → ready → in-progress → done` ladder, the committed Index differs from the Index rendered from the Item blocks, a `Depends on` id doesn't resolve, the dependency graph contains a cycle, or a size-`L` item is marked `ready`.
- A well-formed backlog PR **passes with no human review of format** — reviewers spend attention on the item's content, not its shape.
- The check runs **on every PR, like the sibling lints** (`log-lint`, `docs-budget`, `merge-bar-check`) — its cost is seconds, so a changed-files gate is not worth the extra conditional. <!-- amended 2026-07: the original "only on PRs that touch BACKLOG.md" claim never matched the shipped workflow; the sibling convention won. -->
- The field schema the linter enforces is **single-sourced with `backlog-item.template.md`** — editing the template cannot silently diverge from what CI checks.
- Rollout is **warn-only until the first green run, then enforcing** (matching `docs-budget`), with a documented `backlog-lint:override` label for intentional exceptions (matching `log-lint`).

---

## Scope

### In scope

- `scripts/backlog_lint.py` + `.github/workflows/backlog-lint.yml`, siblings to the existing lint tooling (stdlib-only, matching `log_lint.py`'s dependency posture).
- Validation of: required fields per item; `Done when` is one non-placeholder condition; `Status ∈ {inbox, ready, in-progress, done}`; `Rough size ∈ {S, M, L}` with `L` not permitted at `ready` (must be split first); the Index as a **derived projection** of the Item blocks — a `--write-index` renderer generates it, and the committed table must byte-match the render (subsumes orphan/duplicate/missing-row checks); `Depends on` referential integrity + acyclicity.
- A lightweight public-tier spot check (flag obvious private-repo paths / gated-dataset identifiers appearing in an item — lab-os is public).
- **Schema source:** the required-field list + enums are **parsed from `backlog-item.template.md`** (the single source), distinguishing a filled field from an unfilled `<placeholder>`.
- **`Done when` test is structural:** present, single line, non-placeholder → hard fail. A `Done when` with no concrete artifact reference (command / file / observable behavior) is a **warning**, not a failure — never red a legitimate item over phrasing.
- Warn-until-green rollout + `backlog-lint:override` label handling.
- Unit tests covering each failure class (each rule mutation-proven to fail a bad fixture and pass a good one).

### Out of scope

- Generated views / dashboards (the derived "ready & unblocked" / dependency-graph render) — ships as its own follow-on PR stacked on this one, not here.
- Grooming notifications / scheduled sprint-boundary digests — same follow-on PR, not here.
- Any change to the routing rule or the backlog convention itself — this enforces the existing convention, it does not redefine it.
- Cross-repo issue aggregation.
- Judging item *content* quality (whether a Problem is well-posed) — format and referential integrity only.

---

## Constraints

- **Budget:** none — a Python script + a GitHub Action; no spend, no new service.
- **Timeline:** none hard; slots into a sprint boundary.
- **Data / access:** none; operates on one public file. lab-os is public → the linter and its failure messages must themselves be public-tier safe.
- **Infra:** reuse the existing GitHub Actions + `scripts/` pattern; Python stdlib only (confirm against `log_lint.py`).
- **Approvals:** ships via PR review on lab-os like every lab convention. It changes CI (shared state) → the merge is the gate; warn-only first de-risks the rollout.
- **Dependencies:** depends on the lab-wide backlog (B3, `in-progress` — it lands with the backlog docs PR this branch stacks on). Pairs with — but does not require — the generated-view work; Phase 3 (optional) exposes its parsed model so that view generator can reuse it.

---

## Plan (phased)

### Phase 1 — Linter + single-sourced schema (warn-only)

**Goal:** catch every malformed backlog item on PR, without yet blocking merge.
**Deliverables:** `scripts/backlog_lint.py`, the field schema single-sourced with `backlog-item.template.md`, per-failure-class unit tests, a warn-only `backlog-lint.yml` run on every PR alongside the sibling lints.
**Work bundle:** <!-- link once created -->

### Phase 2 — Enforce + override + document

**Goal:** flip to failing after the first green run; make the check discoverable.
**Deliverables:** enforcing workflow, `backlog-lint:override` label handling, a "what backlog-lint checks" line in `site/docs/tooling-tour.md` — tooling doc, **not** a `.claude/rules/` entry (a lint's behavior isn't a hard rule; don't spend rule-file budget).
**Work bundle:** <!-- link once created -->

### Phase 3 (optional) — Expose the parsed model

**Goal:** let the future generated-view work reuse the parser rather than re-parse `BACKLOG.md`.
**Deliverables:** a small importable parse function returning the structured backlog.
**Work bundle:** <!-- deferred; only if the generated-view idea is picked up -->

---

## Open questions

None open — initial scoping resolved 2026-07-22: schema parsed from `backlog-item.template.md`; `Done when` checked structurally with concreteness as a warning; behavior documented in `site/docs/tooling-tour.md`, not a `.claude/rules/` file; filed as its own backlog item **B5**. Decisions recorded in `project_log.md` with the B5 PR.
