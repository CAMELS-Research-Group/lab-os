# Timeboxing

All working sessions and agent-executed tasks. Owning docs: `docs/timeboxing.md` (session standard v1.0), `docs/timebox_recording.md` (agent task boxes + recording mechanics). At-a-glance: `docs/timeboxing_quickref.md`. Paths are lab-os-root-relative; if this rule is read as a distributed copy outside the repo, resolve them against a lab-os checkout (same convention as the skill's header). This rule is a pointer — the docs own the details; do not restate their tables here.

## Session boxes

- State the box before starting: one line — goal + box length + exit criterion. Defaults per session type: `docs/timeboxing.md` § Default boxes. A box without an exit criterion is not a box.
- One artifact per box; mid-box discoveries go to a next-box list, never into scope.
- Scope-hammer before extending. One extension max, half the original box, written reason. Humans may take the extension when the timer fires; agent boxes may extend only **before** expiry — an agent box found already expired goes straight to handoff (the asymmetry is deliberate: agent expiry is usually discovered late, at a checkpoint seam).
- At box end: append one planned-vs-actual row to the owning calibration file. **Exit met?** is judged at the timer, before any extension. Rows never go to `project_log.md` (telemetry, not a log entry — `03-logging.md` entry triggers).

## Agent task boxes

- The `timeboxing` skill (`.claude/skills/timeboxing/SKILL.md`) is the protocol: timestamped state file with recorded pauses, elapsed-time checkpoints, ≥80% scope-hammer, expiry → notify user + handoff (flow carve-out on the stated goal only), auto-appended calibration row, exit-met judged by the agent with user override at review.
- Row target: the calibration file of the repo the task belongs to **where that repo has adopted the practice** (its calibration file exists, or `TIMEBOX_CALIBRATION_FILE` is set); otherwise lab-os's `docs/timebox_calibration.md`. Agents never create a repo's calibration file — that is a human adoption act.

## Calibration loop

Every few weeks, compare planned vs actual per session type; a consistently blown box means the default is wrong — update `docs/timeboxing.md` and the quickref table together.
