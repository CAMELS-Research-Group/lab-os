---
title: "Planning surface"
description: Turn a raw idea into a tracked, sized backlog item — a small, repeatable workflow for getting work out of your head and into a place you can act on it.
---

# Planning surface

This page is where a raw idea becomes a **tracked piece of work**. It is deliberately small: somewhere
to write the idea down, shape it just enough to act on, and land it in the backlog so it isn't lost.
For the deeper idea-to-plan flow — interrogating an idea into a full PRD and decomposing it into an
execution-ready plan — see [the Planning part of the workshop](../workshops/planning/index.md). This
surface is the lightweight front door to that.

## The workflow

1. **Capture.** Write the raw idea in the **Inbox** of `BACKLOG.md`. One line is fine. The goal is to
   stop carrying it in your head.
2. **Shape.** When you're ready to act on an idea, turn it into a structured item using
   `templates/backlog-item.template.md`. The item names the problem, who it helps, a rough size, and —
   the load-bearing field — a single **Done when** condition you could check.
3. **Land it.** Move the shaped item from the Inbox into `## Items`, and add a row to the **Index**
   table so it shows up in the "what's ready now" view.

An item is **ready** when you can state its *Done when* as one observable check. If you can't, it's
either too big (split it) or too vague (shape it more). That is the same bar the
[Planning part](../workshops/planning/index.md) puts on a task before it's execution-ready.

## Shaping an item with Claude

You don't have to fill the template by hand. Point Claude at the idea and let it interview you — the
same "let the agent interrogate you" move from the Planning part. Copy this prompt and replace the
idea line:

```text
I want to add a backlog item to BACKLOG.md. The raw idea is:

  <your one-line idea here>

Interview me one question at a time until you can fill every field of
templates/backlog-item.template.md — especially a single, checkable "Done when".
Don't draft the item until the Done-when is concrete. When it is, show me the
finished item block and, on my OK, append it under ## Items in BACKLOG.md and add
a row to the Index table.
```

Keep answering until the *Done when* is something you could actually run or observe. The interview is
doing the same work the PRD interrogation does, scaled down to one item.

## What this surface is not

It is not a project tracker, a sprint board, or a roadmap. It is a place to get work out of your head
and into a shape you can hand to an autonomous run. When an item grows past a single *Done when*, it
has outgrown this surface — take it to the [Planning part](../workshops/planning/index.md) and give it
a real plan.
