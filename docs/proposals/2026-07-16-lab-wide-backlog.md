# Convention Proposal — Lab-Wide Backlog

**Date:** 2026-07-16 · **Owner:** Kiara · **Mandate:** 2026-07-08 retrospective (Try: "Lab
Wide Backlog", assigned Kiara; Drop: "Watson as Backlog → use team backlog")
**Status:** proposed — ratify by PR review on this repo

---

## Problem context

Open lab work has no shared home. Cross-repo items — convention exercises, workshop
iteration, tooling asks, infra questions — currently route through one maintainer's
personal workspace file, which the 2026-07-08 retrospective explicitly voted to drop as
the mechanism ("Watson as Backlog"). The costs named in the retro: work is invisible until
the maintainer surfaces it, nobody else can groom or claim items, and the two most-wanted
gaps ("Conventions / Runbooks", "Tooling for Agents / Power Users" — each appearing in two
retro quadrants) have nowhere to accumulate as actionable items.

## Existing solutions / conventions

| Mechanism | Where it stands today |
|---|---|
| Maintainer's personal workspace backlog | Works for one person; single-owner bottleneck; not visible or editable by the team — the retro's Drop item |
| Per-repo GitHub issues | Already the lab convention for repo-scoped follow-ups (logging rules route "open work → GitHub issues"); scattered across repos, no lab-wide view, cross-repo items have no natural repo |
| Planning-round spec bundles (`_specs/` on the LSCA collection PR) | Scoped to one subsystem planning round; a bundle's Open Questions is not a standing queue |
| GitHub Projects (org board) | Cross-repo views for free, but new tooling surface, org-admin setup, and item content lives outside the repos the lab greps and versions |
| **`BACKLOG.md` in lab-os** (this proposal) | The format the lab already *teaches*: the building-workshop answer key ships a worked `BACKLOG.md` + `backlog-item.template.md`. This proposal promotes that taught format to one live instance |

## Tradeoffs

**Chosen (lab-os `BACKLOG.md`):**
- **Costs:** one file can conflict under concurrent edits (mitigated: index-first structure,
  one item per block, additive edits); no per-item discussion threads (mitigated: an item
  that needs discussion links a GitHub issue and keeps only the pointer); manual grooming
  (mitigated: sprint-boundary cadence below). lab-os is public — items must be
  public-tier safe: no gated-dataset details, no private-repo paths, no personal data.
- **Buys:** versioned, greppable, PR-reviewed like every other lab convention; zero new
  tooling; the format is already taught in the workshop, so onboarding cost is nil; the
  Drop item is retired the day this merges.
- **Forecloses:** board-style automation (assignee syncing, status columns). If the lab
  outgrows the file, migrating to GitHub Projects later is an export, not a rewrite — the
  item template's fields map 1:1 to board fields.

**Rejected alternatives:** GitHub Projects now — highest ceremony for a five-person lab and
splits content away from the repo; issues-only — leaves cross-repo work homeless and
provides no single "what's ready" surface; status quo — voted down by the retro.

## Recommendations

1. **`BACKLOG.md` at the lab-os repo root** (added in this PR), following the workshop
   answer-key shape: **Index** (the "what's ready right now" table) → **Inbox** (raw,
   unshaped) → **Items** (shaped blocks per `templates/backlog-item.template.md`, whose
   "Done when" field is the readiness bar). The Index is **generated** — edit the Item
   blocks, then run `python3 scripts/backlog_lint.py --write-index`; never hand-edit it.
2. **Routing rule** (extends, does not change, the logging conventions): repo-scoped work
   stays in that repo's issues; **cross-repo / lab-level work goes here**. An item that
   grows a discussion gets a linked issue; an item that grows into a project gets a spec
   bundle and its backlog entry points at it.
3. **Grooming cadence:** at each 2-week sprint boundary (the retro's Keep), the team walks
   the Index — promote shaped Inbox entries, re-check "Done when" on in-progress items,
   mark shipped ones done. Any team member may add to Inbox at any time; shaping an item
   into Items requires an owner.
4. **Migration:** lab-relevant items in the maintainer's personal backlog move here at the
   owner's discretion within one sprint; the personal file remains for personal work.
5. **Template home:** `templates/backlog-item.template.md` (copied in this PR from the
   workshop answer key — single source going forward; the answer-key copy stays as
   teaching material).
