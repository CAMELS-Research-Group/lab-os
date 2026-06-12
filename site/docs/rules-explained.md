---
sidebar_position: 4
title: The Rules, Explained
description: A human tour of the four lab-wide rule files — what each governs, why it exists, and what a new member should internalize first.
---

# The Rules, Explained

lab-os ships four lab-wide rule files under
[`.claude/rules/`](https://github.com/WatsonWBlair/lab-os/tree/main/.claude/rules). Every Claude session
loads them automatically through the junction/symlink you wired in
[Getting Started](/docs/getting-started), and the PR-review tooling reads the same files at review time —
so one `git pull` of lab-os updates the rules everywhere at once. Parts of the rules are also enforced in
CI by three adherence actions — `log-lint`, `docs-budget`, and `merge-bar-check` (a tooling tour covers
them separately).

This page is orientation, not reference. Each section restates a rule in general terms and links the rule
file itself as the **source of truth** — when the two disagree, the rule file wins.

## 01 — Workflow

Source of truth:
[`01-workflow.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/01-workflow.md)

Governs how work moves: commit-message conventions, the PR workflow, the bar a PR must clear to merge,
and the triggers that tell you which docs to update alongside a change. It exists so history stays
readable across every lab repo and so "done" means the same thing everywhere — verified, reviewed,
logged, and documented, not just merged.

Internalize two things:

- **Commit conventions live here.** The lab uses conventional-commit-style prefixes with a short,
  lowercase, present-tense subject. When unsure which prefix applies, check the file's table and
  tie-break notes rather than guessing.
- **The merge bar is a checklist, not a vibe.** Run the repo's verification gate yourself (un-piped, so a
  failure can't hide behind a pipe), and treat documentation drift as a blocker — if the docs no longer
  match the code, the work isn't finished.

## 02 — Data Protection

Source of truth:
[`02-data-protection.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/02-data-protection.md)

Governs what may never enter a repo: raw content from the lab's gated research datasets, anything that
could re-identify a participant, large binaries and model artifacts, and secrets. It exists because the
lab works with license-gated human-subject data — one careless commit can violate a data-use agreement
and damage the lab's credibility permanently, and git history makes such commits effectively permanent.

Internalize two things:

- **Raw gated data never gets committed** — not the recordings, not the text, not anything identifying
  the people in them, in any repo, ever. Each repo declares its own gated datasets in its local rules.
- **Derived artifacts aren't automatically safe.** Plots, embeddings, and summary outputs go through an
  explicit PII review (the rule file has the checklist) before commit. The default when uncertain is to
  aggregate further or leave the artifact out.

## 03 — Logging

Source of truth:
[`03-logging.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/03-logging.md)

Governs project logs: which of the three log levels (lab-wide, per-repo, per-plan) an event belongs to,
what earns an entry at all, the entry format, and the immutability rules. It exists so a future session —
human or agent — can pick up a project cold and re-derive *why* things are the way they are, without the
logs bloating into diaries.

Internalize two things:

- **Log decisions, not activity.** An entry is warranted when a decision carried real weight (reversing
  it would change direction), when something irreversible or externally visible happened, or when the
  project changed course. Routine status, findings,
  and follow-ups have other homes (PR comments, issues, the plan's execution log) — the rule file maps
  each kind of information to its home.
- **Entries are immutable once merged.** Reversing a decision means writing a new entry that supersedes
  the old one — never editing history.

## 04 — Docs

Source of truth:
[`04-docs.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/04-docs.md)

Governs the documentation system itself: every fact has exactly one owning document, docs are tiered by
reader (agent-facing, engineering, public-facing) with size budgets on the always-loaded tier, and the
core engineering documents (PRD, design doc, plan) have defined shapes. It exists because duplicated
facts drift apart silently — and because agent-loaded docs cost context on every session, so they have to
stay dense and bounded.

Internalize two things:

- **One owner per fact.** Before restating something a spec, README, or rule already states, link to it
  (or clearly derive from it, naming the source) instead of copying. If you're asking "which doc owns
  this fact?", this rule file is where the answer's logic lives.
- **Write for the tier.** Agent docs are dense and budget-capped; engineering docs are skimmable with
  explicit contracts; public docs carry no internal jargon or codenames. Know which one you're writing
  before you start.

## Quick answers

| Question | Rule file |
|---|---|
| Where do commit conventions live? | [`01-workflow.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/01-workflow.md) |
| What can I never commit? | [`02-data-protection.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/02-data-protection.md) |
| When do I write a log entry? | [`03-logging.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/03-logging.md) |
| Which doc owns a fact? | [`04-docs.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/04-docs.md) |
