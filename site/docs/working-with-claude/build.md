---
sidebar_position: 2
title: Build
description: The Build stage of the lab's SDD lifecycle — implement by delegating to subagents, with a backlog that makes completion git-authoritative.
---

# Build — subagent-driven development

The **Build** stage implements the plan by **delegating to subagents** (helper AI agents), not
hand-coding everything in one session.

- **The plan is the human→agent handoff artifact.** Breaking the design into code-free tasks is
  work you do up front (the [Plan](./plan.md) stage) — don't hand an agent a design spec and expect
  it to break it down too.
- **Subagents discard context and return only their report.** Design each task so the brief is
  sufficient on its own and the returned report is the thing you actually need back.
- **A backlog structures the parallel work:** a task table with stable IDs, an agent-suitability
  classification (which tasks a subagent can own vs. which need you), a dependency graph, and
  **git-authoritative completion** — a task is done when the commit exists, not when an agent says
  so.

For unattended runs — overnight or while you're away — the Build stage has a dedicated safety
contract: see **[Autonomous / overnight loops](./autonomous-loops.md)**.

→ Back to the [lifecycle overview](./index.md).
