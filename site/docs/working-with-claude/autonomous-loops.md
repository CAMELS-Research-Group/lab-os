---
sidebar_position: 3
title: Autonomous loops
description: A Build-stage deep dive — the safety contract for autonomous and overnight Claude loops, so you wake to a verified increment or a clean halt, never an unchecked feature.
---

# Autonomous / overnight loops

A deep dive on the [Build](./build.md) stage: running Claude unattended — overnight or while you're
away. The target is to **wake to either an increment that passed the verification gate or a clean
halt plus an actionable digest (summary report)** — never "wake to a finished feature you haven't
checked."

- **Halt contract.** Give the loop an explicit escape hatch: halt and report, don't press on.
  Phrase the completion signal as *digest-written* (true at done OR documented halt), not
  task-success — otherwise a stuck agent churns to the iteration cap.
- **Budget caps + wall-clock kill.** A session **cannot** read its own subscription-quota usage —
  "stop at X% of quota" is not buildable. Use a hard iteration cap plus a wall-clock kill (stop
  after a set time, no matter what), and cap conservatively on early runs.
- **Test the halt path before trusting a run** — a run that ended naturally never exercised its
  escape hatch (missing dependencies have silently broken the stop mechanism).
- **Forbid hard-to-reverse ops in the mandate.** No history-rewriting git operations (rebase,
  amend, force-push) inside a loop — halt-and-report instead. Start branches from the correct base.
- **Human-gated authorization is correct, not a nuisance.** Launching a loop trips Claude Code's
  safety checks by design; the agent must not retry or route around the denial.

→ Back to [Build](./build.md) · [lifecycle overview](./index.md).
