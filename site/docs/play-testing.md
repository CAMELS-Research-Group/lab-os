---
sidebar_position: 6
title: How to Play-Test
description: Protocol card for play-testing the handbook — the arc to walk, the friction-capture rules, the 15-minute stall rule, and the success bar.
---

# How to Play-Test

This site is being play-tested before it counts as done. If you're one of the 1–3 testers — a lab
member with an existing Claude subscription — this card is your protocol.
You're not just following the handbook; you're stress-testing it. **Friction is the data we want.**

## The arc

Walk these in order, as if you were a brand-new lab member:

1. **Landing** — start at the landing page, cold. Does it tell you where to go next?
2. **Guided environment setup** — follow [Getting Started](/docs/getting-started) end to end,
   including the Claude-guided flow that produces your personalized `~/.claude/CLAUDE.md`.
3. **Methods read** — read [Working with Claude](/docs/working-with-claude).
4. **Sandbox-project brainstorm & spec** — start the [Onboarding Project](/docs/onboarding-project):
   create your own disposable repo per [Setting Up a New Repo](/docs/repo-setup), then brainstorm
   and spec your first surface (one screen or module of your sandbox app). The play-test arc ends
   at the committed spec — the full two-week
   build is the onboarding project itself, not this protocol.

## Capturing friction

Every snag gets a **friction issue** — file it through the
[lab-os issue chooser](https://github.com/CAMELS-Research-Group/lab-os/issues/new/choose) and pick the
**Friction report** template. It asks four things and auto-labels the issue `playtest`:

- **Where I was** — page and step
- **What I expected**
- **What happened**
- **Severity** — from cosmetic to fully blocked

Small annoyances count. A confusing sentence, a link you expected that wasn't there, a step that
assumed something you didn't have — file them all. Cheap issues now beat a broken onboarding later.

## The 15-minute stall rule

Stuck for more than 15 minutes on one step? **File a friction issue first, then ask Watson**
(the lab manager). That order is the point: if you unblock via Watson without filing, the handbook
never learns what stalled you, and the next person hits the same wall.

## End of arc: the retro

When you finish the arc, write a short retro (retrospective) and send it to Watson directly (email or Discord DM —
not a friction issue; those track individual snags). Three prompts:

- **What confused you** — even if you eventually figured it out
- **What was missing** — things you needed that no page covered
- **What you'd cut** — anything that wasted your time or said nothing

## Success bar

The play-test passes for you when you reach both of these **without Watson unblocking you**:

- A **working environment**, including a personalized `~/.claude/CLAUDE.md` produced through the
  guided setup flow
- A **brainstormed sandbox-project spec, committed** in your disposable repo

And one process criterion: **every stall produced an issue.** Needing help isn't failure — needing
help without a friction issue on record is.
