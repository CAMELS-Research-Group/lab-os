# Timebox calibration record

Append-only reference class for [`timeboxing.md`](timeboxing.md)'s calibration
loop. One row per box end, newest last. This is telemetry, not a decision log —
it deliberately lives outside `project_log.md` (see the calibration-loop section
of the timeboxing doc for why).

Columns: **Planned** and **Actual** in minutes; **Exit met?** is yes/no against
the box's stated exit criterion, judged at the timer, not after any extension;
**Note** is optional (scope hammered, extended +N, interrupted, …).

Rows are append-only and never rewritten once committed, so two exclusion
markers carry the corrections. A Note containing `non-conforming` or
`superseded by <date> <planned>/<actual> row` means **excluded from the
reference class** — skip it when computing defaults. A superseding row names
the row it replaces by that same `<date> <planned>/<actual>` key, which is
unique per row; "the <date> row" is not a usable reference once a date
carries more than one.

Session types prefixed `agent-` are agent-executed task boxes — see
[`timebox_recording.md`](timebox_recording.md) for their defaults, recording
mechanics, and the review-override convention. Agent boxes log to the
calibration file of the repo the task belongs to; rows here are lab-os work,
plus rows that fell through from repos that have not adopted the practice
(no calibration file of their own — see that guide's § Logging rules).

| Date | Session type | Artifact | Planned | Actual | Exit met? | Note |
|---|---|---|---|---|---|---|
| 2026-07-31 | agent-dev | timeboxing skill + recording guide (PRD, timebox_recording.md, SKILL.md) | 45 | 31 | yes | first row; homed in lab-os mid-box (was marketplace plugin); helper script cut to v2 |
| 2026-07-31 | agent-dev | review remediation + fork PRs Aryaa-K/lab-os#1/#3 (per-repo rows, expiry handoff, log entries) | 20 | 18 | yes | paused ~mid-box for approval gate; wait excluded; pre-pause portion estimated at 15 (row predates the pause-timestamp rule) |
| 2026-07-31 | agent-dev | upstream review remediation for #66 (13 findings dispositioned) | 30 | 4 | yes | single agent turn; timestamps 16:54:58-16:58:32; remediation box — weigh separately from feature work in defaults reviews |
| 2026-07-31 | agent-dev | round-2 remediation for #66 (blocker + 3 important + 2 suggestions + intake sync) | 20 | 6 | yes | state file skipped; duration reconstructed from reflog and command log, ±2 min; non-conforming — excluded from the reference class |
| 2026-08-01 | agent-docs | round-3 remediation for #66 (5 importants + 3 suggestions + 3 open-question answers) | 15 | 4 | yes | self-proposed criterion; superseded by 2026-08-01 15/7 row — excluded from the reference class (Actual here was written at commit time, before the PR-surface edits the box also covered) |
| 2026-08-01 | agent-docs | round-3 remediation for #66 — corrected total | 15 | 7 | yes | self-proposed criterion; supersedes the 2026-08-01 15/4 row — same box end, Actual restated to cover the PR-surface edits (title/body/verifier evidence); remediation box — weigh separately from feature work in defaults reviews |
| 2026-08-07 | agent-docs | round-4 remediation for #66 (blocker + 5 important + 3 post-#59 items across 9 files) | 15 | 7 | yes | self-proposed criterion; timestamps 10:47:16-10:52:39 measured, +~1.5 to cover commit and push through box end; three items left as maintainer flags rather than guessed (PR-body edit, companion issue, PRD homing) — see handoff |
| 2026-08-07 | agent-docs | round-5 for #66 — PRD homing per Watson Sug 4 (PRD.template.md) | 15 | 4 | yes | self-proposed criterion; timestamps 11:15:32-11:17:32 measured, +~2 for commit/push/body; scope cut mid-box on requester correction — an in-flight rewrite of round-4 wording to Watson's phrasings was reverted when the instruction narrowed to item 3 only |
| 2026-08-18 | agent-docs | LSCA dw bundle: wayfinder map graduation (13 tickets folded to log.md, map deleted) + Caravan SDLC ticket 10 | 30 | 8 | yes | self-proposed criterion; box atypical for docs (six steps, two repos) so planned 30 not the 15 default; two artifacts in two repos — a one-artifact-per-box deviation, left in the reference class; no scope hammer, no extension |
