# Timebox calibration record

Append-only reference class for [`timeboxing.md`](timeboxing.md)'s calibration
loop. One row per box end, newest last. This is telemetry, not a decision log —
it deliberately lives outside `project_log.md` (see the calibration-loop section
of the timeboxing doc for why).

Columns: **Planned** and **Actual** in minutes; **Exit met?** is yes/no against
the box's stated exit criterion, judged at the timer, not after any extension;
**Note** is optional (scope hammered, extended +N, interrupted, …).

Session types prefixed `agent-` are agent-executed task boxes — see
[`timebox_recording.md`](timebox_recording.md) for their defaults, recording
mechanics, and the review-override convention. Agent boxes log to the
calibration file of the repo the task belongs to; rows here are lab-os work.

| Date | Session type | Artifact | Planned | Actual | Exit met? | Note |
|---|---|---|---|---|---|---|
| 2026-07-31 | agent-dev | timeboxing skill + recording guide (PRD, timebox_recording.md, SKILL.md) | 45 | 31 | yes | first row; homed in lab-os mid-box (was marketplace plugin); helper script cut to v2 |
| 2026-07-31 | agent-dev | review remediation + fork PRs Aryaa-K/lab-os#1/#3 (per-repo rows, expiry handoff, log entries) | 20 | 18 | yes | paused ~mid-box for approval gate; wait excluded; pre-pause portion estimated at 15 (row predates the pause-timestamp rule) |
| 2026-07-31 | agent-dev | upstream review remediation for #66 (13 findings dispositioned) | 30 | 4 | yes | single agent turn; timestamps 16:54:58-16:58:32; remediation box — weigh separately from feature work in defaults reviews |
| 2026-07-31 | agent-dev | round-2 remediation for #66 (blocker + 3 important + 2 suggestions + intake sync) | 20 | 6 | yes | state file skipped; duration reconstructed from reflog and command log, ±2 min; non-conforming — excluded from the reference class |
| 2026-08-01 | agent-docs | round-3 remediation for #66 (5 importants + 3 suggestions + 3 open-question answers) | 15 | 4 | yes | self-proposed criterion; remediation box — weigh separately from feature work in defaults reviews |
| 2026-08-01 | agent-docs | round-3 remediation for #66 — correction | 15 | 7 | yes | corrects 2026-08-01 row: Actual there was written at commit time, before the PR-surface edits (title/body/verifier evidence) the box also covered |
