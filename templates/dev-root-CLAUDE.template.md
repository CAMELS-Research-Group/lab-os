# Development root — <your team / project> workspace orientation

> **Template.** Copy this to the `.claude/CLAUDE.md` at the root of your dev home — your local clone
> of your lab-os fork (referred to below as `<DEV_ROOT>`); your own projects nest inside it as their
> own repos:
> - **Windows:** `<DEV_ROOT>\.claude\CLAUDE.md` (e.g. `C:\Users\<you>\Development\lab-os\.claude\CLAUDE.md`)
> - **macOS / Linux:** `<DEV_ROOT>/.claude/CLAUDE.md` (e.g. `~/Development/lab-os/.claude/CLAUDE.md`)
>
> This file gives Cowork workspace-wide orientation when you open a session at `<DEV_ROOT>` (not just
> inside a single repo). Your personal persona / approval gates / model defaults live in your global
> `~/.claude/CLAUDE.md` (see `global-CLAUDE.template.md`) and apply everywhere. Per-repo `CLAUDE.md`
> cascades when a session works inside a sub-project. Delete this blockquote when done.

Working from the workspace root (not just inside a single repo) is intentional — it's where cross-project
coordination happens: multi-repo planning, workspace-wide tooling and rules, cross-repo logs. Per-repo
`CLAUDE.md` cascades when a session works inside a sub-project; nothing is lost by having the broader
view available.

## Project lineage

List the lineage of your project — earlier repos, POCs, or coursework that newer work descends from, so
a session checks upstream before treating a question as new. Oldest-first; one line per entry naming what
it was and what it proved or surfaced.

Example chain (replace with your own):

1. **`<earliest-prototype>`** — brief description of what it was and what it established.
2. **`<intermediate-repo>`** — what question or finding it handed forward.
3. **`<current-repo>`** — current home. See `<current-repo>/CLAUDE.md` for project specifics.

When a "why this design?" or "where does X come from?" question lands, check upstream in this chain before
assuming the answer is new.

## Active or foundational repos

List every repo a session at `<DEV_ROOT>` might touch. Columns: **Repo** (name as cloned) · **Role**
(one-line purpose) · **Status** (active / foundational / paused / reference).

| Repo | Role | Status |
|---|---|---|
| `<your-primary-repo>` | `<what it does — the main active work>` | Active — `<phase>` |
| `<your-secondary-repo>` | `<what it does>` | Active — `<phase>` |
| `<your-earlier-prototype>` | `<what it was; why it's kept>` | Foundational; reference for design decisions |

Nested project repos: list the project *code* repos you've nested inside your dev home (each its own
git repo, gitignored from the fork). Your plans and backlog are not nested here — they track in the
fork itself (see Logs and tracking). Foundational or reference repos are added on demand when a
question sends you upstream.

## Tooling

The shared tooling your workspace relies on — wired in during setup:

- **`lab-os`** — the spec-driven-development conventions. This dev home **is** your fork of lab-os, so
  its `.claude/rules/` live here natively; `git pull upstream main` pulls rule updates from the source.
  Treat these as your **starting standards** — adapt or extend them as your project's needs diverge.
  (Prefer many repos under one neutral workspace? The junction/symlink consumption pattern is the
  power-user alternative — see the handbook's Getting Started.) Source-of-truth: github.com/CAMELS-Research-Group/lab-os.
- **`lab-claude-plugins`** — the Claude Code plugins installed during setup (e.g. `pr-review-loop`),
  via the plugin marketplace. Source-of-truth: github.com/WatsonWBlair/lab-claude-plugins.
- **`superpowers`** — workflow plugins from the official Claude Code marketplace (brainstorming,
  planning, TDD, subagent-driven execution).

As your project grows, list your own repos and any further tooling here so a session at `<DEV_ROOT>`
sees the whole workspace.

## Logs and tracking

- **Plans / backlog:** `<DEV_ROOT>/_plans/` — tracked by the fork, alongside the rules and log
- **Per-repo logs:** `<repo>/project_log.md`
- **Workspace-level decisions** (cross-repo tooling, infra, workspace-wide conventions): `<DEV_ROOT>/project_log.md`
- **Cost tracking** (inference spend, infra): `<DEV_ROOT>/cost-tracking.md`

Entry format defined in your global `~/.claude/CLAUDE.md`.

## Approval gates

Defined in your global `~/.claude/CLAUDE.md`. Cross-cutting items at this level:

- External-facing posts (PRs, issue comments, anything under your name; bot identity OK, user identity not)
- Cloud spend above your stated ceiling
- Data exposure risks (raw sensitive or restricted data; derived artifacts need PII review per
  `.claude/rules/02-data-protection.md`)
- Destructive operations on shared state
