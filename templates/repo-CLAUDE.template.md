# <Repo name> — <one-line repo purpose>

<!-- AUTHOR GUIDANCE — AI-tier writing standard (read before filling):
     Dense, deterministic, count-free, zero narrative. State facts an agent acts on.
     No "this repo was created to…" prose, no counts that restale, no passive voice.
     Budget: 8 KB (CI warn at 8 KB, fail at 12 KB; warn-only until the repo first
     passes green). Keep this file ≤ 6 KB
     so growth stays inside budget. When a section balloons, push the detail out:
       design rationale → design doc (ENG tier)
       how-to steps → runbook (ENG tier)
       decision history → project_log.md
       conventions & rules → .claude/rules/ or lab-rules (already loaded)
     Delete these guidance comments when the file is committed. -->

> **Template.** Copy to `<repo>/.claude/CLAUDE.md` or `<repo>/CLAUDE.md`. Fill every `<...>`
> placeholder. Delete this blockquote and all HTML guidance comments when done.
> Lab-wide rules (`.claude/rules/0x-*`) load automatically — do not restate them here.

## What this repo is

<!-- One short paragraph. Answer: what problem does this repo solve, for whom, and what
     it is NOT responsible for. Boundaries here save agents from scope-creep mistakes.
     Design rationale belongs in docs/; put only the load-bearing "what" here. -->

<One paragraph: purpose, primary consumer, explicit out-of-scope.>

## Commands

<!-- List only the commands an agent in this repo needs to run. Gate command first.
     No prose; no explanations beyond what the command does. How-tos → runbook. -->

| Command | Purpose |
|---|---|
| `<gate command>` | Verification gate — run unpiped before every merge |
| `<dev command>` | <what it starts / runs> |
| `<install command>` | <what it installs> |

<!-- Add or remove rows. If this is a docs-only repo with no gate command, say so:
     Gate: none (docs-only — PR template Verification section is the gate). -->

## Architecture orientation

<!-- What an agent needs to navigate the repo: key directories, primary entry points,
     module boundaries. One or two lines per item max.
     Detailed system design → design doc or TRD (ENG tier). -->

| Path | Role |
|---|---|
| `<path>` | <one-line role> |
| `<path>` | <one-line role> |

<!-- Add rows. Remove section entirely for very small or single-file repos. -->

## Boundaries and invariants

<!-- Hard rules an agent must never violate in this repo. Examples:
       - Never commit X — stored in Y instead
       - Module A must not import from module B
       - All external I/O goes through Z
     Keep this list short; if it exceeds ~6 bullets the invariants belong in a rules file.
     Data-protection invariants specific to this repo → .claude/rules/10-data-protection.md -->

- <Invariant 1>
- <Invariant 2>

## Conventions

<!-- Pointer only — do not restate what the rules files already say. -->

Lab-wide conventions: `.claude/rules/01-workflow.md` (commits, PRs, merge bar),
`02-data-protection.md`, `03-logging.md` (log format), `04-docs.md` (tiers, budgets).

Repo-local rules (this repo, `10+` numbering): `.claude/rules/10-<name>.md`

<!-- Add a line per repo-local rule file that exists. Remove the placeholder line if none.
     Per D12 of the logging-and-docs standard: lab-rules owns 0x-*, per-repo rules use 10+. -->
