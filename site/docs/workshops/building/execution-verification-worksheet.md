---
title: Execution and Verification Worksheet
description: Walk your first autonomous run from hand-off to proof, then scale to a multi-task chunk — recording what you framed, what ran, what the diff showed, and what the verification command returned.
---

# Execution and Verification Worksheet

This worksheet runs alongside [Exercise 4 — First autonomous execution](./exercises.md#exercise-4--first-autonomous-execution)
and [Exercise 5 — Scaling execution](./exercises.md#exercise-5--scaling-execution). Work through
it during those exercises. Fill each section as you go — the record is not a formality; it is what
proves the skill.

---

## Before the run — write down your verification command

Before you hand off any task to an autonomous run, write down your project's verification command
here. Do this before running anything.

**Your project's single verification command:**

```
(fill this in — example: npm test  |  pytest  |  cargo test  |  make check)
```

**What this command actually checks:**
_(name two or three behaviors the command exercises — e.g. "runs unit tests for the data parser and
the formatter; does not hit the live API")_

**What this command does not check:**
_(name at least one thing the command leaves unchecked — e.g. "does not verify the login flow
against a real provider; does not exercise failure-recovery paths")_

The verification command is your source of truth. Exit code 0 means the gate passes; anything
non-zero means it fails. Claude's statement that a task is done is not the verification. The
command is.

> **Why write it down first?** An autonomous run can produce output that looks correct but breaks
> something the command doesn't cover. Writing the command and its limits before the run forces
> you to think about what "passing" actually means — before the result is in front of you.

---

## Part A — First autonomous run (Exercise 4)

### A1 — The task you are handing off

**Task name / description** (from your task list):

```
(copy the task from your plan)
```

**Verification note for this task** (from your task list):

```
(the specific command or check for this particular task)
```

---

### A2 — The hand-off framing

A well-framed hand-off has three parts. Fill each section before you start the run.

**Context** (what the project is, what exists, what this task fits into):
> Over-include rather than under-include. Claude cannot ask follow-up questions mid-run.

```
(write your context here)
```

**The task** (paste from your plan, including the verification note):

```
(paste the task text)
```

**The stop condition** (what "done" looks like, and what Claude should verify before stopping):

```
(e.g. "stop when the verification step passes; do not continue to other tasks")
```

---

### A3 — During the run

Let Claude run without intervening. Do not correct it mid-run unless it has completely stalled —
stopped producing output and is clearly stuck, not just between steps.

Note anything that surprises you while it runs (questions Claude tries to ask, unexpected
behavior, a long pause):

```
(optional — fill in if something notable happens)
```

---

### A4 — After the run: inspect the diff

When Claude says it is done, do not accept the self-report as proof. Inspect what actually changed.

Run this in the worktree directory:

```
git diff HEAD
```

or, if changes are committed:

```
git log -1 -p
```

Read the actual diff output — not the summary Claude produces. A summary can omit changes or
describe intent rather than what happened.

**What the diff showed** (describe in your own words — which files changed, what was added or
removed or modified):

```
(write what you saw in the diff)
```

**Did the diff match what the task described?** _(yes / no / partially — explain if no or partial)_

```
(your answer)
```

---

### A5 — After the run: run the verification command

Run your project's verification command (the one you wrote in the section above). Run it in the
worktree directory.

**Command you ran:**

```
(the exact command)
```

**Exit code returned:** _(0 = pass, non-zero = fail)_

```
(the exit code)
```

**Output summary** (paste the final lines of output, or the error message if it failed):

```
(paste output here)
```

---

### A6 — Result record

| | |
|---|---|
| Did the diff show the expected changes? | yes / no / partial |
| Did the verification command exit 0? | yes / no |
| Task result | passed / failed / partial |

**If the verification command failed — what did you learn?**

```
(describe the failure; what the command said; what you think went wrong)
```

> **Catching a failure is the lesson.** A run where the verification command exits non-zero is
> not a failed exercise. It is the skill in action: verify-don't-trust. If you caught it, you
> proved the hand-off works correctly. Record the failure and note it as a success at the
> verification discipline — then decide whether to re-run after fixing the issue.

---

## Part B — Multi-task chunk hand-off (Exercise 5)

This section covers handing off two or more tasks in a single framed request and verifying each
result separately.

---

### B1 — The tasks in this chunk

List the tasks you are handing off in this run. Each task needs its own verification note.

| Task | Verification note for this task |
|---|---|
| (task 1 name / description) | (its specific check) |
| (task 2 name / description) | (its specific check) |
| (add rows as needed) | |

**Why these tasks can run together** (no dependencies between them, or explain the sequence if
they run in order):

```
(write your reasoning here)
```

---

### B2 — The hand-off framing for the chunk

**Context** (same project context as the first run; include what changed as a result of that run):

```
(write context — include the state the project is in after Exercise 4)
```

**The tasks** (paste each task description with its verification note):

```
(paste tasks here)
```

**The stop condition** (complete each task, run its verification step, report the result of each
before moving to the next):

```
(write your stop condition)
```

---

### B3 — Result record per task

Fill this section as each task completes — verify one task before moving to the next.

#### Task 1

**Diff review** (what actually changed):

```
(describe the diff)
```

**Verification command run:**

```
(exact command)
```

**Exit code:** _(0 = pass, non-zero = fail)_

```
(exit code)
```

**Result:** passed / failed / partial

---

#### Task 2

**Diff review** (what actually changed):

```
(describe the diff)
```

**Verification command run:**

```
(exact command)
```

**Exit code:**

```
(exit code)
```

**Result:** passed / failed / partial

---

_(Add a section per additional task if the chunk had more than two.)_

---

### B4 — Chunk summary

| | |
|---|---|
| Tasks in the chunk | (count) |
| Tasks that passed verification | (count) |
| Tasks that failed verification | (count) |

**For any task that failed:** describe what the failure showed and whether you will re-run it
after investigating:

```
(your notes)
```

**Did any tasks interfere with each other** (e.g. edited the same file in incompatible ways)?
If yes, what hidden dependency did that reveal?

```
(your notes, or "no interference")
```

---

## Reflection

After completing both Part A and Part B, answer these:

**What is the most useful thing the verification step caught** (or would have caught if a run had
failed)?

```
(your answer)
```

**What does your verification command not cover** that you are now thinking about more carefully?

```
(your answer)
```

**How would you change your hand-off framing next time to produce a cleaner autonomous run?**

```
(your answer)
```

---

## Next steps

When this worksheet is filled for both Part A and Part B, open the
[completion checklist](./completion-checklist.md) to self-attest that you have completed the
Building part.

If you have not yet defined your quality gate and review standard, open the
[quality-gate and review worksheet](./quality-gates-worksheet.md) before the completion checklist
— the checklist asks about both.
