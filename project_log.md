# lab-os — project log

Format: lab standard, `lab-os/.claude/rules/03-logging.md`. Skeleton per
`lab-os/templates/docs/project_log.template.md` (normative — `log-lint` parses this structure).
The `## Standing Decisions` and `## Entries` headings are load-bearing lint anchors: exact
text, one each, never renamed. Entry headers are the only other `##` headings allowed.

## Standing Decisions

- 2026-09-01 20:05 — Apache-2.0 is bounded by author; pre-2026-09-01 co-author work awaits consent · #115
- 2026-09-01 19:48 — lab-os is Apache-2.0; vendored third-party work keeps its own license (NOTICE) · #115
- 2026-08-12 17:33 — Raise always-loaded doc budgets to 12/8 KB; add a 48 KB aggregate cap · #79
- 2026-08-13 10:55 — Terminal bundles fold into the scope's main bundle, then delete · #81
- 2026-08-13 16:39 — lab-os owns Claude Code plugins; they vendor under `.claude/skills/` · #78
- 2026-08-07 13:08 — Adopt timeboxing v1.0: session standard + agent task boxes · #66
- 2026-08-06 15:10 — PR #68 remediation: renderers fail closed, backlog-views enforced · #68
- 2026-08-06 14:00 — Backlog-lint fails closed on structural defects · #67
- 2026-08-06 12:32 — Lab-wide backlog: cross-repo open work routes to BACKLOG.md · #55
- 2026-07-31 14:03 — spec-plan-analyzer originates in lab-os and derives standards at read time · #61
- 2026-07-31 13:22 — Adopt the workspace fork's rules at current HEAD; agent-runtime HARD RULE stands · #58
- 2026-07-24 12:40 — Specialist panel ports to lab-os; taxonomy staged in the fork, not yet carried · #61 → project_log_archive.md
- 2026-07-24 16:20 — lab-os owns shared Claude skills; deploy is user-scope symlinks · #59 → project_log_archive.md
- 2026-07-23 11:13 — Backlog-lint enforces BACKLOG.md item hygiene via CI · #67
- 2026-06-23 07:51 — Plans track at the fork level; only project code nests · #44 → project_log_archive.md
- 2026-06-23 06:30 — Fork-of-lab-os is the default Claude-powered dev home · #43 → project_log_archive.md
- 2026-06-23 03:05 — Building sample plan ships as a facilitator-only fallback · #42 → project_log_archive.md
- 2026-06-19 05:58 — Workshop Program supersedes onboarding-project and one-day Building · #39 → project_log_archive.md
- 2026-06-13 15:00 — Handbook content rework gates tester launch · #25 → project_log_archive.md
- 2026-06-12 12:00 — Plan-execution logs close with their shipping PR · #18 → project_log_archive.md
- 2026-06-11 19:45 — Site owns human-facing docs · #15 → project_log_archive.md
- 2026-06-10 21:54 — Split the combined rule into 03-logging.md and 04-docs.md · #9 → project_log_archive.md
- 2026-06-10 17:45 — Adopt lab-wide logging & documentation standard · #6 → project_log_archive.md

## Entries

---

## 2026-09-01 20:05 — Apache-2.0 bounded by author, not by a flat effective date

**Decision:** The `NOTICE` grant covers Watson Blair's contributions in full, before and after
2026-09-01; contributions by Kiara Jamal, Arya Kurup and Jean Moncayo made before that date are
excluded pending recorded consent (Caravan#311). Everything from 2026-09-01 onward is
Apache-2.0 for all authors; opening a PR is an offer under those terms.
**Why:** A repo owner cannot license away a contributor's copyright unilaterally, and this repo
was previously unlicensed, so there is no prior grant to fall back on. Bounding by author rather
than by date alone grants what is the owner's — the great majority of the repo, five of the six
`0x` rule files — instead of holding the whole grant hostage to three acks.
**Consequence:** `.claude/rules/06-timeboxing.md` is authored entirely by Arya Kurup and
`03-logging.md` carries one Kiara Jamal commit, so repos vendoring the `0x` bytes hold no grant
over those contributions until Caravan#311 closes. The `NOTICE` section is removed when it does.
**Alternatives:** Block on all three acks (rejected — leaves the repo unlicensed meanwhile). A
flat going-forward date as in Caravan#275 (rejected — that PR tightened a grant, where a date
bound limits the assertion; here it would exclude the owner's own existing bytes for nothing).
**Refs:** #115, CAMELS-Research-Group/Caravan#311

---

## 2026-09-01 19:48 — lab-os licensed Apache-2.0; vendored work carved out in NOTICE

**Decision:** Add a root `LICENSE` (unmodified Apache-2.0) and a root `NOTICE` scoping the grant
to lab-authored content, with vendored third-party assets (Anthropic `pr-review-toolkit`,
Apache-2.0; Cursor rubric content, MIT) retaining their upstream licenses. `README.md` gains a
License section pointing at both.
**Why:** This repo was public and unlicensed while `04-docs.md` §Rules numbering designates it
the upstream owner of the canonical `0x` bytes member repos copy verbatim — nobody held a right
to copy them. Apache-2.0 over MIT for the express patent grant (§3) and the attribution
condition (§4), which turns the vendored-file sync header into a license term. The `NOTICE` is
the §4(d) slot: without it the root grant reads as covering bytes the two `ATTRIBUTION.md` files
place under other licenses.
**Alternatives:** Proprietary All-Rights-Reserved, as every other active lab repo takes under
the same sweep — rejected here, it would forbid copying the one repo whose job is to be copied.
Editing `LICENSE` itself to state the carve-out — rejected; the text stays verbatim and the
carve-out belongs in `NOTICE`.
**Irrevocable:** an Apache-2.0 grant, once published, cannot be withdrawn for anyone who
obtained the work under it.
**Refs:** #115, CAMELS-Research-Group/Caravan#274
## 2026-08-12 17:33 — Raise always-loaded doc budgets to 12/8 KB; add a 48 KB aggregate cap

**Decision:** `CLAUDE.md` 8→12 KB, each `.claude/rules/*.md` 5→8 KB, and a new 48 KB
**aggregate** cap over the always-loaded tier (`project_log.md` excluded, unchanged at 15 KB).
Over aggregate the remedy is demoting a surface to grep-only, not raising the cap. A scan that
cannot measure a present always-loaded surface reports PARTIAL and fails closed under
`--enforce`. The always-loaded set is **flat**, case-insensitive:
`.claude/rules/` direct children, per the rule's `*.md` glob; a subdirectory is
unmeasurable, not walked. `.claude/rules/04-docs.md` owns the bytes;
`scripts/docs_budget.py` enforces.
**Why:** The 8/5 KB numbers were never calibrated — the design doc logged them as a first guess
(§13) and recorded every flagship repo already over 8 KB at adoption (§7.2). A per-file raise
alone removes the pressure that forces the always-loaded/grep-only tiering decision, so the
aggregate — what context costs per session and per subagent — is the binding number, and it
only binds if a surface it could not measure cannot silently shrink it.
**Alternatives:** Flat 20 KB per file (rejected: +75 KB of ceiling on a glob that only grows).
Per-file raise with no aggregate (rejected: leaves the add-a-rules-file hole open). Partial
aggregate reported but green (rejected: a short total still reads authoritative).
**Refs:** #79, `.claude/rules/04-docs.md`, `scripts/docs_budget.py`

---

## 2026-08-13 10:55 — Terminal bundles fold into the scope's main bundle, then delete

**Decision:** Each scope keeps one main bundle — `_specs/<scope>/main/{prd,spec,design,plan,log}.md`,
never dated, never deleted, budget-exempt — the single source of truth for what is implemented. A
bundle reaching a terminal `Status:` folds file-to-file into it and its directory is deleted; git
history is the archival record. `design.md` joins the bundle file contract in the same change: the
main bundle is a slice's file set and names it. Reverses the retain-in-place clause, which arrived
as vendored text under 2026-07-31 13:22 (#58) and was never ratified here (origin:
WatsonWBlair/Agentic_Workspace#95).
**Why:** Retain-in-place cannot produce a decision register: an index over N dated bundles still
makes every reader reconstruct current state — the gap that left lab-os without one. Deletion is
what forces the fold to happen; git history already archives the rest.
**Alternatives:** Retain-in-place with the main bundle as an index over retained bundles (rejected —
two authoritative surfaces whose divergence nobody owns). Defer (rejected — lab-os holds one bundle
and nothing terminal, so the change is free now and only gets costlier).
**Refs:** #81, CAMELS-Research-Group/Caravan#9, CAMELS-Research-Group/Caravan#14

---

## 2026-08-13 16:39 — lab-os owns Claude Code plugins; they vendor under `.claude/skills/`

**Decision:** lab-os owns the lab's Claude Code **plugins** alongside its skills, and distributes them
the same way: vendored under `.claude/skills/<name>/` carrying a `.claude-plugin/plugin.json`, which
Claude Code loads as `<name>@skills-dir` — no marketplace, no install step. `context-gc` (recovers
session state after auto-compaction, via a `SessionStart(compact)` hook) is the first.
**Why:** one distribution mechanism, not two. 2026-07-24 16:20 already rejected marketplace
distribution for skills, and `ATTRIBUTION.md` states a clone or fork is self-contained; a marketplace
for plugins alone re-introduces the install step that decision removed. Vendoring widens the stated
security boundary from assets executing as *instructions* to assets executing as *code*, so
`plugin-tests` gates the suite in CI.
**Alternatives:** an org marketplace at `.claude-plugin/marketplace.json` — built and reviewed on this
PR, then dropped as the second mechanism; per-member-repo vendoring (N copies the sync rules exist to
prevent — skills-dir needs one, `~/.claude/skills/` being machine-level).
**Refs:** #78, `.claude/skills/context-gc/`, `.claude/scripts/link-lab-assets.sh`

---

## 2026-08-07 13:08 — Adopt timeboxing v1.0: session standard + agent task boxes

**Decision:** lab-os adopts the session timeboxing standard v1.0 (`docs/timeboxing.md`: default
boxes per session type, written exit criterion, scope-hammer before extension, one extension max)
together with its agent-task extension (`docs/timebox_recording.md` + `.claude/skills/timeboxing/`):
agent dev tasks run inside a stated box, expiry notifies the user and hands off, and one
planned-vs-actual calibration row is appended per box — to the calibration file of the repo the task
belongs to where that repo has adopted the practice, else to `docs/timebox_calibration.md` here.
Rules surface: `.claude/rules/06-timeboxing.md`.
**Why:** Manual discipline never closed the calibration loop in the originating fork (zero rows
2026-07-24 → 07-31); dev work happens in the agent phase of brainstorm → agent-dev → hard-pass
review, so that phase is where boxing prevents drift.
**Alternatives:** Marketplace-plugin homing (portable; re-scoped — lab conventions live in lab-os).
Central-only row logging (rejected: rows live beside the work; roll-up is a grep away).
**Refs:** #66; fork provenance: Aryaa-K/lab-os#1 (standard, adopted there 2026-07-24),
Aryaa-K/lab-os#3 (agent extension, 2026-07-31)

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

## 2026-07-31 13:22 — Adopt the workspace fork's rules at current HEAD; agent-runtime HARD RULE stands

**Decision:** Drops the snapshot pin. `.claude/rules/03-logging.md` and `04-docs.md` are re-copied
verbatim from the workspace fork's HEAD, landing #54's ratified bundle contract here: the planning
bundle is four files (`prd`/`spec`/`plan`/`log`), `spec.md` is the bundle's design authority with a
decision summary table and the `DECIDED / RECOMMENDED / PARKED` legend, chore/docs-only bundles omit
it, three-file bundles are grandfathered. The spec-log's exemption from the per-entry byte cap is now
explicit. The rest of the superseded entry stands: `05-agent-runtime.md` as a HARD RULE, the three
log altitudes, upstream lab-os owning the canonical bytes.
**Why:** The pin was defended as diff hygiene, but it would ship a rules set member repos must
immediately re-vendor, and it left `04-docs.md` asserting a four-file bundle while `03-logging.md`
still described three. Parity with the authoring surface is this PR's purpose; a half-synced pair is
worse than either end.
**Alternatives:** Defer the newer text to its own slice (the superseded call) — rejected: publishes a
self-contradicting pair and doubles downstream vendoring.
**Supersedes:** 2026-07-28 22:26 — Adopt the workspace fork's rules as lab-os standards; add agent-runtime HARD RULE
**Refs:** #58, .claude/rules/03-logging.md, .claude/rules/04-docs.md

---

## 2026-07-28 22:26 — Adopt the workspace fork's rules as lab-os standards; add agent-runtime HARD RULE

**Decision:** lab-os re-syncs `.claude/rules/` from the workspace fork, pinned to the 2026-07-24
snapshot, and gains a fifth rule: `05-agent-runtime.md`, a HARD RULE binding any asset hosting a
guardrailed local coding-agent runtime — max-clean `claude -p`, per-run caps, a named halt
vocabulary, fail-closed per-repo permission and commit-destination policy, a deny-by-default
approval gate, bot-identity-only posting, disabled-by-default. Logging is re-architected onto three
altitudes (lab / project / spec-log) with the planning bundle as the spec-log's home. Upstream
lab-os owns the convention and canonical bytes; the fork is where edits are staged.
**Why:** The fork has been the de facto authoring surface while lab-os shipped the published
standard, so they drifted. Pulling that work back as one reviewed slice restores a single source
before member repos vendor from it. The runtime contract needs rule-tier permanence because it
constrains spend and identity, not style.
**Alternatives:** Keep the runtime contract in the hosting repo's docs — rejected: it binds every
asset hosting a runtime, not one repo. Also re-copy the fork's newer `03-logging.md`/`04-docs.md` —
rejected: those landed after this PR was cut; pinning to the reviewed snapshot keeps the diff
auditable and defers them to their own slice.
**Refs:** #58, .claude/rules/05-agent-runtime.md

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
