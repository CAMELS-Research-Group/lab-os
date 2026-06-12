---
sidebar_position: 3
title: Onboarding Project
description: A two-week, throwaway sandbox build — a mission-control-style work surface — that gets new lab members fluent in the spec-driven, sub-agent-driven lab workflow and conventions.
---

# Onboarding Project — Mission-Control Sandbox

A two-week, throwaway build that gets you fluent in how the lab actually works: spec-driven
development, agentic and sub-agent-driven workflows, and the lab's conventions. You build a
small "mission-control" style work surface — the kind of dashboard we use to run the lab — in your
own disposable repo.

**The repo is meant to be thrown away.** The deliverable that matters is what you *learn*: a writeup
of the design patterns and feature sets worth carrying into real work. Build fast, build loose,
capture what worked.

Read [Working with Claude](/docs/working-with-claude) first if you haven't — it's the methods this
project makes you practice. Then your first step is creating the disposable repo itself: follow the
"Setting up a new repo" runbook. {/* TODO(task 13): link /docs/repo-setup */}

---

## Why this project

We don't yet know the best design patterns for an agent-driven work-surface app, and we'd rather
discover them on throwaway code than on the real `mission-control` dashboard (the lab's internal
work-surface app this sandbox is modeled on). You're a contributor to that discovery. By the end
you will have:

- Run the full lab workflow loop — brainstorm → spec → plan → sub-agent build → review — end to end,
  independently, several times.
- Practiced secret handling, the commit/PR conventions, and the data-protection rules on low-stakes code.
- Produced a **patterns & findings retro** the lab can mine for the real product.

This is not the real `mission-control`. Don't aim for production. Aim for *learning velocity*.

---

## Three axes

The project has three axes, and you hold them separately. The *what* is open for you to design and
explore; the *how* is fixed; the stack is a discovery surface in its own right.

### Axis 1 — Capability checklist (the *what*: open, design it yourself)

Build surfaces that, together, cover this checklist. Items marked *(optional)* are trimmable — do
them if the timebox allows.

| # | Requirement | Notes |
|---|---|---|
| 1 | **2 data integrations**, spanning ≥2 distinct shapes | e.g. external REST API, local file / SQLite, a webhook or polled stream. Don't use two of the same shape. |
| 2 | **At least one integration is authenticated** | SSO / OAuth / bearer token / API key. This is one of your two integrations, not a third — it forces real secret handling. |
| 3 | **1 CRUD surface** | Create / read / update / delete over something you own. |
| 4 | **1 data-visualization surface** | Charts, a table with derived metrics, a timeline — turn data into a view. |
| 5 | *(optional)* **1 background / async-job or agent-driven surface** | A job queue, a scheduled task, or a surface an agent drives. Ties into the lab's overnight-agent work (see the [autonomous-loops section](/docs/working-with-claude#6-autonomous--overnight-loops) of [Working with Claude](/docs/working-with-claude)). |
| 6 | *(optional)* **1 command-palette or cross-surface action** | A single action that reaches across surfaces. Good for discovering work-surface UX patterns. |

How these compose into pages/modules is yours to design. That design *is* the discovery.

### Axis 2 — Workflow spine (the *how*: fixed, non-negotiable)

Every surface you build passes through the full loop. If the timebox squeezes, you trim **checklist
items (Axis 1)** — never spine steps.

1. **Brainstorm** — use the `superpowers:brainstorming` skill to shape the surface before you design it.
2. **Spec** — write a short design doc. What it does, how it's used, what it depends on.
3. **Code-free implementation plan** — per the lab plan format (Files / Depends on / Spec / Acceptance /
   Verification / Commit per task). No literal code in the plan. See the [code-free-plans section](/docs/working-with-claude#2-code-free-implementation-plans)
   of [Working with Claude](/docs/working-with-claude) for the format and the reasoning behind it.
4. **Sub-agent-driven build** — implement by dispatching agents (`Explore` to map, `Plan` to design,
   agents to check), not by hand-coding everything in one context. Practice delegating.
5. **Review** — run a review pass using `superpowers:requesting-code-review` (which queues a
   `pr-review-loop` cycle), or `pr-review-loop` directly to drive multi-pass review-fix cycles. The
   lab's automated PR reviewer runs only on covered lab repos — not your personal sandbox repo.
6. **Log** — a `project_log.md` entry: ISO date, one-line subject, body with the *why*. Format and
   entry triggers: [`03-logging.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/03-logging.md).

Spec and plan are committed **before** the code for that surface. That ordering is the point.

### Axis 3 — Stack & deployment (a discovery surface in its own right)

**Choose your own stack.** You're not constrained to the lab's FastAPI + React default — explore.

But you must produce a **deployment-tradeoff writeup**: weigh at least local-first vs. container vs.
serverless vs. managed-PaaS for *this* app, on cost, secret management, cold-start, operational
overhead, and lab-fit. Land on a choice and justify it. One module may deliberately use a different
stack as a documented experiment — note what that cost or bought you.

---

## Deliverables

Per member, in your own throwaway repo:

- [ ] The working surfaces covering the Axis-1 checklist (minus any optional items you trimmed).
- [ ] A spec + code-free plan committed **before** the code, for each surface.
- [ ] The **deployment-tradeoff writeup**.
- [ ] A `project_log.md` tracking decisions as you go.
- [ ] **Patterns & findings retro** — your proof of completion (see below).

### The retro (this is what completion means)

A single markdown doc capturing:

- **≥3 reusable design patterns** you found — what the pattern is, where it helped, when it wouldn't.
- **Notable feature sets** worth keeping for the real `mission-control`.
- **What to avoid** — dead ends, things that fought you, patterns that looked good and weren't.
- **Workflow reflections** — where sub-agent delegation paid off, where it didn't, where the
  spec-first discipline helped or felt like overhead.

The repo gets thrown away. The retro doesn't.

---

## Guardrails

Lab rules apply even on throwaway code — practicing them here is part of the point:

- **Commits & PRs** — follow
  [`01-workflow.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/01-workflow.md)
  (conventional-commit format, PR template).
- **Data protection** —
  [`02-data-protection.md`](https://github.com/WatsonWBlair/lab-os/blob/main/.claude/rules/02-data-protection.md).
  **No gated datasets** (IEMOCAP, CANDOR, MOSEI) anywhere in this project. Use synthetic,
  openly-licensed, or your own throwaway data.
- **Secrets** — for the authenticated integration: tokens/keys go in a gitignored `.env`, never
  committed. Run secret-scanning before you push (e.g. `gitleaks detect`).
- **File hygiene** — 5 MB/file limit; no checkpoints or binary artifacts committed.
- **Spend** — stay in free tiers; run agent inference through your Claude Max subscription rather
  than a metered API key. Flag anything that would cost more than $10 before you incur it.

---

## Timebox

**Two weeks.** Favor breadth and learning velocity over polish. A rough surface that taught you a
pattern beats a beautiful surface that taught you nothing. When in doubt, trim an optional checklist
item and protect the workflow spine and the retro.
