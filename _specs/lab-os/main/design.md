# lab-os — main design (implemented state)

**Scope:** lab-os · **Living** — the implemented-state authority.
**PRD:** [prd.md](./prd.md) · **Spec:** [spec.md](./spec.md) ·
**Plan:** [plan.md](./plan.md) · **Log:** [log.md](./log.md)

---

## Overview

lab-os is a conventions-and-tooling repository, not an application. It holds the canonical
bytes of the lab-wide rules, the review assets that apply them, the CI that scores adherence,
the scaffolds new work starts from, and the handbook that explains all of it to humans. It
ships no runtime.

Consumers reach it three ways: agents read the rules as always-loaded context; member repos
call its CI as reusable workflows; members read the handbook site. A fourth path — vendored
verbatim copies of the rules inside member repos — is driven from Caravan, not from here.

## Architecture

**`.claude/rules/`** — the always-loaded convention tier, numbered `0x` and owned here.
`01-workflow` (commit format, PR workflow, merge bar, doc-sync triggers) · `02-data-protection`
(gated data, PII, secrets, binary limits) · `03-logging` (altitudes, entry triggers, format,
immutability, overflow) · `04-docs` (single-source, tiers and budgets, ENG doc standards,
bundle lifecycle and the main bundle, rules numbering) · `05-agent-runtime` (forward-binding
HARD RULE for any asset hosting a local coding-agent runtime; nothing here hosts one) ·
`06-timeboxing` (session and agent task boxes; owning docs under `docs/`).

Per-repo rules number from `10+` and are never authored here.

**`.claude/agents/`** — specialist review agent bodies dispatched by the review skills:
`comment-analyzer`, `pr-test-analyzer`, `silent-failure-hunter`, `spec-plan-analyzer`,
`type-design-analyzer`. Part vendored from Anthropic's `pr-review-toolkit`, part
lab-authored; provenance in `ATTRIBUTION.md`. These bodies execute as instructions, so
review rigor over `.claude/agents/**` is a security boundary, not a style preference.

**`.claude/skills/`** — shared skills: `pr-round` (one round of PR work across every PR
connected to you) and `timeboxing`. `.claude/scripts/link-lab-assets.sh` symlinks skills and
commands into `~/.claude/` per machine so they resolve from any repo.

**`reference/`** — the contracts rules and skills derive from at read time rather than copy:
`code-quality-taxonomy.md` and `specialist-dispatch.md` (specialist triggers, per-pass cap,
model tier, finding schema, merge/dedup, degradation). Read-time derivation is why renaming a
rule section does not silently break a deriving agent.

**`scripts/` + `.github/workflows/`** — the adherence gates. Each check is a Python script
with a `--self-test` mode plus a reusable `workflow_call` wrapper: `log_lint.py` /
`docs_budget.py` / `merge_bar_check.py` / `backlog_lint.py` / `backlog_view.py`.
`backlog_digest.py` sits beside them but is not a check and has no `workflow_call` wrapper:
it posts a scheduled digest, and `backlog-digest.yml` runs on `schedule` /
`workflow_dispatch` only. `backlog_lint_selftest.py` holds `backlog_lint`'s self-test cases,
split out so the check itself stays inside its line budget. `standards.yml` is lab-os's own
caller and doubles as the copy-paste template for member repos, which call the same workflows
by `@main` rather than copying the scripts — so there is never a second copy of a check to
drift.

**`tests/`** — the fixture corpora every `--self-test` reads, one directory per check
(`backlog_digest`, `backlog_view`, `docs_budget`, `log_lint`, `merge_bar_check`). The
self-tests are the checks' own regression suite; there is no separate test runner.

**`tools/`** — one-off authoring utilities that are not gates and are not called by CI:
`build_deck.py` and its `README.md`.

**`templates/`** — scaffolds: three `CLAUDE.md` seeds (`repo-`, `global-`, and
`dev-root-CLAUDE.template.md`), the normative `project_log.template.md` that `log-lint`
parses, `docs/planning/` (per-slice bundle) and `docs/main-bundle/` (per-scope current
state), `backlog-item.template.md`, and the `desktop-control-panel` app starter.

**`docs/`** — long-form human docs that are not part of the site build: the workshop
program (`workshops/`), the timeboxing runbooks and quickref, `conventions-collection/`,
the generated `backlog-views.md` (written by `backlog_view.py --write`), and the legacy
planning surfaces `prds/`, `proposals/`, and `superpowers/{plans,specs}` that predate the
`_specs/` convention this scope codifies.

**`.claude/commands/`** — slash-command entry points (`pr-round.md`), symlinked into
`~/.claude/` by the same `link-lab-assets.sh` that deploys the skills.

**`site/`** — the Docusaurus handbook, deployed to GitHub Pages by `deploy-site.yml`; it owns
the human-facing docs. `BOOTSTRAP.md` and `WORKING-WITH-CLAUDE.md` are pointer stubs into it.

**`_specs/lab-os/`** — this main bundle plus any in-flight dated slices.

## Contracts & schemas

- **Gate contract.** Every check script exits `0` on pass and `2` on usage error, and carries
  a `--self-test`. **The violation code is per script, not universal:** `log_lint.py` and
  `merge_bar_check.py` exit `1` on any violation; `docs_budget.py` and `backlog_lint.py` exit
  `0` warn-only and `1` under `--enforce`; `backlog_view.py --check` reserves `3` for
  staleness and uses `1` for script/parse failure, so `backlog-views.yml` can branch on the
  two without reporting a broken source as staleness. An `enforce` input separating warn-only
  from failing is carried by the `docs-budget`, `backlog-lint`, and `backlog-views` wrappers
  only; `log-lint` and `merge-bar-check` declare no such input. Each wrapper under
  `.github/workflows/` owns its own posture, stated in its header. lab-os's own caller runs
  `docs-budget` and `backlog-views` with `enforce: true`; `backlog-lint` warn-only.
  `merge-bar-check` validates a PR body against `.github/pull_request_template.md`, the
  artifact that defines the checklist it scores.
- **Budget contract.** The per-surface byte budgets — `CLAUDE.md`, each
  `.claude/rules/*.md`, `project_log.md` — are owned by
  [`04-docs.md`](../../../.claude/rules/04-docs.md) §Tiers & budgets and are read there, never
  restated here. WARN above 1.0x, FAIL above 1.5x; `docs_budget.py` is the enforcer, not the
  source.
- **Log contract.** `project_log.template.md` is normative — `## Standing Decisions` and
  `## Entries` are exact-text lint anchors. Entries are immutable post-merge, reverse-chron,
  top-inserted; overflow moves oldest entries to `project_log_archive.md` as a verbatim block
  in a dedicated `chore: archive log overflow` PR. Owned by
  [`03-logging.md`](../../../.claude/rules/03-logging.md) §File structure & overflow.
- **Bundle contract.** `_specs/<scope>/<DATE>-<handle>/{prd,spec,plan,log}.md`, plus
  `design.md` when the slice meaningfully touches code. One `Status:` in the PRD header is
  the state marker. Terminal ⇒ fold file-to-file into `_specs/<scope>/main/`, then delete;
  git history is the archive. Owned by
  [`04-docs.md`](../../../.claude/rules/04-docs.md) §Bundle lifecycle & the main bundle.
- **Rules-sync contract.** Member repos hold verbatim copies of the `0x` rules and the
  manifest assets under a one-line sync header. The manifest and the drift check live in
  `scripts/rules_sync.py` **in Caravan**, not here; lab-os holds the canonical bytes.

## Known gaps (enforcement vs. intent)

- **`README.md` describes a pre-Caravan onboarding path.** It opens "conventions for
  `WatsonWBlair`'s lab repos" and presents forking lab-os as the default dev home. D16 made
  `CAMELS-Research-Group/Caravan` the shared dev home and D17 made it the rules staging
  surface — both owned by `WatsonWBlair/Agentic_Workspace`
  `_specs/lab-os/2026-08-06-spec-home-migration/decisions.md` §D16/§D17, with D17 carried
  forward in Caravan's `project_log.md` at 2026-08-10 19:40 (Caravan #2). The rules are
  correct; the README has not caught up. *Tracking:* [plan.md](./plan.md) §Blocked on the
  operator.
- **`templates/PRD.template.md` is a superseded scaffold** retained pending consolidation
  with `templates/docs/planning/prd.template.md`. *Tracking:*
  [`BACKLOG.md`](../../../BACKLOG.md) B13.
- **The `06-timeboxing` rule has no member-repo vendoring row.** `rules_sync.py`'s manifest
  (in Caravan) lists five base rules; `06` is not among them, so member repos do not receive
  it. *Tracking:* Caravan
  [B22](https://github.com/CAMELS-Research-Group/Caravan/blob/main/BACKLOG.md#b22--carry-06-timeboxingmd-through-the-vendoring-path-to-member-repos)
  (Caravan's `B<n>` namespace, not this repo's), which owns the `MANIFEST_FILES` edit and the
  `templates/repo-CLAUDE.template.md` mismatch that rides with it. B22 presumes `06` should
  vendor, reading `04-docs.md` §Rules numbering on verbatim copies; it does not pose the
  alternative of stating the rule's scope as lab-os-only.
- **`docs-budget` warns on the rules tier.** At least one `.claude/rules/*.md` file sits
  above 1.0x of the per-file budget in force, so the check reports a WARN on every PR. A
  budget raise plus an always-loaded aggregate cap is staged in Caravan and reaches this repo
  as a round-trip; whichever numbers are in force are the ones in
  [`04-docs.md`](../../../.claude/rules/04-docs.md) §Tiers & budgets. *Tracking:*
  [`BACKLOG.md`](../../../BACKLOG.md) B20.
- **Path-filtered CI jobs must never become required status checks.** A required context that
  never reports holds PRs at "Expected" forever. This is a live constraint on branch
  protection, enforced by convention rather than by anything mechanical. *Tracking:*
  [`BACKLOG.md`](../../../BACKLOG.md) B14.
- **`project_log.md` is over budget and close to failing CI.** `docs-budget` runs
  `enforce: true` in `standards.yml`, and the file sits inside the 1.5x FAIL threshold by a
  margin one more entry can spend; `python3 scripts/docs_budget.py --root .` reports the
  figures in force. The prescribed `chore: archive log overflow` PR (`03-logging.md` §File
  structure & overflow) is filed as
  [#82](https://github.com/CAMELS-Research-Group/lab-os/pull/82) and is stack-ordered ahead
  of this bundle. *Tracking:* [`BACKLOG.md`](../../../BACKLOG.md) B17.
- **`project_log.md`'s Standing Decisions index still lists superseded entries.**
  "2026-06-23 06:30 — Fork-of-lab-os is the default Claude-powered dev home · #43" and
  "2026-06-23 07:51 — Plans track at the fork level; only project code nests · #44" are
  indexed as standing, yet D16/D17 moved the dev home to Caravan. No local superseding entry
  has been written, so this bundle's current-state claims and the project-altitude index
  disagree; `03-logging.md` §Immutability & supersession describes the entry that would
  remove those index lines. *Tracking:* [`BACKLOG.md`](../../../BACKLOG.md) B18.
- **`docs/`'s legacy planning surfaces have no stated disposition.** `docs/prds/`,
  `docs/proposals/`, and `docs/superpowers/{plans,specs}` predate the `_specs/` bundle
  convention this scope codifies, and nothing states whether they are in scope, frozen, or
  slated to fold. Until that is decided, `_specs/lab-os/` is not the whole planning record
  for this repo. *Tracking:* [`BACKLOG.md`](../../../BACKLOG.md) B16.
- **`_specs/lab-os/` bundle statuses are not mechanically checked.** Nothing fails CI when a
  PRD `Status:` goes stale against its shipping PR; `spec-plan-analyzer` flags it only when a
  PR happens to touch the bundle. *Tracking:* [`BACKLOG.md`](../../../BACKLOG.md) B15.
- **The one dated slice carries fewer files than the bundle contract grandfathers.**
  `_specs/lab-os/2026-07-31-timeboxing/` holds `prd.md` and `log.md` only;
  [`04-docs.md`](../../../.claude/rules/04-docs.md) §ENG grandfathers *three*-file bundles,
  not two, so the slice is outside both the current contract and its grandfather clause.
  *Tracking:* [`BACKLOG.md`](../../../BACKLOG.md) B15.
