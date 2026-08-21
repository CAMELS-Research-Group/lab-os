# Skills — attribution & provenance

lab-os is the **source of truth** for the lab's shared Claude Code skills and commands. They live
here as project assets (`.claude/skills/`, `.claude/commands/`, `.claude/scripts/`) so a clone or
fork of this repo is self-contained — no marketplace install required. Deploy them user-scope with
`bash .claude/scripts/link-lab-assets.sh` (see each skill's Deployment section).

**Review rigor on `.claude/skills/**` is the security boundary.** These assets deploy user-scope, so
anything merged here later executes as instructions under every member's identity, in every session,
on every machine that has run the link script. A skills PR is not a docs PR: read it as code.

That boundary now carries **code, not only instructions.** A directory here that holds a
`.claude-plugin/plugin.json` is loaded by Claude Code as a plugin (`<name>@skills-dir`) rather than
as a skill, and a plugin's hooks, MCP servers and LSP servers are executed by the harness — not
offered to the model to choose. An instruction asset misbehaves only if Claude acts on it; a plugin
asset runs on every member's machine on its own trigger, before anyone reads a word of it. Review
a plugin PR as executable code, and check its tests as the safety argument they are — `plugin-tests`
runs them in CI (`.github/workflows/plugin-tests.yml`).

This file records third-party and cross-skill provenance for content vendored into skills. Each
carrier file also preserves the attribution in its own header.

## Original lab work

Assets without an entry in the third-party section below are original lab work by default; this
section records only the ones whose provenance needs stating explicitly.

- `context-gc` — authored in-lab (bundle
  `WatsonWBlair/Agentic_Workspace:_specs/lab-os/2026-07-02-context-gc/`, `Status: complete`);
  hardened through `/pr-review-loop` and `/pr-round` review on PR #78. No third-party
  attribution owed.
  **A plugin, not a skill** — it carries `.claude-plugin/plugin.json` and `hooks/hooks.json`,
  ships no `SKILL.md`, and is never model-invoked; see its `README.md`. It is the one asset in
  this tree that versions itself (`plugin.json`) rather than per §Versioning below.

## Third-party attribution

### Code-quality rubric — Cursor (MIT)

`pr-round/reference/rubric-universal.md` adapts structural-class content from Cursor's
"thermo-nuclear-code-quality-review" skill (github.com/cursor/plugins, MIT licensed, © 2026 Cursor):
tier 1 of `pr-round`'s three-tier review standard, restated project-neutrally so the skill works in
any repository. Other lab skills carry independently adapted copies of the same source material;
copies are expected to drift and none is canonical for another.

### pr-round vendored classifier

`pr-round/reference/classify-blockers.md` is a copy of the `pr-review-loop` skill's file of the same
name (maintained in the lab's workspace forks), taken once and stripped of the age ledger and
cross-pass fingerprint machinery, which a single-round skill has no use for. Vendored rather than
referenced so `pr-round` carries no runtime dependency on a sibling skill — a user-scope skill that
degrades when a neighbour is absent fails worse than two copies drifting.

## Versioning

Project skills carry no `plugin.json`. Version notes, where a skill needs one, live in the skill's
own `SKILL.md`.

**Vendored plugins are the exception**, because the rule above is about skills. A plugin is
identified and versioned by its own `.claude-plugin/plugin.json`, which Claude Code reads — the
version is machinery, not a note, so it cannot be relocated into prose. Bump it in that manifest
when the plugin's behaviour changes. Current: context-gc 0.0.1.
