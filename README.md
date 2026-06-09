# lab-rules

Cross-repo conventions for `WatsonWBlair`'s lab repos.

## New to the lab? Start here

**[`BOOTSTRAP.md`](BOOTSTRAP.md)** is the full new-member runbook — workspace layout, the core repos to
clone, wiring lab-rules into Cowork, the two `CLAUDE.md` templates, and the lab plugins. Cross-platform
(Windows reference setup + macOS / Linux equivalents). Follow it end-to-end for a working environment.

The rest of this README is reference for how the conventions in this repo are consumed.

## What's here

- `.claude/rules/` — markdown files defining lab-wide conventions. Consumed by Cowork locally (via a junction/symlink at `<DEV_ROOT>\.claude\rules\`) and by the PR-review GitHub Action at review time.
- `templates/` — starter `CLAUDE.md` files for new members:
  - `global-CLAUDE.template.md` — personal-global persona + lab operating philosophy (→ `~/.claude/CLAUDE.md`)
  - `dev-root-CLAUDE.template.md` — genericized lab orientation (→ `<DEV_ROOT>/.claude/CLAUDE.md`)
- `BOOTSTRAP.md` — the setup runbook that ties it together.
- `WORKING-WITH-CLAUDE.md` — established lab methods and best practices for working with Claude (read during onboarding).

## Lab plugins

The lab's Claude Code plugins live in [`lab-claude-plugins`](https://github.com/WatsonWBlair/lab-claude-plugins):

```
/plugin marketplace add WatsonWBlair/lab-claude-plugins
/plugin install pr-review-loop@lab-claude-plugins
```

## How repos consume it

**Locally (Cowork)**: clone this repo to `C:\Users\watso\Development\lab-rules\`. Create a Windows junction so Cowork sees it at the Development root:

```powershell
cmd /c mklink /J "C:\Users\watso\Development\.claude\rules" "C:\Users\watso\Development\lab-rules\.claude\rules"
```

No admin required.

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

## Override semantics

Per-repo rules extend or override lab rules. Specific wins over general. A per-repo `02-data-protection.md` listing the repo's specific gated datasets supplements the lab-wide PII checklist; a per-repo rule contradicting a lab rule applies only in that repo.

## Scope discipline

This repo holds **hard rules** — commit format, PR template usage, data-protection invariants, security, approval gates. Soft conventions (code style, library preferences) stay per-repo. Audit periodically to keep this lean — every file here is loaded into every Cowork session and every PR review.
