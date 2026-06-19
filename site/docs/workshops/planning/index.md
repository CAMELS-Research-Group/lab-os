---
title: "Part 1 — Planning"
description: Take your project from a raw idea to an executable plan — journaling the raw idea, letting Claude interrogate you into a solid PRD, and decomposing the result into an ordered list of individually-verifiable tasks.
---

# Part 1 — Planning

This part takes you from a raw idea to an **executable plan**: a written PRD (problem, success
criteria, scope, constraints) produced through AI interrogation, decomposed into an ordered set of
individually-verifiable tasks. That plan is what you bring into Part 2 — Building.

## Prerequisites

Before starting, confirm:

- **Your workspace is set up** — you have Claude Code installed, authenticated, and working.
  If not, follow [Getting Started](/docs/getting-started) first.
- **You are comfortable with the lifecycle** — specifically the spec-first and verification
  discipline explained in [Working with Claude](/docs/working-with-claude).
- **You have brought a project** — a real, low-stakes idea you actually want to build.
  See the [pre-work worksheet](./pre-work.md) to prepare before this part begins.

## What you will produce

By the end of this part you will have:

1. A short **raw-idea capture** — your project stated in your own words
2. A **PRD** surfaced through dialogue with Claude — problem, success criteria, scope, constraints
3. A **decomposed task list** — ordered, individually verifiable, with a verification note per task

This is the "execution-ready" artifact the completion checklist checks before you move on.

---

## Movement 1 — Capture the raw idea

Before you can interrogate the requirements, you need the raw material. The goal here is to
externalize what is in your head — quickly, without over-editing — so Claude has something to work
with in the next step.

### Journaling styles

Pick the approach that fits how you think. Each produces the same input to the interrogation step.

**Option A — Paragraph dump.** Write freely for five to ten minutes on your idea: what problem it
solves, for whom, what "done" looks like in your head, and anything you already know you do not
want to build. No structure required. Completeness matters more than clarity at this stage.

**Option B — Question chain.** Answer three questions in writing:
1. What problem am I solving, and who has it?
2. What would success look like if I could check in six months?
3. What am I explicitly not building?

Answer each in two to four sentences. If an answer leads to a follow-on question, write that too.

**Option C — Voice-to-text transcript.** Talk through your idea out loud as if explaining it to
someone unfamiliar with the problem. Record or transcribe as you go. This surfaces constraints and
assumptions that writing suppresses.

**Option D — Existing notes.** If you already have notes, a rough spec, or a prior attempt at
describing this project, paste them into a scratch doc and mark what still feels right vs. what has
changed. That annotation is your raw capture.

The deliverable from Movement 1 is a short document — a few paragraphs at most — in whatever form
you chose. You do not need a PRD yet. You need something concrete enough to be interrogated.

---

## Movement 2 — Let Claude interrogate you into a PRD

Most people write specs front-to-back and stop too early: the first draft of a success criterion
is usually a vibe, not a measurable bar. The interrogation technique flips the process — instead of
you front-loading a finished spec, Claude asks the clarifying questions until each section of the
PRD has concrete, falsifiable content.

### How the interrogation works

1. Open a Claude session and paste your raw capture from Movement 1.
2. Use one of the interrogation prompts from the
   [PRD-interrogation worksheet](./prd-interrogation-worksheet.md) to ask Claude to play
   the role of a demanding requirements interviewer.
3. Answer each question Claude asks honestly. If you do not know an answer, say so — "I do not know
   yet" is a valid answer that surfaces a constraint or open question.
4. Keep going until each PRD section (problem, success criteria, scope, constraints) has specific,
   falsifiable content. The interrogation is done when Claude cannot find another meaningful
   clarifying question.
5. Ask Claude to summarize the filled PRD.

### What a finished PRD section looks like

- **Problem:** A concrete statement of what is broken, for whom, and why it matters — not "I want
  to build X."
- **Success criteria:** Measurable and falsifiable — something you can check, not something you
  can feel. "Users can complete the core task without asking for help in three out of three test
  runs" is measurable. "It feels polished" is not.
- **Scope:** What is explicitly in — and what is explicitly out. The out-of-scope list is as
  important as the in-scope list.
- **Constraints:** What bounds the work — time, budget, data access, infrastructure, approvals, or
  anything else that is a hard limit rather than a preference.

The [PRD-interrogation worksheet](./prd-interrogation-worksheet.md) has the interrogation prompts
and the skeleton you fill as the interview progresses. Work through it during this movement.

---

## Movement 3 — Decompose the PRD into an executable task list

A PRD is a design artifact. A task list is an execution artifact. The decomposition step bridges
them: you take the success criteria and in-scope items from your PRD and break them down into
tasks small enough that each one can be finished and verified on its own.

### What makes a task "individually verifiable"

A task is individually verifiable when you can answer: *how will I know this task is done, without
asking Claude?* That answer is the verification note — a command, an observable behavior, or a file
that exists with specific content. If you cannot answer the question, the task is either too large
or too vague.

### Task granularity

The right size for a task is: **one sitting, one verification step.** If completing a task would
take more than one work session, split it. If verifying a task requires checking several unrelated
things, split it. A task list with fifty small, verifiable tasks is more useful than one with five
tasks that each conceal a week of work.

### Dependency order

Once you have a list of tasks, mark which tasks block others. Grouping tasks into waves — first
wave (no dependencies), second wave (depends on first), and so on — gives you the parallel
execution structure you will use in Building.

The [plan/decomposition worksheet](./plan-decomposition-worksheet.md) walks you through this step
with prompts and a task-list template. Work through it during this movement.

---

## Worksheets and materials

| Material | When to use it |
|---|---|
| [Pre-work worksheet](./pre-work.md) | Before this part — confirm workspace and bring your idea |
| [PRD-interrogation worksheet](./prd-interrogation-worksheet.md) | During Movement 2 — the interrogation prompts and PRD skeleton |
| [Plan/decomposition worksheet](./plan-decomposition-worksheet.md) | During Movement 3 — task list and dependency map |
| [Completion checklist](./completion-checklist.md) | After all three movements — self-attest before moving to Building |
| [Homework](./homework.md) | After completing the part — reps to build planning speed |

---

## Completion

When you have finished all three movements, work through the
[completion checklist](./completion-checklist.md). Every item must be checkable by inspecting your
artifact — not by judgment. The checklist enforces the "execution-ready" bar: a plan that passes it
can be handed to an autonomous run without further clarification.

When the checklist is fully checked, you are ready for **Part 2 — Building**.
