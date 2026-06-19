---
title: "Part 2 — Building"
description: Take your execution-ready plan through the three execution modes — monitored sequential, autonomous subagent-driven, and multi-agent dynamic workflows — proving each result with a single deterministic verification command.
---

# Part 2 — Building

This part takes you from an execution-ready plan to a working, verified increment. You will run your
plan through three execution modes in order — starting with close, monitored work and progressing
toward autonomous delegation. Along the way, you will define hard quality gates, practice the
verify-don't-trust discipline, and learn what a gate can and cannot catch.

The project you bring is the project you build. There is no prescribed sample; the exercises run
against your own plan from Part 1.

## Prerequisites

Before starting, confirm:

- **You have an execution-ready plan** — a PRD with measurable success criteria and explicit
  out-of-scope, decomposed into an ordered, individually-verifiable task list. Use the
  [Planning completion checklist](../planning/completion-checklist.md) to confirm before you begin.
  If the plan isn't execution-ready yet, return to Part 1.
- **Your environment is verified** — Claude Code installed and authenticated, Git working, your
  project cloned on an experimentable branch, and git worktrees confirmed working on your machine.
  The [pre-flight worksheet](./pre-flight.md) walks every check.

## What you will produce

By the end of this part you will have:

1. **Defined your quality gate** — the single command that proves your project's passing state
2. **Run at least one task autonomously** — and proven it with that command (not Claude's self-report)
3. **Handed off a multi-task chunk** — and verified each result
4. **A written review standard** — what you inspect before trusting any autonomous output

---

## The three execution modes

The exercises move through three modes in order. Each mode covers more ground per run; each requires
more trust in the quality gate you defined. You earn that trust by proving the gate works — not by
assuming it.

### Mode 1 — Monitored sequential

You stay in the loop for each step: you direct the work, Claude executes one task, you inspect and
verify the result, then you decide what to hand over next. This is the slowest mode and the most
forgiving — every decision is yours, every result is checked before the next task starts.

This mode is how you learn what good autonomous output looks like. Run your first few tasks here
before handing off a full wave.

Exercises 1 and 2 are conversational and can run in the Claude desktop app or the terminal. They
are about plan structure, not execution speed.

### Mode 2 — Autonomous subagent-driven

You hand Claude a single well-framed task — with full context, clear acceptance criteria, and a
named verification step — and let it run end to end without intervening. When it finishes, you run
your verification command and inspect the diff. The agent's self-report is not the evidence; the
verification result is.

This mode is the pivot. The skill being built is **let go, then verify** — not micromanage, not
trust blindly. Exercise 4 is this mode's core practice.

Autonomous runs are terminal-centric: they use git worktrees to isolate work on a throwaway branch,
so a run that goes wrong cannot damage your main branch. Exercise 3 sets that isolation up before
you run anything autonomously.

### Mode 3 — Multi-agent dynamic workflows

You hand off a multi-task chunk — a wave of tasks that can run in parallel or sequence — and let
Claude manage the execution across them. You verify each result as tasks complete.

Claude Code supports parallel subagent dispatch natively: a single session can spawn coordinated
subagents that each work an independent task, report back, and then hand off to the next wave. The
discipline is the same as Mode 2, applied at scale. Exercise 5 introduces this shape using
Claude Code's built-in capabilities — no external orchestration tooling is required.

Going further with dynamic-workflow patterns beyond what a single session supports is optional and
advanced: Claude Code's own documentation is the source of truth for its native multi-agent
coordination capabilities. Those patterns are useful context, but completing the core exercises does
not require them.

---

## Quality gates — the precondition for trusting autonomy

You cannot safely delegate autonomous work without a hard, meaningful quality gate. "Hard" means the
gate is a command with an exit code — it passes or fails, without interpretation. "Meaningful" means
it actually checks something that would catch a real defect.

Before your first autonomous run, you will define your gate and confront two facts the
[quality-gate worksheet](./quality-gates-worksheet.md) walks you through:

1. **Green is not reviewed.** A passing gate proves "tests pass." It does not prove the claimed
   behavior is correct, the coverage is honest, or that real external services work. When the same
   agent wrote both code and tests, the coverage is self-referential — the gate checks that the
   agent's assumptions are internally consistent, not that they match the real world.

2. **Some things a gate cannot catch at all.** Credential and data paths are never verified by
   mocked tests — those paths need a manual smoke test and human review every time. Failure-recovery
   bugs (what happens when the API is down?) are rarely exercised by a unit suite. These are not
   deficiencies to fix; they are the permanent boundary of what automation can verify.

The [Working with Claude page](/docs/working-with-claude) is the source of truth for the
verification and review discipline behind these rules. The worksheet restates the method for your
project; that page owns it.

---

## Exercises

The five exercises run in order. Each builds on the previous one.

| Exercise | Mode | What you do |
|---|---|---|
| [Exercise 1 — Plan to roadmap](./exercises.md#exercise-1--plan-to-roadmap) | Monitored | Decompose your task list into individually-executable work |
| [Exercise 2 — Sequencing and dependencies](./exercises.md#exercise-2--sequencing-and-dependencies) | Monitored | Mark what blocks what; group parallelizable waves |
| [Exercise 3 — Worktree isolation](./exercises.md#exercise-3--worktree-isolation) | Monitored | Create a throwaway workspace so autonomous runs can't damage main |
| [Exercise 4 — First autonomous execution](./exercises.md#exercise-4--first-autonomous-execution) | Autonomous | Hand Claude one task end to end, then verify it |
| [Exercise 5 — Scaling execution](./exercises.md#exercise-5--scaling-execution) | Multi-agent | Hand off a multi-task chunk; verify each result |

Work through the [exercises page](./exercises.md) during the session. Each exercise follows the
same shape: **Goal · You start with · What you do · Done when · If you're stuck.**

---

## Worksheets and materials

| Material | When to use it |
|---|---|
| [Pre-flight worksheet](./pre-flight.md) | Before this part — environment checks before the session |
| [Exercises](./exercises.md) | During the session — the five execution exercises |
| [Execution and verification worksheet](./execution-verification-worksheet.md) | During exercises 4 and 5 — first autonomous run and multi-task chunk |
| [Quality-gate and review worksheet](./quality-gates-worksheet.md) | Before exercise 4 — define your gate, write your review standard |
| [Completion checklist](./completion-checklist.md) | After all exercises — self-attest before moving to Closeout |
| [Homework](./homework.md) | After completing the part — reps to build execution speed |

---

## Completion

When you have worked through all five exercises, complete the
[completion checklist](./completion-checklist.md). Every item is checkable by inspecting your
artifact or the verification output — not by how the run felt.

When the checklist is fully checked, you are ready for **Part 3 — Closeout**.
