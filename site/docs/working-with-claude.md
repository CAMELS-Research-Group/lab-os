---
sidebar_position: 2
title: Working with Claude
description: The lab's established methods for working with Claude effectively — process-first workflows, code-free plans, verification and review discipline, and autonomous-loop safety.
---

# Working with Claude — lab methods and best practices

Methods the lab established **in practice** — most earned by hitting a failure mode and correcting
it. This is the "how we work with Claude effectively" companion to the hard rules in
[`.claude/rules/`](https://github.com/WatsonWBlair/lab-os/tree/main/.claude/rules); the operating
*philosophy* (PRD-first, pushback, reversibility, review mode) lives in the
[global `CLAUDE.md` template](https://github.com/WatsonWBlair/lab-os/blob/main/templates/global-CLAUDE.template.md)
you personalized during setup. Several methods lean on the `superpowers` plugin's process skills
(installed in [Getting Started](/docs/getting-started)); where a lab convention conflicts with a
skill's default, the lab convention wins.

---

## 1. Process before code

- **Brainstorm before building** (`superpowers:brainstorming`) for any creative work. Process
  skills decide *how* to approach; implementation skills come second.
- **PRD before a non-trivial build.** Problem · Success criteria · Scope · Constraints · Plan ·
  Open questions. No keys touched until sign-off; silence isn't approval.
- **Check what exists first.** Search the lab's repos — including upstream in the research lineage
  (charted in your dev-root `CLAUDE.md`) — before proposing custom work. Extending close-enough
  beats building new.

## 2. Code-free implementation plans

Plans specify **what** the implementation must satisfy, not **how** to write it. Per task, six
elements: **Files** · **Depends on** · **Spec** (link) · **Acceptance** (bulleted behaviors) ·
**Verification** (the exact command that confirms done) · **Commit** (subject).

**No literal code, no test code, no TDD walkthroughs.** The behaviors *are* the test surface; the
implementing agent owns names, fixtures, and assertions. Only code-blocks allowed: short shell
commands in `**Verification:**` lines. Code-heavy plans rot faster and discourage the implementer
from owning their tests; contract-and-behavior plans survive re-runs. This **overrides**
`superpowers:writing-plans`' show-code default. Source of truth:
[`04-docs.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/04-docs.md).

## 3. Subagent-driven development

- **The plan is the human→agent handoff artifact.** Decomposition into code-free tasks is *daytime*
  work — don't hand an agent a design spec and expect it to also decompose.
- **Subagents discard context and return only their report.** Design tasks so the brief is
  sufficient and the returned report is the thing you actually need.
- **A backlog scaffolds the fan-out:** a task table with stable IDs, an agent-suitability
  classification, a dependency DAG, and **git-authoritative completion** — a task is done when the
  commit exists, not when an agent says so.

## 4. Verification discipline

The single most important lesson: **an agent's self-report is not evidence.**

- **Beware the optimistic narrator.** Agents report success they didn't achieve. Re-run the gate
  yourself (the repo's designated verification command — source of truth:
  [`01-workflow.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/01-workflow.md),
  Merge Bar); don't trust the digest.
- **Green ≠ reviewed.** A passing gate proves "tests pass," not that the claimed guarantees are
  tested — when the same agent wrote both code and tests, coverage is self-referential.
- **Run the gate unpiped.** Piping (`gate | tail`) swallows the exit code; an agent has committed
  on a red gate because the pipe masked it.
- **Credential / data paths are never gate-verified.** Mocked tests don't touch live OAuth / APIs —
  those need a manual smoke test and human review before merge, every time.
- Use `superpowers:verification-before-completion` before claiming anything is done.

## 5. Review discipline

- **Multi-agent first pass + audit pass.** Cheaper model first; escalate to the stronger model both
  when the first pass returns *zero* findings (rubber-stamp risk) and to recheck severities. The
  audit pass has caught real latent bugs the gate could not see.
- **Review catches what the gate cannot** — rollback/atomicity bugs, self-referential coverage
  gaps, credential-path assumptions. Pre-merge review is **load-bearing, not optional**.
- **Outsider's eye.** Review what's there, not what's meant. If you helped author it, declare that
  and harden the review.
- **For a requested review, the review *is* the deliverable** — post it; don't ask permission to
  deliver what was asked. Unsolicited posts under your name still need the approval gate in the
  [global `CLAUDE.md` template](https://github.com/WatsonWBlair/lab-os/blob/main/templates/global-CLAUDE.template.md).
- **PR template, merge bar, solo-maintainer bypass:**
  [`PR-LIFECYCLE.md`](https://github.com/WatsonWBlair/lab-os/blob/main/PR-LIFECYCLE.md) for the
  lifecycle; hard rules in
  [`01-workflow.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/01-workflow.md).

## 6. Autonomous / overnight loops

Target: **wake to either a gate-green increment or a clean halt plus an actionable digest** —
never "wake to a finished feature you haven't checked."

- **Halt contract.** Explicit escape hatch: halt and report, don't press on. Phrase the completion
  signal as *digest-written* (true at done OR documented halt), not task-success — otherwise a
  stuck agent churns to the iteration cap.
- **Budget caps + wall-clock kill.** A session **cannot** read its own subscription-quota usage —
  "stop at X% of quota" is not buildable. Use a hard iteration cap plus a wall-clock kill, and cap
  conservatively on early runs.
- **Test the halt path before trusting a run** — a run that ended naturally never exercised its
  escape hatch (missing dependencies have silently broken stop-hooks).
- **Forbid hard-to-reverse ops in the mandate.** No rebase / amend / force-push / history rewrites
  inside a loop — halt-and-report instead.
- **Human-gated authorization is correct, not a nuisance.** Launching a loop trips Claude Code's
  safety checks by design; the agent must not retry or route around the denial.

## 7. Communication discipline

- **Overclaim scrub on external-facing writing.** A *dedicated* pass checking every load-bearing
  word against evidence (*foundational, first, only, state-of-the-art, real-time, production-ready,
  shipped*…). Separate what something *does* from what it's *designed to do*; any number needs a
  source.
- **Partners vs prospects.** Don't pitch shared values *at* partners — selling-point language is
  for prospects who haven't bought in. Test: "am I pitching this *recipient* on something we
  already agree on?"

## 8. Memory and note-taking

- **Continuous capture.** Log decisions and open threads as they happen; load-bearing decisions get
  the *why*. Altitudes, routing, format — source of truth:
  [`03-logging.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/03-logging.md).
- **Checkpoint** before switching domains, before a long chat compacts, before a context-heavy
  subagent handoff.
- **Long-lived facts go to auto-memory** — Claude Code's persistent memory directory. When you
  discover a durable working preference, capture it there so it survives the session.
