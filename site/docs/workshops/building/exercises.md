---
title: Building Exercises
description: Five exercises that take your execution-ready plan through the three execution modes — from monitored-sequential through first autonomous execution and on to scaling with a multi-task chunk.
---

# Building Exercises

These five exercises run against your own plan — the execution-ready task list you produced in
Part 1. Work through them in order. Each builds directly on the previous one.

The exercises use a consistent shape throughout:

> **Goal** — what this exercise produces
> **You start with** — what you need before beginning
> **What you do** — the shape of the work (not a script; you talk to Claude in your own words)
> **Done when** — how you know the exercise is complete
> **If you're stuck** — where to look

---

## Execution modes

The five exercises move through three execution modes in order.

| Exercises | Mode | What changes |
|---|---|---|
| 1–2 | Monitored sequential | You direct each step; Claude executes one task; you inspect and verify before moving on |
| 3–4 | Autonomous | You frame the task and let Claude run end to end; you verify the result |
| 5 | Scaling (multi-agent / multi-task) | You hand off a wave of tasks and verify each result as they complete |

Do not skip ahead. The monitored exercises in 1 and 2 teach you what good output looks like before
you hand off a full task. Exercise 3 sets up the safety net that Exercise 4 depends on.

---

## Exercise 1 — Plan to roadmap

**Mode:** Monitored sequential

---

**Goal:** Convert your task list into an actionable roadmap — every task stated clearly enough that
Claude could pick one up without additional context from you.

**You start with:** Your execution-ready plan (the output of the Planning part). It should already
have tasks and verification notes. If the plan is not execution-ready, return to the
[Planning completion checklist](../planning/completion-checklist.md) before continuing.

**What you do:**

Open a Claude session alongside your plan. Your goal is to review each task and confirm it meets
the "individually-executable" standard: Claude can start it, finish it, and you can verify it
— all without you supplying information mid-run.

Walk through the list task by task. For each one, check:

- Is the task description specific enough that Claude knows what to build or change?
- Does the task include enough context about its place in the project (what already exists, what
  it connects to)?
- Is the verification note concrete — a command, an observable behavior, or a file with specific
  content?

If a task fails any of these checks, revise it on the spot. Ask Claude to restate the task
description as it would understand it from the text alone — if the restatement misses something
important, the task needs more context.

The deliverable from this exercise is a revised, confirmed roadmap: a task list where every item
is self-contained enough to be handed off without explanation.

**Done when:**

- Every task has a description specific enough to act on without follow-up questions
- Every task has a verification note that names something you can check directly
- You have gone through the whole list, not just skimmed it

**If you're stuck:**

If you're not sure whether a task is self-contained, ask Claude: "If I handed you just this task
description and nothing else, what questions would you need answered before you could start?"
The questions surface what the task is missing. Fill them in.

---

## Exercise 2 — Sequencing and dependencies

**Mode:** Monitored sequential

---

**Goal:** Mark what blocks what in your task list, then group tasks into execution waves — first
wave (no dependencies), second wave (depends on first), and so on. This wave structure is what
makes it possible to hand off parallel work later.

**You start with:** The confirmed roadmap from Exercise 1.

**What you do:**

In a Claude session, paste your task list and ask Claude to identify dependencies: which tasks
cannot start until another task is complete?

Review the dependency map Claude produces. Challenge any dependency you think is wrong — sometimes
what looks like a dependency is actually just an ordering preference. A real dependency means the
task literally cannot run (it would fail or produce garbage) without its predecessor complete.

Once the dependencies look right, group the tasks into waves:

- **Wave 1** — tasks with no dependencies (safe to start immediately, can run in parallel)
- **Wave 2** — tasks whose only dependencies are in Wave 1
- **Wave 3** (and so on) — tasks that depend on earlier waves

Record the wave structure on your task list. This is the execution order you will use in
Exercises 4 and 5.

**Done when:**

- Every task has its dependencies noted (or "none" if it's Wave 1)
- Tasks are grouped into numbered waves
- You can look at Wave 1 and name two or more tasks that could run at the same time

**If you're stuck:**

If you're finding that everything depends on everything, the tasks are probably too coarse. Return
to the granularity check from the [plan/decomposition worksheet](../planning/plan-decomposition-worksheet.md)
and split the large tasks. Tight dependencies often dissolve when tasks are smaller.

---

## Exercise 3 — Worktree isolation

**Mode:** Monitored (setup for the autonomous run)

---

**Goal:** Create a throwaway workspace — a git worktree on a separate branch — so that the
autonomous run in Exercise 4 cannot touch the files you have open in your editor or affect your
main branch. This is the safety net that makes it reasonable to let Claude run without supervision.

**You start with:** A clean working tree on your experiment branch (confirmed in the
[pre-flight worksheet](./pre-flight.md)).

**What you do:**

In your terminal, navigate to your project's repository and create a worktree for the first
autonomous run:

```
git worktree add ../my-project-run1 HEAD
```

Replace `my-project-run1` with whatever name makes sense for your project. The path
(`../my-project-run1`) places the worktree as a sibling directory next to your main checkout —
adjust the path if your directory layout requires it.

Confirm the worktree was created and is on the right branch:

```
git worktree list
```

You should see two entries: your main working tree and the new worktree.

When Exercise 4 is done and you have verified the result, you will remove the worktree:

```
git worktree remove ../my-project-run1
```

Do not remove it yet. Leave it in place and open it in a second terminal window or editor tab
so you know where the autonomous run will execute.

**Done when:**

- `git worktree list` shows the new worktree in addition to your main working tree
- You know the path where Claude will execute during Exercise 4
- Your main working tree is unchanged

**If you're stuck:**

If `git worktree add` fails, check the troubleshooting section in the
[pre-flight worksheet](./pre-flight.md) — the same failure modes that appear there apply here.
The most common cause is that the worktree directory already exists from a previous attempt;
remove the directory and try again.

---

## Exercise 4 — First autonomous execution

**Mode:** Autonomous

---

**Goal:** Hand Claude one task from Wave 1 of your roadmap, let it run end to end without
intervention, then **verify the result yourself** — with a diff and your verification command.
This exercise is the pivot. The skill is **let go, then verify** — not micromanage, not trust
blindly.

**You start with:** The worktree from Exercise 3. Your task list from Exercise 2, with Wave 1
identified. A single Wave 1 task selected for the run.

**What you do:**

Pick one task from Wave 1 — choose something self-contained and not too large. Open a Claude
session in the worktree directory (not in your main working tree).

Frame the hand-off. A well-framed hand-off has three parts:

1. **Context** — what the project is, what already exists, what the task fits into. Over-include
   rather than under-include: Claude cannot ask follow-up questions mid-run.
2. **The task** — paste the task description from your roadmap, including the verification note.
3. **The stop condition** — tell Claude what "done" looks like and that it should stop when the
   verification step passes, not when it thinks the work is done.

Then let Claude run. Do not intervene unless the run stalls completely (it has stopped producing
output and is clearly stuck). Watching is fine; correcting mid-run defeats the purpose of this
exercise.

When Claude says it is done:

1. **Inspect the diff.** In the worktree directory, run `git diff HEAD` or `git log -1 -p`
   to see exactly what changed. Read it. Do not trust the summary — read the actual diff.

2. **Run the verification command.** Execute the single command you named in your task's
   verification note. The exit code is the source of truth: 0 means pass, non-zero means fail.
   Claude's statement that the task is done is not the verification. The command is.

3. **Record the result.** In the [execution and verification worksheet](./execution-verification-worksheet.md),
   write down what happened: did the diff show what you expected, did the verification command pass,
   and if not, what did you learn?

A run where the verification command fails is not a failed exercise. Catching a failure with the
verification step is exactly the skill this exercise builds. If you catch it, you have succeeded
at verify-don't-trust.

**Done when:**

- You have let Claude run to completion without intervening
- You have read the diff
- You have run the verification command and recorded its exit code
- You have filled in the relevant section of the execution and verification worksheet

**If you're stuck:**

- *Claude keeps asking clarifying questions mid-run.* The task framing did not have enough context.
  Stop the run, add the missing context to the hand-off, and start again. This is the most common
  failure mode in Exercise 4.
- *The verification command itself fails for a reason unrelated to Claude's work* (missing
  dependency, wrong directory). Fix the environment issue, then re-run the command. Do not count an
  environment failure as a task failure.
- *The task was too large and the run produced a mess.* Note what happened in the worksheet. Remove
  the worktree, recreate it from HEAD, split the task into smaller pieces, and run Exercise 4 again
  on the first, smaller piece.

---

## Exercise 5 — Scaling execution

**Mode:** Multi-agent / multi-task (scaling toward autonomous)

---

**Goal:** Hand off two or more tasks from your wave structure and verify each result. This exercise
extends the verify-don't-trust discipline from one task to a chunk — the shape that makes
sustained autonomous execution possible.

**You start with:** A completed and verified Exercise 4. Your task list with Wave 1 tasks
(at least two remaining beyond the one you ran in Exercise 4). A fresh worktree for the new run
(or the same worktree if the prior run's output is committed and the tree is clean).

**What you do:**

Select two or more Wave 1 tasks that could run in parallel (no dependencies between them). Frame
a multi-task hand-off:

1. **Context** — same project context as Exercise 4; add what changed as a result of Exercise 4
   (the task that completed and the state it left the project in).
2. **The tasks** — paste both task descriptions, each with its verification note.
3. **The stop condition** — tell Claude to complete each task, run its verification step, and
   report the result of each one before moving to the next.

Claude Code supports dispatching parallel subagents natively within a session: you can describe
the two tasks as parallel work and Claude will coordinate the execution. You do not need external
orchestration tooling for this. If running the tasks sequentially within a single session is more
predictable for your project's structure, that is fine too — the discipline (verify each result
separately) is the same either way.

When Claude reports completion:

- **Verify each task separately.** Run each task's verification command. Do not batch the
  verification — verify one task, record the result, then verify the next.
- **Inspect the diff per task if you can.** A single diff for both tasks is harder to read and
  makes it easier to miss something. If the tasks touched different files, check them separately.

Fill in the multi-task section of the [execution and verification worksheet](./execution-verification-worksheet.md)
for each task.

**Done when:**

- You have handed off at least two tasks in a single framed request
- You have verified each task's result separately with its verification command
- You have recorded both results in the execution and verification worksheet
- You can describe what each task changed and whether the verification passed or failed

**If you're stuck:**

- *One task passed and one failed.* This is a normal and useful outcome. Record both results. The
  task that failed is a candidate to re-run after you investigate and fix the issue. The task that
  passed does not need to be re-run.
- *The tasks interfered with each other* (edited the same file in incompatible ways). That
  interference means they were not truly independent — they had a hidden dependency. Note this in
  the worksheet, add the dependency to your task list, and re-run them sequentially.
- *You're not sure which verification command to run for each task.* Return to your task list. If
  the verification note is too vague to act on, revise it now. A verification note that you cannot
  run is not a verification note.

---

## After the exercises

When you have completed all five exercises, fill in the
[completion checklist](./completion-checklist.md). Every item is checkable by inspecting your
verification output and worksheet — not by how the run felt.

When the checklist is fully checked, you are ready for **Part 3 — Closeout**.
