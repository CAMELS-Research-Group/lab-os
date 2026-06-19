---
title: Planning Completion Checklist
description: Self-attest completion of the Planning part — every item is checkable by inspecting your artifact, not by judgment. Pass every check before moving to Building.
---

# Planning Completion Checklist

Use this checklist after you have worked through all three movements and both worksheets. Tick each
item by inspecting your artifact — not by how it feels. Every item is a yes/no check against
something you can see.

**You are execution-ready when every item below is checked.**

---

## PRD — the requirements document

- [ ] **Problem is stated concretely.** Your PRD names what is broken, for whom, and why it
      matters. It is a problem statement, not a solution pitch. You could hand it to someone who
      was not in the conversation and they would agree on what the problem is.

- [ ] **Success criteria are measurable.** Each criterion names something you can check — a
      number, a yes/no outcome, a behavior you can observe. None of the criteria reads as "it
      feels better" or "users enjoy it."

- [ ] **Out-of-scope is explicit.** Your PRD names at least one thing that is deliberately not
      being built in this version. The boundary between "done for now" and "done eventually" is
      written down.

- [ ] **Constraints are named.** Real limits on the work (time, budget, data access, tools,
      approvals, infrastructure) are listed as facts, not preferences.

- [ ] **No section is vague or missing.** If any section has "TBD" or a placeholder, it is named
      as an open question — and that open question does not block the work from starting.

---

## Task list — the execution plan

- [ ] **Every in-scope PRD item maps to at least one task.** Nothing from your success criteria
      or in-scope list is unaccounted for in the task list.

- [ ] **Every task passes the granularity check.** Each task is small enough to complete and
      verify in one sitting. Any task that was too large has been split.

- [ ] **Every task has a verification note.** Each note names something you can check directly —
      a command you run, a file that exists with specific content, or a behavior you can trigger
      and observe. "It works" and "Claude says done" are not verification notes.

- [ ] **Dependencies are mapped.** You can read the task list and know which tasks to start first.
      Tasks are grouped into at least a first wave (no dependencies) and a second wave (depends on
      first wave).

- [ ] **Nothing in the task list is out of scope.** If a task crept in that is not supported by
      the PRD's in-scope section, it has been either added to the PRD or removed from the task list.

---

## Execution-ready bar

The checklist above operationalizes a single question: **can this plan be handed to an autonomous
run without further clarification?**

A plan is execution-ready when:

- The PRD has measurable success criteria and an explicit out-of-scope list — so the agent knows
  what "done" looks like and what is not its job
- The task list is ordered and individually verifiable — so the agent can take one task at a time
  and you can check the result without asking it to self-report

If any item above is unchecked, that is the thing to fix before Building. An unchecked item is
not a judgment call — it means a specific part of the artifact needs more work.

---

## Next step

When every item is checked: take your PRD and task list to **Part 2 — Building**.

Bring the task list open and readable. The first thing the Building part does is work from it.
