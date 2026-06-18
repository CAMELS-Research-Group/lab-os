# Cold-start handoff — "Building with Claude" co-working day

Hand this to a fresh session to resume workshop prep cold. Last updated 2026-06-18.

## What this is

Planning a **5-hour in-person co-working day** that teaches the *execution* half of the
lab-os SDD handbook (`site/docs/working-with-claude/`). Participants arrive with their own
Specs and Plans (built in a prior session) and leave having executed real Plan tasks with
Claude — turning a Plan into a roadmap, sequencing it, and running large chunks
autonomously with verification checkpoints.

This is **white-label** material: generic SDD-with-Claude, no CAMELS/lab specifics.

## Locked decisions

- **Schedule:** noon–5:00 PM, in-person. Meal cast as a late lunch / early dinner, placed
  mid-afternoon (2:45–3:30) right after the Exercise 4 debrief.
- **Format:** teach-by-doing. Short teach segments (≤15 min) each followed by an exercise on
  the participant's *own* Plan. Longest hands-on block is post-meal (energy-dip → self-directed
  work beats lecture).
- **Terminal required** for the execution exercises (3–5). Plain terminal or Claude desktop
  app were both on the table; desktop app works for the conversational exercises (1–2) but the
  worktree/autonomous-run workflow is terminal-centric, so terminal is the baseline.
- **Plan-readiness moved upstream**, not patched during the day: a virtual pre-session
  introduces plan generation, and execution-readiness worksheets (see Open threads) ensure
  nobody arrives with a too-thin Plan. This removed the need for a fallback sample repo.
- **Verification is written against a contract, not a tool** — see
  `verification-command-contract.md` in this folder.

## Final artifacts (status)

Published in PR #31 (merged to `main`, squash `37786d6`; handbook Workshops section is live):

- **Intro + agenda (noon–5:00)** — filed: `site/docs/workshops/building-with-claude/index.md`.
- **5 exercises** — filed: `site/docs/workshops/building-with-claude/exercises.md` (terminal-based;
  Exercise 5 sequential — no parallel subagents in plain terminal / desktop app).
- **Pre-flight checklist** — filed: `site/docs/workshops/building-with-claude/preflight.md`.
  Still to do: finalize wording and send the day before; require a "ready" reply.
- **Workshops overview + landing card** — filed: `site/docs/workshops/index.md`, `site/src/pages/index.mdx`.

Facilitator-internal (this folder, not published):

- **Verification command contract** — filed: `verification-command-contract.md`.
- **Exercise 4 facilitator runbook** — filed: `exercise-4-facilitator-runbook.md`.

## The 5 exercises (one line each)

1. **Plan → roadmap** (20m) — decompose your Plan into 5–15 executable tasks.
2. **Sequencing & dependencies** (20m) — mark what blocks what; group parallelizable "waves."
3. **Worktrees** (15m) — spin up an isolated, throwaway workspace for the first task.
4. **First autonomous execution** (30m) — hand Claude one task end-to-end, then *verify* it.
   The pivot of the day; highest live-risk moment.
5. **Scaling execution** (25m, sequential) — hand off a multi-task chunk, verify each.

## Open threads / next tasks

1. **Execution-readiness worksheets** (deferred to a future session). First move: define what
   "execution-ready" means concretely enough to put checkboxes on a worksheet — that
   conversation defines the worksheets. Pairs with the virtual plan-generation pre-session.
2. **Schedule the virtual pre-session** on generating plans (before the in-person day).
3. **Finalize + send the pre-flight checklist** the day before; require a "ready" reply.
4. **White-label leak** (found in PR #31 review, out of scope there): the published pages
   `site/docs/working-with-claude/index.md` and `plan.md` say "the lab's SDD lifecycle,"
   "owned by the working group," "Search the lab's repos," and reference dev-root `CLAUDE.md`
   lineage. File a GitHub issue and/or fix in a separate PR to make those pages white-label.
5. **White-label infra implements the verification command contract.** Design lead captured in
   the contract note: a dedicated AI-interaction surface as the default surface.

## Cold-start prompt (deferred items)

Paste into a fresh session to resume the deferred work:

```
Resuming deferred work from the lab-os "Building with Claude" workshop prep.

CONTEXT
- Repo: C:\Users\watso\Development\lab-os — a white-label spec-driven-development
  (SDD) handbook. Published docs are a Docusaurus site under site/ (gate:
  `cd site && npm run build`, which throws on broken links).
- The in-person workshop materials shipped in PR #31, MERGED to main (squash
  37786d6); the handbook Workshops section is live. Full prep state — locked
  decisions, artifacts, open threads — is in
  docs/workshops/building-with-claude/HANDOFF.md (on main). Read that first.

LOCKED DECISIONS (do not re-litigate)
- Published handbook content is WHITE-LABEL: no CAMELS, no lab repo names, no
  internal project names.
- Plan-readiness is handled UPSTREAM (a virtual pre-session + worksheets), not
  patched during the in-person day. Terminal is required for the execution work.

DEFERRED ITEMS TO PICK UP
1. Execution-readiness worksheets. The first move is the defining conversation:
   what does "execution-ready" mean concretely enough to become worksheet
   checkboxes? Produce that definition, then the worksheet. (My CLAUDE.md wants a
   PRD-first / interrogate-vague-requests approach before building.)
2. The virtual plan-generation pre-session. Design + schedule it: it introduces
   participants to generating plans before they arrive with one.
3. White-label leak. The published pages site/docs/working-with-claude/index.md
   and plan.md say "the lab's SDD lifecycle," "owned by the working group,"
   "Search the lab's repos," and reference "your dev-root CLAUDE.md" lineage. File
   a GitHub issue and/or fix in a separate PR to make those pages white-label.

Start by reading the HANDOFF, then ask me which of the three to take first.
```
