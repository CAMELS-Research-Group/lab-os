# lab-os — main plan (execution surface)

**Scope:** lab-os · **Living** — execution map, not a task list.
**PRD:** [prd.md](./prd.md) · **Spec:** [spec.md](./spec.md) ·
**Design:** [design.md](./design.md) · **Log:** [log.md](./log.md)

---

## In flight

Any dated directory beside this bundle is an in-flight slice, by definition — completed
slices are folded and deleted, so the directory listing *is* the in-flight list. Each slice's
own `plan.md` carries its tasks; its PRD `Status:` is the state marker.

Currently: `2026-07-31-timeboxing`. Its PRD reads `Status: active — in review, PR #66`, and
#66 has merged — see [prd.md](./prd.md) §Open questions. If its owner declares it done it
becomes the first bundle to fold into this one.

## Queued

[`BACKLOG.md`](../../../BACKLOG.md) is the queue for cross-repo and lab-level work; read its
Index first. Repo-scoped work stays in that repo's issues. The Index is generated — edit the
Item blocks and run `python3 scripts/backlog_lint.py --write-index`.

## Blocked on the operator

- **Doc-sync of `README.md`** to the post-D16/D17 world (shared dev home, Caravan as staging
  surface). Mechanical once someone decides how much of the fork-as-dev-home onboarding path
  survives — that is a call about how members are onboarded, not a docs edit.
- **A decision on the `06-timeboxing` vendoring row** in Caravan's `rules_sync.py` manifest:
  either member repos receive the rule or its scope is stated as lab-os-only.
- **A decision on the pre-Caravan bundles** — whether and how they enter
  [spec.md](./spec.md)'s register ([prd.md](./prd.md) §Open questions).
