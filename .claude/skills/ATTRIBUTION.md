# Skills — attribution & provenance

lab-os is the **source of truth** for the lab's shared Claude Code skills and commands. They live
here as project assets (`.claude/skills/`, `.claude/commands/`, `.claude/scripts/`) so a clone or
fork of this repo is self-contained — no marketplace install required. Deploy them user-scope with
`bash .claude/scripts/link-lab-assets.sh` (see each skill's Deployment section).

This file records third-party and cross-skill provenance for content vendored into skills. Each
carrier file also preserves the attribution in its own header.

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
