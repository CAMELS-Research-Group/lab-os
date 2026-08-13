# <Project / scope name> — main plan (execution surface)

<!-- The map of what is in flight and what is queued. Living — never dated, never deleted,
     exempt from byte budgets. Carries NO task bodies: tasks live in the dated slice bundles
     and the backlog; this file just says where to look, kept self-maintaining.
     Contract: .claude/rules/04-docs.md §Bundle lifecycle & the main bundle. -->

**Scope:** <scope> · **Living** — execution map, not a task list.
**PRD:** [prd.md](./prd.md) · **Spec:** [spec.md](./spec.md) ·
**Design:** [design.md](./design.md) · **Log:** [log.md](./log.md)

---

## In flight

Any dated directory beside this bundle is an in-flight slice, by definition — completed
slices are folded and deleted, so the directory listing *is* the in-flight list. Each slice's
own `plan.md` carries its tasks; its PRD `Status:` is the state marker.

## Queued

<!-- Pointer to the queue (BACKLOG.md / issues), not a restatement of it. -->

## Blocked on the operator

<!-- Standing actions only a human can take (org settings, credentials, approvals), each
     removed when done. -->
