---
title: Pre-flight checklist
description: Do this the day before Building with Claude — confirm your environment works so the session is spent building, not troubleshooting.
---

# Pre-flight checklist

Do this **the day before** the session. The whole point is that nobody spends the first hour
of an in-person day fighting their environment. Reply *"ready"* once all five boxes pass — or
flag where you're stuck so we fix it before the session, not during it.

## Confirm each of these works

- [ ] **Claude Code is installed and authenticated.** Open a terminal, run `claude`, and
  confirm you reach a prompt without a login error. If you hit auth, sort it now.
- [ ] **Git is installed and working.** Run `git --version` and `git status` inside your
  project. Both should succeed.
- [ ] **Your project is cloned locally**, on a branch you're comfortable experimenting on,
  with a clean working tree (`git status` shows nothing uncommitted you care about).
- [ ] **Your plan is in the repo** (or somewhere you can open it next to Claude). You'll be
  reading it back to Claude all day — know where it is.
- [ ] **Worktrees work on your machine.** Run `git worktree add ../_preflight-test`, then
  `git worktree remove ../_preflight-test`. Both should succeed with no error. *This is the
  one most likely to surprise people — test it now.*

## Bring

- Laptop + charger, your repo, your plan.
- Optional but nice: a second monitor if you're laptop-only.

## If any box fails

Reply with the exact error and we'll get you sorted before the session. Don't show up hoping
it'll work — a failed worktree check at noon stalls you for the whole first exercise block.
