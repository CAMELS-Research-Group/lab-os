# lab-os — main spec (decision register)

**Scope:** lab-os · **Living** — the fold target for completing bundles' decisions.
**PRD:** [prd.md](./prd.md) · **Design:** [design.md](./design.md) ·
**Plan:** [plan.md](./plan.md) · **Log:** [log.md](./log.md)

Recover any folded bundle: `git log --all -- '_specs/lab-os/<bundle>/'`.

---

## What belongs here

Bundle-altitude decisions, appended one row per decision as each bundle folds. A decision
meeting a project-altitude trigger (`03-logging.md` §Entry triggers) has its full entry in
[`project_log.md`](../../../project_log.md) — this register links to it by PR and never
restates the body. The `## Standing Decisions` index in that file remains the
project-altitude "what is still true" surface; this table is its bundle-altitude counterpart.

The register is empty until the first bundle folds. That is the expected state for a scope
whose decisions have so far been project-altitude, and it is not a gap to be backfilled: see
[prd.md](./prd.md) §Open questions on whether the pre-Caravan dev home's terminal bundles
enter this register at all.

## Decision register

| Date | Decision | Source | PR |
|---|---|---|---|
