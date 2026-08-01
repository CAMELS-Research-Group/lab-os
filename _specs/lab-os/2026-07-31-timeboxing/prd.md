# PRD — timeboxing skill + recording guide (lab-os)

**Status:** active — in review, PR #66. Decision history: `project_log.md`.
**Deliverables:** `docs/timebox_recording.md` (guide) · `.claude/skills/timeboxing/SKILL.md` (skill) · `.claude/rules/06-timeboxing.md` (rules surface).

## Problem

The lab timeboxing standard (`docs/timeboxing.md`, v1.0) defines a calibration loop — state box + exit criterion at start, append a planned-vs-actual row at box end — but it depends entirely on manual discipline. In the originating fork (Aryaa-K/lab-os), the calibration file sat at zero rows from the standard's adoption (2026-07-24) until this skill's first dogfood run (2026-07-31): the loop never closed on discipline alone.

The lab's dev workflow is: user + agent brainstorm together → context ironed out → **the agent does the development work** → the user reviews and gives the hard pass. The standard's boxes target human sittings; nothing boxes or instruments the agent's autonomous dev phase — the phase where drift, gold-plating, and unbounded thrash now actually happen.

## Success criteria

1. **Loop closes:** every boxed agent task produces exactly one calibration row (planned, actual, exit-met, note), appended at box end without manual bookkeeping.
2. **Honest telemetry:** actual time is computed from a recorded start timestamp (state file + date arithmetic), never from agent recall.
3. **Box stated before work:** the agent states goal + box + exit criterion before its first development action, and refuses to start a boxed task without an exit criterion.
4. **Scope-hammer over overrun:** near box exhaustion the agent cuts scope to what's done-able and parks the rest on a next-box list; at most one extension, half the original box, with a written reason recorded in the row's note.
5. **Review gate untouched:** the box ends at handoff-to-review; the handoff report shows planned-vs-actual, exit-met judgment with evidence, and what was cut. Review time is never inside the box.
6. **Exit-met is agent-judged, user-overridable:** the agent records yes/no against the stated criterion; the user can override at review and the row is corrected (in place before the row is committed; by appended correction row after).

## Scope

**In:** the guide (recording/logging model for agent-task boxes, row format, target-file rule, override flow, agent-task default boxes) · the skill (the protocol the agent follows: intake, start, checkpoints, expiry notification, scope-hammer, extension, end, handoff) · **target-file rule (decided at review; adoption-gated per upstream review):** rows go to the calibration file of the repo the task belongs to where that file already exists — creating it is a human adoption act, never the agent's — else fall back to lab-os's; `TIMEBOX_CALIBRATION_FILE` env var overrides.

**Out (v1):** boxing the joint brainstorm phase (human standard governs) · hook-based mid-turn enforcement · a helper script (skill uses direct shell date arithmetic; script is v2 if mechanics prove error-prone) · appetite/multi-sitting tracking · marketplace packaging (superseded by lab-os homing; portable design retained so porting stays cheap) · any writes to `project_log.md`.

## Constraints

- Row format must match `docs/timebox_calibration.md` columns exactly: Date · Session type · Artifact · Planned · Actual · Exit met? · Note.
- Calibration rows never go to `project_log.md` (lab logging standard, `03-logging.md`).
- Skill state must not pollute repos: box state lives in the session scratchpad, not the working tree.
- ~$0 cost; no network dependencies.

## Plan

P1 clone/branch + file PRD (done) → P2 guide draft → P3 skill draft → checkpoint: user reviews both → P4 dogfood on one real task (its calibration row is acceptance evidence) → P5 commit/PR per lab PR lifecycle.

## Open questions

1. **Skill distribution:** `.claude/skills/` in lab-os auto-loads only for lab-os sessions. For the skill to fire in LSCA and other repos: user-level `~/.claude/skills/` copy, per-repo mirror (like the rules pattern), or invoke-by-path. Requester decides at review.
2. **Agent-task default boxes** (guide proposes: dev 30 · docs 15 · debug 20 · spike 15) are seeded, not calibrated — first ~10 rows should trigger a defaults review.
