---
sidebar_position: 3
title: Onboarding Project
description: A two-week sandbox project — you build a small dashboard-style app in a disposable repo to practice the lab's full workflow and conventions.
---

# Onboarding Project — Mission-Control Sandbox

A two-week, throwaway build that gets you fluent in how the lab actually works: you build a small
"mission-control" style work surface — the kind of dashboard we use to run the lab — in your own
disposable repo, through the full lab workflow.

**The repo is meant to be thrown away.** The deliverable that matters is what you *learn*: we don't
yet know the best design patterns for an agent-driven work-surface app, and we'd rather discover
them on throwaway code than on the lab's real `mission-control` app. Build fast, build loose, capture what
worked. Don't aim for production — aim for *learning velocity*.

Read [Working with Claude](/docs/working-with-claude) first — it's the methods this project makes
you practice. Then create the disposable repo per [Setting up a new repo](/docs/repo-setup).

---

## Three axes

The *what* is open for you to design; the *how* is fixed; the stack is a discovery surface in its
own right.

### Axis 1 — Capability checklist (the *what*: open, design it yourself)

Build surfaces — this project's word for the screens or modules of your app — that together cover
this checklist. *(optional)* items are trimmable if the timebox squeezes.

| # | Requirement | Notes |
|---|---|---|
| 1 | **2 data integrations**, spanning ≥2 distinct shapes | e.g. external REST API, local file / SQLite, webhook or polled stream — not two of the same shape |
| 2 | **At least one integration is authenticated** | a real sign-in (SSO / OAuth) or an API key — one of your two, forcing real secret handling |
| 3 | **1 CRUD surface** | create / read / update / delete over something you own |
| 4 | **1 data-visualization surface** | charts, derived-metrics table, timeline — turn data into a view |
| 5 | *(optional)* **1 background / async-job or agent-driven surface** | ties into the lab's overnight-agent work — see [autonomous loops](/docs/working-with-claude#6-autonomous--overnight-loops) |
| 6 | *(optional)* **1 command-palette or cross-surface action** | one action reaching across surfaces; good UX-pattern discovery |

How these compose into pages/modules is yours to design. That design *is* the discovery.

### Axis 2 — Workflow spine (the *how*: fixed, non-negotiable)

Every surface passes through the full loop. If the timebox squeezes, trim **optional Axis-1 items
first** — never spine steps.

1. **Brainstorm** — `superpowers:brainstorming` to shape the surface before designing it.
2. **Spec** — a short design doc: what it does, how it's used, what it depends on.
3. **Code-free implementation plan** — per the
   [lab plan format](/docs/working-with-claude#2-code-free-implementation-plans). No literal code.
4. **Sub-agent-driven build** — implement by delegating to subagents (helper AI agents), not
   hand-coding everything in one session. Practice delegating.
5. **Review** — `superpowers:requesting-code-review` (dispatches a reviewer and queues a
   `pr-review-loop` cycle) or `pr-review-loop` directly. (The lab's automated PR reviewer covers
   lab repos only, not your sandbox.)
6. **Log** — `project_log.md` entries per
   [`03-logging.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/03-logging.md).

Spec and plan are committed **before** the code for that surface. That ordering is the point.

### Axis 3 — Stack & deployment (a discovery surface in its own right)

**Choose your own stack** — you're not constrained to the lab's FastAPI + React default. But you
must produce a **deployment-tradeoff writeup**: weigh at least local-first vs. container vs.
serverless vs. managed-PaaS for *this* app, on cost, secret management, cold-start, operational
overhead, and lab-fit; land on a choice and justify it. One module may deliberately use a different
stack as a documented experiment.

---

## Deliverables

- [ ] Working surfaces covering the Axis-1 checklist (minus trimmed optionals)
- [ ] A spec + code-free plan committed **before** the code, per surface
- [ ] The **deployment-tradeoff writeup**
- [ ] A `project_log.md` tracking decisions as you go
- [ ] **Patterns & findings retro** (retrospective writeup) — your proof of completion

### The retro (this is what completion means)

A single markdown doc capturing:

- **≥3 reusable design patterns** — what it is, where it helped, when it wouldn't
- **Notable feature sets** worth keeping for the real `mission-control`
- **What to avoid** — dead ends, patterns that looked good and weren't
- **Workflow reflections** — where sub-agent delegation and spec-first discipline paid off or
  felt like overhead

The repo gets thrown away. The retro doesn't.

---

## Guardrails

Lab rules apply even on throwaway code — practicing them here is part of the point:

- **Commits & PRs** —
  [`01-workflow.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/01-workflow.md)
- **Data protection** —
  [`02-data-protection.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/02-data-protection.md).
  **No gated (license-restricted) datasets** (IEMOCAP, CANDOR, MOSEI) anywhere — synthetic, openly-licensed, or your own
  throwaway data only.
- **Secrets** — tokens/keys in a gitignored `.env`, never committed; secret-scan before push
  (e.g. `gitleaks detect`)
- **File hygiene** — 5 MB/file limit; no checkpoints or binary artifacts
- **Spend** — stay in free tiers; run AI usage through your Claude subscription; flag anything
  over $10 before incurring it

---

## Timebox

**Two weeks.** Favor breadth and learning velocity over polish — a rough surface that taught you a
pattern beats a beautiful one that taught you nothing. When in doubt, trim an optional checklist
item and protect the workflow spine and the retro.
