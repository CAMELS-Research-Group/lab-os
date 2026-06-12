# <Slice title> — implementation plan

<!-- Bundle lifecycle: created in docs/work/YYYY-MM-DD-<slug>/; archived as a unit to
     docs/work/completed/ (done) or docs/work/abandoned/ (abandoned, with reason prepended to
     design.md). Spec: lab-os docs/superpowers/specs/2026-06-10-logging-and-docs-standard-design.md §5. -->

**Goal:** <one sentence — what this plan delivers>

**Design:** [design.md](./design.md)

**Plan format note (lab rule):** tasks specify *what* the implementation must satisfy, not *how*.
No literal code. The only code blocks allowed are short shell commands in **Verification** lines.

---

## Phase A — <phase name>

### Task 1: <task title>

**Files:**
- Create: `<exact/path/to/file>`
- Modify: `<exact/path/to/file>`

**Depends on:** — <!-- task numbers, or — if none -->

**Spec:** [§N <section title>](./design.md#anchor)

<!-- Optional one-sentence context when the why isn't obvious from the title. -->

**Acceptance:**
- <behavior the implementation must demonstrate — no code, no implementation detail>
- <behavior>
- <behavior>

**Verification:**
```shell
<short shell command the implementing agent runs to confirm done>
```

**Commit:** `<type>(<scope>): <subject>`

---

### Task 2: <task title>

**Files:**
- Create: `<exact/path/to/file>`

**Depends on:** 1

**Spec:** [§N <section title>](./design.md#anchor)

**Acceptance:**
- <behavior>
- <behavior>

**Verification:**
```shell
<command>
```

**Commit:** `<type>: <subject>`

---

<!-- Add Task 3, 4… and Phase B, C… as needed. Delete placeholder tasks with no content. -->

---

## Execution Log

<!-- Altitude: plan-execution (see .claude/rules/03-logging.md §altitudes).
     What belongs here: deviations from the plan, implementation-altitude calls, gate evidence
     (the verification output that proved a task done).
     What does NOT belong here: load-bearing decisions (→ project_log.md), status updates
     ("merged, smoke passed" → PR comment), session narrative (→ PR body).
     This log closes when the shipping PR merges — post-merge evidence (deploy green,
     branch cleanup) goes to a comment on that PR, not a trailing entry.

     Entry grammar (one line each):
     YYYY-MM-DD HH:MM · task N · <what happened / why / output> -->

<!-- entries below — newest at top -->
