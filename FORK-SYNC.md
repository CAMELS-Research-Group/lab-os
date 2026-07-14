# Propagating lab-os changes into forks

How a downstream fork (e.g. `WatsonWBlair/Agentic_Workspace`, which uses lab-os as its dev-home
base) pulls convention updates from this repo. Source of truth for the fork relationship:
each fork's own `CLAUDE.md` ("Tooling" / "Project lineage").

## The relationship

- **Upstream:** `CAMELS-Research-Group/lab-os` — the conventions source (this repo).
- **Fork:** adds this repo as the `upstream` remote alongside its own `origin`.

```sh
git remote add upstream https://github.com/CAMELS-Research-Group/lab-os.git
git fetch upstream main
```

## Why not a blanket `git pull upstream main`

A fork and lab-os generally have **unrelated git histories** (the fork was seeded, not
`git clone`d from a shared root) and their trees diverge — a fork extends the rules, adds its own
`.claude/rules/10+`, and carries project code lab-os never had. A whole-tree merge therefore
conflicts on nearly every shared path and drags in files the fork deliberately dropped. Propagation
is **selective**, not wholesale.

## Selective propagation (the supported path)

Cherry-pick the specific upstream commit(s) onto the fork's working branch. Cherry-pick applies the
patch, so it works across unrelated histories as long as the touched paths apply cleanly:

```sh
git fetch upstream                      # or: git fetch upstream <branch>
git checkout -b sync/<slug> origin/main
git cherry-pick <upstream-sha>          # repeat / range as needed
# resolve any path-level conflicts, then open a PR on the fork
```

For a routine convention bump touching only shared files (e.g. `.claude/rules/`, `PR-LIFECYCLE.md`),
the cherry-pick lands clean when the fork's copy of that file matches lab-os. When the fork has
locally extended a shared file, resolve the conflict in favour of keeping the fork's additions plus
the upstream change.

## Verifying a change propagates

Confirm a given file is a clean-apply target before syncing:

```sh
git diff --quiet main upstream/main -- <path> && echo "identical — clean apply"
```

This file is itself the propagation check for branch `claude/lab-os-fork-propagation-9okrb6`:
authored in lab-os, cherry-picked into the fork to prove the path end-to-end.
