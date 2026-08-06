---
sidebar_position: 4
title: The Rules, Explained
description: A human tour of the four lab-wide rule files — what each governs, why it exists, and what a new member should internalize first.
---

# The Rules, Explained

lab-os ships four lab-wide rule files under
[`.claude/rules/`](https://github.com/CAMELS-Research-Group/lab-os/tree/main/.claude/rules). Every Claude session
loads them through the link you wired in [Getting Started](/docs/getting-started), the pull-request
(PR) review tooling reads the same files at review time, and parts are enforced in CI — continuous
integration, the checks run automatically on every PR (`log-lint`, `docs-budget`,
`merge-bar-check` — toured in [Tooling Tour](/docs/tooling-tour)).

This page is orientation, not reference — each section links the rule file as the **source of truth**;
when the two disagree, the rule file wins.

## 01 — Workflow

Source of truth:
[`01-workflow.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/01-workflow.md)

Governs how work moves: commit conventions, the PR workflow, the merge bar (the checklist a change
must clear before merging), and doc-update triggers.
It exists so history stays readable and "done" means the same thing everywhere — verified, reviewed,
logged, documented.

Internalize two things:

- **Commit conventions live here.** Every commit message starts with a type prefix (`feat:`,
  `fix:`, …) plus a short lowercase present-tense subject. Unsure which prefix? Check the file's
  table and tie-break notes, don't guess.
- **The merge bar is a checklist, not a vibe.** Run the repo's verification gate (its designated
  check command) yourself, directly rather than piped through another command so a failure can't
  hide; docs out of sync with the code = not done.

## 02 — Data Protection

Source of truth:
[`02-data-protection.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/02-data-protection.md)

The lab works with gated (license-restricted) human-subject datasets. This rule governs what may
never enter a repo: raw content from those datasets, anything that could re-identify a participant,
large binaries, secrets — one careless commit can violate a data-use agreement, and git history
makes it effectively permanent.

Internalize two things:

- **Raw gated data never gets committed** — recordings, text, anything identifying the people in
  them, in any repo, ever. Which datasets count as gated is defined per-repo.
- **Derived artifacts aren't automatically safe.** Plots, embeddings, and summaries go through the
  rule file's review checklist for personally identifying information (PII) before commit. When uncertain, aggregate further or leave the
  artifact out.

## 03 — Logging

Source of truth:
[`03-logging.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/03-logging.md)

Governs project logs: which of the three levels (lab-wide, per-repo, per-plan) an event belongs to,
what earns an entry, the format, and immutability. It exists so a future session — human or agent —
can pick up a project cold and re-derive *why*, without logs bloating into diaries.

Internalize two things:

- **Log decisions, not activity.** An entry is warranted for a weighty decision, an irreversible or
  external event, or a change of course. Routine status, findings, and follow-ups have other homes —
  the rule file maps each kind of information to its home.
- **Entries are immutable once merged.** Reversing a decision means a new superseding entry — never
  editing history.

## 04 — Docs

Source of truth:
[`04-docs.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/04-docs.md)

Governs the documentation system itself: one owning document per fact, docs tiered by reader
(agent / engineering / public) with size budgets on the always-loaded tier, and defined shapes for
PRDs (product requirements documents), design docs, and plans. Duplicated facts drift apart
silently, and docs the AI loads every session consume its limited working context.

Internalize two things:

- **One owner per fact.** Before restating something a spec, README, or rule already states, link to
  it (or derive from it, naming the source) instead of copying.
- **Write for the tier.** Agent docs are dense and budget-capped; engineering docs skimmable with
  explicit contracts; public docs free of internal jargon and codenames.

## Quick answers

| Question | Rule file |
|---|---|
| Where do commit conventions live? | [`01-workflow.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/01-workflow.md) |
| What can I never commit? | [`02-data-protection.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/02-data-protection.md) |
| When do I write a log entry? | [`03-logging.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/03-logging.md) |
| Which doc owns a fact? | [`04-docs.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/04-docs.md) |
