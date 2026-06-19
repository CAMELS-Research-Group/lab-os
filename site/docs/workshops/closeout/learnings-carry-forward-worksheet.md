---
title: Learnings-Capture and Carry-Forward Worksheet
description: Turn what you built into a durable carry-forward artifact — reusable patterns, dead ends to avoid, and workflow reflections that transfer to your next project.
---

# Learnings-Capture and Carry-Forward Worksheet

This worksheet covers **Movements 2 and 3** of the [Closeout part](./index.md): capturing what you
learned and packaging it as a carry-forward artifact.

You bring the assembled artifacts from the [pre-work worksheet](./pre-work.md) — your repository,
plan, execution and verification records, and your first-draft learnings from Step 3 of pre-work.
By the end of this worksheet you have a single, structured document you can open cold at the start
of your next project.

---

## Before you begin

Confirm:

- [ ] You have completed the [pre-work worksheet](./pre-work.md) — your first-draft learnings
      (patterns, dead ends, something you now understand) are written in Step 3 of that page
- [ ] You have your execution and verification worksheet from Building open alongside this one —
      the specific runs, diffs, and verification results are the raw material for this worksheet
- [ ] You have your quality-gate worksheet from Building open — it names the defect classes you
      discovered and the review findings from your runs

You are not reconstructing from memory. You are refining first drafts into a durable artifact.

---

## Why this artifact matters

Every project produces two kinds of output: **the thing you built** and **the knowledge you gained
building it**. The thing you built exists in your repository. The knowledge is at risk — it lives
only in your working memory, and working memory closes when you move to the next project.

The carry-forward artifact is the structure that prevents that loss. It is not a dump of notes or a
summary of what happened. It is a curated, specific document answering one question: **what does
the next project need to know that this project paid to learn?**

The standard for each entry is that a reader — including your future self, starting a similar
project months from now — could apply it without having been in this project. Specific and
transferable beats comprehensive and vague.

---

## Part 1 — Reusable patterns

A reusable pattern is an approach that worked well enough to carry forward — not just "this was
fine" but "this approach transferred value and I expect it to transfer again."

For each pattern, you need three things:

- **What it is** — describe the approach in one or two sentences
- **Where it helped** — name the specific situation in this project where you used it and what it
  produced
- **When it would and would not transfer** — under what conditions would you use this again, and
  under what conditions would you skip it?

The third question is what makes a pattern genuinely reusable rather than a post-hoc label on
something that happened to work. A pattern that applies everywhere is a truism; a pattern with
conditions is a tool.

**Review your pre-work Step 3 answers and your execution records. For each pattern you identified,
fill in the three elements below.**

---

### Pattern 1

**What it is:**

> _Write here:_

**Where it helped in this project:**

> _Write here (specific task, moment, or outcome — not "planning went well"):_

**When it would transfer / when it would not:**

> _Write here:_

---

### Pattern 2

**What it is:**

> _Write here:_

**Where it helped in this project:**

> _Write here:_

**When it would transfer / when it would not:**

> _Write here:_

---

*(Add more patterns as needed. Most projects surface two to four. More than five usually means some
are not specific enough to be useful — consolidate before continuing.)*

---

## Part 2 — Dead ends and what to avoid

A dead end is a direction you took that cost time without payoff — something you would route around
if starting over. Dead ends are often more transferable than successes because the cost of
re-discovering them is concrete.

For each dead end, capture:

- **What you did** — describe the direction you took
- **What it cost** — time, rework, a wrong assumption that had to be unwound
- **The early signal you missed** — what was the indicator, before the cost was clear, that this
  was the wrong direction? This is the most important part: it is what lets the next project catch
  the problem earlier

**Review your execution records and pre-work Step 3 "Dead ends" answer. For each dead end you
identified, fill in the three elements below.**

---

### Dead end 1

**What you did:**

> _Write here:_

**What it cost:**

> _Write here:_

**The early signal — what to watch for next time:**

> _Write here:_

---

### Dead end 2

**What you did:**

> _Write here:_

**What it cost:**

> _Write here:_

**The early signal — what to watch for next time:**

> _Write here:_

---

*(Add more as needed. If you cannot name an early signal for a dead end, it may not be useful to
carry forward — the goal is actionable hindsight, not a list of what went wrong.)*

---

## Part 3 — Workflow reflections

Workflow reflections are about the process — how spec-first planning and autonomous delegation
worked given the actual shape of this project. Unlike patterns and dead ends, which transfer to
any project, workflow reflections are about calibrating your judgment for future projects: when is
this approach overhead, and when does it pay off?

Work through these questions using your execution and verification records as evidence.

---

**Where did spec-first planning pay off?**

> Name a specific decision that was easier or a problem that was avoided because you had a PRD and
> a task list before you started building. Be concrete — "the PRD prevented scope creep on task 3
> when the agent tried to pull in authentication" is useful; "planning helped" is not.

---

Your answer: ___

---

**Where was spec-first planning overhead for this project's actual size or shape?**

> Was there a part of the process where the planning step cost more than it saved? What was the
> size or type of task where you would have moved faster without it?

---

Your answer: ___

---

**Where did autonomous delegation work well?**

> Name a task or wave of tasks where handing off to Claude and verifying the result was clearly
> the right approach. What made it work — was it the task type, the clarity of the verification
> note, the scope, or something else?

---

Your answer: ___

---

**Where was autonomous delegation overhead or risky?**

> Name a task or moment where you would not hand this off autonomously again, or where you
> hand-held more than you planned. What made it unsuitable — scope ambiguity, missing verification
> signal, credential/data paths, or something else?

---

Your answer: ___

---

**One thing you now understand about this kind of work that you did not at the start:**

> From your pre-work Step 3 answer, refine it into one or two sentences that someone starting a
> similar project could read and act on. Specific is better than general.

---

Your answer: ___

---

## Part 4 — Assemble the carry-forward artifact

The carry-forward artifact is a single document you create from your answers in Parts 1–3. It is
not this worksheet — the worksheet is the working space. The artifact is what you carry forward.

**The artifact is done when it can be read cold.** Open a new document and copy only the finalized
entries from each part. The test: could someone starting a similar project six months from now open
this document and learn something actionable from it, without having been in this project?

**Format for the artifact:**

```
# Carry-Forward: [your project name] — [date]

## Reusable patterns

### [Pattern name]
What it is: ...
When to use it: ...
When not to: ...

## Dead ends

### [Dead end label]
What happened: ...
Early signal: ...

## Workflow reflections
[Paste your refined answers from Part 3]

## Context for next time
[Paste your "one thing you now understand" from Part 3, plus any project-specific
context that the next similar project would benefit from knowing before it starts]
```

**The "Context for next time" section is what distinguishes closing from stopping.** When you
start your next project, open this document first. The context section is your input — not a
blank page.

---

## Done condition

This worksheet is done when:

- [ ] You have at least one pattern with all three elements filled (what it is, where it helped,
      when it would and would not transfer)
- [ ] You have at least one dead end with the early signal named — not just what went wrong,
      but what to watch for earlier next time
- [ ] You have answered the workflow reflections — where the process paid off and where it was
      overhead, with specific evidence from the project
- [ ] You have assembled the carry-forward artifact as a separate document — the worksheet is not
      the artifact; the artifact is what you carry forward into future work
- [ ] The artifact reads coherently without this project's context — a reader starting a similar
      project would find it useful

When every item is checked, you have completed Movements 2 and 3. Take the artifact to the
[completion checklist](./completion-checklist.md).

If you are in a live cohort, bring the artifact to Movement 4 — you will reference it during your
brief presentation.
