# Development root — CAMELS Research Group lab orientation

> **Template.** Copy this to the `.claude/CLAUDE.md` at the root of your local lab workspace (the
> directory you clone all lab repos into — referred to below as `<DEV_ROOT>`):
> - **Windows:** `<DEV_ROOT>\.claude\CLAUDE.md` (e.g. `C:\Users\<you>\Development\.claude\CLAUDE.md`)
> - **macOS / Linux:** `<DEV_ROOT>/.claude/CLAUDE.md` (e.g. `~/Development/.claude/CLAUDE.md`)
>
> This file gives Cowork lab-wide orientation when you open a session at `<DEV_ROOT>` (not just inside a
> single repo). Your personal persona / approval gates / model defaults live in your global
> `~/.claude/CLAUDE.md` (see `global-CLAUDE.template.md`) and apply everywhere. Per-repo `CLAUDE.md`
> cascades when a session works inside a sub-project. Delete this blockquote when done.

Working from the lab root (not just inside a single repo) is intentional — it's where cross-project
coordination happens: multi-repo planning, lab-wide tooling and rules, cross-repo logs. Per-repo
`CLAUDE.md` cascades when a session works inside a sub-project; nothing is lost by having the broader
view available.

## Research lineage

The CAMELS research line is multi-repo and predates the current active repos. Don't treat older repos as
unrelated:

1. **`SSUR_POC`** (public, github.com/WatsonWBlair/SSUR_POC) — original public-facing voice-to-voice POC;
   upstream of `Vibe_App`.
2. **`Vibe_App`** — the spark. Voice-to-voice POC that surfaced the questions driving the program.
3. **`cs627`** — formal initial investigation. CS coursework that operationalized the early CAMELS hypotheses.
4. **`LSCA`** — current home of CAMELS (Conversational Agents in Multimodal Embedded Latent Space).
   APE-architecture, multimodal latent space, training pipeline. See `LSCA/CLAUDE.md` for project specifics.
5. **`Conversational_Agent`** — voice-agent work; lives alongside the Vibe artifacts.

When a "why this design?" or "where does X come from?" question lands, check upstream in this chain before
assuming the answer is new.

## Active or foundational repos

| Repo | Role | Status |
|---|---|---|
| `LSCA` | CAMELS multimodal latent space (Python, PyTorch) | Active research, primary focus |
| `Global_Pathways` | First CAMELS application — Pace ESL language-practice (consumes the `camels` package built from LSCA) | Active — pre-implementation spec phase |
| `Vibe_App` | Voice-to-voice POC; genesis of CAMELS | Foundational; reference for design decisions |
| `cs627` | Initial CAMELS investigation (CS coursework) | Foundational |
| `Conversational_Agent` | Voice agent (Whisper + Ollama + Bark) | Reference / related |
| `FCM_Analysis` | Fuzzy C-Means vs GMM comparative analysis | Back burner — paused |

Only `LSCA`, `Global_Pathways`, and the tooling repos are part of the core bootstrap clone set (see
`BOOTSTRAP.md`). Foundational and paused repos are cloned on demand when a question sends you upstream.

## Lab tooling

- **`lab-rules`** — cross-repo conventions (this repo's source). Consumed by Cowork via the `.claude/rules/`
  junction/symlink at `<DEV_ROOT>`, and by the PR-review GitHub Action at review time. Source-of-truth:
  github.com/WatsonWBlair/lab-rules.
- **`lab-claude-plugins`** — lab Claude Code plugins (e.g. `pr-review-loop`). Installed via the plugin
  marketplace. Source-of-truth: github.com/WatsonWBlair/lab-claude-plugins.

## Logs and tracking

- **Per-repo logs:** `<repo>/project_log.md`
- **Lab-level decisions** (cross-repo tooling, infra, lab-wide conventions): `<DEV_ROOT>/project_log.md`
- **Cost tracking** (inference spend, infra): `<DEV_ROOT>/cost-tracking.md`

Entry format defined in your global `~/.claude/CLAUDE.md`.

## Approval gates

Defined in your global `~/.claude/CLAUDE.md`. Cross-cutting items at this level:

- External-facing posts (PRs, issue comments, anything under your name; bot identity OK, user identity not)
- Cloud spend above your stated ceiling
- Data exposure risks (raw gated-dataset content; derived artifacts need PII review per
  `.claude/rules/02-data-protection.md`)
- Destructive operations on shared state
