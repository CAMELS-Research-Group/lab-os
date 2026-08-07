# lab-os — project log

Format: lab standard, `lab-os/.claude/rules/03-logging.md`. Skeleton per
`lab-os/templates/project_log.template.md` (normative — `log-lint` parses this structure).
The `## Standing Decisions` and `## Entries` headings are load-bearing lint anchors: exact
text, one each, never renamed. Entry headers are the only other `##` headings allowed.

## Standing Decisions

- 2026-08-06 15:10 — PR #68 remediation: renderers fail closed, backlog-views enforced · #68
- 2026-08-06 14:00 — Backlog-lint fails closed on structural defects · #67
- 2026-08-06 12:32 — Lab-wide backlog: cross-repo open work routes to BACKLOG.md · #55
- 2026-07-31 14:03 — spec-plan-analyzer originates in lab-os and derives standards at read time · #61
- 2026-07-24 12:40 — Specialist panel ports to lab-os; taxonomy staged in the fork, not yet carried · #61
- 2026-07-24 16:20 — lab-os owns shared Claude skills; deploy is user-scope symlinks · #59
- 2026-07-23 11:13 — Backlog-lint enforces BACKLOG.md item hygiene via CI · #67
- 2026-06-23 07:51 — Plans track at the fork level; only project code nests · #44
- 2026-06-23 06:30 — Fork-of-lab-os is the default Claude-powered dev home · #43
- 2026-06-23 03:05 — Building sample plan ships as a facilitator-only fallback · #42
- 2026-06-19 05:58 — Workshop Program supersedes onboarding-project and one-day Building · #39
- 2026-06-13 15:00 — Handbook content rework gates tester launch · #25
- 2026-06-12 12:00 — Plan-execution logs close with their shipping PR · #18
- 2026-06-11 19:45 — Site owns human-facing docs · #15
- 2026-06-10 21:54 — Split the combined rule into 03-logging.md and 04-docs.md · #9
- 2026-06-10 17:45 — Adopt lab-wide logging & documentation standard · #6

## Entries

---

## 2026-08-06 15:10 — PR #68 remediation: renderers fail closed, backlog-views enforced

**Decision:** Per Watson's 2026-08-04 panel review of #68: both renderers refuse to render
(exit 1, script-failure lane — never the staleness exit 3) any backlog with parse errors,
zero Item blocks despite non-empty text, or Index/Item id drift. One shared
`source_integrity_errors` + `ready_unblocked` copy lives in `backlog_view.py`, imported by
`backlog_digest.py`; long-term home is `backlog_lint` beside the parser, deferred so this
branch does not fork #67's in-flight file. Titles escaped in Mermaid/table output; digest
CLI errors cleanly on a missing backlog and is self-tested; fixtures moved to `tests/`;
dashboard regenerated; `standards.yml` flips `backlog-views` to `enforce: true`.
**Why:** the review's fail-open Blockers all reduced to "a broken source renders as a
plausible artifact on a green job"; guarding every entry point closes the class, not the
instances. Enforce flip: warn-only-until-first-green is satisfied once the dashboard's
`--check` is green at the same head (docs-budget precedent, 2026-06-10).
**Alternatives:** land the guards in `backlog_lint` now (forks #67); bare `deps[id]`
KeyError only (fails, but names nothing); keep warn-only until #67's lint flip (leaves the
drift class open for the whole warn period).
**Refs:** #68 (review at head 877c602); #67 (parser seam)

---

## 2026-08-06 14:00 — Backlog-lint fails closed on structural defects

**Decision:** Remediating the #67 review round: Items-section text the parser cannot
attribute to an item is now a hard error and stays leak-scanned (it was silently
dropped between `## Items` and the first heading, and inside any block whose heading
failed to parse), and `--write-index` refuses to regenerate while structural parse
errors exist instead of deleting the unparsed blocks' Index rows under a success
message. Same posture extended mechanically: `/home/` joins the leak tripwire, an
unparseable `Depends on` value errors rather than reading as "no dependencies", a
template rename of a rule-bearing field label fails the run, CI annotations anchor
to file+line, and the workflow rejects unrecognized `enforce` values.
**Why:** Both blockers silently inverted the module's own contract — the leak
tripwire was disabled exactly where the file was malformed, and the documented
repair command destroyed committed rows while reporting success. The generalizable
rule: a derived projection is only safe to regenerate from a source that fully
parsed, and ambiguous input fails closed, matching the schema path's existing
posture.
**Alternatives:** fix once on #68 and close this PR (rejected — the review that
found the defects binds here; #68 rebases); warn on unattached text (rejected —
unowned text is exactly what escapes every field-level check).
**Refs:** #67; scripts/backlog_lint.py

---

## 2026-07-23 11:13 — Backlog-lint enforces BACKLOG.md item hygiene via CI

**Decision:** A `backlog_lint` CI check (sibling to `log-lint` / `docs-budget`) validates
`BACKLOG.md` unconditionally on every PR (via `standards.yml`, like the sibling lints):
required fields, a single-condition non-placeholder `Done when`, the status ladder, size
(`L` never `ready`), the Index as a generated projection of the Item blocks (committed
table must byte-match the render), and `Depends on` referential integrity + acyclicity.
Warn-only until first green, then enforcing; `backlog-lint:override` label for exceptions.
Schema parsed from `templates/backlog-item.template.md` (single source, fail-closed); the
`Done when` check is structural, treating a missing artifact reference as a warning, not a
failure; behavior documented in the tooling-tour, not a `.claude/rules/` file. B5.
**Why:** the backlog's readiness bar was enforced only by grooming discipline; a CI check
makes it true by construction, like the other lints. Warn-first + structural
`Done when` avoid false-failing legitimate items. A derived Index kills the
dual-representation drift class instead of policing it.
**Alternatives:** fold under B1 (conflates the tool with the proposal→ratify mechanism it
rides); a `.claude/rules/` entry (rule-budget cost; a lint's behavior is not a hard rule);
hand-authored Index + reconciliation check (leaves two authoritative copies).
**Refs:** #67; docs/prds/backlog-lint.md

---

## 2026-08-06 12:32 — Lab-wide backlog: cross-repo open work routes to BACKLOG.md

**Decision:** Cross-repo / lab-level open work routes to a shared `BACKLOG.md` at the
lab-os root (index → inbox → items per `templates/backlog-item.template.md`); repo-scoped
work stays in that repo's issues. Retires the maintainer-personal backlog as the team
mechanism, per the 2026-07-08 retrospective (Drop: "Watson as Backlog"; Try: "Lab Wide
Backlog", assigned Kiara). Groomed at each 2-week sprint boundary.
**Why:** Single-owner routing made open lab work invisible and ungroomable by the team;
the format was already taught in the building-workshop answer key, so adoption cost is a
promotion, not an invention. Full tradeoffs: `docs/proposals/2026-07-16-lab-wide-backlog.md`.
**Alternatives:** GitHub Projects org board (new tooling surface, content leaves the
repo); issues-only (cross-repo work homeless); status quo (voted down at the retro).
**Refs:** #55, https://github.com/CAMELS-Research-Group/lab-os/blob/main/docs/proposals/2026-07-16-lab-wide-backlog.md

---

## 2026-07-31 14:03 — spec-plan-analyzer originates in lab-os and derives standards at read time

**Decision:** The ENG doc-tier specialist (`spec-plan-analyzer`, planning-bundle review) lands with
the vendored four instead of waiting for its Phase-2 slot, and is **lab-authored here** — the first
agent body lab-os originates rather than vendors; the fork inherits it by `pull upstream`. It
restates no checklist: it resolves `04-docs.md` § ENG and `03-logging.md` in the repo under review
and derives its checks, returning a named not-run dimension where neither resolves. Dispatch is
path-based via a fail-closed per-repo ENG path registry.
**Why:** Bundle shape is repo- and version-specific — lab-os defines PRD/design/plan under
`docs/work/`, the fork a four-file `_specs/` bundle. A restated checklist would violate `04-docs.md`
§ Single source and go stale at the next rules sync; read-time derivation is correct on both.
**Alternatives:** Restate the fork's bundle shape — rejected: it would review lab-os bundles against
criteria lab-os does not hold. Hold for Phase 2 behind the dry-run gate — rejected on request; that
phasing de-risked the *lifted* bodies, and this one is authored.
**Refs:** #61

---

## 2026-07-24 12:40 — Specialist panel ports to lab-os; taxonomy staged in the fork, not yet carried

**Decision:** The four vendored specialist review agents and their dispatch contract
(`reference/specialist-dispatch.md`) are carried here, so a bare clone resolves the panel the review
skills dispatch. `reference/code-quality-taxonomy.md` is **not carried by this slice** — ownership
is unaffected: upstream lab-os owns its convention and canonical bytes like every other manifest
asset, and the fork is only the staging surface where those bytes are edited and where the
fork-side `scripts/rules_sync.py` (§ Manifest) vendors them into member repos. Wherever it does
not resolve, the finding schema's taxonomy citation is absent and merge
dedup falls back to the specialist's dimension. The agent bodies in `.claude/agents/**` are
byte-owned **here** — the fork inherits them via `git pull upstream main`, never by back-port
(`.claude/agents/ATTRIBUTION.md` § Byte ownership).
**Why:** The agents were unreachable from a lab-os-rooted dev home. The taxonomy's absence is a
carry gap, not an ownership split — its sync header already names lab-os as owner.
**Alternatives:** Carry the taxonomy too — rejected on scope, not merits: it belongs with the
rules-parity sync (#58) that lands the rules it cites. Omit the agents entirely —
rejected: runtime `DEV_ROOT` resolution already makes them optional.
**Refs:** #61, #59

---

## 2026-07-24 16:20 — lab-os owns shared Claude skills; deploy is user-scope symlinks

**Decision:** lab-os is the source of truth for the lab's shared Claude Code skills and commands; they
live in-repo (`.claude/skills/`, `.claude/commands/`, `.claude/scripts/`) and deploy user-scope via
`link-lab-assets.sh`, which symlinks them into `~/.claude/`. Recorded when #59 added `ATTRIBUTION.md`,
which states the convention.
**Why:** A clone or fork is self-contained with no marketplace install, and `git pull upstream` keeps
skills current. The load-bearing consequence: because deployment is user-scope, merged skill content
later executes as *instructions* under every member's identity, in every session — so `.claude/skills/**`
review rigor is a security boundary, and a skills PR is read as code, not docs.
**Alternatives:** Marketplace-plugin distribution (a separate repo the lab installs from) — rejected,
reintroduces the install step and a `plugin.json` surface the in-repo model avoids. Per-repo vendoring
(each member repo carries its own copy) — rejected for shared skills; it multiplies drift with no
owning source. (A single skill still vendors a sibling's file where a runtime cross-skill dependency
would fail worse — `ATTRIBUTION.md`.)
**Refs:** #59, .claude/skills/ATTRIBUTION.md

---

## 2026-06-23 07:51 — Plans track at the fork level; only project code nests

**Decision:** Refines the plan/project homing of #43. Methodology artifacts track in the fork itself,
not a nested repo — the plan and backlog at `_plans/`, the dev-home project log at the fork root. Only
the project *code* is re-homed as a separate gitignored nested repo. #43 homed "the plan/project"
together in the nested repo, conflating two artifacts with different needs.
**Why:** The fork is the methodology/coordination home, and plans are methodology — matching how the
lab already works (the dev root tracks the backlog and plan packets, not per-project). #43's
anti-coupling reason — don't couple `git pull upstream` with project history — bites for a full
codebase, not a handful of plan files in a new path upstream never touches. Splitting keeps that
benefit for the code while a dev-home session sees every plan.
**Refs:** #44, site/docs/getting-started/index.mdx, site/docs/workshops/building/pre-flight.md, docs/workshops/program/design.md

---

## 2026-06-23 06:30 — Fork-of-lab-os is the default Claude-powered dev home

**Decision:** Onboarding now forks lab-os (clone fallback) and uses that fork as the participant's
primary dev home, replacing the clone-as-rules-subdir + junction model. Rules live natively in the
fork; `git pull upstream main` keeps them current. A light cleanup makes the fork their own (reset
project log, drop lab-os's own design history + handbook site). Bring-your-own-project is preserved:
the plan/project is re-homed as its own gitignored repo nested in the fork, keeping a clean history
and inheriting the rules. The junction model is retained as the documented multi-repo power-user path.
**Why:** A fork gives a personal, push-able copy (real PR target, upstream-syncable rules) and drops
the most failure-prone bootstrap step. Nesting the project as a separate gitignored repo recovers the
junction's one benefit — lab tooling kept separate from project work — without it.
**Alternatives:** Junction/multi-repo dev-root — retained as the power-user path, not the default.
Commit the project into the fork (monorepo) — rejected, couples upstream pulls with project history.
**Refs:** #43; #42 (sample plan remains the facilitator-only fallback, unchanged by this)

---

## 2026-06-23 03:05 — Building sample plan ships as a facilitator-only fallback

**Decision:** Added `docs/workshops/building/sample-plan.md` — a pre-baked three-task plan (re-home fork identity · brand the handbook · add a backlog/planning surface) that a participant who reaches Building without a plan of their own runs against their own fork. ENG-tier under `docs/`, not published to the site and not in the sidebar; participants point Claude at the file from their fork's CLI. The fork's `cd site && npm run build` (broken-link-throw) is the gate for all three tasks.
**Why:** The mixed-cohort Building kickoff has newcomers who finish setup with no execution-ready plan and so can't practise the three execution modes. A small real plan unblocks them. Kept non-published to hold the program's bring-your-own-project line ("no prescribed sample project") on the public surface — the fallback exists for the blocked without the handbook advertising a sample as the path. Realises the fallback-starter the 2026-06-19 program decision deferred.
**Alternatives:** Publish as a participant page — rejected, contradicts the no-prescribed-sample stance on the public site. Demo-only build by the facilitator, no participant plan — rejected, leaves newcomers watching instead of practising.
**Refs:** #42, docs/workshops/building/sample-plan.md, Development/_packets/lab-os/workshop-program/

---

## 2026-06-19 05:58 — Workshop Program supersedes onboarding-project and one-day Building

**Decision:** The three-part Workshop Program (Planning → Building → Closeout), published under `site/docs/workshops/`, supersedes the two-week onboarding-project sandbox (now a redirect stub into the program) and the standalone one-day Building-with-Claude material (absorbed into the Building part's exercises). Facilitator runbooks are internal under `docs/workshops/`, not published.
**Why:** One coherent bring-your-own-project arc on a single self-paced + live-facilitated handbook surface, instead of a scattered sandbox plus a one-day track that diverge and double the maintenance. Phase-0 design lock approved by Watson at the overnight-run launch.
**Refs:** #39, Development/_packets/lab-os/workshop-program/

---

## 2026-06-13 15:00 — Handbook content rework gates tester launch

**Decision:** The handbook content + IA is being reworked across all seven pages before testers
are invited; tester launch now waits on the rework. This supersedes the prior round's deferral —
page-content restructuring left to play-test friction data, with launch "unblocked" after the
chrome/IA round. The rework is decomposed: a backbone authoring-conventions round first, then
per-page rounds (Getting Started next).
**Why:** The deferral assumed the existing content was good enough for a first cohort and that
friction data should drive structural change. On review of the shipped site, the content needs
reframing (Working-with-Claude → SDD lifecycle; Onboarding Project → Onboarding Workshop), zero-tech
support, and terminal-vs-Claude command clarity before a tester runs the arc — gaps a cohort hits
immediately, not subtle friction worth waiting for.
**Alternatives:** Launch on current content, rework in parallel — rejected, testers would hit content
already judged inadequate. Partial gate (ship some, defer the reframes) — considered, rejected for a
clean gated rework.
**Refs:** #25, docs/superpowers/specs/2026-06-13-handbook-backbone-conventions-design.md, docs/superpowers/specs/2026-06-12-handbook-frontend-design-round-design.md
