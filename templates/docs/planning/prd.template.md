# <Project title> — PRD

<!-- This is a living document at a stable path. Update by amendment; never archive.
     Decisions are never embedded here: resolved ones live once in this bundle's spec.md
     (history in log.md); only decisions that outlive the slice route to project_log.md.
     See the blockquote below and .claude/rules/04-docs.md §ENG document standards.
     Bundle lifecycle for individual slices: _specs/<repo>/YYYY-MM-DD-<slug>/, retained in
     place and flipped to Status: complete — never moved or deleted
     (.claude/rules/04-docs.md §Bundle lifecycle). -->

**Status:** draft <!-- draft | active | paused | complete | superseded | abandoned -->
**Date:** YYYY-MM-DD · **Repo:** <repo>

> **The PRD stays decision-free.** No decision bodies here — the bundle's resolved decisions
> live once in `spec.md` (current design authority; history in `log.md`; bundles that omit
> `spec.md` record them in `log.md` alone); decisions that outlive the slice route to
> `project_log.md` (see `.claude/rules/03-logging.md` entry triggers). Link from the relevant
> Plan or Open Questions section if useful, never restate.

---

## Problem

<!-- What is broken or missing, for whom, and why it matters now.
     Audience test: a stakeholder unfamiliar with internal codenames can read this cold. -->

<problem statement>

---

## Success criteria

<!-- Measurable — not "better" or "improved." Each item falsifiable: you can tell in the future
     whether it was met. Avoid counts that restate ("3 modules") — describe observable behavior. -->

- <criterion: observable, measurable>
- <criterion>
- <criterion>

---

## Scope

### In scope

- <what this PRD covers>
- <what this PRD covers>

### Out of scope

<!-- Explicit out-of-scope prevents scope creep and sets expectations with stakeholders. -->

- <what is intentionally excluded>
- <what is intentionally excluded>

---

## Constraints

<!-- Budget, time, data access, infra, approvals, dependencies on other work. Be specific
     where precision matters ("$500/mo API cap") and intentionally vague where it doesn't. -->

- **Budget:** <spend ceiling and approval gate>
- **Timeline:** <hard deadlines, if any>
- **Data / access:** <gated datasets, licenses, credentials needed>
- **Infra:** <compute, hosting, service dependencies>
- **Approvals:** <sign-offs required before or during execution>
- **Dependencies:** <other in-flight work this blocks or is blocked by>

---

## Plan (phased)

<!-- High-level phases with their deliverables. Each non-trivial phase gets a planning bundle
     (_specs/<repo>/YYYY-MM-DD-<slug>/) with its own prd.md + spec.md + plan.md + log.md
     (chore/docs-only bundles omit spec.md). Link them here once created. This section updates
     by amendment as phases complete or scope shifts. -->

### Phase 1 — <phase name>

**Goal:** <one sentence>
**Deliverables:** <what ships>
**Bundle:** <!-- link once created, e.g. [plan](_specs/<repo>/YYYY-MM-DD-slug/plan.md) -->

### Phase 2 — <phase name>

**Goal:** <one sentence>
**Deliverables:** <what ships>
**Bundle:** <!-- link once created -->

<!-- Add phases as needed. -->

---

## Open questions

<!-- What the requester decides before implementation begins, AND known gaps in the slice — this
     section is the known-gaps surface (the retired standalone Design doc no longer carries one).
     Each item should have an owner and a due date or trigger ("before Phase 2 kicks off").
     Answered questions move to project_log.md as decisions; remove them from this list in the
     same amendment.
     Routing reminder so no surface is silently dropped:
       - known gaps / open decisions a reader must answer → here.
       - resolved decisions (current design authority) → this bundle's spec.md, stated once.
       - rejected alternatives / decisions-with-rationale → this bundle's log.md (Decisions). -->

- [ ] **<question or known gap>** — owner: <name> · due: <date or trigger>
- [ ] **<question or known gap>** — owner: <name> · due: <date or trigger>
