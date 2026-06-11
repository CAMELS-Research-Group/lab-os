# lab-rules

Cross-repo conventions for `WatsonWBlair`'s lab repos.

## New to the lab? Start here

**[`BOOTSTRAP.md`](BOOTSTRAP.md)** is the full new-member runbook — workspace layout, the core repos to
clone, wiring lab-rules into Cowork, the two `CLAUDE.md` templates, and the lab plugins. Cross-platform
(Windows reference setup + macOS / Linux equivalents). Follow it end-to-end for a working environment.

The rest of this README is reference for how the conventions in this repo are consumed.

## What's here

- `.claude/rules/` — markdown files defining lab-wide conventions. Consumed by Cowork locally (via a junction/symlink at `<DEV_ROOT>\.claude\rules\`) and by the PR-review GitHub Action at review time.
  - [`01-workflow.md`](.claude/rules/01-workflow.md) — commit format, PR workflow, merge bar, doc-update triggers
  - [`02-data-protection.md`](.claude/rules/02-data-protection.md) — gated-dataset, PII, and binary/secret protection
  - [`03-logging.md`](.claude/rules/03-logging.md) — project-log standard (altitudes, entry triggers, format, immutability, overflow)
  - [`04-docs.md`](.claude/rules/04-docs.md) — documentation standard (single-source, tiers, byte budgets, ENG doc standards, rules numbering)
- [`PR-LIFECYCLE.md`](PR-LIFECYCLE.md) — end-to-end PR lifecycle: merge bar, solo-maintainer bypass, pre-merge log cleanup.
- [`TROUBLESHOOTING.md`](TROUBLESHOOTING.md) — lab-level expensive findings and gotchas, indexed by symptom.
- `templates/` — starter files for new repos and members:
  - [`global-CLAUDE.template.md`](templates/global-CLAUDE.template.md) — personal-global persona + lab operating philosophy (→ `~/.claude/CLAUDE.md`)
  - [`dev-root-CLAUDE.template.md`](templates/dev-root-CLAUDE.template.md) — genericized lab orientation (→ `<DEV_ROOT>/.claude/CLAUDE.md`)
  - [`repo-CLAUDE.template.md`](templates/repo-CLAUDE.template.md) — per-repo CLAUDE.md seed (rules pointer, gate command, gated datasets)
  - [`project_log.template.md`](templates/project_log.template.md) — normative project-log structure (parsed by `log-lint`)
  - [`PRD.template.md`](templates/PRD.template.md) — PRD scaffold (Problem · Success criteria · Scope · Constraints · Plan · Open questions)
  - [`templates/work-bundle/`](templates/work-bundle/) — design + plan templates for initiative-level work bundles
- `.github/workflows/` — adherence Actions (consume via a thin caller; see [`standards.yml`](.github/workflows/standards.yml) as the copyable example):
  - [`log-lint.yml`](.github/workflows/log-lint.yml) — validates project-log structure and entry format
  - [`docs-budget.yml`](.github/workflows/docs-budget.yml) — warns when CLAUDE.md or rules files exceed byte budgets
  - [`merge-bar-check.yml`](.github/workflows/merge-bar-check.yml) — checklist completeness and log-cleanup gate
- `BOOTSTRAP.md` — the setup runbook that ties it together.
- `WORKING-WITH-CLAUDE.md` — established lab methods and best practices for working with Claude (read during onboarding).

## Lab plugins

The lab's Claude Code plugins live in [`lab-claude-plugins`](https://github.com/WatsonWBlair/lab-claude-plugins):

```
/plugin marketplace add WatsonWBlair/lab-claude-plugins
/plugin install pr-review-loop@lab-claude-plugins
```

## How repos consume it

**Locally (Cowork)**: clone this repo into your lab workspace (`<DEV_ROOT>`) and link its rules so Cowork sees them at the workspace root. BOOTSTRAP.md §3 is the full walkthrough; the short form:

```powershell
# Windows (PowerShell) — junction, no admin required
cmd /c mklink /J "<DEV_ROOT>\.claude\rules" "<DEV_ROOT>\lab-rules\.claude\rules"
```

```bash
# macOS / Linux — symlink
ln -s <DEV_ROOT>/lab-rules/.claude/rules <DEV_ROOT>/.claude/rules
```

**In CI (PR reviewer)**: each lab repo's `.github/workflows/pr-review.yml` checks this repo out alongside the PR repo:

```yaml
- uses: actions/checkout@v4
  with: { path: pr-repo }
- uses: actions/checkout@v4
  with:
    repository: WatsonWBlair/lab-rules
    path: lab-rules
```

The reviewer then concatenates `lab-rules/.claude/rules/*.md` + `pr-repo/.claude/rules/*.md` into its prompt context.

**Adherence Actions**: repos also consume the three enforcement workflows by adding a thin caller that references them from this repo. [`standards.yml`](.github/workflows/standards.yml) is the copyable example.

## Override semantics

Per-repo rules extend or override lab rules. Specific wins over general. Per-repo rules number from `10+` (lab-rules owns `0x-*`): a per-repo `10-data-protection.md` listing the repo's specific gated datasets supplements the lab-wide PII checklist; a per-repo rule contradicting a lab rule applies only in that repo.

## Scope discipline

This repo holds **hard rules** — commit format, PR template usage, data-protection invariants, security, approval gates. Soft conventions (code style, library preferences) stay per-repo. Audit periodically to keep this lean — every file here is loaded into every Cowork session and every PR review.
