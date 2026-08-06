---
description: "Push every connected PR through one round — review others', remediate your own"
argument-hint: "[<PR ref>…] [--limit N] [--no-skip] [--review-only] [--remediate-only] [--dry-run] [--concurrency N] [--no-specialists] [--comment-only] [--hand-back] [--merge-base]"
allowed-tools: ["Agent", "AskUserQuestion", "Bash", "Read", "Write", "Edit", "Glob", "Grep", "EnterWorktree"]
---

# /pr-round

Run the pr-round skill starting at Step 0 with `$ARGUMENTS`.

Resolve the skill root first — `~/.claude/skills/pr-round/` if present, otherwise
`.claude/skills/pr-round/` under the current repository — because this command runs from arbitrary
repositories, including nested project repos whose own root blocks the `.claude/` walk-up to the dev
home. (`<DEV_ROOT>` is derived *from* the skill root at Step 0.1, so it cannot name the fallback.)

Read `<SKILL_ROOT>/SKILL.md` for an overview, then execute `<SKILL_ROOT>/PROMPT.md` from Step 0.
Pass `$ARGUMENTS` as the arguments to parse in Step 0: zero or more PR references
(`lab-os#42`, `CAMELS-Research-Group/lab-os#42`, a pull URL, or a bare number) plus any flags.

With no PR reference, the roster is every open PR you authored plus every one you were asked to
review. Start with `--dry-run` to see the roster without making any write call.

See: `<SKILL_ROOT>/SKILL.md`
