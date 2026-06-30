---
sidebar_position: 4
title: Verify
description: The Verify stage of the lab's SDD lifecycle — the automated checkpoint, where you re-run the gate yourself because an agent's self-report is not evidence.
---

# Verify — the automated checkpoint

**Verify** is the lifecycle's *automated* checkpoint. The single most important lesson behind it:
**an agent's self-report is not evidence.** That failure mode is *why* this stage exists — the gate
is the evidence the narration isn't.

- **Beware the optimistic narrator.** Agents report success they didn't achieve. Re-run the gate
  yourself (the repo's designated verification command — source of truth:
  [`01-workflow.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/01-workflow.md),
  Merge Bar); don't trust the agent's summary.
- **Green ≠ reviewed.** A passing gate proves "tests pass," not that the claimed guarantees are
  tested — when the same agent wrote both code and tests, coverage is self-referential. (Catching
  that gap is the [Review](./review.md) stage's job.)
- **Run the gate unpiped.** Piping (`gate | tail`) hides the command's pass/fail signal (its exit
  code); an agent has committed on a failing gate because the pipe masked it.
- **Credential / data paths are never gate-verified.** Tests with simulated ("mocked") services
  never touch live sign-in or external APIs — those need a manual smoke test (a quick real
  run-through) and human review before merge, every time.
- Use `superpowers:verification-before-completion` before claiming anything is done.

→ Next: the [Review](./review.md) stage · back to the [lifecycle overview](./index.md).
