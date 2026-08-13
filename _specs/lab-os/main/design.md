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
`docs_budget.py` / `merge_bar_check.py` / `backlog_lint.py` / `backlog_view.py` /
`backlog_digest.py`. `standards.yml` is lab-os's own caller and doubles as the copy-paste
template for member repos, which call the same workflows by `@main` rather than copying the
scripts — so there is never a second copy of a check to drift.

**`templates/`** — scaffolds: per-repo and personal `CLAUDE.md` seeds, the normative
`project_log.template.md` that `log-lint` parses, `docs/planning/` (per-slice bundle) and
`docs/main-bundle/` (per-scope current state), `backlog-item.template.md`, and the
`desktop-control-panel` app starter.

**`site/`** — the Docusaurus handbook, deployed to GitHub Pages by `deploy-site.yml`; it owns
the human-facing docs. `BOOTSTRAP.md` and `WORKING-WITH-CLAUDE.md` are pointer stubs into it.

**`_specs/lab-os/`** — this main bundle plus any in-flight dated slices.

## Contracts & schemas

- **Gate contract.** Every check script exits 0 pass / 1 violation / 2 usage error, carries a
  `--self-test`, and has an `enforce` input that separates warn-only from failing. lab-os's
  own caller runs `docs-budget`, `backlog-views` with `enforce: true`; `backlog-lint`
  warn-only.
- **Budget contract.** `CLAUDE.md` 8,192 B · each `.claude/rules/*.md` 5,120 B ·
  `project_log.md` 15,360 B. WARN above 1.0x, FAIL above 1.5x. Owned by `04-docs.md`
  §Tiers & budgets; `docs_budget.py` is the enforcer, not the source.
- **Log contract.** `project_log.template.md` is normative — `## Standing Decisions` and
  `## Entries` are exact-text lint anchors. Entries are immutable post-merge, reverse-chron,
  top-inserted; overflow moves oldest entries to `project_log_archive.md` as a verbatim block
  in a dedicated `chore: archive log overflow` PR.
- **Bundle contract.** `_specs/<scope>/<DATE>-<handle>/{prd,spec,plan,log}.md`, plus
  `design.md` when the slice meaningfully touches code. One `Status:` in the PRD header is
  the state marker. Terminal ⇒ fold file-to-file into `_specs/<scope>/main/`, then delete;
  git history is the archive.
- **Rules-sync contract.** Member repos hold verbatim copies of the `0x` rules and the
  manifest assets under a one-line sync header. The manifest and the drift check live in
  `scripts/rules_sync.py` **in Caravan**, not here; lab-os holds the canonical bytes.

## Known gaps (enforcement vs. intent)

- **`README.md` describes a pre-Caravan onboarding path.** It opens "conventions for
  `WatsonWBlair`'s lab repos" and presents forking lab-os as the default dev home. D16 made
  `CAMELS-Research-Group/Caravan` the shared dev home and D17 made it the rules staging
  surface. The rules are correct; the README has not caught up.
- **`templates/PRD.template.md` is a superseded scaffold** retained pending consolidation
  with `templates/docs/planning/prd.template.md`.
- **The `06-timeboxing` rule has no member-repo vendoring row.** `rules_sync.py`'s manifest
  (in Caravan) lists five base rules; `06` is not among them, so member repos do not receive
  it. Either the manifest or the rule's scope needs a decision.
- **`docs-budget` per-file budgets are near their ceiling on the rules tier.** `04-docs.md`
  sits above 1.0x against the current 5,120 B budget. A raise to 8,192 B plus an
  always-loaded aggregate cap is in flight.
- **Path-filtered CI jobs must never become required status checks.** A required context that
  never reports holds PRs at "Expected" forever. This is a live constraint on branch
  protection, enforced by convention rather than by anything mechanical.
- **`_specs/lab-os/` bundle statuses are not mechanically checked.** Nothing fails CI when a
  PRD `Status:` goes stale against its shipping PR; `spec-plan-analyzer` flags it only when a
  PR happens to touch the bundle.
