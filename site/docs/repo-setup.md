---
sidebar_position: 5
title: Setting Up a New Repo
description: First-cut runbook for creating a repository that follows lab conventions — gitignore, CLAUDE.md seed, project-log skeleton, PR template, and the phase-2 CI pointer.
---

# Setting Up a New Repo

A first-cut checklist for creating a repository that conforms to lab conventions from commit one.
The [lab-os templates](https://github.com/CAMELS-Research-Group/lab-os/tree/main/templates) are its building
blocks — each step links its source of truth rather than restating it.

**This runbook is deliberately first-cut.** It is the first step of the
[onboarding project](/docs/onboarding-project) — the sandbox build exercises it — and it is expected
to grow from the friction data that play-test produces. Hit a gap or a confusing step? That's a
finding; capture it.

## Checklist

### 1. Create the repo

Create the repository on GitHub (under your own account for the onboarding sandbox; under the lab
for real work) and clone it into your `<DEV_ROOT>` alongside the other lab repos (see
[Getting Started](/docs/getting-started) for the workspace layout).

### 2. Add `.gitignore` basics

No lab template yet — start from a standard `.gitignore` for your stack (the file listing what
git must never commit; GitHub's repo-creation flow
offers one, or pull from [github/gitignore](https://github.com/github/gitignore)), then make sure
it covers the lab's protection rules
([`02-data-protection.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/02-data-protection.md)):

- `.env` and other secret-bearing files
- model artifacts and checkpoints (`.pt`, `.npy`, `.task`, `.onnx`, `.bin`, `.safetensors`)
- anything derived from gated datasets

### 3. Seed `CLAUDE.md`

Copy
[`templates/repo-CLAUDE.template.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/templates/repo-CLAUDE.template.md)
to the repo root (or `.claude/CLAUDE.md`) and fill the placeholders. Follow the template's embedded
author guidance — dense writing aimed at an AI reader, 8 KB size budget, no restating lab rules that already load.

### 4. Seed `project_log.md`

Copy
[`templates/project_log.template.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/templates/project_log.template.md)
to the repo root. Its structure is load-bearing (the `log-lint` automated check parses it) — keep the `## Standing Decisions`
and `## Entries` headings exactly as shipped. Entry format and triggers:
[`03-logging.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/03-logging.md).

### 5. Add the PR template

Copy lab-os's
[`.github/pull_request_template.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.github/pull_request_template.md)
into your repo's `.github/` as a starting point, adjusting checklist items to the repo. Every PR
fills it — see
[`01-workflow.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/01-workflow.md).

### 6. Caller YAML pointer — *phase 2, not yet required*

Eventually each repo will carry a small workflow file that runs lab-os's shared CI checks
(automated pull-request checks: log-lint, docs-budget, merge-bar-check). That rollout hasn't happened —
skip this step for now; it's listed so the checklist doesn't silently grow later.

## Done when

- Repo exists and is cloned under `<DEV_ROOT>`
- `.gitignore` covers secrets and binary/model artifacts
- `CLAUDE.md` seeded, no `<...>` placeholders left
- `project_log.md` seeded with the required skeleton intact
- `.github/pull_request_template.md` in place

Then make your first commit following the conventions in
[Rules, Explained](/docs/rules-explained) — and if any step here fought you, log it for the
play-test retro.
