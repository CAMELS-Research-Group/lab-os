# lab-os

Cross-repo conventions for `WatsonWBlair`'s lab repos.

**New to the lab? Start at the handbook: <https://camels-research-group.github.io/lab-os/>** (source: `site/`).
The site owns the human-facing docs — setup runbook, working-with-Claude methods, the onboarding
project, the rules tour. This README is reference for how the conventions in this repo are consumed
by agents and CI.

## What's here

- `.claude/rules/` — markdown files defining lab-wide conventions. Consumed by Cowork locally (natively in a lab-os fork, or via a junction/symlink in the multi-repo setup) and by the PR-review GitHub Action at review time.
  - [`01-workflow.md`](.claude/rules/01-workflow.md) — commit format, PR workflow, merge bar, doc-update triggers
  - [`02-data-protection.md`](.claude/rules/02-data-protection.md) — gated-dataset, PII, and binary/secret protection
  - [`03-logging.md`](.claude/rules/03-logging.md) — project-log standard (altitudes, entry triggers, format, immutability, overflow)
  - [`04-docs.md`](.claude/rules/04-docs.md) — documentation standard (single-source, tiers, byte budgets, ENG doc standards, rules numbering)
  - [`05-agent-runtime.md`](.claude/rules/05-agent-runtime.md) — HARD RULE for lab-os assets hosting a local coding-agent runtime (host/engine boundary, execution guardrails, local-brain and $0-by-construction constraints); forward-binding — no asset hosts one today
  - [`06-timeboxing.md`](.claude/rules/06-timeboxing.md) — timeboxing standard (session boxes, agent task boxes, calibration loop; owning docs under `docs/`)
- `.claude/agents/` — specialist review agent bodies dispatched by the lab review skills (vendored from Anthropic's `pr-review-toolkit` plus lab-authored members; provenance: [`ATTRIBUTION.md`](.claude/agents/ATTRIBUTION.md)). Agent bodies execute as instructions the same way skills do — review rigor over `.claude/agents/**` is the security boundary.
- `.claude/skills/` + `.claude/commands/` — shared Claude Code skills and their slash commands. Currently: [`pr-round`](.claude/skills/pr-round/SKILL.md) (`/pr-round`) — one round of PR work across every PR connected to you: review others' PRs, remediate review feedback on your own; [`timeboxing`](.claude/skills/timeboxing/SKILL.md) — runs a handed-over task inside a stated box with an exit criterion, and records one calibration row per box end (standard: [`06-timeboxing.md`](.claude/rules/06-timeboxing.md)). Note: from a bare `lab-os` clone the tier-2 rubric layer is absent until the rules-parity sync (#58) lands, and degrades silently (details: SKILL.md § Deployment). The specialist panel resolves natively here — this repo carries both `.claude/agents/` and `specialist-dispatch.md`; it still degrades silently in a dev home that lacks them (SKILL.md § The specialist panel). Provenance: [`ATTRIBUTION.md`](.claude/skills/ATTRIBUTION.md).
- `.claude/scripts/` — [`link-lab-assets.sh`](.claude/scripts/link-lab-assets.sh) symlinks the skills/commands into `~/.claude/` so they work from any repo, not just sessions opened in the dev home. Re-run per machine.
- [`reference/specialist-dispatch.md`](reference/specialist-dispatch.md) — the owning dispatch contract for the specialist review panel (triggers, per-pass cap, model tier, finding schema, merge/dedup rules, degradation).
- [`PR-LIFECYCLE.md`](PR-LIFECYCLE.md) — end-to-end PR lifecycle: merge bar, solo-maintainer bypass, pre-merge log cleanup.
- [`BACKLOG.md`](BACKLOG.md) — the lab-wide backlog for cross-repo work (convention: [`docs/proposals/2026-07-16-lab-wide-backlog.md`](docs/proposals/2026-07-16-lab-wide-backlog.md)); repo-scoped work stays in that repo's issues.
- `docs/proposals/` — dated convention proposals (`<YYYY-MM-DD>-<slug>.md`), e.g. the [lab-wide backlog proposal](docs/proposals/2026-07-16-lab-wide-backlog.md).
- [`TROUBLESHOOTING.md`](TROUBLESHOOTING.md) — lab-level expensive findings and gotchas, indexed by symptom.
- `templates/` — starter files for new repos and members:
  - [`global-CLAUDE.template.md`](templates/global-CLAUDE.template.md) — personal-global persona + lab operating philosophy (→ `~/.claude/CLAUDE.md`)
  - [`dev-root-CLAUDE.template.md`](templates/dev-root-CLAUDE.template.md) — genericized lab orientation (→ `<DEV_ROOT>/.claude/CLAUDE.md`)
  - [`repo-CLAUDE.template.md`](templates/repo-CLAUDE.template.md) — per-repo CLAUDE.md seed (rules pointer, gate command, gated datasets)
  - [`docs/project_log.template.md`](templates/docs/project_log.template.md) — normative project-log structure (parsed by `log-lint`)
  - [`docs/planning/`](templates/docs/planning/) — the four-file planning-bundle scaffolds ([`prd`](templates/docs/planning/prd.template.md) · [`spec`](templates/docs/planning/spec.template.md) · [`plan`](templates/docs/planning/plan.template.md) · [`log`](templates/docs/planning/log.template.md)), one bundle per slice under `_specs/<repo>/<DATE>-<handle>/`
  - [`PRD.template.md`](templates/PRD.template.md) — older standalone PRD scaffold, superseded in practice by `docs/planning/prd.template.md`; retained pending consolidation
- `.github/workflows/` — adherence Actions (consume via a thin caller; see [`standards.yml`](.github/workflows/standards.yml) as the copyable example):
  - [`log-lint.yml`](.github/workflows/log-lint.yml) — validates project-log structure and entry format
  - [`docs-budget.yml`](.github/workflows/docs-budget.yml) — warns when CLAUDE.md or rules files exceed byte budgets
  - [`merge-bar-check.yml`](.github/workflows/merge-bar-check.yml) — checklist completeness and log-cleanup gate
  - [`backlog-lint.yml`](.github/workflows/backlog-lint.yml) — lab-os-only: BACKLOG.md item hygiene and derived-Index integrity
- `site/` — the Docusaurus handbook, built and deployed to GitHub Pages by [`deploy-site.yml`](.github/workflows/deploy-site.yml).
- `BOOTSTRAP.md`, `WORKING-WITH-CLAUDE.md` — pointer stubs; their content moved to the handbook site.

## How repos consume it

**Locally (Cowork)**: the default onboarding path forks lab-os as your dev home, where the rules live natively (`git pull upstream` to stay current) — see the handbook's [Getting Started](https://camels-research-group.github.io/lab-os/docs/getting-started). The multi-repo power-user pattern instead clones lab-os under a neutral `<DEV_ROOT>` and links its rules up with a junction/symlink:

```powershell
# Windows (PowerShell) — junction, no admin required
cmd /c mklink /J "<DEV_ROOT>\.claude\rules" "<DEV_ROOT>\lab-os\.claude\rules"
```

```bash
# macOS / Linux — symlink
ln -s <DEV_ROOT>/lab-os/.claude/rules <DEV_ROOT>/.claude/rules
```

**In CI (PR reviewer)**: each lab repo's `.github/workflows/pr-review.yml` checks this repo out alongside the PR repo:

```yaml
- uses: actions/checkout@v4
  with: { path: pr-repo }
- uses: actions/checkout@v4
  with:
    repository: CAMELS-Research-Group/lab-os
    path: lab-os
```

The reviewer then concatenates `lab-os/.claude/rules/*.md` + `pr-repo/.claude/rules/*.md` into its prompt context.

**Adherence Actions**: repos also consume the three shared enforcement workflows (`log-lint`, `docs-budget`, `merge-bar-check`) by adding a thin caller that references them from this repo; the fourth, `backlog-lint`, is lab-os-only (downstream repos use their own issues). [`standards.yml`](.github/workflows/standards.yml) is the copyable example.

## Override semantics

Per-repo rules extend or override lab rules. Specific wins over general. Per-repo rules number from `10+` (lab-os owns `0x-*`): a per-repo `10-data-protection.md` listing the repo's specific gated datasets supplements the lab-wide PII checklist; a per-repo rule contradicting a lab rule applies only in that repo.

## Scope discipline

This repo holds **hard rules** — commit format, PR template usage, data-protection invariants, security, approval gates. Soft conventions (code style, library preferences) stay per-repo. Audit periodically to keep this lean — every file here is loaded into every Cowork session and every PR review.
