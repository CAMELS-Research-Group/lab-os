# Timeboxing standard + agent task-box skill — spec log

Bundle spec-log altitude. Flat chronological stream, oldest-first, bottom-insert.
Entries lifted from the #66 review rounds, which are not durable once the PR squashes.

---

## 2026-07-31 16:55 — Home the standard in lab-os, not a marketplace plugin

**Decision:** The standard and its agent extension live in lab-os (`docs/`, `.claude/rules/`, `.claude/skills/`) rather than shipping as a marketplace plugin.
**Why:** Lab conventions live where the lab's other conventions live; a plugin is portable but puts a lab rule outside the repo that owns rules.
**Alternatives:** Marketplace-plugin homing — re-scoped mid-box, portability did not outweigh convention-locality.

---

## 2026-08-04 20:37 — Rows live beside the work they measure, never in project_log.md

**Decision:** Calibration rows append to the calibration file of the repo the task belongs to; a cross-repo roll-up is a grep, not a central file. Rows never go to any `project_log.md`.
**Why:** Telemetry is not a log entry — the logging standard's entry triggers exclude bare status at every altitude. The reference class is most useful next to the work it describes.
**Alternatives:** Central-only row logging — rejected: one file accumulating every repo's telemetry loses the locality that makes a row interpretable.

---

## 2026-08-06 16:23 — Adoption is gated on the target repo, and agents never seed it

**Decision:** An agent appends rows only where the task repo has already adopted the practice — `docs/timebox_calibration.md` exists at that exact path — otherwise it falls back to lab-os. Creating that file is a human act.
**Why:** The skill deploys user-scope and fires in every session on a linked machine, so without a gate the convention would spread into repos that never chose it, by side effect.
**Alternatives:** Probe a second location (repo root) as well — rejected: file existence is the adoption test, so a second probe makes first rows scatter across per-repo variants.

---

## 2026-08-07 13:08 — The env var is constrained, not absolute

**Decision:** `TIMEBOX_CALIBRATION_FILE` is consulted first but cannot defeat the never-`project_log.md` prohibition: a value whose resolved filename is `project_log.md` or `project_log_archive.md` in any case, or which resolves outside the task repo or lab-os, is rejected and falls through to the adoption probe, with the rejection surfaced in the handoff.
**Why:** `.claude/skills/**` is a security boundary and this skill deploys user-scope into repos that never adopted the convention; an env var that silently redirected appends would be a write primitive pointed at the audit trail. Matching the resolved path rather than the literal is what makes the check hold — `../project_log.md` and `Project_Log.md` reach the same file.
**Alternatives:** Reject straight to lab-os's file — rejected: falling through to the normal resolution order makes a rejected env var behave exactly like an unset one, so an adopting repo's own table still wins.

---

## 2026-08-07 13:08 — Correction rows supersede by a compound key, not by date

**Decision:** A correction row names the row it replaces as `supersedes the <date> <planned>/<actual> row`, with the reciprocal `superseded by …` marker on the row replaced; a superseded row is excluded from aggregation so one box end contributes one row to any average. The convention applies to any post-commit amendment, whoever raised it.
**Why:** Bare dates stopped being unique the first time a date carried two rows, and the append-only rule means the ambiguity cannot be edited away. Excluding from aggregation is the half that matters: without it a corrected box is double-counted in the very numbers the loop computes.

---

## 2026-08-07 13:08 — Deleting a count beats incrementing it

**Decision:** `site/docs/rules-explained.md` defers the rule count to the rules directory rather than stating a number.
**Why:** The page went stale by asserting "four"; incrementing it to "five" would have re-staled on the next rule. Removing the class beats fixing the instance.
**Alternatives:** Update the number as prescribed — rejected in favour of the durable shape, with the reviewer's agreement after the fact.

---

## 2026-08-07 18:12 — Retired scaffolds are historical, not maintained

**Decision:** `templates/PRD.template.md`'s `docs/work/...` paths stay as they are; the round-5 change repointing them at `_specs/` was reverted.
**Why:** #58 marks that file a superseded scaffold whose paths are historical, and names `templates/docs/planning/` the sole PRD home. Rewriting a retired file's paths to the current layout makes it read as maintained and gives a newcomer a second plausible home to copy from.
**Alternatives:** Keep the repoint and argue for an exception — rejected: the earlier suggestion to fix the path predates the four-file bundle landing, so the two answer the same question at different times and the later one wins.
**Refs:** #58

---

## 2026-08-07 21:22 — The bundle carries log.md only, for now

**Decision:** This bundle ships `prd.md` + `log.md`, without `spec.md` or `plan.md`.
**Why:** `log.md` is required because these decisions exist only in the PR body, which does not survive a squash. The other two are ceremony for a docs-only slice, and lab-os is migrating to a new repo — bundle-shape precedent set now would be re-set there.
**Alternatives:** Full four-file bundle — deferred rather than rejected; revisit after the migration.

---

## 2026-08-07 21:22 — Bundle path deferred pending the lab-os migration

**Decision:** The `_specs/lab-os/<DATE>-<handle>/` path stands as-is; whether upstream lab-os counts as workspace-root altitude (keeping the `<repo>` segment) or as a member repo (dropping it) is not settled here.
**Why:** The answer sets the precedent for every future lab-os bundle, and lab-os is migrating to a new repo — deciding now would bind a layout that the migration re-opens.
