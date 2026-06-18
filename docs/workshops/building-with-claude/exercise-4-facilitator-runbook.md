# Exercise 4 facilitator runbook — First autonomous execution

Status: draft · Audience: facilitators running the *Building with Claude* day ·
Exercise spec: `site/docs/workshops/building-with-claude/exercises.md`

The highest-risk 30 minutes of the day. It's the first time participants let Claude run
without driving every step — trust is won or lost here. This runbook is for reading *during*
the block, not just before it: failure modes are scannable, each with a recovery move.

## What this exercise is actually teaching

Not "get the task done." The meta-skill is **let go, then verify** — hand off a whole task,
resist micromanaging, and then *prove* the result instead of trusting the agent's word. A
participant whose task fails but who *catches* the failure has succeeded at the lesson. Say
this out loud; it reframes every failure mode below as the point, not a disaster.

## Before you start the block

- Everyone is in their Exercise 3 worktree, on a clean tree, with one first-wave task chosen.
- Confirm each participant knows their project's **verification command** (the
  [contract](./verification-command-contract.md)). If a project has none, they fall back to
  eyeballing behavior — flag those people now; they'll need closer support at the verify step.
- Set expectations: "Some of your runs will go sideways. That's expected and it's the most
  useful thing that can happen — we'll work it in the debrief."

## Running the 30 minutes

- **0–5 — frame the hand-off.** Each participant writes Claude a single instruction: the task,
  the relevant context from their plan, and "execute this end to end." Coach them to over-include
  context and *under*-specify steps — the opposite of how most people prompt.
- **5–20 — let it run.** Hands off keyboards. Your job is to float and watch for the failure
  modes below, not to solve tasks. Resist fixing things for people.
- **20–30 — verify.** Everyone runs their verification command (or eyeballs behavior) and checks
  the diff. "Done" means the evidence says so. Collect what broke for the debrief.

## Failure modes & on-stage recovery

### 1. Claude does too much (scope creep)
- **Signs:** edits unrelated files, "helpfully" refactors, the diff is 3× the task.
- **Why:** the prompt was open-ended; the agent fills ambiguity with initiative.
- **Recovery:** stop the run, discard the worktree changes, re-prompt with a tighter boundary
  ("only touch X; don't refactor anything else"). Re-run.
- **Lesson to name:** scoping is a skill — a hand-off is a *contract*, not a wish.

### 2. Claims done, but isn't (the headline lesson)
- **Signs:** confident "done!" but the gate fails, or behavior is wrong.
- **Why:** self-report is not evidence — exactly what the Verify stage exists for.
- **Recovery:** *celebrate it.* Have them paste the failing output back to Claude and let it
  fix. This is the single best teachable moment of the day — slow down and make the whole room
  watch one.
- **Lesson to name:** always verify; the agent's word is a claim, not proof.

### 3. Participant won't let go (micromanaging)
- **Signs:** interrupts every few seconds, never actually gets an unattended run.
- **Why:** trust gap — letting an agent type unsupervised feels reckless.
- **Recovery:** sit with them. Have them write a *fuller* prompt up front so they feel safe
  stepping back, then physically take their hands off the keyboard for one run.
- **Lesson to name:** the worktree is the safety net — an autonomous run is safe to throw away,
  so let it run.

### 4. Environment / tooling stall
- **Signs:** wrong directory, worktree confusion, verification command missing or misbehaving.
- **Why:** Exercise 3 setup didn't fully land, or the project has no clean gate.
- **Recovery:** don't debug live for more than ~2 minutes — pair them with a working neighbor
  so they still see a successful run, and fix their setup in the meal break. For a missing gate,
  fall back to eyeballing behavior for now.
- **Lesson to name:** verification has to be one deterministic command — note projects that
  lack one as follow-up.

### 5. Task too big to finish in 30 minutes
- **Signs:** they picked an epic; no chance of a verifiable result in the window.
- **Why:** roadmap decomposition (Exercise 1) wasn't fine-grained enough.
- **Recovery:** help carve a thin, verifiable slice they *can* finish. A small completed task
  beats a big half-done one.
- **Lesson to name:** small, independently-verifiable tasks are what make autonomy work — loop
  back to the roadmap.

## Debrief (the 15 minutes after)

Don't make it a victory lap. Surface the breakage:

- Ask for **one failure** from the room and walk it end to end — what the agent did, how they
  caught it, how they recovered. Failures teach more than successes here.
- Name which failure mode each story maps to, so participants leave with a vocabulary for next
  time.
- Close on the through-line: **hand off fully, then verify ruthlessly.** That's the habit the
  rest of the day (and Exercise 5) builds on.

## Facilitator do / don't

- **Do** keep your hands off participants' keyboards — coach, don't take over.
- **Do** let runs fail when failing is safe (it's a worktree).
- **Don't** rescue every task — a rescued task skips the lesson.
- **Don't** let one stalled setup eat your attention; pair-and-defer so the room keeps moving.
