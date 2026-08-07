# <Slice title> — spec (design authority)

<!-- The bundle's "what is still true" surface: what was decided, stated once, current-state.
     spec.md is to log.md what the Standing Decisions index is to the project log — the
     current-state surface vs. the history. A decision lives HERE once; log.md records how it
     got here (evidence, weighed alternatives, provenance); prd.md and plan.md link, never
     restate. Genuinely-open items stay in prd.md §Open questions.
     Chore/docs-only bundles (log-overflow archives, doc syncs) omit this file entirely.
     Spec: .claude/rules/04-docs.md §ENG document standards. -->

**Date:** YYYY-MM-DD · **Repo:** <repo>
**PRD:** [prd.md](./prd.md) (bundle lifecycle `Status` lives there — never here) ·
**Plan:** [plan.md](./plan.md) · **Log:** [log.md](./log.md)

**Legend.** **DECIDED** — resolved and binding for this bundle. **RECOMMENDED** — proposed,
pending sign-off. **PARKED** — deferred or owned elsewhere (name the owner).

---

## Decision summary

<!-- One row per resolved (or parked) question. Status links to the per-decision section below
     or names the external owner. This table is the review surface — keep resolutions one line. -->

| # | Question | Resolution | Status |
|---|----------|------------|--------|
| D1 | <question> | <one-line resolution> | **DECIDED** (§D1) |
| D2 | <question> | <one-line resolution> | **RECOMMENDED** (§D2) |
| D3 | <question> | <deferred — owner: <bundle or repo>> | **PARKED** ([prd.md §Open questions](./prd.md)) |

---

## Decisions

<!-- One subsection per non-trivial decision: the binding resolution and its contract impact.
     NOT the rationale/alternatives — those live in log.md; link the entry. -->

### D1: <decision title>

<binding resolution — what is now true>

**Contract impact:** <interfaces, schemas, file shapes, or behaviors this pins>

**History:** [log.md](./log.md) <entry ref>

---

## Contracts

<!-- Finalized contracts this bundle pins (schemas, interfaces, formats) — or link to where
     each is single-sourced. Delete the section if no contracts were finalized. -->

---

## Sign-off

<!-- RECOMMENDED items promote to DECIDED here. One checkbox per pending item, with owner. -->

- [ ] <RECOMMENDED item> — owner: <name>
