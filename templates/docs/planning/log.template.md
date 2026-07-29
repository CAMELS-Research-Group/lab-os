# <Slice title> — spec log

<!-- Bundle spec-log altitude — the single per-bundle log. Owns ALL execution + decision detail
     for this slice:
       1. Load-bearing decisions made during this slice (with alternatives weighed), incl.
          rejected alternatives / decisions-with-rationale. This log is the HISTORY — the
          binding current-state resolution lives once in this bundle's spec.md, which links back.
       2. Discarded design detail that would bloat prd.md or plan.md but should survive for
          future reference (rejected approaches, spike findings, constraints that shaped choices).
       3. The Execution Log — plan deviations, implementation-altitude calls, gate evidence.
          plan.md no longer carries an Execution Log; that responsibility moved here.
     Known gaps are NOT logged here — they live in the PRD's "Open questions" section.
     NOT the project-log (Standing-Decisions) shape — this log archives with the bundle.
     Routing: decisions that outlive this slice → project_log.md.
     Spec: .claude/rules/03-logging.md -->

**Date:** YYYY-MM-DD · **Repo:** <repo> · **Slice:** <slug>

---

## Decisions

<!-- One subsection per load-bearing decision. Include real alternatives weighed and why each
     was rejected — "we considered X but chose Y because Z" is the signal a future reader
     needs to avoid re-litigating the choice.
     Entry format per 03-logging.md: Decision / Why / Alternatives / Refs -->

### D1: <decision title>

**Decision:** <what was decided>

**Why:** <load-bearing rationale — not a restatement of the decision>

**Alternatives:**

- **<option A>** — <why rejected>
- **<option B>** — <why rejected>

**Refs:** <!-- PR#, absolute paths, or URLs -->

---

### D2: <decision title>

**Decision:** <what was decided>

**Why:** <why>

**Alternatives:**

- **<option A>** — <why rejected>

**Refs:**

---

<!-- Add D3, D4… as needed. Delete placeholder sections that have no content. -->

## Discarded detail

<!-- Design alternatives, spike findings, constraints, or context that shaped the slice but
     doesn't belong in prd.md or plan.md. Keep it so future readers don't re-litigate.
     Use subsections or a flat bulleted list — whatever fits the material. -->

- <discarded approach or finding>
- <discarded approach or finding>

---

## Execution Log

<!-- Plan-execution altitude (see .claude/rules/03-logging.md §altitudes). This bundle's single
     Execution Log lives here — plan.md no longer carries one.
     What belongs here: deviations from the plan, implementation-altitude calls, gate evidence
     (the verification output that proved a task done).
     What does NOT belong here: load-bearing decisions (→ Decisions section above or
     project_log.md), bare status ("merged, smoke passed" → PR comment), session narrative
     (→ PR body).
     This log closes when the shipping PR merges — post-merge evidence goes to a comment on
     that PR, not a trailing entry.

     Entry grammar (one line each):
     YYYY-MM-DD HH:MM · task N · <what happened / why / output> -->

<!-- entries below — newest at top -->
