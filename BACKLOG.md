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
- **Done when:** next workshop runs with a documented process change traced to collected
  feedback
- **Depends on:** —
- **Status:** ready

## B3 — Lab-wide backlog (this file)

- **Problem:** open lab work routes through one maintainer's personal file (retro Drop:
  "Watson as Backlog")
- **Who it helps:** whole team — shared visibility, anyone can groom or claim
- **Value:** retires a voted-down bottleneck; standing home for the other retro items
- **Owner:** Kiara
- **Rough size:** S
- **Done when:** this file merged to lab-os main and the first sprint-boundary grooming
  has happened
- **Depends on:** —
- **Status:** in-progress

## B4 — Time-boxing PRDs / working sessions

- **Problem:** PRD drafting and working sessions run unbounded (retro Try, assigned Arya)
- **Who it helps:** anyone running a planning or pairing session
- **Value:** predictable sessions; complements the 2-week sprint Keep
- **Owner:** Arya
- **Rough size:** S
- **Done when:** a time-box convention is written (target durations per session type) and
  applied in one real PRD session
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
