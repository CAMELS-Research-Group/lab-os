# <Slice title> — implementation plan

<!-- Bundle lifecycle: created in docs/work/YYYY-MM-DD-<slug>/; archived as a unit to
     docs/work/completed/ (done) or docs/work/abandoned/ (abandoned, with reason prepended to
     design.md). Spec: lab-os docs/superpowers/specs/2026-06-10-logging-and-docs-standard-design.md §5. -->

**Goal:** <one sentence — what this plan delivers>

**Design:** [design.md](./design.md)

**Plan format note (lab-os convention):** tasks specify *what* the implementation must satisfy, not *how*.
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

**Agent-suitable:** <yes | partial | no> <!-- can an agent run this task unattended? yes = Acceptance+Verification fully pin "done"; partial = agent does the bulk but a human checkpoint is needed mid-task; no = human-driven -->

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

**Agent-suitable:** <yes | partial | no>

**Commit:** `<type>: <subject>`

---

<!-- Add Task 3, 4… and Phase B, C… as needed. Delete placeholder tasks with no content. -->

---

<!-- No Execution Log in the plan (see .claude/rules/03-logging.md §Log altitudes and
     .claude/rules/04-docs.md §ENG document standards, Plan): deviations from the plan and
     gate evidence live in the bundle's log.md, load-bearing decisions route per the
     03-logging.md entry triggers, and post-merge status goes to a comment on the PR. -->
