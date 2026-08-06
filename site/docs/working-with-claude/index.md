---
sidebar_position: 2
title: Working with Claude
description: The lab's spec-driven-development lifecycle — the seven stages we move work through with Claude, from brainstorm to close, with verification and review as load-bearing checkpoints.
---

# Working with Claude

This is the **spec-driven-development (SDD) lifecycle** the lab runs with Claude — discovered in
practice, most of it earned by hitting a failure mode and correcting it. It's owned by the working
group and still evolving: if a stage stops earning its keep, we change it. New members walk the
lifecycle **manually** once — feeling each stage — before working independently.

It's the "how we work with Claude effectively" companion to the hard rules in
[`.claude/rules/`](https://github.com/CAMELS-Research-Group/lab-os/tree/main/.claude/rules); the operating
*philosophy* — PRD (product requirements document) first, pushback, reversibility, review mode —
lives in the
[global `CLAUDE.md` template](https://github.com/CAMELS-Research-Group/lab-os/blob/main/templates/global-CLAUDE.template.md)
you personalized during setup. Several stages lean on the `superpowers` plugin's process skills
(installed in [Getting Started](/docs/getting-started)); where a lab convention conflicts with a
skill's default, the lab convention wins.

---

## The lifecycle at a glance

Work moves through seven stages in order. The two checkpoints — **Verify** and **Review** — are
where work earns the right to advance.

1. **Brainstorm** — shape intent and design before building.
2. **Specify** — write the PRD: what, for whom, success criteria, scope.
3. **[Plan](./plan.md)** — turn the spec into a code-free implementation plan.
4. **[Build](./build.md)** — implement by delegating to subagents (incl. [autonomous loops](./autonomous-loops.md)).
5. **[Verify](./verify.md)** — the **automated checkpoint**: re-run the gate yourself; the agent's word isn't evidence.
6. **[Review](./review.md)** — the **human checkpoint**: an outsider's-eye read of what's actually there.
7. **Close** — log the decisions, checkpoint context, capture durable facts.

**[Communication & Memory](#throughout-communication--memory)** run *throughout* — not a final
step, but discipline that applies at every stage.

---

## Brainstorm

- **Brainstorm before building** (`superpowers:brainstorming`) for any creative work. Process
  skills decide *how* to approach; implementation skills come second.
- **Check what exists first.** Search the lab's repos — including the older projects current work
  descends from (the lineage charted in your dev-root `CLAUDE.md`) — before proposing custom work.
  Extending close-enough beats building new.

## Specify

- **PRD before a non-trivial build.** Problem · Success criteria · Scope · Constraints · Plan ·
  Open questions. No code written until sign-off; silence isn't approval.

## Plan

Turn the spec into a plan that says **what** the implementation must satisfy, not **how** to write
it — the human→agent handoff artifact. Per task: Files · Depends on · Spec link · Acceptance ·
Verification · Commit. **No literal code.** → **[Full details: the Plan stage](./plan.md)**.

## Build

Implement by **delegating to subagents**, not hand-coding everything in one session — the plan is
the brief, the returned report is what you need back. → **[Full details: the Build stage](./build.md)**.
For unattended runs, the safety contract lives in
**[Autonomous / overnight loops](./autonomous-loops.md)**.

## Verify — the automated checkpoint

The single most important lesson: **an agent's self-report is not evidence.** Re-run the gate
yourself, unpiped; a passing gate proves "tests pass," not that the claimed guarantees are tested.
→ **[Full details: the Verify stage](./verify.md)**.

## Review — the human checkpoint

Review catches what the gate cannot — failure-recovery bugs, self-referential coverage gaps,
credential-path assumptions. Pre-merge review is **load-bearing, not optional**. →
**[Full details: the Review stage](./review.md)**.

## Close

- **Continuous capture.** Log decisions and open threads as they happen; load-bearing decisions get
  the *why*. Which log an entry belongs in, what earns one, the format — source of truth:
  [`03-logging.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/03-logging.md).
- **Checkpoint** before switching domains, before a long chat is summarized to free up room
  (Claude Code calls this *compacting*), before a context-heavy subagent handoff.
- **Long-lived facts go to auto-memory** — Claude Code's persistent memory directory. When you
  discover a durable working preference, capture it there so it survives the session.

---

## Throughout: Communication & Memory

These aren't stages — they're discipline that applies at every stage of the lifecycle.

### Communication

- **Overclaim scrub on external-facing writing.** A *dedicated* pass checking every load-bearing
  word against evidence (*foundational, first, only, state-of-the-art, real-time, production-ready,
  shipped*…). Separate what something *does* from what it's *designed to do*; any number needs a
  source.
- **Partners vs prospects.** Don't pitch shared values *at* partners — selling-point language is
  for prospects who haven't bought in. Test: "am I pitching this *recipient* on something we
  already agree on?"

### Memory

Continuous capture and checkpointing apply from the first brainstorm, not just at Close — see the
[Close](#close) stage for where each kind of note belongs.
