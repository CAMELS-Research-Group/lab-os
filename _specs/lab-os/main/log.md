# lab-os — main log (fold journal)

**Scope:** lab-os · **Living** — fold journal.
**PRD:** [prd.md](./prd.md) · **Spec:** [spec.md](./spec.md) ·
**Design:** [design.md](./design.md) · **Plan:** [plan.md](./plan.md)

The map from deleted bundles to their history: each entry records what folded in, from where,
and the recovery path. Project-altitude decision history stays in `project_log.md`.

---

## 2026-08-13 11:40 — Main bundle created

**Decision:** `_specs/lab-os/main/` seeded from `templates/docs/main-bundle/`. No fold: nothing
in `_specs/lab-os/` carries a terminal `Status:` yet, so the register starts empty and
[design.md](./design.md) is written from the repository as it stands rather than assembled
from folded slices.
**Why:** the bundle lifecycle that makes a main bundle coherent landed first — creating
a fold target while the rules forbade folding would have moved the conflict rather than
resolved it. Rationale and supersession: `project_log.md` entry 2026-08-13 10:55.
**Refs:** #81 (the lifecycle rule), #83 (this bundle)
