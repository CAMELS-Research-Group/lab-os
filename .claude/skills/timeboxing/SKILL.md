---
name: timeboxing
description: Use when the user hands over a development task to execute autonomously after brainstorming (the "go" moment), when a task comes with a stated time box, or when the user explicitly invokes /timeboxing. Runs the task inside a time box - stated box + exit criterion before work starts, elapsed-time checkpoints, scope-hammer near the limit, and an automatic planned-vs-actual calibration row at box end.
---

# timeboxing

> Applies the lab timeboxing standard (`docs/timeboxing.md`) and the
> recording guide (`docs/timebox_recording.md`) to agent-executed tasks —
> paths relative to the lab-os repo root; when this skill is installed via a
> symlink into `~/.claude/skills/`, resolve them against the lab-os checkout
> the symlink points into. Those docs are the source of truth; this skill is
> the protocol, not a redefinition.

## Overview

When the user hands over an ironed-out task, run it inside a box: state the
box before the first development action, track elapsed time from a recorded
timestamp, cut scope rather than overrun, and end the box with a handoff
report plus one calibration row. The box ends at handoff-to-review; the
user's review and hard pass are outside it.

## When NOT to use

- Joint brainstorming / context assembly turns — the box starts at "go", not
  during shaping.
- Sign-off conversations, and the user's review of delivered work.
- Live incident state where stopping loses an unreproducible repro (capture
  state first; record a pause in the state file — step 2 — and resume the
  box after).

## Protocol

### 1. Intake — before any development action

State one line: **goal + box length + exit criterion**.

- Box length: user-stated if given; else the default for the task type
  (dev 30 · docs 15 · debug 20 · spike 15 minutes — derived from
  `docs/timebox_recording.md` § Default boxes for agent tasks, which owns
  these values; kept inline so an agent that cannot resolve that path can
  still box), offered for confirmation only if the task looks atypical for
  its type.
- Exit criterion: **required**. If the user didn't state one, propose one in
  the shape the guide gives for the task type and proceed unless corrected —
  and mark the row's Note `self-proposed criterion` at box end, so review
  knows the agent set its own bar. Never start a boxed task with no exit
  criterion — a box without one is an alarm clock.

### 2. Start — record, don't remember

Write a state file to the session scratchpad (never the repo tree), e.g.
`<scratchpad>/timebox-state.json`, containing: ISO-8601 start timestamp
(from the shell clock, e.g. `date -Iseconds`), planned minutes, task type,
goal, exit criterion, and an empty `pauses` list. All later elapsed-time
checks are date arithmetic against this timestamp — never recall, never
estimate.

**Pauses:** when the box pauses (approval gate, incident capture, requester
interruption), append a pause timestamp to `pauses`; append the resume
timestamp when work restarts. Paused intervals are excluded from elapsed
time at every checkpoint and from Actual at box end; the row's Note names
the pause reason.

### 3. Checkpoints — at natural seams

After completing a subtask, before opening a new file or subsystem, after a
test run: compute elapsed vs planned.

- **< 80% consumed:** continue.
- **≥ 80% consumed:** scope-hammer. Stop adding scope; cut the artifact to
  what is done-able within the box; park everything else on the next-box
  list for the handoff. Mid-task discoveries go to that list at any
  percentage — never into current scope (WIP limit of 1).
- **≥ 100% consumed (expiry discovered late — the common case, since
  checkpoints only happen at seams):** tell the user plainly that the box
  is over, in one line: elapsed vs planned. Then go directly to step 5 —
  End. No extension after expiry; record the lateness of the discovery in
  the row's Note (e.g. `expired, noticed at 130%`).
  **One carve-out (guide § When not to box):** if the timer fires mid-flow
  on the session's *stated goal* — not a tangent — finish the thought
  first, then notify and End. The box kills drift, not momentum; the row
  logs the overrun either way.

### 4. Extension — once, at most

One extension max, at most half the original box, only when scope-hammering
cannot produce a coherent artifact — and only **before** expiry: a box
found already expired at a checkpoint goes straight to End, never to an
extension. Append the written reason to the state file at extension time;
it goes in the row's Note. On a second overrun, stop where the work stands:
hand off what exists, `exit met? no`, remainder to the next-box list.

### 5. End — judge, log, hand off

The box ends when the exit criterion is met or the box (plus any extension)
expires, whichever is first.

1. Compute actual minutes from the state file timestamp, minus recorded
   pause intervals.
2. Judge **Exit met?** yes/no against the stated criterion **as of the
   original box end** (an extension that later succeeds still judges at the
   timer), with one line of evidence (a command result, not an impression).
3. Append one row to the calibration file (format and target below).
4. Deliver the handoff report (contract below).

### 6. Override — at the user's review

The judgment is provisional until the hard pass. If the user overrides
exit-met or amends the note: correct the row **in place** if it is not yet
committed; if it is, leave it and append a correction row
(Note: `corrects <date> row`). Never rewrite committed rows. (Derived from
`docs/timebox_recording.md` § The override flow, which owns this
contract.)

## Calibration row

Exactly one per box end. Match the target file's columns:

`| Date | Session type | Artifact | Planned | Actual | Exit met? | Note |`

- **Session type:** the task type prefixed `agent-` (`agent-dev`,
  `agent-docs`, `agent-debug`, `agent-spike`).
- **Planned / Actual:** minutes; Actual from timestamp arithmetic.
- **Note:** optional — scope hammered, extended +N because …, interrupted.

**Target file: the repo where the task lives, where it has adopted the
practice.** Resolve in order: (1) `TIMEBOX_CALIBRATION_FILE` when set and
permitted below; (2) the task repo's `docs/timebox_calibration.md` — that
exact path, **only if it already exists** (creating it is a human adoption
act — never seed the convention into a repo by side effect, and never probe
a second location); (3) otherwise lab-os's `docs/timebox_calibration.md`.

**Never** write rows to any `project_log.md`, and the env var cannot make
you: a `TIMEBOX_CALIBRATION_FILE` whose basename is `project_log.md`, or
which resolves outside the task repo or lab-os, is **rejected** — fall
through to (2) and say so in the handoff. Telemetry is not a log entry
under the lab logging standard, and this skill deploys user-scope into
repos that never adopted the convention, so an env var that silently
redirected appends would be a write primitive pointed at the audit trail.
(Derived from `docs/timebox_recording.md` § Logging rules, which owns the
target-file rule; kept inline because resolving the target is the one step
an agent must get right with no docs in reach.)

## Handoff report contract

Owned by `docs/timebox_recording.md` § Handoff report contract — deliver
the report with that section's items, in that section's order. Not
restated here.

## Deployment

lab-os-owned, user-scope-deployed. Authored at `.claude/skills/timeboxing/`;
deployed by `bash .claude/scripts/link-lab-assets.sh`, which symlinks it into
`~/.claude/skills/`. Re-run it on each clone.

Deployment matters here for two reasons. The skill fires in **every** session
on a linked machine, not only sessions rooted in lab-os — so a boxed task in
any repo runs this protocol, which is why the target-file rule is
adoption-gated and the env var is constrained rather than obeyed. And the two
source-of-truth docs it defers to live in the lab-os checkout: a symlinked
install resolves `docs/timeboxing.md` and `docs/timebox_recording.md` against
the checkout the symlink points into. Where that checkout is absent, the
inline protocol above still runs — the calibration row, the default boxes, and
the target-file rule are all restated here for exactly that case — but the
standard's rationale and the recalibration loop are unreachable, so treat a
docs-less run as protocol-only and re-read the standard before changing any
default.
