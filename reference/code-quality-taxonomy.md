# Code-quality finding taxonomy

**Single owning source** of the finding-class catalog. Do not duplicate class definitions elsewhere.

**Derivers** — **fork-resident unless marked otherwise**: they live in the lab's workspace fork
(github.com/WatsonWBlair/Agentic_Workspace), their paths below are fork-relative, and a path that
does not resolve in an upstream lab-os clone is expected rather than broken:

- `standards-aware-planning` skill (`.claude/skills/standards-aware-planning/`) — reads the
  design-constraint forms; emits plan-time checks for classes at altitude `plan`
- pr-review-loop code-quality rubric (`.claude/skills/pr-review-loop/reference/code-quality-rubric.md`)
  — reads the reviewer-finding forms; applies all seven classes on code-touching PRs
- `execution-writing-disposition` skill (drafted — `_specs/lab-os/2026-07-02-execution-writing-disposition/`;
  home `.claude/skills/execution-writing-disposition/` when built) — reads the trigger predicates
  and design-constraint forms; applies writing-disposition checks for classes at altitude
  `execution` while code is being written
- `slop-review` skill (`.claude/skills/slop-review/`) — whole-repo / accumulated lens; derives
  the class catalog by reference alongside its own prose-slop categories (standalone +
  cross-linked per that bundle's PRD OQ3, resolved 2026-07-03; the class sweep under this lens
  is a follow-on scope amendment)
- `pr-round` skill (`.claude/skills/pr-round/`) — **resident in this repo**; reads this file
  **in place** as tier 2 of its
  three-tier rubric (`reference/rubric-layering.md`), applying the lab thresholds — notably the
  1000-line Class 1 budget — over its shipped tier-1 defaults. **Documented exception to the
  no-duplication rule above:** its tier-1 file restates the seven classes in *project-neutral* form
  so the skill still reviews competently in a repo that has no access to this one. That restatement
  is deliberately generic and threshold-free; **this file stays authoritative** wherever it resolves,
  and drift between the two is expected and accepted rather than reconciled.

**Vocabulary source:** the `codebase-design` skill — fork-resident
(`.claude/skills/codebase-design/SKILL.md` in the workspace fork).
Terms used below — module, interface, depth, seam, adapter, leverage, locality, the deletion test,
"the interface is the test surface" — are defined there; this file applies them.

**Scoping is the consumer's lens:** trigger predicates and design-constraint forms carry no scope;
each Reviewer form renders its consumer's lens (the diff lens of `pr-review-loop`). Each deriver
applies its own: `standards-aware-planning` = prospective/plan-time (the planned change surface;
no diff yet); `pr-review-loop` = introduced by this diff; `pr-round` = introduced by this diff,
single round; `execution-writing-disposition` = the in-flight change being written; `slop-review` =
whole-repo / accumulated.

**Size budget:** 1000 lines — Class 1 threshold, single-sourced here.

---

## `[regression]` classes — hard structural regressions

### Class 1 — File crosses the 1000-line budget

- **Reviewer form:** `[regression]` — File crosses 1000 lines in this diff (or a change newly
  creates one above 1000 lines). A file already over 1000 lines the PR edits in-place without
  materially lengthening is pre-existing and out of scope. Remedy: split along a real seam; a
  mechanical line-count trim is not the fix.
- **Design-constraint form:** No target file is planned past the 1000-line budget. A change that
  would cross it names the split up front: which responsibilities move to which module. Shallowness
  symptom — the file is doing too much; depth is insufficient to justify its scope as one module.
- **Trigger predicate:** a change pushes an existing file past 1000 lines, or newly creates one
  above it.
- **Prevention altitude:** `plan`

### Class 2 — New ad-hoc branch bolted into an unrelated flow

- **Reviewer form:** `[regression]` — New ad-hoc branch bolted into an unrelated flow. A new
  conditional / special-case branch wedged into a control flow that does not own that concern —
  "spaghetti" coupling routing one feature's logic through another's code path.
- **Design-constraint form:** New behavior enters at the owning seam / extension point, not as a
  conditional bolted into a foreign flow. The plan names the seam where the concern belongs and
  confirms the implementation routes through it. Shallow-module symptom — the conditional is at the
  wrong seam.
- **Trigger predicate:** a change introduces a conditional or special-case in a control flow that
  does not own that concern.
- **Prevention altitude:** `plan`

### Class 3 — Feature logic leaked into a shared / canonical path

- **Reviewer form:** `[regression]` — Feature logic leaked into a shared / canonical path.
  Feature- or caller-specific logic pushed down into a shared helper, base class, or canonical code
  path so every other caller now pays for, or must reason about, one feature's special case.
- **Design-constraint form:** Feature-specific logic lands in a feature-owned module; shared /
  canonical paths stay general. The plan names the owning module and confirms no feature-conditional
  branch enters a shared path. Locality violation — leakage across the seam between feature and
  shared code forces every caller to reason about one feature's case.
- **Trigger predicate:** a change pushes feature- or caller-specific logic into a shared helper,
  base class, or canonical code path other callers traverse.
- **Prevention altitude:** `plan`

---

## `[simplification]` classes — missed simplifications (code-judo)

### Class 4 — Near-duplicate of a canonical helper

- **Reviewer form:** `[simplification]` — Near-duplicate of a canonical helper. New code that
  re-implements (slightly differently) a helper, validator, or pattern that already exists
  canonically in the repo.
- **Design-constraint form:** Before adding a helper, transform, or pattern, the plan names the
  canonical helper to reuse — depth / leverage: one implementation paying back across N call sites —
  or explicitly states that none exists and why a new one is justified rather than extending the
  canonical one.
- **Trigger predicate:** a change adds new code that re-implements a helper or pattern that already
  exists canonically in the repo.
- **Prevention altitude:** `plan`

### Class 5 — Missed structural simplification

- **Reviewer form:** `[simplification]` — Missed structural simplification. The change adds
  machinery where reframing the problem removes the need for it. Remedy: delete the layer, or
  reframe the state model so the case can't arise.
- **Design-constraint form:** Emergent during writing — only visible once the implementation shape
  is clear. No stable plan-time check; the planning skill does not act on this class.
- **Trigger predicate:** a change adds machinery where reframing the problem would eliminate it
  entirely.
- **Prevention altitude:** `execution`

### Class 6 — Thin wrapper / identity abstraction

- **Reviewer form:** `[simplification]` — Thin wrapper / identity abstraction. A function, class,
  or indirection that forwards to one callee and earns nothing: no added invariant, no seam, no
  test surface. Remedy: inline it; let callers reach the real thing.
- **Design-constraint form:** A module planned behind a seam must earn depth — an added invariant,
  a test surface, or adapter variation across the seam. The deletion test: if deleting it makes
  complexity vanish rather than reappear across callers, it is a pass-through. Whether depth is
  earned is only verifiable once written; the planning skill does not act on this class.
- **Trigger predicate:** a change introduces a function, class, or indirection that forwards to one
  callee and earns nothing: no invariant, no seam, no test surface.
- **Prevention altitude:** `execution`

### Class 7 — Cast / optionality churn

- **Reviewer form:** `[simplification]` — Cast / optionality churn. A value made optional,
  nullable, or `any`-typed and then defended with casts and guards the PR also adds, when the type
  could have stayed precise at the source. Remedy: tighten the type where the value originates;
  delete the downstream guards.
- **Design-constraint form:** Emergent during writing — type precision at the seam is a design
  principle, but optionality churn only becomes visible when guards accumulate in the
  implementation. No plan-time check; the planning skill does not act on this class.
- **Trigger predicate:** a change widens a type to optional / nullable / `any` and adds defensive
  casts and guards it also introduces, when the value could have stayed precise at the source.
- **Prevention altitude:** `review-only`
