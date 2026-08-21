# <Project / scope name> — main log (fold journal)

<!-- One entry per fold (or restructure of this bundle): what folded in, from where, and the
     recovery path. The map from deleted bundles to their history. Chronological, oldest
     first, bottom-insert. Living — never dated, never deleted, exempt from byte budgets and
     the entry cap. Project-altitude decision history stays in project_log.md.
     Contract: .claude/rules/04-docs.md §Bundle lifecycle & the main bundle. -->

**Scope:** <scope> · **Living** — fold journal.
**PRD:** [prd.md](./prd.md) · **Spec:** [spec.md](./spec.md) ·
**Design:** [design.md](./design.md) · **Plan:** [plan.md](./plan.md)

---

## YYYY-MM-DD HH:MM — Fold: <bundle handle> (<n> decisions)

**Bundle:** `_specs/<scope>/<DATE>-<handle>/` — <one line on what the slice was>.
**Folded:** <what went where — decisions into the register, shape into design.md, scope
changes into prd.md>.
**Deleted in:** #<PR>. **Recover:** `git log --all -- '_specs/<scope>/<DATE>-<handle>/'`
