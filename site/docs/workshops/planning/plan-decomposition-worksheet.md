---
title: Plan / Decomposition Worksheet
description: Turn your finished PRD into an ordered list of individually-verifiable tasks — the roadmap an autonomous run can act on.
---

# Plan / Decomposition Worksheet

This worksheet is for **Movement 3** of the [Planning part](./index.md): converting your PRD into
an ordered, individually-verifiable task list — the actual roadmap you hand to the Building part.

You bring a filled PRD (from the [PRD-interrogation worksheet](./prd-interrogation-worksheet.md)).
By the end of this worksheet you have a task list where every item has a clear verification note,
dependencies are mapped, and nothing is too large to finish in one sitting.

---

## How to use this worksheet

1. Open a Claude session alongside this worksheet.
2. Paste your filled PRD and use one of the **decomposition prompts** below to get a first-pass
   task list.
3. Work through the **granularity check** for each task — split anything too large.
4. Fill in a **verification note** for every task.
5. Map **dependencies** so you know what order to work in.
6. Check the **done condition** at the bottom before you close this worksheet.

---

## Decomposition prompts

Choose one to get a first-pass task list from Claude. Then refine using the sections below.

---

**Prompt A — Top-down breakdown**

Paste your PRD, then add:

> "Based on the PRD above, decompose the work into a flat list of implementation tasks. Each task
> should be small enough to complete and verify in one sitting. For each task write: (1) what it
> does, (2) what files or components it touches, and (3) a brief note on how you would verify it
> is done. Do not group tasks into phases yet — just list them."

---

**Prompt B — Success-criteria-first**

Paste your PRD, then add:

> "Take each success criterion in the PRD above and break it down into the smallest tasks whose
> combined completion would make that criterion true. For each task: state what it does, what
> artifact it produces, and one concrete way to verify it is done. Flag any task that looks too
> large to complete in one sitting — those should be split further."

---

**Prompt C — Dependency-first**

Paste your PRD, then add:

> "Decompose the PRD above into tasks, then identify the dependency order: which tasks must
> complete before others can start? Organize the list into waves — a first wave of tasks with no
> dependencies, then subsequent waves that depend on earlier ones. For each task include a short
> verification note: how would you confirm it is done without asking the agent that built it?"

---

## Task-granularity rule

**Each task must be small enough to finish and verify on its own.** The test: can you start this
task, complete it, and check whether it is done — all in one sitting?

If the answer is no, the task is too large. **Split it.**

Signs a task needs to be split:

- Completing it would take more than one uninterrupted work session
- Verifying it requires checking several unrelated things
- The task description contains more than one "and" joining distinct pieces of work
- You find yourself writing "etc." or "and so on" in the task description

There is no penalty for having many small tasks. A list of thirty well-scoped tasks is more useful
than a list of five tasks that each hide a week of work. Granularity now saves debugging time
during autonomous execution.

---

## Task list

Fill in one row per task. You can add rows as needed.

| # | Task | Verification note | Depends on |
|---|---|---|---|
| 1 | | | — |
| 2 | | | |
| 3 | | | |
| 4 | | | |
| 5 | | | |

*Add rows as needed. Keep task descriptions short — one line that names what changes. The
verification note is where the specifics go.*

---

## Writing a verification note

The verification note answers: **how will I know this task is done, without asking Claude?**

A strong verification note names something you can observe directly:

- A command you run and the exit code or output you expect (`npm test` exits 0; the new endpoint
  returns a 200 with the expected shape)
- A file that exists with specific content (the config file is present and contains the required
  key)
- A behavior you can trigger and observe (the button appears and navigating to it loads the correct
  page)
- A metric you can read (the migration script processes all rows without errors and the row count
  in the new table matches the source)

A weak verification note says "it works" or "Claude confirms done" — neither of those is a
verification. If you cannot write a concrete note for a task, the task is probably either too large
or too vague. Split it or tighten the description until a concrete note becomes possible.

The verification notes you write here become the verification commands you run in the Building part.
Getting them specific now means you will not have to figure out "how do I know this worked?" under
pressure during an autonomous run.

---

## Dependency mapping

Once your task list is filled, mark which tasks block others. Use the **Depends on** column to name
the task number(s) that must be complete before each task can start.

Then group the list into **execution waves**:

**Wave 1 — no dependencies (can start immediately):**

- Task # ___
- Task # ___

**Wave 2 — depends on wave 1:**

- Task # ___
- Task # ___

**Wave 3 — depends on wave 2 (and so on):**

- Task # ___

Tasks in the same wave can potentially run in parallel. Tasks in later waves wait for their
dependencies. This wave structure is what the Building part uses to hand off work to Claude
efficiently.

---

## Done condition

This worksheet is done when you can answer **yes** to every item below by inspecting your task
list, not by judgment.

- [ ] Every in-scope item from the PRD maps to at least one task. Nothing from the PRD's success
      criteria is unaccounted for.
- [ ] Every task passes the granularity check: completable and verifiable in one sitting. Any task
      that failed the check has been split.
- [ ] Every task has a verification note that names something you can check directly — not "it
      works" or "Claude says done."
- [ ] Dependencies are mapped. You can read the list and know which tasks to start first.
- [ ] Nothing in the task list is out of scope per the PRD. If a task crept in that is not covered
      by the PRD, either add it to the PRD's in-scope section or remove the task.

When every item is checked, your task list is execution-ready. Take it to the
[completion checklist](./completion-checklist.md) to self-attest before moving to Building.

If a task passes this done condition but you are still uncertain whether it is the right
decomposition, that uncertainty is useful information — write it as an open question on your PRD
rather than letting it stall the worksheet.
