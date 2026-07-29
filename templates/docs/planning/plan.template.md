# <Slice title> — implementation plan

<!-- Bundle lifecycle: bundles live at _specs/<repo>/YYYY-MM-DD-<slug>/; overflow and archival
     are co-located in the bundle (spec-log altitude per .claude/rules/03-logging.md).
     Spec: .claude/rules/04-docs.md. -->

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. On
> code-touching tasks, apply the `execution-writing-disposition` skill while implementing.

**Goal:** <one sentence — what this plan delivers>

**PRD:** [prd.md](./prd.md) · **Spec:** [spec.md](./spec.md) <!-- omit for chore/docs-only bundles -->

**Plan format note (lab-os convention):** tasks specify *what* the implementation must satisfy, not *how*.
No literal code. The only code blocks allowed are short shell commands in **Verification** lines.

---

## Phase A — <phase name>

### Task 1: <task title>

**Files:**
- Create: `<exact/path/to/file>`
- Modify: `<exact/path/to/file>`

**Depends on:** — <!-- task numbers, or — if none -->

**Spec:** [§N <section title>](./spec.md#anchor) <!-- design-authority section this task satisfies; prd.md anchor only for bundles that omit spec.md -->

<!-- Optional one-sentence context when the why isn't obvious from the title. -->

**Architectural constraints:** <!-- code-touching plans: run `standards-aware-planning` (derives from the taxonomy; expresses constraints in `codebase-design` terms) to populate; `none triggered` when empty; doc-only plans omit this element -->

**Acceptance:**
- <behavior the implementation must demonstrate — no code, no implementation detail>
- <behavior>
- <behavior>

**Verification:**
```shell
<short shell command the implementing agent runs to confirm done>
```

**Agent-suitable:** <yes | partial | no> <!-- can an agent run this task unattended? yes = Acceptance+Verification fully pin "done"; partial = agent does the bulk but a human checkpoint is needed mid-task; no = human-driven -->

**Commit:** `<type>(<scope>): <subject>`

---

### Task 2: <task title>

**Files:**
- Create: `<exact/path/to/file>`

**Depends on:** 1

**Spec:** [§N <section title>](./spec.md#anchor)

**Architectural constraints:**

**Acceptance:**
- <behavior>
- <behavior>

**Verification:**
```shell
<command>
```

**Agent-suitable:** <yes | partial | no>

**Commit:** `<type>: <subject>`

---

<!-- Add Task 3, 4… and Phase B, C… as needed. Delete placeholder tasks with no content. -->

---

<!-- Execution detail (plan deviations, implementation-altitude calls, gate evidence) lives in
     this bundle's log.md, not here — log.md owns the Execution Log. See log.template.md. -->
