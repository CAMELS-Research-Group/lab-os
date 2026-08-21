# <Slice title> — design

<!-- The technical shape a code-touching slice commits to. REQUIRED when the slice
     meaningfully touches code (new module/service, contract or schema change,
     cross-component behavior); docs-only and chore bundles omit this file.
     spec.md owns WHAT was decided; this file owns the SHAPE those decisions produce.
     Interface and schema sketches belong here — implementation code does not.
     Spec: .claude/rules/04-docs.md §ENG document standards. -->

**Date:** YYYY-MM-DD · **Repo:** <repo>
**PRD:** [prd.md](./prd.md) · **Spec:** [spec.md](./spec.md) ·
**Plan:** [plan.md](./plan.md) · **Log:** [log.md](./log.md)

---

## Overview

<!-- 3–6 sentences: what this slice changes technically and where it sits in the existing
     system. Name the components touched. -->

## Architecture

<!-- Components and seams: what exists, what is added or changed, who calls whom, which
     module owns which responsibility. A simple ASCII or mermaid sketch beats prose. -->

## Contracts & schemas

<!-- The interfaces this slice commits to: function/endpoint signatures, data schemas, file
     formats, CLI flags. Sketch-level, but pinned enough that two implementers converge on
     compatible code. Breaking changes to existing contracts get called out explicitly. -->

## Data flow

<!-- How data moves through the changed path: inputs → transformations → outputs; where
     state lives; what persists and where. -->

## Failure modes

<!-- What can go wrong and what the design does about it: error paths, partial-failure
     behavior, justified fallbacks, recovery. A fallback without a justification here will
     be flagged in review. -->

## Constraints honored

<!-- Byte budgets, performance floors, platform limits, rule obligations this design is
     shaped around. "None beyond the rules" is a valid entry. -->

<!-- Alternatives weighed and rejected go in log.md, not here — this file states the shape
     that won. Open design questions go in prd.md §Open questions. -->
