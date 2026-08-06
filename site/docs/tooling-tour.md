---
sidebar_position: 7
title: Tooling Tour
description: The automation lab-os runs on itself — three CI adherence checks, the templates directory, an automated PR reviewer, and a Claude Code plugin marketplace.
---

# Tooling Tour

This page describes the real tooling this repository runs on itself — an example of what your own
workspace can grow into once the conventions from the rest of this handbook are in place. Two
audiences, one answer each: if you're assessing the lab, this is the infrastructure it runs; if
you're joining it, this is what will act on your pull requests. Every section links its source of
truth —
when this page and the source disagree, the source wins.

## The three CI adherence checks

CI (continuous integration — the checks GitHub runs automatically on every pull request, or PR) is how
this repository enforces its own standards. The three checks are defined once in lab-os as
*reusable workflows* — GitHub Actions' way of letting many repositories share one check — in
[`.github/workflows/`](https://github.com/CAMELS-Research-Group/lab-os/tree/main/.github/workflows). lab-os
runs them on its own pull requests through a small *caller* file — a workflow whose only job is to
invoke the shared ones —
[`standards.yml`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.github/workflows/standards.yml);
any other repo can adopt all three by copying that one file — a later rollout step, marked
not-yet-required in [Setting Up a New Repo](/docs/repo-setup) (the
[README](https://github.com/CAMELS-Research-Group/lab-os/blob/main/README.md) documents adoption under
"How repos consume it").

### log-lint

Source of truth:
[`log-lint.yml`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.github/workflows/log-lint.yml)
(behavior is spelled out in its header comment).

Checks PR changes to `project_log.md` (and its archive) against the
[logging standard](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/03-logging.md):
entry format, ordering, immutability of merged entries, archive integrity, per-entry size.

**A red check means** a new or changed log entry breaks the standard — or the PR carries the
`log-lint:override` label without stating a reason in the body. Fix the entry (or state the
override reason) and push; it isn't a flaky check.

<details>
<summary>When it stays green without doing anything</summary>

- A PR that doesn't touch the log at all passes without linting.
- The `log-lint:override` label (with a stated reason) skips enforcement — meant for migrations
  and merged-entry corrections.
- The whole-file 15 KB log cap is **docs-budget's** job, not log-lint's; log-lint only budgets
  individual new entries.

</details>

### docs-budget

Source of truth:
[`docs-budget.yml`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.github/workflows/docs-budget.yml).

Checks byte budgets on the always-loaded files agents read every session — `CLAUDE.md`, each
`.claude/rules/*.md` file, and `project_log.md` — so those files don't quietly grow past what an
agent can usefully start with. Budgets and tiers:
[`04-docs.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/04-docs.md).

**A red check means** one of those files has grown past 1.5× its budget — and only in repos that have
opted into enforcement. The default posture is warn-only: the job annotates overages but never
fails, until a repo flips `enforce: true` after its first green run. lab-os itself enforces.

### merge-bar-check

Source of truth:
[`merge-bar-check.yml`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.github/workflows/merge-bar-check.yml).

Verifies the PR description against the repo's PR template: every required section heading must be
present, and on PRs that change code exactly one of the two log checkboxes ("log entries finalized" /
"no loggable events") must be ticked. This check automates part of the full merge bar — the
checklist a change must clear before merging — defined in
[`01-workflow.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/01-workflow.md)
and discussed in [Rules, Explained](/docs/rules-explained).

**A red check means** your PR body is incomplete — a missing template section, or the log
checkboxes unticked (or both ticked) on a PR that changes code. Edit the PR description; no code
change needed. Docs-only PRs skip the checkbox rule (sections are still required).

## The `templates/` directory

[`templates/`](https://github.com/CAMELS-Research-Group/lab-os/tree/main/templates) holds the seed files
the handbook's setup and repo-creation pages copy from — each carries its own embedded author
guidance:

- [`global-CLAUDE.template.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/templates/global-CLAUDE.template.md)
  — your personal `~/.claude/CLAUDE.md`: operating philosophy that loads in every session.
- [`dev-root-CLAUDE.template.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/templates/dev-root-CLAUDE.template.md)
  — workspace-root orientation: which repos exist and how they relate.
- [`repo-CLAUDE.template.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/templates/repo-CLAUDE.template.md)
  — per-repo `CLAUDE.md` seed, kept short to a byte budget (see
  [Rules, Explained](/docs/rules-explained) — 04, Docs).
- [`project_log.template.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/templates/project_log.template.md)
  — the project-log skeleton (log-lint parses this exact structure, so its load-bearing headings
  are never renamed — see [Setting Up a New Repo](/docs/repo-setup)).
- [`PRD.template.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/templates/PRD.template.md)
  — the living PRD (product requirements document) shape: Problem, success criteria, scope, constraints, plan, open questions.
- [`work-bundle/`](https://github.com/CAMELS-Research-Group/lab-os/tree/main/templates/work-bundle)
  — paired design + plan templates for a single unit of work, kept together and filed away as
  one package when that work ships.

## The PR-review agent

An automated reviewer that gives open PRs on lab repos an external-perspective code review and
posts its findings as review comments. It posts via a bot identity (a GitHub App), not as a human
account, and sends run notifications to a Discord channel. It reads the same
[lab rules](https://github.com/CAMELS-Research-Group/lab-os/tree/main/.claude/rules) this repo publishes,
so the automated reviewer and human reviewers work from the same written standard.

Scope worth being precise about: it **reviews and comments — it does not approve or merge**.
A human still owns every merge decision.

Its full specification lives in the agent's own repository, which is private; this description is
deliberately self-contained so nothing here depends on access to it.

## The plugin marketplace

The lab publishes its Claude Code plugins through a public marketplace repo,
[lab-claude-plugins](https://github.com/WatsonWBlair/lab-claude-plugins). Inside Claude Code:

```
/plugin marketplace add WatsonWBlair/lab-claude-plugins
/plugin install pr-review-loop@lab-claude-plugins
```

[Getting Started](/docs/getting-started) walks through this (plus the `superpowers` plugin from
the official marketplace) as part of workspace setup — start there rather than here if you're
installing for the first time.
