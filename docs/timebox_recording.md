# Timebox Recording & Logging Guide

**Status:** v1.0, 2026-07-31. Owner: Arya Kurup. Scope: how timeboxes are
recorded and logged — for human sittings under
[`timeboxing.md`](timeboxing.md) and for **agent-executed task boxes**, which
this guide defines. Companion skill: `.claude/skills/timeboxing/` (the
protocol an agent follows to run a boxed task).

## Why a recording layer

The timeboxing standard's calibration loop only works if rows actually get
written. Manual discipline hasn't produced them. This guide moves recording
from discipline to protocol: the party that ends a box — human or agent — owes
exactly one calibration row, and for agent boxes the skill writes it
automatically.

## Agent task boxes

The lab's dev workflow has three phases with different owners:

| Phase | Owner | Boxed? |
|---|---|---|
| Brainstorm / context assembly | User + agent, joint | By the human standard ([`timeboxing.md`](timeboxing.md)), if at all |
| Development work | Agent, autonomous | **Yes — this guide + the `timeboxing` skill** |
| Review → hard pass | User | Never. Review ends when the reviewer decides |

An **agent task box** starts when the user hands the agent an ironed-out task
(the "go") and ends at handoff-to-review — when the agent presents the work,
or when the box expires, whichever comes first. The user's review time is
outside the box by definition.

### Default boxes for agent tasks

Seeded values — not yet calibrated. After ~10 rows exist, compare and correct
(same reference-class mechanism as the human table).

| Agent task type | Box | Exit criterion shape |
|---|---|---|
| Dev task (feature, refactor, fix) | 30 min | Named behavior demonstrably works (test run, command output) |
| Docs task | 15 min | Named sections present and cross-linked |
| Debug | 20 min, then escalate to user | Root cause named, or reproducible minimal case written down |
| Research spike | 15 min | A written claim, or a written "dead end, because…" |

A box without an exit criterion is just an alarm clock: the agent must refuse
to start a boxed task until one exists — stated by the user, or proposed by
the agent from the shapes above. A self-proposed criterion stands unless the
user corrects it, and is marked `self-proposed criterion` in the row's Note
so the exit-met judgment can be weighed (and overridden) at review.

## Recording mechanics

- **State file, not memory.** At box start the agent writes a state file in
  its session scratchpad (never the repo working tree) recording: ISO start
  timestamp, planned minutes, goal, exit criterion, task type. Elapsed time is
  always computed by date arithmetic against that timestamp — never estimated
  from recall.
- **Pauses are recorded, not estimated.** When a box pauses — an approval
  gate, an incident capture, a requester interruption — the agent appends a
  pause timestamp to the state file, and a resume timestamp when work
  restarts. Actual = elapsed minus the recorded pause intervals; the row's
  Note names the pause reason. A pause with no timestamp is an estimate, and
  estimates are what this mechanic exists to eliminate.
- **Checkpoints at natural seams.** After finishing a subtask, before opening
  a new file/subsystem, after a test run — the agent checks elapsed vs
  planned. At ≥80% consumed: stop adding scope, cut to what's done-able,
  park the rest on the handoff's **next-box list**.
- **One extension max, half the original box, written reason.** The reason
  goes into the state file at extension time and into the row's Note at box
  end. A second overrun ends the box where it stands — the remainder is
  next-box material and the row records `exit met? no`.
- **Expiry discovered late = notify + hand off.** Checkpoints happen at
  natural seams, so an expired box is usually noticed after the fact. When a
  check finds elapsed ≥ 100%: tell the user plainly that the box is over
  (one line, elapsed vs planned), then go straight to the handoff — no
  extension after expiry. The Note records how late the discovery was.

## Logging rules

**One row per box end**, appended to the calibration file. Columns are fixed
by [`timebox_calibration.md`](timebox_calibration.md):

| Date | Session type | Artifact | Planned | Actual | Exit met? | Note |
|---|---|---|---|---|---|---|

- **Session type** for agent boxes is prefixed `agent-`: `agent-dev`,
  `agent-docs`, `agent-debug`, `agent-spike`. Human rows keep the standard's
  types. One column, one reference class, filterable by prefix.
- **Exit met?** is judged **by the agent at the timer**, against the stated
  criterion, with one line of evidence in the handoff (e.g. "yes — pytest 14
  passed"). Judged at the timer means: an extension that later meets the
  criterion still records the judgment as of the original box end, per the
  calibration file's own header. When the criterion was proposed by the
  agent rather than stated by the user, the Note records
  `self-proposed criterion` — the agent set its own bar, so review and
  calibration should weigh that judgment accordingly.
- **Target file: the repo where the task lives, where that repo has
  adopted the practice.** Rows are appended to the calibration file of the
  repo the ticket/task belongs to — `docs/timebox_calibration.md` for
  lab-os work, else that repo's `timebox_calibration.md` **if it already
  exists**. Creating that file is a human's adoption act, not the agent's:
  the agent never seeds the convention into a repo by side effect. Where the
  task repo has no calibration file, rows fall back to lab-os's
  `docs/timebox_calibration.md`. `TIMEBOX_CALIBRATION_FILE` env var
  overrides everything when set. The reference class lives beside the work
  it measures; a cross-repo roll-up is a grep away.
- **Never `project_log.md`.** Rows are telemetry; the lab logging standard's
  entry triggers exclude them, at every altitude.

## The override flow (review gate)

The agent's exit-met judgment is provisional until the hard pass:

1. The handoff report reproduces the appended row verbatim, with the
   judgment's evidence line.
2. At review the user may override `Exit met?` or amend the Note in one
   sentence ("override: no — the test asserts the wrong thing").
3. **Before the row is committed**, the agent corrects it in place — the
   append-only rule protects history, and an uncommitted row isn't history
   yet. **After commit**, the original row stands and the agent appends a
   correction row (Note: `corrects <date> row`).

## Handoff report contract

Every boxed agent task ends with a report containing, in order: what was
built and how it was verified · planned vs actual minutes · exit-met judgment
+ evidence line · the appended calibration row, verbatim · what was
scope-hammered out (the next-box list) · extension reason, if one was taken.

## When not to box an agent task

Same exemptions as the human standard: sign-off conversations, live
incident state with an unreproducible repro, and finishing-the-thought on
the stated goal when the timer fires mid-flow (log the overrun; the box
kills drift, not momentum). The exemptions align; extension timing does
not — humans may extend when the timer fires, agent boxes only before
expiry (see § Recording mechanics), by design.
