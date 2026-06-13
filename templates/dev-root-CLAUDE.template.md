# Development root — <your team / project> workspace orientation

> **Template.** Copy this to the `.claude/CLAUDE.md` at the root of your local workspace (the
> directory you clone all your repos into — referred to below as `<DEV_ROOT>`):
> - **Windows:** `<DEV_ROOT>\.claude\CLAUDE.md` (e.g. `C:\Users\<you>\Development\.claude\CLAUDE.md`)
> - **macOS / Linux:** `<DEV_ROOT>/.claude/CLAUDE.md` (e.g. `~/Development/.claude/CLAUDE.md`)
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

Core bootstrap clone set: list the repos a new contributor must clone to start working (typically your
active repos plus the tooling repos below). Foundational and paused repos are cloned on demand when a
question sends you upstream.

## Tooling

List the tooling repos your workspace relies on — shared conventions/rules, plugins, CI helpers — and
how each is wired in (e.g. a `.claude/rules/` junction/symlink at `<DEV_ROOT>`).

- **`<your-conventions-repo>`** — cross-repo conventions (the conventions repo you cloned, e.g. a fork
  of `lab-os`). Consumed by Cowork via the `.claude/rules/` junction/symlink at `<DEV_ROOT>`, and by
  your PR-review GitHub Action at review time. Source-of-truth: `<URL>`.
- **`<your-plugins-repo>`** _(optional)_ — any Claude Code plugins your workspace uses. Source-of-truth:
  `<URL>`.

## Logs and tracking

- **Per-repo logs:** `<repo>/project_log.md`
- **Workspace-level decisions** (cross-repo tooling, infra, workspace-wide conventions): `<DEV_ROOT>/project_log.md`
- **Cost tracking** (inference spend, infra): `<DEV_ROOT>/cost-tracking.md`

Entry format defined in your global `~/.claude/CLAUDE.md`.

## Approval gates

Defined in your global `~/.claude/CLAUDE.md`. Cross-cutting items at this level:

- External-facing posts (PRs, issue comments, anything under your name; bot identity OK, user identity not)
- Cloud spend above your stated ceiling
- Data exposure risks (raw gated-dataset content; derived artifacts need PII review per
  `.claude/rules/02-data-protection.md`)
- Destructive operations on shared state
