# Logging & Documentation Standard — design

**Date:** 2026-06-10 · **Status:** revised after two-agent outsider review (same date) · **Repo:** lab-rules

## 1. Problem

The lab's working cadence produced four divergent de-facto project-log standards across repos
(Global_Pathways 174 KB top-insert session digests; mission-control 135 KB structured decision
entries; LSCA bottom-append retrospectives; lab-rules/dev-root decision-centric). The two largest
logs can no longer be read whole by an agent, there is no stated contract for how much of a log an
agent should read, log entries require follow-up edits after merge (stale `Status:` markers), and
nothing in lab-rules says how documentation should be segmented by audience. As the lab embraces
Spec-Driven Development, context fatigue — for agents, lab members, and stakeholders — is the
failure mode this standard exists to prevent.

Lab-rules today codifies only three sentences on note-taking (`WORKING-WITH-CLAUDE.md` §8). The
automated PR process is split across four places: `pr-review-agent/SPEC.md` (the cron reviewer),
the `pr-review-loop` plugin docs (the remediation loop), `WORKING-WITH-CLAUDE.md` §5 (review
discipline), and the solo-maintainer admin-bypass pattern, which is practiced but unwritten.

## 2. Goals and non-goals

**Goals**

- One lab-wide project-log standard optimized for AI retrieval: recent-window + search, with
  compression and curation keeping logs lean.
- Structural elimination of follow-up edits to merged log entries.
- Pre-merge log cleanup as a defined step on every PR, enforced at review time.
- A rational bundling + archival shape for planning/logging artifacts (PRDs, slice specs/plans,
  small tasks).
- Audience-segmented documentation: tier definitions, routing, per-tier writing standards,
  context budgets, single-source-of-truth rule.
- One canonical end-to-end PR lifecycle document, including the merge bar and the
  solo-maintainer bypass.
- GitHub Actions that mechanically enforce what can be mechanically enforced.

**Non-goals**

- Migrating existing logs (Global_Pathways, mission-control) — phase 2, separate effort.
  New work follows the new standard; old artifacts stay put until migrated deliberately.
- Replacing the pr-review agent's semantic review with Actions. Mechanical vs semantic is a
  deliberate split (§9).
- Per-feature or per-PR log files. Three altitudes only (§4).

## 3. Decisions reached (interview round + review round, 2026-06-10)

| # | Decision | Why |
|---|----------|-----|
| D1 | Retrieval contract: **recent window + search**, plus active compression/curation | Whole-log reads are already impossible at current sizes; curated-head-only was judged too high-maintenance |
| D2 | Entry unit: **one load-bearing decision** (ADR-style: context/decision/consequences, adapted) | Maximally greppable/compressible; routine work generates no entry; session narrative belongs in PRs |
| D3 | Additional entry triggers: **irreversible/external events, deviations from plan, direction changes** | "When and why did this happen" matters later and git history alone won't say. Deviations route to the plan-execution altitude, not the project log |
| D4 | Expensive findings/gotchas route to **TROUBLESHOOTING.md or issues**, not the log | Different retrieval pattern: looked up by symptom, not by date |
| D5 | Pre-merge cleanup = **verify + finalize, compress, smart archive overflow** | The "no follow-up edits" guarantee comes from validating at merge time against the final diff |
| D6 | **Three log altitudes** anchored to existing artifacts (lab / project / plan-execution) | Plan-execution material is what bloated the big logs; the plan doc already has an archival lifecycle |
| D7 | PR-process deliverable: **end-to-end lifecycle doc + codified merge bar + codified solo-maintainer bypass** | The pieces exist in four places (§1), one unwritten |
| D8 | Doc-audience scope: **tier definitions + routing, per-tier writing standards, context budgets, single-source + derived views** | Single-source initially deferred, brought back in scope 2026-06-10 — it is the anti-drift rule the other three don't provide |
| D9 | Packaging: **lean enforceable rules in `.claude/rules/`, narrative in root docs, plus adherence Actions** | Rules files load into every session and every PR review; narrative is read once at onboarding |
| D10 | Work-artifact bundling: **proportionality ladder + work-bundle folders archived as a unit** | Generalizes mission-control's proven `completed/` pattern; co-locates each initiative's spec + plan + execution log |
| D11 | *(review round)* Entry headers carry **HH:MM**; per-entry budget is **bytes, not lines**; cap overflow is a **warning + dedicated chore PR**, not a merge blocker | Date-only headers can't order same-day concurrent PRs; line budgets are gamed by not wrapping; overflow-at-merge-bar forces unrelated PRs to do log maintenance, violating single-concern |
| D12 | *(review round)* Rules numbering convention: **lab-rules owns `0x-*`, per-repo rules use `10+`** | LSCA already has a repo-local `03-python-style.md`; collisions confuse the concatenated review context and the doc-update-trigger pairing |

## 4. Project-log standard

### 4.1 Altitudes

| Altitude | Anchor | Contents |
|---|---|---|
| **Lab** | `<DEV_ROOT>/project_log.md` | Cross-repo decisions: tooling, infra, conventions, entity/lab formation |
| **Project** | `<repo>/project_log.md` | Decisions that outlive any one plan; irreversible/external events; direction changes / re-scopes |
| **Plan-execution** | `## Execution Log` section inside the plan doc | Deviations from plan, implementation-altitude calls, gate evidence (the verification output that proved a task done). Archives with the plan (§5) |

**Altitude test:** *will this still matter after the plan ships?* Yes → project log. Only explains
how the plan was executed → plan-execution log. Crosses repos → lab log.

**Lab-altitude caveat:** `<DEV_ROOT>` is not a git repo, so the lab log has no PR flow, no merge
event, and no CI. There, the standard is honor-system, enforced by the rules file loaded into
every session: immutability begins once a newer entry exists; Refs use absolute doc paths / full
URLs instead of PR numbers; the archive step triggers whenever an entry is added while the file
exceeds its cap.

### 4.2 Entry triggers and routing table

A project-log entry is written when one of these occurs:

1. A **load-bearing decision** — a call with real alternatives whose reversal would change the
   project's direction or architecture.
2. An **irreversible or external event** — release cut, migration applied, secret rotated,
   org/repo change, data published.
3. A **direction change / re-scope** — pivot, pause, reactivation, supersession of a spec or
   plan. (Pausing or retiring a whole project additionally gets a status banner at the top of
   the repo README: "Status: paused YYYY-MM-DD — see lab log.")

Everything else routes elsewhere:

| Information | Home |
|---|---|
| Deviation from an approved plan | Plan doc `## Execution Log` |
| Expensive finding / gotcha | `TROUBLESHOOTING.md` or a GitHub issue |
| Open work, follow-ups, review findings | GitHub issues (a finding that itself meets a trigger above *additionally* gets a log entry) |
| Bare status ("merged, smoke passed") | PR comment |
| Session narrative / what-I-did | PR body |
| Long-lived facts about people/preferences | Auto-memory |

*Amended 2026-06-12: a plan-execution log closes with the PR that ships it — post-merge evidence (deploy green, runtime checks, branch cleanup) routes to a comment on that PR, never a trailing entry held for a future PR. The merge of a plan's final PR can't be recorded by that PR; without a closure rule, closeout facts hitch onto whatever unrelated PR comes next.*

### 4.3 Entry format

```markdown
## YYYY-MM-DD HH:MM — <subject: the decision or event, one line>

**Decision:** <what was decided / what happened>
**Why:** <the load-bearing rationale>
**Alternatives:** <only when real ones were weighed — what was rejected and why>
**Supersedes:** <YYYY-MM-DD HH:MM — subject>   <!-- only on superseding entries -->
**Refs:** #<PR>, <absolute doc paths or full URLs>
```

- **Budget: ≤ 1,500 bytes per entry** (byte-measured — line counts are gamed by not wrapping).
  Detail beyond that belongs in the PR body or the spec.
- **Count-free phrasing** (no numeric counts in narrative that restale on the next change).
- **PR number is the durable ref.** Never record a squash SHA. (Lab altitude: paths/URLs, §4.1.)
- **No `Status:` field.** Currency lives in the Standing Decisions index, not in entries.

### 4.4 Immutability and supersession

An entry is **immutable once its PR merges**. A reversed or revised decision gets a **new
superseding entry** carrying a `Supersedes:` line; the old entry is never edited. The same PR that
adds the superseding entry removes the superseded line from the Standing Decisions index; the
history keeps both entries. This is the structural fix for follow-up edits: there is nothing in a
merged entry that can claim to be current, so nothing can go stale.

A factually wrong merged entry (e.g. a typo'd PR number) is fixed via a PR carrying the
`log-lint:override` label with the reason stated in the PR body (§9) — not by silent edit, and not
by an absurd supersession entry.

### 4.5 File structure

Top to bottom (the lab-rules **template is normative** for the exact skeleton; `log-lint` parses
the template's structure):

1. **Title + one-line pointer** to this standard (no repo-local conventions block — the
   lab-rules template is the single source).
2. **Standing Decisions index** — one line per still-binding decision:
   `- YYYY-MM-DD HH:MM — <subject> · #<PR-or-archive-link>`, where the date + subject **match the
   entry header verbatim** (this is the mechanical key that makes index ↔ entry consistency
   lintable). The index line is **created in the same PR as its decision entry**; events (§4.2
   trigger 2) don't get index lines. Superseded lines are removed in the same PR as the
   superseding entry. The index covers *all* still-binding decisions — hot window and archive
   alike — so it is the "what is still true" surface an agent reads first.
3. **Entries**, reverse-chronological, **top-insert**: all of a PR's new entries form one
   contiguous, internally date-ordered block at the head of the entries region. Order is
   non-strict descending (same-timestamp ties allowed). Each entry preceded by `---` on its own
   line then a blank line (stable git merge anchors; conflict resolution = keep both blocks,
   reordered by header timestamp).

### 4.6 Hot window and overflow

- The main log file is capped at **15 KB** (whole file, bytes — same measure `docs-budget` uses).
- When an entry lands while the file exceeds the cap, CI **warns** (not blocks — forcing an
  unrelated PR to do log maintenance would violate single-concern scoping). The overflow is then
  executed as a dedicated `chore: archive log overflow` PR: the **oldest** entries move to
  `project_log_archive.md`, **prepended as a block, internal order preserved, byte-identical
  modulo end-of-line normalization** (clones legitimately differ in line endings across
  platforms — `autocrlf`, editor defaults; the standard is platform-agnostic, and cross-platform
  gotchas like this are captured in the lab-rules `TROUBLESHOOTING.md` deliverable, §11). Index lines for
  still-binding archived decisions remain in the index, re-pointed at the archive.
- The archive is a grep target, never read whole, and is exempt from the size cap.

## 5. Work-artifact lifecycle (bundling + archival)

A **proportionality ladder** decides what documentation a unit of work gets:

| Size | Definition | Artifacts | Lifecycle |
|---|---|---|---|
| **Small task** | Single-PR fix/chore; no design choices worth recording | GitHub issue + PR (template). No docs. | Closed issue + merged PR are the archive |
| **Slice / feature** | Multi-task unit with a design; typically one plan | **Work bundle**: `docs/work/YYYY-MM-DD-<slug>/` containing `design.md` + `plan.md` (plan carries `## Execution Log`) | Bundle folder `git mv`'d to `docs/work/completed/` when the owner declares the slice done — as a rider on the closing PR or as its own `chore:` PR (finality is hindsight; don't gate archival on guessing the "final" PR) |
| **Project / PRD** | Multi-slice program of work | PRD at a stable path (living doc, project altitude) + one work bundle per slice | PRD updates by amendment, never archives; slices archive individually |

- The **work bundle archives as one unit** — spec, plan, and execution log travel together, so a
  future "why was this built this way?" lands on the complete record in one folder.
- `docs/work/` top level is therefore always "what's in flight."
- **Abandoned slices** move to `docs/work/abandoned/` with a one-line reason prepended to the
  bundle's `design.md` — they never silt up the in-flight view.
- **Post-archival follow-ups** (the hotfix after the "done" declaration): completed bundles are
  not immutable — a follow-up may append to the bundle's Execution Log in `completed/`. Entry
  immutability (§4.4) applies to project-log entries, not bundles.
- **Cross-repo initiatives**: the bundle lives in the repo where the majority of its PRs land,
  with a lab-log entry pointing at it.
- Existing layouts (mission-control `docs/superpowers/{specs,plans}/`, Global_Pathways
  `documentation/engineering/planning/`) migrate **lazily**: new work uses the new shape; old
  artifacts stay put.

## 6. Single-source + derived views

- Every fact has exactly **one owning doc**. Any other surface that states it must link to or
  visibly derive from the owner, naming it explicitly (e.g. *"source of truth:
  `functional-spec.md` §3"*).
  **Public→private references (edge case):** a public-tier doc must never rely on access to a  private source for required context. public-tier docs should use generalized language to restate the relevant context in a way that is not fragile changes in private documents.
- **Derived public-tier docs carry a reconciliation trigger**: re-verified against their owning
  source before stakeholder-facing events (check-ins, releases, outreach). This codifies the
  practice Global_Pathways already applies to `PROJECT_OVERVIEW.md` informally.
- Review enforcement question: *does this PR restate a fact some other doc owns?* If yes, replace
  the restatement with a link or a sourced derivation.
- The rule is one paragraph in `03-logging-and-docs.md`; this section is its rationale.

## 7. Documentation tiers and context budgets

### 7.1 Tiers, defined by reader

| Tier | Reader | Surfaces | Writing standard |
|---|---|---|---|
| **AI** | Agents | Always-loaded: `CLAUDE.md`, `.claude/rules/`. First-read: `project_log.md` head. Grep-only: log archive, `TROUBLESHOOTING.md` | Dense, deterministic, count-free, zero narrative. Always-loaded surfaces are token-budgeted; grep-only surfaces are exempt (looked up by symptom/date, never read whole) |
| **ENG** | Lab members working with AI | PRDs, specs, plans, TRD/ADD, runbooks | Skimmable; stable heading anchors; explicit contracts; code-free plans per the existing lab rule |
| **Public** | Non-technical stakeholders and consumers | Roadmaps, overviews, update feeds, outreach | Jargon-free; no internal codenames; overclaim-scrubbed; derived-view sourcing per §6 |

### 7.2 Context budgets (always-loaded + first-read AI surfaces)

| Surface | Budget | Over-budget action |
|---|---|---|
| Per-repo `CLAUDE.md` | 8 KB | CI warning → compress or move detail to ENG tier |
| Each `.claude/rules/*.md` | 5 KB | CI warning → split narrative out to root docs |
| `project_log.md` (whole file) | 15 KB | Overflow chore PR (§4.6); CI warns |

Budgets warn at 1.0× and fail at 1.5× — **but `docs-budget` runs warn-only in any repo until that
repo first passes green** (§12: today GP's CLAUDE.md is ~12.5 KB = 1.53×, LSCA ~11.4 KB, MC
~9.4 KB; the standard must not red its own flagship repos on day one). The numbers are starting
calibrations, revisable by a logged lab-level decision.

## 8. PR lifecycle, merge bar, solo-maintainer bypass

### 8.1 Lifecycle (narrated in `PR-LIFECYCLE.md`)

branch → PR from repo template → automated review (pr-review-agent cron 2×/day and/or
`pr-review-loop` driven to the merge bar) → remediation rounds → **merge bar** → merge mechanics
(squash; bundle archival rides per §5; branch delete).

### 8.2 Merge bar (hard rule, added to `01-workflow.md`)

All of the following, verified at merge time:

1. **Gate green, run unpiped.** The gate is the repo's designated verification command per its
   `CLAUDE.md` (e.g. `invoke check`); "unpiped" because piping (`gate | tail`) swallows the exit
   code and lets a red gate look green. Repos with no gate (docs-only): the PR template's
   Verification section is the gate.
2. PR template complete; checklist items ticked only where true.
3. Review findings resolved, or routed to issues (a finding meeting a §4.2 trigger additionally
   gets a log entry).
4. **Log cleanup done:** this PR's loggable events have entries; entries verified against the
   final diff; compressed to budget; refs filled (PR#); no edits to pre-existing entries;
   **Standing Decisions index updated** — line added per new standing decision, superseded lines
   removed; if the log exceeds its cap, the overflow chore PR is filed (§4.6).
5. Doc-sync triggers checked (CLAUDE.md / STANDARDS / READMEs per existing rule).
6. Single concern; bundle archival rides only when the owner declares the slice done (§5).

### 8.3 Solo-maintainer bypass

When the required peer review is impossible (sole maintainer; self-approval blocked): an
**independent multi-agent review** — first pass + audit pass per lab model defaults — must be
completed and **posted to the PR** before admin bypass. The merge note references the posted
review. This codifies the practice from Global_Pathways #139–#142.

## 9. Adherence Actions

Reusable `workflow_call` workflows + scripts living in lab-rules (consumed the same way the
pr-review Action checks out lab-rules alongside the PR repo); each lab repo adds a thin caller
YAML. **lab-rules adds its own caller in phase 1** so the Actions run on at least one repo from
day one.

| Action | Checks | Mode |
|---|---|---|
| **`log-lint`** | Entry header regex; contiguous new-entry block at head of entries region, non-strict descending order; per-entry byte budget; **immutability** (algorithm below); index-line format + index↔entry key match | Fail on violation; `log-lint:override` label (reason required in PR body) bypasses for migrations and merged-entry corrections |
| **`docs-budget`** | Byte sizes of `CLAUDE.md`, `.claude/rules/*.md`, `project_log.md` vs §7.2 budgets | Warn at 1.0×, fail at 1.5×; warn-only per repo until first green (§7.2) |
| **`merge-bar-check`** | PR body contains the template's required sections; exactly one of the two log checkboxes ticked ("log entries finalized" / "no loggable events"); PRs matching the caller-supplied code-path globs (default: everything except `*.md`, `docs/**`, `.github/**`) must tick one or fail | Fail on missing sections or missing tick |

**`log-lint` immutability algorithm** (specified here because naive diffing false-positives on
the design's own mandated operations): parse both versions of `project_log.md` into entry sets
keyed by `## <timestamp> — <subject>` header; **baseline = current target-branch HEAD**, not the
merge-base (three-dot diffs misread rebases after the §4.5 conflict-resolution flow as foreign
additions). Pre-existing entries must survive unchanged; an entry deleted from the main log must
reappear in `project_log_archive.md` **byte-identical modulo EOL normalization**; the Standing
Decisions index region is exempt from immutability (supersession edits it by design).

**Mechanical vs semantic split:** Actions verify only what a parser or byte-count can prove.
Semantic enforcement — *was a decision actually logged? is a restated fact owned elsewhere? is an
entry honest?* — stays with the pr-review agent, which already loads
`lab-rules/.claude/rules/*.md` (including the new `03-logging-and-docs.md`) into its review
prompt.

## 10. ENG-tier document standards (PRD, design, plan)

The work-bundle artifacts (§5) and the PRD are document types this standard governs; each gets
a definition so templates and reviews have something concrete to check against.

- **PRD** — living doc at a stable path; project altitude; updated by amendment. Required
  elements: Problem · Success criteria (measurable) · Scope (in / explicitly out) ·
  Constraints · Plan (phased) · Open questions. A PRD does **not** embed its own decision log —
  `project_log.md` owns decisions, one project-altitude surface per repo (this is why
  mission-control's PRD Part IV becomes a pointer, §12).
- **Design doc** (`design.md` in a work bundle) — the spec for one slice: the problem, the
  decisions made with rationale and rejected alternatives, and known gaps stated honestly.
  Status line at the top (draft / reviewed / superseded-by).
- **Plan** (`plan.md` in a work bundle) — code-free per the existing lab rule: per task,
  Files / Depends on / Spec link / Acceptance / Verification / Commit; no literal code; the
  only code blocks are short shell commands in Verification lines. Carries the
  `## Execution Log` section (§4.1). Canonical exemplar: the Global_Pathways IAS backend plan.
- **Templates make compliance copyable** — each governed doc type has a skeleton in
  `templates/` (§11) that satisfies its own rules as created.

## 11. Deliverables map

| Path | Action |
|---|---|
| `.claude/rules/01-workflow.md` | Extend: merge bar (§8.2) incl. pre-merge log cleanup |
| `.claude/rules/03-logging-and-docs.md` | Create: entry triggers + routing table, altitudes, entry format, supersession, budgets, tier routing, single-source rule, **rules numbering convention (lab `0x`, repo `10+`, D12)**. Target ≤ 5 KB (its own budget) |
| `PR-LIFECYCLE.md` | Create: lifecycle narrative consolidating the four sources named in §1; bypass pattern; overflow/archive mechanics; rationale |
| `TROUBLESHOOTING.md` (lab-rules) | Create: grep-only; seeded with known cross-platform gotchas (line endings / `autocrlf`, junction vs symlink, path separators, PowerShell vs POSIX quoting) — setup steps stay in BOOTSTRAP.md, this holds the failure modes |
| `templates/project_log.template.md` | Create: §4.5 skeleton (normative — `log-lint` parses this structure) |
| `templates/repo-CLAUDE.template.md` | Create: per-repo CLAUDE.md seed, structured to its 8 KB budget (§7.2) |
| `templates/work-bundle/` + `templates/PRD.template.md` | Create: design/plan skeletons (plan carries `## Execution Log`) + PRD skeleton, per the §10 document standards |
| `.github/workflows/log-lint.yml` + script | Create (§9) |
| `.github/workflows/docs-budget.yml` + script | Create (§9) |
| `.github/workflows/merge-bar-check.yml` + script | Create (§9) |
| `.github/workflows/` lab-rules self-caller | Create: lab-rules consumes its own three Actions from day one |
| `.github/pull_request_template.md` | Extend the existing checklist (don't duplicate the "Docs updated" item) with two **separate** checkboxes: `[ ] Log entries finalized (verified against final diff, index updated)` / `[ ] No loggable events in this PR`, plus `[ ] Work-bundle archival included (slice declared done)` where applicable |
| `WORKING-WITH-CLAUDE.md` | §5: trim only the bypass + PR-template bullets to pointers (PR-LIFECYCLE.md / 01-workflow.md); **retain** the audit-pass, outsider's-eye, and review-is-the-deliverable bullets. §8: replace log-location prose with a pointer to `03-logging-and-docs.md`; **retain** checkpoint triggers and auto-memory guidance |
| `README.md` (lab-rules) | Update "What's here" |

## 12. Rollout (phase 2, separate effort)

- Per-repo caller YAMLs for the three Actions; **each caller PR rides with (or is preceded by)
  that repo's CLAUDE.md compression** so `docs-budget` can reach first-green (GP is past the
  1.5× line today, §7.2).
- Migrate Global_Pathways and mission-control logs: archive current content wholesale (under
  `log-lint:override`), distill Standing Decisions indexes, adopt the template head.
- mission-control PRD Part IV becomes a pointer to `project_log.md` (one project-altitude
  surface per repo).
- **LSCA rules cleanup:** delete its stale forked `01-workflow.md` / `02-data-protection.md`
  (verified drifted — e.g. missing the `ci` commit prefix; lab versions reach sessions via the
  dev-root junction and reach CI review via the lab-rules checkout), and renumber
  `03-python-style.md` → `10-python-style.md` per D12.
- LSCA log: reorder to reverse-chron, add index (small enough to do inline).

## 13. Known gaps / open questions

- **Budget calibration** (8/5/15 KB, 1.5 KB per entry) is a first guess; revisit after a month
  of CI data.
- **Lab-altitude log is honor-system** (§4.1): no git, no CI, no merge events at `<DEV_ROOT>`.
  Accepted; revisit if it drifts — the fallback is relocating the lab log into lab-rules where
  the machinery applies.
- **Standing Decisions index growth**: index lines are never archived while binding; if the
  index alone approaches the file cap, that's a signal the project has too many standing
  decisions in one repo — handle by then-current judgment, not a rule.
- **`merge-bar-check` honesty**: a ticked checkbox proves intent, not truth; the semantic check
  remains with the pr-review agent. Accepted residual risk.
- **TROUBLESHOOTING.md is unbudgeted by design** (grep-only tier, §7.1); watch whether it needs
  its own overflow convention in practice.
- **Lab-wide issue/label conventions are unstandardized**: the §4.2 routing table leans on
  GitHub issues as a primary home, but each repo's label/milestone scheme differs
  (mission-control's milestone + `area:` axes are the most developed). Phase-2+ candidate;
  mission-control's scheme is the likely seed.
- **Repo initialization / retirement runbook spun out**: a new-repo setup checklist
  (`REPO-SETUP.md`) was drafted into this spec and cut as off-topic — it's lab operations, not
  a logging/documentation standard. Worth its own small effort; the templates (§11) are its
  building blocks, and the pause/retirement banner convention survives in §4.2.
