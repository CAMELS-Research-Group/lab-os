# Lab Backlog

Shared queue for **cross-repo / lab-level** work (repo-scoped work stays in that repo's
issues — see the routing rule in
[docs/proposals/2026-07-16-lab-wide-backlog.md](docs/proposals/2026-07-16-lab-wide-backlog.md)).
New items follow [templates/backlog-item.template.md](templates/backlog-item.template.md);
anyone may add to Inbox, shaping an item requires an owner. Groomed at each sprint boundary.

Read the **Index** first — it is the "what's ready right now" surface. Full items follow.
`## Items` is the last section: every `## ` heading below it must be an item heading.

## Index

<!-- generated from the Item blocks — edit those, then run:
     python3 scripts/backlog_lint.py --write-index -->

| id | title | owner | size | status |
|---|---|---|---|---|
| B1 | Convention-setting exercise | Jean | M | ready |
| B2 | Workshop process + feedback iteration | Watson | M | ready |
| B3 | Lab-wide backlog (this file) | Kiara | S | in-progress |
| B4 | Time-boxing PRDs / working sessions | Arya | S | ready |
| B5 | Backlog-lint (CI hygiene check for BACKLOG.md) | Kiara | M | in-progress |
| B10 | Give main-PRD success criterion 4 an applicable test | Watson | S | ready |
| B11 | Decide whether main `plan.md` §In flight restates or links | Watson | S | ready |
| B12 | Decide whether the main-bundle log template needs a non-fold entry grammar | Watson | S | ready |
| B13 | Dispose of the superseded `templates/PRD.template.md` | Watson | S | ready |
| B14 | Record the never-required constraint on path-filtered CI jobs | Watson | S | ready |
| B15 | Check `_specs/lab-os/` bundle statuses mechanically | Watson | M | ready |
| B16 | Dispose of the legacy `docs/` planning surfaces | Watson | M | ready |
| B17 | Archive the `project_log.md` overflow (time-critical) | Watson | S | ready |
| B18 | Retire the two superseded Standing Decisions index lines | Watson | S | ready |
| B19 | Amend `04-docs.md` for the main bundle's `**Living**` header | Watson | M | ready |
| B20 | Land the rules-tier budget-raise round-trip in lab-os | Watson | S | ready |

## Inbox

<!-- Raw, unshaped ideas from the 2026-07-08 retrospective. Move one up to Items once it
has a Problem and a Done-when and an owner. -->

- UX pass on work surfaces (tasks / reviews) — retro "Lacked: user-friendly interfaces"
- Gamify progress updates — retro Try
- Working pairs — retro Try
- Multiple subscriptions — retro Try
- Virtual servers — retro Try
- Interoperable milestones / handing off between workstreams — retro Try
- Bot routines — retro running task

## Items

## B1 — Convention-setting exercise

- **Problem:** the lab has conventions scattered across repos and heads; no shared mechanism
  for proposing, ratifying, and reinforcing them (retro "Longed for: conventions/runbooks")
- **Who it helps:** every member and every agent session that inherits the rules
- **Value:** the retro's most-demanded gap; unblocks consistent multi-agent workflows
- **Owner:** Jean
- **Rough size:** M
- **Done when:** a convention-proposal template exists and one convention has gone
  proposal → ratified → written into a `.claude/rules/` or handbook page via it
- **Depends on:** —
- **Status:** ready

## B2 — Workshop process + feedback iteration

- **Problem:** workshop format needs optimization (retro "Lacked: workshop optimization");
  no feedback loop from session to session
- **Who it helps:** workshop facilitators and participants
- **Value:** workshops are the lab's onboarding + convention surface; compounding return
- **Owner:** Watson
- **Rough size:** M
- **Done when:** the next workshop's committed process notes (under `docs/workshops/`)
  record a process change traced to the previous session's collected feedback
- **Depends on:** —
- **Status:** ready

## B3 — Lab-wide backlog (this file)

- **Problem:** open lab work routes through one maintainer's personal file (retro Drop:
  "Watson as Backlog")
- **Who it helps:** whole team — shared visibility, anyone can groom or claim
- **Value:** retires a voted-down bottleneck; standing home for the other retro items
- **Owner:** Kiara
- **Rough size:** S
- **Done when:** the first sprint-boundary grooming commit touching this file is on
  lab-os `main`
- **Depends on:** —
- **Status:** in-progress

## B4 — Time-boxing PRDs / working sessions

- **Problem:** PRD drafting and working sessions run unbounded (retro Try, assigned Arya)
- **Who it helps:** anyone running a planning or pairing session
- **Value:** predictable sessions; complements the 2-week sprint Keep
- **Owner:** Arya
- **Rough size:** S
- **Done when:** a time-box convention (target durations per session type) is committed
  under `docs/` and one committed PRD cites the time-box it ran under
- **Depends on:** —
- **Status:** ready

## B5 — Backlog-lint (CI hygiene check for BACKLOG.md)

- **Problem:** the backlog's item schema + readiness bar ("Done when" = one checkable
  condition) are enforced only by grooming discipline; malformed items, a stale Index,
  dangling `Depends on`, and unsplit `L` items marked `ready` merge silently. No CI
  equivalent to `log-lint` / `docs-budget` / `merge-bar-check`.
- **Who it helps:** everyone who adds or grooms items — and the agent sessions that add
  them; keeps the Index a trustworthy "what's ready" surface.
- **Value:** makes the readiness bar true *by construction* instead of by memory; squarely
  the retro's "Tooling for Agents / Power Users" gap; reuses the lab's existing lint pattern.
- **Owner:** Kiara
- **Rough size:** M
- **Done when:** a PR that violates the item schema (missing field / bad `Done when` / stale
  generated Index / dangling `Depends on` / `L` marked `ready`) fails CI, and a well-formed
  one passes — running on every PR alongside the sibling lints.
- **Depends on:** B3
- **Status:** in-progress

## B10 — Give main-PRD success criterion 4 an applicable test

- **Problem:** the fourth standing success criterion in `_specs/lab-os/main/prd.md` ("a
  reader arriving cold can answer 'what is implemented here right now' from `design.md`")
  states no pass/fail test a reader can apply, so nothing can confirm or refute it;
  criteria 1–3 carry the whole measurable load
- **Who it helps:** anyone auditing the main bundle against its own criteria, and the
  reviewer of every folding PR
- **Value:** an unapplicable criterion dilutes the three that work; cheap to fix while the
  bundle is young
- **Owner:** Watson
- **Rough size:** S
- **Done when:** the fourth success criterion in `_specs/lab-os/main/prd.md` states a
  condition a reader can check, or is gone
- **Depends on:** —
- **Status:** ready

## B11 — Decide whether main `plan.md` §In flight restates or links

- **Problem:** `_specs/lab-os/main/plan.md` §In flight restates the PRD's open question
  about the `2026-07-31-timeboxing` bundle's stale `Status:` instead of resting on its link
  to it, giving one fact two homes that can drift (`04-docs.md` §Single source)
- **Who it helps:** anyone reading the main bundle for the current state of an in-flight
  slice; the next person who has to update that fact
- **Value:** settles a single-source question in the bundle that models the convention for
  every other scope
- **Owner:** Watson
- **Rough size:** S
- **Done when:** `_specs/lab-os/main/plan.md` §In flight no longer restates the PRD's open
  question, or records in place why the restatement is deliberate
- **Depends on:** —
- **Status:** ready

## B12 — Decide whether the main-bundle log template needs a non-fold entry grammar

- **Problem:** `templates/docs/main-bundle/log.template.md` offers one entry shape and it is
  fold-only (`Bundle:` / `Folded:` / `Deleted in:` / `Recover:`), while `03-logging.md`
  §Entry format binds bundle logs generally — a main-bundle entry that is not a fold (a
  restructure, a correction) has no stated shape to follow
- **Who it helps:** every scope that stands up a main bundle from the template and hits a
  non-fold entry
- **Value:** the template is the normative scaffold; an unstated case gets improvised
  differently in each repo
- **Owner:** Watson
- **Rough size:** S
- **Done when:** `templates/docs/main-bundle/log.template.md` carries a second entry shape
  for non-fold entries, or states that `03-logging.md` §Entry format governs them
- **Depends on:** —
- **Status:** ready

## B13 — Dispose of the superseded `templates/PRD.template.md`

- **Problem:** `templates/PRD.template.md` is a superseded scaffold retained pending
  consolidation with `templates/docs/planning/prd.template.md`; two PRD scaffolds means a
  new bundle can be started from the wrong one
- **Who it helps:** anyone scaffolding a PRD, and the agents that copy templates unread
- **Value:** removes a live wrong-answer surface; also closes a `design.md` known gap that
  currently carries no tracking pointer
- **Owner:** Watson
- **Rough size:** S
- **Done when:** `templates/PRD.template.md` no longer exists as a separate scaffold, its
  content consolidated into `templates/docs/planning/prd.template.md`
- **Depends on:** —
- **Status:** ready

## B14 — Record the never-required constraint on path-filtered CI jobs

- **Problem:** a path-filtered job must never become a required status check — a required
  context that never reports holds PRs at "Expected" forever — but nothing in this repo
  states it; the constraint lives only in convention and in this bundle's gap list
- **Who it helps:** whoever next edits branch protection, and every PR that would otherwise
  hang on a check that never runs
- **Value:** the failure mode is silent and repo-wide; one committed sentence prevents it
- **Owner:** Watson
- **Rough size:** S
- **Done when:** the constraint is stated in a committed surface — a grep for it across
  `.github/workflows/` and `.claude/rules/` returns a hit
- **Depends on:** —
- **Status:** ready

## B15 — Check `_specs/lab-os/` bundle statuses mechanically

- **Problem:** nothing fails CI when a bundle PRD `Status:` goes stale against its shipping
  PR; `spec-plan-analyzer` flags it only when a PR happens to touch that bundle, so a
  terminal bundle can sit unfolded and a merged slice can keep reading `active`
- **Who it helps:** every reader who trusts a PRD `Status:` as the answer to "is this the
  active plan?"
- **Value:** makes the bundle lifecycle true by construction, the same way the sibling lints
  do for logs and budgets
- **Owner:** Watson
- **Rough size:** M
- **Done when:** a PR that leaves a `_specs/lab-os/**` PRD `Status:` off-enum, or terminal
  with its bundle directory still present, fails CI
- **Depends on:** —
- **Status:** ready

## B16 — Dispose of the legacy `docs/` planning surfaces

- **Problem:** `docs/prds/`, `docs/proposals/`, and `docs/superpowers/{plans,specs}` predate
  the `_specs/` bundle convention this scope codifies, and nothing states whether they are
  in scope, frozen, or slated to fold — so `_specs/lab-os/` is not the whole planning record
  for this repo
- **Who it helps:** anyone searching for this repo's planning history, and the main bundle's
  claim to be the implemented-state authority
- **Value:** either the surfaces fold in or they are declared archive; both beat the current
  undeclared middle
- **Owner:** Watson
- **Rough size:** M
- **Done when:** those three legacy `docs/` planning surfaces carry a stated disposition —
  in scope, frozen, or slated to fold — in a committed surface
- **Depends on:** —
- **Status:** ready

## B17 — Archive the `project_log.md` overflow (time-critical)

- **Problem:** `project_log.md` measures 22,979 B against a 15,360 B budget while
  `standards.yml` runs `docs-budget` at `enforce: true`; the FAIL threshold is 23,040 B, so
  roughly 61 B of headroom remain and the next entry of any size turns the gate red for
  every PR in the repo. `03-logging.md` §File structure & overflow prescribes the fix: a
  dedicated `chore: archive log overflow` PR moving oldest entries to
  `project_log_archive.md`, prepended as a block, order preserved, byte-identical modulo EOL
- **Who it helps:** every open and future PR — a red `docs-budget` blocks all of them
- **Value:** time-critical: this is the one item here that goes from warning to repo-wide CI
  failure on the next log entry anyone writes
- **Owner:** Watson
- **Rough size:** S
- **Done when:** `python3 scripts/docs_budget.py --root . --enforce` reports `project_log.md`
  below its FAIL threshold
- **Depends on:** —
- **Status:** ready

## B18 — Retire the two superseded Standing Decisions index lines

- **Problem:** `project_log.md`'s Standing Decisions index still lists "2026-06-23 06:30 —
  Fork-of-lab-os is the default Claude-powered dev home · #43" and "2026-06-23 07:51 — Plans
  track at the fork level · #44" as standing, yet D16/D17 moved the dev home to Caravan.
  Both entries are merged and therefore immutable, so the fix is a NEW entry carrying
  `Supersedes:` plus removal of the two index lines in the same PR — never an edit
- **Who it helps:** anyone who reads the index first, as the rules instruct, and gets two
  reversed decisions presented as current
- **Value:** the index is the "what is still true" surface; two false lines make the whole
  surface untrustworthy
- **Owner:** Watson
- **Rough size:** S
- **Done when:** neither line remains in `project_log.md`'s Standing Decisions index and a
  superseding entry records why
- **Depends on:** B17
- **Status:** ready

## B19 — Amend `04-docs.md` for the main bundle's `**Living**` header

- **Problem:** every main-bundle file heads with `**Living**` where `04-docs.md` §Bundle
  lifecycle states an unqualified `Status:` enum, so the templates and the rule disagree;
  the reviewer's read is that this is a gap in the rule, not a defect in the bundle. Rules
  are staged in Caravan and reach lab-os as a round-trip — this edit is authored there,
  never here
- **Who it helps:** every scope standing up a main bundle, and every reviewer checking a
  bundle header against the enum
- **Value:** closes the last disagreement between the main-bundle templates and the rule
  that governs them
- **Owner:** Watson
- **Rough size:** M
- **Done when:** `04-docs.md` §Bundle lifecycle states how a main bundle's header relates to
  the `Status:` enum
- **Depends on:** —
- **Status:** ready

## B20 — Land the rules-tier budget-raise round-trip in lab-os

- **Problem:** `.claude/rules/04-docs.md` sits above 1.0x of the per-file budget in force here,
  so `docs-budget` reports a WARN on every lab-os PR and the check's signal is permanently
  noisy. The raise (12 KB `CLAUDE.md` / 8 KB per rules file) plus the 48 KB always-loaded
  aggregate cap is staged in Caravan; lab-os holds the canonical bytes and receives it as a
  round-trip, open as PR #79 and not yet landed
- **Who it helps:** every lab-os contributor reading a PR's checks, and every member repo that
  calls the reusable `docs-budget` workflow and inherits the thresholds
- **Value:** a check that warns on every PR trains readers to ignore it; landing the round-trip
  restores WARN as a signal rather than background noise
- **Owner:** Watson
- **Rough size:** S
- **Done when:** `python3 scripts/docs_budget.py --root .` reports no warn-zone
  `.claude/rules/*.md` surface on lab-os `main`
- **Depends on:** —
- **Status:** ready
