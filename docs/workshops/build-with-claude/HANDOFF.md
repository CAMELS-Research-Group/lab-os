# Cold-start handoff — "Building with Claude" co-working day

Hand this to a fresh session to resume workshop prep cold. Last updated 2026-06-17.

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

- **Intro paragraph** — final (in chat; not yet filed).
- **Agenda, noon–5:00** — final (in chat; not yet filed).
- **5 exercises** — final, terminal-based, Exercise 5 is sequential (no parallel subagents,
  unavailable in plain terminal / desktop app). In chat; not yet filed as participant handouts.
- **Pre-flight setup checklist** — drafted, terminal-centric (tests Claude Code auth, git,
  clean tree, Plan location, `git worktree add/remove`). Ready to finalize and send the day
  before. In chat; not yet filed.
- **Verification command contract** — filed: `verification-command-contract.md`.

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
4. **Exercise 4 facilitator runbook** (offered, not yet written) — speaker notes + the 3–4
   most likely failure modes and how to recover on stage. Highest-risk moment, worth doing.
5. **File the agenda + 5 exercises** as participant handouts (currently chat-only). This folder
   is the proposed home; confirm location.
6. **White-label infra implements the verification command contract.** Design lead captured in
   the contract note: a dedicated AI-interaction surface as the default surface.
