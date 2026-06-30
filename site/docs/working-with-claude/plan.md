---
sidebar_position: 1
title: Plan
description: The Plan stage of the lab's SDD lifecycle — code-free implementation plans that specify what the implementation must satisfy, not how to write it.
---

# Plan — code-free implementation plans

The **Plan** stage turns a signed-off spec into the human→agent handoff artifact: a plan that
specifies **what** the implementation must satisfy, not **how** to write it. Breaking the design
into tasks is work you do up front — don't hand an agent a design spec and expect it to break the
work down too.

Per task, seven elements:

- **Files** — exact create/modify paths.
- **Depends on** — the task numbers that must land first.
- **Spec** — a link to the design-doc section this task implements.
- **Acceptance** — the bulleted behaviors the implementation must demonstrate.
- **Verification** — the exact command that confirms the task is done.
- **Agent-suitable** — `yes` / `partial` / `no`: can an agent run this task unattended?
- **Commit** — the conventional-commit subject.

**No literal code, no test code, no step-by-step test-first (TDD) walkthroughs.** The behaviors
*are* the test surface; the implementing agent owns function names, test data, and assertions. The
only code-blocks allowed are short shell commands in `**Verification:**` lines.

Code-heavy plans rot faster and discourage the implementer from owning their tests;
contract-and-behavior plans survive re-runs. This **overrides** `superpowers:writing-plans`'
show-code default. Source of truth:
[`04-docs.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/04-docs.md).

→ Back to the [lifecycle overview](./index.md).
