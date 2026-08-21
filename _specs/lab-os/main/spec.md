# lab-os — main spec (decision register)

**Scope:** lab-os · **Living** — the fold target for completing bundles' decisions.
**PRD:** [prd.md](./prd.md) · **Design:** [design.md](./design.md) ·
**Plan:** [plan.md](./plan.md) · **Log:** [log.md](./log.md)

Recover any folded bundle: `git log --all -- '_specs/lab-os/<bundle>/'`.

---

## What belongs here

Two kinds of row. **Folded rows** are bundle-altitude decisions, appended one per decision as
each bundle folds. **Self rows** are decisions this main bundle makes about itself — its path,
its file set, the shape of this register — which arrive with no dated bundle to fold from and
would otherwise have no owning surface; their `Source` is `_specs/lab-os/main/`. A decision
meeting a project-altitude trigger (`03-logging.md` §Entry triggers) has its full entry in
[`project_log.md`](../../../project_log.md) — this register links to it by PR and never
restates the body. The `## Standing Decisions` index in that file remains the
project-altitude "what is still true" surface; this table is its bundle-altitude counterpart.

No bundle has folded yet, so every row below is a self row. That is the expected state for a
scope whose decisions have so far been project-altitude, and it is not a gap to be backfilled:
see [prd.md](./prd.md) §Open questions on whether the pre-Caravan dev home's terminal bundles
enter this register at all.

## Decision register

| Date | Decision | Source | PR |
|---|---|---|---|
| 2026-08-18 | lab-os's main bundle stays at `_specs/lab-os/main/` — the workspace-root path shape — rather than the member-repo `_specs/main/`; one path convention per repo, matching the sibling `2026-07-31-timeboxing` bundle. Reopened only by a ruling or a `03-logging.md` change that settles lab-os's altitude. | `_specs/lab-os/main/` | [#83](https://github.com/CAMELS-Research-Group/lab-os/pull/83) |
