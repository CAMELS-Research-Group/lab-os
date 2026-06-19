---
title: Building Pre-Flight
description: Environment-readiness checklist to complete before the Building session — confirm your tools work, your project is on a safe branch, and git worktrees run on your machine.
---

# Building Pre-Flight

Complete this before the Building session begins (or before you open the Building worksheets
self-paced). The goal is to confirm that your environment is ready so the session is spent on
execution, not on setup troubleshooting.

If you hit a failure on any item, fix it before the session. Bring the checklist fully checked —
or flag which item you're stuck on — so the session can start.

---

## 1 — Claude Code is installed and authenticated

- [ ] `claude --version` runs without error
- [ ] Running `claude` opens a session without prompting for a new login
- [ ] You are able to exchange at least one message in that session

If this is not set up yet, follow [Getting Started](/docs/getting-started) first and return here
when those checks pass.

---

## 2 — Git is working

- [ ] `git --version` prints a version number without error
- [ ] You have a local clone of the project you are bringing to the session

---

## 3 — Your project is on a branch you can experiment on

The Building exercises use git to isolate work and review changes. You need a branch that is
safe to experiment on — one where you can make commits, create worktrees, and discard work
without affecting your main line.

- [ ] You have created or checked out an experiment branch (not `main` or `master`)
- [ ] `git status` reports **nothing to commit, working tree clean** on that branch

If your working tree is not clean, stash or commit the in-progress work before the session.

---

## 4 — Your plan is present and openable

The Building part works from the execution-ready plan you produced in Part 1 — Planning. You
will need it open during the exercises.

- [ ] You have the plan file (your decomposed task list) accessible on your machine
- [ ] You can open it in a text editor or view it in the terminal
- [ ] The plan passes the [Planning completion checklist](../planning/completion-checklist.md) —
      every item is checked (the plan is execution-ready)

If the plan is not yet execution-ready, complete Part 1 — Planning before this session. The
Building exercises require a real plan with individually-verifiable tasks.

---

## 5 — Git worktrees work on your machine

The Building exercises use git worktrees to create an isolated workspace for autonomous runs —
so an agent can work without touching the files you have open in your editor. This is the item
most likely to produce a surprise, so verify it explicitly.

Run these three commands in your project's repository:

```
git worktree add ../preflight-test HEAD
git worktree remove ../preflight-test
git worktree list
```

- [ ] `git worktree add ../preflight-test HEAD` completes without error and creates the directory
- [ ] `git worktree remove ../preflight-test` removes it without error
- [ ] `git worktree list` shows only your main working tree (the test worktree is gone)

If any step fails, see the troubleshooting notes below.

**Troubleshooting worktrees:**

- *"fatal: already exists"* — the `preflight-test` directory already exists from a prior run.
  Remove it manually (`rm -rf ../preflight-test` on Unix / `rmdir /s /q ..\preflight-test`
  on Windows) and try again.
- *"not supported"* or *"bare repository"* — your git version may be older than 2.5 (worktrees
  require git 2.5+). Run `git --version` and update git if needed.
- *Permission denied on Windows* — open your terminal as the same user who owns the repository;
  do not mix admin and non-admin shells.

---

## Ready check

Before the Building session (or before you open the exercises self-paced):

- [ ] All five sections above are fully checked
- [ ] Any failures have been resolved

**If joining a live session:** reply "ready" to confirm you have passed all checks, or flag
which item you are stuck on so it can be addressed before the session starts. Do not wait until
the session begins to discover a worktree failure.

**If working self-paced:** do not advance to the exercises until every item above is checked.
The exercises assume a clean working tree, a working worktree command, and an execution-ready
plan — starting without these will stall the work mid-exercise.

When you are fully checked: open the [Building participant page](./index.md) and begin.
