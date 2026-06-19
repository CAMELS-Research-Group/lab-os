---
title: Building Completion Checklist
description: Self-attest completion of the Building part — every item is checkable by inspecting your artifact or verification output, not by how the run felt. Pass every check before moving to Closeout.
---

# Building Completion Checklist

Use this checklist after you have worked through all five exercises, the
[execution and verification worksheet](./execution-verification-worksheet.md), and the
[quality-gate and review worksheet](./quality-gates-worksheet.md). Tick each item by inspecting
your artifact or the output from your verification command — not by how the session felt.

**You have completed the Building part when every item below is checked.**

---

## Quality gate and review standard

- [ ] **Your gate command is written down.** You have a single command that runs from your repo root
      in one invocation, returns exit 0 on a passing state, and produces human-readable output. The
      command is recorded in your quality-gate worksheet.

- [ ] **You can state what your gate checks.** You can describe in two or three sentences what
      must be true for the gate to pass — specifically enough that someone else could tell you
      whether the gate is meaningful for your project.

- [ ] **You can state what your gate does not check.** You have named at least one defect class
      that your gate passes on without catching it. "Green is not reviewed" is understood, not
      just stated.

- [ ] **You have marked the defect classes that apply to your project.** At least one of the
      following is checked in your quality-gate worksheet: failure-recovery bugs, credential and
      data paths, integration and wiring gaps, or self-referential coverage.

- [ ] **You have a written review standard.** You can describe what you inspect before accepting
      an autonomous run's output — beyond the gate result.

---

## Autonomous execution — single task

- [ ] **You handed off at least one task autonomously.** You ran Exercise 4: you framed a single
      task with context, a stop condition, and a verification note, and you let Claude run it
      without intervening.

- [ ] **You inspected the diff.** After the run, you read the actual diff output (not Claude's
      summary) and confirmed which files changed and what was added, removed, or modified.

- [ ] **You ran the verification command.** You ran your project's gate command in the worktree
      directory and recorded the exit code.

- [ ] **The result is recorded.** Your execution and verification worksheet has an entry for
      this run: what the diff showed, what the verification command returned, and the result
      (passed / failed / partial).

- [ ] **If the gate failed, you treated it as a success at the skill.** A failed gate that you
      caught is verify-don't-trust working correctly. If your run failed, you recorded the failure
      and noted what it revealed — not what went wrong in the session, but what the verification
      discipline caught.

---

## Multi-task chunk

- [ ] **You handed off a multi-task chunk.** You ran Exercise 5: you grouped two or more tasks
      from your plan, framed them together with context and a stop condition, and let Claude run
      them in sequence or parallel.

- [ ] **You verified each task separately.** For each task in the chunk, you inspected its diff
      and ran the verification command before moving to the next task. Tasks were not accepted
      in bulk.

- [ ] **Each result is recorded.** Your execution and verification worksheet has a per-task result
      entry for this chunk: diff, verification command, exit code, and result.

---

## Understanding — state these without looking

These are the three things the Building part teaches you to carry into all future autonomous work.
You should be able to answer each from memory before moving to Closeout:

- [ ] **Your quality gate** — the exact command, what it checks, and one defect class it misses.

- [ ] **Green is not reviewed** — in one sentence, why a passing gate is not the same as a
      reviewed and trusted output.

- [ ] **One defect class a gate cannot catch** — the specific class most relevant to your project
      (credential/data paths, failure-recovery bugs, integration gaps, or self-referential
      coverage) and what it means for your review practice.

---

## Next step

When every item above is checked: complete the [homework](./homework.md) to build execution speed
between now and Closeout, then move to **Part 3 — Closeout**.

Bring your carry-forward artifact from the homework to Closeout — it is the input to the
learnings-capture and presentation work.
