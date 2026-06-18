---
title: The exercises
description: The five hands-on exercises of Building with Claude — each run against your own plan, from decomposition to autonomous multi-task execution.
---

# The exercises

Five exercises, each run against **your own plan** — so you leave with real progress, not a
sandbox toy. Each follows the same shape: **Goal · You start with · What you do · Done when ·
If you're stuck.**

The "what you do" describes a *shape*, not a script. Talk to Claude in your own words — that's
the skill the day is building.

## Exercise 1 — Plan → roadmap (20 min)

- **Goal:** Turn your plan into an ordered, discrete task backlog Claude can act on.
- **You start with:** your plan, open.
- **What you do:** Point Claude at your plan and ask it to produce a roadmap — an ordered list
  of concrete, individually-executable tasks. Push back if a "task" is too big to finish in
  one sitting; ask Claude to split it.
- **Done when:** you have a written roadmap of 5–15 tasks, each small enough to execute on its
  own.
- **If you're stuck:** your plan may be too vague to decompose — that's a finding, not a
  failure. Flag it; we'll workshop it live.

## Exercise 2 — Sequencing & dependencies (20 min)

- **Goal:** Know what blocks what, and what can run in parallel.
- **You start with:** your roadmap from Exercise 1.
- **What you do:** Ask Claude to mark dependencies between tasks — which must come before which
  — and to group the independent ones into "waves" that could run at the same time.
- **Done when:** your roadmap shows a clear first wave of tasks with no unmet dependencies
  (your starting point), and you can name at least one set of tasks that could run in parallel.
- **If you're stuck:** if everything depends on everything, your tasks are probably too
  coarse — go back and split further.

## Exercise 3 — Isolation: worktrees (15 min)

- **Goal:** Create a safe, throwaway workspace so an autonomous run can't damage your main
  branch.
- **You start with:** a clean working tree and your first-wave task picked.
- **What you do:** Create a git worktree for your first task. Confirm you're working *in* it,
  isolated from your main checkout.
- **Done when:** `git worktree list` shows your new worktree and you know which directory
  you're working in.
- **If you're stuck:** this is the [pre-flight](./preflight.md) item — if it failed yesterday
  it'll fail now. Grab a facilitator.

## Exercise 4 — First autonomous execution (30 min)

*The pivot of the day — the first time you let Claude run without driving every step.*

- **Goal:** Hand Claude one real task and let it run to completion.
- **You start with:** your worktree and one first-wave task.
- **What you do:** Give Claude the task, the relevant context from your plan, and tell it to
  execute end to end. Watch — don't micromanage. When it claims done, **verify** it (run the
  thing, check the diff) rather than taking its word.
- **Done when:** the task is actually done — the change is visible in the diff and your
  verification step passes.
- **If you're stuck:** if Claude goes sideways, stop it, note where it lost the thread, and
  we'll cover recovery in the debrief — that's exactly the learning.

## Exercise 5 — Scaling execution (25 min)

- **Goal:** Hand off more than one task at once — a chunk Claude works through start to finish.
- **You start with:** Exercise 4 done; the rest of your first wave available.
- **What you do:** Give Claude a multi-task chunk from your roadmap and tell it to work through
  them in order, verifying as it goes. Stay hands-off; verify the result yourself at the end.
- **Done when:** two or more tasks completed in a single hand-off, each verified.
- **If you're stuck:** scope down — even two tasks chained is a win. The point is the hand-off
  pattern, not volume.
