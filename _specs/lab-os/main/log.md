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

## 2026-08-18 11:49 — Register admits the bundle's own decisions

**Decision:** [spec.md](./spec.md) §What belongs here now admits *self rows* — decisions this
main bundle makes about itself, carrying `_specs/lab-os/main/` as their source — and the
register's first row records that lab-os keeps `_specs/lab-os/main/` rather than the
member-repo `_specs/main/`. [prd.md](./prd.md) §Open questions keeps the question and links to
that row instead of restating the resolution. Separately, the `06-timeboxing` vendoring gap in
[design.md](./design.md) now tracks Caravan `BACKLOG.md` B22 and leaves
[plan.md](./plan.md) §Blocked on the operator, and the rules-tier `docs-budget` WARN tracks
lab-os `BACKLOG.md` B20.
**Why:** `04-docs.md` §ENG forbids decision bodies in a PRD, and a fold-only register left the
resolution with no owning surface — a main bundle has no dated bundle to fold from.
**Alternatives:** route the decision to `project_log.md` (rejected — it sits at 1.50x of its
budget with `docs-budget` enforcing, so one more entry trips FAIL); leave the body in the PRD
(rejected — the rule).
**Refs:** #83
