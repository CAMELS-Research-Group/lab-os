# lab-os — main plan (execution surface)

**Scope:** lab-os · **Living** — execution map, not a task list.
**PRD:** [prd.md](./prd.md) · **Spec:** [spec.md](./spec.md) ·
**Design:** [design.md](./design.md) · **Log:** [log.md](./log.md)

---

## In flight

Any dated directory beside this bundle is an in-flight slice, by definition — completed
slices are folded and deleted, so the directory listing *is* the in-flight list. A slice's own
`plan.md`, where it carries one, holds its tasks; its PRD `Status:` is the state marker in
every case.

Currently: `2026-07-31-timeboxing`. Its PRD reads `Status: active — in review, PR #66`, and
#66 has merged — see [prd.md](./prd.md) §Open questions. If its owner declares it done it
becomes the first bundle to fold into this one.

## Queued

[`BACKLOG.md`](../../../BACKLOG.md) is the queue for cross-repo and lab-level work; read its
Index first. Repo-scoped work stays in that repo's issues. **Two surfaces are generated from
the Item blocks, and both are gated** — edit the blocks, then run
`python3 scripts/backlog_lint.py --write-index` for the in-file Index *and*
`python3 scripts/backlog_view.py --write` for
[`docs/backlog-views.md`](../../../docs/backlog-views.md). `standards.yml` runs
`backlog-views` at `enforce: true`, so regenerating only the Index leaves the dashboard stale
and the PR red.

## Blocked on the operator

- **Doc-sync of `README.md`** — gap stated in [design.md](./design.md) §Known gaps. What is
  blocked here is the call on how much of the fork-as-dev-home onboarding path survives; that
  is a decision about how members are onboarded, not a docs edit.
- **A decision on the pre-Caravan bundles** — whether and how they enter
  [spec.md](./spec.md)'s register ([prd.md](./prd.md) §Open questions).
