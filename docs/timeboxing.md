# Timeboxing Working Sessions

**Status:** v1.0 — lab adoption: PR #66 (2026-07-31); adopted in the
originating fork 2026-07-24. Owner: Arya Kurup. Scope: all working
sessions — spec/PRD drafting, review remediation, research spikes, debugging,
decision sessions. Calibration data lives in
[`timebox_calibration.md`](timebox_calibration.md). At-a-glance version:
[`timeboxing_quickref.md`](timeboxing_quickref.md). Agent-executed task
boxes (recording, logging, defaults): [`timebox_recording.md`](timebox_recording.md).

## Principles

Three findings from established practice do the load-bearing work here:

1. **Work expands to fill the time available** (Parkinson's Law). An unbounded
   session doesn't produce a better artifact — it produces a longer session.
   The box is the forcing function that converts "polish indefinitely" into
   "ship, park, or explicitly extend."
2. **We are systematically bad at estimating our own work** (the planning
   fallacy — Kahneman & Tversky). The fix is reference-class forecasting: set
   box lengths from what past sessions *actually took* (the
   [`timebox_calibration.md`](timebox_calibration.md) record), not from how
   long this one feels like it should take.
3. **Fix time and budget; flex scope** (Basecamp's *Shape Up* — "appetite,"
   not estimate). The question is never "how long will this take?" but "how
   much time is this artifact worth?" When the box runs out, cut scope
   ("scope hammering") rather than extending time.

## Two levels of box

Conflating these is the most common timeboxing failure.

**Appetite — artifact level, measured in sittings.** Before starting a PRD,
spec bundle, or study, decide how many sessions it's worth (e.g., "this bundle
gets three sittings"). Shape Up pairs this with a *circuit breaker*: when the
appetite is spent, the work does not get auto-renewed — it goes back for an
explicit re-decision. Multi-day efforts like a draft → audit → remediation
cycle are appetite-governed.

**Session box — sitting level, measured in minutes.** Scrum boxes every
ceremony the same way: a hard stop, known in advance, that ends the session
regardless of state. Cap deep-work sessions at ~90 minutes — the sustained-
concentration ceiling that shows up consistently in deliberate-practice
research (Ericsson) and aligns with ultradian rest-activity cycles. Batch
sessions on a maker's schedule (Graham): half-day blocks, no meetings inside.

## Default boxes by session type

Exit criteria are definitions of done, not effort spent. A box without an exit
criterion is just an alarm clock.

| Session type | Box | Exit criterion |
|---|---|---|
| PRD / spec first draft | 90 min | All sections present (Problem, Success criteria, Scope, Constraints, Plan, Open questions) — thin is fine, missing is not |
| Review remediation | 45 min | Every finding dispositioned: fixed, rejected-with-reason, or parked |
| Research spike | 25 min (one Pomodoro unit) | A written claim, or a written "dead end, because…" — never an unrecorded wander |
| Debugging | 30 min, then escalate | Root cause found, or a reproducible minimal case logged for next session. Timebox-then-escalate is standard SRE/support practice: the box limits solo thrash, not total effort |
| Decision session | 15 min per decision | A recorded decision with rationale, or a named blocker + who unblocks it. Use note-and-vote (GV Design Sprint) instead of open-ended discussion |

## Rules

1. **State the box before starting.** One line: goal + box length + exit
   criterion. If the goal won't fit in one line, the first 10 minutes are for
   scoping, not producing.
2. **One artifact per box** (Kanban WIP limit of 1). Discoveries mid-session
   go to a "next box" list, never into the current scope.
3. **Scope-hammer before you extend.** When time runs low, cut the artifact to
   what's done-able, per Shape Up. Extension is the last resort, not the
   default.
4. **One extension, max half the original box, with a written reason.** This
   is deliberately softer than Shape Up's hard circuit breaker — solo work has
   no team waiting on the stop — but a second overrun means the appetite was
   wrong: end the sitting and re-decide the artifact's budget.
5. **Park questions that belong to someone else.** Open questions for a
   requester or reviewer go into the artifact's Open Questions section; box
   time is never spent deciding what isn't yours to decide.

## Calibration loop

At every box end, append one row to
[`timebox_calibration.md`](timebox_calibration.md): *planned vs actual*, plus
whether the exit criterion was met. Periodically compare — if PRD drafts
consistently run 120 minutes against a 90-minute box, the box is wrong, not
the discipline. This is the reference class that defeats the planning fallacy;
it's the same mechanism as Scrum velocity, applied to sessions instead of
sprints.

Calibration rows deliberately do **not** go into `project_log.md`. The lab
logging standard scopes tracked logs to three entry triggers (decisions,
irreversible events, re-scopes) and rules out bare status — per-box telemetry
is a changelog by that standard's own terms, in any repo's log. The dedicated
file keeps the reference class queryable without fighting the log's cap or
its lint.

## When not to timebox

- **Sign-off conversations** — they end when the requester decides, not when
  a timer fires.
- **Live incident state** — debugging where stopping loses an unreproducible
  repro. Capture the state first; the box resumes after.
- **Flow on the critical path** — if a 90-minute box ends mid-flow on the
  session's stated goal (not a tangent), finishing the thought beats the
  timer. The box exists to kill drift, not momentum. Log the overrun so
  calibration sees it.

## Sources

- Basecamp, *Shape Up* — appetites, scope hammering, circuit breaker
- Schwaber & Sutherland, *The Scrum Guide* — timeboxed ceremonies, definition of done
- Cirillo, *The Pomodoro Technique* — 25-minute units, recorded interruptions
- Kahneman & Tversky — planning fallacy; reference-class forecasting
- Ericsson et al. — ~90-minute deliberate-practice session ceiling
- Graham, "Maker's Schedule, Manager's Schedule" — half-day deep-work batching
- Knapp et al., *Sprint* (GV) — note-and-vote, boxed decision-making
