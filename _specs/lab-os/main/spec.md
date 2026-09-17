# lab-os — main spec (decisions document)

**Scope:** lab-os · **Living** — the fold target for completing bundles' decisions.
**PRD:** [prd.md](./prd.md) · **Design:** [design.md](./design.md) ·
**Plan:** [plan.md](./plan.md)

Recover any folded bundle: `git log --all -- '_specs/lab-os/<bundle>/'`.

---

## What belongs here

The current design authority for this scope: every binding decision, stated once as a section
below, appended by any amendment act — a bundle's fold, a restructure of this bundle, or, when no
bundle exists to carry it, a decision written directly into a new section in the PR that makes it.
Two kinds of section. **Folded sections** are bundle-altitude decisions, added one per decision as
each bundle folds; their source is that bundle's own decision id. **Self sections** are decisions
this main bundle makes about itself — its path, its file set, the shape of this document — which
arrive with no dated bundle to fold from, sourced `_specs/lab-os/main/`.

This document also carries every decision formerly held in the lab log's `## Standing
Decisions` index, converted in place here rather than left on that surface. When a later decision
reverses an earlier one, the earlier section is rewritten to the current answer and names what it
replaced — there is no frozen row to annotate.

No bundle has folded yet, so every section below is either a self section or a converted standing
decision. That is the expected state for a scope whose decisions have so far been project-altitude,
and it is not a gap to be backfilled: see [prd.md](./prd.md) §Open questions on whether the
pre-Caravan dev home's terminal bundles enter this document at all.

## Decision summary

| # | Question | Resolution | Status |
|---|----------|------------|--------|
| R1 | Where does lab-os's main bundle live? | `_specs/lab-os/main/` — the workspace-root path shape, not the member-repo `_specs/main/` shape | **DECIDED** (§R1) |
| L1 | Is lab-os licensed, and how? | Apache-2.0 root `LICENSE`, with a `NOTICE` carving out vendored third-party assets under their own licenses | **DECIDED** (§L1) |
| L2 | Whose contributions does the Apache-2.0 grant cover, and from when? | Watson Blair's contributions in full; Kiara Jamal's, Arya Kurup's and Jean Moncayo's pre-2026-09-01 contributions are excluded pending recorded consent; everything from 2026-09-01 onward is Apache-2.0 for every author | **DECIDED** (§L2) |
| D1 | What are lab-os's always-loaded doc byte budgets? | `CLAUDE.md` 12 KB, each `.claude/rules/*.md` 8 KB, plus a new 48 KB aggregate cap over the always-loaded tier (the lab log excluded) | **DECIDED** (§D1) |
| B1 | What happens to a bundle once its slice completes? | Folds file-to-file into the scope's main bundle, then its directory is deleted; git history is the archive | **DECIDED** (§B1) |
| P1 | Does lab-os own Claude Code plugins, and how do they distribute? | Yes — vendored under `.claude/skills/<name>/` alongside skills, loaded as `<name>@skills-dir`; no marketplace, no install step | **DECIDED** (§P1) |
| T1 | Does lab-os adopt session timeboxing? | Yes — timeboxing v1.0: a session standard plus an agent-task-box extension with per-box calibration rows | **DECIDED** (§T1) |
| K1 | Where does cross-repo / lab-level open work track? | A shared `BACKLOG.md` at the lab-os root, groomed at each sprint boundary; repo-scoped work stays in that repo's issues | **DECIDED** (§K1) |
| K2 | Does a CI check enforce `BACKLOG.md` item hygiene? | Yes — `backlog-lint`, warn-only until first green, then enforcing | **DECIDED** (§K2) |
| K3 | Does backlog-lint fail closed on structural defects? | Yes — unattributed Items-section text is a hard error, and `--write-index` refuses to regenerate over parse errors instead of silently dropping unparsed rows | **DECIDED** (§K3) |
| K4 | What did the #68 review round change about the backlog renderers? | Both renderers refuse to render (fail closed) any backlog with parse errors, zero Item blocks, or Index/Item id drift, and `backlog-views` enforcement flips on | **DECIDED** (§K4) |
| S1 | Does lab-os re-sync its rules from the workspace fork, and does the agent-runtime HARD RULE stand? | Yes to both, at the fork's current HEAD rather than a pinned snapshot | **DECIDED** (§S1) |
| S2 | Does the specialist review panel port to lab-os? | Yes — the four vendored specialist review agents and their dispatch contract; the code-quality taxonomy they cite is not yet carried | **DECIDED** (§S2) |
| S3 | Does lab-os own the lab's shared Claude Code skills, and how do they deploy? | Yes — in-repo under `.claude/skills/`, deployed user-scope via symlinks | **DECIDED** (§S3) |
| S4 | Where does `spec-plan-analyzer` originate, and how does it resolve the standards it checks? | lab-os — the first agent body lab-os authors rather than vendors; it derives its checks by resolving the repo under review's own rules at read time, never restating a checklist | **DECIDED** (§S4) |
| W1 | Is fork-of-lab-os the default Claude-powered dev home? | Yes — replacing the clone-as-rules-subdir-plus-junction model; the junction stays as the documented multi-repo power-user path | **DECIDED** (§W1) |
| W2 | Where do plans track, versus project code? | Plans track at the fork level; only project code re-homes as a separate gitignored nested repo | **DECIDED** (§W2) |
| W3 | Does a sample plan ship for the Building workshop part? | Yes — a facilitator-only fallback for a participant who arrives without a plan of their own; not published to the site | **DECIDED** (§W3) |
| W4 | Does the Workshop Program supersede onboarding-project and one-day Building? | Yes | **DECIDED** (§W4) |
| W5 | Does handbook content rework gate tester launch? | Yes — tester launch waits on a full seven-page content and IA rework | **DECIDED** (§W5) |
| W6 | When does a plan's execution log close? | With the PR that ships it; post-merge evidence goes to a comment on that PR, never a trailing log entry | **DECIDED** (§W6) |
| W7 | Who owns human-facing docs? | The handbook site; root pointer stubs redirect into it, `.claude/rules/` stay AI-tier repo files | **DECIDED** (§W7) |
| F1 | Is the combined logging/docs rule split? | Yes — into `03-logging.md` and `04-docs.md`, both under the rules-numbering convention | **DECIDED** (§F1) |
| F2 | Is the lab-wide logging & documentation standard adopted? | Yes — the standard this bundle later removes; adopted here to fix retrieval, staleness and format drift across per-repo logs | **DECIDED** (§F2) |

---

## Decisions

## Register & main bundle

### §R1: Where does lab-os's main bundle live

`_specs/lab-os/main/` — the workspace-root path shape — rather than the member-repo
`_specs/main/` shape.

**Why.** One path convention per repo, matching the sibling `2026-07-31-timeboxing` bundle.
Reopened only by a ruling or a `03-logging.md`-successor change that settles lab-os's altitude.

**Landed:** PR #83

## Licensing

### §L1: Is lab-os licensed, and how

Add a root `LICENSE` (unmodified Apache-2.0) and a root `NOTICE` scoping the grant to
lab-authored content, with vendored third-party assets (Anthropic `pr-review-toolkit`,
Apache-2.0; Cursor rubric content, MIT) retaining their upstream licenses. `README.md` gains a
License section pointing at both.

**Why.** The repo was public and unlicensed while `04-docs.md` §Rules numbering designated it
the upstream owner of the canonical `0x` bytes member repos copy verbatim — nobody held a right
to copy them. Apache-2.0 over MIT for the express patent grant and the attribution condition,
which turns the vendored-file sync header into a license term.

**Alternatives rejected.** Proprietary all-rights-reserved, as every other active lab repo takes
under the same sweep — rejected here, since it would forbid copying the one repo whose job is to
be copied. Editing `LICENSE` itself to state the carve-out — rejected; the carve-out belongs in
`NOTICE`, and `LICENSE` stays verbatim.

**Landed:** PR #115

### §L2: Whose contributions does the Apache-2.0 grant cover, and from when

The `NOTICE` grant covers Watson Blair's contributions in full, before and after 2026-09-01;
contributions by Kiara Jamal, Arya Kurup and Jean Moncayo made before that date are excluded
pending recorded consent (Caravan#311). Everything from 2026-09-01 onward is Apache-2.0 for all
authors; opening a PR is an offer under those terms.

**Why.** A repo owner cannot license away a contributor's copyright unilaterally, and the repo was
previously unlicensed, so there is no prior grant to fall back on. Bounding by author rather than
by a flat effective date grants what is already the owner's — the great majority of the repo —
instead of holding the whole grant hostage to three acks.

**Alternatives rejected.** Block on all three acks — rejected, leaves the repo unlicensed
meanwhile. A flat going-forward date — rejected, that would exclude the owner's own existing bytes
for nothing.

**Contract impact:** `.claude/rules/06-timeboxing.md` (authored entirely by Arya Kurup) and
`03-logging.md`'s successor rule (carrying one Kiara Jamal commit) are unlicensed for repos
vendoring the `0x` bytes until Caravan#311 closes; the `NOTICE` section is removed when it does.

**Landed:** PR #115

## Documentation budgets

### §D1: What are lab-os's always-loaded doc byte budgets

`CLAUDE.md` 8→12 KB, each `.claude/rules/*.md` 5→8 KB, and a new 48 KB **aggregate** cap over the
always-loaded tier (the lab log excluded, unchanged at 15 KB). Over aggregate the remedy is
demoting a surface to grep-only, not raising the cap. The always-loaded set is flat and
case-insensitive: `.claude/rules/` direct children, per the rule's `*.md` glob.

**Why.** The 8/5 KB numbers were never calibrated — the design doc logged them as a first guess
and recorded every flagship repo already over 8 KB at adoption. A per-file raise alone removes the
pressure that forces the always-loaded/grep-only tiering decision, so the aggregate is the binding
number.

**Alternatives rejected.** Flat 20 KB per file — rejected, adds ceiling on a glob that only grows.
Per-file raise with no aggregate — rejected, leaves the add-a-rules-file hole open.

**Contract impact:** `.claude/rules/04-docs.md` owns the bytes; `scripts/docs_budget.py` enforces.

**Landed:** PR #79

## Bundle lifecycle

### §B1: What happens to a bundle once its slice completes

Each scope keeps one main bundle — `_specs/<scope>/main/{prd,spec,design,plan,log}.md`, never
dated, never deleted, budget-exempt — the single source of truth for what is implemented. A bundle
reaching a terminal `Status:` folds file-to-file into it and its directory is deleted; git history
is the archival record. Reverses an earlier retain-in-place clause.

**Why.** Retain-in-place cannot produce a single current-state authority: an index over N dated
bundles still makes every reader reconstruct current state. Deletion is what forces the fold to
happen; git history already archives the rest.

**Alternatives rejected.** Retain-in-place with the main bundle as an index over retained bundles —
rejected, two authoritative surfaces whose divergence nobody owns. Defer — rejected, lab-os holds
one bundle and nothing terminal, so the change is free now and only gets costlier.

**Landed:** PR #81

## Claude Code plugins

### §P1: Does lab-os own Claude Code plugins, and how do they distribute

Yes. lab-os owns the lab's Claude Code **plugins** alongside its skills, and distributes them the
same way: vendored under `.claude/skills/<name>/` carrying a `.claude-plugin/plugin.json`, which
Claude Code loads as `<name>@skills-dir` — no marketplace, no install step. `context-gc` (recovers
session state after auto-compaction) is the first.

**Why.** One distribution mechanism, not two. §S3 already rejected marketplace distribution for
skills, and a marketplace for plugins alone would re-introduce the install step that decision
removed. Vendoring widens the security boundary from assets executing as *instructions* to assets
executing as *code*, so `plugin-tests` gates the suite in CI.

**Alternatives rejected.** An org marketplace at `.claude-plugin/marketplace.json` — built and
reviewed, then dropped as a second mechanism. Per-member-repo vendoring — rejected, multiplies
copies the sync rules exist to prevent.

**Landed:** PR #78

## Timeboxing

### §T1: Does lab-os adopt session timeboxing

Yes — the session timeboxing standard v1.0 (default boxes per session type, written exit
criterion, scope-hammer before extension, one extension max) together with its agent-task
extension: agent dev tasks run inside a stated box, expiry notifies the user and hands off, and one
planned-vs-actual calibration row is appended per box. Rules surface: `.claude/rules/06-timeboxing.md`.

**Why.** Manual discipline never closed the calibration loop in the originating fork; dev work
happens in the agent phase of brainstorm → agent-dev → hard-pass review, so that phase is where
boxing prevents drift.

**Alternatives rejected.** Marketplace-plugin homing — rejected, lab conventions live in lab-os.
Central-only row logging — rejected, rows live beside the work; roll-up is a grep away.

**Landed:** PR #66

## Backlog & CI lint

### §K1: Where does cross-repo / lab-level open work track

A shared `BACKLOG.md` at the lab-os root (index → inbox → items per
`templates/backlog-item.template.md`); repo-scoped work stays in that repo's issues. Retires the
maintainer-personal backlog as the team mechanism. Groomed at each 2-week sprint boundary.

**Why.** Single-owner routing made open lab work invisible and ungroomable by the team; the format
was already taught in the building-workshop answer key, so adoption cost is a promotion, not an
invention.

**Alternatives rejected.** GitHub Projects org board — rejected, new tooling surface, content
leaves the repo. Issues-only — rejected, cross-repo work homeless.

**Landed:** PR #55

### §K2: Does a CI check enforce `BACKLOG.md` item hygiene

Yes — a `backlog_lint` CI check (sibling to the docs-budget lint), validating `BACKLOG.md`
unconditionally on every PR: required fields, a single-condition non-placeholder `Done when`, the
status ladder, size, the Index as a generated projection of the Item blocks (committed table must
byte-match the render), and `Depends on` referential integrity and acyclicity. Warn-only until
first green, then enforcing.

**Why.** The backlog's readiness bar was enforced only by grooming discipline; a CI check makes it
true by construction, like the other lints. A derived Index kills the dual-representation drift
class instead of policing it.

**Alternatives rejected.** Fold under §K1's proposal→ratify mechanism — rejected, conflates the
tool with the mechanism it rides. A `.claude/rules/` entry — rejected, a lint's behavior is not a
hard rule. Hand-authored Index plus reconciliation check — rejected, leaves two authoritative
copies.

**Landed:** PR #67

### §K3: Does backlog-lint fail closed on structural defects

Yes. Items-section text the parser cannot attribute to an item is now a hard error and stays
leak-scanned, and `--write-index` refuses to regenerate while structural parse errors exist instead
of deleting the unparsed blocks' Index rows under a success message. The same posture extends
mechanically across several other structural checks.

**Why.** Both prior defects silently inverted the module's own contract — the leak tripwire was
disabled exactly where the file was malformed, and the documented repair command destroyed
committed rows while reporting success. A derived projection is only safe to regenerate from a
source that fully parsed; ambiguous input fails closed.

**Alternatives rejected.** Fix once on a later PR and close this one — rejected, the review that
found the defects binds here. Warn on unattached text — rejected, unowned text is exactly what
escapes every field-level check.

**Landed:** PR #67

### §K4: What did the #68 review round change about the backlog renderers

Both renderers refuse to render (fail closed, never the staleness exit) any backlog with parse
errors, zero Item blocks despite non-empty text, or Index/Item id drift. Titles are escaped in
rendered output, the digest CLI errors cleanly on a missing backlog, and `standards.yml` flips
`backlog-views` to enforcing.

**Why.** The review's fail-open findings all reduced to "a broken source renders as a plausible
artifact on a green job"; guarding every entry point closes the class, not the instances.

**Landed:** PR #68

## Rules parity, specialist panel & skills

### §S1: Does lab-os re-sync its rules from the workspace fork, and does the agent-runtime HARD RULE stand

Yes to both, dropping the earlier snapshot pin: `.claude/rules/03-logging.md`'s successor and
`04-docs.md` are re-copied verbatim from the workspace fork's current HEAD, landing the ratified
four-file bundle contract (`prd`/`spec`/`plan`/`log`, `spec.md` as the bundle's design authority
with a decision summary table). `05-agent-runtime.md` continues to stand as a HARD RULE binding any
asset hosting a guardrailed local coding-agent runtime, the three log altitudes stand, and upstream
lab-os continues to own the canonical bytes.

**Why.** The pin was defended as diff hygiene, but it shipped a rules set member repos would
immediately have to re-vendor, and left `04-docs.md` asserting a four-file bundle while
`03-logging.md`'s predecessor still described three. Parity between the two authoring surfaces is
the point; a half-synced pair is worse than either end.

**Alternatives rejected.** Defer the newer text to its own slice — rejected, publishes a
self-contradicting pair and doubles downstream vendoring.

**Supersedes:** the earlier pinned-snapshot decision that first added `05-agent-runtime.md` and
re-architected logging onto three altitudes.

**Landed:** PR #58

### §S2: Does the specialist review panel port to lab-os

Yes — the four vendored specialist review agents and their dispatch contract
(`reference/specialist-dispatch.md`) are carried here, so a bare clone resolves the panel the
review skills dispatch. `reference/code-quality-taxonomy.md` is not carried by this slice; ownership
is unaffected — upstream lab-os owns its convention and canonical bytes like every other manifest
asset. The agent bodies in `.claude/agents/**` are byte-owned here; the fork inherits them via
`git pull upstream main`, never by back-port.

**Why.** The agents were unreachable from a lab-os-rooted dev home. The taxonomy's absence is a
carry gap, not an ownership split — its sync header already names lab-os as owner.

**Alternatives rejected.** Carry the taxonomy too — rejected on scope, not merits; it belongs with
the rules-parity sync that lands the rules it cites. Omit the agents entirely — rejected, runtime
`DEV_ROOT` resolution already makes them optional.

**Landed:** PR #61

### §S3: Does lab-os own the lab's shared Claude Code skills, and how do they deploy

Yes. lab-os is the source of truth for the lab's shared Claude Code skills and commands; they live
in-repo (`.claude/skills/`, `.claude/commands/`, `.claude/scripts/`) and deploy user-scope via
`link-lab-assets.sh`, which symlinks them into `~/.claude/`.

**Why.** A clone or fork is self-contained with no marketplace install, and `git pull upstream`
keeps skills current. Because deployment is user-scope, merged skill content later executes as
*instructions* under every member's identity, in every session — so `.claude/skills/**` review
rigor is a security boundary, and a skills PR is read as code, not docs.

**Alternatives rejected.** Marketplace-plugin distribution — rejected, reintroduces the install
step and a `plugin.json` surface the in-repo model avoids. Per-repo vendoring — rejected for shared
skills, multiplies drift with no owning source.

**Landed:** PR #59

### §S4: Where does `spec-plan-analyzer` originate, and how does it resolve the standards it checks

lab-os — the first agent body lab-os originates rather than vendors; the fork inherits it by `pull
upstream`. It restates no checklist: it resolves `04-docs.md` §ENG and `03-logging.md`'s successor
in the repo under review and derives its checks from that, returning a named not-run dimension
where neither resolves. Dispatch is path-based via a fail-closed per-repo ENG path registry.

**Why.** Bundle shape is repo- and version-specific — lab-os defines PRD/design/plan under
`docs/work/`, the fork a four-file `_specs/` bundle. A restated checklist would violate
`04-docs.md` §Single source and go stale at the next rules sync; read-time derivation is correct on
both.

**Alternatives rejected.** Restate the fork's bundle shape — rejected, would review lab-os bundles
against criteria lab-os does not hold. Hold for a later phase behind a dry-run gate — rejected on
request; that phasing de-risked lifted bodies, and this one is authored.

**Landed:** PR #61

## Workshop program & dev home

### §W1: Is fork-of-lab-os the default Claude-powered dev home

Yes. Onboarding forks lab-os (clone fallback) and uses that fork as the participant's primary dev
home, replacing the clone-as-rules-subdir plus junction model. Rules live natively in the fork;
`git pull upstream main` keeps them current. Bring-your-own-project is preserved: the plan/project
is re-homed as its own gitignored repo nested in the fork. The junction model is retained as the
documented multi-repo power-user path.

**Why.** A fork gives a personal, push-able copy and drops the most failure-prone bootstrap step.
Nesting the project as a separate gitignored repo recovers the junction's one benefit — lab tooling
kept separate from project work — without it.

**Alternatives rejected.** Junction/multi-repo dev-root — retained as the power-user path, not the
default. Commit the project into the fork (monorepo) — rejected, couples upstream pulls with
project history.

**Landed:** PR #43

### §W2: Where do plans track, versus project code

Plans track at the fork level, not a nested repo — the plan and backlog at `_plans/`. Only the
project *code* is re-homed as a separate gitignored nested repo. Refines §W1, which had homed "the plan/project" together in the nested repo, conflating two
artifacts with different needs.

**Why.** The fork is the methodology/coordination home, and plans are methodology — matching how
the lab already works. §W1's anti-coupling reason (don't couple `git pull upstream` with project
history) bites for a full codebase, not a handful of plan files in a path upstream never touches.

**Landed:** PR #44

### §W3: Does a sample plan ship for the Building workshop part

Yes — a pre-baked three-task plan (re-home fork identity, brand the handbook, add a
backlog/planning surface) that a participant who reaches Building without a plan of their own runs
against their own fork. Not published to the site and not in the sidebar; participants point Claude
at the file from their fork's CLI.

**Why.** The mixed-cohort Building kickoff has newcomers who finish setup with no execution-ready
plan and so can't practise the three execution modes; a small real plan unblocks them. Kept
non-published to hold the program's "no prescribed sample project" line on the public surface.

**Alternatives rejected.** Publish as a participant page — rejected, contradicts the
no-prescribed-sample stance. Demo-only build by the facilitator — rejected, leaves newcomers
watching instead of practising.

**Landed:** PR #42

### §W4: Does the Workshop Program supersede onboarding-project and one-day Building

Yes. The three-part Workshop Program (Planning → Building → Closeout) supersedes the two-week
onboarding-project sandbox (now a redirect stub into the program) and the standalone one-day
Building-with-Claude material (absorbed into the Building part's exercises). Facilitator runbooks
are internal, not published.

**Why.** One coherent bring-your-own-project arc on a single self-paced-plus-live-facilitated
surface, instead of a scattered sandbox plus a one-day track that diverge and double the
maintenance.

**Landed:** PR #39

### §W5: Does handbook content rework gate tester launch

Yes — the handbook content and IA is reworked across all seven pages before testers are invited;
tester launch waits on the rework, superseding a prior deferral that left page-content
restructuring to play-test friction data. The rework is decomposed: a backbone
authoring-conventions round first, then per-page rounds.

**Why.** The deferral assumed existing content was good enough for a first cohort and that friction
data should drive structural change. On review, the shipped site needed reframing, zero-tech
support, and terminal-vs-Claude command clarity before a tester runs the arc — gaps a cohort hits
immediately, not subtle friction worth waiting for.

**Alternatives rejected.** Launch on current content, rework in parallel — rejected, testers would
hit content already judged inadequate. Partial gate — considered, rejected for a clean gated
rework.

**Landed:** PR #25

### §W6: When does a plan's execution log close

With the PR that ships it. Post-merge evidence (deploy green, runtime verification, branch cleanup)
goes to a comment on that PR; bigger facts route per the entry-triggers table. No trailing entries
held for a future PR.

**Why.** The execution log lives in the repo, so merge-time facts always arrive after the last
commit that could carry them. Prior closeouts hitched entries onto whatever PR came next, coupling
unrelated PRs and dangling when no next PR exists.

**Alternatives rejected.** Dedicated one-line closeout PRs — rejected, noise against the
single-concern merge bar. Predictive pre-merge entries — rejected, evidence written before it
exists.

**Landed:** PR #18

### §W7: Who owns human-facing docs

The handbook site (`site/`, deployed to watsonwblair.github.io/lab-os). Root `BOOTSTRAP.md` and
`WORKING-WITH-CLAUDE.md` become pointer stubs to their site pages; `.claude/rules/` stay AI-tier
repo files.

**Why.** One human-facing surface, written for its readers, with build-time link checking —
instead of agent-dense repo markdown doing double duty for stakeholders.

**Alternatives rejected.** Site wraps the repo markdown unchanged — rejected, a brochure over a
codebase. Site renders the repo markdown as-is — rejected, agent-dense prose shown to stakeholders.

**Landed:** PR #15

## Foundational logging & docs standard

### §F1: Is the combined logging/docs rule split

Yes. The combined `03-logging-and-docs.md` rule is split into `03-logging.md` (altitudes, entry
triggers/routing, entry format, immutability, file structure and overflow) and `04-docs.md`
(single-source, tiers and byte budgets, ENG document standards, rules numbering). Numbered names
retained per the rules-numbering convention.

**Why.** The combined file sat at its own byte budget with `docs-budget` enforcement now live — any
rule edit first required an offsetting trim. Two single-responsibility files restore headroom on
both halves.

**Alternatives rejected.** Unnumbered `logging.md`/`docs.md` — rejected, contradicts the
rules-numbering convention shipped the same day. Compression-only — rejected, the margin stays
structurally tight as rules accrete.

**Landed:** PR #9

### §F2: Is the lab-wide logging & documentation standard adopted

Yes, at the time — the `03-logging-and-docs.md` rule, the PR-lifecycle doc, normative templates
(project log, PRD, `CLAUDE.md` tiers, work bundle), and CI adherence actions (`log-lint`,
`docs-budget`, `merge-bar-check`). Project logs converged on one shape — Standing Decisions index +
reverse-chron hot window + grep-only archive — with immutable merged entries and pre-merge log
cleanup on every PR.

**Why.** Per-repo log formats had diverged to the point that the largest logs could no longer be
read whole by an agent, and stale `Status:` markers forced follow-up edits to merged entries. The
standard fixed retrieval (index-first, then hot window, then grep) and removed anything in a merged
entry that could go stale.

**Landed:** PR #6
