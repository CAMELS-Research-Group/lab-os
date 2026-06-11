# lab-os: rename, handbook site, play-test launch — design

Status: draft — pending Watson's review.

## 1. Problem

Three connected gaps, surfaced 2026-06-11:

- **The name no longer fits.** `lab-rules` holds rules, templates, CI enforcement, new-member
  bootstrap, and the Mission-Control sandbox onboarding project — and is about to gain a
  stakeholder-facing handbook site. "Rules" describes one slice. Phase 2 (caller YAMLs in every
  lab repo) has not shipped, so renaming now is cheap; after phase 2 every repo references the
  name.
- **Onboarding is functional but undocumented for humans.** The pieces exist (BOOTSTRAP.md,
  WORKING-WITH-CLAUDE.md, templates) but there is no human-facing surface that explains the
  repo, its purpose, and its features to a new member or outside stakeholder. ONBOARDING-PROJECT.md
  — the centerpiece of the onboarding arc — is stranded on the unmerged `docs/onboarding-project`
  branch and invisible on main.
- **The onboarding has never been play-tested.** Watson wants 1–3 lab members to run the full
  arc and surface friction before the lab grows.

Target end-state (Watson's framing): the repo contains everything a new lab member needs to get
started with using Claude and contributing to lab work.

## 2. Decisions

| # | Decision | Alternatives rejected |
|---|---|---|
| D1 | Rename repo to **`lab-os`** | `lab-handbook` (docs-only connotation), `camels-handbook` (rebrands if lab name changes), `caravan` (not self-describing) |
| D2 | Docusaurus 3 site in `site/` inside this repo, GitHub Pages at `watsonwblair.github.io/lab-os` | Separate site repo (breaks "one repo a member clones"); wiki (no build-time link checking, weaker IA) |
| D3 | **Site owns human-facing docs.** BOOTSTRAP, WORKING-WITH-CLAUDE, ONBOARDING-PROJECT migrate into the site; root files become pointer stubs; rules stay AI-tier repo files | Site wraps repo md (brochure over a codebase); site renders repo md as-is (agent-dense prose shown to stakeholders) |
| D4 | Site is the **front door**: play-test starts at the site, so an MVP site is on the critical path | Play-test from repo markdown in parallel (faster start, but doesn't test the surface stakeholders will see) |
| D5 | **Approach A — lift-and-link MVP.** Migrate with light edits, write only the landing page fresh, start testers ~day 2–3; stakeholder polish happens during the play-test informed by friction data | Polish-first (~1 week delay, polishing before evidence); two-track fast start (testers onboard through a half-migrated surface) |
| D6 | **Claude-guided setup.** Getting Started instructs the member to install Claude Code, then paste a bootstrap prompt that has Claude interview them and perform setup with them: personalize `global-CLAUDE.template.md` → `~/.claude/CLAUDE.md`, seed dev-root CLAUDE.md, wire the rules junction, verify. Templates remain source of truth; the prompt references them | Hand-edited templates (misses the point — onboarding *to Claude* should be done *with* Claude; also the highest-friction step left untested) |
| D7 | Feedback = **issues + retro**: friction issue template (auto-labeled `playtest`) filed per stall, plus an end-of-arc retro writeup | Friction journal (no in-flight visibility); live shadowing (doesn't scale); time-box-only metric (loses the qualitative findings) |

## 3. Rename mechanics

Order of operations:

1. `gh repo rename lab-os` — GitHub auto-redirects old URLs and git remotes.
2. Rename local folder `Development\lab-rules` → `Development\lab-os`.
3. Recreate the dev-root junction: `Development\.claude\rules` → `Development\lab-os\.claude\rules`
   (junctions do not survive target renames). Do this between Cowork sessions — a stale junction
   silently drops rule-loading.
4. Point the local git remote at the canonical new URL.
5. In-repo reference sweep: README, BOOTSTRAP.md, WORKING-WITH-CLAUDE.md, PR-LIFECYCLE.md,
   `.claude/rules/*`, `templates/*` (the dev-root and global CLAUDE.md templates both name
   lab-rules), test fixtures if any.
6. Same-day external updates (they steer every session): Watson's live `~/.claude/CLAUDE.md`
   and `Development\.claude\CLAUDE.md`; Claude's memory files.
7. Deferred to phase 2 (covered by GitHub's redirect until then): pr-review-agent SPEC/SETUP,
   per-repo `pr-review.yml` checkout references, future caller YAMLs.

Logging: irreversible/external event → entry in repo log **and** lab log.

## 4. Site architecture

- Docusaurus 3, classic preset, TypeScript config, in `site/`.
- Deploy: new `deploy-site.yml` GitHub Action — build + GitHub Pages deploy on push to main,
  path-filtered to `site/**`. Site URL: `watsonwblair.github.io/lab-os`.
- The Docusaurus build fails on broken internal links — this is the link-rot gate for migrated
  docs. PR-time check: the same workflow runs build-only on PRs touching `site/**`.
- Node artifacts (`node_modules/`, `build/`, `.docusaurus/`) gitignored.
- `docs-budget` unaffected: site pages are ENG/Public tier, which the budget rule does not cover.
- Repo stays public (already is); the site makes lab process discoverable, not newly public.

## 5. Information architecture (MVP)

| Nav item | Source | Work |
|---|---|---|
| Landing — what CAMELS is, what lab-os is, Start Here | new | written fresh (the one real writing task) |
| Getting Started — install Claude Code, paste the bootstrap prompt, Claude-guided setup (D6), verify | `BOOTSTRAP.md` + new prompt | migrate, restructure around the guided flow |
| Working with Claude — lab methods | `WORKING-WITH-CLAUDE.md` | migrate; overclaim/internal-reference scan (written for insiders) |
| The Onboarding Project — Mission-Control sandbox | `ONBOARDING-PROJECT.md` from `docs/onboarding-project` branch | content migrates directly into the site; the branch's README/log changes are superseded and discarded explicitly |
| The Rules, explained | new, short | human tour of 01–04, each section linking to the rule file as source of truth |
| How to play-test | new, short | the arc, the >15-min stall rule, the retro prompt |
| Setting up a new repo | new, thin | first-cut runbook: repo creation checklist (gitignore, repo-CLAUDE.md seed, project-log skeleton, caller YAML pointer) — the templates are its building blocks; iterated during the play-test (the sandbox project's first step exercises it) |
| Tooling tour — CI actions, templates, pr-review agent | new | **post-MVP**, written during the play-test window |

After migration: `BOOTSTRAP.md` and `WORKING-WITH-CLAUDE.md` at repo root become 3-line pointer
stubs to their site pages. README slims to AI/CI-consumer reference (junction setup, caller
YAML pattern, override semantics) + prominent site link for humans.

## 6. Single-source map (per 04-docs)

| Fact domain | Owner | Site's relationship |
|---|---|---|
| Human onboarding narrative | site | owns |
| Conventions / rules | `.claude/rules/` | restates-and-links ("source of truth: `01-workflow.md`") |
| Templates | `templates/` | links; bootstrap prompt references, never inlines |
| Log & doc standards | `03-logging.md` / `04-docs.md` | links |

## 7. Play-test protocol

- **Testers:** 1–3 lab members with existing Claude subscriptions and lab-repo access.
- **Arc:** site landing → Claude-guided environment setup → methods read → sandbox-project
  brainstorm/spec. Sandbox project itself runs in the tester's own disposable repo, created
  per the Setting-up-a-new-repo page.
- **Capture:** `.github/ISSUE_TEMPLATE/friction.yml` — where I was (page/step) · what I
  expected · what happened · severity; auto-labeled `playtest`. Stall >15 min = file an issue,
  then ask Watson. End of arc: retro writeup (what confused, what was missing, what you'd cut).
- **Success criteria:** each tester reaches (a) a working environment including a personalized
  `~/.claude/CLAUDE.md` produced through the guided flow, and (b) a brainstormed sandbox-project
  spec — both without Watson unblocking; every stall produces an issue.
- Inviting testers is comms under Watson's name — his action, not Claude's.

## 8. Sequencing

- **Day 1:** rename (§3) · site scaffold + deploy pipeline · landing page draft. Separate PRs
  per the merge bar (single concern each).
- **Day 2:** content migration (3 docs + rules-explained + how-to-play-test + repo-setup
  runbook + pointer stubs + README slim-down) · friction template.
- **Day 2–3:** Watson invites testers; play-test runs.
- **During play-test:** tooling tour, jargon scrub, friction fixes as `playtest` issues arrive.

Log entries: rename (event — repo + lab altitude); site-as-owning-home (standing decision);
play-test launch (event, lab altitude).

## 9. Out of scope

- Phase 2 rollout (caller YAMLs, log migrations, LSCA cleanup) — pinned, unchanged.
- Custom theming/branding; site search; doc versioning.
- Lab-wide issue/label standardization (spec §13 of the logging/docs design); the `playtest`
  label is a one-off, not a convention.
- Any rewrite of the rules files themselves.

## 10. Known gaps

- The bootstrap prompt (D6) is new surface with no prior art in the lab — expect the play-test
  to hit hardest here; that is intentional.
- Budget calibration of the slimmed README/pointer stubs is unverified until `docs-budget` runs.
- GitHub Pages must be enabled on the repo (one-time settings change — shared-state, flagged at
  execution time).
- WORKING-WITH-CLAUDE.md may reference private docs or insider context; the migration scan
  (§5) handles it, but depth of rewrite is unknown until read closely.
