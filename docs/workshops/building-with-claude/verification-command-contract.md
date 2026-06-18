# Verification command contract

Status: draft · Owner: workshop facilitators · Audience: white-label infrastructure implementers

## Why this exists

The "Building with Claude" co-working day has participants execute real Plan tasks
autonomously (Exercises 4 and 5). Each task is only "done" when the participant can
**prove** it — not take Claude's self-report as evidence (the Verify stage of the
handbook lifecycle: source of truth is `site/docs/working-with-claude/verify.md`).

The exercise handouts are written against a *contract*, not a specific tool, so any
white-label implementation can satisfy them and the handouts never change. Implementations
are free to choose command names and mechanics; the behavior below is fixed.

## The contract

Whatever the infrastructure provides, the following must hold:

- **A single verification command exists** that runs the project's gate — tests, lint,
  typecheck, whatever the project defines as "passing."
- **It runs from the repo root in one invocation.** No multi-step sequence, no
  "first cd into…, then…".
- **Exit code is the source of truth:** `0` = pass, non-zero = fail. This is what makes
  "done when verification passes" a deterministic instruction instead of a judgment call.
- **Output is human-readable.** A participant eyeballs it to see *what* failed, not just
  that something did.
- **No interactive prompts.** It runs to completion unattended, so it's safe to invoke
  inside an autonomous run.
- **(If the project is runnable) a single run/start command** with the same properties,
  so participants can also eyeball behavior, not only green checks.

The command *names* belong to the implementation. The *behavior* above is the contract the
workshop materials depend on.

## Default surface (recommendation, not contract)

A dedicated AI-interaction surface is a strong candidate for the **default** surface on
white-label implementations — a consistent place participants drive Claude and invoke these
commands, rather than each implementation inventing its own. Treat this as a design lead for
the infra, not a hard requirement: the contract above is what the exercises bind to.
