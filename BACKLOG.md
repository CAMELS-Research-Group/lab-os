# Lab Backlog

Shared queue for **cross-repo / lab-level** work (repo-scoped work stays in that repo's
issues — see the routing rule in
[docs/proposals/2026-07-16-lab-wide-backlog.md](docs/proposals/2026-07-16-lab-wide-backlog.md)).
New items follow [templates/backlog-item.template.md](templates/backlog-item.template.md);
anyone may add to Inbox, shaping an item requires an owner. Groomed at each sprint boundary.

Read the **Index** first — it is the "what's ready right now" surface. Full items follow.
`## Items` is the last section: every `## ` heading below it must be an item heading.

## Index

<!-- generated from the Item blocks — edit those, then run:
     python3 scripts/backlog_lint.py --write-index -->

| id | title | owner | size | status |
|---|---|---|---|---|
| B1 | Convention-setting exercise | Jean | M | ready |
| B2 | Workshop process + feedback iteration | Watson | M | ready |
| B3 | Lab-wide backlog (this file) | Kiara | S | in-progress |
| B4 | Time-boxing PRDs / working sessions | Arya | S | ready |
| B5 | Backlog-lint (CI hygiene check for BACKLOG.md) | Kiara | M | in-progress |
| B6 | Decide the context-gc plugin's author identity | Watson | S | ready |
| B7 | Pin the context-gc provenance reference to a URL and sha | Watson | S | ready |
| B8 | Correct the flush-regression test's platform in its title | Watson | S | ready |
| B9 | Sweep the remaining fork-era staging-surface residue | Watson | S | ready |

## Inbox

<!-- Raw, unshaped ideas from the 2026-07-08 retrospective. Move one up to Items once it
has a Problem and a Done-when and an owner. -->

- UX pass on work surfaces (tasks / reviews) — retro "Lacked: user-friendly interfaces"
- Gamify progress updates — retro Try
- Working pairs — retro Try
- Multiple subscriptions — retro Try
- Virtual servers — retro Try
- Interoperable milestones / handing off between workstreams — retro Try
- Bot routines — retro running task

## Items

## B1 — Convention-setting exercise

- **Problem:** the lab has conventions scattered across repos and heads; no shared mechanism
  for proposing, ratifying, and reinforcing them (retro "Longed for: conventions/runbooks")
- **Who it helps:** every member and every agent session that inherits the rules
- **Value:** the retro's most-demanded gap; unblocks consistent multi-agent workflows
- **Owner:** Jean
- **Rough size:** M
- **Done when:** a convention-proposal template exists and one convention has gone
  proposal → ratified → written into a `.claude/rules/` or handbook page via it
- **Depends on:** —
- **Status:** ready

## B2 — Workshop process + feedback iteration

- **Problem:** workshop format needs optimization (retro "Lacked: workshop optimization");
  no feedback loop from session to session
- **Who it helps:** workshop facilitators and participants
- **Value:** workshops are the lab's onboarding + convention surface; compounding return
- **Owner:** Watson
- **Rough size:** M
- **Done when:** the next workshop's committed process notes (under `docs/workshops/`)
  record a process change traced to the previous session's collected feedback
- **Depends on:** —
- **Status:** ready

## B3 — Lab-wide backlog (this file)

- **Problem:** open lab work routes through one maintainer's personal file (retro Drop:
  "Watson as Backlog")
- **Who it helps:** whole team — shared visibility, anyone can groom or claim
- **Value:** retires a voted-down bottleneck; standing home for the other retro items
- **Owner:** Kiara
- **Rough size:** S
- **Done when:** the first sprint-boundary grooming commit touching this file is on
  lab-os `main`
- **Depends on:** —
- **Status:** in-progress

## B4 — Time-boxing PRDs / working sessions

- **Problem:** PRD drafting and working sessions run unbounded (retro Try, assigned Arya)
- **Who it helps:** anyone running a planning or pairing session
- **Value:** predictable sessions; complements the 2-week sprint Keep
- **Owner:** Arya
- **Rough size:** S
- **Done when:** a time-box convention (target durations per session type) is committed
  under `docs/` and one committed PRD cites the time-box it ran under
- **Depends on:** —
- **Status:** ready

## B5 — Backlog-lint (CI hygiene check for BACKLOG.md)

- **Problem:** the backlog's item schema + readiness bar ("Done when" = one checkable
  condition) are enforced only by grooming discipline; malformed items, a stale Index,
  dangling `Depends on`, and unsplit `L` items marked `ready` merge silently. No CI
  equivalent to `log-lint` / `docs-budget` / `merge-bar-check`.
- **Who it helps:** everyone who adds or grooms items — and the agent sessions that add
  them; keeps the Index a trustworthy "what's ready" surface.
- **Value:** makes the readiness bar true *by construction* instead of by memory; squarely
  the retro's "Tooling for Agents / Power Users" gap; reuses the lab's existing lint pattern.
- **Owner:** Kiara
- **Rough size:** M
- **Done when:** a PR that violates the item schema (missing field / bad `Done when` / stale
  generated Index / dangling `Depends on` / `L` marked `ready`) fails CI, and a well-formed
  one passes — running on every PR alongside the sibling lints.
- **Depends on:** B3
- **Status:** in-progress

## B6 — Decide the context-gc plugin's author identity

- **Problem:** `.claude/skills/context-gc/.claude-plugin/plugin.json` names a personal author
  with a personal email on an org asset that `link-lab-assets.sh` deploys onto every member's
  machine and that `ATTRIBUTION.md` files under "Original lab work"; the replacement value is
  not sourceable from this tree — no org contact address exists anywhere in it
- **Who it helps:** every member who installs the plugin, and anyone reading its manifest to
  find who owns it
- **Value:** an org-deployed asset should carry the identity that actually maintains it;
  deciding once sets the pattern for the plugins that follow
- **Owner:** Watson
- **Rough size:** S
- **Done when:** the `author` field in `.claude/skills/context-gc/.claude-plugin/plugin.json`
  carries a value the lab has ratified for an org-deployed asset
- **Depends on:** —
- **Status:** ready

## B7 — Pin the context-gc provenance reference to a URL and sha

- **Problem:** the `context-gc` entry under "Original lab work" in `.claude/skills/ATTRIBUTION.md`
  names a bundle path in a personal fork with no URL and no commit sha, so a reader cannot resolve
  it; under fold-and-delete the referenced bundle directory may no longer exist, so the pin likely
  has to be a git-history reference rather than a live path
- **Who it helps:** any reviewer or auditor tracing where a deployed plugin came from
- **Value:** provenance that cannot be resolved is not provenance — the review-as-code boundary on
  `.claude/skills/**` rests on it
- **Owner:** Watson
- **Rough size:** S
- **Done when:** the `context-gc` provenance entry in `.claude/skills/ATTRIBUTION.md` resolves to a
  reachable URL at a fixed commit sha
- **Depends on:** —
- **Status:** ready

## B8 — Correct the flush-regression test's platform in its title

- **Problem:** the large-manifest flush test in `.claude/skills/context-gc/test/recover.test.mjs`
  is titled as a Windows race, while its own inline comment and `src/recover.mjs` both state the
  race is a darwin condition (pipe writes are synchronous on Linux and Windows, asynchronous on
  macOS) — on the exact test that justifies the `macos-latest` CI leg
- **Who it helps:** anyone reading the test to learn which platform the guard protects, and
  whoever next prunes CI legs
- **Value:** a title naming the wrong platform argues for dropping the leg that actually catches
  the bug
- **Owner:** Watson
- **Rough size:** S
- **Done when:** the flush-regression test title in
  `.claude/skills/context-gc/test/recover.test.mjs` names darwin/macOS, matching its own comment
  and `src/recover.mjs`
- **Depends on:** —
- **Status:** ready

## B9 — Sweep the remaining fork-era staging-surface residue

- **Problem:** re-pointing the load-bearing taxonomy staging pointer at Caravan left adjacent
  fork-era text standing on related facts — where derivers, bundles and skills live, and index
  currency. Known locations: `reference/code-quality-taxonomy.md:5-6` and `:30-31`;
  `reference/specialist-dispatch.md:6`, `:16`, `:117`; `.claude/agents/ATTRIBUTION.md:56`, which
  has the workspace fork inheriting agent bodies through `git pull upstream main` where post-D17
  the inheritor is Caravan. Review routed these out of the re-pointing PR to keep it
  single-concern, and they were tracked nowhere until this item. One related surface needs a
  decision rather than an edit and is out of scope here: the `project_log.md` Standing Decisions
  line "2026-07-24 12:40 — … taxonomy staged in the fork, not yet carried · #61" is stale, but its
  entry is merged and therefore immutable, so retiring it takes a superseding entry.
- **Who it helps:** anyone — member or agent session — who reads these files to find where an
  asset is staged, and lands in a repo the lab no longer edits
- **Value:** the cutover is only half-landed while the pointers disagree; each stale mention is a
  wrong-repo edit waiting to happen, and the set is small and known now
- **Owner:** Watson
- **Rough size:** S
- **Done when:** no line in `reference/code-quality-taxonomy.md`, `reference/specialist-dispatch.md`,
  or `.claude/agents/ATTRIBUTION.md` names the workspace fork as the staging surface or as the
  inheritor of vendored copies
- **Depends on:** —
- **Status:** ready
